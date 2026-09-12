# MemHooks

**Mnemonic devices for agents.** 🎣

MemHooks is a filesystem-scoped retrieval-routing protocol for AI agents. It solves a deceptively simple failure mode:

> The memory exists. The agent could use it. The agent never thinks to retrieve it.

MemHooks puts the **cue** near the work. When an agent enters a project area, the applicable `MEMHOOKS.md` files tell the runtime what prior decisions, constraints, failures, fixes, rejected approaches, entities, and resources are worth recalling **here**.

```text
project/
├── MEMHOOKS.md
├── backend/
│   ├── MEMHOOKS.md
│   └── auth/
│       ├── MEMHOOKS.md
│       └── refresh.rs

agent works in backend/auth/
        ↓
resolve root → leaf hooks
        ↓
apply role routing + local overrides
        ↓
hand retrieval intent to the active memory backend
        ↓
retrieve bounded relevant context
        ↓
do the work with the right history in mind
```

## The important distinction

**Memory systems know how to remember. MemHooks tells the agent what to recall here.**

MemHooks does not replace Hindsight, Mem0, OpenViking, Honcho, or another memory system. It sits above them as a backend-neutral routing layer.

Provider-neutral intent lives in the core. Provider-native controls live under namespaced configuration such as:

```yaml
backends:
  mem0:
    top_k: 8
    rerank: true
  hindsight:
    memory_types: [experience]
```

The MemHooks core preserves those provider settings without pretending they are universal concepts.

## Normal users do not curate hook files

In ordinary use, the human user should not be deciding how long a hook should be, which local hook to create, or how to word every recall query.

- **User:** installs/enables MemHooks and configures the host/runtime.
- **Agent/runtime:** creates and maintains retrieval cues.
- **MemHooks engine:** parses, validates, resolves, and maintains the structured routing metadata.
- **Memory backend:** stores and retrieves memories.

See [[Agent Maintenance]] for the actual ownership model.

## Start here

If you want to **use MemHooks**, read [[Getting Started]].

If you want to understand **why it exists**, read [[Why MemHooks]].

If you are building an **agent/runtime integration**, read [[How MemHooks Works]], [[Architecture]], and [[Security Model]].

If you want the schema in approachable language, read [[MEMHOOKS File Reference]].

If something behaves strangely, start at [[FAQ and Troubleshooting]].

## Wiki versus specification

> **This Wiki explains MemHooks. It does not define the protocol.**

The normative protocol contract is the repository's [`references/memhooks-format.md`](https://github.com/AronAxe/MemHooks/blob/main/references/memhooks-format.md). If the Wiki and that file ever disagree, the normative specification wins.

The Wiki is deliberately optimized for understanding, examples, architecture, and practical use; the repository specification is optimized for exact implementation semantics.

## Core ideas in one minute

1. A project directory can contain `MEMHOOKS.md`.
2. Hooks resolve from repository root toward the active path.
3. A Git repository root is a hard containment boundary.
4. `inherits: false` can intentionally cut off ancestors.
5. Recall queries can carry priority and role applicability.
6. More-local declarations can refine or replace broader routing.
7. Generic entities/resources/tags help cue retrieval.
8. Provider-native controls live only under `backends.<provider>`.
9. Markdown bodies may provide retrieval guidance, but structured routing lives in YAML frontmatter.
10. Repository-controlled hook data never grants itself system/developer authority or tool permission.

That is MemHooks: **a small protocol that helps an agent remember the right things at the right place and time.**