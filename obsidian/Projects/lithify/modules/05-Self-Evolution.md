---
date: 2026-05-13
type: project
status: planning
tags: [project, lithify, evolution, self-improvement, skills]
ai-first: true
---

## For future Claude
lithify 子模块设计文档——自进化机制。核心原则：进化权限与特权级反向绑定；Skills（用户层）全自动进化，Harness（内核层）只能通过外部 CI/CD 演化。关键类比：体细胞突变（skills）vs 生殖细胞突变（harness 源码）。

---

## 问题陈述

"用得越多越好用"是 agent 系统的理想目标，但自进化能力引入了严重的安全风险：一个能修改自身内核的 agent 可以删除自己的安全约束。需要一个分层的自进化架构，在实现进化收益的同时保持安全边界。

---

## OS 类比

| OS 自修改机制 | Agent 对应 | 风险级别 |
|---|---|---|
| 应用自动更新 | Skills 自动更新 | 低 |
| `sysctl` 参数调整 | Harness 配置调优 | 中 |
| 内核模块加载（signed） | 受信任的 Harness 扩展 | 中高 |
| 内核重编译/替换 | Harness 源码修改 | 极高 |
| 自修改代码（禁止） | 运行时修改自身逻辑 | 禁止 |

**生物类比（更直观）**：
- Skills 进化 = **体细胞突变**：影响当前实例，自动，可逆
- Harness 源码进化 = **生殖细胞突变**：影响所有未来实例，必须外部审查

---

## 设计方案

### 三层进化架构

```
Layer 3: Skills（用户层）     → 全自动，高频，低风险
           ↑  propose changes
Layer 2: Harness 配置（sysctl）→ 半自动，需人工确认
           ↑  propose changes
Layer 1: Harness 源码（CI/CD）→ 人工主导，外部流程
```

**核心原则**：进化权限与特权级反向绑定。Ring 3 自由进化，Ring 0 禁止运行时自修改。

---

### Layer 3：Skills 自动进化（全自动）

Skills 运行在用户层（Ring 3），自进化不需要内核权限。

**进化信号来源**：
- 用户显式反馈（"这个回答很好" / "不对，重来"）
- 隐式信号（task 完成率、重试次数、用户修改了 agent 输出）
- A/B 测试（同一任务用新旧版本 skill 各跑一次，比较结果）

**进化机制**：
```
对话结束后 → Harness 收集进化信号
           → 生成 skill patch（调整 prompt、示例、触发条件）
           → 在 Ring 3 sandbox 中测试 patch
           → 测试通过 → 自动部署新版本 skill
           → 失败 → 回滚，记录失败原因
```

**版本管理**：每个 skill 维护版本历史，可随时回滚（见 [[13-Package-Manager]]）。

---

### Layer 2：Harness 配置调优（sysctl 层，半自动）

可调参数（举例）：

| 参数 | 说明 | 默认值 |
|---|---|---|
| `context.eviction_policy` | LRU / LFU / learned | LRU |
| `context.i_segment_ratio` | I-segment 占 context 的比例上限 | 0.3 |
| `scheduler.default_budget` | 默认 token budget per agent | 8192 |
| `tool.timeout_ms` | Tool 调用超时 | 30000 |
| `ring3.max_output_size` | Ring 3 agent 输出注入上层的大小限制 | 4096 |

**调优流程**：
1. Agent 运行时收集性能指标（task 成功率、context 利用率等）
2. 检测到可改进的配置 → 生成调优建议
3. 用户确认 → 生效（无需重启 session）
4. 观察效果 → 若指标恶化 → 自动回滚

---

### Layer 1：Harness 源码演化（外部 CI/CD）

Harness（Ring 0）**不得**在运行时修改自身逻辑。

正确流程：
```
Agent 发现潜在改进 → 生成 GitHub Issue / PR → 人工 Review
                  → 测试套件通过 → Merge → 构建新版本
                  → 部署新版本（新 session 生效）
```

Agent 可以**贡献**进化材料（bug report、改进建议、PR），但**不执行**这些修改。执行权在人类和 CI/CD 流程。

---

## 安全约束

- Ring 0 组件的任何修改必须经过外部签名验证
- sysctl 参数有明确的安全边界（不可通过参数调整绕过 Ring 3 隔离）
- Skills 进化不得修改 I-segment 中的核心安全规则
- 所有进化操作写入不可篡改的审计日志（见 [[09-Security-Model]]）

---

## 开放问题

- [ ] Skills 进化的测试沙箱如何设计？隔离程度要求？
- [ ] 进化信号的权重如何设计？显式反馈 vs 隐式信号的可信度差异？
- [ ] sysctl 参数的安全边界如何形式化？防止通过合法参数绕过安全约束？
- [ ] 多用户场景下，skills 进化是全局共享还是用户私有？
