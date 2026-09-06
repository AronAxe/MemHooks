# Generic / unknown memory backend

MemHooks must not become an adapter framework. If the current backend is not listed, **infer the mapping from the tools or documentation already available to you**.

Use Hindsight, OpenViking, and Honcho as examples of the pattern, not as required dependencies.

## Step 1 — inspect capabilities

Identify the backend's native equivalents, if present, for:

- direct memory search / recall;
- memory/fact categories or source types;
- deeper synthesis / reasoning over memory;
- semantic, temporal, entity, causal, graph, or relationship-aware retrieval;
- entity-aware lookup and explicit entity typing;
- tags / metadata filters;
- temporal filters;
- hierarchical pages, summaries, mental models, or curated answers;
- namespace / bank / peer / session selection.

Do not invent capabilities the backend does not expose.

## Step 2 — map MemHooks fields

- `recall_queries`: map to the backend's most direct retrieval/search primitive first.
- `memory_types`: use native memory/fact-category filters when an actual equivalent exists; otherwise treat the values as query context rather than fabricating a filter.
- `connection_types`: treat semantic/temporal/entity/causal values as retrieval emphasis. Use a native relationship/strategy control only if one exists.
- `entities`: use native entity filtering when available; otherwise put names into the natural-language query. Preserve `{name, type}` only when the backend can use the type or when it remains useful descriptive metadata; do not guess an entity type.
- `tags`: use metadata filters when available; otherwise use as query hints.
- `mental_models`: map to an existing precomputed standing-answer/curated-model construct if one exists. Ignore if there is no equivalent.
- `knowledge_pages`: map to an existing summary/page/mental-model-like construct if one exists. Do not manufacture one.
- `exclude`: use negative filtering if supported; otherwise post-filter results before placing them in working context.

If the backend has a `reflect`, `reason`, `synthesize`, `ask memory`, or similar operation, reserve it for hooks requiring synthesis or conflict resolution rather than routine lookup.

## Step 3 — preserve distinctions

Do not collapse independent dimensions merely because the backend names them differently.

For example, a memory category, a graph/relationship mechanism, and an entity type are conceptually separate things. Translate each to the closest real backend capability and leave unsupported dimensions as descriptive routing hints.

## Step 4 — execute, don't overbuild

Do not stop the user's task to write integration code. A competent LLM can usually translate a question like `What did we decide about X?` into whatever search primitive the current memory system exposes.

If the mapping proves reusable **and** you have permission to edit this skill, add a concise new reference file under `references/memory-systems/` modeled on the existing three. Keep it descriptive rather than executable unless the backend genuinely requires code.

## If there is no memory backend

Fail open. Continue the task without claiming recall occurred.
