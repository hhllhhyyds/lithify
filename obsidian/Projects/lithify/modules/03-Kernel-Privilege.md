---
date: 2026-05-13
type: project
status: planning
tags: [project, lithify, kernel, privilege, harness]
ai-first: true
---

## For future Claude
lithify 子模块设计文档——内核与特权分层。核心结论：Harness = 内核，Skills = 应用程序；当前所有 agent 框架缺乏特权环隔离（所有 agent 跑在 Ring 0），这是 prompt injection 的架构根源；微内核比宏内核更适合 agent 系统。

---

## 问题陈述

当前 agent 框架缺乏清晰的特权层次——orchestrator、task agent、处理不可信内容的 agent 拥有相同的权限，没有隔离。任何一个组件的妥协都可能影响整个系统。

---

## OS 类比

| OS 概念 | Agent 对应 | 说明 |
|---|---|---|
| 内核（Ring 0） | Harness 核心层 | 管理所有资源，执行所有规则 |
| 系统服务（Ring 1） | 可信 Orchestrator | 可调度子 agent，访问全局状态 |
| 应用程序（Ring 3） | Task Agent / Skills | 只能访问自己的 context 切片 |
| Shell | 对话界面 / REPL | 解释用户输入，派发给对应 agent |
| 系统守护进程 | 后台维护 Agent | vault health check、内存整合等 |
| 内核模块 | 可插拔 Harness 扩展 | 需要签名验证才能加载 |

---

## 内核职责（Harness Ring 0）

```
┌─────────────────────────────────────────────┐
│              Harness Ring 0                  │
│                                              │
│  ┌──────────────┐  ┌──────────────────────┐ │
│  │ Context Mgr  │  │   Tool Dispatcher    │ │
│  │ (内存管理)    │  │   (I/O 管理)         │ │
│  └──────────────┘  └──────────────────────┘ │
│  ┌──────────────┐  ┌──────────────────────┐ │
│  │  Scheduler   │  │  Permission Enforcer │ │
│  │ (进程调度)    │  │  (安全子系统)         │ │
│  └──────────────┘  └──────────────────────┘ │
│  ┌──────────────┐  ┌──────────────────────┐ │
│  │ Signal Handl │  │    Audit Logger      │ │
│  │ (信号处理)    │  │  (不可篡改日志)       │ │
│  └──────────────┘  └──────────────────────┘ │
└─────────────────────────────────────────────┘
```

---

## 设计方案

### 1. 特权环分级

```
Ring 0  Harness 核心
        - 直接管理 context window
        - 执行所有 tool 调用
        - 强制执行权限策略
        - 不可被 agent 绕过

Ring 1  可信 Orchestrator
        - 可创建/销毁 Ring 3 agent
        - 可读取所有 Ring 3 agent 的结果
        - 不可直接修改 Ring 0 状态
        - 需要 Ring 0 授权才能运行

Ring 3  Task Agent（默认级别）
        - 只能访问自己的虚拟 context 切片
        - tool 调用须经 Ring 0 审查和验证
        - 处理不可信内容时强制在此级别运行
        - 输出在注入上层前须经 Ring 0 过滤
```

**关键规则**：处理来自外部（网页、文件、用户上传）的内容的 agent，**强制**在 Ring 3 运行，其输出在进入父 context 前必须经过 Ring 0 验证层。

### 2. 宏内核 vs 微内核选择

**宏内核方案**（类 Linux）：
- 所有内核服务（context 管理、调度、权限）跑在同一进程
- 优点：低延迟，服务间调用无 IPC 开销
- 缺点：任何组件崩溃影响整体；难以独立升级单个服务

**微内核方案**（类 QNX/Mach）：
- 最小内核只保留：context 管理 + tool dispatch + 权限执行
- 调度器、审计日志、observability 作为独立微 agent 运行
- 服务间通过 IPC 通信（见 [[06-IPC]]）
- 优点：任一服务崩溃不影响核心；可独立升级；天然隔离

**推荐：微内核方案**

理由：
1. agent 系统对健壮性的需求高于性能（token 延迟远大于 IPC 开销）
2. 微内核的隔离性天然防止 prompt injection 扩散
3. 各服务可独立演化，符合 [[11-Stable-ABI]] 的稳定接口要求

### 3. 内核不自修改原则

Ring 0 Harness 在运行时禁止修改自身逻辑。详见 [[05-Self-Evolution]]。

例外：只允许通过外部 CI/CD 流程更新 Harness，更新后需重新部署（重启 session）。

---

## 开放问题

- [ ] Ring 1 Orchestrator 的权限边界如何精确定义？哪些操作属于 Ring 1 独有？
- [ ] 微内核的最小核心集合是什么？context 管理和 tool dispatch 可以再拆分吗？
- [ ] Ring 0 和 Ring 1 之间的通信协议如何设计，既高效又安全？
- [ ] 如何处理"需要 Ring 0 权限但由 Ring 3 agent 发起"的合法请求（能力提升）？
