---
date: 2026-05-13
type: project
status: planning
tags: [project, lithify, security, identity, audit]
ai-first: true
---

## For future Claude
lithify 子模块设计文档——安全模型。涵盖 agent 身份认证、细粒度能力授权（capabilities）、信任链传播、不可篡改审计日志。核心问题：当前 agent 没有身份概念，subagent 权限可以无限放大。

---

## 问题陈述

当前 agent 安全模型几乎不存在：
- 没有 agent 身份——任何代码都可以声称是"可信 orchestrator"
- 权限只有 tool 级别（允许/不允许），缺乏细粒度控制
- subagent 调用链中权限可以无限放大（用户授权了 tool A → orchestrator 将权限传给子 agent → 子 agent 再传给子子 agent）
- 没有不可篡改的操作审计日志

---

## OS 类比

| OS 安全机制 | Agent 等价 |
|---|---|
| uid / gid | Agent Identity（签名身份） |
| Linux Capabilities | Agent Capability Set（细粒度权限） |
| `sudo` / `setuid` | 受限权限提升（需 Ring 0 授权） |
| `/etc/sudoers` | Capability Grant Policy |
| SELinux / AppArmor | Mandatory Access Control for agents |
| `auditd` | Immutable Agent Audit Log |
| 证书链（TLS） | Agent Trust Chain |

---

## 设计方案

### 1. Agent 身份模型

每个 agent 实例有一个不可伪造的身份，由 Harness Ring 0 在创建时颁发：

```json
{
  "agent_id": "uuid",
  "identity": {
    "skill": "code-review",
    "version": "1.2.3",
    "origin": "local | mcp_server | user_defined",
    "signature": "harness_signed_hash",
    "created_at": "ISO8601",
    "parent_id": "uuid"            // 谁 fork 了我
  }
}
```

- 身份由 Ring 0 签名，agent 自身不可伪造
- `parent_id` 建立了不可篡改的调用树，用于权限追溯

### 2. Capability-Based 权限模型

取代粗粒度的"allow/deny tool"，使用细粒度 capability 集合：

```
Capability 示例：
  FILE_READ            # 读取文件系统
  FILE_WRITE           # 写入文件系统
  FILE_WRITE_VAULT     # 只允许写入 vault 目录
  NETWORK_READ         # 发起 HTTP GET 请求
  NETWORK_WRITE        # 发起 HTTP POST/PUT 请求
  CODE_EXEC_SANDBOX    # 在沙箱中执行代码
  CODE_EXEC_HOST       # 在宿主机执行代码（高危）
  SPAWN_AGENT_RING3    # 创建 Ring 3 subagent
  SPAWN_AGENT_RING1    # 创建 Ring 1 subagent（需要 Ring 1 自身才能授权）
  CONTEXT_READ_GLOBAL  # 读取全局 context（仅 Ring 1）
```

每个 skill 声明所需的最小 capability 集合，用户在安装时审批。

### 3. 权限继承与不可放大原则

**核心规则：subagent 的 capability 集合不得超过父 agent**。

```
用户授权 orchestrator: {FILE_READ, NETWORK_READ, SPAWN_AGENT_RING3}

orchestrator 创建 subagent_A:
  ✅ 可以授予: {FILE_READ}           # 父集的子集
  ✅ 可以授予: {NETWORK_READ}
  ❌ 不能授予: {FILE_WRITE}          # 父 agent 自己都没有
  ❌ 不能授予: {FILE_READ, NETWORK_READ, FILE_WRITE}  # 超出父集
```

违反此规则的 spawn 请求由 Ring 0 拒绝，记入审计日志。

### 4. 不可篡改审计日志

所有安全相关操作写入 append-only 审计日志，由 Ring 0 维护：

```
[2026-05-13T14:23:01Z] AGENT_CREATED  agent:abc123 by agent:root  skill:code-review ring:3
[2026-05-13T14:23:02Z] TOOL_CALL      agent:abc123 tool:read_file path:/vault/index.md  ALLOWED
[2026-05-13T14:23:05Z] TOOL_CALL      agent:abc123 tool:bash cmd:"rm -rf /"  DENIED cap:CODE_EXEC_HOST
[2026-05-13T14:23:19Z] CAPABILITY_REQ agent:abc123 requested:FILE_WRITE  DENIED (not in parent set)
[2026-05-13T14:23:41Z] AGENT_EXIT     agent:abc123 status:DONE tokens:3041
```

审计日志特性：
- Ring 3 agent 无法读取或修改
- 支持按 agent_id、capability、时间范围查询（仅 Ring 0 / 授权管理员）
- 可以导出到外部系统（SIEM）

### 5. 信任链验证

当 MCP server 或外部 agent 加入系统时，需要建立信任链：

```
用户信任根
    └─ 信任: Harness Ring 0
         └─ 颁发身份: Orchestrator（Ring 1）
               └─ fork + 授权: Task Agent（Ring 3）
                     └─ fork + 授权: Sub-task Agent（Ring 3）
```

跨越信任边界（如引入外部 MCP server）需要用户显式授权，并限制其 capability 集合。

---

## 开放问题

- [ ] Capability 集合的粒度如何平衡安全性和可用性？过细会导致配置爆炸
- [ ] 审计日志的存储位置：本地 vault 还是需要外部不可篡改存储？
- [ ] 如何处理 skill 升级时的 capability 变更？需要用户重新审批吗？
- [ ] 多用户场景下的身份模型：不同用户的 agent 如何隔离？
