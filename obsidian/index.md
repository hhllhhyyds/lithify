# Vault Index

> Claude 进入 vault 时优先读此文件，比全局搜索更快更省。
> 此文件由 Claude 维护，每次新增/删除笔记时自动更新。
> 最后更新: 2026-05-13

---

## 系统文件

- [[_CLAUDE.md]] — Claude 操作手册，vault 规则与约定
- [[index]] — 本文件，所有笔记目录
- [[log]] — 操作历史记录

---

## Projects/

- [[Projects/lithify/Overview]] — lithify 主架构文档，LLM agent 与操作系统的完整同构设计
  - [[Projects/lithify/Phase1-Spec]] — Phase 1 最小原型规格：验证 H1/H2/H3 三个核心假设，~1200 行 Rust，1~2 周
  - [[Projects/lithify/Dev-Skills]] — 开发技能组：agentic-engineering-framework 使用策略、std-rust/lithify-rust 规范体系、各阶段 skill 使用方法
  - [[Projects/lithify/Docs-Code-Protocol]] — 文档与代码协同演进协议：职责划分（vault=Why/代码=What）、四条同步规则、ADR 格式、信任优先级
  - [[Projects/lithify/modules/00-Data-Access-Model]] — 数据访问模型（VFS 等价），skill 读写数据的统一接口、durability 语义、层次路由
  - [[Projects/lithify/modules/01-Memory-Hierarchy]] — Context window/Skills/RAG 存储层次设计
  - [[Projects/lithify/modules/02-Peripheral-MCP]] — Tools/MCP 外设体系，MCP=USB，中断驱动模型
  - [[Projects/lithify/modules/03-Kernel-Privilege]] — Harness 内核，特权环分级，微内核架构
  - [[Projects/lithify/modules/04-Process-Scheduling]] — Agent 生命周期，token budget 调度，NUMA 协调
  - [[Projects/lithify/modules/05-Self-Evolution]] — 三层自进化架构，体细胞/生殖细胞隔离原则
  - [[Projects/lithify/modules/06-IPC]] — Agent 间通信原语：Result Stream、Task Queue、Event Bus
  - [[Projects/lithify/modules/07-Signal-System]] — Agent 中止协议，SIGTERM/SIGKILL 等价物，检查点
  - [[Projects/lithify/modules/08-Observability]] — /proc 等价物，tool call tracer，分布式追踪
  - [[Projects/lithify/modules/09-Security-Model]] — Agent 身份，capability 权限，信任链，审计日志
  - [[Projects/lithify/modules/10-Virtual-Context]] — 虚拟 context 视图，CoW 隔离，context 段管理
  - [[Projects/lithify/modules/11-Stable-ABI]] — Skill ↔ Harness 稳定接口契约，版本管理，compat 层
  - [[Projects/lithify/modules/12-Init-Sequence]] — 五阶段 session 启动序列，崩溃恢复，session profile
  - [[Projects/lithify/modules/13-Package-Manager]] — Skill 包管理，依赖解析，签名验证，Registry

---

## People/

_暂无人员笔记_

---

## Ideas/

- [[Ideas/Agent Harness 存储层次类比]] — LLM context/skills/RAG 与 CPU 寄存器/缓存/磁盘的深度类比，含 context pressure、eviction 策略、多 agent NUMA 等推论

---

## Daily/

_暂无日常笔记_

---

## Knowledge/

_暂无知识笔记_

---

## Dev Logs/

_暂无开发日志_

---

## Tasks/

_暂无任务笔记_

---

*格式: `- [[笔记名]] — 一行描述`*
