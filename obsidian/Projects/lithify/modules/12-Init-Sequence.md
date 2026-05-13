---
date: 2026-05-13
type: project
status: planning
tags: [project, lithify, init, bootloader, session]
ai-first: true
---

## For future Claude
lithify 子模块设计文档——Init 序列。类比 BIOS → bootloader → kernel → init → services 的启动序列，设计 agent session 的确定性启动流程。解决当前 session 初始状态不确定、崩溃后无法恢复的问题。

---

## 问题陈述

当前 agent session 的启动是隐式的：system prompt 如何加载、skills 以什么顺序初始化、上次 session 的状态如何恢复——全都没有明确规范。Session 崩溃后，没有标准的检查点恢复机制。

---

## OS 类比

| OS 启动阶段 | Agent Session 等价 |
|---|---|
| BIOS / UEFI | Harness 自检（检查 ABI 兼容性、资源可用性） |
| Bootloader | Session 初始化器（决定加载哪个配置） |
| Kernel 加载 | Harness Ring 0 启动（context 管理器就绪） |
| init / systemd | Orchestrator 启动，加载 skill 依赖树 |
| 系统服务启动 | 后台 agent 启动（vault health、index agent 等） |
| 登录提示 | Session Ready（用户可以开始交互） |

---

## 设计方案

### 1. 五阶段启动序列

```
Stage 0: Harness Self-Check（类 BIOS POST）
  - 检查 ABI 版本兼容性
  - 验证所有已安装 skill 的签名
  - 检查 vault 挂载状态
  - 失败 → 启动失败，报告错误，不进入 Stage 1

Stage 1: Core Context 初始化（类 Kernel Load）
  - 加载 _CLAUDE.md（vault 操作手册）
  - 初始化 context 段管理器
  - 建立 I-segment：系统规则、安全约束
  - 建立 audit log 端点

Stage 2: Skill 依赖树解析（类 systemd dependency resolution）
  - 读取 session profile（用户偏好的 skill 集合）
  - 解析 skill 间依赖（skill A depends on skill B）
  - 按拓扑顺序加载 skills 到 cache
  - 依赖缺失 → 警告但不阻塞（降级运行）

Stage 3: 状态恢复（类 fsck + mount）
  - 检查是否有上次 session 的检查点（见 [[07-Signal-System]]）
  - 有检查点 → 提示用户是否恢复
  - 恢复 → 重建 context 段，从检查点继续
  - 不恢复 / 无检查点 → 全新 session

Stage 4: 后台 Agent 启动（类 systemd services）
  - 启动 vault health check agent（如配置）
  - 启动 index 维护 agent（如配置）
  - 启动用户定义的 always-on agents

Stage 5: Session Ready
  - 发布 session.ready 事件（见 [[06-IPC]]）
  - 开始接受用户输入
```

### 2. Session Profile（类 `/etc/systemd/system/`）

用户可以定义不同的 session 配置文件，类比 systemd target：

```yaml
# .claude/profiles/dev.yaml
name: "Dev Session"
skills:
  - code-review
  - obsidian-save
  - troubleshooting
background-agents:
  - vault-health: { schedule: "1h" }
context:
  i_segment_ratio: 0.25
  default_budget: 16384
vault: /Users/hhl/Documents/Ideas/spark
```

启动时选择 profile：`claude --profile dev`

### 3. 崩溃恢复

Session 意外中断时的恢复流程：

```
正常运行中 → Harness 每 N 次推理自动保存 mini-checkpoint
崩溃 / 强制退出
    ↓
下次启动 Stage 3 检测到 checkpoint
    ↓
显示恢复提示：
  "检测到上次未完成的会话（2026-05-13 14:23）
   任务：review PR #42
   进度：已完成 2/5 个文件
   是否恢复？[Y/n]"
    ↓
Y → 恢复 context 段 + agent 状态 → 继续执行
N → 丢弃 checkpoint → 全新 session
```

### 4. 启动时间优化

类比操作系统的启动时间优化：

- **Lazy loading**：不预加载所有 skills，只加载 session profile 中声明的 + 按需加载
- **Skill 预编译**：将常用 skill 解析结果缓存到磁盘，避免每次重新解析
- **并行 Stage**：Stage 2（skill 加载）和 Stage 3（状态恢复）可以并行执行

---

## 开放问题

- [ ] Stage 0 的 skill 签名验证：签名颁发机构是谁？用户自定义 skill 如何处理？
- [ ] Checkpoint 的存储格式：需要对 LLM context 快照做压缩吗？
- [ ] Session profile 的继承机制：能否基于 default profile 做差异化配置？
- [ ] 多 vault 场景下，init 序列如何处理 vault 切换？
