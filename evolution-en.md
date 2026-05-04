# Memory Sedimentation: A Self-Evolving Four-Layer Memory Architecture for AI Agents

Drawing inspiration from human memory and behavioral systems, I propose a four-layer memory architecture for Harness Agents. This document first lays out each layer, then examines how the architecture enables agent self-evolution.

Large language models have made extraordinary advances in context windows, scaling from a few thousand tokens to million-token contexts today. But context cannot grow indefinitely—not just because of API cost and inference latency, but because of fundamental constraints: attention dilution in long contexts, retrieval precision degradation, and the difficulty of finding the right information in a vast context ("needle in a haystack"). A million tokens is already enough to cover the instantaneous information needs of most tasks. Pushing context length further yields sharply diminishing returns. **The next competitive frontier is not larger context, but smarter memory systems**—how to determine what information is worth keeping, in what form to keep it, and when to inject it into limited context. This is precisely the problem the four-layer memory architecture aims to solve.


### The Four-Layer Memory Architecture

An AI Agent should possess four layers of memory, ordered from near to far, from precise to fuzzy, from flexible to crystallized:

1. **Working Memory** (immediate memory)
2. **Short-term Precise Memory**
3. **Long-term Fuzzy Memory**
4. **Ultra-long-term Muscle Memory**

**Working Memory** is the smallest and most expensive of the four layers, but it is the only one that directly interacts with the brain (the LLM)—in an Agent, this is the current conversation's **context window**. Working memory is never persisted; it clears when the conversation ends. Precisely because it is so scarce, every byte must justify its presence. The purpose of all persistent memory, at bottom, is to **load the right information into working memory at the right moment, at the lowest possible cost**.

**Short-term Precise Memory** is the next smallest, but the most exact and the most mutable. It stores complete instructions for "how to do something"—precise, executable, and therefore consuming significant context tokens on every invocation. It is suited for knowledge that is still undergoing rapid iteration and has not yet stabilized.

**Long-term Fuzzy Memory** offers vastly greater capacity at the cost of gradually fading detail. It no longer provides verbatim instructions, but rather compressed experience fragments and general principles. Only the few retrieved passages are injected into context, occupying far less than a full Skill. Its value lies not in exact reproduction, but in **generalized matching**—trading precision for adaptability across diverse scenarios.

**Ultra-long-term Muscle Memory** is fully crystallized—invoking it consumes zero working memory. What it stores is no longer "knowledge" or "instructions," but **capability**—a pre-compiled, repeatable program. The brain does not need to understand how it works internally; it only needs to know when to call it and what arguments to pass.

The relationship between the four layers is:

```
Working Memory (context window) ← Brain (LLM) thinks here
    ↑ loaded on demand
Short-term Precise Memory (Skill) ← Brain actively retrieves exact instructions
    ↑ demoted / archived
Long-term Fuzzy Memory (RAG)       ← retrieved, then injected into working memory
    ↑ crystallized
Ultra-long-term Muscle Memory (Tool) ← invoked without occupying working memory
```

This diagram reveals a clear gradient: from top to bottom, storage capacity increases, invocation cost decreases, mutability decreases, and crystallization increases. Agent evolution, in essence, is the process of information and capability **settling downward** along this gradient.

Mapped to mainstream Agent designs today (Claude Code, OpenClaw, etc.):

- **Working Memory** is the context window. This is where the LLM gathers the full text of Skills, RAG retrieval results, Tool outputs, and conversation history—all information ultimately converges here, and the LLM performs understanding and reasoning here. But it is extremely expensive and volatile, vanishing when the conversation ends.
- **Short-term Precise Memory** is the Skill system. Skills consist of precise textual instructions that may be modified across conversation iterations—highly flexible, but each Skill invocation consumes substantial context tokens. A complex Skill can run thousands of words; loading them all into working memory is unsustainable.
- **Long-term Fuzzy Memory** is the RAG database. Retrieval is fuzzy-matched, but capacity far exceeds that of Skill documents. It bears repeating: "fuzzy" here is not a defect—it means **generalization**. A single RAG entry can cover multiple related scenarios, whereas a Skill might fail to match if a single variable name changes. This is the core value that distinguishes long-term memory from short-term memory: **trading precision for breadth**.
- **Ultra-long-term Muscle Memory** is the Tool system (including external tools mounted via the MCP protocol). Tools have fixed behavior; their execution consumes no LLM tokens. The caller only needs to know the interface signature and function description. A well-developed Tool ecosystem is, in essence, the Agent's "muscle memory library"—the more frequently a function is used, the more it deserves to exist in Tool form.

And the LLM itself? It belongs to none of the four layers. A large model has no persistent memory—its weights are frozen after training and do not change because of a conversation. It is merely the **brain**—a pure information processor. The memory system is an external structure independent of the brain, just as human memory is not stored in neuronal cell bodies alone, but distributed across synaptic connections and external cognitive tools.


### Retrieval Routing: How Does the Brain Find the Right Memory?

Once the four memory layers are in place, every task forces the Agent to answer a meta-question first: **which memory layer should I look in?**

This is not a trivial detail. The more memory layers there are, the more retrieval paths exist. Without a routing mechanism, the retrieval overhead itself can consume the very benefits the memory system is meant to provide. Imagine: for every task, the Agent sequentially searches all Skills, scans the entire RAG store, and checks every Tool signature—these operations alone consume substantial context and reasoning time.

Three routing strategies are available:

- **Top-down**: Search Skills first (for exact matches), then fall back to RAG (fuzzy match), and finally rely on the LLM to reason from scratch if both miss. Working memory and Tools do not need "retrieval"—the former is the context currently being processed, the latter is invoked directly by function name. This strategy is the simplest, but suffers from ordering dependency: if the routing layer incorrectly concludes a Skill "matches," it will never proceed to RAG and may miss more generalized experience.
- **Parallel retrieval**: Query Skills and RAG simultaneously, merge the candidates, and let the LLM adjudicate which to adopt. This avoids ordering dependency but increases per-query computation and context usage. It suits scenarios where latency is less critical and recall is paramount.
- **Index layer**: Maintain a lightweight summary index of all Skills and Tools (one or two sentences each). Search the index first to locate the right resource, then retrieve from that layer. This is the most scalable approach—the index itself is a tiny routing table that can reside permanently in working memory. Its value grows more apparent as the number of Skills and Tools increases.

Retrieval routing is the critical infrastructure that determines whether the four-layer architecture actually works. Without it, memory is not an asset—it is a liability.


### Four-Layer Memory and Agent Self-Evolution

The core motivation for the four-layer memory architecture is to enable **agent self-evolution**. The direction of evolution: experience and capability settle downward layer by layer, with each descent reducing context cost by an order of magnitude.

This closely mirrors how humans master a skill: at first you rely on detailed tutorials (Skill), getting every step right; with practice, you only need to remember a few key points and common pitfalls (RAG); eventually you develop muscle memory, and your hands execute without conscious thought (Tool). The Agent's evolution follows the same pattern:

1. **Skill-ification**: The Agent executes tasks and accumulates experience, which initially settles in short-term precise memory as exact text (e.g., `skill.md`). This stage is characterized by **high fidelity, high cost**—every piece of experience is a complete natural-language instruction that occupies context on every load.
2. **RAG-ification**: As experience crystallizes and stops changing frequently, Skill documents are migrated to the RAG database, becoming long-term fuzzy memory. The migration is not a verbatim copy—it involves compression and distillation, stripping away one-off details and retaining reusable principles. Compared to loading a full Skill, RAG only injects the retrieved fragments, drastically reducing context usage. This stage is characterized by **medium fidelity, medium cost**.
3. **Tool-ification**: When a capability's invocation frequency is high enough, its pattern stable enough, and its input/output contract clear enough, the Agent can crystallize it into code as a Tool, becoming ultra-long-term muscle memory. From then on, invocation only requires knowing the function name and parameters; the internal implementation is entirely removed from context—the LLM no longer needs to "understand" how it works, only that it exists and what it takes. This stage is characterized by **low fidelity (externally), zero cost**.

These three steps form a complete evolution cycle. As the Agent is used continuously, its long-term memory and muscle memory thicken, while short-term memory capacity remains stable. The Agent becomes capable of more and more, but its cognitive burden does not grow proportionally—that is what makes this evolution.


### Refining the Evolution Process

#### Who Triggers Migration?

The previous section described the *effect* of evolution but not the *timing* or the *decision-maker*. Three possible mechanisms:

- **Agent autonomous judgment**: After each task, the Agent evaluates on its own whether "this experience is ready to be archived" and writes it to RAG; when a particular RAG entry is repeatedly matched, it proactively prompts the user about generating a Tool. This aligns most closely with the self-evolution vision, but the judgment bar is the highest—how does the Agent determine what is worth archiving? Is the archiving granularity an entire Skill or a single step within it? What happens if the judgment is wrong?
- **Harness-layer automation**: The framework monitors Skill invocation frequency and content stability. When a Skill has been executed N consecutive times without modification, it automatically triggers the archiving process. The rules are clear and predictable, but the Harness can only judge "stability"—not "memorability." A Skill that is stable but useless may still get archived.
- **Manual user trigger**: The user manually advances the process via commands like `/archive` or `/solidify`. This is the most controllable—every migration passes through human judgment—but it depends on human diligence and initiative, meaning the Agent cannot grow on its own.

The three mechanisms can coexist, but what is more interesting is the **self-evolution of the trigger mechanism itself**: early on, when the Agent's capability is limited, archiving should be manually triggered by the user, ensuring every migration passes human judgment; as the Agent accumulates more experience and its judgment sharpens, trigger authority can gradually be delegated to Harness automation, eventually transitioning to Agent autonomous judgment. The trigger mechanism itself settles downward through the same evolution pipeline.

#### Skill → RAG: The Compression Step in Between

Not all Skills are suitable for being dumped verbatim into RAG. Some Skills contain one-off parameters, specific environment variables, or project-specific paths. Direct archiving creates noise—the next time this content is retrieved, the Agent may be misled by irrelevant details.

During migration, the Agent must perform a **compression/summarization** step: distilling the experience of multiple similar executions into a single general principle. Good compression preserves three things: **key decision points** (which path to take at each fork), **common failure modes** (what can go wrong and how to avoid it), and **applicability conditions** (under what circumstances this experience is valid). It strips away specific one-off details (e.g., "the branch name used in the last deployment was `feature/xxx`").

For example, a deployment Skill might contain dozens of lines of specific steps. The compressed RAG entry might be just three sentences: "Deploying this service requires Node ≥ 20; build output is in `/dist`; environment variables are injected via the platform Dashboard, not `.env` files." Information density is greatly increased, and the entry applies to a wider range of situations.

#### RAG → Tool: Code Generation Is the Hardest Problem

Automatically generating a runnable Tool from a fuzzily-retrieved block of text means the Agent must cross an enormous chasm—from "knowing how it is done" to "turning that into code." Specifically, it requires the following capabilities:

- **Interface abstraction**: From natural-language descriptions in a RAG entry, accurately infer the function's input/output contract—parameter names, types, required vs. optional, defaults. A single interface design error makes the Tool uncallable.
- **Code generation**: Produce correct and maintainable code, not a throwaway script. Tool code must handle both happy paths and error paths; it cannot assume inputs are always valid.
- **Edge cases and error handling**: Think through "what can go wrong"—timeouts, permission denials, network interruptions—and handle them gracefully in code, returning structured error messages rather than bare crashes.
- **Sandbox verification**: Run the generated Tool in a controlled environment, validating its output against expectations with real or simulated inputs. Deploying an unverified Tool to production is equivalent to shipping untested code.
- **Versioning and compatibility**: Once a Tool is live, it may be called many times; its behavior must be stable and predictable. If the Tool's implementation is later updated, existing call patterns must not suddenly break.

This is, at its core, **Agent self-programming**—the hardest step to automate in the entire four-layer architecture. The near-term viable path is "semi-automatic": the Agent generates a Tool code draft and test cases; the user verifies them in a sandbox and confirms before production deployment.

But if this step can genuinely be achieved—if the Agent can not only use tools but **create tools for itself**—then in my view, that is true intelligence. Throughout the history of human civilization, from stone tools to the steam engine to the compiler, every qualitative leap came not from humans getting smarter, but from humans inventing new tools that extended the boundaries of their own capability. The Agent's transition from "using tools" to "inventing tools" is a leap of the same magnitude. An Agent that can build tools for itself is no longer a tool—it is a **toolmaker**.

#### Version Control in Evolution

After a Skill is archived to RAG, should the original Skill be deleted or kept? What if RAG retrieves outdated information? When an external API that a Tool depends on changes, how does the Tool detect and adapt?

These are not one-time problems but persistent tensions in long-running systems. Every layer's contents should carry timestamps and provenance traces:

- RAG entries record "originating from which Skill, archived when, at what version"
- Tools record "originating from which RAG entries, crystallized when, dependent on which external API versions"
- When underlying information is marked as stale, upstream crystallized artifacts can trace to their source and update or deprecate accordingly

Furthermore, this traceability chain enables **rollback**: if a Tool is found to be defective, one can follow the chain to locate which RAG entry contained the faulty information, then degrade to RAG mode or even Skill mode to continue working while the Tool is repaired. Memory evolution is not a one-way "write → forget" process—it is a traceable, rollback-able, repairable chain.

#### Example: The Full Evolution of Deploying to Cloudflare Pages

Here is a concrete scenario demonstrating the full three-stage evolution:

1. **Skill stage**: The user writes `deploy-cloudflare.md`, a detailed deployment guide—"first confirm the local Git state is clean, then confirm the current branch is `main`, run `npm run build`, build output is in `/dist`, configure environment variables via the Cloudflare Dashboard, finally trigger deployment..." Each deployment, the Agent loads this entire text and follows it step by step. The advantage at this stage is precision; the cost is that every deployment consumes the full context.

2. **RAG stage**: After roughly 10 deployments, the Agent recognizes that the core pattern is the same each time—only the project name and environment variable values differ. It compresses the stable experience for archiving: "Cloudflare Pages deployment essentials: Git branch must be `main`; build output directory is `/dist`; environment variables are set via Dashboard, not `.env` files; deployment trigger is Git push or Dashboard manual trigger." The original Skill is pared down to a one-line pointer: "For deployment, refer to Cloudflare deployment experience in RAG." Context consumption per deployment drops from hundreds of words to tens of words. The Agent can still deploy correctly because it retrieves the key points from RAG.

3. **Tool stage**: After another 50 or so deployments confirm the deployment pattern is stable and unchanging, the Agent generates an MCP Tool `deploy-to-cloudflare` with the following interface: accepts `--project-path` (project path), `--env-vars` (environment variable list), `--branch` (target branch, default `main`), internally encapsulating all deployment logic—Git state check, build, environment variable injection, deployment trigger, result polling. From then on, the Agent deploys with a single command: `deploy-to-cloudflare --project-path . --env-vars "KEY1=val1,KEY2=val2"`—**zero tokens spent understanding the deployment process**. The LLM does not even need to know how deployment works; it only needs to know "there is a tool that does this, and these are the parameters."

Whether this evolution path succeeds depends on three criteria: **execution count threshold** (how many repetitions count as "stable"—too few and Tool quality suffers, too many and evolution is too slow), **correctness verification** (how the generated Tool proves it is correct—requiring automated tests and human spot-checks), and **fallback mechanism** (if the Tool fails, can it degrade back to RAG or even Skill mode to continue working, rather than simply failing).


### Conclusion

The four-layer memory system—Working Memory, Short-term Precise Memory, Long-term Fuzzy Memory, and Ultra-long-term Muscle Memory—each layer bears a distinct memory form: from expensive precise instructions, to compressed generalized experience, to zero-cost crystallized capability. Together, from top to bottom, they form a gradual evolution pipeline along which the Agent continuously settles information downward, freeing up limited working memory space to support ever more complex capabilities.

When this memory system is combined with Agent self-programming—the Agent not only remembering experience, but crystallizing it into tools of its own creation—we approach the inflection point where the Agent transitions from "the used" to "the self-evolving." This should be the core direction for the next generation of AI Agent architectures.
