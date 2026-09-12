# MemHooks documentation

MemHooks is a filesystem-scoped, **backend-neutral retrieval-routing protocol** for AI agents. These guides cover `memhooks/v2`, the Rust reference resolver/validator/maintainer, runtime integration, security boundaries, and provider adapter mappings.

## Wiki

For the more approachable, cross-linked explanation of MemHooks, start with the [Wiki source](../wiki/Home.md). The repository keeps the Wiki source version-controlled under `wiki/` and can mirror it to GitHub's Wiki tab.

The Wiki is explanatory. The normative protocol contract remains [`references/memhooks-format.md`](../references/memhooks-format.md).

## Start here

- [Quickstart](quickstart.md) — enable MemHooks and understand what happens automatically.
- [CLI reference](cli.md) — `memhooks init`, `note`, `event`, `validate`, and `explain`.
- [Rust library guide](rust-library.md) — call the reference parser/resolver/maintainer directly from Rust.
- [Agent integration guide](integrating-an-agent.md) — wire MemHooks into an agent runtime/harness.
- [Protocol guide](protocol-guide.md) — practical `memhooks/v2` core, root, inheritance, and backend namespace semantics.
- [Troubleshooting](troubleshooting.md) — validator diagnostics and common failure modes.
- [Publishing the Rust crate](publishing.md) — maintainer notes for crates.io releases.
- [Security policy](../SECURITY.md) — trust boundary and private vulnerability reporting.
- [Contributing](../CONTRIBUTING.md) — test/review expectations.

## Who maintains hook files?

Ordinary users normally **do not** hand-maintain `MEMHOOKS.md`.

The normal division is:

1. the user enables MemHooks and optionally configures a memory backend;
2. the agent/runtime creates and updates local retrieval cues through the reference maintainer;
3. the reference engine writes, validates, resolves, and explains the same YAML frontmatter data model;
4. the active memory adapter performs bounded retrieval.

Operational advice about keeping cues concise, pruning stale routing, and choosing provider hints is therefore documented in agent/runtime material rather than presented as a user chore.

## One structured data path

v0.5.1 deliberately has one normative structured store:

```text
memhooks note / memhooks event
        ↓
YAML frontmatter
        ↓
reference parser/resolver
        ↓
memhooks explain / Rust API
        ↓
runtime adapter
```

The Markdown body remains source-attributed free-form guidance. It is not a second machine-routing store.

## Canonical specification

The normative format contract is [`references/memhooks-format.md`](../references/memhooks-format.md). If an explanatory guide conflicts with that reference, the reference wins.

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