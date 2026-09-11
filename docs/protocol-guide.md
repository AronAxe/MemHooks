# Protocol guide

This document explains the practical semantics of `memhooks/v1`. The normative specification is [`references/memhooks-format.md`](../references/memhooks-format.md).

## Core shape

```md
---
schema: memhooks/v1
inherits: true
bank: optional-bank
scope: optional/subsystem

memory_types: []
connection_types: []
mental_models: []
knowledge_pages: []

recall_queries:
  - "What decisions matter here?"
  - query: "What security boundaries must never be violated?"
    priority: 1.0
    when:
      roles: [reviewer, architect]
    memory_types: [experience]
    connection_types: [causal, temporal]
    entities:
      - name: Authentication
        type: CONCEPT
        salience: 0.95

entities: []
tags: []
exclude: []
sensitivity: private
---

Optional free-form retrieval guidance goes here.
```

## `schema`

Required. Current value:

```yaml
schema: memhooks/v1
```

The software package version can change without changing the protocol version. MemHooks 0.4.x still uses `memhooks/v1` because the new fields are backward-compatible additions.

## `inherits`

Optional, default `true`.

```yaml
inherits: false
```

starts a new local hook scope. When the resolver encounters it, ancestor hooks above that file are discarded for the active subtree.

## `recall_queries`

The central field. Each query should describe context whose answer could materially improve work in this directory.

Simple form:

```yaml
recall_queries:
  - "Why was the current queue architecture selected?"
```

Structured form:

```yaml
recall_queries:
  - query: "Which previous failures involved queue ordering?"
    priority: 0.9
    when:
      roles: [reviewer]
    memory_types: [experience]
    connection_types: [causal, temporal]
```

### `priority`

Optional float from `0.0` through `1.0`.

It means **importance of this recall request when context is limited**. It does not mean confidence, truth probability, semantic similarity, or backend relevance.

If omitted, the protocol does not invent a numeric default.

### `when.roles`

Optional open list of runtime/project role strings.

```yaml
when:
  roles: [reviewer, refactor]
```

A restricted query applies if any declared role exactly matches any active role. If no active role is known, the reference resolver preserves the query instead of guessing.

## `memory_types`

Optional memory-category routing hints.

For the Hindsight mapping:

```text
world | experience | observation
```

Scope-wide values are defaults. Query-local values override those defaults for that query.

## `connection_types`

Optional retrieval-structure hints:

```text
semantic | temporal | entity | causal
```

These are separate from memory categories and separate from entity types.

Scope-wide values are defaults. Query-local values override them for that query.

## `entities`

Named retrieval cues.

Simple:

```yaml
entities:
  - Authentication
```

Structured:

```yaml
entities:
  - name: OpenAI
    type: ORG
    salience: 0.9
```

`type` is optional and intentionally not a universal closed taxonomy. Preserve explicit backend/user/source types when known.

`salience` is an optional `0.0..1.0` measure of the entity's importance as a retrieval cue. It is distinct from query priority.

Query-local entities supplement the merged top-level entity set for that effective query.

## `mental_models` and `knowledge_pages`

These refer to **existing** higher-level synthesized resources when the memory backend supports such concepts.

They are not raw memory types, and loading a hook does not authorize creating or rewriting them.

## `tags`

Optional backend-neutral relevance/scoping hints. Use native tag/metadata filters when a backend exposes them; otherwise treat them as query context.

## `exclude`

First-class anti-recall.

```yaml
exclude:
  - obsolete OAuth prototype
```

Use this to keep semantically tempting but obsolete/misleading context out of the active task. A backend adapter can use native negative filtering or post-filter returned results.

## `bank`

Optional namespace/bank/session hint. It only has meaning when the configured memory backend has an equivalent concept and the runtime is authorized to access it.

## `scope`

Optional descriptive subsystem/path label. It is a hint, not authoritative filesystem state.

## `sensitivity`

Optional advisory handling metadata such as:

```text
public | internal | private
```

The host runtime remains responsible for real access control and privacy boundaries.

## Merge semantics

For a target directory:

1. discover `MEMHOOKS.md` from root to leaf;
2. if a leafward file sets `inherits: false`, discard earlier ancestors;
3. merge remaining hooks root to leaf;
4. append list fields and remove exact duplicates;
5. use the most local value for scalar fields such as `bank`, `scope`, and `sensitivity`;
6. preserve query-local priority/roles/routing metadata on that query;
7. resolve query-local memory/connection defaults when producing effective queries;
8. append free-form guidance root to leaf.

The reference implementation preserves source paths for merged queries/entities/guidance so adapters can retain provenance.

## Non-goals

`MEMHOOKS.md` is not:

- a memory database;
- a transcript dump;
- a README replacement;
- a prompt-privilege declaration;
- a memory-retention policy;
- an arbitrary command execution manifest;
- a vector-store configuration file;
- a directive store.

The file should remain small enough to function as an index of what must be remembered, not the memory itself.
