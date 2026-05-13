---
date: 2026-05-13
type: project
status: planning
tags: [project, lithify, observability, debugging, monitoring]
ai-first: true
---

## For future Claude
lithify 子模块设计文档——可观测性。当前 agent 系统是完全黑箱，出问题无从调试。本文档设计 `/proc` 等价物、tool 调用追踪（strace）、资源监控（top），以及跨 agent 的分布式追踪。

---

## 问题陈述

agent 系统出问题时，工程师面对的是：
- 不知道某个 subagent 的 context 里有什么
- 不知道它在等哪个 tool 返回
- 不知道 token 消耗分布在哪里
- 不知道为什么它做了某个决定

这是当前 agent 工程实践中最痛的问题。**不可观测的系统不可调试，不可调试的系统不可信。**

---

## OS 类比

| OS 工具 | Agent 等价 | 用途 |
|---|---|---|
| `/proc/[pid]/` | Agent State API | 实时查看任意 agent 的内部状态 |
| `strace` | Tool Call Tracer | 追踪 agent 发出的所有 tool 调用 |
| `top` / `htop` | Agent Resource Monitor | 实时查看 token 消耗、活跃 agent |
| `perf` | Token Profiler | 分析 token 消耗热点 |
| `dmesg` / `journalctl` | Harness Event Log | 内核级事件日志 |
| `ltrace` | LLM Inference Tracer | 追踪每次 LLM 推理调用 |
| OpenTelemetry | Agent Distributed Trace | 跨 agent 的端到端追踪 |

---

## 设计方案

### 1. Agent State API（`/proc` 等价物）

每个运行中的 agent 暴露只读状态端点（由 Ring 0 Harness 提供）：

```
GET /agents/{agent_id}/state

Response:
{
  "id": "uuid",
  "skill": "code-review",
  "ring": 3,
  "status": "waiting",          # running | waiting | done
  "current_step": "tool_call",  # 当前在做什么
  "context": {
    "total_capacity": 200000,
    "used": 47832,
    "i_segment": 8192,
    "d_segment": 31640,
    "w_segment": 8000,
    "pressure": 0.24             # 0.0 ~ 1.0
  },
  "token_budget": {
    "total": 8192,
    "used": 3041,
    "remaining": 5151
  },
  "active_tool_calls": [
    {"tool": "web_search", "started_at": "...", "elapsed_ms": 1240}
  ],
  "parent_id": "uuid",
  "children": ["uuid", "uuid"]
}
```

### 2. Tool Call Tracer（strace 等价物）

记录 agent 的所有 tool 调用，格式类似 strace 输出：

```
[14:23:01.234] agent:abc123 → web_search(query="MCP protocol spec") [CALL]
[14:23:02.891] agent:abc123 ← web_search → {results: [...]} [OK, 1657ms, 2341 chars]
[14:23:02.901] agent:abc123 → read_file(path="/vault/index.md") [CALL]
[14:23:02.912] agent:abc123 ← read_file → {content: "..."} [OK, 11ms, 4832 chars]
[14:23:05.012] agent:abc123 → bash(cmd="pytest tests/") [CALL, SANDBOXED]
[14:23:18.440] agent:abc123 ← bash → {exit_code: 1, stderr: "..."} [ERR, 13428ms]
```

可以按 agent_id、tool 类型、时间范围过滤。

### 3. Agent Resource Monitor（top 等价物）

实时显示所有活跃 agent 的资源消耗：

```
lithify Monitor - 2026-05-13 14:23:19
Active: 4 agents | Total tokens/min: 12,847

 AGENT_ID  SKILL           RING  STATUS   CTX_USED  BUDGET  TOK/MIN
 abc123    code-review       3   waiting   47832    5151    2,341
 def456    obsidian-save     3   running   12041    7800      891
 ghi789    web-search        3   running   8912     6200    4,203
 jkl012    orchestrator      1   waiting   89123    ∞       5,412
```

### 4. 分布式追踪（OpenTelemetry 等价物）

跨 agent 调用链的端到端追踪，每个 agent 调用附带 trace_id 和 span_id：

```
Trace: task_abc [user: "review my PR"]
  │
  ├─ Span: orchestrator.plan [12ms]
  │
  ├─ Span: agent.code-review [4.2s]
  │    ├─ Span: tool.read_file [11ms]
  │    ├─ Span: llm.inference [2.1s, 3041 tokens]
  │    └─ Span: tool.write_comment [45ms]
  │
  └─ Span: agent.summarize [1.1s]
       └─ Span: llm.inference [1.1s, 891 tokens]

Total: 5.4s | 3932 tokens | 2 subagents
```

trace_id 在整个调用树中传递，出问题时可以精确定位到哪个 span 出了什么问题。

---

## 数据访问权限

| 数据 | 可访问方 | 理由 |
|---|---|---|
| 自己的 state | Agent 本身 + 父 Agent + Ring 0 | 需要了解自身状态 |
| 其他 agent 的 state | 仅 Ring 0 + Ring 1 | 隔离原则 |
| Tool call traces | Ring 0 + 所属 agent | 调试需要 |
| 全局 resource monitor | Ring 1 Orchestrator | 调度决策需要 |
| 分布式 trace | Ring 0 统一收集，Ring 1 可查询 | 追踪属于基础设施 |

---

## 开放问题

- [ ] Agent state API 的查询是否会影响 agent 的 token 消耗？如何设计为零开销？
- [ ] Tool call trace 的存储：内存中 ring buffer？还是持久化到 vault？
- [ ] 分布式 trace 的采样策略：全量追踪 vs 采样（高延迟/高消耗的请求全量追踪）？
- [ ] 如何对 LLM 推理本身进行追踪（attention 分布、token 概率）而不增加过多开销？
