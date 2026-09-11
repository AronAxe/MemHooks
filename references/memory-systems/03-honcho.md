# Honcho adapter mapping — memhooks/v2

Use this reference when the active memory backend is **Honcho** or the runtime exposes Honcho-compatible memory-provider tools.

Honcho-specific controls belong under:

```yaml
backends:
  honcho:
    ... Honcho/adapter-native controls ...
```

The MemHooks core preserves the namespace without interpreting it.

## Retrieval model

Depending on the integration/version, Honcho may expose operations analogous to:

- semantic memory search;
- current session/peer context retrieval;
- synthesized or dialectic reasoning over remembered context;
- profile/conclusion write operations.

MemHooks is retrieval-routing only. It should use read/search/reasoning capabilities when needed, not create conclusions or update profiles merely because a hook loaded.

## Recommended adapter conventions

A Honcho-aware adapter may use a provider namespace such as:

```yaml
backends:
  honcho:
    strategy: search
    peer: project-agent
    session: current-project
```

These are adapter conventions, not universal MemHooks fields and not guaranteed raw Honcho API parameter names. Use the actual controls exposed by the installed Honcho integration.

Possible strategy hints:

- `search` — direct retrieval for concrete prior context;
- `context` — use an existing session/peer representation when it already contains what is needed;
- `reasoning` — use Honcho's synthesis/dialectic capability when multiple memories must be reconciled.

## Example

```yaml
schema: memhooks/v2

recall_queries:
  - query: "Why was the retry policy changed?"
    priority: 0.9
    entities: [RetryPolicy]
    backends:
      honcho:
        strategy: reasoning

backends:
  honcho:
    peer: project-agent
```

The query, priority, and entity are core MemHooks intent. `strategy` and `peer` are provider-specific.

## Mapping generic MemHooks cues

### `recall_queries`

Use direct Honcho search for concrete prior decisions, events, excerpts, or conclusions. Use reasoning only when synthesis materially helps.

### `entities`

Include important names/components explicitly in search or reasoning. Use native metadata/entity filters only when the actual Honcho interface supports them.

### `resources`

Map generic resources to the nearest existing Honcho representation the runtime can actually retrieve: session context, peer representation, an existing conclusion, or another known resource. Retrieval does not imply creating or updating one.

### `tags`

Use native filters if exposed; otherwise treat tags as query hints.

### `exclude`

Filter obsolete results before use and distinguish current conclusions from rejected/old approaches.

### `priority` and salience

Use them to choose which recall tasks/cues survive context pressure. Do not confuse them with Honcho-native relevance/confidence.

## Search versus reasoning

A reasonable adapter default:

```text
"what happened / what did we decide / find context"
    -> search

"give me the mapped current context"
    -> context

"why / reconcile / synthesize remembered evidence"
    -> reasoning
```

Provider strategy hints may override this when the actual tool supports the requested operation.

## Provider namespace merging

MemHooks performs structural merging only:

- nested mappings recursively merge;
- local scalar values replace parent values;
- local lists replace parent lists;
- query-local `backends.honcho` overlays scope-level Honcho configuration.

The Honcho adapter interprets the final mapping.

## Sources

- Honcho docs: https://docs.honcho.dev/
- Hermes Honcho integration: https://hermes-agent.nousresearch.com/docs/user-guide/features/honcho/
- Honcho GitHub: https://github.com/plastic-labs/honcho
