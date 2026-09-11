# Quickstart

This guide gets a repository from no MemHooks to a useful, validated routing setup.

## 1. Add a root `MEMHOOKS.md`

Create `MEMHOOKS.md` at the repository root:

```md
---
schema: memhooks/v1
inherits: true

recall_queries:
  - "What architectural decisions, constraints, failures, and unresolved issues should be remembered before changing this project?"
---

Use direct recall first. Retrieve only context that can materially affect the current task.
```

That is already a valid MemHook. You do not need weights, types, roles, entities, or backend-specific metadata unless they are actually useful.

## 2. Add local hooks where context changes

Suppose the repository contains:

```text
repo/
├── MEMHOOKS.md
└── backend/
    ├── MEMHOOKS.md
    └── auth/
        ├── MEMHOOKS.md
        └── refresh.rs
```

An agent working in `backend/auth/` resolves the hooks root to leaf. Broad project context comes first; the auth hook adds the local retrieval cues.

Example `backend/auth/MEMHOOKS.md`:

```md
---
schema: memhooks/v1
inherits: true

recall_queries:
  - query: "What security boundaries must never be violated in authentication?"
    priority: 1.0
    when:
      roles: [reviewer, architect]

  - query: "Why was the current token-refresh flow chosen?"
    priority: 0.85
    connection_types: [causal, temporal]

entities:
  - name: Authentication
    type: CONCEPT
    salience: 0.95

exclude:
  - obsolete OAuth prototype
---
```

## 3. Validate the repository

Until the first crates.io publication is completed, build the CLI from the repository:

```bash
git clone https://github.com/AronAxe/MemHooks.git
cd MemHooks
cargo build --release
```

Then validate a MemHooks-enabled project:

```bash
/path/to/MemHooks/target/release/memhooks validate /path/to/project --all
```

After the crate is published, the shorter installation path is:

```bash
cargo install memhooks
memhooks validate --all
```

A successful validation prints:

```text
MemHooks validation passed with no diagnostics.
```

## 4. Inspect what an agent will actually see

Use `explain` on the directory being worked in:

```bash
memhooks explain backend/auth
```

If the host runtime knows an active role:

```bash
memhooks explain backend/auth --role reviewer
```

This resolves inheritance first and then filters role-targeted queries.

## 5. Connect retrieval to a memory backend

MemHooks itself does not search a memory system. A runtime adapter translates the effective queries into the backend's native retrieval operations.

At minimum, an adapter should:

1. resolve the active directory;
2. load the root-to-leaf hook chain;
3. apply known active roles;
4. retrieve the requested context from the configured memory backend;
5. respect `exclude` and the context budget;
6. preserve priority, salience, and source provenance when useful.

See [Agent integration](integrating-an-agent.md) for the runtime contract.

## 6. Keep hooks small

Good MemHooks questions describe context that could change a future decision:

- "Why was this architecture selected?"
- "Which production failures involved this subsystem?"
- "Which invariants must remain true?"
- "Which approaches were rejected, and why?"

Avoid turning a hook into a README, memory dump, transcript, or generic keyword list. The memory backend stores the facts; MemHooks stores the cues that make those facts retrievable.
