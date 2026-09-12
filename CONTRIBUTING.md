# Contributing to MemHooks

MemHooks is a small protocol with a reference implementation. Changes should keep the protocol understandable without requiring one memory vendor, runtime, or model.

## Before opening a pull request

Run the same checks as CI:

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
```

CI also checks Rust 1.78 explicitly and runs Python tests on 3.10 and 3.12.

If package metadata changed, also run:

```bash
cargo publish --dry-run --locked
```

## Dependency changes

MemHooks ships a binary as well as a library, so `Cargo.lock` is committed. When `Cargo.toml` changes:

1. regenerate `Cargo.lock` with the normal current toolchain;
2. commit the manifest and lockfile together;
3. keep the declared Rust 1.78 MSRV job green;
4. if a transitive dependency raises its MSRV beyond ours, constrain the compatible dependency family in `Cargo.toml` rather than relying only on a lucky lockfile.

Do not delete `Cargo.lock` before CI or publication checks.

## Protocol changes

`references/memhooks-format.md` is normative. Update it together with parser/resolver/validator/maintainer behavior and tests.

Keep the `memhooks/v2` core backend-neutral. Provider-specific concepts belong under `backends.<provider>` and in the relevant provider reference documentation.

## Tests

Prefer regression tests that reproduce an observed failure end-to-end. In particular:

- maintenance changes should prove that what the writer stores is visible through the same reference resolver used by adapters;
- runtime-adapter changes should prove schema/root/trust-boundary behavior;
- diagnostic changes should assert source line/column and SARIF paths where relevant;
- file-maintenance changes should include concurrency/durability coverage.

## Security-sensitive changes

Changes to root discovery, repository-controlled context injection, path extraction, file writes, provider credentials, or prompt placement deserve explicit adversarial tests. See [SECURITY.md](SECURITY.md).