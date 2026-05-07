# 技术选型

## 总览

| 层面 | 选型 | 理由 |
|------|------|------|
| 语言 | Rust 2024 edition | 内存安全、零成本抽象、async 一等公民；2024 edition 带来 `impl Trait` 位置扩展等改进 |
| 异步运行时 | tokio | Rust 异步生态事实标准 |
| 序列化 | serde + serde_json | Rust 序列化标配 |
| LLM API | reqwest（裸 HTTP） | Anthropic Messages API，raw HTTP 调用，无 SDK 依赖 |
| 向量存储 | sqlite-vec | 嵌入式、零运维，与 SQLite 共用存储层 |
| Embedding | fastembed / local 模型 | 本地推理，无需外部 API |
| 沙箱执行 | tokio::process | 子进程模式，简单可靠，无需 Wasm 运行时 |
| 错误处理 | thiserror + anyhow | trait 内 thiserror 具体类型，应用层 anyhow |
| 日志追踪 | tracing + tracing-subscriber | 异步友好，结构化日志 |
| 测试 | cargo test + mockall | trait 天然适合 mock |
| 唯一 ID | uuid (v4) | 事件、会话唯一标识 |
| 时间戳 | chrono | UTC 时间戳 |
| CLI | clap | Rust CLI 事实标准 |
| CI/CD | GitHub Actions（后期） | 自动化测试、lint、发布 |

## 各选型详细说明

### Rust 2024 Edition

- trait 系统天然适合"接口有立场，实现无假设"的设计原则
- async/await 适配 LLM API 调用、沙箱执行、文件 I/O 等密集操作
- `Arc<dyn Trait>` 支持跨线程安全共享
- 编译期内存安全保证，适合长期运行的 Agent 守护进程
- 零成本抽象让四层记忆系统的路由开销降到最低

### tokio

- Rust 异步生态的事实标准
- `tokio::process` 直接用于本地沙箱子进程
- `tokio::sync` 提供异步锁、channel 等并发原语
- 丰富的 tower 中间件生态（后期 HTTP 服务可复用）

### sqlite-vec

- SQLite 的向量扩展，支持在 SQLite 内直接做 embedding 相似度搜索
- 避免引入独立的向量数据库（Milvus、Qdrant 等），符合"简单栈"原则
- 与业务数据共用同一存储层（Session 事件、RAG 条目、Tool 元数据均存 SQLite）
- 磁盘上的 embeddable 方案，一台笔记本能跑通

### tokio::process（沙箱）

- 子进程模式，`tokio::process::Command` 原生支持异步等待
- 生成的 Tool 作为独立进程执行，宿主通过 stdin/stdout 通信
- 无需引入 Wasm 运行时，依赖更少，构建更快
- 进程级别的隔离对 CLI 场景已足够

### thiserror + anyhow

- **thiserror**：在 core crate 中为每个 trait 定义精确的错误类型（`StorageError`、`LLMError`、`SandboxError`、`ToolError`）
- **anyhow**：在 runtime 和 example 中做顶层错误聚合
- 分层清晰：trait 消费者看到精确错误，应用层简化处理

### mockall

- 所有核心组件都是 trait，mockall 自动生成 mock 实现
- 测试 ControlLoop 时可 mock SkillLoader、RagStore、ToolSet、LLMClient
- 无需启动真实 LLM 或沙箱即可跑完整控制循环测试

## 依赖版本策略

- 使用 workspace 级别的依赖管理（`[workspace.dependencies]`）
- 所有 crate 共享同一版本的公共依赖
- 初版不锁死具体版本，使用兼容范围（如 `tokio = "1"`）
