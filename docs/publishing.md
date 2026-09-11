# Publishing the Rust crate

This is maintainer documentation for publishing `memhooks` to crates.io.

## Current registry state

MemHooks v0.4.1 is already published on crates.io. The repository has an authenticated crates.io publishing secret configured and every release candidate is package-tested with:

```bash
cargo publish --dry-run
```

The first publication was also verified by installing the registry release and running the installed binary.

## Before every release

Run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
cargo run --quiet -- validate --all
rm -f Cargo.lock
cargo publish --dry-run
```

The standard CI workflow performs these checks.

## Authentication

Publishing automation uses the GitHub Actions repository secret:

```text
CARGO_REGISTRY_TOKEN
```

A fallback `CRATES_IO_TOKEN` may be supported by one-shot release workflows, but `CARGO_REGISTRY_TOKEN` is preferred.

Never commit registry credentials to the repository or expose them in logs/issues/chats.

## Package contents

The crates.io package intentionally contains the Rust package and documentation needed by crate/CLI users:

```text
Cargo.toml
LICENSE
README.md
src/**
docs/**
```

Repository artwork, Python runtime hooks, example projects, and unrelated assets do not need to inflate the Rust crate package.

## docs.rs

`src/lib.rs` includes `docs/rust-library.md` as crate-level documentation. Because `docs/**` is packaged, docs.rs builds the API landing page from the same maintained guide used in the repository.

When public Rust APIs change, update `docs/rust-library.md` in the same pull request. CI treats Rustdoc warnings as errors.

## Package version versus protocol version

They are intentionally separate.

Current v0.5 release line:

```text
package/release: 0.5.0
protocol schema:  memhooks/v2
```

A package patch/minor release does not inherently require another protocol schema change. Change the protocol version only when the protocol contract itself changes incompatibly or deliberately establishes a new schema generation.

## Recommended release sequence

1. update `Cargo.toml`, `SKILL.md`, README/status, docs, examples, tests, and changelog;
2. run full CI including Rustdoc and `cargo publish --dry-run`;
3. merge the release PR;
4. publish the crate with the configured registry credential;
5. verify the exact version can be installed from crates.io;
6. verify/update the matching GitHub release/tag;
7. confirm docs.rs can build the packaged documentation;
8. remove one-shot publisher workflows after successful publication.

## Provider-neutral release check

For `memhooks/v2`, a release review should explicitly verify that provider-native vocabulary has not leaked into the universal core.

Provider-specific concepts belong under `backends.<provider>` and provider reference docs. Examples:

- Hindsight `bank`, memory categories, Reflect, Mental Models;
- Mem0 filters, entity-scope IDs, result limits, thresholds, reranking, graph toggles;
- OpenViking hierarchy/native retrieval controls;
- Honcho peer/session/reasoning controls.

The generic Rust model/validator must not require one provider SDK or provider ontology.

## Trusted publishing

Now that crate ownership exists, crates.io trusted/passwordless publishing can be preferred when configured and appropriate. Until then, keep the registry token limited to the repository secret and rotate/revoke it according to normal credential hygiene.
