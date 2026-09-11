# `MEMHOOKS.md` format — memhooks/v2

A `MEMHOOKS.md` file is Markdown with YAML frontmatter. The frontmatter is retrieval-routing metadata. The optional Markdown body is additional agent/runtime-readable retrieval guidance.

`memhooks/v2` deliberately separates **backend-neutral retrieval intent** from **provider-native controls**.

The core protocol does not define Hindsight memory types, Mem0 filters, Honcho dialectic controls, OpenViking hierarchy controls, or any equivalent provider vocabulary. Those belong under `backends.<provider>` and are interpreted only by the corresponding runtime adapter.

The MemHooks package version and protocol schema version are separate. MemHooks package v0.5.0 implements `memhooks/v2`.

## Canonical shape

```md
---
schema: memhooks/v2
inherits: true
scope: backend/auth

recall_queries:
  - "What architectural decisions govern this subsystem?"

  - query: "Why did the authentication design change after the outage?"
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
        rerank: true
      hindsight:
        memory_types: [experience]
        connection_types: [causal, temporal]
        strategy: reflect

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

Use concrete recall first. Use provider-specific synthesis only when the active
backend exposes it and the task actually requires it.
```

## Required field

### `schema`

```yaml
schema: memhooks/v2
```

The v0.5.x reference resolver accepts `memhooks/v2`. Unsupported schemas must fail clearly rather than being silently reinterpreted.

## Core fields

### `inherits`

```yaml
inherits: true
```

Optional boolean. Defaults to `true`.

A hook with `inherits: false` cuts off all ancestor hooks above it for that target path. The hook itself and any deeper hooks still participate.

### `scope`

```yaml
scope: backend/auth
```

Optional string. This is descriptive/backend-neutral scope metadata. More local values replace parent values.

### `recall_queries`

A list of concrete questions worth asking memory when work occurs in this scope.

Simple form:

```yaml
recall_queries:
  - "What failures have happened in this subsystem before?"
```

Structured form:

```yaml
recall_queries:
  - query: "Why was token rotation split into two stages?"
    priority: 0.9
    when:
      roles: [reviewer]
    entities:
      - Authentication
    resources:
      - auth-postmortem
    tags: [security]
    backends:
      mem0:
        top_k: 6
```

A structured query may contain:

| Field | Type | Meaning |
|---|---|---|
| `query` | string | required natural-language retrieval request |
| `priority` | number `0.0..1.0` | optional importance under context pressure |
| `when.roles` | list of strings | optional role applicability |
| `entities` | entity list | query-local entity cues |
| `resources` | resource list | query-local existing resources |
| `tags` | string list | query-local neutral routing labels |
| `backends` | mapping | query-local provider-native hints |

The core validator does not define provider-native query fields outside `backends`.

### `priority`

`priority` belongs to a recall request.

It means: **if the context budget cannot satisfy every applicable retrieval request equally, preserve higher-priority requests first.**

It does **not** mean:

- semantic similarity;
- backend search score;
- truth probability;
- memory confidence;
- a mandatory multiplier in one universal ranking formula.

If omitted, there is no protocol-mandated numeric default.

### `when.roles`

```yaml
when:
  roles: [reviewer, architect]
```

Role names are open project/runtime-defined strings.

When active roles are known, a restricted query applies if **any** listed role exactly matches an active role. A query without `when.roles` applies universally.

If the runtime has no active-role concept, it must preserve role-restricted queries rather than silently dropping them.

### `entities`

Entities are named retrieval cues.

Simple form:

```yaml
entities:
  - Authentication
```

Structured form:

```yaml
entities:
  - name: OpenAI
    type: ORG
    salience: 0.9
```

`type` is an optional open string. MemHooks does not define a universal entity taxonomy.

`salience`, when present, is `0.0..1.0` and expresses importance of that entity **as a retrieval cue**. It is separate from query priority and backend relevance/confidence.

### `resources`

Resources are named existing things that may deserve retrieval/read access when working in the scope.

Simple form:

```yaml
resources:
  - auth-architecture
```

Structured form:

```yaml
resources:
  - name: outage-postmortem
    kind: postmortem
    salience: 0.9
```

`kind` is an optional open string. It may describe a document, standing answer, postmortem, knowledge page, design note, or another project/runtime resource without forcing every backend to share one taxonomy.

`salience`, when present, is `0.0..1.0` and expresses cue importance.

A provider may map a generic resource to its native resource concept, but provider-native resource controls belong under `backends.<provider>`.

### `tags`

```yaml
tags: [security, backend]
```

Backend-neutral routing/relevance labels. An adapter may map them to native metadata filters when a real equivalent exists. Otherwise they remain generic hints.

### `exclude`

```yaml
exclude:
  - obsolete OAuth prototype
```

Material that should not enter the current working context when matched by retrieval. The active adapter may use native negative filters or post-filter results.

### `sensitivity`

```yaml
sensitivity: private
```

Optional advisory handling metadata. The protocol does not define a closed sensitivity taxonomy and never overrides host authorization/security policy.

## Provider namespaces: `backends`

```yaml
backends:
  mem0:
    filters:
      user_id: alice
    top_k: 8
    threshold: 0.15
    rerank: true

  hindsight:
    bank: project-memory
    memory_types: [experience]
    connection_types: [causal, temporal]
```

`backends` is a mapping from provider/adapter name to an **opaque mapping/object**.

The MemHooks core:

- preserves the namespace;
- validates that each namespace contains a mapping/object;
- structurally merges it through inheritance;
- exposes the resolved mapping to adapters/CLI consumers;
- does **not** validate or interpret provider-internal keys.

This is the central v2 boundary.

### Provider namespace merge semantics

Given parent:

```yaml
backends:
  mem0:
    filters:
      user_id: alice
    threshold: 0.2
```

and child:

```yaml
backends:
  mem0:
    threshold: 0.3
    rerank: true
```

resolved scope configuration is structurally:

```yaml
backends:
  mem0:
    filters:
      user_id: alice
    threshold: 0.3
    rerank: true
```

Rules are intentionally provider-agnostic:

1. mapping/object + mapping/object → recursive merge;
2. scalar/sequence/other value at a more local scope → replace parent value;
3. query-local `backends.<provider>` overlays the resolved scope-level namespace using the same rules.

MemHooks does not concatenate provider lists because it cannot know whether a provider interprets a list as additive, ordered, exclusive, or something else.

## Provider examples

These are examples of namespacing, not universal schema fields.

### Hindsight

```yaml
backends:
  hindsight:
    bank: project-memory
    memory_types: [world, experience]
    connection_types: [causal, temporal]
    strategy: reflect
    mental_models:
      - auth architecture
```

See [`memory-systems/01-hindsight.md`](memory-systems/01-hindsight.md).

### Mem0

```yaml
backends:
  mem0:
    filters:
      user_id: alice
    top_k: 10
    threshold: 0.1
    rerank: true
```

See [`memory-systems/04-mem0.md`](memory-systems/04-mem0.md).

### OpenViking

```yaml
backends:
  openviking:
    # adapter-defined OpenViking-native controls
```

See [`memory-systems/02-openviking.md`](memory-systems/02-openviking.md).

### Honcho

```yaml
backends:
  honcho:
    # adapter-defined Honcho-native controls
```

See [`memory-systems/03-honcho.md`](memory-systems/03-honcho.md).

## Root-to-leaf resolution

For:

```text
/repo/MEMHOOKS.md
/repo/backend/MEMHOOKS.md
/repo/backend/auth/MEMHOOKS.md
```

an agent working in `/repo/backend/auth/` resolves hooks root → leaf.

Core semantics:

- `inherits: false` clears ancestors above that hook;
- `scope` and `sensitivity` use the most local specified value;
- `recall_queries`, `entities`, `resources`, `tags`, and `exclude` accumulate root → leaf with exact duplicate suppression;
- priority and role conditions stay attached to their structured query;
- query-local entities/resources/tags supplement resolved top-level cues;
- role filtering happens after resolution when active roles are known;
- `backends` follows the structural merge rules above.

Free-form Markdown bodies from applicable hooks remain retrieval guidance with source provenance.

## Generic resources versus provider resources

The core `resources` field exists because “there is a named existing thing worth consulting” is useful across providers and runtimes.

It does not assert that every backend has Mental Models, Knowledge Pages, pages, peers, documents, or another specific resource class.

If a provider needs native resource identifiers or retrieval modes, place those under its backend namespace.

## Validation boundary

The reference validator checks core syntax/semantics such as:

- schema must be `memhooks/v2`;
- query/entity/resource names are non-empty;
- priority and salience are within `0.0..1.0`;
- role names are non-empty;
- backend namespace names are non-empty;
- each backend namespace contains a mapping/object;
- duplicate core cues and obvious recall/exclude contradictions;
- unknown core fields.

The validator deliberately does **not** claim to validate arbitrary provider-native configuration inside `backends.<provider>`.

Provider adapters may add their own validation layer.

## Agent/runtime ownership

`MEMHOOKS.md` is normally maintained by the agent/runtime, not manually curated by the end user.

Agents should keep routing metadata concise enough to remain an index rather than memory content. This is an operational rule for agents and adapter authors.

Deterministic maintainers must not guess semantic priority, roles, entities, resources, or provider controls merely from touched file paths.

## Security and trust boundary

A hook file is repository-controlled retrieval metadata. It does not grant itself execution or prompt privilege.

Loading a hook must not by itself:

- execute arbitrary commands;
- write, alter, consolidate, or delete memories;
- elevate repository text to system/developer authority;
- bypass host authorization;
- fabricate backend support;
- start background services.

The host runtime owns execution, authorization, prompt placement, backend credentials, and context-budget policy.

## Design principle

The v2 split is intentionally simple:

```text
MemHooks core = what should be remembered here?

backends.<provider> = how this particular memory system should help retrieve it
```

The core carries provider-native configuration without becoming a lowest-common-denominator imitation of every memory system.
