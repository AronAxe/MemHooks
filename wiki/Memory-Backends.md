# Memory Backends

MemHooks is backend-neutral by design.

The core describes **what should be recalled**. Provider namespaces describe **how one particular memory system may help retrieve it**.

## The split

```text
MemHooks core
    = retrieval intent that makes sense across providers

backends.<provider>
    = provider-native controls
```

Example:

```yaml
recall_queries:
  - query: "Why did authentication change after the outage?"
    priority: 0.9
    entities: [Authentication]
    backends:
      mem0:
        top_k: 8
        rerank: true

backends:
  mem0:
    filters:
      user_id: project-agent
```

`query`, `priority`, and `entities` are MemHooks concepts.

`filters`, `top_k`, and `rerank` are Mem0 concepts carried under `backends.mem0`.

## Namespace presence does not select a backend

A hook may contain several namespaces:

```yaml
backends:
  hindsight: {...}
  mem0: {...}
  openviking: {...}
  honcho: {...}
```

That does **not** mean the runtime should call all four systems.

The host runtime decides which backend is:

- configured;
- connected;
- authorized;
- appropriate for the current session.

Then it interprets only the matching provider namespace.

## Hindsight

Hindsight-specific concepts belong under:

```yaml
backends:
  hindsight:
    bank: project-memory
    memory_types: [experience]
    connection_types: [causal, temporal]
    strategy: reflect
    mental_models:
      - auth architecture
```

MemHooks does not treat `bank`, `memory_types`, Reflect, or Mental Models as universal schema fields.

The adapter may use them when the active backend is Hindsight.

Reference: [Hindsight mapping](https://github.com/AronAxe/MemHooks/blob/main/references/memory-systems/01-hindsight.md)

## Mem0

Mem0-specific controls belong under:

```yaml
backends:
  mem0:
    filters:
      user_id: alice
      agent_id: auth-agent
    top_k: 10
    threshold: 0.1
    rerank: true
```

A MemHooks `entity` is a generic retrieval cue. Mem0 identifiers such as `user_id`, `agent_id`, `app_id`, or `run_id` are provider-native memory-scope/filter concepts and should not be conflated with core entities.

Reference: [Mem0 mapping](https://github.com/AronAxe/MemHooks/blob/main/references/memory-systems/04-mem0.md)

## OpenViking

OpenViking-native hierarchy/resource/retrieval controls remain under:

```yaml
backends:
  openviking:
    # provider-owned configuration
```

Reference: [OpenViking mapping](https://github.com/AronAxe/MemHooks/blob/main/references/memory-systems/02-openviking.md)

## Honcho

Honcho-native session/peer/reasoning controls remain under:

```yaml
backends:
  honcho:
    # provider-owned configuration
```

Reference: [Honcho mapping](https://github.com/AronAxe/MemHooks/blob/main/references/memory-systems/03-honcho.md)

## Unknown or custom backend

A runtime does not need first-class MemHooks support to use the generic core.

At minimum it can translate:

```text
recall query → native search/query
entities     → query expansion/filter hints where sensible
resources    → existing resource lookup where supported
tags         → metadata filters where a real equivalent exists
exclude      → negative filtering/post-filtering
priority     → context-budget preference
```

If no native equivalent exists, the adapter should preserve the generic intent as far as practical without inventing fake provider features.

Reference: [Generic/unknown backend](https://github.com/AronAxe/MemHooks/blob/main/references/memory-systems/99-generic-or-unknown.md)

## Structural merging

Provider namespaces are opaque, so MemHooks uses structural rules rather than provider-specific semantics.

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

Mappings recursively merge. More-local scalars/lists replace broader values.

## Provider validation

The generic validator can verify that:

```yaml
backends:
  mem0:
    ...
```

contains a mapping/object.

It deliberately does not claim to know whether every key inside that object is valid for the provider's current SDK/API.

Provider adapters may add stricter validation.

## Why this matters

Without namespacing, a protocol tends to become one of two bad things:

1. a lowest-common-denominator abstraction that strips useful provider features; or
2. a universal schema polluted by whichever backend was integrated first.

MemHooks chooses a third option:

> **Keep universal retrieval intent small; let each backend remain itself.**