---
date: 2026-05-13
type: project
status: planning
tags: [project, lithify, package-manager, skills, distribution]
ai-first: true
---

## For future Claude
lithify 子模块设计文档——Skill 包管理器。类比 apt/brew，设计 skill 的发现、安装、版本管理、依赖解析、安全验证机制。当前 skills 是手动管理的文件，无法规模化。

---

## 问题陈述

当前 skill 管理完全手工：
- 从某处复制 skill 文件到 `~/.claude/skills/`
- 无版本管理，升级即覆盖，无法回滚
- 无依赖声明，skill A 依赖 skill B 时用户需手动解决
- 无安全验证，任何文件都可以声称是"skill"
- Skill 生态无法规模化

---

## OS 类比

| apt / brew 功能 | Skill 包管理器等价 |
|---|---|
| 软件包仓库（apt repository） | Skill Registry |
| 包描述文件（`Package` / `Formula`） | `skill.yaml` manifest |
| 依赖解析（`apt-get install` resolves deps） | Skill dependency resolver |
| 包签名验证（GPG key） | Skill signature verification |
| 版本锁文件（`poetry.lock`） | `skills.lock` |
| 沙箱安装（`brew` in `/usr/local`） | Skill 隔离安装目录 |
| `apt upgrade` | `claude skill upgrade` |

---

## 设计方案

### 1. Skill Manifest（`skill.yaml`）

每个 skill 包含 manifest 文件，声明元数据和依赖：

```yaml
# skill.yaml
name: code-review
version: 1.2.3
description: "审查代码变更，提供 review 意见"
author: "anthropic"
license: MIT

abi-version: ">=2.0, <3.0"         # 见模块 11

capabilities:                        # 见模块 09
  - FILE_READ
  - NETWORK_READ

dependencies:
  skills:
    - name: "troubleshooting"
      version: ">=1.0.0"
  optional-skills:
    - name: "obsidian-save"
      version: ">=2.0.0"

triggers:                            # 何时自动触发此 skill
  - pattern: "review.*code"
  - pattern: "code.*review"

entry: "SKILL.md"
tests: "tests/"
signature: "sha256:abc123..."        # 发布者签名
```

### 2. Skill Registry

类比 apt repository 的中央注册表：

```
官方 Registry:    registry.claude.ai/skills
社区 Registry:    github.com/user/claude-skills
私有 Registry:    本地目录 / 企业内网

优先级：本地私有 > 企业私有 > 社区 > 官方
```

### 3. 核心命令

```bash
# 搜索
claude skill search "code review"

# 安装（自动解析依赖）
claude skill install code-review

# 升级
claude skill upgrade                    # 升级所有
claude skill upgrade code-review        # 升级指定

# 回滚
claude skill rollback code-review 1.1.0

# 列出已安装
claude skill list

# 查看依赖树
claude skill deps code-review

# 验证签名
claude skill verify code-review
```

### 4. 依赖解析

类比 poetry / cargo 的依赖解析：

```
安装 workflow-code-generation@2.0.0
    depends on: workflow-requirements-clarification@>=1.5.0
                bp-coding-best-practices@>=1.0.0
    └─ 解析 workflow-requirements-clarification@1.6.2
            depends on: (无额外依赖)
    └─ 解析 bp-coding-best-practices@1.2.0
            depends on: (无额外依赖)

生成 skills.lock：
  workflow-code-generation: 2.0.0
  workflow-requirements-clarification: 1.6.2
  bp-coding-best-practices: 1.2.0
```

`skills.lock` 确保环境可重现——同一个 lock 文件在任何机器上安装出完全相同的 skill 集合。

### 5. 安全验证

三级验证：

| 级别 | 验证内容 | 失败行为 |
|---|---|---|
| 签名验证 | 发布者的 GPG/Ed25519 签名 | 拒绝安装 |
| Capability 审计 | 实际 capability 使用是否超出声明 | 警告 + 用户确认 |
| Sandbox 测试 | 在隔离环境运行 skill 自带的测试套件 | 警告，可强制跳过 |

未签名 skill 默认不允许安装，可通过 `--allow-unsigned` 强制（记入审计日志）。

### 6. Skill 自进化与包管理的整合

Skills 自动进化（见 [[05-Self-Evolution]]）产生的新版本，通过包管理器发布：

```
Skill 进化 patch 生成
    ↓
本地测试通过
    ↓
bump version（patch 级别：1.2.3 → 1.2.4）
    ↓
签名 + 发布到本地/私有 Registry
    ↓
下次 session 启动时 Stage 2 检测到更新，提示升级
```

---

## 开放问题

- [ ] 官方 Registry 的治理模型：谁来审核社区提交的 skills？
- [ ] Skill 沙箱测试的隔离级别：需要真实的 LLM 调用还是 mock？成本问题？
- [ ] 企业私有 Registry 的认证机制：token？mTLS？
- [ ] Skill 依赖的钻石问题（diamond dependency）如何解决？
