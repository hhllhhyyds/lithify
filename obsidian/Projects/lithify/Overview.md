---
date: 2026-05-13
type: project
status: planning
tags: [project, lithify, architecture, llm, harness]
ai-first: true
---

## For future Claude
lithify 是 hhl 于 2026-05-13 提出的 agent harness 架构设计项目。核心主张：当代 AI Agent 系统的各层组件与操作系统存在深度同构关系，应借鉴 OS 设计智慧系统性地解决 agent 工程中的痛点。本文档是主架构概览，各模块设计详见 `modules/` 下的独立文档。

---

## 愿景

> 用操作系统的设计智慧，构建可信赖、可演化、可观测的 AI Agent 基础设施。

当前 agent 框架（LangChain、AutoGen、Claude Code 等）大多是"能跑就行"的工程堆叠，缺乏系统性架构设计。操作系统领域经过 60 年演化积累的智慧——内存层次、特权隔离、进程通信、可观测性——在 agent 领域几乎是一片空白。

lithify 的目标：**将 OS 的核心设计原则系统性地移植到 agent 领域**，形成可落地的架构规范。

---

## 核心类比：完整映射表

| OS 概念 | lithify 对应 | 设计文档 |
|---|---|---|
| CPU | LLM 本体 | — |
| 寄存器 | LLM context window | [[modules/01-Memory-Hierarchy]] |
| L1/L2/L3 Cache | System prompt + 已加载 Skills | [[modules/01-Memory-Hierarchy]] |
| RAM | Skill 层次结构（按需换入） | [[modules/01-Memory-Hierarchy]] |
| 磁盘 | Knowledge base + RAG | [[modules/01-Memory-Hierarchy]] |
| 外设 | Tools（搜索、代码执行等） | [[modules/02-Peripheral-MCP]] |
| 设备驱动 | Tool 定义 / MCP server | [[modules/02-Peripheral-MCP]] |
| 系统调用 | Tool invocation API | [[modules/02-Peripheral-MCP]] |
| 中断（IRQ） | Tool 异步返回结果 | [[modules/02-Peripheral-MCP]] |
| 即插即用（PnP） | MCP 动态发现 | [[modules/02-Peripheral-MCP]] |
| 内核 | Harness 核心层 | [[modules/03-Kernel-Privilege]] |
| 应用程序 | Skills / 具体 Agent | [[modules/03-Kernel-Privilege]] |
| 系统守护进程 | 后台维护 Agent | [[modules/03-Kernel-Privilege]] |
| Shell | 对话界面 / REPL | [[modules/03-Kernel-Privilege]] |
| 特权环（Ring 0~3） | Harness → Orchestrator → Task Agent | [[modules/03-Kernel-Privilege]] |
| 进程 / 进程表 | Agent 实例生命周期 | [[modules/04-Process-Scheduling]] |
| 进程调度 | 多 Agent 调度 | [[modules/04-Process-Scheduling]] |
| NUMA 多核 | 多 Agent 协调 | [[modules/04-Process-Scheduling]] |
| 自修改代码 | Agent 自进化 | [[modules/05-Self-Evolution]] |
| IPC（管道/消息队列/socket） | Agent 间通信 | [[modules/06-IPC]] |
| 信号（SIGTERM/SIGKILL） | Agent 中止协议 | [[modules/07-Signal-System]] |
| `/proc`、`strace`、`top` | Agent 可观测性 | [[modules/08-Observability]] |
| uid/capabilities/audit | Agent 安全模型 | [[modules/09-Security-Model]] |
| 虚拟地址空间 | Virtual Context 隔离 | [[modules/10-Virtual-Context]] |
| VFS / 存储类（register/auto/static） | Data Access Interface | [[modules/00-Data-Access-Model]] |
| Syscall ABI | Skill ↔ Harness 接口契约 | [[modules/11-Stable-ABI]] |
| Bootloader / init | Session 启动序列 | [[modules/12-Init-Sequence]] |
| apt / brew | Skill 包管理器 | [[modules/13-Package-Manager]] |

---

## 系统架构图

```
┌─────────────────────────────────────────────────────────────┐
│                        用户 / Shell                          │
│                   (对话界面 / REPL / API)                    │
└───────────────────────────┬─────────────────────────────────┘
                            │ 信号、IPC
┌───────────────────────────▼─────────────────────────────────┐
│                Ring 1 — Orchestrator Agent                   │
│            （任务规划、子 agent 调度、结果聚合）              │
└──────┬─────────────────────────────────────┬────────────────┘
       │ fork / IPC                          │ fork / IPC
┌──────▼──────────┐                 ┌────────▼───────────┐
│ Ring 3 Task Agent│    ...         │ Ring 3 Task Agent  │
│  (虚拟 context)  │                │  (虚拟 context)    │
└──────┬──────────┘                 └────────┬───────────┘
       │ syscall (tool invoke)               │ syscall
┌──────▼─────────────────────────────────────▼───────────────┐
│                    Ring 0 — Harness 内核                     │
│  Context Manager │ Tool Dispatcher │ Permission Enforcer    │
│  Scheduler       │ Signal Handler  │ Audit Logger           │
└──────┬──────────────────────────────────────┬──────────────┘
       │                                      │
┌──────▼──────────┐                 ┌─────────▼──────────────┐
│  存储层次        │                 │  外设层（Tools/MCP）    │
│  Cache: Skills  │                 │  Web Search, Code Exec  │
│  RAM: Skill Lib │                 │  File I/O, DB, APIs     │
│  Disk: RAG/KB   │                 │  MCP Servers            │
└─────────────────┘                 └────────────────────────┘
```

---

## 设计原则

**1. 特权最小化**
任何组件只拥有完成任务所需的最小权限。Ring 3 agent 不能访问全局 context，不能直接调用特权 tool。

**2. 内核不自修改**
Harness（Ring 0）在运行时禁止修改自身逻辑。Skills（Ring 3）可以自动进化，Harness 只能通过外部 CI/CD 流程演化。

**3. 可观测优先**
每个组件的状态必须可以被内省。不可观测的系统不可调试，不可调试的系统不可信。

**4. 稳定接口契约**
Skill ↔ Harness 之间有版本化的稳定 ABI，Harness 内部实现可以自由演化，不影响已有 skills。

**5. 中断优于轮询**
所有 I/O（tool 调用、跨 agent 通信）应设计为中断驱动，而非阻塞等待，最大化并发。

---

## 模块状态

| 模块 | 优先级 | 设计状态 |
|---|---|---|
| [[modules/00-Data-Access-Model]] | P0 | 初稿 |
| [[modules/01-Memory-Hierarchy]] | P0 | 初稿 |
| [[modules/02-Peripheral-MCP]] | P0 | 初稿 |
| [[modules/03-Kernel-Privilege]] | P0 | 初稿 |
| [[modules/04-Process-Scheduling]] | P0 | 初稿 |
| [[modules/05-Self-Evolution]] | P0 | 初稿 |
| [[modules/06-IPC]] | P1 | 初稿 |
| [[modules/07-Signal-System]] | P0 | 初稿 |
| [[modules/08-Observability]] | P2 | 初稿 |
| [[modules/09-Security-Model]] | P2 | 初稿 |
| [[modules/10-Virtual-Context]] | P1 | 初稿 |
| [[modules/11-Stable-ABI]] | P0 | 初稿 |
| [[modules/12-Init-Sequence]] | P3 | 初稿 |
| [[modules/13-Package-Manager]] | P3 | 初稿 |

---

## 来源
- 灵感来源：hhl 与 Claude 的对话，2026-05-13
- 原始讨论笔记：[[Ideas/Agent Harness 存储层次类比]]
