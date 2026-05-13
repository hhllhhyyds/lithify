---
date: 2026-05-13
type: project
status: planning
tags: [project, lithify, dev-skills, toolchain, rust]
ai-first: true
---

## For future Claude
lithify Phase 1 开发技能组文档，记录本项目使用的 AI Coding Agent 框架、skill 规范体系、以及各开发阶段的 skill 使用策略。开发框架为 hhllhhyyds/agentic-engineering-framework（fork 自 davidYichengWei/agentic-engineering-framework）。需要为项目创建 `lithify-rust` 项目级 skill，继承全局 `std-rust` 并补充 lithify 特有约定。

---

## 开发框架

**仓库**：https://github.com/hhllhhyyds/agentic-engineering-framework

**结构**：
```
agents/    → Subagent 定义（专项 reviewer，由 workflow skill 调度）
commands/  → 用户触发入口（/code-generation、/code-review 等）
skills/    → 具体工作流和知识定义（按需加载）
```

**安装**：`cp -R agents skills commands ~/.claude/`

---

## Skill 规范三层继承

```
std-rust（全局）
    └─ lithify-rust（项目级，本项目使用）   ← 待创建
    └─ lithify-rust（Lithify 项目，参考）
```

- `std-rust`：通用 Rust 规范，全局加载
- `lithify-rust`：仅写与 `std-rust` 的差异，放在项目 `.claude/skills/lithify-rust/SKILL.md`
- `lithify-rust` 位于 `/Users/hhl/Documents/projs/lithify/.claude/skills/lithify-rust/SKILL.md`，用于参考

---

## std-rust 内容速览

全局通用规范，已覆盖：
- 命名规范（CamelCase / snake_case / SCREAMING_SNAKE_CASE）
- 错误处理：library 用 `thiserror`，app 用 `anyhow`；禁用 `unwrap()`
- 异步：`tokio` + `#[async_trait]`；trait 对象需 `Send + Sync`
- 文档：所有 `pub` 项必须有英文 `///` doc comment
- 测试：TDD（Red/Green/Refactor）+ `mockall` + 集成测试在 `tests/`
- Commit 规范：`feat/fix/docs/test/refactor/chore(<scope>): <desc>`
- 提交前：`cargo test && cargo clippy -- -D warnings && cargo fmt --check`

---

## lithify-rust 需要补充的内容

### 从 lithify-rust 借鉴

| 规则 | 说明 |
|---|---|
| `tracing` 做日志 | 观测性模块基础；关键分支覆盖 `info!`/`warn!`/`error!` |
| `wiremock` mock Claude API | 测试时不能每次真实调 API，成本高且不稳定 |
| `#[ignore]` 标注真实 LLM 测试 | H1/H2/H3 验收实验需要真实 LLM，但 CI 不跑 |
| 覆盖率不得下降 | 保证验收测试质量，使用 `cargo llvm-cov` |
| `scripts/check.sh` 五步检查 | fmt → clippy → test → coverage → doc |
| Git 工作流 | rebase + `git push --force-with-lease` |

### lithify 特有规则

**① 单 crate 结构（非 workspace）**

Phase 1 是单 crate，不使用 `[workspace.dependencies]`。`std-rust` 的 workspace 规范暂不适用。

**② Claude Tool Use 多轮循环规范**

```rust
// 固定模式，所有 tool use 调用必须遵循此结构
loop {
    let resp = self.claude.messages(messages.clone()).await?;
    match resp.stop_reason {
        StopReason::ToolUse => {
            let tool_results = self.execute_tools(&resp.content).await?;
            messages.push(Message::assistant(resp.content));
            messages.push(Message::tool_results(tool_results));
        }
        StopReason::EndTurn => {
            break resp.content;
        }
    }
}
```

**③ `tiktoken-rs` token 计数约定**

```rust
// encoding 统一使用 cl100k_base
// tokenizer 用 OnceLock 单例，避免重复初始化
static TOKENIZER: OnceLock<CoreBPE> = OnceLock::new();

fn tokenizer() -> &'static CoreBPE {
    TOKENIZER.get_or_init(|| tiktoken_rs::cl100k_base().unwrap())
}

pub fn count_tokens(text: &str) -> usize {
    tokenizer().encode_ordinary(text).len()
}
```

**④ EvictionPolicy trait 使用约定**

`EvictionPolicy` 实现须为 `Send + Sync`，通过 `Arc<dyn EvictionPolicy>` 持有：

```rust
pub struct ContextManager {
    policy: Arc<dyn EvictionPolicy>,
    // ...
}
```

**⑤ Ring 分级的运行时检查**

Ring 3 agent 调用受限操作时，Harness 必须在运行时 panic 而非静默放行：

```rust
fn assert_ring_permission(caller: Ring, required: Ring) {
    assert!(
        caller <= required,
        "Ring violation: {:?} attempted operation requiring {:?}",
        caller, required
    );
}
```

---

## 开发阶段 Skill 使用策略

Phase 1 已完成需求澄清（Phase1-Spec.md）和系统设计（Overview + modules）阶段。剩余阶段：

### 编码阶段（每个 Rust 模块）

```
/code-generation     加载 std-rust + lithify-rust + bp-coding-best-practices
                     按 Phase1-Spec.md 逐模块实现
```

推荐实现顺序（按依赖关系）：
1. `src/claude/client.rs`（Claude API 封装，其他模块依赖）
2. `src/harness/context.rs`（ContextManager + EvictionPolicy trait）
3. `src/harness/eviction.rs`（Fifo / Lru / Semantic 三种实现）
4. `src/harness/validator.rs`（ToolResultValidator）
5. `src/harness/dai.rs`（DataAccessInterface）
6. `src/harness/core.rs`（AgentHarness，组装所有组件）
7. `src/skills/`（Demo Skills × 2）
8. `src/main.rs`（入口）

### 测试阶段

```
/test-generation     基于 Phase1-Spec.md 的验收标准生成测试
                     H1：eviction_test.rs（对比 FIFO/LRU/SEMANTIC）
                     H2：injection_test.rs（构造 prompt injection 样本）
                     H3：dai_test.rs（DAI vs 裸代码对比）
```

真实 LLM 测试用 `#[ignore]` 标注，手动运行：
```bash
cargo test -- --ignored   # 运行真实 LLM 测试
```

### Review 阶段

```
/code-review         7 个专项 subagent 并行审查
                     重点关注：ToolResultValidator（安全关键）、ContextManager（性能关键）
```

### 提交前

```bash
scripts/check.sh     fmt → clippy → test → coverage → doc
```

### 迭代总结

```
/reflect             每次 session 结束，纠错经验沉淀回 lithify-rust skill
```

---

## 待办

- [ ] 创建 `lithify-rust` SKILL.md（项目 `.claude/skills/lithify-rust/SKILL.md`）
- [ ] 创建 `scripts/check.sh`（参考 lithify 的五步检查）
- [ ] 初始化 Cargo 项目（`cargo new lithify --name lithify`）

---

## 关联文档
- Phase 1 规格：[[Phase1-Spec]]
- 主架构：[[Overview]]
- 参考 skill 路径：`/Users/hhl/Documents/projs/lithify/.claude/skills/lithify-rust/SKILL.md`
