<h1 align="center">MemHooks</h1>
<p align="center"><strong>Mnemonic devices for agents.</strong></p>
<p align="center">Filesystem-scoped memory recall · <em>Hook the right memories into the right context.</em> 🎣</p>

<p align="center">
  <img alt="Agent Skills" src="https://img.shields.io/badge/Agent%20Skills-compatible-7c4dff" />
  <img alt="Hermes" src="https://img.shields.io/badge/Hermes-compatible-00bcd4" />
  <img alt="Memory agnostic" src="https://img.shields.io/badge/memory-backend%20agnostic-2ea44f" />
  <img alt="Version" src="https://img.shields.io/badge/version-0.4.0-orange" />
  <img alt="License" src="https://img.shields.io/badge/license-MIT-blue" />
</p>

<p align="center">
  <img src="assets/memhook2.png" alt="How MemHooks works" width="100%" />
</p>

## The idea

> **You can't recall what you don't know you know.**

An agent can have the right memory stored perfectly and still fail to use it because retrieval begins with a cue. If the agent no longer remembers that an old decision, failure, workaround, constraint, or insight even exists, it may never formulate the search that would bring that memory back.

MemHooks stores the **cue to remember**, close to the code or folder where that cue matters.

> **Memory systems know how to remember. MemHooks tells the agent what to recall here.**

A MemHook contains retrieval routing: concrete recall questions, optional memory categories, connection emphasis, entities, **priority**, **role applicability**, tags, existing synthesized resources, and things that should **not** be recalled. Before substantive work, the agent walks from the workspace root to the active directory, merges the applicable hooks, adapts them to whatever memory system is available, performs bounded recall, and then gets on with the job.

```text
workspace/
├── MEMHOOKS.md
├── backend/
│   ├── MEMHOOKS.md
│   └── auth/
│       ├── MEMHOOKS.md
│       └── refresh.py
```

Working in `backend/auth/` means:

```text
root hook
   ↓ inherit
backend hook
   ↓ inherit
local auth hook
   ↓
resolve + apply role routing
   ↓
translate to current memory backend
   ↓
recall the highest-value context that matters
   ↓
do the work
```

## What's new in v0.4.0

v0.4.0 turns MemHooks from a convention plus runtime adapter into a convention with a **reference parser/resolver/validator**.

- **Weighted retrieval:** structured recall queries may carry `priority: 0.0..1.0`.
- **Entity salience:** structured entities may carry `salience: 0.0..1.0`.
- **Role-based routing:** queries may declare `when.roles` so reviewers, architects, refactoring agents, feature agents, etc. can receive different recall plans without forking the directory tree.
- **Rust reference implementation:** reusable library plus the `memhooks` CLI.
- **Validation:** human, JSON, and SARIF diagnostics suitable for pre-commit hooks and CI.
- **Explainability:** `memhooks explain` shows exactly which hooks were inherited and which queries are effective for a directory/role.
- **Backward compatible:** the schema remains `memhooks/v1`; old string queries/entities continue to work unchanged.

## Weighted retrieval and role routing

A security boundary and a naming convention are both useful memories, but they are not equally expensive to lose when the context window is full. v0.4.0 lets the author express that without confusing retrieval importance with backend relevance scores.

```yaml
recall_queries:
  - query: "What security boundaries must never be violated in this module?"
    priority: 1.0
    when:
      roles: [reviewer, architect]

  - query: "What naming conventions are preferred here?"
    priority: 0.35
```

`priority` belongs to the **recall request**. It says how important that request is when a context manager must trim competing material. It is not semantic similarity, confidence, truth probability, or an instruction to multiply a backend's score by a particular formula.

If `priority` is omitted, MemHooks assigns **no mandatory numeric default**. Existing queries retain ordinary/unweighted behavior.

Entities have their own concept:

```yaml
entities:
  - name: Authentication
    type: CONCEPT
    salience: 0.95
```

`salience` is the importance of the **entity as a retrieval cue**. It is deliberately separate from query priority.

Role names are open strings defined by the project/runtime:

```yaml
when:
  roles: [reviewer, refactor]
```

If an adapter knows the active roles, a restricted query applies when **any** listed role matches. A query without `when.roles` applies to all roles. A generic resolver with no active-role information can preserve all queries and defer filtering to the role-aware consumer.

## One-command project bootstrap

MemHooks is deliberately **opt-in per project**. From inside the project in Hermes:

```text
/memhooks init
```

Low-level equivalent:

```bash
python3 scripts/memhooks_update.py init /path/to/project
```

After that, the `post_tool_call` maintainer can create/update more local `MEMHOOKS.md` files automatically as work touches subdirectories.

## The Rust reference CLI

Build the reference tool from the repository:

```bash
cargo build --release
```

Then:

```bash
# Validate hooks below the current path
./target/release/memhooks validate

# Validate the entire repository
./target/release/memhooks validate --all

# CI/tooling output
./target/release/memhooks validate --all --format json
./target/release/memhooks validate --all --format sarif

# Show the inherited/effective context for a directory
./target/release/memhooks explain backend/auth

# Show the effective query set for one or more active roles
./target/release/memhooks explain backend/auth --role reviewer
./target/release/memhooks explain backend/auth --role reviewer --role architect --format json
```

The Rust crate is intentionally split into a reusable library and a thin CLI. Agent runtimes can depend on the parser/resolver instead of independently reimplementing inheritance semantics.

The reference tool **does not execute shell commands and does not contact a memory backend**. It parses, resolves, filters, explains, and validates the MemHooks routing plan.

## Why it exists

Long-term memory does not automatically imply good recall. Semantic similarity alone can miss old architectural decisions, rejected approaches, weird platform gotchas, or the reason some apparently ridiculous line of code exists.

A folder is already a strong contextual cue. MemHooks makes that cue explicit.

- **Filesystem-scoped** — context follows the part of the project being touched.
- **Inherited** — broad project knowledge at the root, increasingly specific hooks deeper down.
- **Agent-agnostic** — the specification describes behavior, not one specific harness.
- **Memory-agnostic** — Hindsight, OpenViking and Honcho mappings are included; other systems can adapt by capability.
- **Retrieval-only** — MemHooks does not create, rewrite, consolidate or delete memories merely because a hook was loaded.
- **Typed when known** — memory category, connection emphasis and entity type can be preserved without guessing.
- **Budget-aware** — priority and salience let a consumer trim lower-value context first.
- **Role-aware** — one directory can expose different recall plans to different agent roles.
- **Anti-recall too** — `exclude` keeps obsolete but tempting memories out of working context.
- **Cheap by design** — small files, bounded retrieval, no database or daemon of its own.

## A typed, weighted, routed hook

```md
---
schema: memhooks/v1
inherits: true

memory_types:
  - world
  - experience
  - observation

connection_types:
  - semantic
  - temporal
  - entity
  - causal

recall_queries:
  - query: "Why did the authentication design change after the outage?"
    priority: 1.0
    when:
      roles: [reviewer, architect]
    memory_types:
      - experience
    connection_types:
      - causal
      - temporal
    entities:
      - Authentication
      - name: OpenAI
        type: ORG
        salience: 0.9

entities:
  - name: Authentication
    type: CONCEPT
    salience: 0.95

exclude:
  - obsolete OAuth prototype
---

Use direct recall first. Use deeper memory reasoning only if the retrieved
facts disagree or the rationale is still unclear.
```

The file contains **no memory itself**. It tells the agent which memories are worth retrieving and, where known, how to route and prioritize retrieval more precisely.

Legacy string-only queries and entities remain valid.

## Hindsight: keep the dimensions separate

MemHooks keeps Hindsight's public ontology separate rather than inventing one overloaded `type` field.

### Memory categories

Hindsight Recall accepts:

```text
world | experience | observation
```

Each selected memory type runs Hindsight's full four-strategy retrieval pipeline independently. A memory type is therefore not the same thing as a graph/retrieval connection.

### Connection emphasis

Hindsight organizes knowledge using:

```text
semantic | temporal | entity | causal
```

MemHooks preserves those as `connection_types` routing hints. They do not become a fake Hindsight API filter: adapters use native controls only when those controls genuinely exist and otherwise sharpen the natural-language query.

### Entities

MemHooks uses the backend-neutral shape:

```yaml
- name: OpenAI
  type: ORG
  salience: 0.9
```

The type and salience are optional. MemHooks does **not** invent a closed universal entity taxonomy. If the type is unknown, leave it untyped.

### Observations and mental models

Observations are consolidated, evidence-backed beliefs built from raw facts. Mental models sit above them as deliberately curated standing answers. Hindsight Reflect's retrieval ladder is:

```text
mental models
    ↓
observations
    ↓
raw facts
```

MemHooks therefore keeps `mental_models` / `knowledge_pages` separate from raw-memory `memory_types`.

See [`references/memory-systems/01-hindsight.md`](references/memory-systems/01-hindsight.md) for the mapping.

## Who fills `MEMHOOKS.md`?

A hook that never changes eventually becomes stale, so MemHooks includes a **zero-LLM maintainer**.

There are two maintenance paths:

1. **Deterministic auto-anchors — zero model tokens.** `scripts/memhooks_update.py event` inspects touched file paths and creates/refreshes a small machine-managed recall block in the relevant directory.
2. **Same-turn semantic cues — no extra model call.** If the current agent has already discovered a non-obvious decision, failure mode, constraint, or rejected approach, it can record a future retrieval question while it is already reasoning.

The deterministic path hook deliberately stays semantically conservative: a script can know which file changed, but should not guess memory type, causal structure, entity type, priority, salience, or role applicability.

Simple semantic note:

```bash
python3 scripts/memhooks_update.py note \
  --cwd "$PWD" \
  --query "Why was refresh-token rotation split into two stages?"
```

Routed note when the metadata is genuinely known:

```bash
python3 scripts/memhooks_update.py note \
  --cwd "$PWD" \
  --query "Why did the authentication design change after the outage?" \
  --priority 1.0 \
  --role reviewer \
  --role architect \
  --memory-type experience \
  --connection-type causal \
  --connection-type temporal \
  --entity 'Authentication' \
  --entity '{"name":"OpenAI","type":"ORG","salience":0.9}'
```

The note helper stores structured cues as bounded JSON inside `MEMHOOKS.md`; old markdown-bullet notes remain readable and migrate on the next write.

> **The memory backend stores what happened. MemHooks stores the cue that tells a future agent there is something worth recalling.**

## How the skill adapts

| Memory system | Typical mapping |
|---|---|
| **Hindsight** | use `types` for `world/experience/observation`; recall for concrete history; reflect when synthesis is needed; use connection/entity/mental-model hints where the exposed API supports them |
| **OpenViking** | search the `viking://` context hierarchy, then progressively read only the needed detail |
| **Honcho** | semantic search/context for concrete memory; dialectic reasoning only when synthesis is needed |
| **Anything else** | inspect available memory tools, use bundled mappings as examples, and infer the closest native operations without inventing unsupported filters |

> **Do not require a bespoke MemHooks plugin for every memory system. A capable agent should adapt the retrieval intent to the tools it actually has.**

## Runtime hook support

The convention, maintainer, `SKILL.md`, and Rust reference library are agent-agnostic. Runtime plumbing is necessarily agent-specific because harnesses expose lifecycle hooks differently.

Today:

- **Hermes / Hermes Desktop:** working `pre_llm_call` loading and `post_tool_call` maintenance under [`hooks/hermes/`](hooks/hermes/).
- **Other agents:** can use the spec/reference resolver immediately, but need a thin adapter to wire native lifecycle events and active roles.

A runtime adapter needs roughly:

```text
before model call -> resolve MEMHOOKS.md -> apply active roles -> retrieve bounded context
after tool call   -> pass cwd + tool input to memhooks_update.py event
```

## Installation — Hermes / Hermes Desktop

Clone or copy this repository into the active Hermes skills directory:

```bash
git clone https://github.com/AronAxe/MemHooks.git ~/.hermes/skills/memhooks
```

Install the lifecycle scripts:

```bash
mkdir -p ~/.hermes/agent-hooks
cp ~/.hermes/skills/memhooks/hooks/hermes/memhooks_pre_llm.py ~/.hermes/agent-hooks/
cp ~/.hermes/skills/memhooks/scripts/memhooks_update.py ~/.hermes/agent-hooks/
chmod +x ~/.hermes/agent-hooks/memhooks_pre_llm.py ~/.hermes/agent-hooks/memhooks_update.py
```

Add to `~/.hermes/config.yaml`:

```yaml
hooks:
  pre_llm_call:
    - command: "python3 ~/.hermes/agent-hooks/memhooks_pre_llm.py"
      timeout: 5
  post_tool_call:
    - command: "python3 ~/.hermes/agent-hooks/memhooks_update.py event"
      timeout: 5
```

Then enable MemHooks once in a project:

```text
/memhooks init
```

See [`hooks/hermes/README.md`](hooks/hermes/README.md) for details.

## File format

| Field | Purpose |
|---|---|
| `inherits` | inherit parent-directory hooks (`true` by default) |
| `memory_types` | optional memory categories, e.g. Hindsight `world/experience/observation` |
| `connection_types` | optional semantic/temporal/entity/causal retrieval emphasis |
| `recall_queries` | concrete questions worth asking memory; strings or structured records |
| `recall_queries[].priority` | optional `0.0..1.0` importance under a finite context budget |
| `recall_queries[].when.roles` | optional role-based applicability; open/project-defined role strings |
| `entities` | named entities; optionally `{name, type, salience}` when metadata is known |
| `mental_models` | existing standing answers worth reading first when supported |
| `knowledge_pages` | existing stable synthesized pages/resources |
| `tags` | backend-neutral relevance/scoping hints |
| `exclude` | obsolete or misleading context that should not enter the current workspace |
| `bank` | optional namespace/bank/peer/session hint |
| `sensitivity` | advisory handling metadata |

Read the full contract in [`references/memhooks-format.md`](references/memhooks-format.md).

## Root-to-leaf behavior

Given:

```text
/repo/MEMHOOKS.md
/repo/backend/MEMHOOKS.md
/repo/backend/auth/MEMHOOKS.md
```

an agent working in `/repo/backend/auth/` reads all three **in that order**.

- Lists accumulate and exact duplicates are removed.
- More local scalar values win.
- `inherits: false` cuts off the parent chain.
- Query-local memory/connection types override scope defaults for that query.
- Query-local entities supplement inherited entities.
- Priority/role conditions stay attached to the structured query that declared them.
- Entity salience stays attached to that entity.
- Role filtering happens after inheritance resolution when active-role information exists.
- Retrieval remains bounded; the merged result is a routing plan, not an excuse to dump the entire memory store into context.

## What MemHooks is *not*

MemHooks is **not** a vector database, memory provider, automatic memory-writing system, entity-relationship schema, directive store, knowledge-page generator, GraphRAG framework, shell-command manifest, or excuse to shove more tokens into every prompt.

It is deliberately boring infrastructure:

> **When an agent works here, remember these things first.**

## Related: Token Terminator

If MemHooks is about **retrieving the right context**, [**Token Terminator**](https://github.com/AronAxe/Token-Terminator) is about **not wasting tokens on the wrong context**.

They are separate projects, but share the same prejudice: an AI agent should not need to carry its entire history around as a giant linear transcript just to remember what matters.

## Status

**v0.4.0 — experimental convention + reference resolver/linter + deterministic load-and-maintain runtime.**

The format remains intentionally small and backward-compatible. Issues, backend mappings and real-world examples are welcome.

<p align="center">
  <img src="assets/memhooklogo.png" alt="MemHooks logo" width="300" />
</p>

## License

MIT © 2026 Aron Bijl
