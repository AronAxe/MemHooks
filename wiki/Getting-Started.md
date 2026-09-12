# Getting Started

This is the shortest path from zero to a working MemHooks project.

## 1. Install the CLI

```bash
cargo install memhooks
```

Verify:

```bash
memhooks --version
```

## 2. Enable MemHooks in a project

From the project root:

```bash
memhooks init .
```

This creates a root `MEMHOOKS.md` using the current `memhooks/v2` schema.

A minimal hook looks roughly like:

```md
---
schema: memhooks/v2
inherits: true
---

# MemHooks

Recall prior decisions, constraints, failures, fixes, rejected approaches,
and unresolved issues concerning this project before substantive changes.
```

Normal users should not have to manually curate this file afterward. See [[Agent Maintenance]].

## 3. Validate the project

```bash
memhooks validate --all
```

A clean result means the reference validator found no errors.

For machine-readable output:

```bash
memhooks validate --all --format json
memhooks validate --all --format sarif
```

## 4. Inspect what an agent would receive

```bash
memhooks explain .
```

For a nested directory:

```bash
memhooks explain backend/auth
```

With a known runtime role:

```bash
memhooks explain backend/auth --role reviewer
```

JSON output is useful for runtime adapters:

```bash
memhooks explain backend/auth --role reviewer --format json
```

## 5. Add a durable retrieval cue

Agents/runtimes can write a semantic cue through the same reference engine that later reads it:

```bash
memhooks note \
  --cwd "$PWD" \
  --query "Why was refresh-token rotation split into two stages, and what alternatives were rejected?" \
  --priority 0.9 \
  --role reviewer \
  --tag security
```

The cue is written into YAML frontmatter as a real `recall_queries` entry. There is no hidden second routing database in the Markdown body.

## 6. Automatic file activity

A runtime may feed a structured post-tool event to:

```bash
memhooks event
```

The event path extracts only explicit path-like fields supplied by the host tool payload, verifies that they resolve to real files inside the project boundary, and maintains conservative file-resource anchors.

The deterministic maintainer deliberately does **not** guess:

- semantic priority;
- future agent roles;
- entity meaning;
- provider-specific search controls;
- memory content.

Those require actual semantic knowledge from the active agent/runtime.

## 7. Connect a memory backend

MemHooks itself does not store memories. The host runtime selects an authorized backend and interprets only that backend's namespace.

For example:

```yaml
backends:
  mem0:
    filters:
      user_id: project-agent
    top_k: 8

  hindsight:
    bank: project-memory
```

Namespace presence does not mean “call every backend.” The runtime decides which backend is active.

See [[Memory Backends]].

## Hermes users

For Hermes/Hermes Desktop, continue with [[Hermes Integration]]. The Hermes adapter asks the reference resolver for a validated routing plan instead of independently reimplementing root discovery or inheritance.

## Next

- [[How MemHooks Works]] — mental model and lifecycle
- [[MEMHOOKS File Reference]] — field-by-field explanation
- [[CLI Cookbook]] — practical commands
- [[Security Model]] — trust and containment boundaries