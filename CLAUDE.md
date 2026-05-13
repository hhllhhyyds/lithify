# CLAUDE.md

> 本文件记录**当前实现状态**和开发约定。设计意图和架构原理在 `obsidian/` 里。

## 读这个项目

1. `obsidian/Projects/lithify/Overview.md` — 核心类比和系统架构图
2. `obsidian/Projects/lithify/Phase1-Spec.md` — Phase 1 实现规格（三个假设、6 个组件、验收标准）
3. 编码前读对应 `obsidian/Projects/lithify/modules/XX.md` — 具体模块设计

## 当前实现状态

Phase 1 实现尚未开始。

| 组件 | 文件 | 状态 |
|---|---|---|
| Claude API 封装 | `src/claude/client.rs` | 待实现 |
| Context Manager | `src/harness/context.rs` | 待实现 |
| Eviction 策略 | `src/harness/eviction.rs` | 待实现 |
| Tool 验证层 | `src/harness/validator.rs` | 待实现 |
| Data Access Interface | `src/harness/dai.rs` | 待实现 |
| Harness 核心 | `src/harness/core.rs` | 待实现 |
| Demo Skills × 2 | `src/skills/` | 待实现 |

## 开发约定

- **语言**：Rust 单 crate（非 workspace），`package.name = "lithify"`
- **Skill**：编写 Rust 代码时加载 `.claude/skills/lithify-rust/`（继承全局 `std-rust`）
- **提交前**：运行 `scripts/check.sh`（fmt → clippy → test → coverage → doc）
- **真实 LLM 测试**：用 `#[ignore]` 标注，手动运行 `cargo test -- --ignored`
- **API 密钥**：Claude API key 在 `.env`（不提交）

## Docs-Code 同步规则

- 设计有变化 → 先改 `obsidian/`，再改代码
- 模块实现完成 → 更新 `obsidian/Projects/lithify/Overview.md` 模块状态表
- 实现与设计有重大偏差 → 在对应 `modules/XX.md` 末尾写 ADR

完整协议见 `obsidian/Projects/lithify/Docs-Code-Protocol.md`。
