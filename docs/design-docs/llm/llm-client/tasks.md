# 实施任务清单

> 由 spec.md 生成
> 任务总数: 3
> 核心原则: 基础设施优先——先建 Config 和依赖，再建 Convert，最后组装 Client

## 依赖关系总览

```
Task 1 (依赖 + Config + lib.rs)
  ├── Task 2 (Convert 模块)  ← 可并行于 Task 1
  │
  └── Task 3 (Client + 测试)  ← 依赖 Task 1, 2
```

Task 1 与 Task 2 无依赖关系，理论可并行。Task 3 依赖 Task 1 和 2 的产出。

## 变更影响概览

### 文件变更清单

| 文件 | 操作 | 涉及任务 | 说明 |
|------|------|---------|------|
| `crates/llm/Cargo.toml` | 修改 | Task 1 | 新增 reqwest, tracing, tokio 等依赖 |
| `crates/llm/src/config.rs` | 新建 | Task 1 | Config struct + from_env() |
| `crates/llm/src/lib.rs` | 修改 | Task 1 | 模块声明 |
| `crates/llm/src/convert.rs` | 新建 | Task 2 | 类型转换函数 |
| `crates/llm/src/client.rs` | 新建 | Task 3 | AnthropicClient + LLMClient 实现 |
| `scripts/test_real_llm.sh` | 新建 | Task 3 | 真实 LLM 测试脚本 |

### 受影响接口

| 接口 | 变更类型 | 涉及任务 |
|------|---------|---------|
| `lithify-llm` lib.rs | 新增 pub mod | Task 1 |
| 新增 `Config` struct | 新增 pub | Task 1 |
| 新增 `AnthropicClient` struct | 新增 pub | Task 3 |
| 新增 `LLMClient for AnthropicClient` | 新增 impl | Task 3 |

### 构建系统变更

- `crates/llm/Cargo.toml`：新增 `reqwest`, `tracing`, `tokio`, `serde_json` 依赖（Task 1）

## 风险与假设

| # | 描述 | 影响任务 | 假设/处理 |
|---|------|---------|----------|
| 1 | Anthropic API 响应格式可能变更 | Task 2, 3 | 假设遵循 Messages API 当前版本格式 |
| 2 | 真实 LLM 测试需要 API Key | Task 3 | 通过 `#[ignore]` + 本地脚本，CI 中跳过 |

## 任务列表

### 任务 1: [x] 依赖 + Config + lib.rs
- 依赖: 无
- spec 映射: 3.1 Config 模块, 3.2 超时控制
- dev_plan: `dev_plan/01-deps-config-lib.md` (dev_plan/ 与 tasks.md 同目录)

### 任务 2: [x] Convert 模块 + 单元测试
- 依赖: 无（只依赖 core 已有类型）
- spec 映射: 3.1 Convert 模块
- dev_plan: `dev_plan/02-convert.md` (dev_plan/ 与 tasks.md 同目录)

### 任务 3: [x] Client 实现 + 测试
- 依赖: Task 1, Task 2
- spec 映射: 3.1 Client 模块, 3.1 错误处理, 3.1 真实 LLM 测试, 3.2 日志
- dev_plan: `dev_plan/03-client.md`

## Spec 覆盖映射

| Spec 章节 | 任务 | 说明 |
|-----------|------|------|
| 3.1 Config 模块 | Task 1 | Config + from_env |
| 3.1 Convert 模块 | Task 2 | 类型转换 |
| 3.1 Client 模块 | Task 3 | AnthropicClient + LLMClient |
| 3.1 错误处理 | Task 3 | HTTP 错误映射到 LLMError |
| 3.1 真实 LLM 测试 | Task 3 | #[ignore] 测试 + 脚本 |
| 3.2 超时控制 | Task 1 | Config 中定义 timeout 字段 |
| 3.2 API Key 安全 | Task 1 | Config::from_env 从环境变量读取 |
| 3.2 日志 | Task 3 | tracing 日志 |
| 4.1-4.4 所有设计 | Task 1, 2, 3 | 按模块划分覆盖 |
