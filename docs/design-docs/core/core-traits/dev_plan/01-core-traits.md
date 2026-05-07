# 任务 1: 定义 Core Traits

## 目标

在 `crates/core/` 中定义 Phase 1 所有 crate 共享的 trait 接口和数据类型。

`lithify-core` 是纯接口层，零外部依赖。它不实现任何逻辑，只声明"什么是一个 LLM 客户端""什么是一个 Skill 加载器""什么是一个 Tool"——让上层 crate 各自实现，互不耦合。

对应架构层：**跨层基础设施**（所有层都依赖 core 的类型和接口）。

## 涉及 Crate

| Crate | 职责 |
|-------|------|
| `core` | 定义 trait + 关联类型 + 错误类型 |

## 需要定义的 Trait

### 1. `LLMClient`

LLM API 调用抽象。Phase 1 只关注请求/响应式调用，不包含 streaming。

```rust
#[async_trait]
pub trait LLMClient {
    /// 发送消息列表到 LLM，返回响应。
    /// 响应可能包含文本内容和/或 tool call 请求。
    async fn chat(&self, messages: &[Message]) -> Result<Response, LLMError>;
}
```

### 2. `SkillLoader`

从文件系统加载 Skill 文件。

```rust
pub trait SkillLoader {
    /// 按名称加载一个 Skill 的完整内容。
    fn load(&self, name: &str) -> Result<Skill, SkillError>;
    /// 列出所有可用 Skill 的元数据（不含 content）。
    fn list(&self) -> Result<Vec<SkillMeta>, SkillError>;
}
```

### 3. `SkillStore`

Skill 文件的持久化 CRUD。Agent 通过此接口创建/修改/删除 Skill 文件。

```rust
pub trait SkillStore {
    fn save(&self, skill: &Skill) -> Result<(), SkillError>;
    fn delete(&self, name: &str) -> Result<(), SkillError>;
}
```

### 4. `Tool`

单个工具的定义和执行。

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value;
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, ToolError>;
}
```

### 5. `ToolRegistry`

Tool 注册表，管理已注册的 Tool 实例。

```rust
pub trait ToolRegistry {
    fn register(&mut self, tool: Arc<dyn Tool>);
    fn find(&self, name: &str) -> Option<Arc<dyn Tool>>;
    fn list(&self) -> Vec<Arc<dyn Tool>>;
}
```

### 6. `Sandbox`

子进程沙箱，用于执行 Shell 命令。

```rust
#[async_trait]
pub trait Sandbox {
    /// 在沙箱中执行命令，返回执行结果。
    async fn run(&self, command: &str, args: &[String], working_dir: Option<&Path>) -> Result<ExecutionResult, SandboxError>;
}
```

## 关联类型

### `Message`

LLM 对话中的单条消息。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentBlock {
    Text(String),
    ToolCall(ToolCall),
    ToolResult(ToolResult),
}
```

### `Response`

LLM 返回的响应。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub content: Vec<ContentBlock>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}
```

### `ToolCall` / `ToolResult`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub content: String,
    pub is_error: bool,
}
```

### `Skill` / `SkillMeta`

对应 Anthropic 标准 skill 格式。`SkillMeta` 不含 content，仅用于列表展示。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMeta {
    pub name: String,
    pub description: String,
}
```

### `ExecutionResult`

沙箱执行命令后的结果。

```rust
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}
```

## 错误类型

三种错误类型，使用 `thiserror` 派生。

```rust
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
```

## 依赖关系

`core` 不依赖任何 workspace 内的其他 crate。

外部依赖（仅限必要的生态基础）：

| 依赖 | 用途 |
|------|------|
| `async-trait` | `LLMClient`、`Tool`、`Sandbox` trait 的 async fn 支持 |
| `serde` + `serde_json` | `Message`、`Response`、`Skill` 等类型的序列化 |
| `thiserror` | 错误类型派生 |

## 验收标准

1. `cargo check -p lithify-core` 通过（编译通过，无警告）
2. 所有 pub trait 和 pub 类型有英文 `///` doc comment
3. 零外部运行时依赖（无 tokio、无 reqwest、无数据库驱动）
4. 类型定义能被后续 crate 直接使用，无需重复定义

## 任务编号

**#1** — 主任务，编号 01
