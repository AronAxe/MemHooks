# Hindsight mapping

Use this mapping when the active memory backend is **Hindsight** (Vectorize) or Hermes exposes Hindsight memory tools.

Hindsight's core operations are:

- **Recall** — retrieves ranked memories using semantic, keyword/BM25, graph, and temporal strategies.
- **Reflect** — reasons over memories and synthesized knowledge; slower and more expensive than direct recall.
- **Retain** — writes memory. **MemHooks does not call this merely because a hook was loaded.**

## Hindsight's ontology

Keep these layers separate:

### Memory categories

Hindsight Recall accepts exactly:

- `world` — objective facts about the outside world;
- `experience` — events, conversations, and the bank agent's own experiences;
- `observation` — deduplicated, evidence-grounded beliefs consolidated from multiple memories.

A key detail from Hindsight's public Recall API: **each selected memory type runs the full four-strategy retrieval pipeline independently**. `world`, `experience`, and `observation` are therefore parallel memory categories, not containers for different retrieval/link mechanisms.

### Knowledge connections

Hindsight documents four kinds of connections between memories:

- **entity** connections — facts linked through shared entities;
- **temporal** connections — time-based proximity/order;
- **semantic** connections — meaning-based similarity;
- **causal** connections — cause/effect structure.

These are not memory categories and are not entity types.

The public Recall API itself runs four retrieval strategies in parallel: semantic similarity, keyword/BM25, graph traversal, and temporal retrieval. Its graph traversal uses entity/temporal/causal structure. Do not pretend a public `connection_types=` filter exists when it does not.

### Entities

Hindsight entities are named things extracted from or attached to memories. The public docs describe people, organizations, places, products, and concepts. Explicit retain entities use the shape `{text, type?}`; examples include `PERSON`, `ORG`, and `CONCEPT`, and an omitted type defaults to `CONCEPT`.

The entity `type` field is a string rather than a documented closed enum. MemHooks should therefore preserve an explicit backend/user-provided type when one exists, but must **not guess or invent a universal type taxonomy**. If the type is unclear, keep the entity untyped.

A proposition such as a decision, constraint, requirement, failure, or conclusion is not automatically an entity. It normally remains information *about* entities unless Hindsight/the user has explicitly modeled it as one.

Entity-to-entity relationships are a separate part of the Hindsight bank and are not defined by MemHooks v1.

### Observations

Observations are produced automatically by consolidation after raw facts are retained. They are evidence-backed beliefs, each grounded in supporting memories and refined as evidence changes. `observation` is a valid Recall `types` value even though it is not directly written as a raw retained fact.

### Mental models and Knowledge Pages

A mental model is a deliberately curated, stored standing answer to a question about a bank. Hindsight's Reflect retrieval ladder is:

1. mental models;
2. observations;
3. raw facts.

Knowledge Pages use the mental-model layer to expose living documents. They are higher-level synthesized retrieval targets, not another value of Recall `types` and not entities.

### Documents, relationships, directives

A Hindsight bank also contains documents, relationships, and directives. These remain backend-native bank data:

- documents are indexed source content;
- relationships connect entities in the knowledge graph;
- directives are hard rules applied during Reflect.

MemHooks should not relabel any of these as a memory type or entity type.

## Field mapping

### `memory_types`

Map MemHooks `memory_types` directly to Hindsight Recall/Reflect `types` when the active tool exposes that parameter.

Valid Hindsight values:

```text
world | experience | observation
```

If a structured MemHooks query has query-local `memory_types`, use those. Otherwise use the merged scope-wide default. If neither is present, omit the filter so all relevant Hindsight memory categories can be searched.

Perspective still matters when interpreting `world` versus `experience`: a user's first-person statement is ordinarily a world fact about the user; the bank agent's own action is an experience.

### `connection_types`

MemHooks may store any subset of:

```text
semantic | temporal | entity | causal
```

These values are routing emphasis, not a fake Hindsight API parameter.

Use them as follows:

- preserve the natural-language wording that makes the requested connection explicit;
- use a deeper Recall budget when indirect graph structure is important;
- use temporal expressions/ranges when `temporal` is important;
- name relevant entities explicitly when `entity` is important;
- phrase cause/effect questions clearly when `causal` is important;
- rely on Hindsight's normal semantic retrieval when `semantic` is important.

If a future Hindsight wrapper exposes an actual native control for one of these dimensions, use it. Otherwise do not invent one.

### `recall_queries`

Run each relevant query through Hindsight Recall first. Do not collapse several precise questions into one vague query.

Example:

```yaml
- query: "Why did the authentication design change after the outage?"
  memory_types: [experience]
  connection_types: [causal, temporal]
  entities:
    - Authentication
```

This should map to a Hindsight recall with the query text and `types=["experience"]`. The `causal` and `temporal` values sharpen how the query is posed; they are not passed as a nonexistent filter.

### `entities`

MemHooks supports either an untyped string or:

```yaml
- name: OpenAI
  type: ORG
```

For Hindsight's explicit entity shape, map `name` → `text` and preserve `type` when it is known.

Do not infer a type merely because a name looks like a person, project, place, or product. If the hook only contains a string, treat it as untyped routing metadata.

For retrieval, use entity-aware constraints only when the active Hindsight tool actually exposes them. Otherwise include the relevant entity names in the natural-language query so Hindsight's graph retrieval can use them.

### `mental_models`

If the hook names existing Hindsight mental models, fetch/read them before ordinary Recall when they directly answer the active question. A mental model is already-written synthesized knowledge, so using it can avoid redundant retrieval and synthesis.

Do not create or refresh a mental model merely because MemHooks mentions it.

### `knowledge_pages`

If the hook names existing Hindsight Knowledge Pages, retrieve them as stable synthesized context before descending to raw Recall when appropriate.

Do not create or update a Knowledge Page merely because a hook was loaded.

### `tags`

Use Hindsight tag filters where available. Otherwise treat tags as relevance/scoping hints.

### `exclude`

Use negative/tag filtering where exposed; otherwise post-filter obsolete or explicitly excluded memories before they enter working context.

## Recall vs reflect

```text
specific fact / event / decision
    -> recall

existing mental model/page directly answers it
    -> fetch that synthesized resource

multiple memories must be reconciled
or the hook asks for synthesis
    -> reflect
```

Use `reflect` deliberately. Routine folder entry should not trigger expensive reflection when direct recall or an existing mental model already supplies the context.

## Hermes-native names

Depending on version, tools may be exposed as names similar to `hindsight_recall`, `hindsight_reflect`, and `hindsight_retain`. Use the actual tools present rather than assuming exact names or parameter names.

## Sources

- Hindsight Memory Banks: https://hindsight.vectorize.io/developer/api/memory-banks
- Hindsight Retain: https://hindsight.vectorize.io/developer/retain
- Hindsight Recall API: https://hindsight.vectorize.io/developer/api/recall
- Hindsight Observations: https://hindsight.vectorize.io/developer/observations
- Hindsight Mental Models: https://hindsight.vectorize.io/developer/mental-models
- Hindsight Knowledge Pages: https://hindsight.vectorize.io/developer/knowledge-pages
