# Publishing the Rust crate

This is maintainer documentation for publishing `memhooks` to crates.io.

## Current state

MemHooks v0.4.1 is package-tested and passes:

```bash
cargo publish --dry-run
```

The first real crates.io upload is still waiting for the crate owner's one-time registry authentication.

## First publication

1. Sign in to crates.io with the owning GitHub account.
2. Verify the crates.io account email if required.
3. Create a crates.io API token.
4. In the GitHub repository, create an Actions repository secret named:

   ```text
   CARGO_REGISTRY_TOKEN
   ```

5. Put the crates.io token in that secret. Never commit it to the repository.
6. Rerun the prepared v0.4.1 publishing workflow or run locally:

   ```bash
   cargo publish
   ```

A fallback `CRATES_IO_TOKEN` secret name is supported by the prepared one-shot publisher, but `CARGO_REGISTRY_TOKEN` is preferred because Cargo recognizes that convention directly.

## Before every release

Run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo run --quiet -- validate --all
rm -f Cargo.lock
cargo publish --dry-run
```

The standard CI workflow already performs these checks.

## Package contents

The crates.io package intentionally contains only the Rust package and documentation needed by users:

```text
Cargo.toml
LICENSE
README.md
src/**
docs/**
```

Repository artwork, Python runtime hooks, examples, and unrelated project assets do not need to inflate the Rust crate package.

## docs.rs

`src/lib.rs` includes `docs/rust-library.md` as crate-level documentation. Because `docs/**` is packaged, docs.rs can build the API landing page from the same maintained guide used in the repository.

When changing public Rust APIs, update `docs/rust-library.md` in the same pull request.

## Versioning

Package version and protocol version are intentionally separate.

Example:

```text
package/release: 0.4.1
protocol schema:  memhooks/v1
```

A patch/minor package release does not require a new protocol schema when changes remain backward-compatible.

## Recommended release sequence

1. update `Cargo.toml`, `SKILL.md`, README/status, and changelog;
2. run CI including `cargo publish --dry-run`;
3. merge the release PR;
4. publish the crate;
5. verify the version is visible on crates.io and docs.rs begins building;
6. create/verify the matching GitHub release tag;
7. only then change README language from `crates.io-ready` to `published` if necessary.

## After the first publication

Once the crate exists and ownership is established, prefer crates.io's trusted/passwordless publishing mechanism when available for the repository. That removes the need to keep a long-lived API token in GitHub Actions.

Keep the first authenticated publication and any ownership/security changes as explicit maintainer actions rather than hiding credentials in automation.
