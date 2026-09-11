# `MEMHOOKS.md` format — memhooks/v1

A `MEMHOOKS.md` file is Markdown with YAML frontmatter. The frontmatter is retrieval-routing metadata. The body is optional human/agent-readable retrieval guidance.

`memhooks/v1` is intentionally backward-compatible. Existing string-only `recall_queries` and `entities` remain valid. Structured entries can preserve memory type, connection emphasis, typed entities, retrieval priority, entity salience, and role routing when those are genuinely known.

The MemHooks release/package version and schema version are separate. MemHooks **v0.4.0** continues to use `schema: memhooks/v1` because the new fields are optional and backward-compatible.

## Canonical shape

```md
---
schema: memhooks/v1
bank: optional-memory-namespace
scope: optional/human-readable/subsystem
inherits: true

# Optional memory categories. Omit/empty = search all relevant categories.
# Hindsight: world | experience | observation
memory_types: []

# Optional connection emphasis for retrieval. Omit/empty = no preference.
# Hindsight knowledge connections: semantic | temporal | entity | causal
connection_types: []

# Existing standing answers / synthesized resources worth reading first.
mental_models: []
knowledge_pages:
  - "Architecture/Authentication"

recall_queries:
  # Legacy/simple form remains valid.
  - "What previous failures or rejected fixes involved token refresh?"

  # Structured form keeps routing metadata with the query.
  - query: "Why was the current authentication architecture selected?"
    priority: 1.0
    when:
      roles:
        - reviewer
        - architect
    memory_types:
      - experience
    connection_types:
      - causal
      - temporal
    entities:
      - name: OpenAI
        type: ORG
        salience: 0.9
      - Authentication

entities:
  # Legacy/untyped form.
  - Authentication

  # Structured form when metadata is actually known.
  - name: OpenAI
    type: ORG
    salience: 0.8

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

## Ontology: keep the layers separate

MemHooks preserves distinctions made by the attached memory backend rather than flattening them into one generic `type` or `weight` field.

For Hindsight in particular:

1. **Memory categories**: `world`, `experience`, `observation`.
2. **Knowledge connections**: entity, time-based/temporal, meaning-based/semantic, and causal connections.
3. **Entities**: named things such as people, organizations, places, products, and concepts; an entity may have its own entity type.
4. **Mental models / Knowledge Pages**: higher-level, precomputed standing answers or documents; they are not another raw-memory category.
5. **Relationships/directives/documents**: backend-native bank data. MemHooks may route toward them when a dedicated field exists, but must not relabel them as memory or entity types.
6. **Query priority**: the importance of satisfying a recall request under a limited context budget; it is not semantic similarity or confidence.
7. **Entity salience**: the importance of an entity as a retrieval cue; it is not query priority or a backend relevance score.
8. **Role routing**: applicability of a query to an agent/runtime role; role names are project/runtime-defined rather than an ontology of memories.

These dimensions are independent. A query can target an `experience`, emphasize `causal` and `temporal` connections, mention a salient `PERSON` entity, carry high priority, and apply only to a reviewer at the same time.

## Fields

### `schema`

Required. For this version use:

```yaml
schema: memhooks/v1
```

### `inherits`

Optional; default `true`. With `false`, this file starts a new hook scope and ancestors above it are ignored for the current subtree.

### `scope`

Optional descriptive path/subsystem label. It is a hint, not authoritative filesystem state.

### `bank`

Optional memory namespace/bank/peer/session hint. Use it only when the current backend has an equivalent and access is appropriate.

### `memory_types`

Optional memory-category routing hints for this hook scope. An empty or omitted list means no category restriction.

For Hindsight, the public Recall API accepts exactly these memory categories:

- `world` — facts about the outside world;
- `experience` — the bank agent's own experiences/events/interactions;
- `observation` — deduplicated, evidence-grounded beliefs consolidated from multiple memories.

Hindsight documents that **each selected memory type runs the full four-strategy retrieval pipeline independently**. Do not treat semantic/temporal/entity/causal as subtypes of `world`, `experience`, or `observation`.

Scope-wide `memory_types` are defaults. A structured `recall_queries` entry may supply its own `memory_types`; query-local values take priority for that query.

Do not guess merely to fill the field. If classification is ambiguous, omit it.

### `connection_types`

Optional routing hints describing which knowledge connections are especially relevant to the query:

- `semantic` — meaning-based similarity;
- `temporal` — time-based proximity/order;
- `entity` — shared or connected entities;
- `causal` — cause/effect relationships.

These are **not memory types** and they are **not entity types**.

For Hindsight, these four names mirror the documented connection classes used to organize memories. The public Recall API still runs its retrieval strategies together rather than exposing a simple `connection_types=` filter. Therefore a Hindsight adapter should use these values to sharpen the natural-language query, choose an appropriate retrieval depth, or use a backend-native control only when that control is actually exposed. Never invent an unsupported filter.

A query-local `connection_types` list overrides the scope-wide default for that query.

### `recall_queries`

The heart of MemHooks. Each entry should be a specific natural-language retrieval question whose answer would materially improve work in this directory.

Two forms are valid:

```yaml
# Simple/backward-compatible form
- "What production failures have involved token refresh?"

# Structured form
- query: "Why was the current token-refresh architecture selected?"
  priority: 0.9
  when:
    roles: [reviewer, architect]
  memory_types: [experience]
  connection_types: [causal, temporal]
  entities:
    - name: OpenAI
      type: ORG
      salience: 0.8
```

Structured entries are preferred when routing metadata is known. Query-local entities supplement the hook's top-level entities for that query.

Prefer `What architectural decisions govern this subsystem, and why were they made?` over a vague keyword such as `architecture`.

#### `priority`

Optional on a structured recall query. Valid values are floating-point numbers from `0.0` through `1.0` inclusive.

`priority` represents **how important it is to preserve/satisfy this recall request when retrieval results must compete for a finite context budget**. Higher priority may be used by a context manager to retain this query's results before lower-priority material.

It does **not** mean:

- semantic similarity;
- memory confidence;
- truth probability;
- entity importance;
- a requirement to multiply the backend's native relevance score by this number.

MemHooks deliberately defines no mandatory default numerical value. If `priority` is omitted, the query retains ordinary/unweighted behavior. An adapter may have its own normal/default ranking policy, but it must not rewrite the hook as though an explicit priority had been authored.

A priority is a routing hint, not permission to exceed context limits or bypass safety/privacy controls.

#### `when.roles`

Optional applicability condition for a structured query:

```yaml
when:
  roles: [reviewer, refactor]
```

Role names are open strings defined by the project or runtime. MemHooks does not impose a universal taxonomy such as `reviewer`, `architect`, `feature_dev`, etc.

Semantics:

- no `when.roles` (or an empty list) means the query is not role-restricted;
- when an adapter knows one or more active roles, a role-restricted query applies when **any** listed role matches any active role;
- when a generic parser/resolver has no active-role information, it may preserve all queries and defer role filtering to the role-aware consumer;
- role matching in the reference implementation is exact string matching; runtimes wanting aliases should normalize roles before passing them to the resolver rather than silently changing MemHooks semantics.

`when` is intentionally an object rather than a bare `roles` field so future backward-compatible applicability conditions can be added without redesigning the query shape.

### `entities`

Named entities likely to improve recall precision. Backends with entity-aware retrieval may use them directly. Others can incorporate them into natural-language queries.

Both forms are valid:

```yaml
entities:
  - Authentication
  - name: OpenAI
    type: ORG
    salience: 0.9
```

A string entry is an untyped/unweighted entity. A mapping has:

- `name` — required canonical/display name for retrieval;
- `type` — optional entity type, **only when known**;
- `salience` — optional retrieval-cue importance from `0.0` through `1.0`.

For Hindsight, explicit retain entities use `{text, type?}` and an omitted type defaults to `CONCEPT`. The public docs give `PERSON`, `ORG`, and `CONCEPT` as examples and separately describe automatic recognition of people, organizations, places, products, and concepts. Hindsight's API accepts the entity type as a string rather than publishing a closed enum.

Therefore MemHooks does **not** invent a universal entity-type taxonomy. Preserve backend-native or user-defined type strings when supplied, and otherwise leave the entity untyped. A decision, requirement, constraint, failure, or other proposition should not be promoted to an entity merely to classify the memory containing it.

#### `salience`

Optional on a structured entity. Valid values are `0.0` through `1.0` inclusive.

`salience` says how important that entity is as a cue when an adapter has to prioritize or trim retrieval context. It is deliberately distinct from recall-query `priority` and from backend-produced relevance/confidence scores. Omission means no explicit salience was authored; MemHooks defines no mandatory numeric default.

Entity-to-entity relationship/edge types are a separate backend concern and are intentionally not defined by MemHooks v1.

### `mental_models`

Optional names/identifiers of existing standing answers that should be read before ordinary recall when the backend supports them.

In Hindsight, a mental model is a deliberately curated, stored answer to a question about a bank. It sits above observations and raw facts in Reflect's retrieval ladder. Loading a MemHooks file never authorizes creation or rewriting of a mental model.

### `knowledge_pages`

Optional references to existing Knowledge Pages or equivalent stable synthesized documents.

For Hindsight, Knowledge Pages are built on the mental-model layer. Treat them as retrieval targets, not raw memories and not entities.

**Retrieval only.** Loading a MemHooks file never authorizes the agent to create, ensure, rewrite, refresh, or delete such pages.

### `tags`

Optional tag/metadata hints. Use native filters when available; otherwise treat them as context for query construction.

### `exclude`

Memories, approaches, entities, branches, or topics that should not be returned as current context for this subtree. This is intentionally first-class because obsolete but semantically similar memories can poison an otherwise good retrieval.

Use native negative filtering where available; otherwise filter returned results before adding them to working context.

### `sensitivity`

Optional advisory classification such as `public`, `internal`, or `private`. Respect the privacy boundary of the already configured memory system.

## Machine-maintained body blocks

A runtime may maintain bounded retrieval cues in the Markdown body without rewriting hand-authored frontmatter. The reference maintainer uses two reserved blocks:

````md
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
    "priority": 0.9,
    "when": {"roles": ["reviewer"]},
    "memory_types": ["experience"],
    "connection_types": ["causal", "temporal"],
    "entities": [{"name": "OpenAI", "type": "ORG", "salience": 0.8}]
  }
]
```
<!-- memhooks:notes:end -->
````

`auto` contains deterministic path-scoped anchors generated from tool activity. Since zero-LLM path detection cannot reliably classify semantic memory metadata, auto anchors remain intentionally untyped/unweighted/unrouted.

`notes` contains concise future-retrieval questions recorded during the same turn in which a non-obvious decision/failure/constraint was discovered. Structured notes may preserve priority, role applicability, memory categories, connection hints, and typed/salient entities when those values are actually known.

These blocks are still **retrieval metadata, not memory content**. Implementations should preserve everything outside their own managed markers and keep the blocks bounded.

## Merge semantics

For the active directory:

1. Find all `MEMHOOKS.md` files from workspace root to leaf.
2. If a leafward file sets `inherits: false`, discard ancestors above that file.
3. Merge remaining files root → leaf.
4. Append list fields (`memory_types`, `connection_types`, `recall_queries`, `entities`, `mental_models`, `knowledge_pages`, `tags`, `exclude`) and de-duplicate exact duplicates.
5. For a structured query, query-local `memory_types` and `connection_types` take priority over scope-wide defaults; its entities supplement the merged top-level entities.
6. `priority` and `when` remain properties of their structured query. `salience` remains a property of its structured entity. They are not separately merged into unrelated entries.
7. If two structured entries have the same natural-language query/entity name but different routing metadata, they are not exact duplicates; tooling may warn about the ambiguity rather than silently inventing precedence.
8. For scalar fields (`bank`, `scope`, `sensitivity`), the most local value wins.
9. Append free-form guidance root → leaf; local guidance has priority when instructions conflict.
10. Treat machine-maintained body blocks exactly as local retrieval guidance; do not interpret them as durable memory facts.
11. Role filtering happens **after** inheritance resolution when active-role information is available.

## Reference resolver and validator (v0.4.0+)

The Rust reference implementation defines the interoperable parsing/resolution behavior and is also exposed through the `memhooks` CLI.

```bash
# Validate hooks under the current path
memhooks validate

# Validate the complete repository
memhooks validate --all

# Machine-readable validation
memhooks validate --all --format json
memhooks validate --all --format sarif

# Explain the effective inherited context for a directory
memhooks explain backend/auth

# Explain it as seen by a role-aware consumer
memhooks explain backend/auth --role reviewer
```

The reference tooling validates schema values, priority/salience ranges, memory/connection categories, routing-condition structure, duplicates, and other contradictions it can identify without inventing backend semantics. It deliberately does not execute shell commands or contact a memory backend.

## Maintenance principle

The memory backend stores the actual event/decision/fact. `MEMHOOKS.md` stores only enough information to make a future agent realize **what it should ask memory about**, plus optional routing metadata that makes that retrieval more precise.

Prefer deterministic maintenance where possible. A separate LLM summarization pass should not be required merely to keep the routing file alive.

## Non-goals

`MEMHOOKS.md` is not a memory database, hidden prompt dump, README replacement, memory-retention policy, dreaming/consolidation trigger, entity-relationship schema, directive store, arbitrary command-execution manifest, or reason to retrieve everything remotely related to the task.
