# Hindsight adapter mapping — memhooks/v2

Use this reference when the active memory backend is **Hindsight** (Vectorize) or the runtime exposes Hindsight-compatible memory tools.

In `memhooks/v2`, Hindsight vocabulary is **provider-native**, not part of the MemHooks core.

```yaml
backends:
  hindsight:
    ... Hindsight-specific controls ...
```

The Rust core preserves and structurally merges this mapping but does not interpret it.

## Hindsight operations

Hindsight's main memory operations include:

- **Recall** — direct ranked retrieval;
- **Reflect** — deeper reasoning/synthesis over stored knowledge;
- **Retain** — memory writing.

MemHooks is retrieval-routing infrastructure. Loading a hook must not call Retain merely because retrieval metadata exists.

## Recommended namespace conventions

A Hindsight-aware adapter may interpret these keys under `backends.hindsight`:

| Key | Meaning |
|---|---|
| `bank` | target Hindsight memory bank/namespace when the runtime exposes banks |
| `memory_types` | Hindsight memory categories such as `world`, `experience`, `observation` |
| `connection_types` | retrieval emphasis such as `semantic`, `temporal`, `entity`, `causal` |
| `strategy` | adapter retrieval strategy hint such as `recall`, `reflect`, or `auto` |
| `mental_models` | existing Hindsight Mental Models worth consulting |
| `knowledge_pages` | existing Hindsight Knowledge Pages worth consulting |

These keys are an **adapter convention**, not universal MemHooks fields. The generic validator intentionally does not validate their internal values.

## Example

```yaml
schema: memhooks/v2

recall_queries:
  - query: "Why did authentication change after the outage?"
    priority: 1.0
    entities:
      - Authentication
    backends:
      hindsight:
        memory_types: [experience]
        connection_types: [causal, temporal]
        strategy: reflect

backends:
  hindsight:
    bank: project-memory
    mental_models:
      - authentication architecture
```

The query, priority, and entity are backend-neutral. Everything inside `backends.hindsight` is Hindsight-specific.

## Hindsight ontology

### Memory categories

Hindsight distinguishes:

- `world` — facts about the outside world;
- `experience` — events/conversations/experiences from the bank agent's perspective;
- `observation` — consolidated evidence-backed beliefs.

These belong under:

```yaml
backends:
  hindsight:
    memory_types: [world, experience, observation]
```

or under a structured query's `backends.hindsight` mapping.

They must **not** appear as top-level MemHooks v2 fields.

### Knowledge connections

Hindsight knowledge can be connected semantically, temporally, through entities, and causally.

A v2 adapter may preserve emphasis as:

```yaml
backends:
  hindsight:
    connection_types: [semantic, temporal, entity, causal]
```

Do not invent a public Hindsight API parameter when the actual wrapper does not expose one. When no native control exists, use the hint to sharpen the natural-language query or choose an appropriate retrieval path.

### Entities

MemHooks core entities remain generic:

```yaml
entities:
  - name: OpenAI
    type: ORG
    salience: 0.9
```

A Hindsight adapter may map `name` to Hindsight's entity text and preserve an explicit type when appropriate. Do not guess entity types merely to populate metadata.

### Observations

Observations are Hindsight-native consolidated beliefs. They remain a Hindsight memory category, not a universal MemHooks concept.

### Mental Models and Knowledge Pages

Mental Models and Knowledge Pages are Hindsight-native synthesized resources.

Name them explicitly when the adapter should consult them:

```yaml
backends:
  hindsight:
    mental_models:
      - authentication architecture
    knowledge_pages:
      - Architecture/Authentication
```

Do not create or refresh either resource merely because it is referenced by MemHooks.

The generic MemHooks `resources` field may also name an existing resource when its provider type is not important to the core. Use the Hindsight namespace when the native resource class or retrieval behavior matters.

### Banks, documents, relationships, directives

Banks, documents, entity relationships, and directives remain Hindsight-native objects. MemHooks does not promote them into the core protocol.

If an adapter needs bank selection, use `backends.hindsight.bank`. Other Hindsight-native controls should remain under the same namespace.

## Recall versus Reflect

This decision is Hindsight-specific and therefore belongs in the Hindsight adapter.

A useful default:

```text
specific fact / event / prior decision
    -> Recall

existing Mental Model or Knowledge Page directly answers it
    -> consult that existing resource

multiple memories must be reconciled / synthesized
    -> Reflect
```

A query may request an adapter hint:

```yaml
backends:
  hindsight:
    strategy: reflect
```

The adapter should still respect runtime capabilities and cost/budget policy. If Reflect is unavailable, do not pretend it ran.

## Mapping generic MemHooks cues

### `recall_queries`

Use the query text as the primary Hindsight retrieval question. Preserve separate precise questions rather than collapsing them into one vague query.

### `priority`

Use MemHooks priority for context-budget decisions around competing recall requests. Do not confuse it with Hindsight relevance or confidence scores.

### `entities`

Use entity-aware retrieval when the active Hindsight interface exposes it; otherwise include important entity names in the query text so graph-aware retrieval can exploit them.

### `resources`

Generic resources may be read/retrieved when the runtime knows how to resolve them. Provider-specific resource classes stay under `backends.hindsight`.

### `tags`

Use native metadata/tag filtering if the active Hindsight interface supports an equivalent. Otherwise treat them as generic relevance hints.

### `exclude`

Use native negative filters when available; otherwise remove matched obsolete/misleading results before they enter working context.

## Provider namespace merge behavior

MemHooks itself performs only structural merging:

- nested mappings merge recursively;
- a local scalar replaces its parent value;
- a local list replaces its parent list;
- query-local `backends.hindsight` overlays the resolved scope-level Hindsight namespace.

The Hindsight adapter then interprets the final mapping.

## Hermes tool names

Tool names and exact parameters may vary by Hermes/Hindsight version. Use the actual connected tools rather than assuming names such as `hindsight_recall` or `hindsight_reflect` always exist.

## Sources

- Hindsight Memory Banks: https://hindsight.vectorize.io/developer/api/memory-banks
- Hindsight Retain: https://hindsight.vectorize.io/developer/retain
- Hindsight Recall API: https://hindsight.vectorize.io/developer/api/recall
- Hindsight Observations: https://hindsight.vectorize.io/developer/observations
- Hindsight Mental Models: https://hindsight.vectorize.io/developer/mental-models
- Hindsight Knowledge Pages: https://hindsight.vectorize.io/developer/knowledge-pages
