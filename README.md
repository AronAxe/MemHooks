<h1 align="center">MemHooks</h1>
<p align="center"><strong>Mnemonic devices for agents.</strong></p>
<p align="center">Filesystem-scoped memory recall · <em>Hook the right memories into the right context.</em> 🎣</p>

<p align="center">
  <img alt="Agent Skills" src="https://img.shields.io/badge/Agent%20Skills-compatible-7c4dff" />
  <img alt="Hermes" src="https://img.shields.io/badge/Hermes-compatible-00bcd4" />
  <img alt="Backend neutral" src="https://img.shields.io/badge/memory%20backend-neutral-2ea44f" />
  <img alt="Crates.io" src="https://img.shields.io/crates/v/memhooks" />
  <img alt="Version" src="https://img.shields.io/badge/version-0.5.1-orange" />
  <img alt="License" src="https://img.shields.io/badge/license-MIT-blue" />
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/AronAxe/MemHooks/main/assets/memhook2.png" alt="How MemHooks works" width="100%" />
</p>

## The idea

> **You can't recall what you don't know you know.**

An agent can have the right memory stored perfectly and still fail to use it because retrieval begins with a cue. MemHooks stores that cue close to the code or folder where it matters.

> **Memory systems know how to remember. MemHooks tells the agent what to recall here.**

When an agent works in a directory, MemHooks resolves `MEMHOOKS.md` root → leaf, applies inheritance and role routing, and produces a bounded retrieval plan for the memory backend the runtime actually uses.

```text
workspace/
├── MEMHOOKS.md
├── backend/
│   ├── MEMHOOKS.md
│   └── auth/
│       ├── MEMHOOKS.md
│       └── refresh.py
```

```text
root hook
   ↓
local hooks
   ↓
reference resolver
   ↓
backend-neutral retrieval intent + provider hints
   ↓
memory adapter
   ↓
bounded relevant context
   ↓
do the work
```

## Who does what?

**Normal users generally do not hand-maintain `MEMHOOKS.md`.**

- **User:** installs/enables MemHooks and may configure a memory backend.
- **Agent/runtime:** creates, updates, prunes, and consumes local retrieval cues.
- **MemHooks reference engine:** parses, validates, resolves, explains **and maintains** the hook frontmatter.
- **Memory backend:** stores and retrieves the actual memories.

There is deliberately **one structured data path** in v0.5.1:

```text
agent/runtime
    ↓
memhooks note / memhooks event
    ↓
YAML frontmatter
    ↓
reference parser + resolver
    ↓
memhooks explain / Rust API
    ↓
adapter
```

The Markdown body is optional source-attributed retrieval guidance. It is not a second hidden routing database.

## Backend-neutral core

`memhooks/v2` separates universal retrieval intent from provider-native controls.

The core understands only provider-neutral concepts:

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

Provider-native controls live under:

```yaml
backends:
  <provider>:
    ... provider-owned configuration ...
```

MemHooks preserves and structurally merges provider mappings but deliberately does not interpret their internal keys.

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
        strategy: reflect

entities:
  - name: Authentication
    type: CONCEPT
    salience: 0.95

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

Here priority, roles, entities, resources, tags, and exclusions are MemHooks concepts. Mem0 filters/reranking and Hindsight banks/types/Reflect are provider-owned hints.

## Provider references

First-class adapter references are included for:

- [Hindsight](references/memory-systems/01-hindsight.md)
- [OpenViking](references/memory-systems/02-openviking.md)
- [Honcho](references/memory-systems/03-honcho.md)
- [Mem0](references/memory-systems/04-mem0.md)
- [Generic / unknown backends](references/memory-systems/99-generic-or-unknown.md)

The provider references ship inside the crates.io package as of v0.5.1.

## Install

```bash
cargo install memhooks
```

For an exact release:

```bash
cargo install memhooks --version 0.5.1
```

Rust runtimes can use the same engine directly:

```bash
cargo add memhooks@0.5.1
```

## CLI

### Enable a project

```bash
memhooks init .
```

### Validate

```bash
memhooks validate --all
memhooks validate --all --format json
memhooks validate --all --format sarif
```

A nonexistent target is an error rather than a misleading empty success.

### Explain the complete handoff

```bash
memhooks explain backend/auth
memhooks explain backend/auth --role reviewer
memhooks explain backend/auth --format json
```

The JSON handoff serializes the resolved structure rather than a manually duplicated field list. It includes source-attributed Markdown `guidance` as well as core/provider routing.

### Add a semantic cue

```bash
memhooks note \
  --cwd "$PWD" \
  --query "Why did authentication change after the outage?" \
  --priority 0.9 \
  --role reviewer \
  --entity '{"name":"Authentication","type":"CONCEPT","salience":0.95}' \
  --resource '{"name":"auth-postmortem","kind":"postmortem","salience":0.9}' \
  --tag security \
  --backends '{"mem0":{"top_k":8}}'
```

The cue is written directly into YAML frontmatter and is immediately visible through `memhooks explain` and the Rust resolver.

### Maintain deterministic file anchors

Runtime adapters can pipe one `post_tool_call` JSON event into:

```bash
memhooks event
```

Automatic anchors only inspect explicit path-bearing tool-input fields and only retain files that actually exist inside the canonical project root. The maintainer does not regex arbitrary file contents for dotted strings.

## v0.5.1 hardening

v0.5.1 is an implementation hardening release; the protocol remains `memhooks/v2`.

Notable changes:

- **one normative data model** — the reference maintainer writes the same YAML frontmatter the resolver reads;
- **complete resolver handoff** — `guidance` is no longer dropped by `explain`;
- **canonical root semantics** — a Git root is a hard boundary unless an explicit containing `MEMHOOKS_ROOT` is supplied;
- **local query override** — same trimmed query text at a child scope replaces parent query metadata instead of issuing the query twice;
- **precise diagnostics** — YAML AST spans locate the actual malformed `priority`, `salience`, etc.;
- **tolerant linting** — malformed structured entries such as `quer:` reach targeted diagnostics rather than killing the whole file as an enum parse error;
- **trust-boundary hardening** — the Hermes adapter consumes validated resolver JSON and injects it explicitly as untrusted repository-controlled data; raw BEGIN/END fences are gone;
- **atomic maintenance** — exclusive lock + same-directory atomic replacement;
- **SARIF hardening** — repository-relative artifact paths and rule metadata;
- **reproducible builds** — committed `Cargo.lock`, declared MSRV CI, doctests, Python 3.10/3.12, Ruff, and locked Rust builds;
- **maintained YAML stack** — deprecated `serde_yaml` replaced with `serde_yaml_ng`, with Saphyr used for source spans;
- `.gitignore`, `SECURITY.md`, `CONTRIBUTING.md`, corrected funding metadata, and a root `MEMHOOKS.md` for dogfooding.

## Canonical root semantics

Resolver, maintainer, and bundled runtime adapters use the same rules:

1. an explicit containing `MEMHOOKS_ROOT` wins;
2. otherwise the nearest Git root is a hard boundary;
3. outside Git, the highest hooked ancestor may serve as root.

A stray hook above a Git repository cannot silently capture maintenance writes for that repository.

## Query inheritance

For same-text queries, locality behaves like an override:

```yaml
# root
recall_queries:
  - query: "What security invariant matters here?"
    priority: 0.2
```

```yaml
# child
recall_queries:
  - query: "What security invariant matters here?"
    priority: 0.9
```

The resolved plan contains the query **once**, at priority `0.9`, sourced from the child hook.

Other generic list cues accumulate with duplicate suppression; more-local scalar core fields win. Provider mappings use recursive mapping merge with local scalar/list replacement.

## Hermes / Hermes Desktop

Install the reference binary first:

```bash
cargo install memhooks
```

Clone the skill and install the pre-LLM adapter:

```bash
git clone https://github.com/AronAxe/MemHooks.git ~/.hermes/skills/memhooks
mkdir -p ~/.hermes/agent-hooks
cp ~/.hermes/skills/memhooks/hooks/hermes/memhooks_pre_llm.py ~/.hermes/agent-hooks/
```

Then configure Hermes:

```yaml
hooks:
  pre_llm_call:
    - command: "python3 ~/.hermes/agent-hooks/memhooks_pre_llm.py"
      timeout: 5
  post_tool_call:
    - command: "memhooks event"
      timeout: 5
```

The pre-LLM adapter does **not** parse YAML itself. It asks `memhooks explain --format json` for one validated routing plan and injects structurally bounded JSON marked as **untrusted repository-controlled retrieval metadata**.

The legacy `scripts/memhooks_update.py` remains only as a compatibility launcher to the Rust CLI; it no longer owns parsing or persistence.

## Documentation

- [Human-friendly Wiki source](wiki/Home.md) — approachable, cross-linked explanation intended for the GitHub Wiki tab
- [Documentation index](docs/README.md)
- [Quickstart](docs/quickstart.md)
- [CLI reference](docs/cli.md)
- [Rust library/API guide](docs/rust-library.md)
- [Agent integration guide](docs/integrating-an-agent.md)
- [Protocol guide](docs/protocol-guide.md)
- [Troubleshooting](docs/troubleshooting.md)
- [Security policy](SECURITY.md)
- [Contributing](CONTRIBUTING.md)

The Wiki is explanatory. The normative protocol contract is [`references/memhooks-format.md`](references/memhooks-format.md).

## What MemHooks is not

MemHooks is **not** a vector database, memory provider, automatic memory-writing system, GraphRAG framework, prompt-privilege mechanism, shell-command manifest, or background daemon.

It is deliberately narrow infrastructure:

> **When an agent works here, remember these things first.**

## Related: Token Terminator

If MemHooks is about **retrieving the right context**, [**Token Terminator**](https://github.com/AronAxe/Token-Terminator) is about **not wasting tokens on the wrong context**.

## Status

**v0.5.1 — backend-neutral `memhooks/v2` + one normative resolver/maintainer data path + hardened runtime/diagnostic/security boundaries.**

<p align="center">
  <img src="https://raw.githubusercontent.com/AronAxe/MemHooks/main/assets/memhooklogo.png" alt="MemHooks logo" width="300" />
</p>

## License

MIT © 2026 Aron Bijl