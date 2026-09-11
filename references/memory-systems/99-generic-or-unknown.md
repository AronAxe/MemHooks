# Generic / unknown backend — memhooks/v2

Use this when the active memory backend does not have a dedicated MemHooks reference.

`memhooks/v2` is designed so an unknown backend can still use the **core retrieval intent** without pretending to understand another provider's ontology.

## Core rule

Translate these generic fields to the nearest real capability the backend exposes:

- `recall_queries`
- `priority`
- `when.roles`
- `entities`
- `resources`
- `tags`
- `exclude`
- `scope`
- `sensitivity`

Do not invent unsupported provider controls.

## Unknown provider namespaces

A hook may contain:

```yaml
backends:
  some-provider:
    custom_option: value
```

The MemHooks core preserves this mapping. If the active adapter does not recognize that provider namespace, it should ignore the provider-native mapping rather than reinterpret it as core semantics.

The core retrieval query still applies unless some other core condition excludes it.

## Step 1 — inspect actual capabilities

Identify real equivalents, if present, for:

- direct memory search/recall;
- metadata filtering;
- entity-aware or graph-aware search;
- time-aware search;
- session/user/agent/project scoping;
- existing named resources/pages/summaries;
- deeper synthesis/reasoning over remembered material;
- result limits, thresholds, reranking, or similar retrieval tuning.

Do not infer capabilities merely because another provider has them.

## Step 2 — translate core intent

### `recall_queries`

Use the backend's most direct search/recall primitive first.

### `priority`

Use priority for deciding which recall requests receive context budget. It is not a substitute for the provider's own relevance/confidence score.

### `entities`

Use native entity filters when they genuinely exist; otherwise include important entity names in the natural-language query. Preserve explicit type metadata without guessing a taxonomy.

### `resources`

If the runtime can resolve a named existing resource, retrieve/read it. Do not create a resource merely because it is named by a hook.

### `tags`

Use real metadata filters when available; otherwise use tags as query/routing hints.

### `exclude`

Use native negative filtering if available; otherwise post-filter obsolete/misleading results before they enter context.

### `sensitivity`

Treat as advisory metadata only. Host authorization/security policy remains authoritative.

## Step 3 — use provider namespaces only when understood

When an adapter knows its own namespace, it may interpret those native controls.

For example:

```yaml
backends:
  my_memory_system:
    result_limit: 8
    search_mode: hybrid
```

The generic MemHooks resolver will carry this mapping unchanged (subject only to structural inheritance merging). It does not validate whether `result_limit` or `search_mode` exists.

Provider-specific validation belongs in the provider adapter.

## Step 4 — keep retrieval bounded

Retrieve enough to satisfy the applicable cues, not the entire store. Use explicit MemHooks priority/entity/resource salience when context pressure forces choices, then let the backend's own ranking choose among results for a given retrieval request.

## Step 5 — preserve the retrieval-only boundary

Do not call memory write/update/delete operations just because a hook loaded.

If the backend exposes a `reason`, `reflect`, `synthesize`, or equivalent operation, use it only when the current task actually requires synthesis or conflict resolution.

## Adding a new provider mapping

If the mapping becomes reusable, add a dedicated file under `references/memory-systems/` and keep all provider-native vocabulary inside `backends.<provider>`.

Do **not** add provider-specific fields to the universal MemHooks schema merely because one integration benefits from them.

## No memory backend

Fail open. Continue the user's task without pretending retrieval occurred.
