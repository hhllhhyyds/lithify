---
date: 2026-05-13
type: project
status: planning
tags: [project, lithify, abi, interface, versioning]
ai-first: true
---

## For future Claude
lithify 子模块设计文档——稳定 ABI。类比 Linux syscall 向后兼容承诺：Harness 内部实现可以自由演化，只要 Skill ↔ Harness 接口契约不变，已有 skills 就不会崩坏。这是 skills 和 harness 能独立进化的前提。

---

## 问题陈述

当前 agent 框架中，skill（prompt 文件）和 harness（执行引擎）紧耦合：harness 一升级，已有 skill 可能静默失效，没有任何警告。没有接口版本管理，没有向后兼容承诺，skills 无法独立演化。

---

## OS 类比

| OS 概念 | Agent 等价 |
|---|---|
| Linux syscall 接口（POSIX） | Skill ↔ Harness ABI |
| syscall 编号稳定性 | Skill 调用原语的稳定性 |
| 向后兼容承诺 | "旧 skill 在新 harness 上必须能跑" |
| ABI 版本（`libc.so.6`） | Harness ABI 版本（`harness-abi-v2`） |
| 弃用警告（`__deprecated__`） | Skill API 弃用通知机制 |

---

## 设计方案

### 1. Skill ↔ Harness 接口的最小集合

ABI 定义 skill 可以合法调用的原语，分为三类：

**① Context 访问原语**
```
context.read(segment, key) → value
context.write(segment, key, value) → void     # 仅限 RW 段
context.get_budget() → int                    # 剩余 token budget
context.get_agent_id() → uuid
```

**② Tool 调用原语**
```
tool.invoke(name, args) → result              # 同步调用
tool.invoke_async(name, args) → future        # 异步调用（见模块 02）
tool.list_available() → [ToolSpec]            # 列出可用 tool
```

**③ Agent 生命周期原语**
```
agent.spawn(skill, context_map, budget) → agent_id   # 创建 subagent
agent.wait(agent_id) → result                          # 等待 subagent 完成
agent.send_signal(agent_id, signal) → void             # 发送信号
agent.exit(result, status) → void                      # 退出
```

**④ IPC 原语**
```
ipc.publish(event, payload) → void
ipc.subscribe(event, filter) → subscription
ipc.send(to, message) → void
```

### 2. ABI 版本管理

每个 skill 声明所需的最低 ABI 版本：

```yaml
# skill frontmatter
abi-version: ">=2.0, <3.0"
```

Harness 启动时检查所有已安装 skill 的 ABI 兼容性：

```
Harness v3.1 → ABI v2.4
  ✅ code-review@1.2.3 requires abi >=2.0 → compatible
  ✅ obsidian-save@2.0.1 requires abi >=2.1 → compatible
  ⚠️  legacy-tool@0.9.0 requires abi >=1.0,<2.0 → DEPRECATED, running in compat mode
  ❌ future-skill@4.0.0 requires abi >=3.0 → INCOMPATIBLE, disabled
```

### 3. 向后兼容规则

ABI 版本遵循语义化版本：

| 版本变更 | 规则 | 示例 |
|---|---|---|
| Patch（2.0.x） | 只修 bug，绝对向后兼容 | 修复 context.read 的竞态 |
| Minor（2.x.0） | 新增原语，不修改现有原语 | 新增 ipc.subscribe_once() |
| Major（x.0.0） | 允许破坏性变更，旧版进入 compat 模式 | 重构 tool.invoke 参数格式 |

**破坏性变更流程**：
1. 新版本发布时，旧 API 标记为 `@deprecated`
2. Deprecated API 在 compat 模式下继续工作（至少两个 major 版本）
3. 发送 `SKILL_ABI_DEPRECATED` 事件通知 skill 维护者
4. 两个 major 版本后，compat 模式移除

### 4. ABI Shim（兼容层）

对于无法升级的旧 skill，Harness 提供 ABI shim：

```
旧 skill (abi v1) → ABI v1 Shim → 转换调用格式 → 新 Harness (abi v2)
```

Shim 自动将旧格式调用翻译为新格式，skill 无需感知版本差异。

---

## 开放问题

- [ ] ABI 的形式化规范用什么语言描述？OpenAPI？Protobuf IDL？还是自定义 DSL？
- [ ] Compat 模式的性能开销如何控制？
- [ ] skill 如何测试自己对新版 ABI 的兼容性？CI 中的自动兼容性测试方案？
- [ ] 当 ABI 与 [[09-Security-Model]] 的 capability 模型版本不一致时如何处理？
