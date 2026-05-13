# lithify

> 用操作系统的设计智慧，构建可信赖、可演化、可观测的 AI Agent 基础设施。

## 核心思想

当前主流 agent 框架（LangChain、AutoGen、Claude Code 等）大多是"能跑就行"的工程堆叠，缺乏系统性架构设计。而操作系统领域经过 60 年演化积累了大量智慧——内存层次、特权隔离、进程通信、可观测性——这些在 agent 领域几乎是空白。

lithify 将 OS 核心概念系统性地映射到 agent 系统：

| OS 概念 | lithify 对应 |
|---|---|
| 内核 | Agent Harness（Ring 0） |
| CPU 寄存器 | LLM context window |
| L1/L2/L3 缓存 | System prompt + 已加载 Skills |
| RAM | Skill 库（按需换入） |
| 磁盘 | Knowledge base + RAG |
| 外设 | Tools（搜索、代码执行等） |
| USB / 即插即用 | MCP 动态发现 |
| 特权环（Ring 0~3） | Harness → Orchestrator → Task Agent |
| 虚拟地址空间 | Virtual context 隔离 |
| `/proc`、`strace`、`top` | Agent 可观测性 |

## Phase 1 目标

用约 1200 行 Rust 最小原型验证三个核心假设：

| 假设 | 主张 |
|---|---|
| **H1** | 显式 context 分段（I/D/W）+ 智能 eviction 优于朴素 FIFO 截断 |
| **H2** | Ring 3 隔离 + tool 结果验证层能实际拦截 prompt injection |
| **H3** | 统一的数据访问接口（DAI）让 skill 开发更简单、行为更可预期 |

## 仓库结构

```
lithify/
├── obsidian/                    # 架构设计文档（Obsidian vault）
│   ├── Projects/lithify/        # Phase 1 规格、14 个模块设计文档
│   └── Ideas/                   # 原始探索笔记
├── src/                         # Rust 实现（Phase 1，进行中）
└── .claude/skills/lithify-rust/ # 项目级 Rust 编码规范
```

设计文档在 `obsidian/` 里，用 Obsidian 打开可获得完整的 wikilink 和 graph view 体验。

## 状态

Phase 1：设计完成，实现启动中。

## License

Apache-2.0
