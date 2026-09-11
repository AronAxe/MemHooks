# OpenViking adapter mapping — memhooks/v2

Use this reference when the active memory/context backend is **OpenViking** or the runtime exposes OpenViking-compatible context tools.

OpenViking-specific routing belongs under:

```yaml
backends:
  openviking:
    ... adapter/OpenViking-native controls ...
```

The MemHooks core preserves the mapping but does not interpret it.

## Retrieval model

OpenViking organizes context through a filesystem-style `viking://` hierarchy and supports progressive/hierarchical retrieval.

Depending on the runtime/version, operations may resemble:

- semantic/context search;
- reading a selected URI/resource;
- browsing the hierarchy;
- memory write/delete operations.

MemHooks is retrieval-routing only. Loading a hook must not trigger remember/write/delete operations.

## Recommended adapter conventions

An OpenViking adapter may use keys such as:

```yaml
backends:
  openviking:
    root_uri: viking://project/backend
    strategy: search
```

These names are adapter conventions rather than universal MemHooks fields. If the actual OpenViking wrapper exposes different controls, the adapter should translate or use its real native interface.

Useful strategy hints may include:

- `search` — discover relevant context from a query;
- `read` — prefer a known resource/URI;
- `browse` — navigate hierarchy before selecting detail.

Do not treat these as guaranteed OpenViking API parameter names.

## Mapping generic MemHooks cues

### `recall_queries`

Use OpenViking's current search/context retrieval operation. Keep precise recall questions separate.

### `entities`

Use entity names as query/hierarchy cues where helpful.

### `resources`

Generic named resources map naturally to known `viking://` paths/resources when the runtime can resolve them. Read progressively rather than loading an entire subtree.

If an exact OpenViking URI is provider-specific, place it under `backends.openviking` rather than forcing URI semantics into the core `resources` shape.

### `tags`

Use native metadata filtering if exposed; otherwise incorporate tags as query/path hints.

### `exclude`

Avoid excluded URIs/subtrees when possible or post-filter candidates before reading them into context.

### `priority` and salience

Use MemHooks priority/entity/resource salience only for context-budget decisions. They are not OpenViking search scores.

## Progressive retrieval

A good default is:

```text
search/browse
    -> overview/abstract
        -> selected resource
            -> full detail only if needed
```

This preserves OpenViking's hierarchical advantage without turning MemHooks into a context dump.

## Provider namespace merging

MemHooks structurally merges `backends.openviking` root → leaf:

- nested mappings recursively merge;
- local scalars replace parent scalars;
- local lists replace parent lists;
- query-local OpenViking hints overlay scope-level hints.

The OpenViking adapter interprets the result.

## Runtime tool names

Use the actual OpenViking tools exposed by the current runtime rather than assuming exact names such as `viking_search`, `viking_read`, or `viking_browse` always exist.

## Sources

- OpenViking docs: https://docs.openviking.ai/
- OpenViking memory docs: https://docs.openviking.ai/en/api/16-memory
- Hermes memory providers: https://hermes-agent.nousresearch.com/docs/user-guide/features/memory-providers/
