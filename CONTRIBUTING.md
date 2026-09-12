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
python -m compileall -q hooks scripts
pytest -q
```

If package metadata changed, also run:

```bash
cargo publish --dry-run --locked
```

## Protocol changes

`references/memhooks-format.md` is normative. Update it together with parser/resolver/validator behavior and tests.

Keep the `memhooks/v2` core backend-neutral. Provider-specific concepts belong under `backends.<provider>` and in the relevant provider reference documentation.

## Tests

Prefer regression tests that reproduce an observed failure end-to-end. In particular, maintenance changes should prove that what the writer stores is visible through the same reference resolver used by adapters.

## Security-sensitive changes

Changes to root discovery, repository-controlled context injection, path extraction, file writes, provider credentials, or prompt placement deserve explicit adversarial tests. See [SECURITY.md](SECURITY.md).
