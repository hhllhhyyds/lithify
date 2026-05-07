# Task 1: 依赖 + Config + lib.rs

## 目标

搭建 `lithify-llm` crate 的基础设施：添加外部依赖、实现 `Config` struct 用于读取环境变量配置、更新 `lib.rs` 声明模块。

`Config` 从环境变量读取：
- `ANTHROPIC_API_KEY`（必需）
- `ANTHROPIC_MODEL`（可选，默认 `claude-sonnet-4-6`）
- `ANTHROPIC_MAX_TOKENS`（可选，默认 4096）
- `ANTHROPIC_TIMEOUT_SECS`（可选，默认 60）
- `ANTHROPIC_BASE_URL`（可选，默认 `https://api.anthropic.com`）

## 涉及文件

| 文件 | 操作 | 说明 |
|------|------|------|
| `crates/llm/Cargo.toml` | 修改 | 新增 reqwest, tracing, tokio 依赖 |
| `crates/llm/src/config.rs` | 新建 | Config struct + from_env() |
| `crates/llm/src/lib.rs` | 修改 | 声明 pub mod config |

## 依赖

无

## context

- `crates/core/src/lib.rs:106-114` — `LLMError` 类型，`from_env` 返回此错误
- `crates/llm/Cargo.toml` — 当前依赖只有 `lithify-core`
- `crates/llm/src/lib.rs` — 当前仅注释

## 验收标准

- [x] `cargo check -p lithify-llm` 通过
- [x] `Config::from_env()` 在 `ANTHROPIC_API_KEY` 未设置时返回 `LLMError::Api`
- [x] `Config::from_env()` 在 `ANTHROPIC_API_KEY` 已设置时返回 `Ok(config)`
- [x] `Config.model` 默认值为 `claude-sonnet-4-20250514`

## 子任务

- [x] 1.1: 更新 Cargo.toml，添加 reqwest（+json, rustls-tls）、tracing、tokio 依赖
- [x] 1.2: 创建 config.rs，实现 Config 结构体和 from_env()
- [x] 1.3: 更新 lib.rs，声明 pub mod config
- [x] 1.4: config 单元测试（环境变量读取、默认值、缺失 key 的错误处理）
