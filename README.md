<h1 align="center">MemHooks</h1>
<p align="center"><strong>Mnemonic devices for agents.</strong></p>
<p align="center">Filesystem-scoped memory recall · <em>Hook the right memories into the right context.</em> 🎣</p>

<p align="center">
  <img alt="Agent Skills" src="https://img.shields.io/badge/Agent%20Skills-compatible-7c4dff" />
  <img alt="Hermes" src="https://img.shields.io/badge/Hermes-compatible-00bcd4" />
  <img alt="Backend neutral" src="https://img.shields.io/badge/memory%20backend-neutral-2ea44f" />
  <img alt="Crates.io" src="https://img.shields.io/crates/v/memhooks" />
  <img alt="Version" src="https://img.shields.io/badge/version-0.5.0-orange" />
  <img alt="License" src="https://img.shields.io/badge/license-MIT-blue" />
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/AronAxe/MemHooks/main/assets/memhook2.png" alt="How MemHooks works" width="100%" />
</p>

## The idea

> **You can't recall what you don't know you know.**

An agent can have the right memory stored perfectly and still fail to use it because retrieval begins with a cue. MemHooks stores that cue close to the code or folder where it matters.

> **Memory systems know how to remember. MemHooks tells the agent what to recall here.**

When an agent works in a directory, MemHooks resolves the applicable `MEMHOOKS.md` files from repository root to that directory, applies role routing, and produces a bounded retrieval plan for whatever memory backend the runtime actually uses.

```text
workspace/
├── MEMHOOKS.md
├── backend/
│   ├── MEMHOOKS.md
│   └── auth/
│       ├── MEMHOOKS.md
│       └── refresh.py
```

Working in `backend/auth/` means:

```text
root hook
   ↓ inherit
backend hook
   ↓ inherit
local auth hook
   ↓
resolve + apply role routing
   ↓
hand backend-neutral intent + provider hints to the runtime
   ↓
recall bounded relevant context
   ↓
do the work
```

## Who does what?

**Normal users generally do not hand-maintain `MEMHOOKS.md`.**

- **User:** installs/enables MemHooks and may choose or configure a memory backend.
- **Agent/runtime:** creates and maintains local retrieval cues as work evolves, and decides which backend is active.
- **MemHooks reference engine:** parses, inherits, validates, filters, explains, and preserves provider-specific hints without interpreting them.
- **Memory backend:** stores and retrieves the actual memories.

The hook files are infrastructure for agents. Advice about pruning, cue quality, or routing-file size is therefore directed at agents/runtime authors, not at end users.

## v0.5.0: genuinely backend-neutral

`memhooks/v2` separates universal retrieval intent from provider-native controls.

The **core** understands only concepts that are useful regardless of memory vendor:

- `recall_queries`
- query `priority`
- `when.roles`
- `entities`
- `resources`
- `tags`
- `exclude`
- `scope`
- `sensitivity`
- filesystem inheritance

Anything native to a particular memory system belongs under:

```yaml
backends:
  <provider>:
    ... provider-owned configuration ...
```

MemHooks deliberately does **not** know what those provider keys mean.

### Example

```yaml
---
schema: memhooks/v2
inherits: true
scope: backend/auth

recall_queries:
  - query: "Why did the authentication design change after the outage?"
    priority: 1.0
    when:
      roles: [reviewer, architect]
    entities:
      - Authentication
    resources:
      - name: auth-postmortem
        kind: postmortem
        salience: 0.9
    tags: [security]
    backends:
      mem0:
        top_k: 8
        rerank: true
      hindsight:
        memory_types: [experience]
        connection_types: [causal, temporal]
        strategy: reflect

entities:
  - name: Authentication
    type: CONCEPT
    salience: 0.95

resources:
  - auth-architecture

tags: [backend]
exclude:
  - obsolete OAuth prototype

backends:
  mem0:
    filters:
      user_id: auth-agent
  hindsight:
    bank: project-memory
---
```

Here `priority`, roles, entities, resources, tags, and exclusions are MemHooks concepts. `top_k`, `rerank`, Mem0 filters, Hindsight memory types, `bank`, and `reflect` are **not**. They are opaque provider hints carried under their own namespace.

## Provider namespaces

v0.5.0 ships first-class reference mappings for:

- [Hindsight](references/memory-systems/01-hindsight.md)
- [OpenViking](references/memory-systems/02-openviking.md)
- [Honcho](references/memory-systems/03-honcho.md)
- [Mem0](references/memory-systems/04-mem0.md)
- [Generic / unknown backends](references/memory-systems/99-generic-or-unknown.md)

These mappings explain how an adapter can interpret its own namespace. The Rust core treats all backend namespace contents as opaque YAML mappings.

### Merge behavior for provider hints

Provider mappings inherit root → leaf just like the surrounding hook chain, but MemHooks does not invent provider semantics:

- mapping/object values are recursively merged;
- a more local scalar replaces the parent scalar;
- a more local list replaces the parent list;
- query-local `backends.<provider>` overlays the resolved scope-level provider configuration.

That gives adapters predictable configuration without pretending MemHooks understands every provider API.

## Install the Rust tooling

### CLI

```bash
cargo install memhooks
```

Useful commands:

```bash
memhooks validate --all
memhooks validate --all --format json
memhooks validate --all --format sarif

memhooks explain backend/auth
memhooks explain backend/auth --role reviewer
memhooks explain backend/auth --role reviewer --format json
```

### Rust library

```bash
cargo add memhooks
```

A Rust runtime can use the reference resolver directly instead of reimplementing inheritance and role-routing semantics.

For this release specifically:

```bash
cargo install memhooks --version 0.5.0
cargo add memhooks@0.5.0
```

The reference implementation does **not** contact a memory backend and does **not** execute commands from hook files. It resolves retrieval intent and provider configuration; the host runtime performs retrieval.

## Core fields at a glance

| Field | Meaning |
|---|---|
| `schema` | currently `memhooks/v2` |
| `inherits` | inherit ancestor hooks; defaults to `true` |
| `scope` | optional backend-neutral scope label |
| `recall_queries` | concrete questions worth asking memory |
| `recall_queries[].priority` | optional `0.0..1.0` importance under context pressure |
| `recall_queries[].when.roles` | optional exact-string role applicability |
| `entities` | named retrieval cues, optionally with open `type` and `salience` |
| `resources` | named existing resources, optionally with open `kind` and `salience` |
| `tags` | backend-neutral routing/relevance labels |
| `exclude` | obsolete or misleading context to keep out |
| `sensitivity` | advisory handling metadata |
| `backends` | opaque provider namespaces interpreted by adapters, not by the core |

The normative contract is [`references/memhooks-format.md`](references/memhooks-format.md).

## Root-to-leaf behavior

Given:

```text
/repo/MEMHOOKS.md
/repo/backend/MEMHOOKS.md
/repo/backend/auth/MEMHOOKS.md
```

an agent working in `/repo/backend/auth/` resolves all three in that order unless a local `inherits: false` cuts off the parent chain.

Core lists accumulate with exact duplicate removal; more local scalar core values win. Query-local entities/resources/tags supplement inherited core cues. Role filtering happens after inheritance resolution when active roles are actually known.

Provider namespace mappings use the merge behavior described above.

## Hermes / Hermes Desktop

Clone the skill:

```bash
git clone https://github.com/AronAxe/MemHooks.git ~/.hermes/skills/memhooks
```

Install the lifecycle hooks:

```bash
mkdir -p ~/.hermes/agent-hooks
cp ~/.hermes/skills/memhooks/hooks/hermes/memhooks_pre_llm.py ~/.hermes/agent-hooks/
cp ~/.hermes/skills/memhooks/scripts/memhooks_update.py ~/.hermes/agent-hooks/
chmod +x ~/.hermes/agent-hooks/memhooks_pre_llm.py ~/.hermes/agent-hooks/memhooks_update.py
```

Add to `~/.hermes/config.yaml`:

```yaml
hooks:
  pre_llm_call:
    - command: "python3 ~/.hermes/agent-hooks/memhooks_pre_llm.py"
      timeout: 5
  post_tool_call:
    - command: "python3 ~/.hermes/agent-hooks/memhooks_update.py event"
      timeout: 5
```

Then enable it once inside a project:

```text
/memhooks init
```

After initialization, the runtime/agent maintains the routing cues. See [`hooks/hermes/README.md`](hooks/hermes/README.md).

## Documentation

- [Documentation index](docs/README.md)
- [Quickstart](docs/quickstart.md)
- [CLI reference](docs/cli.md)
- [Rust library/API guide](docs/rust-library.md)
- [Agent integration guide](docs/integrating-an-agent.md)
- [Protocol guide](docs/protocol-guide.md)
- [Troubleshooting](docs/troubleshooting.md)

## What MemHooks is not

MemHooks is **not** a vector database, memory provider, automatic memory-writing system, GraphRAG framework, prompt-privilege mechanism, shell-command manifest, or background daemon.

It is deliberately narrow infrastructure:

> **When an agent works here, remember these things first.**

## Related: Token Terminator

If MemHooks is about **retrieving the right context**, [**Token Terminator**](https://github.com/AronAxe/Token-Terminator) is about **not wasting tokens on the wrong context**.

## Status

**v0.5.0 — experimental backend-neutral retrieval-routing protocol + Rust reference resolver/linter + deterministic agent-maintenance runtime.**

<p align="center">
  <img src="https://raw.githubusercontent.com/AronAxe/MemHooks/main/assets/memhooklogo.png" alt="MemHooks logo" width="300" />
</p>

## License

MIT © 2026 Aron Bijl
