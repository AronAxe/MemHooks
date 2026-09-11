<h1 align="center">MemHooks</h1>
<p align="center"><strong>Mnemonic devices for agents.</strong></p>
<p align="center">Filesystem-scoped memory recall · <em>Hook the right memories into the right context.</em> 🎣</p>

<p align="center">
  <img alt="Agent Skills" src="https://img.shields.io/badge/Agent%20Skills-compatible-7c4dff" />
  <img alt="Hermes" src="https://img.shields.io/badge/Hermes-compatible-00bcd4" />
  <img alt="Memory agnostic" src="https://img.shields.io/badge/memory-backend%20agnostic-2ea44f" />
  <img alt="Crates.io" src="https://img.shields.io/crates/v/memhooks" />
  <img alt="Version" src="https://img.shields.io/badge/version-0.4.1-orange" />
  <img alt="License" src="https://img.shields.io/badge/license-MIT-blue" />
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/AronAxe/MemHooks/main/assets/memhook2.png" alt="How MemHooks works" width="100%" />
</p>

## The idea

> **You can't recall what you don't know you know.**

An agent can have the right memory stored perfectly and still fail to use it because retrieval begins with a cue. MemHooks stores that cue close to the code or folder where it matters.

> **Memory systems know how to remember. MemHooks tells the agent what to recall here.**

A `MEMHOOKS.md` file contains retrieval routing: concrete recall questions, optional memory categories, connection emphasis, entities, priority, role applicability, exclusions, and references to existing synthesized resources. Before substantive work, an agent resolves the applicable files from repository root to the active directory and performs bounded recall.

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
translate to current memory backend
   ↓
recall the highest-value context that matters
   ↓
do the work
```

## Install the Rust tooling

### CLI

Once published on crates.io, installation is simply:

```bash
cargo install memhooks
```

Then use:

```bash
# Validate hooks below the current directory
memhooks validate

# Validate every MEMHOOKS.md in the repository
memhooks validate --all

# Machine-readable output for CI or other agents
memhooks validate --all --format json
memhooks validate --all --format sarif

# Explain the inherited/effective routing for a directory
memhooks explain backend/auth

# Explain it for an active agent role
memhooks explain backend/auth --role reviewer
memhooks explain backend/auth --role reviewer --role architect --format json
```

### Rust library

Rust agent frameworks can use the same parser/resolver directly instead of reimplementing MemHooks semantics:

```bash
cargo add memhooks
```

In plain English: **the CLI and the reusable engine are the same project.** A framework can either run the `memhooks` command or call the Rust library internally.

For an exact release:

```bash
cargo install memhooks --version 0.4.1
cargo add memhooks@0.4.1
```

The reference implementation **does not execute shell commands and does not contact a memory backend**. It parses, resolves, filters, explains, and validates the routing plan.

## What's in v0.4.x

- **Weighted retrieval** — structured queries may carry `priority: 0.0..1.0`.
- **Entity salience** — structured entities may carry `salience: 0.0..1.0`.
- **Role-based routing** — queries may declare `when.roles`.
- **Rust reference implementation** — reusable parser/model/resolver/validator library plus CLI.
- **Validation** — human, JSON, and SARIF diagnostics with CI-friendly exit codes.
- **Explainability** — `memhooks explain` shows which hooks were inherited and which queries are effective.
- **Backward compatibility** — the schema remains `memhooks/v1`; legacy string queries/entities continue to work.
- **v0.4.1 packaging** — publish-ready crates.io metadata and a lean crate package containing the Rust implementation, README, and license rather than repository artwork/runtime extras.

## Weighted retrieval and role routing

```yaml
recall_queries:
  - query: "What security boundaries must never be violated in this module?"
    priority: 1.0
    when:
      roles: [reviewer, architect]

  - query: "What naming conventions are preferred here?"
    priority: 0.35
```

`priority` is the importance of the **recall request** under a finite context budget. It is not semantic similarity, confidence, truth probability, or a mandatory backend score multiplier.

If `priority` is omitted, MemHooks assigns no mandatory numeric default. Existing queries remain ordinary/unweighted queries.

Entities have their own concept:

```yaml
entities:
  - name: Authentication
    type: CONCEPT
    salience: 0.95
```

`salience` describes the importance of an **entity as a retrieval cue** and is deliberately separate from query priority.

Role names are open project/runtime-defined strings:

```yaml
when:
  roles: [reviewer, refactor]
```

When active roles are known, a restricted query applies if **any** listed role matches. Unrestricted queries apply to every role.

## A complete hook example

```md
---
schema: memhooks/v1
inherits: true

memory_types:
  - world
  - experience
  - observation

connection_types:
  - semantic
  - temporal
  - entity
  - causal

recall_queries:
  - query: "Why did the authentication design change after the outage?"
    priority: 1.0
    when:
      roles: [reviewer, architect]
    memory_types: [experience]
    connection_types: [causal, temporal]
    entities:
      - Authentication
      - name: OpenAI
        type: ORG
        salience: 0.9

entities:
  - name: Authentication
    type: CONCEPT
    salience: 0.95

exclude:
  - obsolete OAuth prototype
---

Use direct recall first. Use deeper memory reasoning only if the retrieved
facts disagree or the rationale is still unclear.
```

The file contains **retrieval metadata, not the memory itself**.

## Root-to-leaf behavior

Given:

```text
/repo/MEMHOOKS.md
/repo/backend/MEMHOOKS.md
/repo/backend/auth/MEMHOOKS.md
```

an agent working in `/repo/backend/auth/` resolves all three in that order.

- Lists accumulate and exact duplicates are removed.
- More local scalar values win.
- `inherits: false` cuts off the parent chain.
- Query-local memory/connection types override scope defaults for that query.
- Query-local entities supplement inherited entities.
- Priority and role conditions remain attached to the query that declared them.
- Entity salience remains attached to that entity.
- Role filtering happens after inheritance resolution when active-role information exists.
- Retrieval remains bounded; the result is a routing plan, not permission to dump an entire memory store into context.

Read the complete contract in [`references/memhooks-format.md`](references/memhooks-format.md).

## Hindsight ontology mapping

MemHooks keeps these dimensions separate:

- memory categories: `world | experience | observation`
- connection emphasis: `semantic | temporal | entity | causal`
- entities: actual named things, optionally with entity type and salience
- mental models / Knowledge Pages: higher-level existing synthesized retrieval targets

See [`references/memory-systems/01-hindsight.md`](references/memory-systems/01-hindsight.md) for the mapping. OpenViking, Honcho, and generic-backend mappings are also included under [`references/memory-systems/`](references/memory-systems/).

## Zero-LLM maintenance

MemHooks includes `scripts/memhooks_update.py` so routing files can evolve without a separate model call.

Simple note:

```bash
python3 scripts/memhooks_update.py note \
  --cwd "$PWD" \
  --query "Why was refresh-token rotation split into two stages?"
```

Routed note:

```bash
python3 scripts/memhooks_update.py note \
  --cwd "$PWD" \
  --query "Why did the authentication design change after the outage?" \
  --priority 1.0 \
  --role reviewer \
  --role architect \
  --memory-type experience \
  --connection-type causal \
  --connection-type temporal \
  --entity 'Authentication' \
  --entity '{"name":"OpenAI","type":"ORG","salience":0.9}'
```

Deterministic path auto-anchors deliberately do **not** guess memory type, role, priority, entity type, salience, or causal structure.

## Hermes / Hermes Desktop

Clone the repository into the active Hermes skills directory:

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

Then enable MemHooks once inside a project:

```text
/memhooks init
```

See [`hooks/hermes/README.md`](hooks/hermes/README.md) for the runtime-specific details.

## File format at a glance

| Field | Purpose |
|---|---|
| `inherits` | inherit parent-directory hooks (`true` by default) |
| `memory_types` | optional memory categories |
| `connection_types` | optional semantic/temporal/entity/causal emphasis |
| `recall_queries` | concrete questions worth asking memory |
| `recall_queries[].priority` | optional `0.0..1.0` importance under a finite context budget |
| `recall_queries[].when.roles` | optional role-based applicability |
| `entities` | named entities, optionally `{name, type, salience}` |
| `mental_models` | existing standing answers worth retrieving first when supported |
| `knowledge_pages` | existing stable synthesized pages/resources |
| `tags` | backend-neutral relevance/scoping hints |
| `exclude` | obsolete or misleading context to keep out |
| `bank` | optional memory namespace/bank/session hint |
| `sensitivity` | advisory handling metadata |

## What MemHooks is not

MemHooks is **not** a vector database, memory provider, automatic memory-writing system, entity-relationship schema, directive store, Knowledge Page generator, GraphRAG framework, shell-command manifest, or excuse to shove more tokens into every prompt.

It is deliberately boring infrastructure:

> **When an agent works here, remember these things first.**

## Related: Token Terminator

If MemHooks is about **retrieving the right context**, [**Token Terminator**](https://github.com/AronAxe/Token-Terminator) is about **not wasting tokens on the wrong context**.

## Status

**v0.4.1 — experimental convention + published Rust reference resolver/linter + deterministic load-and-maintain runtime.**

The format remains intentionally small and backward-compatible. Issues, backend mappings, adapters, and real-world examples are welcome.

<p align="center">
  <img src="https://raw.githubusercontent.com/AronAxe/MemHooks/main/assets/memhooklogo.png" alt="MemHooks logo" width="300" />
</p>

## License

MIT © 2026 Aron Bijl
