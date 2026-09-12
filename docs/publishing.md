# Publishing the Rust crate

This is maintainer documentation for publishing `memhooks` to crates.io.

## Release baseline

MemHooks is published on crates.io and the repository has an authenticated publishing secret configured. Every release must be validated from the **committed lockfile** and the exact public registry version must be installed back after publication.

v0.5.1 keeps protocol schema `memhooks/v2`; it is an implementation hardening release.

## Before every release

Run:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo test --locked --doc
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
cargo run --locked --quiet -- validate --all
python -m compileall -q hooks scripts tests
ruff check hooks scripts tests
pytest -q
cargo publish --dry-run --locked
```

CI also checks the declared Rust 1.78 MSRV and Python 3.10/3.12 compatibility.

Do **not** remove `Cargo.lock` as part of the release flow. MemHooks ships a binary as well as a library, so the lockfile is committed and CI uses `--locked`.

### Rust 1.78 dependency floor

The `rust-version = "1.78"` declaration is tested, not aspirational. Some dependency families later adopted Edition 2024 or raised their MSRV while remaining semver-compatible with broad dependency ranges, so v0.5.1 constrains the affected families in `Cargo.toml` as well as committing `Cargo.lock`.

Current compatibility pins include the Clap, ignore/globset, indexmap, Saphyr, ordered-float, and tempfile lines used by the reference CLI/parser. Do not casually widen those ranges: first prove the replacement graph with the Rust 1.78 CI job, then update the manifest and lockfile together.

## Authentication

Publishing automation uses the GitHub Actions repository secret:

```text
CARGO_REGISTRY_TOKEN
```

Never commit registry credentials or expose them in logs, issues, pull requests, documentation, or chat.

## Package contents

The crates.io package intentionally contains the reference Rust package plus the documentation necessary to understand the protocol and its trust boundary:

```text
Cargo.toml
Cargo.lock
LICENSE
README.md
SECURITY.md
CONTRIBUTING.md
src/**
docs/**
references/**
```

`references/memhooks-format.md` is the normative protocol contract and therefore **must ship in the crate**. Provider references under `references/memory-systems/` ship too so README/docs links remain meaningful to registry users.

Repository artwork, Python/Hermes runtime adapters, and example projects remain repository artifacts rather than Rust crate payload.

## docs.rs and doctests

`src/lib.rs` includes `docs/rust-library.md` as crate-level documentation. Public Rust API changes must update that guide in the same pull request.

CI treats Rustdoc warnings as errors and runs:

```bash
cargo test --locked --doc
```

A green `cargo test --all-targets` alone is not sufficient because Cargo's doctests are a separate target class.

## Package version versus protocol version

They are intentionally separate.

```text
package/release: 0.5.1
protocol schema:  memhooks/v2
```

A patch release does not require another schema generation when the protocol contract remains compatible.

## Recommended release sequence

1. update package/skill versions, README/status, docs, tests, and changelog;
2. regenerate and commit `Cargo.lock` when dependencies changed;
3. run the full PR CI suite: format, Clippy, tests, doctests, Rustdoc, self-validation, MSRV, Python matrix/Ruff/tests, package verification;
4. merge only the green release PR;
5. run `cargo publish --locked` with the configured registry credential;
6. install the exact published version from crates.io and run `memhooks --version`;
7. create/verify the matching GitHub release tag against the clean merged implementation commit;
8. verify docs.rs builds the published documentation;
9. remove one-shot release workflows after successful publication;
10. run ordinary `main` CI on the final housekeeping tree.

## CI workflow policy

The normal workflow uses:

- explicit read-only `contents` permission;
- pinned action commit SHAs;
- concurrency cancellation for superseded runs;
- committed/locked Rust dependencies;
- a Rust 1.78 MSRV job;
- Python 3.10 and 3.12;
- Ruff + pytest;
- explicit doctests.

`cargo publish --dry-run --locked` is reserved for pushes to `main`/release tags. Pull requests use `cargo package --locked --no-verify` in addition to the normal compile/test suite, avoiding a redundant heavy registry dry-run on every intermediate PR revision.

## Provider-neutral release check

For `memhooks/v2`, explicitly verify that provider-native vocabulary has not leaked into the universal core.

Provider-specific concepts belong under `backends.<provider>` and provider reference docs, for example:

- Hindsight bank/categories/Reflect/Mental Models;
- Mem0 scope filters/result limits/thresholds/reranking/graph toggles;
- OpenViking native hierarchy/retrieval controls;
- Honcho peer/session/reasoning controls.

The generic Rust model/validator must not require one provider SDK or ontology.

## Reference-path consistency check

Because the resolver, maintainer, and runtime adapters must share one protocol implementation, release review should include regression coverage for:

- maintainer-written cue immediately visible through `resolve`/`explain`;
- canonical Git-root containment;
- same-text child-query override;
- body `guidance` present in adapter handoff;
- schema rejection in bundled runtime adapters;
- atomic/locked maintenance;
- exact YAML diagnostic locations;
- repository-relative SARIF paths.

## Trusted publishing

Now that crate ownership exists, crates.io trusted/passwordless publishing can be preferred when configured. Until then, keep the registry token limited to the repository secret and rotate/revoke it according to normal credential hygiene.
