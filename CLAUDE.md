# Lithify — AI 编程上下文

## 项目简介

Lithify 是一个 Rust 编写的**自进化 CLI Agent**，基于四层记忆沉降架构（Memory Sedimentation）。它被使用得越多，能力就越强，而 context 消耗不会等比增长。

## 核心原则

接手任务时，先读 [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) 理解项目结构和跨 crate 规范，再读 [`docs/TECH_STACK.md`](docs/TECH_STACK.md) 了解技术选型背景。

## 文档索引

| 文档 | 内容 |
|------|------|
| [README.md](./README.md) | 项目说明、设计原则、开发路线 |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | 四层记忆架构、沉降管道、crate 职责、项目结构 |
| [docs/TECH_STACK.md](docs/TECH_STACK.md) | 技术选型及理由 |
| [evolution-en.md](./evolution-en.md) | 四层记忆架构完整阐述（英文） |
| [evolution-zh.md](./evolution-zh.md) | 四层记忆架构完整阐述（中文） |

## 实现任务时的工作流

基于 [Agentic Engineering Framework](https://github.com/hhllhhyyds/agentic-engineering-framework)，通过 `/cmds` 查看所有可用命令。按需加载对应 skill 指导 AI 开发：

| 阶段 | 命令 | 说明 |
|------|------|------|
| 需求澄清 | `/requirements-clarification` | 明确要解决什么问题 |
| 系统设计 | `/system-design` | 架构与方案设计 |
| 代码实现 | `/code-generation` | TDD 模式实现 |
| 测试生成 | `/test-generation` | 单元/集成/性能测试 |
| 代码审查 | `/code-review` | 多 Agent 并行审查 |
| 问题排查 | `/troubleshooting` | 编译/运行/现网问题 |
| 性能优化 | `/performance-optimization` | 性能分析与优化 |
| Rust 规范 | `/std-rust` | Rust 编码规范速查与完整参考 |
| Lithify Rust 规范 | `/lithify-rust` | 项目特有 Rust 规则（check.sh 提交前检查、错误处理、测试等） |
