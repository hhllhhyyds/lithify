---
date: 2026-05-13
type: idea
tags: [idea, agent-harness, llm, memory-hierarchy, architecture]
status: exploring
ai-first: true
---

## For future Claude
这是一个关于 AI Agent Harness 架构的类比灵感，由 hhl 于 2026-05-13 提出。核心观点：LLM agent 的各层组件与操作系统存在深度对应关系——存储层次、外设体系、内核/应用程序分层、自进化安全设计均精确吻合，并系统梳理了 lithify 尚未解决的 8 个缺失模块（按 P0~P3 优先级排列）。最重要的推论：①特权环缺失是 prompt injection 的架构根源；②自进化遵循体细胞/生殖细胞隔离原则；③IPC 和可观测性是当前所有 agent 框架最痛的缺失（P1/P2）。

---

## 核心类比

| 计算机存储层次 | AI Agent 对应组件 | 共同 Tradeoff |
|---|---|---|
| CPU 寄存器 | LLM context window | 最快、最贵、容量极限 |
| L1/L2/L3 Cache | System prompt + Skills（已加载） | 预热、低延迟、有限容量 |
| Skill 层次结构 | Skill 库（按需换入） | 结构化、可寻址、需换入 |
| RAM | 完整 skill hierarchy | 按需加载，有组织 |
| Disk / 磁盘 | Knowledge base + RAG | 容量大、延迟高、需要检索 |

**LLM = CPU**：处理单元，所有"思考"在此发生。

---

## 关键延伸洞察

### 1. 寄存器压力 = Context Pressure
编译器面临的"寄存器分配"问题在 agent 里完全重现：context 有限，变量（信息块）太多就要 spill。当前大多数 harness 靠截断解决，相当于没有分配策略。

**工程含义**：agent harness 的核心问题本质上是一个**编译器寄存器分配问题**。

### 2. Cache Eviction Policy = Context Compaction 策略
- OS 有 LRU、LFU、ARC 等策略
- 当前 agent compaction ≈ FIFO（丢最老的）——这是最差的缓存策略之一
- 问题：**什么信息应该留在 context 里？** 这是未被认真对待的研究问题

### 3. I-cache vs D-cache = 行为规则 vs 背景知识
- System prompt 中的"行为规则" → 指令缓存（I-cache）
- 注入的背景知识 → 数据缓存（D-cache）
- 两者混在 system prompt 里 = 架构污染，应分层管理

### 4. Pre-fetching = Skill 预加载
- OS 根据访问模式预取数据
- 优秀的 agent harness 应根据任务上下文预判所需 skills，提前加载
- 当前实现：每次从头检索，相当于关闭了预取

### 5. Cache 透明性 = Agent Harness 的根本差异
**最重要的张力点**：CPU cache 对程序员完全透明（硬件自动管理），但 LLM context 完全需要人工显式管理，且管理质量直接影响输出质量，不只是速度。

这意味着 agent harness 需要一个**显式的内存管理层**，类似操作系统的内存管理子系统，而不是让开发者手动堆 prompt。

---

## 推论：NUMA 与多 Agent 协调

如果单 agent = 单核 CPU，那多 agent = **NUMA 多核架构**：

- 每个 agent 有本地 context（本地内存，访问快）
- 跨 agent 共享状态 = 跨 NUMA 节点访问（慢、需同步）
- 多 agent 系统最难的问题（context coherency）直接对应多核 CPU 最难的问题（**缓存一致性**）

**预言**：多 agent 框架最终会演化出类似 MESI 协议的状态同步机制。

---

## 延伸：内核程序 vs 应用程序

_2026-05-13 追加_

### 基本映射

| OS 概念 | Agent 对应 | 具体例子 |
|---|---|---|
| 内核 | Harness 核心层 | Claude Code harness、LangGraph runtime、AutoGen orchestrator core |
| 应用程序 | Skills / 具体 Agent | code-review skill、obsidian-save skill、用户自定义 workflow |
| 系统守护进程 | 后台维护 Agent | 定期 vault health check、内存整合 agent、index 维护 agent |
| Shell | 对话界面 / REPL | Claude Code 的命令行界面、聊天 UI |
| 进程 | 单次 Agent 调用 | 每个 subagent 调用 = fork() 出一个子进程 |
| PID 1 / init | 根 Orchestrator | 主对话循环，所有 subagent 的祖先进程 |

内核职责与 harness 的精确对应：
- 内存管理 → Context 管理（eviction、compaction、RAG 检索）
- 进程调度 → 多 agent 调度（串行/并行、优先级）
- I/O 管理 → Tool 调用路由与执行
- 安全子系统 → 权限系统（哪些 tool 可被调用、需要用户确认）

### 最重要的推论：特权环缺失

Linux 有 Ring 0（内核态）→ Ring 3（用户态）的特权分级：

```
Ring 0  内核        直接操作硬件、管理所有内存
Ring 3  应用程序    只能通过 syscall 请求内核代劳，无法直接访问内核内存
```

**当前所有 agent 框架都运行在"Ring 0"**——orchestrator、subagent、处理不可信内容的 agent，全部同等特权，没有隔离。

正确的设计：

```
Ring 0  Harness 核心        context 管理、tool 执行、权限执行
Ring 1  可信 Orchestrator   有权调度子 agent、访问全局状态
Ring 3  任务 Agent          只能访问自己的 context 切片，tool 调用须经 Ring 0 审查
```

处理不可信内容（爬取网页、用户上传文件）的 agent 必须在 Ring 3 运行。其输出在进入上层 context 前须经"syscall 验证层"过滤。

**这从架构上解决了 prompt injection：用户态程序无法写内核内存。**

（与前文"内核/用户态隔离缺失 = prompt injection 根源"形成呼应，见外设章节。）

### 宏内核 vs 微内核

| 架构 | OS 代表 | Agent 等价 | 特点 |
|---|---|---|---|
| 宏内核 | Linux | 单一巨型 orchestrator（处理路由、规划、内存、工具） | 快，但一个模块崩溃影响全局 |
| 微内核 | QNX、Mach | 最小 harness（只做 context 管理 + tool dispatch）+ 独立微 agent 通过消息传递协作 | 健壮，任一子 agent 崩溃不污染整体 |

微内核架构对 agent 系统可能更优——隔离性天然防御 prompt injection 扩散，且每个"内核服务"可以独立升级替换。

---

## 延伸：Tools / MCP = OS 外设体系

_2026-05-13 追加_

| OS 外设体系 | Agent 工具体系 | 说明 |
|---|---|---|
| 外设（键盘、磁盘、网卡） | Tools（搜索、代码执行、文件读写） | 外部能力，非 CPU 原生 |
| 设备驱动 | Tool 定义 / MCP server connector | 对上提供统一接口，屏蔽底层细节 |
| 系统调用（syscall） | Tool invocation API | 受控的能力穿越边界 |
| 中断（IRQ） | Tool 返回结果注入 context | 设备完成后通知 CPU；tool 完成后结果注入 |
| 即插即用（PnP） | MCP 动态发现 | 运行时发现可用工具，无需硬编码 |

### MCP = USB 标准

USB 之前：串口、并口、PS/2 各自为政，驱动不互通。USB 统一了物理接口与协议，实现即插即用。

MCP 在做同样的事：在它之前，每个框架有自己的 tool 格式（OpenAI function calling、LangChain tool……）。MCP 是 AI 工具生态的 USB 标准。

**预言**：MCP 最终会分化出性能层级，类似 USB 2.0 / 3.0 / 4.0——低延迟本地工具和高带宽远程服务走不同"总线"。

### 中断模型 vs 轮询模型

当前 agent tool call 全部是**同步阻塞**（PIO 模式）：LLM 发出调用，停下来等结果。

OS 早已淘汰轮询，改用**中断驱动 I/O**：CPU 发出请求后继续执行，设备完成时发中断。

Agent 的对应设计：发出 tool call 后继续推进其他子任务，结果异步注入。这正是多 agent 并行架构的本质动机——但很少有人从"中断 vs 轮询"角度分析它。

### 内核态 / 用户态隔离缺失 = Prompt Injection 根源

OS 中应用程序不能直接操作硬件，必须通过 syscall 由内核代理，内核验证请求合法性。

当前 agent 框架：LLM 既"决定"调用哪个工具，又直接"解释"工具返回结果——没有内核/用户态边界。恶意或错误的 tool 返回可以直接污染 LLM 推理，相当于**用户态代码可以直接写内核内存**。

**这是 prompt injection 攻击的架构根源。** 真正安全的设计需要一个独立的"内核层"验证和过滤 tool 返回值，再决定哪些内容可进入 context。

---

## 延伸：自进化应该在内核层还是用户层？

_2026-05-13 追加_

### 两种进化的本质差异

| 进化类型 | OS 类比 | 风险等级 |
|---|---|---|
| Skills 更新 | 用户态应用程序更新 | 低，可回滚，影响范围有限 |
| lithify 源码更新 | 内核重编译/替换 | 极高——内核是所有规则的执行者，若能自改则可删除"需用户确认"等安全约束 |

**核心原则：进化权限应与特权级反向绑定。**

```
Ring 3  任务 Agent     → 自由自进化，自动、连续
Ring 1  Orchestrator   → 可以提议修改，需要人工审批
Ring 0  Harness 内核   → 禁止运行时自修改
```

### Skills 进化 → 用户层，全自动

- 低风险，可回滚，影响范围有限
- 高频率，每次对话均可学习
- 可 A/B 测试新版本，不影响 harness
- 类比：Chrome 应用自动更新，不需要重装 OS

**"用得越多越好用"这个目标完全可以在用户层实现，不需要动内核。**

### Harness 进化 → 分两层处理

**第一层：参数调优（sysctl 级别）**
- 调整 eviction 策略权重、context 分配阈值、tool timeout 等
- 半自动：agent 提议 → 人工确认 → 生效
- 放在受限的"配置层"，而非内核核心

**第二层：源码级修改（内核重编译）**
- 不应在运行时发生
- 正确路径：agent 生成 PR → 人工 review → 测试 → 部署新版本
- 这正是 Claude Code 自身的演化方式——用户提 issue，工程师改 harness，发新版本

### 最强类比：体细胞 vs 生殖细胞突变

生物学已经解决了自进化安全问题：

- **体细胞突变**（somatic）：单个细胞在一生中适应环境，不遗传，影响个体
- **生殖细胞突变**（germline）：影响所有后代，有严格的 DNA 修复机制，极度保守

对应到 agent：
- **Skills 进化** = 体细胞突变：影响当前实例，自动连续
- **Harness 源码进化** = 生殖细胞突变：影响所有未来实例，必须外部审查

生物用"体细胞/生殖细胞隔离"解决了自进化安全问题，lithify 应做同样的分离。

### 结论

| 进化类型 | 推荐位置 | 自动化程度 |
|---|---|---|
| Skills 更新 | 用户层（Ring 3） | 全自动 |
| Harness 配置调优 | 内核参数层（sysctl） | 半自动，需确认 |
| Harness 源码修改 | 系统外部（CI/CD） | 人工主导 |

用户层进化反而比内核进化更快——skills 可实时更新，内核升级需要"重启"（重新部署）。

---

## lithify 缺失模块分析

_2026-05-13 追加_

已覆盖：存储层次、外设/MCP、内核/用户态、进程调度、自进化。以下是尚未设计的模块，按优先级排列。

### P0 — 安全关键

**信号系统（Signals）**
- OS：SIGTERM（优雅退出）、SIGKILL（强制终止）、SIGINT（用户中断）
- 缺失：当前 agent 无优雅中止协议，失控 agent 只能被强行截断生成
- 影响：runaway agent 无法被安全终止，是安全设计的根本漏洞

**稳定 ABI / Syscall 接口**
- OS：Linux 承诺 syscall 向后兼容，内核可随意重写内部实现，用户态程序不崩
- 缺失：harness 升级即可能导致 skills 失效，无稳定接口契约
- 影响：harness 和 skills 无法独立进化，形成强耦合

### P1 — 功能关键

**IPC（进程间通信）**
- OS：pipe、消息队列、共享内存、socket
- 缺失：多 agent 通信只能通过父 agent context 中转（贴纸条模式），无直连通道
- 影响：多 agent 系统吞吐受限，一致性难以保证

**虚拟内存 / Context 隔离**
- OS：虚拟地址空间——每个进程认为自己拥有全部内存，实际是隔离切片
- 缺失：subagent 要么继承父 agent 全部 context（权限过大），要么完全隔离（权限过小）
- 影响：无法实现细粒度的 context 视图控制，也无法防止跨 agent 越界读写

### P2 — 工程必须

**可观测性 / `/proc` 等价物**
- OS：`/proc`、`strace`、`top`、`perf` 可实时内省任意进程状态和资源消耗
- 缺失：agent 系统完全黑箱——无法看到 subagent 的 context 内容、tool 等待状态、token 消耗分布
- 影响：agent 系统出问题无从调试，工程实践中最痛的缺失

**安全模型（身份 + 授权 + 审计）**
- OS：用户身份（uid/gid）、细粒度 capabilities、不可篡改的 audit log
- 缺失：
  - 无 agent 身份认证——subagent 调用链中权限如何继承？能否无限放大？
  - 权限只有 tool 级别，缺乏 Linux capabilities 式的细粒度控制
  - 无不可篡改的操作审计日志
- 影响：多 agent 系统的权限管理混乱，无法事后追溯

### P3 — 生态完善

**Init 序列 / Bootloader**
- OS：BIOS → bootloader → kernel → init → services，每层为下一层建立环境
- 缺失：agent session 启动顺序隐式，skills 初始化顺序不确定，崩溃后恢复无标准流程
- 影响：session 初始状态混乱，难以复现和调试

**包管理器**
- OS：apt/brew 处理版本、依赖解析、分发、回滚、安全签名
- 缺失：skills 是手动管理的文件，无版本锁、无依赖解析、无签名验证
- 影响：skill 生态无法规模化，升级存在隐患

---

## 开放问题

- [ ] 是否存在"TLB"等价物？——快速查找哪些 skills 可用，而无需加载所有 skill
- [ ] Context compaction 的最优 eviction 策略是什么？是否可以学习？
- [ ] I-cache / D-cache 分离在 system prompt 设计上如何实践？
- [ ] 多 agent coherency 协议的最小可行设计是什么？
- [ ] MCP 性能层级如何设计？本地 tool 与远程 MCP server 的最优调度策略？
- [ ] agent "内核层"的最小可行设计——如何在不损失灵活性的前提下过滤 tool 返回值？
- [ ] Ring 3 agent 的 context 隔离如何实现？隔离粒度是 agent 级还是 task 级？
- [ ] 微内核 agent 架构的 IPC 机制如何设计？消息格式、路由、错误传播？
- [ ] Skills 自进化的具体机制：基于对话反馈打分？还是显式用户标注？
- [ ] Harness 配置调优（sysctl 层）的安全边界如何划定，防止通过配置绕过安全约束？
- [ ] IPC 的消息格式与路由协议如何设计？
- [ ] Agent 信号系统的最小可行设计：SIGTERM/SIGKILL 等价物如何实现？
- [ ] 可观测性层：agent `/proc` 的数据模型是什么？
- [ ] Agent 身份链：subagent 权限继承规则，如何防止权限放大？
- [ ] 稳定 ABI：skill ↔ harness 接口契约的最小集合是什么？

---

## 来源与背景
- 灵感来源：hhl 与 Claude 的对话，2026-05-13
- 触发背景：观察 agent harness（如 Claude Code）的 skill/context 机制，联想到 OS 存储层次；进一步延伸至外设/驱动体系
- 信心等级：类比框架 `high`；具体工程推论 `medium`
