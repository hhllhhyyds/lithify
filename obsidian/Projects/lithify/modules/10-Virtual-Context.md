---
date: 2026-05-13
type: project
status: planning
tags: [project, lithify, virtual-context, isolation, memory]
ai-first: true
---

## For future Claude
lithify 子模块设计文档——虚拟 Context 隔离。类比 OS 虚拟内存：每个 agent 看到"完整的 context"，实际上只是全局 context 的隔离切片。解决当前 subagent 要么权限过大（继承全部父 context）要么过小（完全看不到）的问题。

---

## 问题陈述

当前 subagent 的 context 继承策略只有两种极端：
- **全继承**：subagent 看到父 agent 的全部 context → 权限过大，信息泄露，context 浪费
- **全隔离**：subagent 只有任务描述 → 权限过小，缺少必要背景，重复输入

需要一个细粒度的 context 视图机制，让每个 agent 看到恰好需要的内容。

---

## OS 类比

| OS 虚拟内存机制 | Agent 等价 |
|---|---|
| 虚拟地址空间 | 每个 agent 的虚拟 context 视图 |
| 物理内存页 | 实际的 context 片段（存在 Harness） |
| 页表（Page Table） | Context Map（虚拟视图 → 物理片段的映射） |
| 内存映射（mmap） | 将共享 context 段挂载到 agent 视图 |
| Copy-on-Write（CoW） | agent 修改 context 时，创建私有副本 |
| 内存保护位（rwx） | context 段的读/写/执行权限 |
| 地址空间隔离 | agent 间 context 完全隔离，不可越界 |

---

## 设计方案

### 1. Context 段类型

物理 context 存储按段管理，每段有独立的权限和生命周期：

| 段类型 | 内容 | 默认权限 | 生命周期 |
|---|---|---|---|
| `system` | Harness 核心规则 | RO（全局只读） | 永久 |
| `skill_def` | 当前 skill 定义 | RO | Session 级 |
| `shared_bg` | 共享背景知识 | RO | 按需 |
| `task_input` | 当前任务输入 | RO | Task 级 |
| `tool_results` | Tool 返回结果 | RW（本 agent） | Task 级 |
| `working` | agent 的工作区 | RW | Task 级 |
| `parent_summary` | 父 agent 提供的摘要 | RO | Task 级 |

### 2. 虚拟 Context 视图

每个 agent 有自己的 Context Map，决定它能"看到"哪些段：

```
Agent abc123 的 Context Map：

虚拟地址    物理段 ID       权限    备注
0x0000     seg:system      RO      Harness 全局规则
0x1000     seg:skill_cr    RO      code-review skill 定义
0x2000     seg:shared_proj RO      项目背景（共享，CoW）
0x3000     seg:task_4821   RO      本次任务输入
0x4000     seg:tools_abc   RW      本 agent 的 tool 结果
0x5000     seg:work_abc    RW      本 agent 工作区
```

父 agent 创建 subagent 时，通过 Context Map 精确控制子 agent 能看到什么：

```python
# orchestrator 创建 subagent，精确控制 context 视图
subagent = harness.spawn(
    skill="code-review",
    context_map=[
        Mount(seg="system",        perm=RO),
        Mount(seg="skill_cr",      perm=RO),
        Mount(seg="shared_proj",   perm=RO, cow=True),   # CoW：修改时创建私有副本
        Mount(seg="task_4821",     perm=RO),
        Alloc(name="tools",        perm=RW, size=8192),  # 分配新段
        Alloc(name="work",         perm=RW, size=16384),
    ]
)
```

### 3. Copy-on-Write（CoW）

当 subagent 需要"修改"共享段时，不直接修改，而是创建私有副本：

```
原始共享段: seg:shared_proj (只读)
           ↓  agent 尝试写入
Harness 拦截 → 创建私有副本: seg:shared_proj_copy_abc123
agent 的写操作作用于私有副本，不影响原始段和其他 agent
```

父 agent 可以选择是否将子 agent 的修改"合并回"共享段（需要 Ring 0 仲裁）。

### 4. Context 段的生命周期管理

```
Task 开始 → Harness 分配 task_input、tools、work 段
          → 构建 agent 的 Context Map
          → agent 运行
          → agent exit() 时返回 work 段内容作为结果
          → Harness 释放 tools、work 段（或按策略保留用于调试）
          → task_input 段降级为历史，可被 eviction
```

---

## 开放问题

- [ ] Context Map 的构建由谁负责？Orchestrator 提供意图，Harness 自动构建，还是 Orchestrator 直接指定？
- [ ] CoW 的粒度：整段 CoW 还是页级 CoW？对 context 内容的细粒度修改如何处理？
- [ ] 子 agent 修改合并回父段的冲突解决策略：最后写入胜？还是需要 LLM 仲裁？
- [ ] Context Map 的动态修改：运行中的 agent 能否请求挂载新的段？需要什么授权？
