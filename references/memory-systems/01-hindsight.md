# Hindsight mapping

Use this mapping when the active memory backend is **Hindsight** (Vectorize) or Hermes exposes Hindsight memory tools.

Hindsight's core operations are:

- **Recall** — retrieves ranked memories using semantic, keyword/BM25, graph/entity, and temporal strategies.
- **Reflect** — reasons over memories and synthesizes an answer; slower and more expensive than direct recall.
- **Retain** — writes memory. **MemHooks does not call this merely because a hook was loaded.**

For recall, Hindsight accepts the fact/source types `world`, `experience`, and `observation`. Direct `retain()` extraction produces `world` and `experience` facts; `observation` is consolidated knowledge produced after retention. Hindsight also has entities, relationships, labels, observations, and mental models.

## Field mapping

### `memory_types`
Map MemHooks `memory_types` directly to Hindsight recall/reflect `fact_types` when the active Hindsight tool exposes that filter.

- `world` — facts about users, other people, places, projects, systems, things, and outside events.
- `experience` — the memory bank agent's **own** first-person actions, observations, and interactions.
- `observation` — consolidated/synthesized knowledge produced from retained evidence.

Perspective matters. A user saying "I changed the auth flow" is normally a `world` fact about that user, not an `experience` of the memory-bank agent. The agent saying "I changed the auth flow" about its own action is an `experience`.

Do not try to retain a new item directly as an `observation`; Hindsight creates observations through consolidation. `observation` is nevertheless a valid retrieval type.

If a MemHooks query has query-local `memory_types`, use those for that query. Otherwise use the merged scope-wide `memory_types`. If neither is present, do not invent a restrictive filter.

For older hooks with no stored type, infer one only when clear:

- external/project/user facts → `world`
- the bank agent's own past interactions/incidents/actions → `experience`
- consolidated settled knowledge/patterns → `observation`

### `recall_queries`
Run each relevant query through Hindsight `recall` / Hermes `hindsight_recall` first. Use natural-language questions. Do not collapse several precise questions into one vague query.

A structured MemHooks query such as:

```yaml
- query: "Why was the current token-refresh architecture selected?"
  memory_types: [experience]
  entities:
    - name: Authentication
      type: COMPONENT
```

should become the closest native Hindsight recall call: the query text plus `fact_types=["experience"]` and entity-aware sharpening/filtering if the active tool exposes it.

### `entities`
Hindsight `retain()` accepts explicit entities shaped as `{text, type?}`. MemHooks uses the backend-neutral shape `{name, type?}`; map `name` → Hindsight `text` and preserve `type`.

Hindsight documentation gives `PERSON`, `ORG`, and `CONCEPT` as examples, and its automatic recognizer also describes people, organizations, places, products, and concepts. The explicit Hindsight entity `type` field is a string rather than a closed enum, so more precise project/domain types can be supplied when useful.

MemHooks therefore recommends a stable extensible vocabulary including:

`PERSON`, `ORG`, `PLACE`, `PROJECT`, `PRODUCT`, `SOFTWARE`, `SERVICE`, `COMPONENT`, `API`, `REPOSITORY`, `FILE`, `DOCUMENT`, `DATASET`, `MODEL`, `TECHNOLOGY`, `EVENT`, `CONCEPT`, `DECISION`, `REQUIREMENT`, `CONSTRAINT`, and `ISSUE`.

Example mapping:

```yaml
# MemHooks
entities:
  - name: MemHooks
    type: PROJECT
  - name: memhooks_update.py
    type: FILE
```

```json
// Hindsight-style entity inputs when retaining/ensuring explicit entities
[
  {"text": "MemHooks", "type": "PROJECT"},
  {"text": "memhooks_update.py", "type": "FILE"}
]
```

For **retrieval**, use entity constraints when the exposed Hindsight tool supports them. Otherwise mention the typed entity names explicitly in the natural-language query so graph retrieval can help. A type should sharpen interpretation, not cause a useful recall to be skipped just because the wrapper lacks a type-filter parameter.

Legacy string entities remain valid; treat them as untyped. Do not silently relabel an existing entity with a guessed type unless the classification is clear.

### Entity relationship/edge types
MemHooks currently records node/entity identity and type only. Hindsight graph **relationship/edge types are a separate backend concern** and are intentionally not defined by the MemHooks v1 routing schema.

### `tags`
Use tag/metadata filters when available. Otherwise incorporate meaningful tag concepts into the query.

### `knowledge_pages`
If the local integration has curated Knowledge Pages or an equivalent mental-model layer, retrieve them as stable context first. Do not create or update them here.

### `exclude`
Use filtering metadata where possible. Otherwise remove excluded/obsolete results before they enter working context and sharpen queries to distinguish current from obsolete approaches.

## Recall vs reflect

```text
specific fact / event / decision
    -> recall

multiple memories must be reconciled
or the hook asks for synthesis
    -> reflect
```

Use `reflect` deliberately. Routine folder entry should not trigger expensive reflection when direct recall already supplies the context.

## Hermes-native names

Depending on version, tools may be exposed as names similar to `hindsight_recall`, `hindsight_reflect`, and `hindsight_retain`. Use the actual tools present rather than assuming exact names or parameter names.

## Sources

- Hindsight developer docs: https://github.com/vectorize-io/hindsight/tree/main/skills/hindsight-docs
- Hindsight retain guide: https://github.com/vectorize-io/hindsight/blob/main/skills/hindsight-docs/references/developer/retain.md
- Hindsight retain API: https://github.com/vectorize-io/hindsight/blob/main/skills/hindsight-docs/references/developer/api/retain.md
- Hindsight docs: https://docs.hindsight.vectorize.io/
- Hermes native Hindsight integration: https://github.com/vectorize-io/hindsight
