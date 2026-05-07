use lithify_core::{ContentBlock, LLMError, Message, Response, Role, ToolCall, Usage};
use serde_json::Value;

/// Convert lithify messages into an Anthropic Messages API request body.
///
/// System messages are extracted into the top-level `system` field.
/// User and Assistant messages populate the `messages` array.
pub(crate) fn messages_to_request(model: &str, max_tokens: u32, messages: &[Message]) -> Value {
    let mut system_parts: Vec<String> = Vec::new();
    let mut api_messages = Vec::with_capacity(messages.len());

    for msg in messages {
        match msg.role {
            Role::System => {
                let text: String = msg
                    .content
                    .iter()
                    .filter_map(|b| {
                        if let ContentBlock::Text(t) = b {
                            Some(t.as_str())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                if !text.is_empty() {
                    system_parts.push(text);
                }
            }
            Role::User | Role::Assistant => {
                let blocks: Vec<Value> = msg.content.iter().map(content_block_to_api).collect();
                let role_str = match msg.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    _ => unreachable!(),
                };
                api_messages.push(serde_json::json!({
                    "role": role_str,
                    "content": blocks,
                }));
            }
        }
    }

    let mut body = serde_json::json!({
        "model": model,
        "max_tokens": max_tokens,
        "messages": api_messages,
    });

    if !system_parts.is_empty() {
        body["system"] = Value::String(system_parts.join("\n"));
    }

    body
}

fn content_block_to_api(block: &ContentBlock) -> Value {
    match block {
        ContentBlock::Text(text) => serde_json::json!({
            "type": "text",
            "text": text,
        }),
        ContentBlock::ToolCall(tc) => serde_json::json!({
            "type": "tool_use",
            "id": tc.id,
            "name": tc.name,
            "input": tc.arguments,
        }),
        ContentBlock::ToolResult(tr) => serde_json::json!({
            "type": "tool_result",
            "tool_use_id": tr.tool_call_id,
            "content": tr.content,
            "is_error": tr.is_error,
        }),
    }
}

/// Parse an Anthropic Messages API response into a lithify [`Response`].
pub(crate) fn response_from_value(value: Value) -> Result<Response, LLMError> {
    let content = value
        .get("content")
        .and_then(|c| c.as_array())
        .ok_or_else(|| LLMError::Api("response missing 'content' array".into()))?;

    let mut blocks = Vec::with_capacity(content.len());
    for block in content {
        blocks.push(parse_content_block(block)?);
    }

    let usage = value
        .get("usage")
        .ok_or_else(|| LLMError::Api("response missing 'usage'".into()))?;

    let input_tokens = usage
        .get("input_tokens")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| LLMError::Api("usage missing 'input_tokens'".into()))?;

    let output_tokens = usage
        .get("output_tokens")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| LLMError::Api("usage missing 'output_tokens'".into()))?;

    Ok(Response {
        content: blocks,
        usage: Usage {
            input_tokens,
            output_tokens,
        },
    })
}

fn parse_content_block(block: &Value) -> Result<ContentBlock, LLMError> {
    let typ = block
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LLMError::Api("content block missing 'type'".into()))?;

    match typ {
        "text" => {
            let text = block
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| LLMError::Api("text block missing 'text'".into()))?;
            Ok(ContentBlock::Text(text.to_string()))
        }
        "tool_use" => {
            let id = block
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| LLMError::Api("tool_use block missing 'id'".into()))?;
            let name = block
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| LLMError::Api("tool_use block missing 'name'".into()))?;
            let arguments = block
                .get("input")
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            Ok(ContentBlock::ToolCall(ToolCall {
                id: id.to_string(),
                name: name.to_string(),
                arguments,
            }))
        }
        other => Err(LLMError::Api(format!(
            "unknown content block type: {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lithify_core::ToolResult;

    // -- messages_to_request tests -------------------------------------------

    #[test]
    fn simple_text_message() {
        let msgs = [Message {
            role: Role::User,
            content: vec![ContentBlock::Text("hello".into())],
        }];
        let req = messages_to_request("claude", 100, &msgs);

        assert_eq!(req["model"], "claude");
        assert_eq!(req["max_tokens"], 100);
        assert_eq!(req["messages"][0]["role"], "user");
        assert_eq!(req["messages"][0]["content"][0]["type"], "text");
        assert_eq!(req["messages"][0]["content"][0]["text"], "hello");
    }

    #[test]
    fn system_message_becomes_top_level_field() {
        let msgs = vec![
            Message {
                role: Role::System,
                content: vec![ContentBlock::Text("You are helpful.".into())],
            },
            Message {
                role: Role::User,
                content: vec![ContentBlock::Text("hi".into())],
            },
        ];
        let req = messages_to_request("claude", 100, &msgs);

        assert_eq!(req["system"], "You are helpful.");
        assert_eq!(req["messages"].as_array().unwrap().len(), 1);
        assert_eq!(req["messages"][0]["role"], "user");
    }

    #[test]
    fn tool_call_message() {
        let msgs = [Message {
            role: Role::Assistant,
            content: vec![
                ContentBlock::Text("I'll check.".into()),
                ContentBlock::ToolCall(ToolCall {
                    id: "tc_1".into(),
                    name: "read_file".into(),
                    arguments: serde_json::json!({"path": "/tmp/test"}),
                }),
            ],
        }];
        let req = messages_to_request("claude", 100, &msgs);

        let blocks = req["messages"][0]["content"].as_array().unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0]["type"], "text");
        assert_eq!(blocks[1]["type"], "tool_use");
        assert_eq!(blocks[1]["id"], "tc_1");
        assert_eq!(blocks[1]["input"]["path"], "/tmp/test");
    }

    #[test]
    fn tool_result_message_success() {
        let msgs = [Message {
            role: Role::User,
            content: vec![ContentBlock::ToolResult(ToolResult {
                tool_call_id: "tc_1".into(),
                content: "file contents".into(),
                is_error: false,
            })],
        }];
        let req = messages_to_request("claude", 100, &msgs);

        let block = &req["messages"][0]["content"][0];
        assert_eq!(block["type"], "tool_result");
        assert_eq!(block["tool_use_id"], "tc_1");
        assert_eq!(block["content"], "file contents");
        assert_eq!(block["is_error"], false);
    }

    #[test]
    fn tool_result_message_error() {
        let msgs = [Message {
            role: Role::User,
            content: vec![ContentBlock::ToolResult(ToolResult {
                tool_call_id: "tc_1".into(),
                content: "Error: file not found".into(),
                is_error: true,
            })],
        }];
        let req = messages_to_request("claude", 100, &msgs);

        let block = &req["messages"][0]["content"][0];
        assert_eq!(block["is_error"], true);
    }

    #[test]
    fn multiple_system_messages_concatenated() {
        let msgs = vec![
            Message {
                role: Role::System,
                content: vec![ContentBlock::Text("You are helpful.".into())],
            },
            Message {
                role: Role::System,
                content: vec![ContentBlock::Text("Respond in JSON.".into())],
            },
            Message {
                role: Role::User,
                content: vec![ContentBlock::Text("hi".into())],
            },
        ];
        let req = messages_to_request("claude", 100, &msgs);

        assert!(req["system"].as_str().unwrap().contains("You are helpful."));
        assert!(req["system"].as_str().unwrap().contains("Respond in JSON."));
        assert_eq!(req["messages"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn multiple_messages_ordered() {
        let msgs = vec![
            Message {
                role: Role::User,
                content: vec![ContentBlock::Text("hello".into())],
            },
            Message {
                role: Role::Assistant,
                content: vec![ContentBlock::Text("world".into())],
            },
        ];
        let req = messages_to_request("claude", 100, &msgs);

        assert_eq!(req["messages"].as_array().unwrap().len(), 2);
        assert_eq!(req["messages"][0]["role"], "user");
        assert_eq!(req["messages"][1]["role"], "assistant");
    }

    // -- response_from_value tests ------------------------------------------

    #[test]
    fn simple_text_response() {
        let json = serde_json::json!({
            "content": [{"type": "text", "text": "Hello!"}],
            "usage": {"input_tokens": 10, "output_tokens": 5},
        });

        let resp = response_from_value(json).unwrap();
        assert_eq!(resp.content.len(), 1);
        assert!(matches!(&resp.content[0], ContentBlock::Text(t) if t == "Hello!"));
        assert_eq!(resp.usage.input_tokens, 10);
        assert_eq!(resp.usage.output_tokens, 5);
    }

    #[test]
    fn response_with_tool_use() {
        let json = serde_json::json!({
            "content": [
                {"type": "text", "text": "Let me check."},
                {"type": "tool_use", "id": "tc_1", "name": "read_file", "input": {"path": "/tmp/x"}}
            ],
            "usage": {"input_tokens": 20, "output_tokens": 15},
        });

        let resp = response_from_value(json).unwrap();
        assert_eq!(resp.content.len(), 2);
        assert!(matches!(resp.content[0], ContentBlock::Text(_)));
        assert!(matches!(&resp.content[1], ContentBlock::ToolCall(tc) if tc.id == "tc_1"));
    }

    #[test]
    fn response_missing_content() {
        let json = serde_json::json!({"usage": {"input_tokens": 1, "output_tokens": 1}});
        let err = response_from_value(json).unwrap_err();
        assert!(err.to_string().contains("content"));
    }

    #[test]
    fn response_missing_usage() {
        let json = serde_json::json!({"content": [{"type": "text", "text": "hi"}]});
        let err = response_from_value(json).unwrap_err();
        assert!(err.to_string().contains("usage"));
    }

    #[test]
    fn response_unknown_block_type() {
        let json = serde_json::json!({
            "content": [{"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": "..."}}],
            "usage": {"input_tokens": 1, "output_tokens": 1},
        });
        let err = response_from_value(json).unwrap_err();
        assert!(err.to_string().contains("unknown content block type"));
    }

    // -- Roundtrip test ------------------------------------------------------

    #[test]
    fn roundtrip_text_only() {
        let msgs = [Message {
            role: Role::User,
            content: vec![ContentBlock::Text("hello".into())],
        }];
        let _req = messages_to_request("claude", 100, &msgs);

        let response_json = serde_json::json!({
            "content": [{"type": "text", "text": "Hi there!"}],
            "usage": {"input_tokens": 5, "output_tokens": 3},
        });

        let resp = response_from_value(response_json).unwrap();
        assert!(matches!(&resp.content[0], ContentBlock::Text(t) if t == "Hi there!"));
        assert_eq!(resp.usage.input_tokens, 5);
    }
}
