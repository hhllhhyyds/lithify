---
date: 2026-05-13
type: project
status: planning
tags: [project, lithify, signal, interrupt, safety]
ai-first: true
---

## For future Claude
lithify 子模块设计文档——信号系统。当前 agent 无优雅中止协议，失控 agent 只能被强行截断。本文档设计类 Unix 信号的 agent 中止与状态控制机制，这是安全关键（P0）的缺失模块。

---

## 问题陈述

当前 agent 系统没有异步控制机制：
- 无法优雅地要求一个 running agent "收尾后退出"
- 无法区分"立即停止"和"完成当前步骤后停止"
- 无法向 agent 发送状态变更通知（如"优先级提升了"）
- 失控 agent（无限循环、token 耗尽）只能靠强行截断，可能丢失中间结果

---

## OS 类比

| Unix 信号 | Agent 信号 | 语义 |
|---|---|---|
| `SIGTERM` | `AGENT_TERM` | 请求优雅退出，agent 可以收尾 |
| `SIGKILL` | `AGENT_KILL` | 立即强制终止，不可被忽略 |
| `SIGINT` | `AGENT_INT` | 用户中断（Ctrl+C），建议停止 |
| `SIGPAUSE` | `AGENT_PAUSE` | 暂停执行，保存检查点 |
| `SIGCONT` | `AGENT_CONT` | 恢复被暂停的 agent |
| `SIGUSR1/2` | `AGENT_CHECKPOINT` | 触发中间结果保存 |
| `SIGALRM` | `AGENT_TIMEOUT` | Token budget / 时间超时通知 |

---

## 设计方案

### 1. 信号优先级与可屏蔽性

```
不可屏蔽信号：
  AGENT_KILL      # 无论 agent 状态，立即终止
  AGENT_TIMEOUT   # budget 耗尽，由 Harness 自动发送

可屏蔽信号（agent 可声明临时屏蔽）：
  AGENT_TERM      # 正在写关键输出时可延迟处理
  AGENT_PAUSE     # 正在原子操作时可延迟

通知型信号（不中断执行）：
  AGENT_CHECKPOINT
  AGENT_INT
```

### 2. AGENT_TERM 处理流程（优雅退出）

```
Harness 发送 AGENT_TERM
    ↓
Agent 收到信号
    ↓
Agent 完成当前"原子步骤"（不可在 tool call 中途退出）
    ↓
Agent 生成"中间结果摘要"（保存进度，方便后续恢复）
    ↓
Agent 调用 exit(partial_result)
    ↓
Harness 收取结果，标记 agent 状态为 TERMINATED_GRACEFULLY
```

超时机制：发送 AGENT_TERM 后等待 N 秒，若 agent 未退出 → 自动升级为 AGENT_KILL。

### 3. AGENT_PAUSE / AGENT_CONT（检查点机制）

允许暂停 agent 并保存检查点，后续恢复执行：

```
harness.send_signal(agent_id, AGENT_PAUSE)
    ↓
agent 完成当前 token → 序列化状态（context 快照 + 执行位置）
    ↓
harness.save_checkpoint(agent_id, checkpoint)
    ↓
... 稍后 ...
    ↓
harness.send_signal(agent_id, AGENT_CONT, checkpoint_id)
    ↓
agent 从检查点恢复执行
```

应用场景：
- 用户临时切换任务，稍后回来继续
- 系统负载过高，暂停低优先级 agent
- Session 迁移（跨设备继续任务）

### 4. 信号传播规则

类比进程组信号（`kill -TERM -pgid`）：

- 向 Orchestrator 发送信号 → 默认传播到所有子 agent
- 子 agent 可以选择屏蔽来自父 agent 的非紧急信号
- AGENT_KILL 不可屏蔽，无论层级直接生效

```
orchestrator ──AGENT_TERM──► subagent_A (传播)
                           ► subagent_B (传播)
                           ► subagent_C (屏蔽中，延迟)
```

### 5. 与 Token Budget 的整合

Token budget 耗尽时 Harness 自动发送 AGENT_TIMEOUT：

```
budget_remaining < threshold → Harness 发送 AGENT_TIMEOUT（警告）
budget_remaining == 0       → Harness 发送 AGENT_KILL（强制）
```

agent 可以在收到 AGENT_TIMEOUT 后主动优化输出（截短、摘要），而不是被强制截断。

---

## 开放问题

- [ ] 检查点序列化的格式：只保存 context 快照，还是也保存执行状态（当前 skill 的位置）？
- [ ] 跨 session 的检查点恢复：如何处理 skill 版本变化导致的兼容性问题？
- [ ] 信号屏蔽的授权：Ring 3 agent 是否可以屏蔽任何信号？还是需要 Ring 0 授权？
- [ ] 多个并发信号的处理顺序：是否需要信号队列？优先级如何？
