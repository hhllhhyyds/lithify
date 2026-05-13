---
date: 2026-05-13
type: project
status: planning
tags: [project, lithify, memory, context, rag]
ai-first: true
---

## For future Claude
lithify 子模块设计文档——存储层次。描述 LLM context window、skills、RAG 与 OS 存储层次的映射关系，并提出基于此类比的具体工程设计方案（context 分配策略、eviction policy、I/D cache 分离、预取机制）。

---

## 问题陈述

LLM context window 是整个 agent 系统中最稀缺的资源——有限、昂贵、直接决定输出质量。当前 agent 框架对 context 的管理极其粗糙：要么手动堆 prompt，要么到上限就截断。没有任何框架认真对待"什么应该在 context 里"这个问题。

---

## OS 类比

| OS 存储层次 | Agent 对应 | 特征 |
|---|---|---|
| CPU 寄存器 | LLM context window | 最快、最贵、容量 ≤ 200K token |
| L1/L2/L3 Cache | System prompt + 已加载 Skills | 预热、低延迟、session 级持久 |
| RAM | Skill 库（按需换入） | 结构化、可寻址、换入有延迟 |
| 磁盘 | Knowledge base + RAG | 容量无限、检索延迟高 |

**LLM = CPU**：所有推理在此发生，只能直接访问"寄存器"（context）中的内容。

---

## 核心问题：寄存器压力 = Context Pressure

编译器中寄存器分配是 NP-hard 问题——变量太多就要 spill 到内存，性能骤降。Agent 面临完全一样的问题：

- 当前应对方式：FIFO 截断（丢最老的内容）
- FIFO 是最差的缓存策略之一
- **Agent harness 的核心工程问题本质上是一个编译器寄存器分配问题**

---

## 设计方案

### 1. 分层 Context 管理器

```
┌─────────────────────────────────────┐
│         LLM Context Window          │  ← 寄存器（直接可用）
│  [I-segment] 行为规则 + 活跃 Skills  │
│  [D-segment] 当前任务数据 + 工具结果 │
│  [W-segment] 工作记忆（当前对话）    │
└─────────────────────────────────────┘
         ↑ load/evict          ↑ retrieve
┌────────┴──────────┐  ┌───────┴────────────┐
│   Skill Cache     │  │   RAG / KB         │
│  （已解析 Skills） │  │  （向量数据库等）   │
└───────────────────┘  └────────────────────┘
```

**I-segment（指令段）**：行为规则、系统约束、已激活的 skill 定义。对应 CPU I-cache，内容稳定，跨对话复用。

**D-segment（数据段）**：当前任务相关的背景知识、工具返回结果。对应 CPU D-cache，任务切换时可换出。

**W-segment（工作段）**：当前对话历史、进行中的推理。最新、最热，优先保留。

### 2. 智能 Eviction Policy

当 context 接近上限时，按以下优先级驱逐（从低到高保留）：

| 优先级（保留） | 内容类型 | 类比 |
|---|---|---|
| 最高 | 当前任务的直接输入和约束 | 寄存器 pinned |
| 高 | 最近 N 轮对话 | Working set |
| 中 | 本次任务相关的工具结果 | Hot cache line |
| 低 | 早期对话摘要 | Cold cache line |
| 最低 | 通用背景知识（可 RAG 回取） | Evictable |

**关键原则**：可以从 RAG 重新检索的内容应优先驱逐；不可再生的内容（用户原始输入、决策上下文）不可驱逐。

### 3. Pre-fetching（预取）

OS 根据访问模式预取数据。Agent 等价：

- 分析当前任务意图，预判需要哪些 skills
- 在任务开始前主动加载相关 skills 和知识片段
- 减少任务执行中的"cache miss"（临时检索导致的中断）

### 4. I-cache / D-cache 分离

当前所有 agent 框架把行为规则和背景知识混在 system prompt 里——架构污染。

正确做法：
- **I-segment** 在 session 初始化时一次性加载，跨对话不变
- **D-segment** 随任务动态换入换出
- 两段有独立的 eviction 策略和生命周期管理

---

## 开放问题

- [ ] Eviction 策略是否可以通过强化学习优化？以任务完成率为奖励信号
- [ ] I-segment 和 D-segment 的最优比例是多少？是否与任务类型相关？
- [ ] Pre-fetching 的意图识别如何实现？基于 embedding 相似度还是显式任务规划？
- [ ] Context 压缩（lossy）vs 驱逐（lossless retrieval）的权衡如何量化？
