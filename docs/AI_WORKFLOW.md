# AI 编程流程规范

## 概述

Lithify 项目全程使用 AI 辅助编程（Claude Code）。本文档定义了从设计到交付的完整工作流，确保每个环节都能被 AI 正确理解和执行。

## 核心原则

1. **文档即真相**：所有设计决策写在仓库文档中，不存在于任何人脑子里。
2. **CLAUDE.md 是入口**：Claude Code 通过 CLAUDE.md 找到所有需要的上下文。
3. **小步快跑**：每次任务范围小而明确，一个 PR 解决一个问题。
4. **TDD 测试先于实现**：测试是规格的具体化；先写会失败的测试（Red），等人审查确认规格后再实现代码（Green）。
6. **四层感知**：每次实现都要清楚当前工作落在哪一层（Skill / RAG / Tool / 路由），不做跨层的过度设计。
7. **先跑通再优化**：最小可运行优先，不提前引入分布式或云依赖。

## 工作流阶段

### 阶段 1：Spec（需求/设计）

**由人完成**，AI 辅助讨论。

- 明确这个任务对应四层记忆的哪一层（Skill / RAG / Tool / 路由）
- 在 `docs/` 下写清楚要做什么（若有新设计决策）
- 更新 CLAUDE.md 中的文档索引

**产出**：清晰的任务描述

### 阶段 2：Plan（任务拆解）

**由 AI 完成**，人审核。

- 基于设计文档，将实现拆解为具体的、可独立完成的任务
- 每个任务写入一个 markdown 文件，包含：目标、涉及的 crate、依赖的 trait/类型、验收标准
- **所有任务文档放在 `dev_plan/` 目录下**
- 遵守任务的编号规范

**任务编号规范**：

- **主任务**：使用整数编号（1, 2, 3, ...），按开发轮次顺序递增
- **子任务**：使用小数编号（X.Y）
  - 格式：`主任务编号.子任务序号`
  - 示例：`10.1` 表示第 10 个任务的第 1 个子任务
  - 使用场景：Bug 修复、功能增强、技术债务、性能优化

- **文件命名**：
  - 主任务：`NN-task-name.md`
  - 子任务：`NN.M-task-name.md`

- **任务状态标记**：
  - ✅ — 已完成
  - 🚧 — 进行中
  - ⬜ — 未开始

**产出**：任务文档（`dev_plan/NN-task-name.md`）

### 阶段 3：Implement（实现）

#### TDD Phase 1 — 写测试（先于实现）

- 每个任务启动一个新分支：`feat/<task-name>`
- Claude Code 读取 CLAUDE.md → 找到相关文档 → 理解上下文
- **先写会失败的测试**，定义功能的预期行为（单元测试 + 集成测试）
- 测试文件必须能编译，但断言会失败（Red 阶段）
- 提交消息格式：`test: add tests for #XX (<feature name>)`
- **停下来，等人工审查测试**

#### TDD Phase 2 — 实现代码（等测试批准后）

- 人工批准测试后，Claude Code 实现最小代码让所有测试通过
- 提交消息格式：`feat: implement #XX (<feature name>)`

**给 Claude Code 的标准指令模板（Phase 1）**：

```
读取 CLAUDE.md 了解项目上下文。

任务：<具体任务描述>

要求：
1. 先写会失败的测试（单元测试 + 集成测试），定义预期行为
2. 测试必须能编译，但断言会失败（Red）
3. 用 "test: add tests for #XX (<feature name>)" 提交
4. 停下来等人工审查测试
```

**给 Claude Code 的标准指令模板（Phase 2）**：

```
人工已批准测试。现在实现最小代码让所有测试通过。

要求：
1. 实现前先跑测试确认处于 Red 状态
2. 实现后确保所有测试通过（Green）
3. 如需要可做重构（Refactor）
4. 运行 cargo test && cargo clippy 确保全部通过
5. 用 "feat: implement #XX (<feature name>)" 提交
```

### 阶段 4：Test（测试验证）

**由 CI 自动完成 + 人工审查**。

- `cargo test` — 运行所有单元测试和集成测试
- `cargo clippy` — 代码风格和常见错误检查
- `cargo fmt --check` — 代码格式检查

#### 测试覆盖率

- 使用 `cargo-llvm-cov` 检测代码覆盖率
- 覆盖率报告作为 CI 的一部分运行，不阻断合并，但作为参考指标
- 新增代码应附带对应的测试，保持覆盖率不下降

### 阶段 5：AI Review（自动代码审查）

**由 Claude Code 完成**，在人工 review 之前执行。

每个 feature 分支完成后，执行以下 review 指令：

```
读取 CLAUDE.md 了解项目上下文。

Review 当前分支相对于 main 的所有改动（git diff main...HEAD）。

检查以下维度：
1. 架构合规 — 是否符合四层记忆架构的核心原则
2. 代码质量 — 错误处理、命名、代码组织
3. 测试覆盖 — 是否覆盖了任务中要求的测试场景
4. 文档注释 — pub 类型和方法是否有英文 doc comment
5. 简单栈原则 — 是否引入了不必要的分布式/云依赖
6. 语言规范 — 代码注释是否全部使用英文

输出格式：
- ✅ 通过的项
- ⚠️ 建议改进的项（非阻塞）
- ❌ 必须修改的项（阻塞合并）

如果有 ❌ 项，直接修复后重新提交。
如果只有 ⚠️ 项，列出建议，交由人工判断是否修改。
```

**流程**：
1. Claude Code 完成实现 → 自己跑 review 指令
2. 有 ❌ 项 → 自行修复 → 重新 review → 直到无 ❌
3. 输出 review 报告
4. 交给人工做最终审查

### 阶段 6：更新 DOCS（文档同步）

**由 Claude Code 完成**，在 AI Review 通过后、人工 Review 之前执行。

代码改完了不算完——文档必须跟着代码走。每次任务完成后，从修改的代码位置出发，**自下而上**扫描哪些文档需要更新：

```
从修改的代码位置向上追溯：
1. 修改的代码属于哪个 crate？
   → 确认该 crate 是否需有专属说明
2. 修改影响了四层记忆架构中的哪一层？
   → 检查是否需要更新相关设计文档
3. 引入了新依赖？
   → 更新 docs/TECH_STACK.md
4. 任务在 tasks/ 中有对应条目？
   → 更新任务状态和描述
5. 对整个项目结构或核心原则有影响？
   → 更新根目录 CLAUDE.md（文档索引、crate 结构图等）
```

**原则**：
- 文档即真相——如果文档和代码不一致，后来的人（包括 AI）会被误导
- 先改完文档，再提交代码。不要留"回头补文档"的债
- 不确定该不该改的文档，宁可多改一行，也别漏掉

### 阶段 7：人工 Review（最终审查）

**由人完成**，是合并前的最后一道关。

- 查看 AI Review 报告
- 浏览代码改动，关注设计判断和架构方向
- 通过 → 合并
- 不通过 → 反馈给 Claude Code 修改

### 阶段 8：Merge

- Squash merge 到 main
- 后期：GitHub Actions 自动运行测试

## 分支策略

```
main                    ← 稳定分支，永远可编译
├── feat/<task-name>    ← 功能分支
├── fix/<issue>         ← 修复分支
└── docs/<topic>        ← 文档分支
```

## Commit 规范

格式：`<type>(<scope>): <description>`

类型：
- `feat` — 新功能
- `fix` — 修复
- `docs` — 文档
- `test` — 测试
- `refactor` — 重构
- `chore` — 构建/工具

示例：
- `feat(core): define MemoryLayer and Storage traits`
- `feat(runtime): implement conversation loop with skill injection`
- `test(rag): add integration tests for vector store`
- `docs: update TECH_STACK.md with sqlite-vec rationale`
- `chore: set up workspace dependencies`

每个阶段的实现都应遵循本文档定义的 TDD 流程。
