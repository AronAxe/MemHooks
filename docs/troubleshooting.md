# Troubleshooting

This page covers the reference validator's diagnostics and the most common integration mistakes.

## Diagnostic codes

| Code | Severity | Meaning |
|---|---|---|
| `MH000` | error | file/frontmatter could not be parsed |
| `MH001` | error | missing or unsupported `schema`; expected `memhooks/v1` |
| `MH002` | warning | unknown top-level field |
| `MH003` | error | empty recall query |
| `MH004` | error | query `priority` is not finite or outside `0.0..1.0` |
| `MH005` | warning | unknown structured recall-query field |
| `MH006` | error | empty role name |
| `MH007` | warning | unknown routing condition under `when` |
| `MH008` | error | empty entity name |
| `MH009` | error | entity `salience` is not finite or outside `0.0..1.0` |
| `MH010` | warning | duplicate recall query text in one hook |
| `MH011` | warning | duplicate top-level entity name in one hook |
| `MH012` | warning | the same text is both recalled and excluded |
| `MH013` | warning | no `MEMHOOKS.md` files were found in the requested validation scope |
| `MH014` | warning | unknown structured entity field |
| `MH015` | error | invalid value in `memory_types` or `connection_types` |

Warnings do not make the CLI return failure; errors do.

## `memhooks validate --all` says no hooks were found

Make sure you are inside the intended repository or pass the project path explicitly:

```bash
memhooks validate /path/to/project --all
```

The repository-wide scanner respects Git ignore rules.

## `explain` shows a query I expected to be filtered out

If you run:

```bash
memhooks explain backend/auth
```

without a `--role`, the reference resolver deliberately preserves role-restricted queries. Unknown role must not be treated as a non-match.

Supply the known runtime role:

```bash
memhooks explain backend/auth --role reviewer
```

## A parent hook disappeared

Check for:

```yaml
inherits: false
```

in a leafward `MEMHOOKS.md`. That setting intentionally cuts off ancestors above that file for the local subtree.

## Query-local memory or connection types seem to replace parent defaults

That is expected. Scope-wide `memory_types` and `connection_types` act as defaults. A structured query's own non-empty list takes priority for that query.

Query-local entities behave differently: they **supplement** merged top-level entities rather than replacing them.

## `priority` and backend relevance disagree

They answer different questions:

- backend relevance/similarity: "How well does this returned memory match the search?"
- MemHooks priority: "How costly is it to omit this recall request when context is constrained?"

Do not automatically overwrite one with the other. A host may combine them, but the protocol does not prescribe one scoring formula.

## An entity has high salience but comes from a low-priority query

Also valid. Entity salience and query priority are independent dimensions. Salience says the entity is an important cue; priority says the retrieval request itself is important under budget pressure.

## Rust `cargo publish --dry-run` reports a dirty `Cargo.lock`

The repository intentionally does not publish `Cargo.lock`. Rust build/test commands can generate it locally. Remove the generated lock file before a publish check:

```bash
rm -f Cargo.lock
cargo publish --dry-run
```

The CI workflow does this automatically.

## crates.io publication fails before uploading

For the first crates.io publication, the owner must configure an API token. The repository publisher expects one of these GitHub Actions repository secrets:

```text
CARGO_REGISTRY_TOKEN
CRATES_IO_TOKEN
```

Prefer `CARGO_REGISTRY_TOKEN`.

Do not commit or paste the token into repository files, issues, pull requests, or chat logs.

See [Publishing](publishing.md).

## The memory backend has no equivalent for a field

Do not invent a fake API parameter. Preserve the intent using the nearest safe mechanism:

- enrich the natural-language query;
- use supported metadata/entity filters;
- post-filter results;
- or ignore a nonessential hint while preserving the core recall request.

See `references/memory-systems/99-generic-or-unknown.md`.

## A repository tries to declare system-level prompt injection

That is outside the current protocol. MemHooks routing metadata must not grant a repository its own prompt privilege. Prompt placement/trust remains a host-runtime decision.
