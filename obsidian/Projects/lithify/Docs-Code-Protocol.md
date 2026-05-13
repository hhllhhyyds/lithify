---
date: 2026-05-13
type: project
status: active
tags: [project, lithify, protocol, docs, code, workflow]
ai-first: true
---

## For future Claude
lithify 文档与代码协同演进协议，由 hhl 于 2026-05-13 确定。解决设计文档（vault）与代码实现如何保持同步、各自承担什么职责的问题。核心原则：vault 记录"为什么"，代码记录"是什么"。

---

## 问题背景

随着项目推进，设计文档和代码实现往往出现漂移：
- 代码迭代快，设计文档更新滞后
- 实现中发现设计不合理，改了代码却没有更新文档
- 新人（包括 future Claude）不知道应该信任哪一份材料

---

## 职责划分

### Vault（本知识库）= **Why**

> 设计意图、架构决策、推理过程 — 稳定，轻易不改

- 存放：架构原则、模块设计、假设与权衡、ADR（Architecture Decision Records）
- 特征：即使实现细节变了，"为什么这样设计"通常仍然有效
- 更新频率：低 — 有重大架构变更时才改

### 代码仓库 = **What**

> 当前实现 — 频繁变动，以代码为准

- 存放：实际的数据结构、函数签名、接口约定
- `CLAUDE.md`（代码仓库根目录）：记录**当前状态**，不解释 Why
  - 实现进度（哪些模块已完成）
  - 与 vault 设计的已知偏差
  - 开发约定（如何运行测试、代码风格要求等）
- 特征：以最新 commit 为准，vault 里的代码片段可能过时

---

## 四条同步规则

| 规则 | 触发时机 | 动作 |
|---|---|---|
| **R1 编码前读 vault** | 开始实现某模块前 | 读对应 `modules/XX.md`，理解设计意图 |
| **R2 设计变更先改 vault** | 发现设计需要调整时 | 先在 vault 更新设计，再改代码；不允许只改代码不改文档 |
| **R3 实现后更新模块状态** | 某模块实现完成后 | 在 [[Overview]] 的模块状态表中将状态从"初稿"改为"已实现" |
| **R4 重大偏差写 ADR** | 实现与设计出现不可避免的偏差时 | 在 vault 中写 ADR，记录偏差原因和权衡 |

---

## ADR 格式（Architecture Decision Record）

当实现与 vault 设计出现有意义的偏差时，在对应模块文档中追加：

```markdown
### ADR-[YYYYMMDD]-[编号]: [标题]

**背景**：实现时遇到了什么问题
**决策**：做了什么不同于原设计的选择
**后果**：这个决策带来的权衡（正面+负面）
**状态**：已接受 / 待评审 / 已废弃
```

---

## 信任优先级

当 vault 设计与代码实现出现矛盾时：

```
代码实现（当前 commit）> 代码仓库 CLAUDE.md > vault 模块文档
```

**例外**：若 vault 中有明确的"这是目标设计，代码尚未跟上"标记，则以 vault 为准。

---

## Vault 中代码片段的处理

vault 文档（包括 Phase1-Spec.md）中有大量 Rust 代码片段。这些代码片段是**设计说明**，不是权威实现：

- 代码片段可能随实现演进而过时
- 不要把 vault 中的代码直接复制到实现里，应以 vault 的**意图**为准
- 真实签名以 `src/` 下的代码为准

---

## 实践示例

**场景 A：开始实现 ContextManager**
1. 读 [[modules/01-Memory-Hierarchy]] 和 [[Phase1-Spec]] — 理解 Why
2. 实现 `src/harness/context.rs`
3. 实现后在 [[Overview]] 模块状态表更新：01-Memory-Hierarchy → "已实现"

**场景 B：实现中发现 EvictionPolicy 接口需要调整**
1. 先在 [[Phase1-Spec]] 或 [[modules/01-Memory-Hierarchy]] 更新接口定义
2. 写 ADR 说明为何调整
3. 再改代码

**场景 C：Phase 2 开始，架构有较大变化**
1. 在 vault 新建 `Phase2-Spec.md`
2. Phase1-Spec.md 标记 `status: completed`
3. 更新 Overview.md 的模块状态

---

## 关联文档
- 开发技能组：[[Dev-Skills]]
- Phase 1 规格：[[Phase1-Spec]]
- 主架构：[[Overview]]
