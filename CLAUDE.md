# Lithify — AI 编程上下文

## 项目简介

Lithify 是一个 Rust 编写的**自进化 CLI Agent**，基于四层记忆沉降架构（Memory Sedimentation）。它被使用得越多，能力就越强，而 context 消耗不会等比增长。

## 核心原则

项目设计原则见 [`README.md`](./README.md#principles)，AI 编程流程见 [`docs/AI_WORKFLOW.md`](docs/AI_WORKFLOW.md)。

接手任务时，先读 [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) 理解项目结构和跨 crate 规范，再读 [`docs/TECH_STACK.md`](docs/TECH_STACK.md) 了解技术选型背景。

## 文档索引

| 文档 | 内容 |
|------|------|
| [README.md](./README.md) | 项目说明、设计原则、开发路线 |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | 四层记忆架构、沉降管道、crate 职责、项目结构 |
| [docs/TECH_STACK.md](docs/TECH_STACK.md) | 技术选型及理由 |
| [docs/AI_WORKFLOW.md](docs/AI_WORKFLOW.md) | AI 编程流程规范 |
| [evolution-en.md](./evolution-en.md) | 四层记忆架构完整阐述（英文） |
| [evolution-zh.md](./evolution-zh.md) | 四层记忆架构完整阐述（中文） |

## 实现任务时的工作流

1. 读 `docs/ARCHITECTURE.md` 理解项目结构和 crate 职责，再读 `docs/TECH_STACK.md` 了解技术选型背景
2. 读 `docs/AI_WORKFLOW.md` 了解完整流程，确认当前任务属于哪个阶段
3. **TDD Phase 1（测试先行）**：写能编译但断言失败的测试（Red），提交并等待人工审查
4. **人工审查测试**：确认测试覆盖了你期望的行为后继续
5. **TDD Phase 2（实现）**：实现最小代码让所有测试通过（Green），提交
6. 按照 `docs/AI_WORKFLOW.md` 中的检查清单验证全部通过
7. AI 自我 review 后交人工审查
