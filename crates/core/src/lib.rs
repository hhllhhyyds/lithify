// Core traits and types for Lithify.
// Zero external runtime dependencies — this crate defines all shared interfaces.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Message types for LLM conversation
// ---------------------------------------------------------------------------

/// A single message in an LLM conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentBlock>,
}

/// The role of a message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
}

/// A block of content within a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentBlock {
    Text(String),
    ToolCall(ToolCall),
    ToolResult(ToolResult),
}

/// LLM response returned by [`LLMClient::chat`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub content: Vec<ContentBlock>,
    pub usage: Usage,
}

/// Token usage statistics for an LLM API call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// A tool call requested by the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// The result of executing a tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub content: String,
    pub is_error: bool,
}

// ---------------------------------------------------------------------------
// Skill types (Anthropic standard skill format)
// ---------------------------------------------------------------------------

/// A Skill — a precise, executable instruction document.
///
/// Corresponds to the Anthropic standard skill format: YAML frontmatter with
/// `name` and `description`, followed by markdown content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub content: String,
}

/// Metadata for a Skill, without the full content.
/// Used for listing available skills.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMeta {
    pub name: String,
    pub description: String,
}

// ---------------------------------------------------------------------------
// Sandbox types
// ---------------------------------------------------------------------------

/// Result of executing a command in the sandbox.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum LLMError {
    #[error("API error: {0}")]
    Api(String),
    #[error("Rate limited")]
    RateLimited,
    #[error("Network error: {0}")]
    Network(String),
}

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Tool execution failed: {0}")]
    Execution(String),
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),
}

#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("Command failed: {0}")]
    Command(String),
    #[error("Timeout after {0}ms")]
    Timeout(u64),
}

#[derive(Debug, Error)]
pub enum SkillError {
    #[error("Skill not found: {0}")]
    NotFound(String),
    #[error("Invalid skill format: {0}")]
    InvalidFormat(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// Traits
// ---------------------------------------------------------------------------

/// Abstract interface for calling an LLM.
#[async_trait]
pub trait LLMClient {
    /// Send messages to the LLM and return the response.
    async fn chat(&self, messages: &[Message]) -> Result<Response, LLMError>;
}

/// Loads Skill files from the filesystem.
pub trait SkillLoader {
    /// Load a Skill by name, returning its full content.
    fn load(&self, name: &str) -> Result<Skill, SkillError>;
    /// List all available Skills (metadata only, no content).
    fn list(&self) -> Result<Vec<SkillMeta>, SkillError>;
}

/// Persists Skill files. Used by the agent to create, update, or delete skills.
pub trait SkillStore {
    /// Save a Skill to the filesystem, creating or overwriting it.
    fn save(&self, skill: &Skill) -> Result<(), SkillError>;
    /// Delete a Skill by name from the filesystem.
    fn delete(&self, name: &str) -> Result<(), SkillError>;
}

/// A tool that can be invoked by the LLM.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Unique name used to invoke this tool.
    fn name(&self) -> &str;
    /// Human-readable description of what the tool does.
    fn description(&self) -> &str;
    /// JSON Schema describing the tool's parameters.
    fn parameters(&self) -> serde_json::Value;
    /// Execute the tool with the given arguments.
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, ToolError>;
}

/// Registry of available tools.
pub trait ToolRegistry {
    /// Register a tool instance.
    fn register(&mut self, tool: Arc<dyn Tool>);
    /// Find a tool by name.
    fn find(&self, name: &str) -> Option<Arc<dyn Tool>>;
    /// List all registered tools.
    fn list(&self) -> Vec<Arc<dyn Tool>>;
}

/// Sandbox for executing shell commands in a subprocess.
#[async_trait]
pub trait Sandbox {
    /// Run a command with arguments, optionally in a specific working directory.
    async fn run(
        &self,
        command: &str,
        args: &[String],
        working_dir: Option<&Path>,
    ) -> Result<ExecutionResult, SandboxError>;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Type construction tests -------------------------------------------

    #[test]
    fn skill_construction() {
        let skill = Skill {
            name: "deploy".into(),
            description: "Deploy instructions".into(),
            content: "# Deploy\n...".into(),
        };
        assert_eq!(skill.name, "deploy");
    }

    #[test]
    fn skill_meta_construction() {
        let meta = SkillMeta {
            name: "deploy".into(),
            description: "Deploy instructions".into(),
        };
        assert_eq!(meta.name, "deploy");
    }

    #[test]
    fn skill_meta_no_content_field() {
        let meta = SkillMeta {
            name: "deploy".into(),
            description: "Deploy instructions".into(),
        };
        // SkillMeta must not have a content field — this is the key difference from Skill.
        assert_eq!(meta.description, "Deploy instructions");
    }

    #[test]
    fn execution_result_construction() {
        let result = ExecutionResult {
            exit_code: 0,
            stdout: "hello".into(),
            stderr: "".into(),
        };
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn message_construction() {
        let msg = Message {
            role: Role::User,
            content: vec![ContentBlock::Text("hello".into())],
        };
        assert!(matches!(msg.role, Role::User));
    }

    #[test]
    fn response_construction() {
        let resp = Response {
            content: vec![ContentBlock::Text("hi".into())],
            usage: Usage {
                input_tokens: 10,
                output_tokens: 5,
            },
        };
        assert_eq!(resp.usage.input_tokens, 10);
    }

    #[test]
    fn tool_call_construction() {
        let tc = ToolCall {
            id: "call_1".into(),
            name: "read_file".into(),
            arguments: serde_json::json!({"path": "/tmp/test"}),
        };
        assert_eq!(tc.name, "read_file");
    }

    #[test]
    fn tool_result_construction() {
        let tr = ToolResult {
            tool_call_id: "call_1".into(),
            content: "file contents".into(),
            is_error: false,
        };
        assert!(!tr.is_error);
        assert_eq!(tr.content, "file contents");
    }

    // -- Serde roundtrip tests ---------------------------------------------

    #[test]
    fn skill_serde_roundtrip() {
        let skill = Skill {
            name: "test-skill".into(),
            description: "A test skill".into(),
            content: "# Hello\nWorld".into(),
        };
        let json = serde_json::to_string(&skill).unwrap();
        let decoded: Skill = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, "test-skill");
    }

    #[test]
    fn message_serde_roundtrip() {
        let msg = Message {
            role: Role::Assistant,
            content: vec![
                ContentBlock::Text("Sure!".into()),
                ContentBlock::ToolCall(ToolCall {
                    id: "tc_1".into(),
                    name: "shell".into(),
                    arguments: serde_json::json!({"cmd": "ls"}),
                }),
            ],
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: Message = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded.role, Role::Assistant));
    }

    #[test]
    fn response_serde_roundtrip() {
        let resp = Response {
            content: vec![ContentBlock::Text("done".into())],
            usage: Usage {
                input_tokens: 100,
                output_tokens: 50,
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        let decoded: Response = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.usage.output_tokens, 50);
    }

    #[test]
    fn skill_meta_serde_has_no_content() {
        let meta = SkillMeta {
            name: "s".into(),
            description: "d".into(),
        };
        let json = serde_json::to_string(&meta).unwrap();
        // SkillMeta JSON must NOT contain the word "content"
        assert!(!json.contains("content"));
    }

    // -- Error Display tests -----------------------------------------------

    #[test]
    fn llm_error_display() {
        let err = LLMError::Api("bad gateway".into());
        assert_eq!(err.to_string(), "API error: bad gateway");
    }

    #[test]
    fn tool_error_display() {
        let err = ToolError::Execution("permission denied".into());
        assert_eq!(err.to_string(), "Tool execution failed: permission denied");
    }

    #[test]
    fn sandbox_error_display() {
        let err = SandboxError::Timeout(5000);
        assert_eq!(err.to_string(), "Timeout after 5000ms");
    }

    #[test]
    fn skill_error_display() {
        let err = SkillError::NotFound("deploy".into());
        assert_eq!(err.to_string(), "Skill not found: deploy");
    }

    #[test]
    fn skill_error_io_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let skill_err: SkillError = io_err.into();
        assert_eq!(skill_err.to_string(), "IO error: file missing");
    }

    // -- Trait implementation tests (mock structs) ------------------------

    struct MockTool;
    #[async_trait]
    impl Tool for MockTool {
        fn name(&self) -> &str {
            "mock"
        }
        fn description(&self) -> &str {
            "a mock tool"
        }
        fn parameters(&self) -> serde_json::Value {
            serde_json::json!({})
        }
        async fn execute(&self, _args: serde_json::Value) -> Result<ToolResult, ToolError> {
            Ok(ToolResult {
                tool_call_id: "x".into(),
                content: "ok".into(),
                is_error: false,
            })
        }
    }

    #[tokio::test]
    async fn mock_tool_execute() {
        let tool = MockTool;
        let result = tool.execute(serde_json::json!({})).await.unwrap();
        assert_eq!(result.content, "ok");
    }

    struct MockLLMClient;
    #[async_trait]
    impl LLMClient for MockLLMClient {
        async fn chat(&self, _messages: &[Message]) -> Result<Response, LLMError> {
            Ok(Response {
                content: vec![ContentBlock::Text("mock response".into())],
                usage: Usage {
                    input_tokens: 1,
                    output_tokens: 1,
                },
            })
        }
    }

    #[tokio::test]
    async fn mock_llm_client_chat() {
        let client = MockLLMClient;
        let msgs = [Message {
            role: Role::User,
            content: vec![ContentBlock::Text("hi".into())],
        }];
        let resp = client.chat(&msgs).await.unwrap();
        assert!(matches!(&resp.content[0], ContentBlock::Text(t) if t == "mock response"));
    }
}
