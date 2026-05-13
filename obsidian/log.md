# Vault Log

> 每次 Claude 对 vault 执行操作时追加一条记录。
> 格式: `## [YYYY-MM-DD] 操作类型 | 描述`
> 只追加，不修改历史记录。

---

## [2026-05-13] init | Vault 初始化，生成 _CLAUDE.md、index.md、log.md
## [2026-05-13] delete | 删除 欢迎.md（默认欢迎页）；更新 vault 用途为"技术灵感记录"
## [2026-05-13] save | 新建 Ideas/Agent Harness 存储层次类比.md — 探索中
## [2026-05-13] update | 追加 Tools/MCP = OS 外设体系延伸（含 MCP=USB、中断模型、prompt injection 根源分析）
## [2026-05-13] update | 追加内核/应用程序分层延伸（含特权环缺失分析、宏内核 vs 微内核对比）；更新 For future Claude 摘要
## [2026-05-13] update | 追加自进化安全设计延伸（体细胞/生殖细胞隔离原则、三层进化策略）；更新 For future Claude 摘要
## [2026-05-13] update | 追加 lithify 缺失模块分析（P0~P3 共 8 个模块）；开放问题扩展至 11 条
## [2026-05-13] project | 新建 Projects/lithify/：主架构文档 + 13 个子模块设计文档（共 14 个文件）
## [2026-05-13] update | 补充 modules/00-Data-Access-Model.md（VFS/存储类类比，DAI 统一接口，durability/scope/share 语义）；更新 Overview 映射表和模块状态
## [2026-05-13] save | 新建 Phase1-Spec.md：3 个核心假设、6 个必须实现组件、验收标准、~700 行 Python 估算
## [2026-05-13] update | Phase1-Spec.md 切换实现语言为 Rust，更新代码签名、Cargo 依赖、项目结构，估算调整为 ~1200 行
## [2026-05-13] update | Phase1-Spec.md 记录 tool 接入方式决策：Phase 1 用 Claude Tool Use 直接 API，Phase 2 再迁移 MCP
## [2026-05-13] save | 新建 Dev-Skills.md：开发框架（agentic-engineering-framework）、skill 三层继承体系、lithify-rust 待补充规则、各阶段 skill 使用策略
## [2026-05-13] save | 新建 Docs-Code-Protocol.md：文档与代码协同演进协议（vault=Why / 代码=What、四条同步规则 R1~R4、ADR 格式、信任优先级）
## [2026-05-13] rename | 项目全局重命名：Agent OS → lithify；文件夹 Projects/Agent OS/ → Projects/lithify/；所有文档内 agent-os → lithify
