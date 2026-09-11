# Protocol guide

This document explains the practical semantics of **`memhooks/v2`**. The normative specification is [`references/memhooks-format.md`](../references/memhooks-format.md).

## Mental model

`memhooks/v2` has two layers:

```text
core fields
    = provider-neutral retrieval intent

backends.<provider>
    = opaque provider-native retrieval controls
```

The core never promotes a provider's ontology into universal fields.

## Core shape

```md
---
schema: memhooks/v2
inherits: true
scope: backend/auth

recall_queries:
  - "What decisions matter here?"
  - query: "What security boundaries must never be violated?"
    priority: 1.0
    when:
      roles: [reviewer, architect]
    entities:
      - Authentication
    resources:
      - name: auth-postmortem
        kind: postmortem
        salience: 0.9
    tags: [security]
    backends:
      mem0:
        top_k: 8
      hindsight:
        memory_types: [experience]

entities:
  - name: Authentication
    type: CONCEPT
    salience: 0.95

resources:
  - auth-architecture

tags: [backend]
exclude:
  - obsolete OAuth prototype

sensitivity: private

backends:
  mem0:
    filters:
      user_id: auth-agent
  hindsight:
    bank: project-memory
---

Optional free-form retrieval guidance goes here.
```

## `schema`

Required:

```yaml
schema: memhooks/v2
```

The package version and protocol schema version are separate. The v0.5.x reference resolver implements v2 and rejects unsupported schemas during resolution.

## `inherits`

Optional boolean, default `true`.

```yaml
inherits: false
```

starts a new local hook scope: ancestor hooks above that file are discarded for the active subtree.

## `recall_queries`

The central field. Each query describes context whose answer could materially change current work.

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
    entities: [Queue]
    resources: [queue-postmortem]
    tags: [reliability]
    backends:
      mem0:
        rerank: true
```

### `priority`

Optional float `0.0..1.0`.

It means **importance of this recall request when context is limited**. It does not mean confidence, truth probability, semantic similarity, or provider relevance.

If omitted, the protocol does not invent a numeric default.

### `when.roles`

Optional open list of project/runtime role strings:

```yaml
when:
  roles: [reviewer, refactor]
```

A restricted query applies if any declared role exactly matches any active role. If no active role is known, the reference resolver preserves the query rather than guessing.

### Query-local cues

Structured queries can add query-local:

- `entities`
- `resources`
- `tags`
- `backends`

Generic entity/resource/tag cues supplement resolved scope cues. Query-local backend configuration overlays scope-level provider configuration structurally.

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

`type` is optional and intentionally open. MemHooks does not define a universal entity taxonomy.

`salience` is optional `0.0..1.0` cue importance, separate from query priority and provider search scores.

## `resources`

Named existing resources that may deserve retrieval/read access.

Simple:

```yaml
resources:
  - auth-architecture
```

Structured:

```yaml
resources:
  - name: outage-postmortem
    kind: postmortem
    salience: 0.9
```

`kind` is an optional open string. It can describe a document, postmortem, standing answer, design note, page, or another existing resource without forcing all providers to share one resource ontology.

Provider-native resource classes/IDs remain inside `backends.<provider>` when exact provider semantics matter.

## `tags`

Backend-neutral relevance/routing labels.

```yaml
tags: [security, backend]
```

An adapter may map them to native metadata filters when a real equivalent exists. Otherwise they remain query/routing hints.

## `exclude`

First-class anti-recall:

```yaml
exclude:
  - obsolete OAuth prototype
```

Adapters should use native negative filtering when appropriate or post-filter matching obsolete/misleading results before they enter context.

## `scope`

Optional descriptive scope label. It is not authoritative filesystem state.

## `sensitivity`

Optional advisory handling metadata. MemHooks intentionally does not define a closed sensitivity taxonomy. Host security/access control remains authoritative.

## `backends`

Opaque provider namespaces:

```yaml
backends:
  mem0:
    filters:
      user_id: alice
    top_k: 8

  hindsight:
    bank: project-memory
    memory_types: [experience]
```

The core knows only that:

- provider names are non-empty strings;
- each provider namespace is a mapping/object;
- the mapping must survive resolution with deterministic structural merge semantics.

The core does **not** know what `user_id`, `top_k`, `bank`, or `memory_types` means.

### Provider merge semantics

Parent:

```yaml
backends:
  mem0:
    filters:
      user_id: alice
    threshold: 0.1
```

Child:

```yaml
backends:
  mem0:
    threshold: 0.2
    rerank: true
```

Resolved:

```yaml
backends:
  mem0:
    filters:
      user_id: alice
    threshold: 0.2
    rerank: true
```

Rules:

- mapping + mapping → recursively merge;
- local scalar replaces parent scalar;
- local sequence/list replaces parent sequence/list;
- query-local namespace overlays resolved scope-level namespace.

Lists are replaced rather than automatically concatenated because MemHooks cannot know the provider's list semantics.

## Provider mappings

Provider-native concepts live in separate references:

- [Hindsight](../references/memory-systems/01-hindsight.md)
- [OpenViking](../references/memory-systems/02-openviking.md)
- [Honcho](../references/memory-systems/03-honcho.md)
- [Mem0](../references/memory-systems/04-mem0.md)
- [Generic / unknown](../references/memory-systems/99-generic-or-unknown.md)

Examples such as Hindsight Reflect/Mental Models or Mem0 filters/reranking are intentionally absent from the **core** field list.

## Root-to-leaf merge semantics

For a target directory:

1. discover `MEMHOOKS.md` root → leaf;
2. reject unsupported schema versions;
3. if a leafward hook sets `inherits: false`, discard earlier ancestors;
4. merge remaining hooks root → leaf;
5. accumulate core list fields with exact duplicate suppression;
6. use the most local specified scalar core value;
7. preserve structured-query priority/roles/local cues;
8. structurally merge provider mappings;
9. append free-form guidance with source provenance;
10. apply role filtering when active roles are actually known.

The reference implementation preserves source paths for merged queries/entities/resources/guidance where applicable.

## Agent/runtime ownership

`MEMHOOKS.md` is normally maintained by the **agent/runtime**, not manually curated by the end user.

Operational guidance such as keeping routing cues concise or removing stale cues applies to the component doing automatic maintenance.

The human-facing workflow should normally be: enable MemHooks once, configure the memory system if needed, then let the agent/runtime maintain routing metadata.

## Non-goals

`MEMHOOKS.md` is not:

- a memory database;
- a transcript dump;
- a README replacement;
- a prompt-privilege declaration;
- a memory-retention policy;
- an arbitrary command execution manifest;
- a universal provider configuration schema;
- a directive store.

The **agent/runtime maintainer** should keep routing metadata concise enough to function as an index of what must be remembered rather than storing the memories themselves.
