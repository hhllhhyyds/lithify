# 架构设计

## 1. 四层记忆架构

Lithify 的核心架构是四层记忆系统。信息从精确到模糊、从昂贵到免费、从灵活到固化，沿着四层逐步沉降。

| 层 | 名称 | 精度 | 调用成本 | 容量 | 可变性 | 工程载体 |
|----|------|------|----------|------|--------|----------|
| L1 | 工作记忆 | 最高 | 极高 | 最小（~200K tokens） | 挥发性 | Context window |
| L2 | 短期精确记忆 | 精确 | 高（全文加载） | 小（数十个 Skill） | 高 | Markdown 文件（`skills/` 目录） |
| L3 | 长期模糊记忆 | 模糊（泛化） | 中（仅注入检索命中片段） | 大（数万条 RAG 条目） | 低 | SQLite + sqlite-vec 向量存储 |
| L4 | 超长期肌肉记忆 | 最低（对外部） | 零 | 极大（数百个 Tool） | 无（固化） | Tool 注册表 + 子进程沙箱 |

### 层间关系

```
L1 工作记忆（Context Window）     ← LLM 在此汇聚所有信息并推理
    ↑ 按需加载
L2 短期精确记忆（Skill）          ← 精确文本指令，全文注入 context
    ↑ 压缩/归档
L3 长期模糊记忆（RAG）            ← 模糊检索，仅注入命中片段
    ↑ 固化为代码
L4 超长期肌肉记忆（Tool）         ← 零 context 占用，按函数名调用
```

核心梯度：**从上到下，存储容量递增，调用成本递减，可变性递减，固化程度递增**。

Agent 的进化，本质上就是让信息和能力沿着这个梯度向下沉降。每沉一层，context 消耗降一个数量级。

LLM（大模型）本身不属于四层中的任何一层。它的权重在训练完成后冻结，不会因为对话而改变。它只是 **大脑**——一个纯粹的信息处理器。记忆系统是独立于大脑的外部结构。

### 设计动机

百万 token 级别的 context 已经足够覆盖绝大多数任务的瞬时信息量。继续卷 context 长度边际收益急剧递减。**接下来的竞争高地不是更大的 context，而是更聪明的记忆系统**——如何判断什么信息值得保留、以什么形态保留、在什么时机注入有限的 context。这正是四层记忆架构要解决的问题。

---

## 2. 四个层的工程实现

### L1 工作记忆：Context Window

- **载体**：LLM API 调用的 messages 数组
- **内容**：系统提示词 + Skill 全文 + RAG 检索片段 + Tool 调用结果 + 对话历史
- **特性**：不做持久化，对话结束即清空
- **职责**：所有信息最终汇合于此，LLM 在此完成理解和推理
- **约束**：每一字节都应花在刀刃上。所有持久记忆存在的意义，归根结底都是**在正确的时刻、以最小的代价，把正确的信息加载到 context**

### L2 短期精确记忆：Skill 系统

- **载体**：`skills/` 目录下的 Markdown 文件
- **内容**：完整的"如何做一件事"的指令，精确、可执行
- **生命周期**：
  1. 用户创建或 Agent 在对话中生成/修改 Skill 文件
  2. 每次任务开始时，CLI 框架加载相关 Skill 全文，注入系统提示词
  3. 随着使用，Skill 不断被修正、精炼
  4. 当 Skill 稳定不再频繁修改时，触发压缩归档到 RAG

- **核心组件**：
  - `crates/skill/`：SkillLoader（文件读取、解析）、SkillInjector（注入系统提示词）
- **特点**：高保真、高成本。每次调用消耗可观的 context token，适合仍在快速迭代、尚未固化的知识
- **能力**：
  - Skill 自我修改：Agent 可以在对话中通过工具更新 Skill 文件（追加新技术、修正过时步骤、精炼指令）
  - Skill 从零生成：Agent 完成任务后，总结新经验写入 Skill 文件

### L3 长期模糊记忆：RAG 系统

- **载体**：SQLite 数据库 + sqlite-vec 向量扩展
- **内容**：经过压缩的经验片段和通用原则（非逐字逐句的指令）
- **生命周期**：
  1. Skill 满足稳定条件（连续 N 次使用无修改）时，触发压缩归档
  2. Agent 对 Skill 内容做一次压缩/总结，提炼通用原则
  3. 生成 Embedding 存入向量数据库
  4. 每次任务前，通过检索路由查询 RAG，将命中片段注入 context

- **核心组件**：
  - `crates/rag/`：RagStore（向量存储接口）、CompressionPipeline（压缩管道）、StabilityTracker（稳定检测）
  - `crates/storage/`：SQLite + sqlite-vec 存储层

- **特点**：中保真、中成本。检索时只注入命中片段，context 占用远低于 Skill。核心价值是**泛化匹配**——模糊换来对多变场景的适应力。一个 RAG 条目能覆盖多种相关场景，而 Skill 换一个变量名就可能匹配不上。

- **压缩原则**：好的压缩保留三样东西——
  - **关键决策点**：在每个岔路口走哪边
  - **常见失败模式**：什么情况下会出问题、怎么避免
  - **适用条件**：什么场景下这条经验有效
  - 剥离一次性细节（如"上次部署时用的分支名是 feature/xxx"）

### L4 超长期肌肉记忆：Tool 系统

- **载体**：Tool 注册表 + 子进程沙箱
- **内容**：可重复执行的固化程序（Rust 代码编译为二进制）
- **生命周期**：
  1. RAG 条目被高频命中、模式稳定后，Agent 生成 Tool 代码
  2. Tool 在沙箱中编译、验证
  3. 用户确认后注册到 Tool 注册表
  4. 此后调用只需函数名和参数，零 context 占用

- **核心组件**：
  - `crates/tools/`：ToolRegistry（Tool 注册、发现、调用）、Tool 标准实现
  - `crates/sandbox/`：子进程沙箱（编译、执行、验证）

- **特点**：低保真（对外部）、零成本。LLM 不需要知道它内部如何运作，只需知道何时调用、传什么参数。越是高频使用的功能，越应该以 Tool 形态存在。

---

## 3. 记忆沉降管道

记忆沉降是 Lithify 的核心数据流。经验从 L2 逐步固化到 L4，每一步降低 context 成本：

### 3.1 Skill-ification（经验 → Skill）

| 项目 | 说明 |
|------|------|
| **触发** | Agent 完成任务后，识别出新的可复用经验 |
| **输入** | 对话中的执行经验、用户纠正、Agent 自身总结 |
| **动作** | Agent 在 `skills/` 下创建或更新 Markdown 文件 |
| **输出** | 精确的可执行指令文本 |
| **成本** | 高（每次加载全文进 context）|
| **特点** | 高保真、高灵活性、适合快速迭代 |

### 3.2 RAG-ification（Skill → RAG）

| 项目 | 说明 |
|------|------|
| **触发** | StabilityTracker 检测到 Skill 连续 N 次使用未被修改（N 默认可配置） |
| **输入** | 已稳定的 Skill 文档全文 |
| **动作** | CompressionPipeline 对 Skill 做压缩/总结：去掉一次性细节，提炼通用原则 |
| **输出** | 压缩后的 RAG 条目 + Embedding 向量 |
| **成本** | 中（仅注入检索命中片段） |
| **关键挑战** | 压缩质量——保留关键决策点、常见失败模式、适用条件，剥离一次性参数 |

**压缩前后对比**：

```
压缩前（Skill，几十行）：
  "先确认本地 Git 状态干净，再确认当前分支是 main，运行 npm run build，
   构建产物在 /dist 目录，用 Cloudflare Dashboard 设置环境变量 KEY1=xxx，
   KEY2=yyy，最后在 Dashboard 点击 Deploy 按钮触发部署……"

压缩后（RAG 条目，三句话）：
  "Cloudflare Pages 部署要点：Git 分支必须为 main；build output 目录为
   /dist；环境变量通过 Dashboard 设置而非 .env 文件；部署触发方式为
   Git push 或 Dashboard 手动触发。"
```

### 3.3 Tool-ification（RAG → Tool）

| 项目 | 说明 |
|------|------|
| **触发** | RAG 条目被高频命中（命中次数超过阈值），且输入输出契约清晰 |
| **输入** | RAG 条目中的自然语言经验描述 |
| **动作** | Agent 生成 Tool 代码（Rust）→ 沙箱编译 → 运行验证 → 用户确认 |
| **输出** | 注册到 ToolRegistry 的可执行程序 |
| **成本** | 零（按函数名调用，不占 context） |
| **关键挑战** | 接口抽象、代码生成正确性、沙箱验证、版本兼容性 |

**代码生成需要的能力**：

1. **接口抽象**：从自然语言描述中推断参数名、类型、是否必填、默认值
2. **代码生成**：生成正确且可维护的 Rust 代码，处理正常路径和异常路径
3. **边界与异常处理**：超时、权限不足、网络中断等异常的结构化处理
4. **沙箱验证**：在受控环境中编译运行，用真实或模拟输入验证输出
5. **版本与兼容性**：Tool 上线后行为必须稳定、可预期，后续更新不能破坏旧调用方式

这一步本质上是 **Agent 的自我编程**——整个四层架构中最难自动化的一环。近期的可行路径是**半自动**：Agent 生成 Tool 代码草稿和测试用例，用户在沙箱中验证后确认上线。

### 3.4 完整进化示例

以"部署到 Cloudflare Pages"为例，演示完整的三层进化路径：

1. **Skill 阶段**：用户编写 `deploy-cloudflare.md`，几十行详尽步骤。每次部署时 Agent 全文加载，逐条执行。精确但每次消耗完整 context。
2. **RAG 阶段**：执行约 10 次后，Agent 发现核心模式不变，压缩为三条关键要点存入 RAG。原 Skill 精简为一句话指针指向 RAG。每次 context 开销从几百字降到几十字。
3. **Tool 阶段**：再执行约 50 次后，Agent 生成 `deploy-to-cloudflare` Tool，接收 `--project-path`、`--env-vars`、`--branch` 三个参数。此后部署只需一行命令，**零 token 理解部署流程**。LLM 甚至不需要知道部署怎么完成，只知道"有一个 Tool 能做这件事"。

---

## 4. 检索路由

四层记忆就位后，Agent 面对每个任务必须先回答一个元问题：**去哪个记忆层找信息？**

这是四层架构能否真正工作的关键基础设施。没有路由，记忆不是资产，是负担。

### 三种路由策略

| 策略 | 流程 | 优点 | 缺点 |
|------|------|------|------|
| **自顶向下** | Skill → RAG → LLM 推理 | 简单，精确匹配优先 | 顺序依赖，Skill 误判命中则错过 RAG |
| **并行检索** | Skill + RAG 同时查询，合并后 LLM 裁决 | 无顺序依赖，召回率高 | 单次计算开销大，context 占用多 |
| **索引层** | 先查轻量索引定位，再按图索骥 | 可扩展，索引常驻 context | 需要维护索引一致性 |

### 初始实现选择

MVP 阶段采用 **自顶向下** 策略，理由：
- 实现最简单，适合 Phase 1 快速验证
- Skill 数量少时，顺序依赖的影响有限
- 后续可平滑升级到索引层策略

索引层策略随 Skill 和 Tool 数量增长自然引入。索引本身是一个极小的、常驻工作记忆的路由表——所有 Skill 和 Tool 各一两条摘要，先查索引定位再定向检索。这是最可扩展的方案。

---

## 5. 进化触发与版本控制

### 5.1 触发机制

记忆沉降由谁触发？三种机制共存，按 Agent 成熟度逐步放权：

| 阶段 | 机制 | 触发方式 | Agent 成熟度 |
|------|------|----------|-------------|
| 前期 | **用户手动触发** | `/archive`、`/solidify` 命令 | 低 |
| 中期 | **Harness 层自动处理** | 框架监控 Skill 使用频率和内容稳定性，自动触发 | 中 |
| 后期 | **Agent 自主判断** | Agent 自行评估"这段经验可以归档了"并执行 | 高 |

**触发机制本身的自我进化**：前期由用户手动触发，确保每次迁移都经过人类判断；随着 Agent 积累的经验增多、判断力增强，逐步下放给 Harness 自动处理，最终过渡到 Agent 自主判断。触发机制本身也在四层记忆的进化管道中逐级下沉。

### 5.2 版本追溯

每层内容的元数据携带来源追溯：

- **RAG 条目**：记录"来自哪个 Skill、何时归档、归档时版本号"
- **Tool**：记录"来自哪些 RAG 条目、何时固化、依赖的外部 API 版本"
- 当底层信息被标记为过期时，上层固化产物可追溯根源、随之更新或标记为废弃

### 5.3 回退机制

如果 Tool 运行出错，系统不应直接宕掉，而是沿追溯链降级：

```
Tool 出错 → 定位到来源 RAG 条目 → 临时降级为 RAG 模式（检索经验，LLM 手动执行）
    → 如 RAG 也失效 → 继续降级为 Skill 模式 → 同时修复或重新生成 Tool
```

记忆的进化不是单向的"写入→遗忘"，而是一个**可追溯、可回降、可修复**的链。

---

## 6. Crate 职责与核心 Trait

### 6.1 core/ — 核心抽象

零外部依赖，纯接口层。定义整个项目共享的 trait 和类型。

关键类型（预期）：

| 类型 | 说明 |
|------|------|
| `trait LLMClient` | LLM 调用抽象：`chat() -> Response` |
| `trait SkillLoader` | Skill 加载抽象：`load(name) -> Skill` |
| `trait SkillStore` | Skill 持久化抽象：`save(skill)`、`delete(name)` |
| `trait Storage` | 底层持久化抽象：`put(key, value)`、`get(key)`、`delete(key)`、`search_vectors(collection, vec, limit)` |
| `trait RagStore` | RAG 领域抽象：`search(query) -> Vec<Entry>`、`insert(entry)`、`delete(id)`。实现内部将文本转为向量，调 `Storage.search_vectors()` |
| `trait ToolExecutor` | Tool 执行抽象：`execute(tool, args) -> Result` |
| `trait ToolRegistry` | Tool 注册表抽象：`register(tool)`、`find(name) -> Tool`、`list() -> Vec<Tool>` |
| `trait Sandbox` | 沙箱执行抽象：`run(code, input) -> ExecutionResult` |
| `trait SkillStabilityTracker` | Skill→RAG 稳定检测：`record_usage(skill)`、`is_stable(skill) -> bool` |
| `trait RagStabilityTracker` | RAG→Tool 稳定检测：`record_hit(entry)`、`ready_for_tool(entry) -> bool` |
| `trait CompressionPipeline` | Skill→RAG 压缩管道：`compress(skill) -> RagEntry` |
| `trait ToolCodeGenerator` | RAG→Tool 代码生成：`generate(entry) -> ToolCode` |
| `trait RetrievalRouter` | 检索路由抽象：`route(task) -> SearchTarget` |

错误类型：`LLMError`、`StoreError`、`SandboxError`、`ToolError`

### 6.2 runtime/ — 控制循环

- **职责**：CLI shell + 对话循环。接收用户输入，编排 L1-L4 的交互
- **流程**：
  1. 接收用户任务
  2. 调用 RetrievalRouter 决定检索策略
  3. 按策略加载 Skill / 检索 RAG
  4. 组装系统提示词（Skill 全文 + RAG 命中片段 + Tool 列表）
  5. 调用 LLMClient 推理
  6. LLM 可能调用 Tool → 沙箱执行 → 结果回注 context
  7. 输出结果给用户
  8. （可选）记录经验用于后续沉降

- **依赖**：`core`、`skill`、`rag`、`tools`、`llm`、`sandbox`、`storage`（编排层，负责实例化并组装所有组件）

### 6.3 skill/ — Skill 系统

- **职责**：Skill 文件的 CRUD、解析、注入系统提示词；Skill 稳定检测
- **实现**：
  - `SkillLoader` trait → 从 `skills/` 目录读取 Markdown，解析 frontmatter（名称、描述、触发条件），注入到系统提示词
  - `SkillStore` trait → 保存和删除 Skill 文件
  - `SkillStabilityTracker` trait → 记录 Skill 使用次数和修改频率，判断是否达到归档阈值
- **自我修改**：Agent 通过文件工具直接修改 `skills/` 下的文件
- **依赖**：`core`

### 6.4 rag/ — RAG 系统

- **职责**：向量检索、Skill→RAG 压缩管道、RAG 条目 Tool 化检测、RAG→Tool 代码生成
- **实现**：
  - `RagStore` trait → SQLite + sqlite-vec 作为后端，支持模糊检索
  - `CompressionPipeline` trait → LLM 驱动的压缩：输入 Skill 全文，输出压缩后的 RAG 条目
  - `RagStabilityTracker` trait → 记录 RAG 条目命中频率，判断是否达到 Tool 化阈值
  - `ToolCodeGenerator` trait → LLM 驱动的代码生成：输入 RAG 条目，输出可编译的 Rust Tool 代码
- `RagStore.search(query)` 流程：把用户输入的一句话（如"怎么部署到 Cloudflare"）编码为一串浮点数向量，然后把向量传给 `Storage.search_vectors()`，由 Storage 在向量索引中找到最相近的几条 RAG 条目。类比：把一句话压成条形码，去数据库扫出条码最接近的几条记录
- **依赖**：`core`、`storage`、`llm`（压缩和代码生成需调用 LLM）

### 6.5 tools/ — Tool 系统

- **职责**：Tool 注册表、标准工具库
- **实现**：`ToolRegistry` trait → 内存索引 + 文件系统持久化；标准工具包括 bash、文件读写、grep、glob 等
- **依赖**：`core`、`sandbox`

### 6.6 sandbox/ — 沙箱执行

- **职责**：在隔离环境中编译并运行生成的 Tool
- **实现**：`Sandbox` trait → `tokio::process::Command` 启动子进程，通过 stdin/stdout 通信
- **依赖**：`core`

### 6.7 llm/ — LLM 调用

- **职责**：封装 LLM API 调用、消息格式转换、响应解析
- **实现**：`LLMClient` trait → Anthropic Claude 实现（`reqwest` 裸 HTTP）
- **依赖**：`core`

### 6.8 storage/ — 持久化存储

- **职责**：底层 KV + 向量存储，为上层 crate 提供数据持久化
- **实现**：`Storage` trait → SQLite + sqlite-vec 作为统一存储后端；提供 KV 读写（Session 事件、Tool 元数据）和向量查询（供 `RagStore` 组合使用）
- **依赖**：`core`

### Crate 依赖图

```
                    ┌─────────────┐
                    │    core     │  ← 纯 trait，零依赖
                    └──────┬──────┘
           ┌───────┬───────┼───────┬───────┬───────┐
           ▼       ▼       ▼       ▼       ▼       ▼
       storage   llm   sandbox   skill    rag    tools
         │        │                │      │ │      │ │
         │        │                │      │ │      │ │
         └────────┴────────────────┴──────┴─┴──────┴─┘
                           │
                           ▼
                       runtime        ← 组装所有组件，启动对话循环
```

各依赖说明：

| crate | 直接依赖（除 core 外） | 原因 |
|-------|----------------------|------|
| `core` | 无 | 纯 trait + 类型 |
| `storage` | 无 | SQLite + sqlite-vec |
| `llm` | 无 | reqwest（裸 HTTP） |
| `sandbox` | 无 | tokio::process |
| `skill` | 无 | 只操作文件系统 |
| `rag` | `storage`、`llm` | 向量存储靠 storage，压缩和代码生成靠 llm |
| `tools` | `sandbox` | Tool 执行扔给沙箱 |
| `runtime` | 全部 | 实例化所有组件，组装并启动 |

---

## 7. 项目结构

```
Lithify/
├── CLAUDE.md              # AI 编程入口
├── README.md              # 项目说明
├── Cargo.toml             # workspace 定义
├── docs/
│   ├── ARCHITECTURE.md    # 本文
│   ├── TECH_STACK.md      # 技术选型
│   └── AI_WORKFLOW.md     # AI 编程流程规范
├── crates/
│   ├── core/              # 核心 trait + 类型定义（零外部依赖）
│   ├── runtime/           # 控制循环 + 对话循环
│   ├── skill/             # Skill 加载、解析、注入
│   ├── rag/               # RAG 存储、向量检索、压缩管道
│   ├── tools/             # 标准工具库 + Tool 注册
│   ├── sandbox/           # 沙箱执行环境（子进程）
│   ├── llm/               # LLM 调用抽象 + Anthropic 实现
│   └── storage/           # 持久化存储（SQLite + 向量索引）
└── examples/
    ├── hello-agent/       # Mock LLM 端到端示例
    └── real-agent/        # 真实 LLM 端到端示例
```

---

## 8. 跨 Crate 规范

- **语言**：Rust 2024 edition
- **异步运行时**：tokio
- **序列化**：serde + serde_json
- **命名**：snake_case（Rust 标准），trait 名大驼峰
- **错误处理**：trait 内部用 `thiserror` 定义具体错误类型，应用层可用 `anyhow`
- **文档注释**：所有 pub 类型和方法必须有 `///` doc comment（英文）
- **语言约定**：文档用中文，代码注释全部用英文
- **测试**：每个 crate 必须有单元测试，trait 实现必须有集成测试
- **commit 格式**：`<type>(<scope>): <description>`
- **模块文件**：使用 `foo.rs` 而非 `foo/mod.rs`

---

## 9. 四层记忆开发路线

开发路线按照架构的沉降方向进行——从最上层的"能用"开始，逐步向下沉降：

| 阶段 | 内容 | 对应架构层 |
|------|------|-----------|
| Phase 1: Skeleton | CLI shell + 对话循环 + LLM 集成 + 基础工具 + Skill 加载 + 自我修改 | Skill（短期记忆）就绪 |
| Phase 2: Skill → RAG | 向量存储 + 稳定检测 + 压缩管道 + 检索路由 | RAG（长期记忆）就绪 |
| Phase 3: RAG → Tool | Tool 代码生成 + 沙箱验证 + 注册系统 | Tool（肌肉记忆）就绪 |
| Phase 4: Self-Evolution | 触发机制 + 版本追踪 + 回退 + 自动化 | 全闭环进化 |

每个阶段的实现都应遵循 `docs/AI_WORKFLOW.md` 中定义的 TDD 流程。
