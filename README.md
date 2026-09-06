<h1 align="center">MemHooks</h1>
<p align="center"><strong>Mnemonic devices for agents.</strong></p>
<p align="center">Filesystem-scoped memory recall · <em>Hook the right memories into the right context.</em> 🎣</p>

<p align="center">
  <img alt="Agent Skills" src="https://img.shields.io/badge/Agent%20Skills-compatible-7c4dff" />
  <img alt="Hermes" src="https://img.shields.io/badge/Hermes-compatible-00bcd4" />
  <img alt="Memory agnostic" src="https://img.shields.io/badge/memory-backend%20agnostic-2ea44f" />
  <img alt="Version" src="https://img.shields.io/badge/version-0.3.0-orange" />
  <img alt="License" src="https://img.shields.io/badge/license-MIT-blue" />
</p>

<p align="center">
  <img src="assets/memhook2.png" alt="How MemHooks works" width="100%" />
</p>

## The idea

> **You can't recall what you don't know you know.**

An agent can have the right memory stored perfectly and still fail to use it, because retrieval begins with a cue. If the agent no longer remembers that an old decision, failure, workaround, constraint, or insight even exists, it may never formulate the search that would bring that memory back.

MemHooks stores the **cue to remember**, close to the code or folder where that cue matters.

> **Memory systems know how to remember. MemHooks tells the agent what to recall here.**

A MemHook contains retrieval routing: concrete recall questions, optional memory categories, connection emphasis, entities, tags, existing synthesized resources, and things that should **not** be recalled. Before substantive work, the agent walks from the workspace root to the active directory, merges the applicable hooks, adapts them to whatever memory system is available, performs bounded recall, and then gets on with the job.

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
translate to current memory backend
   ↓
recall the exact context that matters
   ↓
do the work
```

## One-command project bootstrap

MemHooks is deliberately **opt-in per project**. From inside the project:

```text
/memhooks init
```

Hermes exposes installed skills as slash commands, so the skill delegates to the deterministic initializer and creates the root `MEMHOOKS.md` if it does not already exist.

Low-level equivalent:

```bash
python3 scripts/memhooks_update.py init /path/to/project
```

After that, the `post_tool_call` maintainer can create/update more local `MEMHOOKS.md` files automatically as work touches subdirectories.

## Why it exists

Long-term memory does not automatically imply good recall. Semantic similarity alone can miss old architectural decisions, rejected approaches, weird platform gotchas, or the reason some apparently ridiculous line of code exists.

A folder is already a strong contextual cue. MemHooks makes that cue explicit.

- **Filesystem-scoped** — context follows the part of the project being touched.
- **Inherited** — broad project knowledge at the root, increasingly specific hooks deeper down.
- **Agent-agnostic** — the skill describes the behavior, not one specific harness.
- **Memory-agnostic** — Hindsight, OpenViking and Honcho mappings are included; other systems can be adapted by capability.
- **Retrieval-only** — MemHooks does not create, rewrite, consolidate or delete memories.
- **Typed when known** — memory category, connection emphasis and entity type can be preserved without guessing.
- **Anti-recall too** — `exclude` keeps obsolete but tempting memories out of working context.
- **Cheap by design** — small files, bounded retrieval, no database or daemon of its own.

## A typed hook

```md
---
schema: memhooks/v1
inherits: true

# Optional scope defaults.
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
    memory_types:
      - experience
    connection_types:
      - causal
      - temporal
    entities:
      - authentication
      - name: OpenAI
        type: ORG

entities:
  - authentication

exclude:
  - obsolete OAuth prototype
---

Use direct recall first. Use deeper memory reasoning only if the retrieved
facts disagree or the rationale is still unclear.
```

The file contains **no memory itself**. It tells the agent which memories are worth retrieving and, where known, how to route the retrieval more precisely.

Legacy string-only queries and entities remain valid.

## Hindsight: keep the dimensions separate

MemHooks v0.3 was tightened against Hindsight's public documentation rather than inventing its own ontology.

### Memory categories

Hindsight Recall accepts:

```text
world | experience | observation
```

The crucial detail is that **each selected memory type runs Hindsight's full four-strategy retrieval pipeline independently**. A memory type is therefore not the same thing as a graph/retrieval connection.

### Connection emphasis

Hindsight organizes knowledge using:

```text
semantic | temporal | entity | causal
```

MemHooks can preserve those as `connection_types` routing hints. They do not become a fake Hindsight API filter: the adapter uses native controls only when they genuinely exist and otherwise sharpens the natural-language query.

### Entities

Hindsight explicitly supports entities shaped like `{text, type?}`. Its docs give `PERSON`, `ORG`, and `CONCEPT` as examples and describe automatic recognition of people, organizations, places, products and concepts.

MemHooks uses the backend-neutral shape:

```yaml
- name: OpenAI
  type: ORG
```

The type is optional. MemHooks does **not** invent a closed universal taxonomy. If the type is unknown, leave the entity untyped.

A decision or constraint is normally information **about** entities, not an entity merely because we want to classify it.

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

See [`references/memory-systems/01-hindsight.md`](references/memory-systems/01-hindsight.md) for the exact mapping.

## Who fills `MEMHOOKS.md`?

A hook that never changes would eventually become useless, so MemHooks includes a **zero-LLM maintainer**.

There are two maintenance paths:

1. **Deterministic auto-anchors — zero model tokens.** `scripts/memhooks_update.py event` inspects touched file paths and creates/refreshes a small machine-managed recall block in the relevant directory.
2. **Same-turn semantic cues — no extra model call.** If the current agent has already discovered a non-obvious decision, failure mode, constraint, or rejected approach, it can record a future retrieval question while it is already reasoning.

The deterministic path hook deliberately stays **untyped**: a script can know which file changed, but not reliably whether the right memory category is `world`, `experience`, or `observation`, nor whether causal or temporal structure matters, nor what an entity's type should be.

Simple semantic note:

```bash
python3 memhooks_update.py note \
  --cwd "$PWD" \
  --query "Why was refresh-token rotation split into two stages?"
```

Typed note when the metadata is genuinely known:

```bash
python3 memhooks_update.py note \
  --cwd "$PWD" \
  --query "Why did the authentication design change after the outage?" \
  --memory-type experience \
  --connection-type causal \
  --connection-type temporal \
  --entity 'Authentication' \
  --entity '{"name":"OpenAI","type":"ORG"}'
```

The note helper stores structured cues as bounded JSON inside `MEMHOOKS.md`; old markdown-bullet notes are still read and migrate on the next write.

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

The convention, maintainer, and `SKILL.md` are agent-agnostic. Runtime plumbing is necessarily agent-specific because harnesses expose lifecycle hooks differently.

Today:

- **Hermes / Hermes Desktop:** working `pre_llm_call` loading and `post_tool_call` maintenance under [`hooks/hermes/`](hooks/hermes/).
- **Other agents:** can use the skill/spec and generic maintainer immediately, but need a thin adapter to wire native lifecycle events.

A runtime adapter needs only:

```text
before model call -> load + merge MEMHOOKS.md and inject routing
after tool call   -> pass cwd + tool input to memhooks_update.py event
```

## Installation — Hermes / Hermes Desktop

Clone or copy this repository into the active Hermes skills directory:

```bash
git clone https://github.com/AronAxe/MEMhooks.git ~/.hermes/skills/memhooks
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
| `recall_queries` | concrete questions worth asking memory; may be strings or structured records |
| `entities` | named entities; optionally `{name, type}` when the type is known |
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

- Lists accumulate and deduplicate.
- More local scalar values win.
- `inherits: false` cuts off the parent chain.
- Query-local memory/connection types override scope defaults for that query.
- Query-local entities supplement inherited entities.
- Retrieval remains bounded; the merged result is a routing plan, not an excuse to dump the entire memory store into context.

## What MemHooks is *not*

MemHooks is **not** a vector database, memory provider, automatic memory-writing system, entity-relationship schema, directive store, knowledge-page generator, GraphRAG framework, or excuse to shove more tokens into every prompt.

It is deliberately boring infrastructure:

> **When an agent works here, remember these things first.**

## Related: Token Terminator

If MemHooks is about **retrieving the right context**, [**Token Terminator**](https://github.com/AronAxe/Token-Terminator) is about **not wasting tokens on the wrong context**.

They are separate projects, but share the same prejudice: an AI agent should not need to carry its entire history around as a giant linear transcript just to remember what matters.

## Status

**v0.3.0 — experimental convention / agent skill + deterministic load-and-maintain runtime.**

The format remains intentionally small and backward-compatible. Issues, backend mappings and real-world examples are welcome.

<p align="center">
  <img src="assets/memhooklogo.png" alt="MemHooks logo" width="300" />
</p>

## License

MIT © 2026 Aron Bijl
