---
name: lithify-rust
description: Lithify Rust 开发规范。当编写或 review Rust 代码（.rs 文件）时使用。
---

# Lithify Rust 开发规范

> 基于 Rust 2024 edition + Lithify 项目约定。
> 通用标准参见全局 `std-rust` skill，本文档仅列出 Lithify 项目特定的规则和流程。

## 项目特有规则

| 类别 | 规则 |
|------|------|
| **提交前检查** | 必须运行 `scripts/check.sh`（包含 fmt → clippy → test → coverage → doc 五步），全部通过后方可提交 |
| **覆盖率红线** | 新增代码必须覆盖对应测试用例，整体覆盖率不得下降。若覆盖率下降，必须补充测试至不低于原覆盖率 |
| **错误处理** | trait 内用 `thiserror` 定义具体错误类型，应用层用 `anyhow` |
| **测试策略** | 单元测试在 `#[cfg(test)] mod tests` 内；HTTP mock 用 `wiremock`；真实 LLM 测试用 `#[ignore]` + 本地脚本 |
| **异步** | 统一使用 `tokio` + `#[async_trait]`；trait 对象需 `Send + Sync` |
| **依赖管理** | 公共依赖在 `[workspace.dependencies]` 统一管理，各 crate 按需启用 features |
| **文档注释** | 所有 `pub` 项必须有英文 `///` doc comment；设计文档用中文，代码注释用英文 |
| **日志** | 使用 `tracing` crate，关键分支覆盖 `info!`/`warn!`/`error!` |

## 提交前检查流程

修改任何代码或文档后，必须运行：

```bash
scripts/check.sh
```

该脚本依次执行：
1. **fmt** — `cargo fmt --check`（格式化检查）
2. **clippy** — `cargo clippy -- -D warnings`（静态分析）
3. **test** — `cargo test`（单元 + 集成测试）
4. **coverage** — `cargo llvm-cov`（代码覆盖率。新增代码必须有对应测试，整体覆盖率不得低于提交前基线，否则需补充测试）
5. **doc** — `cargo doc --no-deps`（文档生成）

五项全部通过后方可提交。

## Git 工作流

| 步骤 | 命令 | 说明 |
|------|------|------|
| **Push 前 rebase** | `git fetch origin main && git rebase origin/main` | 确保基于远端最新 main 重放本地 commit，避免分叉。不要依赖本地 main，可能过期 |
| **Force push** | `git push --force-with-lease` | rebase 后需要 force push，使用 `--force-with-lease` 而非 `--force` 以防止覆盖他人提交 |
