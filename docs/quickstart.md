# Quickstart

For a normal user, enabling MemHooks should be simple. You are **not** expected to hand-curate `MEMHOOKS.md` files as the project evolves.

## 1. Install the tooling

```bash
cargo install memhooks
```

For Hermes/Hermes Desktop, install the MemHooks skill/runtime hooks as described in [`hooks/hermes/README.md`](../hooks/hermes/README.md).

## 2. Enable MemHooks once in the project

Inside an installed Hermes skill environment:

```text
/memhooks init
```

Low-level equivalent used by runtimes:

```bash
python3 scripts/memhooks_update.py init /path/to/project
```

Initialization creates a root `MEMHOOKS.md` using `schema: memhooks/v2`.

A minimal generated hook looks conceptually like:

```md
---
schema: memhooks/v2
inherits: true
entities: []
resources: []
backends: {}
---

# MemHooks

Recall prior decisions, constraints, failures, fixes, rejected approaches, and
unresolved issues concerning this project before making substantive changes.
```

## 3. Let the agent/runtime maintain it

After initialization, the normal lifecycle is automatic:

```text
agent works in a project area
        ↓
runtime notices relevant file activity
        ↓
local retrieval cues are created/refreshed
        ↓
future agent enters that area
        ↓
root→leaf MemHooks are resolved
        ↓
relevant memory is recalled before substantive work
```

The human user does not need to decide how many local hooks exist, how long they should be, or how individual retrieval questions are phrased. Those are agent/runtime maintenance concerns.

## 4. Optional: inspect or validate

Developers and curious users can inspect the routing plan:

```bash
memhooks validate --all
memhooks explain backend/auth
memhooks explain backend/auth --role reviewer
```

A clean validation prints:

```text
MemHooks validation passed with no diagnostics.
```

## 5. Memory backend integration

MemHooks does not store memories itself. The active runtime translates the resolved routing plan into the configured memory backend.

`memhooks/v2` keeps the core provider-neutral. Provider-native controls live under namespaced configuration such as:

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

Normal users should generally configure their memory provider through the host runtime rather than manually inserting provider internals into every hook.

Provider references:

- [Hindsight](../references/memory-systems/01-hindsight.md)
- [OpenViking](../references/memory-systems/02-openviking.md)
- [Honcho](../references/memory-systems/03-honcho.md)
- [Mem0](../references/memory-systems/04-mem0.md)

## For runtime/agent developers

If you are building an integration, continue with [Agent integration](integrating-an-agent.md). That document covers hook maintenance, cue quality, bounded retrieval, provider namespaces, and trust boundaries—the parts the runtime should handle on the user's behalf.
