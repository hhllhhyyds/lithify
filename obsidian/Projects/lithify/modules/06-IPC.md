---
date: 2026-05-13
type: project
status: planning
tags: [project, lithify, ipc, multi-agent, communication]
ai-first: true
---

## For future Claude
lithify 子模块设计文档——进程间通信（IPC）。当前多 agent 通信只能通过父 context 中转（"黑板模式"），相当于 OS 没有 pipe/socket，所有进程只能通过读写同一块共享内存通信。本文档设计 agent IPC 的基础原语。

---

## 问题陈述

当前多 agent 通信的唯一方式：subagent 返回结果 → 写入父 context → 父 agent 读取。

问题：
- 所有通信经过父 context 中转，形成瓶颈
- 无法支持 peer-to-peer agent 通信
- 无法支持 pub/sub 事件通知
- 无法支持流式结果（只能等 agent 完全结束）

---

## OS 类比

| OS IPC 原语 | Agent 等价 | 用途 |
|---|---|---|
| Pipe（单向） | Agent Result Stream | subagent 向父 agent 流式传输结果 |
| Message Queue | Agent Task Queue | 异步任务分发 |
| Shared Memory（只读） | Shared Context Segment | 多 agent 共享只读知识 |
| Unix Socket | Agent Direct Channel | peer-to-peer agent 通信 |
| Pub/Sub（类 D-Bus） | Agent Event Bus | 事件广播（任务完成、状态变更） |
| Semaphore | Context Lock | 防止多 agent 同时写同一 context 片段 |

---

## 设计方案

### 1. 四种基础 IPC 原语

**① Result Stream（类 Pipe）**

subagent 生成结果时流式推送给父 agent，不等待完成：

```
subagent → [Result Stream] → parent agent
```

- 支持父 agent 在 subagent 未完成时开始处理部分结果
- 对应中断驱动模型（见 [[02-Peripheral-MCP]]）

**② Task Queue（类 Message Queue）**

Orchestrator 将子任务放入队列，空闲 agent 从队列拉取：

```
orchestrator → [Task Queue] → worker agent 1
                            → worker agent 2
                            → worker agent N
```

- 天然支持负载均衡
- 任务可附带优先级

**③ Shared Context Segment（类 Shared Memory，只读）**

多个 agent 共享同一块只读 context 片段（如：项目背景、用户偏好）：

```
shared_segment = harness.create_shared("project_context", content)
agent_A.mount(shared_segment, mode=READ_ONLY)
agent_B.mount(shared_segment, mode=READ_ONLY)
```

- 减少重复内容占用各 agent 的 context budget
- 只读保证不会有写竞争（无需锁）
- 修改共享 segment 需要经过 Ring 0 仲裁

**④ Event Bus（类 Pub/Sub）**

agent 可以发布/订阅系统级事件：

```
# 订阅
harness.subscribe(agent_id, event="task.completed", filter={"skill": "code-review"})

# 发布（由 Harness 在任务完成时自动发布）
harness.publish(event="task.completed", payload={...})
```

常见事件类型：
- `task.completed` / `task.failed`
- `context.pressure.high`（context 快满了）
- `skill.updated`（某个 skill 有新版本）
- `signal.received`（收到外部信号，见 [[07-Signal-System]]）

### 2. 消息格式

所有 IPC 消息遵循统一格式：

```json
{
  "msg_id": "uuid",
  "from": "agent_id",
  "to": "agent_id | queue_name | broadcast",
  "type": "result | task | event | signal",
  "trust_level": "ring0 | ring1 | ring3",
  "payload": { ... },
  "timestamp": "ISO8601",
  "ttl": 60
}
```

`trust_level` 由发送方的特权级别决定，接收方可以据此决定处理优先级和验证强度。

### 3. Context Lock（类 Semaphore）

防止多个 agent 并发写同一 context 片段：

```
with harness.context_lock(segment_id, timeout=5.0):
    harness.write_context(segment_id, content)
```

- 锁由 Ring 0 Harness 统一管理，不暴露给 Ring 3 agent
- 死锁检测：持锁超时自动释放，触发 `deadlock.detected` 事件

---

## 开放问题

- [ ] Shared Context Segment 的一致性协议：写入时如何通知所有挂载者？
- [ ] Event Bus 的持久化：agent 离线期间发生的事件如何补收？
- [ ] 跨 session 的 IPC：两个不同 session 的 agent 能否通信？需要什么授权？
- [ ] Task Queue 的公平调度：如何防止高优先级任务饿死低优先级任务？
