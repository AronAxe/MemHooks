# Quickstart

For a normal user, enabling MemHooks should be simple. You are **not** expected to hand-curate `MEMHOOKS.md` files as the project evolves.

## 1. Install the tooling

```bash
cargo install memhooks
```

For Hermes/Hermes Desktop, also install the pre-LLM adapter as described in [`hooks/hermes/README.md`](../hooks/hermes/README.md).

## 2. Enable MemHooks once in the project

Inside an Agent Skill environment:

```text
/memhooks init
```

Direct CLI equivalent:

```bash
memhooks init /path/to/project
```

Initialization creates a root `MEMHOOKS.md` using `schema: memhooks/v2` at the canonical project root.

A minimal hook is conceptually:

```md
---
schema: memhooks/v2
inherits: true
---

# MemHooks

Recall prior decisions, constraints, failures, fixes, rejected approaches, and
unresolved issues concerning this project before substantive changes.
```

## 3. Let the agent/runtime maintain it

After initialization, the normal lifecycle is automatic:

```text
agent works in project area
        ↓
runtime reports explicit touched paths / agent learns semantic cue
        ↓
memhooks event / memhooks note
        ↓
YAML frontmatter is atomically maintained
        ↓
future agent enters that area
        ↓
reference resolver reads the same frontmatter
        ↓
relevant memory is recalled before substantive work
```

There is no separate Markdown-body JSON note database in v0.5.1. The writer and resolver share one structured data model.

The human user does not need to decide how many local hooks exist, how long they should be, or how individual retrieval questions are phrased. Those are agent/runtime maintenance concerns.

## 4. Optional: inspect or validate

```bash
memhooks validate --all
memhooks explain backend/auth
memhooks explain backend/auth --role reviewer
memhooks explain backend/auth --format json
```

`explain --format json` includes the complete resolved adapter handoff, including source-attributed Markdown guidance.

A clean validation prints:

```text
MemHooks validation passed with no diagnostics.
```

A typo in the target path is an error rather than a misleading empty success.

## 5. Memory backend integration

MemHooks does not store memories itself. The active runtime translates the resolved routing plan into the configured memory backend.

Provider-native controls remain namespaced:

```yaml
backends:
  mem0:
    filters:
      user_id: project-agent
    top_k: 8

  hindsight:
    bank: project-memory
    memory_types: [experience]
```

Normal users should generally configure provider selection through the host runtime rather than manually inserting provider internals into every hook.

Provider references:

- [Hindsight](../references/memory-systems/01-hindsight.md)
- [OpenViking](../references/memory-systems/02-openviking.md)
- [Honcho](../references/memory-systems/03-honcho.md)
- [Mem0](../references/memory-systems/04-mem0.md)

## Canonical root rule

Inside Git, the nearest Git root is a hard boundary unless the host explicitly supplies a containing `MEMHOOKS_ROOT`. A hook in your home directory cannot silently capture maintenance writes for an uninitialized repository below it.

## For runtime/agent developers

Continue with [Agent integration](integrating-an-agent.md). It covers the canonical resolver/maintainer APIs, bounded retrieval, provider namespaces, atomic maintenance, and trust boundaries that runtimes should handle on the user's behalf.