# Feature: LLM Client

**作者**: hhl  
**日期**: 2026-05-07  
**状态**: Draft

---

## 1. 背景 (Background)
### 1.1 问题描述
- Lithify 作为一个自进化 CLI Agent，核心能力是与 LLM 交互
- 当前 `lithify-llm` crate 仅有骨架，没有 LLM API 调用能力
- 需要从零搭建 Anthropic Claude API 的调用通路，使上层 crate（runtime、rag）能通过统一接口发送消息、获取回复
### 1.2 现状分析
- Task 1（Core Traits）已完成并合并入 main
  - `lithify-core` 已定义 `LLMClient` trait（`async fn chat(&self, messages: &[Message]) -> Result<Response, LLMError>`）
  - 消息类型体系已完备：`Message`、`ContentBlock`、`ToolCall`、`ToolResult`、`Response`、`Usage`
  - 错误类型 `LLMError` 包含 `Api(String)`、`RateLimited`、`Network(String)` 三个变体
  - `MockLLMClient` 已在 core 的测试中存在，可供上层 crate 测试使用
- `lithify-llm` crate 骨架已创建，`src/lib.rs` 仅含注释，无实现代码
- llm crate 当前仅依赖 `lithify-core`，无其他依赖
- 项目未配置 workspace 级公共依赖（`[workspace.dependencies]`），各 crate 各自声明
### 1.3 主要使用场景
- `runtime` crate 在对话循环中调用 `LLMClient::chat()` 发送用户消息并获取 LLM 回复
- `rag` crate 在后续 Phase 中调用 LLM 进行压缩或代码生成
- `tools` crate 在执行 tool call 后将结果回注给 LLM，需要格式转换

## 2. 目标 (Goals)
- 从零搭建 Anthropic Claude API 调用通路，实现 `LLMClient` trait
- 支持消息发送、响应解析、tool call 格式转换
- 提供真实 LLM 测试脚本，可在本地手动验证

### 2.1 非目标 (Non-Goals)
- **不做 streaming**：Phase 2 再支持
- **不做多 provider**：当前只对接 Anthropic，不抽象多 provider 接口
- **不做重试逻辑**：仅做错误映射（`Api`、`RateLimited`、`Network`），重试策略由上层决定
- **不做 CI 集成**：真实 LLM 测试因依赖 API Key，仅在本地手动运行，CI 中跳过

## 3. 需求细化 (Requirements)
### 3.1 功能性需求
- **Config 模块**（`config.rs`）：从环境变量 `ANTHROPIC_API_KEY` 读取 API Key，支持配置模型名称（默认 `claude-sonnet-4-20250514`）、`max_tokens`、超时、API base URL 等参数
- **Client 模块**（`client.rs`）：实现 `LLMClient::chat()`，调用 Anthropic Messages API，发送消息并返回 `Response`
- **Convert 模块**（`convert.rs`）：Lithify 内部类型（`Message`、`ContentBlock`、`ToolCall`、`ToolResult`）与 Anthropic API wire format 的双向转换
- **错误处理**：将 API 返回的错误（HTTP 状态码、网络错误、限流）映射到 `LLMError`（`Api`、`RateLimited`、`Network`）
- **真实 LLM 测试**：`#[ignore]` 标记的集成测试，通过本地脚本 `scripts/test_real_llm.sh` 读取环境变量并触发
### 3.2 非功能性需求
- **超时控制**：HTTP 请求需设置超时（默认 60s），防止 LLM 无响应时永久阻塞
- **API Key 安全**：仅从环境变量 `ANTHROPIC_API_KEY` 读取，不写死在代码中，不持久化到磁盘
- **日志**：使用 `tracing` crate 记录请求/响应摘要（不记录完整消息内容），便于调试和监控
- **API 版本**：不固定 Anthropic API 版本头，由 SDK 默认行为决定

## 4. 设计方案 (Design)
### 4.1 方案概览
- **整体思路**：通过 `reqwest` 直接调用 Anthropic Messages REST API，封装在 `AnthropicClient` 中实现 `LLMClient` trait。不做 streaming，不做多 provider。
- **模块划分**：
  - `config.rs` → `Config` struct（`api_key`、`model`、`max_tokens`、`timeout`），从环境变量构造
  - `client.rs` → `AnthropicClient`，持有 `Config` + `reqwest::Client`，实现 `LLMClient::chat()`
  - `convert.rs` → Lithify 内部类型与 Anthropic API JSON 格式的双向转换
- **依赖方向**：`client` → `convert`（消息转换），`client` → `config`（配置读取），单向无循环
- **数据流**：
  ```
  caller 传入 &[Message]
    → convert 转为 Anthropic API 请求 JSON
    → reqwest POST {base_url}/v1/messages  (base_url 来自 Config，默认 https://api.anthropic.com)
    → 解析响应 JSON
    → convert 转回 lithify Response
    → 返回 Result<Response, LLMError>
  ```
- **关键权衡**：裸 HTTP 而非 SDK → 更可控，但需自行处理 auth header、错误码映射、限流检测；不做 streaming → 实现简单，后续加不影响接口
### 4.2 组件设计 (Component Design)
#### 4.2.1 核心类/模块设计
- **`config.rs`**：`Config` struct，包含 `api_key`、`model`、`max_tokens`、`timeout`、`base_url` 字段。`Config::from_env() -> Result<Self, LLMError>` 从环境变量构造（`ANTHROPIC_API_KEY`、`ANTHROPIC_MODEL`、`ANTHROPIC_MAX_TOKENS`、`ANTHROPIC_TIMEOUT_SECS`、`ANTHROPIC_BASE_URL`）
- **`client.rs`**：`AnthropicClient` struct，持有 `Config` + `reqwest::Client`，实现 `LLMClient` trait
- **`convert.rs`**：纯函数，`messages_to_request(messages: &[Message]) -> serde_json::Value` 和 `response_from_value(value: serde_json::Value) -> Result<Response, LLMError>`，无状态
#### 4.2.2 接口设计
- **Config**: `Config::from_env() -> Result<Self, LLMError>` — 从环境变量构造
- **AnthropicClient**: `AnthropicClient::new(config: Config) -> Self` — 内部创建 `reqwest::Client`
- **LLMClient**: `async fn chat(&self, messages: &[Message]) -> Result<Response, LLMError>` — core 定义的 trait，AnthropicClient 实现它
- **convert**: `messages_to_request()` / `response_from_value()` 均为 `pub(crate)`，仅 crate 内部使用
#### 4.2.3 数据模型
- N/A — 无持久化数据，消息类型已在 core 中定义
#### 4.2.4 并发模型
- `AnthropicClient` 只持有 `Config`（不可变）和 `reqwest::Client`（线程安全）
- `LLMClient::chat()` 签名 `&self`，天然支持跨线程共享
- 无可变共享状态，不需要锁，`Send + Sync`
#### 4.2.5 错误处理
- HTTP 429 → `LLMError::RateLimited`
- HTTP 4xx(非429) → `LLMError::Api(status + body)`
- HTTP 5xx → `LLMError::Api(status + body)`
- reqwest 网络错误 / 超时 → `LLMError::Network(description)`
- 超时通过 `reqwest::Client::builder().timeout()` 设置（默认 60s）
- 不做重试，重试策略由上层调用方决定
### 4.3 核心逻辑实现

**chat() 调用路径：**

1. `messages_to_request(messages)`：将 `&[Message]` 转为 Anthropic Messages API JSON body（model, max_tokens, messages[]）
2. `reqwest::Client::post("{base_url}/v1/messages")`（base_url 来自 `Config.base_url`，默认 `https://api.anthropic.com`），设置 headers（`x-api-key`、`anthropic-version`、`content-type`），超时控制，`.json(&body)` 发送
3. 状态码检查：429 → `RateLimited`，4xx/5xx → `Api`，200 → 继续
4. `response_from_value(response_json)`：提取 content blocks 和 usage，区分 text 和 tool_use block，组装为 `Response`
5. 返回 `Ok(Response)`

**Convert 映射：**

| Lithify | Anthropic API |
|---------|---------------|
| `Message(Role::User, [ContentBlock::Text(t)])` | `{"role":"user", "content":[{"type":"text","text":t}]}` |
| `Message(Role::Assistant, [ContentBlock::ToolCall(tc)])` | `{"role":"assistant", "content":[{"type":"tool_use",...}]}` |
| `Message(Role::User, [ContentBlock::ToolResult(tr)])` | `{"role":"user", "content":[{"type":"tool_result",...}]}` |
| 响应中 `{"type":"text"}` | `ContentBlock::Text` |
| 响应中 `{"type":"tool_use"}` | `ContentBlock::ToolCall` |
| 响应中 `{"usage":...}` | `Usage{input_tokens, output_tokens}` |
### 4.4 方案优劣分析
- **优势**：无外部 SDK 依赖，HTTP 层面完全可控，实现简单（~200 行核心代码）
- **劣势**：需自行处理 auth header、错误码映射；Anthropic API 格式变更需手动跟进

## 5. 备选方案 (Alternatives Considered)
- **anthropic SDK**：考虑过使用官方 SDK，但选择裸 HTTP 以减少外部依赖，且对 API 交互有完全控制权

## 6. 业界调研 (Industry Research)
- N/A — Anthropic Messages API 是标准 REST API，实现方式直接

## 7. 测试计划 (Test Plan)
### 7.1 单元测试
- `convert.rs` 的序列化/反序列化测试（mock JSON → lithify 类型的双向验证）
- `config.rs` 的环境变量读取测试（设置/清除 env var）
### 7.2 集成测试
- Mock HTTP 层的 `chat()` 测试（通过 `wiremock` 或 `mockito` mock Anthropic API 端点）
- 真实 LLM 测试（`#[ignore]` 标记，通过 `scripts/test_real_llm.sh` 触发）

## 8. 可观测性 & 运维 (Observability & Operations)
### 8.1 可观测性
- **日志**: 使用 `tracing`，在 `chat()` 调用前后记录请求摘要和响应状态（不记录完整消息内容）
