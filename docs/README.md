# MemHooks documentation

MemHooks is a filesystem-scoped retrieval-routing convention for AI agents. The files in this directory explain how to use the convention, the Rust reference implementation, and the runtime integration points without requiring readers to reverse-engineer the repository.

## Start here

- [Quickstart](quickstart.md) — get from zero to a working `MEMHOOKS.md`.
- [CLI reference](cli.md) — `memhooks validate` and `memhooks explain`.
- [Rust library guide](rust-library.md) — call the reference parser/resolver directly from Rust.
- [Agent integration guide](integrating-an-agent.md) — wire MemHooks into an agent runtime or harness.
- [Protocol guide](protocol-guide.md) — practical explanation of the `memhooks/v1` fields and merge semantics.
- [Troubleshooting](troubleshooting.md) — diagnostic codes and common failure modes.
- [Publishing the Rust crate](publishing.md) — maintainer notes for crates.io releases.

## Canonical specification

The normative format contract remains [`references/memhooks-format.md`](../references/memhooks-format.md). The documentation here is explanatory. If a prose example here ever conflicts with the canonical format reference, the canonical format reference wins.

## Runtime-specific documentation

- [Hermes / Hermes Desktop integration](../hooks/hermes/README.md)
- [Hindsight mapping](../references/memory-systems/01-hindsight.md)
- [OpenViking mapping](../references/memory-systems/02-openviking.md)
- [Honcho mapping](../references/memory-systems/03-honcho.md)
- [Generic / unknown backend mapping](../references/memory-systems/99-generic-or-unknown.md)

## Design boundaries

MemHooks deliberately does **not** become a memory database, a shell-command manifest, a prompt-privilege system, or a daemon. It describes what context should be recalled for a location and task. The host runtime still owns memory access, trust policy, execution, prompt construction, and context-budget enforcement.
