# Phase 1 Roadmap: Skeleton

## 目标

一个可用的 CLI Agent 对话循环，具有：
- LLM 集成（Anthropic API）
- 基础工具调用（文件读写、Shell 执行）
- Skill 系统（加载、注入、自我修改）

对应架构层：**L2 Skill（短期记忆）就绪**，L3/RAG 和 L4/Tool 固化留到后续 Phase。

## 涉及 Crate

| Crate | Phase 1 职责 |
|-------|-------------|
| `core` | 定义 Phase 1 所需 trait：`LLMClient`、`SkillLoader`、`SkillStore`、`Tool`、`ToolRegistry`、`Sandbox` |
| `llm` | 实现 Anthropic Claude API 调用 |
| `sandbox` | 子进程沙箱，供 Shell 工具使用 |
| `tools` | 基础工具：`read_file`、`write_file`、`shell_exec` |
| `skill` | Skill 文件加载、解析 frontmatter、注入 system prompt |
| `runtime` | CLI shell + 对话循环，组装所有组件 |

`storage` 和 `rag` 属于 Phase 2，Phase 1 不涉及。

## 任务列表

### 任务 1: 定义 Core Traits

**文件**: `crates/core/src/lib.rs`

定义 Phase 1 所有共享接口：

- `LLMClient` trait — `async fn chat(messages) -> Response`
- `SkillLoader` trait — `fn load(name) -> Skill`
- `SkillStore` trait — `fn save(skill)`, `fn delete(name)`, `fn list() -> Vec<SkillMeta>`
- `Tool` trait — `fn name()`, `fn description()`, `fn execute(args) -> Result`
- `ToolRegistry` trait — `fn register(tool)`, `fn find(name)`, `fn list()`
- `Sandbox` trait — `async fn run(command, args) -> ExecutionResult`

以及关联类型：
- `Message`、`Response`、`ToolCall`、`ToolResult`
- `Skill` — name, description, content
- `SkillMeta` — name, description（不含 content，用于列表）
- `ExecutionResult` — exit_code, stdout, stderr
- 错误类型：`LLMError`、`ToolError`、`SandboxError`

**验收**: `cargo check -p lithify-core` 通过

---

### 任务 2: LLM 客户端实现

**文件**: `crates/llm/src/lib.rs`、`crates/llm/src/anthropic.rs`

实现 `LLMClient` trait，对接 Anthropic Messages API：
- API Key 从环境变量或配置读取
- 发送 messages，返回 Response（含文本 + tool_calls）
- 支持 streaming（Phase 1 可 skip，后续再加）

**关键点**:
- 依赖 `anthropic` SDK crate
- 需要处理 rate limit、超时、网络错误
- Tool call 的请求/响应格式转换

**验收**: `cargo test -p lithify-llm` 通过（mock Anthropic API）

---

### 任务 3: Sandbox 实现

**文件**: `crates/sandbox/src/lib.rs`

实现 `Sandbox` trait：
- `tokio::process::Command` 启动子进程
- 捕获 stdout、stderr、exit code
- 超时控制
- 可选的工作目录和环境变量隔离

**验收**: `cargo test -p lithify-sandbox` 通过

---

### 任务 4: 基础工具实现

**文件**: `crates/tools/src/read_file.rs`、`crates/tools/src/write_file.rs`、`crates/tools/src/shell.rs`、`crates/tools/src/registry.rs`

实现三个基础工具和注册表：
- **ReadFileTool** — 读取文件内容，参数：`path`
- **WriteFileTool** — 写入文件，参数：`path`, `content`
- **ShellTool** — 执行 Shell 命令（通过 Sandbox），参数：`command`, `args`

以及 `ToolRegistry` 的默认实现。

**验收**: `cargo test -p lithify-tools` 通过

---

### 任务 5: Skill 系统实现

**文件**: `crates/skill/src/lib.rs`、`crates/skill/src/loader.rs`、`crates/skill/src/parser.rs`

实现 Skill 加载、解析、注入：

1. **Skill 目录结构**: `skills/` 下的 Markdown 文件，使用 Anthropic 标准 skill 格式（YAML frontmatter）：
   ```markdown
   ---
   name: deploy-cloudflare
   description: Deploy to Cloudflare Pages. Use when the user asks to deploy, publish to Cloudflare, or set up Pages hosting.
   ---
   # Deploy to Cloudflare Pages
   ...steps...
   ```

2. **SkillLoader**: 扫描 `skills/` 目录，按需加载
3. **SkillInjector**: 将匹配的 Skill 全文注入 system prompt
4. **SkillStore**: 支持 Agent 通过工具创建/修改/删除 Skill 文件

**验收**: `cargo test -p lithify-skill` 通过

---

### 任务 6: Runtime CLI + 对话循环

**文件**: `crates/runtime/src/main.rs`、`crates/runtime/src/loop.rs`、`crates/runtime/src/prompt.rs`

组装所有组件，实现对话循环：

1. **CLI shell**: clap 解析参数，REPL 循环
2. **对话循环**:
   ```
   loop:
     1. 接收用户输入
     2. 检索匹配的 Skill → 注入 system prompt
     3. 组装 messages（system + history + user）
     4. 调用 LLMClient.chat()
     5. 如 LLM 返回 tool_call → Sandbox 执行 → 结果回注 messages
     6. 输出 LLM 文本响应
   ```
3. **System prompt 模板**: 包含 Tool 列表、Skill 内容、基础行为指令
4. **对话历史管理**: 维护消息列表，控制 context 窗口不溢出

**验收**: `cargo test -p lithify-runtime` 通过（mock 所有依赖的端到端测试）

---

### 任务 7: 端到端示例

**文件**: `examples/hello-agent/`

一个可实际运行的示例：
- 使用真实 Anthropic API
- `skills/` 目录下有示例 Skill
- 展示一次完整的"用户提问 → Skill 注入 → LLM 推理 → 工具调用"流程

**验收**: `cargo run --example hello-agent` 正常工作

---

## 依赖顺序

```
任务 1: core traits
    ├── 任务 2: llm
    ├── 任务 3: sandbox
    │       └── 任务 4: tools
    ├── 任务 5: skill (可并行)
    └── 以上全部 → 任务 6: runtime → 任务 7: 示例
```

- 任务 2、3、5 可以在任务 1 完成后并行开发
- 任务 4 依赖任务 3
- 任务 6 依赖 2、4、5 全部完成
- 任务 7 依赖任务 6

## 开发流程

每个任务遵循 `docs/AI_WORKFLOW.md`:

1. **TDD Phase 1**: 写测试 → `test: add tests for #N (<feature>)` → 人工审查
2. **TDD Phase 2**: 实现 → `feat: implement #N (<feature>)` → 人工审查
3. `cargo test && cargo clippy && cargo fmt --check` 全部通过
4. AI Self-Review → 人工 Review → Merge

## 进度跟踪

| 任务 | 状态 | 分支 |
|------|------|------|
| 1. Core Traits | 🚧 | `feat/01-core-traits` |
| 2. LLM Client | ⬜ | `feat/02-llm-client` |
| 3. Sandbox | ⬜ | `feat/03-sandbox` |
| 4. Basic Tools | ⬜ | `feat/04-basic-tools` |
| 5. Skill System | ⬜ | `feat/05-skill-system` |
| 6. Runtime | ⬜ | `feat/06-runtime` |
| 7. Example | ⬜ | `feat/07-example` |
