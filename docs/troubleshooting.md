# Troubleshooting

This page covers the v0.5.x reference validator diagnostics and common `memhooks/v2` integration mistakes.

## Diagnostic codes

| Code | Severity | Meaning |
|---|---|---|
| `MH000` | error | file/frontmatter could not be parsed |
| `MH001` | error | missing/unsupported `schema`; expected `memhooks/v2` |
| `MH002` | warning | unknown top-level core field |
| `MH003` | error | empty recall query |
| `MH004` | error | query `priority` is non-finite or outside `0.0..1.0` |
| `MH005` | warning | unknown structured recall-query core field |
| `MH006` | error | empty role name |
| `MH007` | warning | unknown routing condition under `when` |
| `MH008` | error | empty entity name |
| `MH009` | error | entity `salience` is non-finite or outside `0.0..1.0` |
| `MH010` | warning | duplicate recall query text in one hook |
| `MH011` | warning | duplicate top-level entity name in one hook |
| `MH012` | warning | same text is both recalled and excluded |
| `MH013` | warning | no `MEMHOOKS.md` files found in requested validation scope |
| `MH014` | warning | unknown structured entity field |
| `MH015` | warning | duplicate top-level resource name |
| `MH016` | error | provider-specific v1-style field placed in v2 top-level core |
| `MH017` | error | provider-specific v1-style field placed directly on a v2 query |
| `MH018` | error | empty backend namespace name |
| `MH019` | error | backend namespace value is not a mapping/object |
| `MH020` | error | empty resource name |
| `MH021` | error | resource `salience` is non-finite or outside `0.0..1.0` |
| `MH022` | warning | unknown structured resource field |

Warnings do not make the CLI fail; errors do.

## `memhooks validate --all` says no hooks were found

Make sure you are inside the intended repository or pass the project path explicitly:

```bash
memhooks validate /path/to/project --all
```

The repository-wide scanner respects Git ignore rules.

## `explain` says `memhooks/v1` is unsupported

v0.5.x resolves `memhooks/v2`. The resolver fails unsupported schemas deliberately instead of silently ignoring provider-specific fields.

The intended v2 form keeps provider-native fields under a provider namespace, for example:

```yaml
schema: memhooks/v2
backends:
  hindsight:
    memory_types: [experience]
```

There is no generic top-level `memory_types` in v2.

## Validator says a provider-specific field is not part of the core

Fields such as Hindsight `bank`, `memory_types`, `connection_types`, `mental_models`, or `knowledge_pages` do not belong at the v2 top level.

Use the appropriate namespace:

```yaml
backends:
  hindsight:
    bank: project-memory
    memory_types: [experience]
```

The same rule applies to provider-native fields on a structured query.

## Backend namespace must contain a mapping/object

Invalid:

```yaml
backends:
  mem0: fast
```

Valid:

```yaml
backends:
  mem0:
    top_k: 8
```

The generic validator does not validate keys *inside* that mapping; provider adapters may do so.

## `explain` shows a role-restricted query I expected to disappear

If you run:

```bash
memhooks explain backend/auth
```

without `--role`, the resolver preserves restricted queries. Unknown role is not the same as non-matching role.

Supply a known role:

```bash
memhooks explain backend/auth --role reviewer
```

## A parent hook disappeared

Check for:

```yaml
inherits: false
```

in a leafward hook. It intentionally cuts off ancestors above that file for the local subtree.

## A provider list replaced the parent's list

That is deliberate v2 behavior.

Provider namespaces are opaque. MemHooks cannot know whether a provider-owned list is additive, ordered, exclusive, or something else, so structural merging uses:

- mapping + mapping → recursive merge;
- local scalar/list/other value → replace parent value.

If an adapter needs additive provider semantics, express that explicitly inside the provider's own configuration model.

## Query-local provider settings did not erase scope filters

Also deliberate.

Example scope:

```yaml
backends:
  mem0:
    filters:
      user_id: alice
    threshold: 0.1
```

query overlay:

```yaml
backends:
  mem0:
    top_k: 5
```

The effective query structurally retains `filters` and `threshold` while adding `top_k`.

## `priority` and backend relevance disagree

They answer different questions:

- backend relevance/similarity → how well a returned memory matches a search;
- MemHooks priority → how costly it is to omit this **recall request** under context pressure.

Do not automatically overwrite one with the other.

## Entity/resource salience and query priority disagree

Valid. Salience describes importance of a cue; priority describes importance of the recall request.

## The configured backend is Mem0 but the hook also has Hindsight hints

Do not call both systems merely because both namespaces exist.

Backend selection belongs to runtime configuration/authorization. Interpret only the namespace for the active backend. Generic core intent still applies regardless of namespace presence.

## Mem0 search returns a scope/filter error

Current Mem0 v3-style search expects scope identifiers such as `user_id`, `agent_id`, `app_id`, or `run_id` inside `filters` for search/get operations.

Keep that configuration under `backends.mem0` and use the exact shape supported by the connected Mem0 Platform/OSS version. See [`references/memory-systems/04-mem0.md`](../references/memory-systems/04-mem0.md).

## The memory backend has no equivalent for a provider hint

Do not invent a fake API parameter.

- keep the core recall query;
- use the nearest safe native retrieval mechanism;
- ignore an unsupported provider-native hint when necessary;
- optionally log/provider-validate the unsupported hint;
- never claim the control was applied when it was not.

See [`references/memory-systems/99-generic-or-unknown.md`](../references/memory-systems/99-generic-or-unknown.md).

## Rust `cargo publish --dry-run` reports a dirty `Cargo.lock`

The repository intentionally does not publish `Cargo.lock`. Build/test commands may generate it locally:

```bash
rm -f Cargo.lock
cargo publish --dry-run
```

CI performs this cleanup automatically.

## crates.io authentication fails

Repository publishing uses the GitHub Actions secret:

```text
CARGO_REGISTRY_TOKEN
```

A fallback `CRATES_IO_TOKEN` may be used by release automation when configured. Never commit tokens to repository files/logs.

See [Publishing](publishing.md).

## A repository tries to declare system-level prompt injection

That is outside the protocol. `MEMHOOKS.md` cannot grant itself prompt privilege. Prompt placement/trust remains a host-runtime decision.

## The hook file is getting bloated

This is an **agent/runtime maintenance issue**, not a task the end user should have to solve manually.

The maintainer should prune stale/redundant routing cues and keep the file functioning as an index rather than storing the memories themselves.
