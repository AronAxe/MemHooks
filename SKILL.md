---
name: memhooks
description: Directory-scoped memory retrieval routing. Use MEMHOOKS.md files from the workspace root to the active directory to recall and prioritize the specific past decisions, events, entities, constraints, failures, and context needed before substantive work. Supports weighted retrieval and role-based routing and adapts itself to Hindsight, OpenViking, Honcho, or another available memory system without changing the memory backend.
version: 0.4.0
author: Aron Bijl
license: MIT
compatibility: Agent Skills / agentskills.io; Hermes Agent and Hermes Desktop; other skill-capable agents with filesystem access and optional memory tools.
metadata:
  hermes:
    tags: [memory, context, retrieval, continuity, coding, agent-memory]
    category: productivity
---

# MemHooks

MemHooks is a **retrieval-routing convention**, not a memory system.

Core rule:

> Before substantive work, locate every `MEMHOOKS.md` from the workspace root to the active directory, merge them from broadest to most local, apply any known role conditions, then execute the prescribed bounded retrieval using the memory system currently available to the agent.

A MemHooks file tells you **what to recall here, how important that recall is, and when it applies**. The attached memory backend determines **how to retrieve it**.

## Hermes slash command: `/memhooks init`

Hermes automatically exposes installed skills as slash commands. Therefore this skill's project bootstrap is:

```text
/memhooks init
```

Treat `init` as a special bootstrap instruction, not as a normal memory-retrieval request.

When invoked as `/memhooks init`:

1. Resolve the target project from the active workspace/backend working directory.
2. If an explicit path follows `init`, use that path instead when accessible.
3. Resolve `scripts/memhooks_update.py` relative to this installed skill directory.
4. Run its deterministic initializer:

   ```bash
   python3 scripts/memhooks_update.py init <target>
   ```

5. Verify that the target Git/repository root now contains `MEMHOOKS.md`.
6. Return a short confirmation that MemHooks is enabled for that project.

Do **not** perform a separate memory-retrieval pass merely because `init` was invoked.

## Deterministic loader mode

When the bundled Hermes `pre_llm_call` shell hook is installed, it has already walked the actual working directory root → leaf and injected the applicable files into the current user turn before this model call.

Treat the injected `[MemHooks — deterministic pre-LLM retrieval routing]` block as authoritative routing input and execute its requested memory retrieval before substantive work.

The loader is in `hooks/hermes/memhooks_pre_llm.py`. It reads files locally and performs no model call of its own.

## Reference resolver / validator

v0.4.0 includes the Rust reference implementation. When available, prefer it for validation and for explaining effective inherited routing:

```bash
memhooks validate --all
memhooks explain path/to/subsystem
memhooks explain path/to/subsystem --role reviewer
```

The library/CLI parses and resolves hooks; it does not contact the memory backend and does not execute arbitrary commands.

## When to use

Use this skill when:

- the user invokes `/memhooks init` or asks to initialize MemHooks for a project;
- the current workspace or any parent directory contains `MEMHOOKS.md`;
- the user asks to update MemHooks for a project;
- you are about to edit, debug, redesign, delete, or substantially reason about files in a MemHooks-enabled tree;
- a task refers to previous project decisions, failures, constraints, events, or entities that may live in long-term memory.

Do not repeatedly re-run identical retrieval during the same local task unless the working directory, task, active role, or relevant hook changed.

## Procedure

### 1. Locate the hook chain

Determine the workspace/repository root and active working directory.

Find `MEMHOOKS.md` at each directory level from root to the active directory. Read them root to leaf.

### 2. Merge root to leaf

Follow `references/memhooks-format.md`.

Default behavior:

- parent hooks are inherited;
- deeper hooks add specificity;
- exact duplicate list entries are de-duplicated;
- a local `inherits: false` cuts off inheritance above that file;
- the most local scalar value wins when scalars conflict;
- query-local `memory_types` and `connection_types` override scope-wide defaults for that query;
- query-local entities supplement the merged top-level entities;
- `priority` and `when` remain attached to the structured query that declared them;
- `salience` remains attached to its structured entity;
- role filtering is applied after inheritance is resolved when active-role information is available.

Do not turn the merged hooks into a giant context dump. They are instructions for **targeted retrieval**.

### 3. Apply role routing when roles are known

A structured query may contain:

```yaml
when:
  roles: [reviewer, architect]
```

Role names are project/runtime-defined open strings.

- No `when.roles` (or an empty list) means the query is unrestricted.
- If one or more active roles are known, a restricted query applies when **any** declared role exactly matches any active role.
- If the runtime has no concept of active roles, preserve the query and defer filtering rather than inventing a role.
- Normalize runtime-specific role aliases before applying MemHooks rather than silently changing the file's semantics.

### 4. Identify the available memory system

Inspect the tools, configured memory provider, or environment already available to the agent.

If it matches one of the bundled references, read that reference before retrieval:

1. `references/memory-systems/01-hindsight.md`
2. `references/memory-systems/02-openviking.md`
3. `references/memory-systems/03-honcho.md`

If the backend is different, read `references/memory-systems/99-generic-or-unknown.md` and infer the closest native operations without inventing unsupported capabilities.

### 5. Execute the retrieval intent

Interpret the merged fields as follows:

- `recall_queries`: run these as specific memory searches/questions.
- `priority`: optional `0.0..1.0` importance of a structured recall request under a finite context budget. Prefer retaining higher-priority recall results when lower-priority context must be dropped. Do not confuse it with semantic similarity, confidence, or truth probability. If omitted, do not fabricate an explicit numeric priority.
- `when.roles`: optional role applicability for a structured recall query as described above.
- `memory_types`: use native memory/fact-category filters when the backend exposes them. For Hindsight these are `world`, `experience`, and `observation`.
- `connection_types`: treat `semantic`, `temporal`, `entity`, and `causal` as retrieval emphasis. Use native controls only when they really exist; otherwise sharpen the query appropriately. Do **not** confuse connection type with memory type or entity type.
- `entities`: use named entities as entity filters/graph cues when available. A mapping may include `{name, type, salience}`. Preserve an explicit type/salience when known; do not guess them merely to fill fields.
- entity `salience`: optional `0.0..1.0` importance of that entity as a retrieval cue. It is distinct from query priority and backend relevance/confidence.
- `mental_models`: retrieve named existing standing answers if the backend has that concept and they directly cover the task.
- `knowledge_pages`: retrieve named established pages/synthesized resources if the backend has an equivalent. Do not create or update them here.
- `tags`: use native tag/metadata filtering where available; otherwise treat them as relevance hints.
- `exclude`: prevent obsolete or unwanted memories from entering working context. Use native negative filters if available; otherwise post-filter results.
- `bank`: use the requested memory namespace only if that concept exists and the agent is authorized to access it.
- free-form Markdown below the frontmatter: treat as retrieval guidance, especially instructions about when to use shallow recall versus deeper synthesis.

### Hindsight-specific invariant

For Hindsight, do not flatten the ontology:

- memory categories: `world | experience | observation`;
- connection classes: `semantic | temporal | entity | causal`;
- entities: actual named things, optionally with their own entity type and MemHooks salience hint;
- mental models / Knowledge Pages: higher-level synthesized retrieval targets;
- query priority / role applicability: MemHooks routing metadata, not Hindsight memory types.

Hindsight's public Recall API states that **each selected memory type runs the full four-strategy retrieval pipeline independently**. Therefore a memory category and a connection emphasis are independent dimensions.

### 6. Keep retrieval bounded

Fetch enough context to satisfy the applicable hooks, not the whole memory store. When a context limit forces a choice, use explicit query priority and entity salience as routing hints while preserving hard constraints and exclusions.

Stop when required queries have useful results and further retrieval is redundant or unrelated.

If a required query returns nothing, note that internally and continue. Do not fabricate continuity.

### 7. Do the actual task

Use the recalled context as ordinary task context. Preserve provenance when the memory system exposes it.

MemHooks should disappear into the workflow: it is successful when the agent simply remembers the right things before acting.

## Maintaining the routing file

A `MEMHOOKS.md` file must evolve with the code or it becomes stale. Prefer the bundled deterministic maintainer wherever the runtime can fire it after tool calls.

`scripts/memhooks_update.py` has three modes:

- `init`: create the root opt-in hook for a repository;
- `event`: inspect a runtime tool-event payload and maintain small file-path recall anchors with **zero LLM calls**;
- `note`: add one concise semantic retrieval question discovered during the current turn, optionally preserving priority, applicable roles, memory category, connection emphasis, and typed/salient entities.

### Deterministic auto-anchors

The `event` path can know which files were touched, but it cannot reliably know semantic priority, agent-role applicability, whether the relevant memory is `world`, `experience`, or `observation`, which graph connection matters, or what entity type/salience is correct. **It must not guess.** Auto-anchors therefore remain semantically untyped/unweighted/unrouted.

### Same-turn semantic notes

When the current agent has already discovered a durable retrieval cue during normal reasoning, record it with `note`. This does not require another model call.

Simple legacy-compatible note:

```bash
python3 scripts/memhooks_update.py note \
  --cwd "$PWD" \
  --query "Why was refresh-token rotation split into two stages?"
```

Routed/weighted note when the metadata is genuinely known:

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

Repeat `--role`, `--memory-type`, `--connection-type`, and `--entity` as needed. `--priority` replaces the previous explicit priority for the same stored query; repeated role/category/connection/entity metadata enriches rather than erases the existing cue.

The structured note is stored as routing metadata inside the MemHooks notes block. It is still **not the memory itself**.

## Creating or updating `MEMHOOKS.md`

When asked to add or refine MemHooks for a directory:

1. Inspect what that folder/subsystem is responsible for.
2. Identify the past context that would materially change future work there.
3. Write **specific recall questions**, not broad topic labels.
4. Add `priority` only when the relative importance under context pressure is meaningful; do not sprinkle arbitrary numbers everywhere.
5. Add `when.roles` only when the query genuinely applies to particular project/runtime roles.
6. Add `memory_types` only when the intended category is genuinely known.
7. Add `connection_types` only when the retrieval relationship is genuinely known and useful.
8. Add important named entities; add entity `type` and `salience` only when known/meaningful.
9. Add existing mental models/Knowledge Pages when they are useful retrieval targets.
10. Add known obsolete approaches to `exclude` when they are likely retrieval traps.
11. Keep the file small. A hook file is a routing index, not memory content.

Never manufacture classifications or weights to make the YAML look complete.

## Adapting to an unlisted memory backend

Do **not** require a bespoke MemHooks plugin.

Use the bundled backend references as worked examples and determine the new system's closest equivalents for direct recall, deeper synthesis, memory categories, entity-aware retrieval, graph/relationship cues, metadata filters, temporal filters, hierarchical summaries, namespaces, and context-budget prioritization.

Then execute the hooks using those native operations.

If you have permission to improve this skill and the mapping is reusable, create a concise new file under `references/memory-systems/` following the style of the existing examples. Do not block the user's task merely because such a file does not yet exist.

## Hard boundary: retrieval only

`MEMHOOKS.md` must **not** itself trigger memory creation, retention, consolidation, deletion, relationship rewriting, directive creation, mental-model creation, Knowledge Page generation, or arbitrary command execution.

Those belong to the memory system's normal lifecycle or a separate explicitly invoked process.

MemHooks may point at existing resources. It does not manufacture them.

## Failure behavior

- `/memhooks init` with no existing root hook: create it using the deterministic initializer.
- `/memhooks init` with an existing root hook: succeed without duplicating it.
- No `MEMHOOKS.md` during ordinary work: continue normally.
- No memory backend/tools: continue normally; do not pretend retrieval occurred.
- Unknown backend: infer the mapping from available tools/docs and the reference examples.
- Unknown active role: do not fabricate one; preserve role-restricted queries for a later role-aware consumer.
- Search returns nothing: continue without invented context.
- Conflicting memories: use the backend's synthesis/reasoning operation if available, or surface the conflict rather than silently choosing.
- Invalid priority/salience or malformed routing: treat it as a validation problem; do not silently reinterpret out-of-range values.

## Verification

Before acting on a MemHooks-enabled subtree, confirm that:

- the complete applicable hook chain was read;
- root-to-leaf inheritance was respected;
- active roles were applied only when actually known;
- query priority and entity salience remained distinct;
- the current memory backend was identified;
- applicable queries were translated into native retrieval operations;
- memory category, connection emphasis, entity type, priority, and salience were not conflated;
- no semantic type, weight, or role was guessed merely to populate a field;
- excluded/obsolete material was not injected as current context;
- no memory or synthesized backend object was created or rewritten merely because MemHooks was loaded.
