# Feature: Sandbox

**作者**: -  
**日期**: 2026-05-07  
**状态**: Draft

---

## 1. 背景 (Background)
### 1.1 问题描述
- Agent 需要安全地执行 Shell 命令（文件读写、Git 操作、编译运行等），直接使用系统调用有风险。
- Phase 1 目标为**可用**而非完全隔离——通过子进程避免意外副作用即可，无需容器级安全。

### 1.2 现状分析
- `crates/core/src/lib.rs` 已定义 `Sandbox` trait（L193-202）、`ExecutionResult`（L94-100）、`SandboxError`（L125-130）。
- `crates/sandbox/src/lib.rs` 为空，仅一个注释。
- `crates/sandbox/Cargo.toml` 只依赖 `lithify-core`，尚未添加 `tokio`。
- ROADMAP 已明确要求：`tokio::process::Command`、capture stdout/stderr/exit code、超时控制、工作目录和环境变量隔离。

### 1.3 主要使用场景
- Agent 的 `ShellTool` 调用沙箱执行用户指令（如 `git status`、`cargo build`）。
- 文件读写工具通过沙箱指定工作目录操作。
- 未来 Phase 3 沙箱化执行生成的 Tool 代码。

## 2. 目标 (Goals)
- 实现 `Sandbox` trait 的最小可用形态（选项 A），通过 `tokio::process::Command` 安全执行 bash 命令，为 Task 4 的 ShellTool 提供执行环境。

### 2.1 非目标 (Non-Goals)
Phase 1 明确不做：
- 环境变量隔离（直接继承父进程 env）
- 工作目录跨调用记忆（无 `cd` 状态）
- stdin 传入
- 资源限制（CPU/内存/磁盘）
- Windows 支持

### 2.2 后续扩展（Phase 2+）
保留 B、C 两项作为未来迭代方向：

| 功能 | 优先级 | 描述 |
|------|--------|------|
| 环境变量控制 | 中 | 支持传入自定义环境变量，可选覆盖或追加 |
| 工作目录跨调用记忆 | 低 | `set_working_dir` / `current_dir` 方法 |
| stdin 传入支持 | 低 | 允许 Agent 传入 stdin 内容给子进程 |

## 3. 需求细化 (Requirements)
### 3.1 功能性需求

| 编号 | 功能 | 说明 |
|------|------|------|
| F1 | 实现 `Sandbox` trait | `SandboxImpl` 实现 `async fn run()` |
| F2 | 子进程执行 | `tokio::process::Command` 启动子进程 |
| F3 | 捕获输出 | 捕获 stdout、stderr、exit_code → `ExecutionResult` |
| F4 | 超时控制 | `tokio::time::timeout` 包裹，默认 300s，超时返回 `SandboxError::Timeout` |
| F5 | 工作目录 | `Option<&Path>`，每次调用独立指定，不跨调用记忆 |
### 3.2 非功能性需求

| 类别 | 要求 |
|------|------|
| 兼容性 | 严格遵循 `core::Sandbox` trait 已定义的签名，不可修改 |
| 测试 | `#[tokio::test]` 异步测试，用真实命令验证（echo、exit、sleep） |
| 超时行为 | 超时后子进程必须被终止（kill_on_drop），不留下僵尸进程 |

## 4. 设计方案 (Design)
### 4.1 方案概览
- `SandboxImpl` 单 struct，包含 `default_timeout: Duration`，`new(Duration)` 构造，`Default` 为 300s。
- `run()` 流程：`tokio::process::Command` 构建 → `kill_on_drop(true)` → `stdin(Stdio::null())` → spawn → `tokio::time::timeout` 包裹 `wait_with_output()` → 超时 drop 触发 kill。
- stdout/stderr 通过 `Stdio::piped()` 捕获，`wait_with_output()` 自动读取为 `Vec<u8>`。
- 超时后 `wait_with_output` future 被 drop，`Child::Drop` 由于 `kill_on_drop(true)` 自动向子进程发送 SIGKILL。
- stdin 设置为 `Stdio::null()` 防止命令误读父进程 stdin 导致阻塞。
- exit_code：正常退出取 `status.code()`，信号终止 fallback 为 -1。

### 4.2 组件设计 (Component Design)
#### 4.2.1 核心类/模块设计
- 单 struct `SandboxImpl`，含 `default_timeout: Duration`。
- 单文件 `lib.rs`，无子模块。

#### 4.2.2 接口设计
- `SandboxImpl::new(default_timeout: Duration) -> Self` — 构造函数
- `impl Default for SandboxImpl` — 默认 300s
- `impl Sandbox for SandboxImpl` — `async fn run(command, args, working_dir) -> Result<ExecutionResult, SandboxError>`

#### 4.2.3 数据模型
N/A — 所有类型（`ExecutionResult`、`SandboxError`）已在 `lithify-core` 定义，Sandbox 不新增数据模型。

#### 4.2.4 并发模型
N/A — 无共享状态，纯异步子进程执行，单次调用内不涉及多线程。

#### 4.2.5 错误处理
| 错误来源 | 映射 |
|---------|------|
| `std::io::Error`（spawn 失败） | `SandboxError::Command(e.to_string())` |
| `tokio::time::elapsed()`（超时） | `SandboxError::Timeout(timeout_ms)` |
| 进程非零退出 | 不是错误，正常返回 `ExecutionResult { exit_code: !0, stdout, stderr }` |

### 4.3 核心逻辑实现
```
async fn run(command, args, working_dir):
    cmd = tokio::process::Command::new(command)
         .args(args)
         .stdout(Stdio::piped())
         .stderr(Stdio::piped())
         .stdin(Stdio::null())                   // 防止命令误读父进程 stdin
         .kill_on_drop(true)                     // 超时后正确 kill 子进程
         [.current_dir(dir)]                     // if working_dir is Some
    child = cmd.spawn()                          // io error → Command
    match timeout(self.default_timeout, child.wait_with_output()).await:
        Ok(Ok(output)) → ExecutionResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into(),
            stderr: String::from_utf8_lossy(&output.stderr).into(),
        }
        Ok(Err(io_err)) → SandboxError::Command(io_err.to_string())
        Err(_elapsed)   → SandboxError::Timeout(default_timeout_ms)  // drop → kill_on_drop
```

### 4.4 方案优劣分析
**优点**：实现极简，单文件 ~40 行；tokio 异步不阻塞 runtime；Child::Drop 自动 kill 无需手动信号处理。
**局限**：无环境变量控制、无 stdin、无工作目录记忆（按 spec 非目标），后续迭代按 2.2 扩展表追加。
- 本方案的优点和局限性

## 5. 备选方案 (Alternatives Considered)
- **`std::process::Command`（同步）**：简单但会阻塞 tokio runtime。Trait 定义是 `async fn`，选 tokio 版是自然的。未采纳。

## 6. 业界调研 (Industry Research)
N/A — 基础子进程执行，不涉及分布式、协议或新算法。tokio::process 的 kill_on_drop 行为是 Rust 异步生态的通用做法。

## 7. 测试计划 (Test Plan)
### 7.1 单元测试

| 编号 | 测试用例 | 预期结果 |
|------|---------|---------|
| T1 | `echo hello` | exit_code=0, stdout 含 "hello", stderr="" |
| T2 | `bash -c "exit 3"` | exit_code=3 |
| T3 | `sleep 10`，timeout=1s | `SandboxError::Timeout` |
| T4 | 不存在的命令 | `SandboxError::Command` |
| T5 | `pwd`，指定 working_dir=/tmp | stdout 含 "/tmp" |
| T6 | 写入 stderr 的命令 | stderr 被正确捕获 |

### 7.2 集成测试
N/A — Sandbox 是底层组件，集成测试在 Tool crate 使用 Sandbox 时覆盖。

### 7.3 性能测试
N/A — MVP 阶段不涉及。

## 8. 可观测性 & 运维 (Observability & Operations)
N/A — MVP 阶段。Phase 2+ 按需追加执行耗时、超时次数等指标。

## 9. Changelog
| 日期 | 变更内容 | 作者 |
|------|----------|------|
| 2026-05-07 | 初始版本 | - |

## 10. 参考资料 (References)
- [tokio::process](https://docs.rs/tokio/latest/tokio/process/index.html)
- [Sandbox trait - lithify-core](crates/core/src/lib.rs)
