# CLI Cookbook

The `memhooks` binary is both the reference CLI and a thin surface over the same Rust library used by adapters.

## Install

```bash
cargo install memhooks
```

## Initialize a project

```bash
memhooks init .
```

This creates the root `MEMHOOKS.md` using `memhooks/v2`.

## Validate one file or subtree

```bash
memhooks validate MEMHOOKS.md
memhooks validate backend/auth
```

## Validate the whole repository

```bash
memhooks validate --all
```

Machine-readable formats:

```bash
memhooks validate --all --format json
memhooks validate --all --format sarif
```

SARIF paths are normalized for repository-relative code-scanning use when a repository root is available, and diagnostic rule metadata is emitted so code-scanning tools can display useful titles/help.

## Explain the effective routing plan

```bash
memhooks explain backend/auth
```

With a known runtime role:

```bash
memhooks explain backend/auth --role reviewer
```

Several active roles:

```bash
memhooks explain backend/auth \
  --role reviewer \
  --role security
```

JSON adapter handoff:

```bash
memhooks explain backend/auth --format json
```

The JSON output includes the resolved structure itself, including source-attributed guidance, rather than a manually duplicated subset of fields.

## Add a semantic recall cue

Simple:

```bash
memhooks note \
  --cwd "$PWD" \
  --query "What previous failures affect this code?"
```

Weighted:

```bash
memhooks note \
  --cwd "$PWD" \
  --query "Why was this locking strategy selected?" \
  --priority 0.9
```

Role-routed:

```bash
memhooks note \
  --cwd "$PWD" \
  --query "Which rejected designs should a reviewer know about?" \
  --role reviewer
```

With generic cues:

```bash
memhooks note \
  --cwd "$PWD" \
  --query "Which outage lessons affect authentication?" \
  --entity Authentication \
  --resource auth-postmortem \
  --tag security
```

With provider-native hints:

```bash
memhooks note \
  --cwd "$PWD" \
  --query "Which outage lessons affect authentication?" \
  --backends '{"mem0":{"top_k":8,"rerank":true}}'
```

## Process a runtime event

`memhooks event` reads structured event JSON from stdin.

A runtime might send:

```json
{
  "hook_event_name": "post_tool_call",
  "cwd": "/repo",
  "tool_input": {
    "path": "src/auth/refresh.rs"
  }
}
```

Then:

```bash
printf '%s' "$EVENT_JSON" | memhooks event
```

Only explicit path-bearing fields are considered for automatic path anchors, and candidates must resolve to real files inside the repository boundary.

## Check a CI pipeline

Typical Rust/project checks:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo test --locked --doc
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
cargo run --locked --quiet -- validate --all
```

## Nonexistent paths

Typos should fail rather than silently producing a plausible empty report.

For example:

```bash
memhooks explain /definitely/not/a/path
```

should be treated as an error.

Likewise, validating a nonexistent explicit path should not be reported as a clean project.

## Useful workflow patterns

### Before a risky refactor

```bash
memhooks explain src/critical --role reviewer
```

### After discovering a durable constraint

```bash
memhooks note \
  --cwd src/critical \
  --query "Why must writes remain idempotent across retries?" \
  --priority 1.0 \
  --tag reliability
```

### Audit all hooks before merging

```bash
memhooks validate --all
```

### Generate code-scanning output

```bash
memhooks validate --all --format sarif > memhooks.sarif
```

## What the CLI does not do

The CLI does not:

- contact your memory provider;
- decide which provider is authorized;
- validate arbitrary provider-internal API keys/options;
- execute commands from hook files;
- create/delete memories merely because a hook exists;
- grant prompt privilege;
- start a daemon.

It is intentionally a parser/resolver/validator/maintainer surface, not a complete agent harness.