---
date: 2026-05-13
type: project
status: planning
tags: [project, lithify, data-access, memory-model, vfs]
ai-first: true
---

## For future Claude
lithify 子模块设计文档——数据访问模型（DAM）。这是一个横切关注点：[[01-Memory-Hierarchy]] 定义了存储层次，[[10-Virtual-Context]] 定义了 context 内部的段结构，但两者之间缺少一个统一的访问接口——skill 读写数据时不知道数据在哪一层、写入时不知道持久化到哪里。本模块定义 skill 与存储层次交互的标准契约，类比 OS 的 VFS（虚拟文件系统）和 C 的存储类（storage class）。

---

## 问题陈述

当前 skill 与数据交互有三个未定义的问题：

**① 读：没有查找顺序规范**
skill 需要"项目背景"时，应先查 context（D-segment）命中就用，miss 了再触发 RAG 检索？还是每次都重新检索？没有规范，每个 skill 自行决定，行为不一致，性能不可预期。

**② 写：没有持久化语义**
skill 写出一个"决策"，它会落到哪里？W-segment（任务结束丢弃）？还是 vault（跨 session 持久）？目前 skill 想持久化必须显式调用 `write_file` tool，没有声明式的耐久度（durability）概念。

**③ 共享：agent 间数据流动没有规范**
agent A 产出的数据，agent B 想读，应通过父 context 传递（L0）、写成 shared segment（L0 共享区）还是写入 RAG（L3）？层次选择直接影响延迟、token 消耗和一致性，但目前没有任何规范。

---

## OS 类比

### 类比一：VFS（虚拟文件系统）

Linux VFS 让应用程序用统一的 `read()`/`write()` 接口操作任意存储后端（ext4、NFS、tmpfs、/proc），不需要感知底层实现。

Agent DAM 做同样的事：skill 用统一的 `data.read()` / `data.write()` 接口访问任意存储层，不需要感知数据在 context、skills cache 还是 RAG 里。

### 类比二：C 存储类（Storage Class）

| C 存储类 | 生命周期 | Agent 等价 | 存储层 |
|---|---|---|---|
| `register` | 表达式内 | 推理中间状态 | W-segment（L0） |
| `auto`（栈） | 函数/task 内 | 当前任务数据 | D-segment（L0） |
| `static` | 程序/session 内 | Session 级知识 | Skills cache（L1/L2） |
| `extern`/文件 | 跨 session | 持久化知识 | Vault / RAG（L3） |

skill 声明数据的"存储类"，harness 决定具体落到哪一层。

### 类比三：写回策略

| 缓存策略 | 语义 | Agent 对应 |
|---|---|---|
| Write-through | 写入立即同步到下层 | `durability: persistent` → 立即写 vault |
| Write-back | 写入先留在缓存，eviction 时刷回 | `durability: session` → session 结束时可选持久化 |
| Write-around | 绕过缓存直接写底层 | 大块知识直接写 RAG，不占 context |

---

## 设计方案

### 1. 数据访问接口（DAI）

所有 skill 通过以下统一接口访问数据，不直接操作存储层：

```python
# 读接口
data.read(
    key: str,
    scope: Scope,           # 查找范围
    fallback: bool = True   # miss 时是否自动向下层查找
) → value | None

# 写接口
data.write(
    key: str,
    value: Any,
    durability: Durability, # 持久化语义
    share: ShareMode        # 共享范围
) → void

# 查询接口（语义检索，触发 RAG）
data.search(
    query: str,
    scope: Scope,
    top_k: int = 5
) → [Result]
```

### 2. Scope（查找范围）

```
Scope.WORKING       只查 W-segment（最快，只有当前推理步骤的数据）
Scope.TASK          查 D-segment（当前任务数据 + tool 结果）
Scope.SESSION       查 D-segment + Skills cache（session 内所有数据）
Scope.VAULT         查 RAG / Knowledge base（触发向量检索）
Scope.AUTO          按层次顺序查：W → D → Cache → RAG（推荐默认值）
```

`Scope.AUTO` 的查找顺序（类比 CPU 缓存查找）：

```
query key
  → L0 W-segment         hit? → 返回（0 token 成本）
  → L0 D-segment         hit? → 返回（0 token 成本）
  → L1/L2 Skills cache   hit? → 加载到 D-segment → 返回（小成本）
  → L3 RAG 检索           → 加载到 D-segment → 返回（高成本）
  → 未找到 → 返回 None
```

### 3. Durability（持久化语义）

```
Durability.TRANSIENT    只存 W-segment，推理步骤结束即丢弃
Durability.TASK         存 D-segment，task 结束释放（默认）
Durability.SESSION      存 D-segment，session 结束时提示是否持久化
Durability.PERSISTENT   写穿到 vault（write-through），跨 session 有效
Durability.SHARED       写入 shared_bg segment，本 session 内跨 agent 共享
```

### 4. ShareMode（共享范围）

```
ShareMode.PRIVATE       只有本 agent 可读写（默认）
ShareMode.PARENT        父 agent 可读（通过 result 返回）
ShareMode.SESSION       本 session 所有 agent 可读（只读共享）
ShareMode.BROADCAST     通过 Event Bus 广播（见 [[06-IPC]]）
```

### 5. 数据放置策略（完整映射）

| 数据类型 | 推荐 Durability | 推荐 ShareMode | 落到哪一层 |
|---|---|---|---|
| 推理草稿、中间变量 | TRANSIENT | PRIVATE | W-segment |
| 当前任务输入 | TASK | PRIVATE | D-segment |
| Tool 返回结果 | TASK | PRIVATE | D-segment |
| 本次任务的决策 | SESSION | PARENT | D-segment → 父 context |
| 跨任务的结论 | PERSISTENT | PRIVATE | Vault |
| 需要所有子 agent 共享的背景 | SESSION | SESSION | shared_bg segment |
| 大块知识（文档、代码库） | PERSISTENT | SESSION | RAG（write-around）|

### 6. 成本模型

DAI 在查找时自动感知层次成本，帮助 skill 做出经济决策：

```
data.read(key, scope=AUTO, cost_budget=MEDIUM)

# 内部逻辑：
# - 如果 L0 命中 → 返回（成本：0）
# - 如果需要 RAG 检索 + context budget 已紧张 → 降级返回摘要（节省 token）
# - 如果 cost_budget=LOW → 只查 L0，不触发 RAG
```

| 操作 | Token 成本 | 延迟 |
|---|---|---|
| 读 W/D-segment（已在 context） | 0（已占用） | ~0ms |
| 从 Skills cache 加载到 context | 低（注入 tokens） | ~10ms |
| RAG 检索 + 注入 | 高（检索 + 注入） | ~200ms |
| 写 vault（write-through） | 0（tool 调用） | ~50ms |

---

## 与其他模块的关系

- **[[01-Memory-Hierarchy]]**：DAM 是模块 01 存储层次的访问接口层，是 VFS 对应 ext4/NFS 的关系
- **[[10-Virtual-Context]]**：DAM 的 `TRANSIENT/TASK/SESSION` 数据实际落到模块 10 定义的具体 segment
- **[[06-IPC]]**：`ShareMode.BROADCAST` 通过模块 06 的 Event Bus 实现
- **[[11-Stable-ABI]]**：DAI 接口（`data.read/write/search`）是 Stable ABI 的核心组成部分

---

## 架构位置

```
┌─────────────────────────────────────────┐
│              Skill（Ring 3）             │
│   data.read() / data.write()            │  ← Skill 只看到这一层
└──────────────────┬──────────────────────┘
                   │
┌──────────────────▼──────────────────────┐
│     Data Access Interface（DAI）         │  ← 本模块（VFS 等价）
│  scope 解析 │ durability 路由 │ 成本感知  │
└──────┬───────────────────────┬──────────┘
       │                       │
┌──────▼──────┐         ┌──────▼──────────┐
│  Virtual    │         │  Memory         │
│  Context    │         │  Hierarchy      │
│ (模块 10)   │         │  (模块 01)      │
│ W/D/I 段    │         │  Cache/RAG/Vault│
└─────────────┘         └─────────────────┘
```

---

## 开放问题

- [ ] `Scope.AUTO` 的层次穿透是否应该有成本阈值？避免 skill 无意中触发高成本 RAG 检索
- [ ] `Durability.SESSION` 在 session 结束时的持久化提示：由用户决定还是 skill 声明默认行为？
- [ ] Write-around（大块知识直接写 RAG）的触发条件：按数据大小阈值自动选择？
- [ ] 多 agent 并发写同一 key 时的冲突解决：DAI 层处理还是交给 [[10-Virtual-Context]] 的 CoW 机制？
- [ ] DAI 的查找结果是否应该附带"命中层"元数据，让 skill 感知数据来源和可信度？
