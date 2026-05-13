---
date: 2026-05-13
type: project
status: planning
tags: [project, lithify, process, scheduling, multi-agent]
ai-first: true
---

## For future Claude
lithify 子模块设计文档——进程与调度。描述 agent 实例的生命周期管理（类比进程表、fork/exec/wait）、多 agent 调度策略（token budget 作为 CPU 时间片），以及 NUMA 类比对多 agent 协调的设计含义。

---

## 问题陈述

当前 agent 框架没有统一的 agent 生命周期管理机制：subagent 如何创建、如何传递结果、如何正常/异常终止，缺乏标准。多 agent 并行时的调度（谁先跑、资源怎么分）完全由应用层自行处理。

---

## OS 类比

| OS 概念 | Agent 对应 |
|---|---|
| 进程（Process） | 单次 Agent 调用实例 |
| 进程表（Process Table） | Harness 维护的 agent 注册表 |
| `fork()` | 创建 subagent（继承部分 context） |
| `exec()` | 切换 agent 类型（加载不同 skill） |
| `wait()` | 等待 subagent 完成并收取结果 |
| `exit()` | Agent 正常完成，返回结果 |
| PID | Agent Instance ID |
| 进程树 | Orchestrator → Subagent 调用树 |
| CPU 时间片 | Token budget |
| NUMA 节点 | 多 agent 的本地 context |

---

## 设计方案

### 1. Agent 进程表

Harness Ring 0 维护全局 agent 注册表，每条记录包含：

```
AgentEntry {
  id: UUID                    # 唯一标识
  parent_id: UUID | null      # 父 agent（orchestrator）
  skill: string               # 当前执行的 skill
  ring: 0 | 1 | 3            # 特权级别
  status: running | waiting | done | error | zombie
  token_budget: int           # 剩余 token 配额
  token_used: int             # 已消耗 token
  context_slice: ContextRef   # 虚拟 context 引用（见模块 10）
  created_at: timestamp
  signals: Signal[]           # 待处理信号队列
}
```

### 2. Agent 生命周期

```
                  ┌─────────────────┐
                  │   Orchestrator  │
                  └────────┬────────┘
                           │ spawn(skill, context_slice, budget)
                  ┌────────▼────────┐
                  │    CREATED      │
                  └────────┬────────┘
                           │ harness 分配资源
                  ┌────────▼────────┐
             ┌───►│    RUNNING      │◄────┐
             │    └────────┬────────┘     │
    SIGCONT   │             │ tool call    │ tool result
             │    ┌────────▼────────┐     │
             └────│    WAITING      │─────┘
                  └────────┬────────┘
                     SIGTERM│ / task done
                  ┌────────▼────────┐
                  │  TERMINATING    │  # 优雅收尾
                  └────────┬────────┘
                           │ return result
                  ┌────────▼────────┐
                  │     DONE        │  # 父 agent wait() 收取
                  └─────────────────┘
```

**Zombie 状态**：agent 完成但父 agent 尚未收取结果。Harness 应有 zombie 清理机制，防止资源泄漏。

### 3. Token Budget 作为时间片

类比 CPU 时间片调度：

- 每个 agent 实例分配 token_budget（由父 agent 或 harness 策略决定）
- Budget 耗尽时 agent 必须"yield"——返回中间结果，等待父 agent 决定是否续期
- Harness 可以实现优先级调度：高优先级 agent 获得更大 budget 配额
- 类比 Linux CFS（完全公平调度）：可按 agent 类型和紧急程度加权分配

### 4. NUMA：多 Agent 协调

单 agent = 单核 CPU，多 agent = NUMA 多核架构：

- 每个 agent 有本地 context（本地内存，访问快）
- 跨 agent 共享状态 = 跨 NUMA 节点访问（慢、需同步）
- **多 agent 最难的问题（context coherency）= 多核最难的问题（缓存一致性）**

类比 MESI 缓存一致性协议，agent context 状态机：

| 状态 | 含义 |
|---|---|
| **M**odified | 本地已修改，其他 agent 不可读 |
| **E**xclusive | 本地独占，未修改 |
| **S**hared | 多个 agent 共享只读副本 |
| **I**nvalid | 缓存失效，需重新从 orchestrator 获取 |

---

## 开放问题

- [ ] Token budget 耗尽后的"续期"决策由谁做？Orchestrator 还是 Harness 自动续期？
- [ ] Zombie agent 的清理策略：等待超时？还是父 agent 必须显式调用 wait()？
- [ ] MESI 等价协议在 agent context 同步中的具体实现？消息传递还是共享内存？
- [ ] 如何处理 agent 调用树的循环依赖（deadlock）？
