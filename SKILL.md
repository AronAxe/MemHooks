---
name: memhooks
description: Directory-scoped memory retrieval routing. Use MEMHOOKS.md files from the workspace root to the active directory to recall the specific past decisions, events, entities, constraints, failures, and context needed before substantive work. Adapts itself to Hindsight, OpenViking, Honcho, or another available memory system without changing the memory backend.
version: 0.3.0
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

> Before substantive work, locate every `MEMHOOKS.md` from the workspace root to the active directory, merge them from broadest to most local, then execute the prescribed bounded retrieval using the memory system currently available to the agent.

A MemHooks file tells you **what to recall here**. The attached memory backend determines **how to retrieve it**.

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

## When to use

Use this skill when:

- the user invokes `/memhooks init` or asks to initialize MemHooks for a project;
- the current workspace or any parent directory contains `MEMHOOKS.md`;
- the user asks to update MemHooks for a project;
- you are about to edit, debug, redesign, delete, or substantially reason about files in a MemHooks-enabled tree;
- a task refers to previous project decisions, failures, constraints, events, or entities that may live in long-term memory.

Do not repeatedly re-run identical retrieval during the same local task unless the working directory, task, or relevant hook changed.

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
- query-local entities supplement the merged top-level entities.

Do not turn the merged hooks into a giant context dump. They are instructions for **targeted retrieval**.

### 3. Identify the available memory system

Inspect the tools, configured memory provider, or environment already available to the agent.

If it matches one of the bundled references, read that reference before retrieval:

1. `references/memory-systems/01-hindsight.md`
2. `references/memory-systems/02-openviking.md`
3. `references/memory-systems/03-honcho.md`

If the backend is different, read `references/memory-systems/99-generic-or-unknown.md` and infer the closest native operations without inventing unsupported capabilities.

### 4. Execute the retrieval intent

Interpret the merged fields as follows:

- `recall_queries`: run these as specific memory searches/questions.
- `memory_types`: use native memory/fact-category filters when the backend exposes them. For Hindsight these are `world`, `experience`, and `observation`.
- `connection_types`: treat `semantic`, `temporal`, `entity`, and `causal` as retrieval emphasis. Use native controls only when they really exist; otherwise sharpen the query appropriately. Do **not** confuse connection type with memory type or entity type.
- `entities`: use named entities as entity filters/graph cues when available. A mapping may include `{name, type}`. Preserve an explicit type when known; do not guess one merely to fill the field.
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
- entities: actual named things, optionally with their own entity type;
- mental models / Knowledge Pages: higher-level synthesized retrieval targets.

Hindsight's public Recall API states that **each selected memory type runs the full four-strategy retrieval pipeline independently**. Therefore a memory category and a connection emphasis are independent dimensions.

### 5. Keep retrieval bounded

Fetch enough context to satisfy the hooks, not the whole memory store. Stop when required queries have useful results and further retrieval is redundant or unrelated.

If a required query returns nothing, note that internally and continue. Do not fabricate continuity.

### 6. Do the actual task

Use the recalled context as ordinary task context. Preserve provenance when the memory system exposes it.

MemHooks should disappear into the workflow: it is successful when the agent simply remembers the right things before acting.

## Maintaining the routing file

A `MEMHOOKS.md` file must evolve with the code or it becomes stale. Prefer the bundled deterministic maintainer wherever the runtime can fire it after tool calls.

`scripts/memhooks_update.py` has three modes:

- `init`: create the root opt-in hook for a repository;
- `event`: inspect a runtime tool-event payload and maintain small file-path recall anchors with **zero LLM calls**;
- `note`: add one concise semantic retrieval question discovered during the current turn, optionally preserving memory category, connection emphasis, and typed entities.

### Deterministic auto-anchors

The `event` path can know which files were touched, but it cannot reliably know whether the relevant memory is `world`, `experience`, or `observation`, nor which graph connection or entity type is semantically correct. **It must not guess.** Auto-anchors therefore remain untyped.

### Same-turn semantic notes

When the current agent has already discovered a durable retrieval cue during normal reasoning, record it with `note`. This does not require another model call.

Simple legacy-compatible note:

```bash
python3 memhooks_update.py note \
  --cwd "$PWD" \
  --query "Why was refresh-token rotation split into two stages?"
```

Typed note when the classifications are genuinely known:

```bash
python3 memhooks_update.py note \
  --cwd "$PWD" \
  --query "Why did the authentication design change after the outage?" \
  --memory-type experience \
  --connection-type causal \
  --connection-type temporal \
  --entity 'Authentication' \
  --entity '{"name":"OpenAI","type":"ORG"}'
```

Repeat `--memory-type`, `--connection-type`, and `--entity` as needed.

The structured note is stored as routing metadata inside the MemHooks notes block. It is still **not the memory itself**.

## Creating or updating `MEMHOOKS.md`

When asked to add or refine MemHooks for a directory:

1. Inspect what that folder/subsystem is responsible for.
2. Identify the past context that would materially change future work there.
3. Write **specific recall questions**, not broad topic labels.
4. Add `memory_types` only when the intended category is genuinely known.
5. Add `connection_types` only when the retrieval relationship is genuinely known and useful.
6. Add important named entities; add entity `type` only when known from the backend, user, or source data.
7. Add existing mental models/Knowledge Pages when they are useful retrieval targets.
8. Add known obsolete approaches to `exclude` when they are likely retrieval traps.
9. Keep the file small. A hook file is a routing index, not memory content.

Never manufacture classifications to make the YAML look complete.

## Adapting to an unlisted memory backend

Do **not** require a bespoke MemHooks plugin.

Use the bundled backend references as worked examples and determine the new system's closest equivalents for direct recall, deeper synthesis, memory categories, entity-aware retrieval, graph/relationship cues, metadata filters, temporal filters, hierarchical summaries, and namespaces.

Then execute the hooks using those native operations.

If you have permission to improve this skill and the mapping is reusable, create a concise new file under `references/memory-systems/` following the style of the existing examples. Do not block the user's task merely because such a file does not yet exist.

## Hard boundary: retrieval only

`MEMHOOKS.md` must **not** itself trigger memory creation, retention, consolidation, deletion, relationship rewriting, directive creation, mental-model creation, or Knowledge Page generation.

Those belong to the memory system's normal lifecycle or a separate explicitly invoked process.

MemHooks may point at existing resources. It does not manufacture them.

## Failure behavior

- `/memhooks init` with no existing root hook: create it using the deterministic initializer.
- `/memhooks init` with an existing root hook: succeed without duplicating it.
- No `MEMHOOKS.md` during ordinary work: continue normally.
- No memory backend/tools: continue normally; do not pretend retrieval occurred.
- Unknown backend: infer the mapping from available tools/docs and the reference examples.
- Search returns nothing: continue without invented context.
- Conflicting memories: use the backend's synthesis/reasoning operation if available, or surface the conflict rather than silently choosing.
- `exclude` conflicts with a parent include: the more local hook wins for the active subtree.

## Verification

Before acting on a MemHooks-enabled subtree, confirm that:

- the complete applicable hook chain was read;
- root-to-leaf inheritance was respected;
- the current memory backend was identified;
- listed queries were translated into native retrieval operations;
- memory category, connection emphasis, and entity type were not conflated;
- no semantic type was guessed merely to populate a field;
- excluded/obsolete material was not injected as current context;
- no memory or synthesized backend object was created or rewritten merely because MemHooks was loaded.
