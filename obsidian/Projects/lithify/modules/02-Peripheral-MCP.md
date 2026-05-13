---
date: 2026-05-13
type: project
status: planning
tags: [project, lithify, tools, mcp, peripheral]
ai-first: true
---

## For future Claude
lithify 子模块设计文档——外设与 MCP。描述 Tools/MCP 与 OS 外设体系的映射，核心结论：MCP = USB 标准，当前 tool call 是低效的 PIO 模式应改为中断驱动，以及 tool 返回值缺乏内核验证层导致 prompt injection 的架构根源。

---

## 问题陈述

Agent 通过 tools 与外部世界交互，但当前设计存在三个根本问题：
1. Tool 接口标准碎片化（各框架格式不同）
2. Tool 调用是同步阻塞的，严重限制并发
3. Tool 返回值直接注入 context，无安全验证层

---

## OS 类比

| OS 外设体系 | Agent 对应 | 说明 |
|---|---|---|
| 外设（键盘、磁盘、网卡） | Tools（搜索、代码执行、文件读写） | 外部能力，非 LLM 原生 |
| 设备驱动 | Tool 定义 / MCP server connector | 统一接口，屏蔽底层实现 |
| 系统调用 | Tool invocation API | 受控的能力穿越边界 |
| 中断（IRQ） | Tool 返回结果 | 外部事件异步通知 |
| 即插即用（PnP） | MCP 动态发现 | 运行时发现工具，无需硬编码 |
| USB 标准 | MCP 协议 | 统一的连接标准 |

---

## 设计方案

### 1. MCP 作为 USB 标准

USB 统一了设备连接标准，MCP 正在做同样的事。**但 MCP 需要演化出性能层级**，类似 USB 2.0/3.0/4.0：

| 层级 | 适用场景 | 延迟目标 |
|---|---|---|
| MCP Local | 本地进程内 tool（代码执行、文件读写） | < 10ms |
| MCP IPC | 本机跨进程 MCP server | < 100ms |
| MCP Remote | 远程 HTTP MCP server | < 1s |

不同层级走不同"总线"，调度策略不同。

### 2. 中断驱动 I/O（替代当前的 PIO 模式）

**当前（PIO 模式）**：
```
LLM 生成 → 停止 → 调用 tool → 等待结果 → 继续生成
```
LLM 在等待 tool 期间完全空闲，相当于 CPU 用轮询方式等待磁盘。

**目标（中断驱动）**：
```
LLM 生成 → 发出 tool 调用请求 → 继续处理其他子任务
                                    ↓ tool 完成（中断）
                               结果注入 context → LLM 处理结果
```

实现要求：
- Harness 维护一个异步 tool 执行队列
- LLM 在 tool 执行期间可以处理其他并行子任务
- Tool 结果以异步事件的形式注入 context

### 3. Tool 结果验证层（内核边界）

**当前问题**：tool 返回值直接注入 LLM context，无过滤。恶意 tool 结果可以注入指令，劫持 agent 行为（prompt injection）。

**设计方案**：在 Ring 0（Harness）内设置 tool 结果验证层：

```
Tool 执行结果
     ↓
┌─────────────────────────────┐
│    Tool Result Validator    │  ← Ring 0 内核组件
│  - 内容类型检查              │
│  - 大小限制                  │
│  - 指令注入检测               │
│  - 来源信任级别标注           │
└─────────────────────────────┘
     ↓ 安全结果
注入 Context（D-segment）
```

验证层输出的结果必须附带**来源信任级别**标注，LLM 在处理时可以感知数据可信度。

### 4. Tool 权限模型

基于来源和风险级别的四级权限：

| 级别 | 示例 | 默认行为 |
|---|---|---|
| Trusted | 内置 read/write | 自动执行 |
| Reviewed | 用户安装的 MCP server | 首次询问，后续自动 |
| Sandboxed | 处理不可信内容的 tool | 每次询问，结果隔离 |
| Blocked | 被明确禁止的操作 | 拒绝执行 |

---

## 开放问题

- [ ] 中断驱动模型中，多个并发 tool 的结果如何有序地注入 context？
- [ ] Tool result validator 的规则集如何维护？静态规则还是动态学习？
- [ ] MCP 性能层级的自动选择策略：如何检测 MCP server 的实际延迟并动态路由？
- [ ] Sandboxed tool 的 context 隔离如何与 [[10-Virtual-Context]] 的设计整合？
