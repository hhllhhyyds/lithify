# Claude Operating Manual — hhl's Vault

> Read this file before doing anything in this vault.
> This is the single source of truth for how Claude operates here.

---

## Section 0 — AI-First Vault Rule (read first, applies to every note)

This vault is designed for **future-Claude** to read and reason over, not for human review. The owner rarely reads notes directly — they call Claude to retrieve, synthesize, and connect dots across years of accumulated knowledge.

**Every note Claude writes to this vault must follow these rules:**

1. **Self-contained context** — Each note must explain itself. Future-Claude may pull this single note via search with no surrounding context. Don't rely on backlinks alone for meaning.
2. **"For future Claude" preamble** — Every note begins with a 2-3 sentence summary in plain English so Claude can decide relevance in 10 seconds before parsing the structured data.
3. **Rich, consistent frontmatter** — Filterable metadata (`type`, `date`, `tags`, `ai-first: true`, plus type-specific fields). Different note types may have different schemas, but every note has machine-readable frontmatter.
4. **Recency markers per claim** — When stating external facts, attach the date: "Mem0 raised $24M (as of 2026-04)" so future-Claude knows what to verify before trusting.
5. **Sources preserved verbatim** — Every external claim has its source URL inline so it can be re-verified or refreshed.
6. **Cross-links are mandatory** — Every person, project, idea, decision, or concept referenced uses `[[wikilinks]]` so the graph is traversable.
7. **Confidence levels** — Where applicable, mark claims as `stated | high | medium | speculation` so future-Claude knows what to trust vs verify.

This rule applies to all `/obsidian-*` and `/research*` commands, all scheduled agents, and any direct vault writes.

---

## Vault Identity

- **Owner:** hhl
- **Vault path:** `/Users/hhl/Documents/Ideas/spark`
- **Primary purpose:** 技术灵感记录 — 捕捉、整理、演化技术想法与创意
- **Language:** 中文为主，英文为辅
- **Last updated:** 2026-05-13

---

## Folder Map

| 文件夹 | 用途 |
|---|---|
| `Daily/` | 每日笔记，命名格式 `YYYY-MM-DD.md` |
| `Projects/` | 进行中和已归档的项目 |
| `Tasks/` | 独立任务笔记（与看板联动） |
| `Boards/` | 看板：工作、个人等 |
| `People/` | 每人一个笔记 |
| `Ideas/` | 想法捕捉与探索 |
| `Knowledge/` | 参考资料和永久笔记 |
| `Learning/` | 书籍、课程、内容消费记录 |
| `Dev Logs/` | 技术工作日志，带日期和项目标签 |
| `Templates/` | 笔记模板 |

> 注：vault 目前为空，上述文件夹在首次使用时创建即可。

---

## Key Files

- **首页/仪表板:** `[[Home]]` — 主导航和 dataview 查询（待创建）
- **工作看板:** `[[Boards/Work]]`（待创建）
- **个人看板:** `[[Boards/Personal]]`（待创建）
- **索引:** `[[index]]` — 所有笔记目录（由 Claude 维护）
- **日志:** `[[log]]` — 操作历史记录（由 Claude 维护）

---

## Active Context

> 在每个重要项目或专注阶段开始时更新此部分。

**当前首要优先级:** [待填写]
**当前工作/角色:** [待填写]
**关键协作者:** [待填写]

---

## Auto-Save Rules

Claude 应**无需询问**自动保存以下内容：
- 对话中做出的决策 → 相关项目笔记 + 当日笔记
- 提到的新人员 → `People/`（如不存在则创建存根）
- 分配或承诺的任务 → 看板 + `Tasks/` 笔记
- 开发工作 → `Dev Logs/` + 项目笔记 + 当日笔记
- 完成的任务 → 在看板移至 ✅ Done

Claude 应**先询问再保存**：
- 涉及财务或个人敏感数据的内容
- 任何删除或归档现有笔记的操作

---

## Naming Conventions

- 日常笔记: `YYYY-MM-DD.md`
- 开发日志: `YYYY-MM-DD — 描述.md`
- 任务: 描述性标题，无日期前缀
- 人员: 全名（如 `张三.md`，不用 `张.md`）
- 归档前缀: `_archived_`

---

## Frontmatter Requirements

每个笔记至少包含：
```yaml
---
date: YYYY-MM-DD
type: <note-type>
tags:
  - <note-type>
ai-first: true
---
```

Note types: `daily` | `project` | `task` | `person` | `devlog` | `idea` | `decision` | `knowledge` | `review` | `research`

---

## Kanban Convention

看板列: `📥 Backlog` · `📋 This Week` · `🔨 In Progress` · `⏳ Waiting On` · `✅ Done`

优先级: 🔴 紧急 · 🟡 重要 · 🟢 低优

条目格式:
```
- [ ] 🔴 **标题** · @{YYYY-MM-DD}
    描述。[[相关项目]] [[相关人员]]
```

完成格式:
```
- [x] ~~🔴 **标题**~~ ✅ YYYY-MM-DD
```

---

## Propagation Rules

| 事件 | 同步更新 |
|---|---|
| 新项目 | 看板 (Backlog) + 当日笔记 |
| 任务完成 | 看板 (Done，加删除线) + 项目笔记 + 当日笔记 |
| 开发会话 | `Dev Logs/` + 项目笔记 (Recent Activity) + 当日笔记 |
| 人员互动 | 当日笔记 + 对应 `People/` 笔记 |
| 做出决策 | 项目笔记 (Key Decisions) + 当日笔记 |

---

## Do Not Touch

- `Templates/` — 正常操作时不修改模板
- `_CLAUDE.md` — 仅在用户明确要求时更新

---

## Communication

- **语言:** 始终用中文回复用户，英文仅作备选
- **不使用:** 韩文、日文

---

*此文件由 obsidian-second-brain skill 于 2026-05-13 生成。*
*重新生成: "Claude，更新我的 _CLAUDE.md"*
