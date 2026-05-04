# Lithify

**An AI agent that evolves. Experience settles into skill, skill compresses into memory, memory crystallizes into tools.**

Lithify is a self-evolving CLI agent built on a four-layer memory architecture. It grows more capable the more you use it—without burning more context.

## The Idea

LLMs now have million-token context windows. That's enough. The next frontier isn't bigger context—it's smarter memory.

Lithify implements **Memory Sedimentation**: a four-layer memory system where information continuously settles downward, from expensive precise instructions to zero-cost crystallized tools. Each layer transition cuts context consumption by an order of magnitude.

```
Working Memory (context window)  ← LLM thinks here
    ↑ loaded on demand
Short-term Precise Memory (Skill) ← exact instructions
    ↑ archived
Long-term Fuzzy Memory (RAG)      ← compressed experience
    ↑ crystallized
Ultra-long-term Muscle Memory (Tool) ← zero-context invocation
```

Read the full architecture: [`evolution-en.md`](./evolution-en.md) | [`evolution-zh.md`](./evolution-zh.md)

Development workflow: [`CLAUDE.md`](./CLAUDE.md) | [`docs/AI_WORKFLOW.md`](docs/AI_WORKFLOW.md)
Tech stack details: [`docs/TECH_STACK.md`](docs/TECH_STACK.md)

## MVP Scope

A minimal CLI agent you can use as a daily coding assistant, with all four memory layers wired end-to-end:

| Layer | Implementation |
|-------|---------------|
| **Working Memory** | Conversation context window |
| **Short-term Memory (Skill)** | Local markdown files, loaded on demand |
| **Long-term Memory (RAG)** | Auto-vectorized experience store; Skills auto-archive when stable |
| **Muscle Memory (Tool)** | Agent generates + registers new Tools (semi-auto, user-confirmed) |
| **Retrieval Routing** | Lightweight index layer for deciding where to look |

## Development Plan

### Phase 1: Skeleton

A working CLI agent loop with basic tool calling.

- [ ] CLI shell with conversation loop
- [ ] LLM provider integration (Anthropic API to start)
- [ ] Basic tool system (file read/write, shell execution)
- [ ] Skill loader: read markdown files from a `skills/` directory, inject into system prompt
- [ ] Skill self-modification: agent can update skill files through conversation (append new techniques, fix outdated steps, refine instructions)

### Phase 2: Skill → RAG

Skills that stabilize get compressed and archived automatically.

- [ ] Vector store for RAG entries (SQLite + embeddings, or something simple)
- [ ] Skill stability tracker: monitor invocation count and modification frequency
- [ ] Compression pipeline: when a skill is deemed stable, summarize it into a RAG entry
- [ ] Retrieval routing: before each task, check Skills first, then RAG

### Phase 3: RAG → Tool

Frequently-matched RAG entries crystallize into executable tools.

- [ ] Tool code generation: prompt the agent to write a tool implementation from RAG entries
- [ ] Sandbox verification: run generated tools in a controlled environment, validate correctness
- [ ] Tool registry: register generated tools, expose them to the agent
- [ ] User confirmation gate: semi-automatic, require human approval before tool activation

### Phase 4: Self-Evolution Loop

The full pipeline runs continuously.

- [ ] Trigger mechanism: manual (`/archive`, `/solidify`) with progressive automation
- [ ] Version tracking: provenance chain from Tool → RAG → Skill
- [ ] Rollback: degrade from Tool to RAG to Skill when something breaks
- [ ] Polish, performance, edge cases

## Principles

- **Dogfood first.** Lithify should be useful for developing Lithify. If it can't help build itself, it's not ready.
- **Semi-automatic before fully automatic.** Tool generation starts with human review. Automation grows as the agent proves itself.
- **Simple stack.** No distributed systems, no cloud dependencies. SQLite, local files, embeddings on disk. Keep it runnable on a laptop.
- **Every component maps to the architecture.** If a feature doesn't fit one of the four layers, question whether it belongs.

## Tech Stack

- **Runtime:** Rust
- **LLM:** Anthropic API (Claude), with provider abstraction for others
- **Vector store:** SQLite + local embeddings (`pgvector` / `sqlite-vec`)
- **Tool sandbox:** Isolated subprocess

## Inspired by

The Memory Sedimentation architecture is original work. If you build something based on it—whether a fork, a derivative project, a research paper, or a commercial product—please credit this project and link back to the [architecture document](./evolution-en.md).

This isn't a legal requirement (see [LICENSE](./LICENSE)), but a request for scholarly and professional courtesy. Ideas have provenance. Respect it.

## Why "Lithify"

*Lithification* is the geological process where loose sediment, under pressure and time, compacts into solid rock. That's exactly what this agent does: loose experiences settle, compress, and crystallize into durable capability.
