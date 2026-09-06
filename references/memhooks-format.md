# `MEMHOOKS.md` format — memhooks/v1

A `MEMHOOKS.md` file is Markdown with YAML frontmatter. The frontmatter is retrieval-routing metadata. The body is optional human/agent-readable retrieval guidance.

`memhooks/v1` is intentionally backward-compatible: existing string-only `recall_queries` and `entities` remain valid, while richer structured entries can add memory/fact types and entity types.

## Canonical shape

```md
---
schema: memhooks/v1
bank: optional-memory-namespace
scope: optional/human-readable/subsystem
inherits: true

knowledge_pages:
  - "Architecture/Authentication"

# Optional scope-wide memory/fact-type hints. Empty means no type filter.
memory_types:
  - world
  - experience
  - observation

recall_queries:
  # Legacy/simple form remains valid.
  - "What previous failures or rejected fixes involved token refresh?"

  # Structured form attaches precise routing metadata to one query.
  - query: "Why was the current authentication architecture selected?"
    memory_types:
      - experience
    entities:
      - name: Authentication
        type: COMPONENT
      - name: Refresh Token
        type: CONCEPT

entities:
  # Legacy/simple form remains valid.
  - OAuth

  # Preferred typed form when the entity type is known.
  - name: Authentication
    type: COMPONENT
  - name: Auth Service
    type: SERVICE

tags:
  - project:example
  - subsystem:auth

exclude:
  - obsolete OAuth prototype
  - deprecated session-store experiment

sensitivity: private
---

# Retrieval guidance

Use direct recall for concrete decisions and incidents.
Use deeper synthesis only when the retrieved memories conflict or the rationale remains unclear.
```

## Fields

### `schema`
Required. For this version use `schema: memhooks/v1`.

### `inherits`
Optional; default `true`. With `false`, this file starts a new hook scope and ancestors above it are ignored for the current subtree.

### `scope`
Optional descriptive path/subsystem label. It is a hint, not authoritative filesystem state.

### `bank`
Optional memory namespace/bank/peer/session hint. Use it only when the current backend has an equivalent and access is appropriate.

### `memory_types`
Optional backend-neutral memory/fact-type routing hints for this hook scope. An empty or omitted list means no type filtering.

When the backend supports native fact/source types, map these values to its closest native filters. For Hindsight the native recall values are `world`, `experience`, and `observation`.

Scope-wide `memory_types` are defaults. A structured `recall_queries` entry may supply its own `memory_types`; query-local values take priority for that query rather than being broadened by the scope-wide defaults.

Do not invent a type merely to populate the field. If classification is genuinely ambiguous, omit it and let retrieval search across relevant types.

### `recall_queries`
The heart of MemHooks. Each entry should be a specific natural-language retrieval question whose answer would materially improve work in this directory.

Two forms are valid:

```yaml
# Simple/backward-compatible form
- "What production failures have involved token refresh?"

# Structured form
- query: "Why was the current token-refresh architecture selected?"
  memory_types: [experience]
  entities:
    - name: Authentication
      type: COMPONENT
```

Structured entries are preferred when the memory type or relevant typed entities are known. Query-local `entities` supplement the hook's top-level `entities` for that query.

Prefer `What architectural decisions govern this subsystem, and why were they made?` over a vague keyword such as `architecture`.

### `entities`
Named people, projects, services, components, concepts, or other entities likely to improve recall precision. Backends with entity-aware retrieval may use them directly. Others can incorporate them into natural-language queries.

Both forms are valid:

```yaml
entities:
  - Authentication
  - name: Auth Service
    type: SERVICE
```

A string entry is an untyped legacy entity. A mapping has:

- `name` — required canonical/display name for retrieval;
- `type` — optional semantic entity type.

Entity types are deliberately extensible rather than a closed enum. Use a backend-native type when one exists. Otherwise prefer a stable uppercase vocabulary. Recommended general/project types include:

`PERSON`, `ORG`, `PLACE`, `PROJECT`, `PRODUCT`, `SOFTWARE`, `SERVICE`, `COMPONENT`, `API`, `REPOSITORY`, `FILE`, `DOCUMENT`, `DATASET`, `MODEL`, `TECHNOLOGY`, `EVENT`, `CONCEPT`, `DECISION`, `REQUIREMENT`, `CONSTRAINT`, and `ISSUE`.

Custom types are allowed when they materially improve graph precision. Do not create multiple near-synonyms (`APP`, `APPLICATION`, `SOFTWARE_APP`) unless the backend itself distinguishes them.

Entity **edge/relationship types are not defined by MemHooks v1**. They belong to the memory backend's graph model and are a separate concern from identifying the node/entity type used for retrieval.

### `tags`
Optional tag/metadata hints. Use native filters when available; otherwise treat them as context for query construction.

### `knowledge_pages`
Optional references to existing curated summaries, mental models, project pages, or equivalent stable context.

**Retrieval only.** Loading a MemHooks file never authorizes the agent to create, ensure, rewrite, refresh, or delete such pages.

### `exclude`
Memories, approaches, entities, branches, or topics that should not be returned as current context for this subtree. This is intentionally first-class because obsolete but semantically similar memories can poison an otherwise good retrieval.

Use native negative filtering where available; otherwise filter returned results before adding them to working context.

### `sensitivity`
Optional advisory classification such as `public`, `internal`, or `private`. Respect the privacy boundary of the already configured memory system.

## Machine-maintained body blocks

A runtime may maintain bounded retrieval cues in the Markdown body without rewriting hand-authored frontmatter. The reference maintainer uses two reserved blocks:

```md
<!-- memhooks:auto:start -->
## Auto-maintained recall anchors
...
<!-- memhooks:auto:end -->

<!-- memhooks:notes:start -->
## In-session retrieval cues

```json
[
  {
    "query": "Why was refresh-token rotation split into two stages?",
    "memory_types": ["experience"],
    "entities": [{"name": "Authentication", "type": "COMPONENT"}]
  }
]
```
<!-- memhooks:notes:end -->
```

`auto` contains deterministic path-scoped anchors generated from tool activity. `notes` contains concise future-retrieval questions recorded during the same turn in which a non-obvious decision/failure/constraint was discovered. The reference maintainer stores notes as a small JSON array so optional memory types and typed entities survive maintenance without a YAML dependency.

These blocks are still **retrieval metadata, not memory content**. Implementations should preserve everything outside their own managed markers and keep the blocks bounded.

## Merge semantics

For the active directory:

1. Find all `MEMHOOKS.md` files from workspace root to leaf.
2. If a leafward file sets `inherits: false`, discard ancestors above that file.
3. Merge remaining files root → leaf.
4. Append list fields (`memory_types`, `recall_queries`, `entities`, `tags`, `knowledge_pages`, `exclude`) and de-duplicate exact duplicates.
5. For a structured query, its own `memory_types` take priority over scope-wide `memory_types` for that query; its entities supplement the merged top-level entities.
6. For scalar fields (`bank`, `scope`, `sensitivity`), the most local value wins.
7. Append free-form guidance root → leaf; local guidance has priority when instructions conflict.
8. Treat machine-maintained body blocks exactly as local retrieval guidance; do not interpret them as durable memory facts.

## Maintenance principle

The memory backend stores the actual event/decision/fact. `MEMHOOKS.md` stores only enough information to make a future agent realize **what it should ask memory about**, plus optional type information that makes that retrieval more precise.

Prefer deterministic maintenance where possible. A separate LLM summarization pass should not be required merely to keep the routing file alive.

## Non-goals

`MEMHOOKS.md` is not a memory database, hidden prompt dump, README replacement, memory-retention policy, dreaming/consolidation trigger, graph edge schema, or reason to retrieve everything remotely related to the task.
