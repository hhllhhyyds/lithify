# Task 3: Client 实现 + 测试

## 目标

实现 `AnthropicClient`，通过 `reqwest` 调用 Anthropic Messages REST API，完成 `LLMClient` trait 的实现。

包含完整的测试：mock HTTP 集成测试 + `#[ignore]` 真实 LLM 测试 + 本地测试脚本。

## 涉及文件

| 文件 | 操作 | 说明 |
|------|------|------|
| `crates/llm/src/client.rs` | 新建 | AnthropicClient + LLMClient 实现 |
| `crates/llm/src/lib.rs` | 修改 | 声明 pub mod client |
| `scripts/test_real_llm.sh` | 新建 | 真实 LLM 测试脚本 |

## 依赖

- Task 1: Config, reqwest::Client
- Task 2: messages_to_request, response_from_value

## context

- `crates/core/src/lib.rs:147-151` — `LLMClient` trait
- `crates/core/src/lib.rs:106-114` — `LLMError` 类型
- `crates/llm/src/config.rs` — Config 结构体（Task 1）
- `crates/llm/src/convert.rs` — 转换函数（Task 2）
- `crates/llm/Cargo.toml` — reqwest, tracing 等依赖（Task 1）

## 验收标准

- [x] `cargo test -p lithify-llm` 全部通过
- [x] Mock HTTP 测试验证 chat() 正常返回和错误处理
- [x] `#[ignore]` 测试可用 `scripts/test_real_llm.sh` 触发
- [x] 超时设置正确映射到 reqwest::Client::builder()

## 子任务

- [ ] 3.1: 实现 `AnthropicClient`（new() + LLMClient::chat()），包含 reqwest 调用、tracing 日志
- [ ] 3.2: Mock HTTP 集成测试（模拟 Anthropic API 端点，验证 chat() 行为）
- [ ] 3.3: 真实 LLM 测试（#[ignore]），测试纯文本消息
- [ ] 3.4: 创建 `scripts/test_real_llm.sh`
