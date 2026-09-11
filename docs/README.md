# MemHooks documentation

MemHooks is a filesystem-scoped, **backend-neutral retrieval-routing protocol** for AI agents. These guides cover the `memhooks/v2` protocol, Rust reference implementation, runtime integration, and provider adapter mappings.

## Start here

- [Quickstart](quickstart.md) — enable MemHooks and understand what happens automatically.
- [CLI reference](cli.md) — `memhooks validate` and `memhooks explain`.
- [Rust library guide](rust-library.md) — call the reference parser/resolver directly from Rust.
- [Agent integration guide](integrating-an-agent.md) — wire MemHooks into an agent runtime/harness.
- [Protocol guide](protocol-guide.md) — practical `memhooks/v2` core and backend namespace semantics.
- [Troubleshooting](troubleshooting.md) — validator diagnostics and common failure modes.
- [Publishing the Rust crate](publishing.md) — maintainer notes for crates.io releases.

## Who maintains hook files?

Ordinary users normally **do not** hand-maintain `MEMHOOKS.md`.

The normal division is:

1. the user enables MemHooks and optionally configures a memory backend;
2. the agent/runtime creates and updates local retrieval cues;
3. the reference resolver validates/merges those cues;
4. the active memory adapter performs retrieval.

Operational advice about keeping cues concise, pruning stale routing, and choosing provider hints is therefore documented in the agent/runtime material rather than presented as a user chore.

## Canonical specification

The normative format contract is [`references/memhooks-format.md`](../references/memhooks-format.md). If an explanatory guide conflicts with the format reference, the format reference wins.

## Provider mappings

Provider-native vocabulary lives only inside provider namespaces:

- [Hindsight](../references/memory-systems/01-hindsight.md) — `backends.hindsight`
- [OpenViking](../references/memory-systems/02-openviking.md) — `backends.openviking`
- [Honcho](../references/memory-systems/03-honcho.md) — `backends.honcho`
- [Mem0](../references/memory-systems/04-mem0.md) — `backends.mem0`
- [Generic / unknown backend](../references/memory-systems/99-generic-or-unknown.md)

## Runtime-specific documentation

- [Hermes / Hermes Desktop integration](../hooks/hermes/README.md)

## Design boundary

MemHooks does **not** become a memory database, lowest-common-denominator provider API, shell-command manifest, prompt-privilege system, or daemon.

The core answers:

> **What should the agent remember here?**

Provider namespaces answer:

> **How should this particular memory backend help retrieve it?**

The host runtime still owns memory access, credentials, trust policy, execution, prompt construction, and context-budget enforcement.
