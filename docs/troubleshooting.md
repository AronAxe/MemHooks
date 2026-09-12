# Troubleshooting

This page covers the v0.5.x reference validator diagnostics and common `memhooks/v2` integration mistakes.

## Diagnostic codes

| Code | Severity | Meaning |
|---|---|---|
| `MH000` | error | file/frontmatter could not be parsed |
| `MH001` | error | missing/unsupported `schema`; expected `memhooks/v2` |
| `MH002` | warning | unknown top-level core field |
| `MH003` | error | structured recall query has no usable `query` string |
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
| `MH016` | error | provider-specific field placed in the v2 top-level core |
| `MH017` | error | provider-specific field placed directly on a v2 query |
| `MH018` | error | empty backend namespace name |
| `MH019` | error | backend namespace value is not a mapping/object |
| `MH020` | error | empty resource name |
| `MH021` | error | resource `salience` is non-finite or outside `0.0..1.0` |
| `MH022` | warning | unknown structured resource field |
| `MH024` | error | requested target path does not exist |
| `MH025` | error | malformed recall-query entry or known query field has wrong YAML type |
| `MH026` | error | malformed entity entry or known entity field has wrong YAML type |
| `MH027` | error | malformed resource entry or known resource field has wrong YAML type |

Warnings do not make the CLI fail; errors do.

## A diagnostic points at the wrong repeated field

v0.5.1 indexes source positions from the YAML frontmatter AST. For example, if the second of two `priority:` values is invalid, MH004 points at that second value rather than the first textual `priority` occurrence or a similarly named key in Markdown prose.

If you can reproduce an incorrect location on v0.5.1+, report it as a validator bug because SARIF depends on those positions being trustworthy.

## `quer:` or another typo kills the whole file

It should not on v0.5.1+.

Malformed structured query/entity/resource entries are parsed tolerantly enough for the linter to emit targeted diagnostics. For example:

```yaml
recall_queries:
  - quer: "typo"
  - query: "the rest of the file still lints"
```

produces a missing-query/unknown-field diagnostic rather than only MH000 with an internal Rust enum error.

Truly invalid YAML can still produce MH000 because there is no safe structure to continue linting.

## A nonexistent path validates successfully

It should not on v0.5.1+:

```bash
memhooks validate ./does/not/exist
memhooks explain ./does/not/exist
```

Both treat the typo as an error (`MH024` for validation).

## `memhooks validate --all` says no hooks were found

Make sure you are inside the intended repository or pass the project path explicitly:

```bash
memhooks validate /path/to/project --all
```

The repository-wide scanner respects Git ignore rules.

## A hook above my Git repository was used or modified

That is a bug on v0.5.1+.

Canonical root semantics are:

1. explicit containing `MEMHOOKS_ROOT` when configured;
2. otherwise nearest Git root as a hard boundary;
3. outside Git only, highest ancestor containing `MEMHOOKS.md`.

A `~/MEMHOOKS.md` must not capture writes for an uninitialized Git repository below it.

## `explain` says `memhooks/v1` is unsupported

v0.5.x resolves `memhooks/v2`. Unsupported schemas fail deliberately instead of being silently reinterpreted.

Provider-native fields belong under provider namespaces, for example:

```yaml
schema: memhooks/v2
backends:
  hindsight:
    memory_types: [experience]
```

## A parent hook disappeared

Check for top-level:

```yaml
inherits: false
```

in a leafward hook. It intentionally cuts off ancestors above that file.

An `inherits:` key nested inside `backends.<provider>` is opaque provider data and must **not** affect core inheritance. The bundled Hermes adapter delegates parsing to the Rust resolver so both behave identically.

## The same query appears in parent and child hooks

In v0.5.1, trimmed query text is the query identity across the inheritance chain. A more-local declaration replaces the parent's query-local metadata.

So:

```yaml
# parent
- query: "same question"
  priority: 0.2
```

and:

```yaml
# child
- query: "same question"
  priority: 0.9
```

resolve to **one** query at priority `0.9`, sourced from the child.

## Provider namespace merge surprises

Provider namespaces are opaque. Structural merging uses:

- mapping + mapping → recursive merge;
- local scalar/list/other value → replace parent value;
- query-local provider mapping → overlay resolved scope provider mapping.

MemHooks does not concatenate provider lists because it cannot know provider semantics.

## `priority` and backend relevance disagree

They answer different questions:

- backend relevance/similarity → how well a returned memory matches a search;
- MemHooks priority → how costly it is to omit this recall request under context pressure.

Do not automatically overwrite one with the other.

## The maintainer visibly wrote a cue but `explain` cannot see it

This was a real pre-v0.5.1 bug. The old Python maintainer stored structured notes/anchors in Markdown-body blocks while the Rust resolver read YAML frontmatter.

v0.5.1 removes the second persistence model. `memhooks note` and `memhooks event` write normative YAML frontmatter through the same Rust model the resolver reads.

If a cue written by the v0.5.1 reference maintainer is absent from `memhooks explain`, report it as a regression.

## Automatic anchors contain imports, method calls, or mangled URLs

They should not on v0.5.1+.

`memhooks event` only considers values under explicit path-bearing tool-input keys and requires each candidate to canonicalize to an existing file within the project root. It does not regex arbitrary file contents.

## Concurrent tool calls lose hook updates or leave a truncated hook

The v0.5.1 reference maintainer takes an exclusive sibling lock and writes using same-directory atomic replacement. `.MEMHOOKS.md.lock` is intentionally ignored by Git.

If a third-party adapter bypasses the reference maintainer, it is responsible for equivalent concurrency/durability behavior.

## The Hermes loader accepted invalid/v1 hooks

The v0.5.1 bundled loader no longer parses raw hook files. It delegates to:

```bash
memhooks explain <cwd> --format json
```

so schema enforcement/root/inheritance come from the same reference resolver. A failed resolver produces no injected MemHooks context.

## Repository text contains an old `--- END ... ---` delimiter

That text is inert data in v0.5.1. The bundled Hermes adapter no longer wraps raw repository files in static BEGIN/END fences. It injects one structurally valid JSON plan explicitly labelled as untrusted repository-controlled retrieval metadata.

## The loader exceeds its context budget

Truncation removes whole guidance/query/resource/entity items and always emits closed JSON. It does not cut a file, YAML frontmatter, or delimiter block in half.

## `cargo test` versus CI

CI explicitly runs both normal Rust targets **and doctests**:

```bash
cargo test --locked --all-targets --all-features
cargo test --locked --doc
```

The crate-level documentation examples are written as valid doctests; filesystem-dependent examples use `no_run` where appropriate.

## `Cargo.lock`

v0.5.1 commits `Cargo.lock` because MemHooks ships a binary as well as a library. CI uses `--locked`, and the lockfile is included in the package manifest for reproducible reference builds.

Do not delete the lockfile before normal CI/package checks.

## Declared Rust version

`Cargo.toml` declares Rust 1.78. CI has a dedicated Rust 1.78/Cargo 1.78 job so dependency changes cannot silently raise the practical MSRV.

## YAML implementation

The old archived/deprecated `serde_yaml` dependency was removed in v0.5.1. The reference implementation uses `serde_yaml_ng`; Saphyr supplies marked YAML nodes for diagnostic source locations.

## crates.io authentication fails

Repository publishing uses the GitHub Actions secret:

```text
CARGO_REGISTRY_TOKEN
```

Never commit tokens to repository files or logs. See [Publishing](publishing.md).

## A repository tries to declare system-level prompt injection

That is outside the protocol. `MEMHOOKS.md` cannot grant itself prompt privilege. The host runtime remains authoritative.

See [`SECURITY.md`](../SECURITY.md) for the explicit trust boundary and reporting process.

## The hook file is getting bloated

This is an **agent/runtime maintenance issue**, not a task the end user should have to solve manually.

The runtime should prune stale/redundant routing cues and keep the file functioning as an index rather than storing the memories themselves.