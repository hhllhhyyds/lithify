---
date: 2026-05-13
type: project
status: planning
tags: [project, lithify, phase1, spec, mvp, rust]
ai-first: true
---

## For future Claude
lithify Phase 1 最小原型规格文档，由 hhl 于 2026-05-13 确定。目标是用最小实现验证 lithify 架构的三个核心假设（H1 context 分层管理、H2 Ring 隔离防注入、H3 DAI 统一数据接口），而不是实现完整设计。使用 Rust 实现，预计 1200~1500 行核心代码，1~2 周完成。完整架构详见 [[Overview]]。

---

## 目标

> 用最小实现验证架构的核心假设，而不是实现完整设计。

### 三个核心假设

| 假设 | 内容 |
|---|---|
| **H1** | 显式 context 分层管理（I/D/W segment + 智能 eviction）比 FIFO 截断更有效 |
| **H2** | Ring 3 隔离 + tool 结果验证层，能实际拦截 prompt injection |
| **H3** | 统一的数据访问接口（DAI）让 skill 开发更简单、行为更可预期 |

---

## 主要依赖

```toml
# Cargo.toml
[dependencies]
tokio        = { version = "1", features = ["full"] }   # 异步运行时
reqwest      = { version = "0.12", features = ["json"] } # Claude API HTTP 调用
serde        = { version = "1", features = ["derive"] }  # 序列化
serde_json   = "1"
anyhow       = "1"                                        # 错误处理
async-trait  = "0.1"                                      # async fn in traits
tiktoken-rs  = "0.5"                                      # token 计数
```

---

## Phase 1 实现范围

### 必须实现

#### 1. Harness 内核骨架（Ring 0）

职责：
- 接收用户输入，分发给对应 skill（Ring 3）
- 代理所有 tool 调用，skill 不直接调用任何外部工具
- 维护 agent 注册表（简化版，记录 id、skill、ring、状态）
- 执行基础 ring 分级：区分 Ring 0（harness 自身）和 Ring 3（task agent）

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Ring { Kernel = 0, System = 1, User = 3 }

pub struct AgentHarness {
    registry: HashMap<Uuid, AgentEntry>,
    context:  ContextManager,
    validator: ToolResultValidator,
    claude:   ClaudeClient,
}

impl AgentHarness {
    pub async fn run(&mut self, skill: &dyn Skill, task: &str) -> anyhow::Result<String>;
    pub async fn dispatch_tool(&self, name: &str, args: Value) -> anyhow::Result<ToolResult>;
}
```

#### 2. Context Manager（验证 H1）

职责：
- 维护三段 context：I-segment（行为规则）、D-segment（任务数据）、W-segment（工作区）
- 追踪每段的 token 占用和总压力
- 实现可对比的 eviction 策略（至少两种）

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Segment { ISegment, DSegment, WSegment }

pub struct ContextBlock {
    pub content:     String,
    pub tokens:      usize,
    pub segment:     Segment,
    pub created_at:  Instant,
    pub last_access: Instant,
}

pub struct ContextManager {
    blocks:     Vec<ContextBlock>,
    max_tokens: usize,
}

impl ContextManager {
    pub fn add(&mut self, content: String, segment: Segment) -> anyhow::Result<()>;
    pub fn evict(&mut self, policy: &dyn EvictionPolicy);
    pub fn pressure(&self) -> f32;   // 0.0 ~ 1.0
    pub fn render(&self) -> String;  // 输出给 LLM 的完整 context
    pub fn token_count(&self) -> usize;
}

// Eviction 策略作为 trait，便于对比实验
pub trait EvictionPolicy: Send + Sync {
    fn evict(&self, blocks: &mut Vec<ContextBlock>, target_tokens: usize);
}

pub struct FifoPolicy;    // baseline：丢最老的
pub struct LruPolicy;     // 丢最久未访问的
pub struct SemanticPolicy { pub task_embedding: Vec<f32> }  // 丢语义最不相关的
```

**Eviction 策略对比实验**：三种策略都实现 `EvictionPolicy` trait，通过统一接口切换，便于 H1 实验。

#### 3. Tool 结果验证层（验证 H2）

所有 tool 返回值在注入 context 前必须经过验证层：

```rust
#[derive(Debug, Clone)]
pub enum TrustLevel { Trusted, Sandboxed }

pub struct ValidatedResult {
    pub content:    String,
    pub trust:      TrustLevel,
    pub truncated:  bool,
    pub flagged:    bool,
    pub flag_reason: Option<String>,
}

pub struct ToolResultValidator {
    pub max_chars:        usize,
    pub injection_patterns: Vec<Regex>,
}

impl ToolResultValidator {
    pub fn validate(&self, result: ToolResult) -> ValidatedResult;
    // 检查 1：大小限制（超限截断，附加截断说明，不静默丢弃）
    // 检查 2：来源信任级别标注
    // 检查 3：注入检测（匹配 system prompt / instruction override 等特征）
}
```

验证失败行为：
- **大小超限**：截断 + 附加 `[TRUNCATED: original {n} chars]`
- **注入检测命中**：`flagged = true`，附加 `[WARNING: potential injection detected]`，不阻止但标记
- 所有拦截事件写入 tool call 日志

#### 4. Data Access Interface 最小版（验证 H3）

**核心思想**：skill 声明"我要什么、这个结果留多久"，harness 决定从哪取、存到哪。

```rust
#[derive(Debug, Clone)]
pub enum Scope { Task, Session, Vault, Auto }

#[derive(Debug, Clone)]
pub enum Durability { Task, Persistent }

pub struct DataAccessInterface {
    task_store:    HashMap<String, String>,   // TASK 级：task 结束释放
    session_store: HashMap<String, String>,   // SESSION 级：session 内有效
    harness:       Arc<AgentHarness>,
}

impl DataAccessInterface {
    pub async fn read(&self, key: &str, scope: Scope) -> Option<String>;
    // Auto 查找顺序：task_store → session_store → vault（read_file tool）

    pub async fn write(&self, key: &str, value: String, durability: Durability)
        -> anyhow::Result<()>;
    // Task       → task_store（内存）
    // Persistent → 调用 write_file tool 存 vault
}
```

Phase 1 存储后端：
- `Task / Session` → `HashMap<String, String>`（内存）
- `Vault` → 实际调用 `read_file` / `write_file` tool

#### 5. Demo Skills × 2

Skill 作为 trait，每个实现在 Ring 3 中运行：

```rust
#[async_trait]
pub trait Skill: Send + Sync {
    fn name(&self) -> &str;
    fn ring(&self) -> Ring { Ring::User }
    async fn run(&self, task: &str, dai: &DataAccessInterface) -> anyhow::Result<String>;
}
```

**Skill A：`SummarizeSkill`（无外部 tool，测试 DAI 读写）**

```rust
pub struct SummarizeSkill;

#[async_trait]
impl Skill for SummarizeSkill {
    fn name(&self) -> &str { "summarize" }

    async fn run(&self, task: &str, dai: &DataAccessInterface) -> anyhow::Result<String> {
        // 读：项目背景（透明地从 context 或 vault 取）
        let background = dai.read("project.background", Scope::Auto).await;
        // ... 调用 Claude API 做摘要 ...
        // 写：结果持久化到 vault
        dai.write("summaries.latest", result.clone(), Durability::Persistent).await?;
        Ok(result)
    }
}
```

**Skill B：`WebResearcherSkill`（涉及 tool，测试验证层）**

```rust
pub struct WebResearcherSkill;

#[async_trait]
impl Skill for WebResearcherSkill {
    fn name(&self) -> &str { "web_researcher" }

    async fn run(&self, task: &str, dai: &DataAccessInterface) -> anyhow::Result<String> {
        // tool 调用经由 harness → 结果经验证层 → 才能使用
        // 测试时注入恶意 tool 返回内容，验证验证层能拦截
        dai.write("research.latest", results.clone(), Durability::Task).await?;
        Ok(results)
    }
}
```

#### 6. 基础可观测性

每次 task 结束打印 context 状态报告：

```
=== Context Status ===
I-segment:  2,048 tokens  (10.2%)
D-segment:  8,192 tokens  (40.9%)
W-segment:  3,841 tokens  (19.2%)
──────────────────────
Total:     14,081 / 20,000 tokens  (70.4%) [NORMAL]

=== Tool Call Log ===
[14:23:01] web_search       → OK       1,240ms  2,341 chars  trusted
[14:23:05] read_file        → OK          11ms  4,832 chars  trusted
[14:23:09] web_search       → FLAGGED   891ms  1,204 chars  [injection detected]
```

---

### 明确不做（Phase 2+）

| 模块 | 推迟原因 |
|---|---|
| IPC / Event Bus | 复杂，单 agent 足够验证核心假设 |
| Signal 系统 | 用 `tokio::time::timeout` 简单代替 |
| 自进化 | 不影响核心架构验证 |
| 包管理器 | 手动安装 skill 即可 |
| 完整 capability 安全模型 | 只需基础 ring 分级 |
| Init 序列 | 硬编码启动配置 |
| 多 agent NUMA 协调 | Phase 1 以单 agent 为主 |
| Virtual Context CoW | 简单引用传递代替 |
| Stable ABI 版本管理 | 接口先定死，版本化是 Phase 2 的事 |

---

## 验收标准

| 假设 | 验收实验 | 通过条件 |
|---|---|---|
| H1 | 同一个 long-context 任务分别用 FIFO / LRU / SEMANTIC eviction 跑 | SEMANTIC 或 LRU 的任务完成质量 ≥ FIFO，且 context 利用率提升 |
| H2 | 构造一个 tool 返回值含恶意指令的 prompt injection 样本 | 验证层能标记/拦截，injection 不影响 LLM 输出行为 |
| H3 | 对比：用 DAI 写 `SummarizeSkill` vs 不用 DAI 写同等功能 | 代码量减少，行为（数据在哪、存多久）更可预期 |

---

## 项目结构

```
lithify/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── harness/
│   │   ├── mod.rs
│   │   ├── core.rs          # AgentHarness（Ring 0）
│   │   ├── context.rs       # ContextManager + EvictionPolicy trait
│   │   ├── eviction.rs      # FifoPolicy / LruPolicy / SemanticPolicy
│   │   ├── validator.rs     # ToolResultValidator
│   │   └── dai.rs           # DataAccessInterface
│   ├── skills/
│   │   ├── mod.rs           # Skill trait 定义
│   │   ├── summarize.rs     # Demo Skill A
│   │   └── web_researcher.rs # Demo Skill B
│   ├── tools/
│   │   └── registry.rs      # 可用 tool 注册表
│   └── claude/
│       └── client.rs        # Claude API 封装
└── tests/
    ├── eviction_test.rs     # H1 对比实验
    ├── injection_test.rs    # H2 注入拦截测试
    └── dai_test.rs          # H3 DAI 接口测试
```

---

## 规模估计

| 组件 | 预估代码量 |
|---|---|
| Harness 骨架 + Ring 分级 | ~200 行 |
| Context Manager + 类型定义 | ~200 行 |
| Eviction 策略（3 种） | ~150 行 |
| Tool 验证层 | ~120 行 |
| DAI 最小版 | ~150 行 |
| Demo Skills × 2 | ~150 行 |
| Claude API 封装 | ~150 行 |
| 可观测性日志 | ~80 行 |
| **合计** | **~1200 行** |

预计实现时间：1~2 周。

---

## 下一步

1. ~~确认 tool 接入方式~~ → **已决策：Phase 1 用 Claude Tool Use 直接 API**
2. 开始实现 `src/harness/core.rs`

---

## 决策记录

### [2026-05-13] Tool 接入方式：Claude Tool Use 直接 API

**决策**：Phase 1 使用 Claude Tool Use（直接 API），而非 MCP。

**两种方式的核心区别**：
- **Tool Use 直接 API**：tool 逻辑在 Rust Harness 里实现，Claude 返回 `tool_call` 指令，Harness 执行后把结果传回 Claude
- **MCP**：tool 逻辑跑在独立的 MCP server 进程里，Harness 通过协议调用，支持热插拔和社区生态

**选择 Tool Use 的原因**：
1. Phase 1 只有两个 Demo Skill，tool 数量少（web_search、read_file、write_file），不需要 MCP 的扩展性
2. 验证层（H2 核心）在此模式下结构最清晰——`validate()` 函数直接夹在"Harness 执行 tool → 结果返回 Claude"之间
3. 省去 MCP client 实现成本，保持 ~1200 行规模
4. Phase 2 需要接入社区生态时，只需把 `dispatch_tool()` 内部替换为 MCP client，Harness 接口不变

**调用流程**：
```
用户输入
  → Rust Harness → Claude API（附带 tool schema 列表）
  ← Claude 返回 tool_call { name, args }
  → Harness 执行 tool（本地实现）
  → ToolResultValidator.validate()          ← 验证层在此插入
  → Harness → Claude API（附带验证后结果）
  ← Claude 返回最终回复
```

---

## 关联文档
- 完整架构：[[Overview]]
- Context Manager 设计细节：[[modules/01-Memory-Hierarchy]]
- Tool 验证层背景：[[modules/02-Peripheral-MCP]]
- DAI 完整设计：[[modules/00-Data-Access-Model]]
- Ring 分级设计：[[modules/03-Kernel-Privilege]]
