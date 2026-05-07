# Task 2: Convert 模块 + 单元测试

## 目标

实现 Lithify 内部类型与 Anthropic Messages API JSON 格式的双向转换。

核心函数：
- `pub(crate) fn messages_to_request(messages: &[Message]) -> serde_json::Value`
- `pub(crate) fn response_from_value(value: serde_json::Value) -> Result<Response, LLMError>`

## 涉及文件

| 文件 | 操作 | 说明 |
|------|------|------|
| `crates/llm/src/convert.rs` | 新建 | 类型转换函数 |
| `crates/llm/src/lib.rs` | 修改 | 声明 pub(crate) mod convert |

## 依赖

无（只依赖 `lithify-core` 的已有类型）

## context

- `crates/core/src/lib.rs:14-65` — Message, Role, ContentBlock, ToolCall, ToolResult, Response, Usage
- Anthropic Messages API 格式：
  - 请求: `{"model":"...","max_tokens":N,"messages":[{"role":"user/assistant","content":[{"type":"text","text":"..."}]}]}`
  - tool_use: `{"type":"tool_use","id":"...","name":"...","input":{...}}`
  - tool_result: `{"type":"tool_result","tool_use_id":"...","content":"..."}`
  - 响应: `{"content":[{"type":"text","text":"..."},{"type":"tool_use",...}],"usage":{"input_tokens":N,"output_tokens":N}}`

## 验收标准

- [ ] `cargo test -p lithify-llm` 中 convert 测试通过
- [ ] 纯文本消息正确转为 Anthropic 请求 JSON
- [ ] 含 tool_call 的消息正确转为 Anthropic 请求 JSON
- [ ] 含 tool_result 的消息正确转为 Anthropic 请求 JSON
- [ ] Anthropic 响应 JSON 正确转回 Response
- [ ] 非法 JSON 返回 LLMError::Api

## 子任务

- [ ] 2.1: 实现 `messages_to_request()` — 处理 system/user/assistant 消息，支持 text/tool_use/tool_result content block
- [ ] 2.2: 实现 `response_from_value()` — 解析响应 JSON，提取 content blocks 和 usage
- [ ] 2.3: 单元测试 — 纯文本、含 tool call、含 tool result、非法输入
