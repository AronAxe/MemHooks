# Integrating MemHooks into an agent runtime

MemHooks is intentionally agent-agnostic. An integration is a thin adapter around the reference resolver plus the host runtime's existing memory and context systems.

## Minimal lifecycle

A runtime integration needs two conceptual moments:

```text
before model call
    ↓
resolve MEMHOOKS.md for active directory
    ↓
apply active role(s) if known
    ↓
translate applicable recall queries to memory backend
    ↓
retrieve bounded context
    ↓
inject that context using the host's own trust/prompt policy


after meaningful work/tool activity (optional)
    ↓
maintain small retrieval cues without writing memory content
```

The repository includes a Hermes implementation under `hooks/hermes/` and a zero-LLM maintainer under `scripts/memhooks_update.py`.

## 1. Determine the target directory

Use the actual working directory or the directory containing the file/subsystem being worked on. Do not infer a fake directory solely from the prompt.

Call the reference resolver:

```rust
let resolved = memhooks::resolve(target_path)?;
```

The resolver handles root discovery, inheritance, `inherits: false`, list accumulation, scalar locality, and source provenance.

## 2. Supply active roles only when the runtime knows them

Examples of runtime/project roles might include `reviewer`, `architect`, `refactor`, or `feature_dev`. MemHooks does not define a global role taxonomy.

```rust
let queries = resolved.effective_queries(&active_roles);
```

Rules:

- no role condition means the query applies universally;
- role-restricted queries use exact-string OR matching;
- an empty active-role list preserves all queries;
- runtime-specific aliases should be normalized by the adapter before filtering.

Do not invent an active role just to make routing look more precise.

## 3. Translate fields to the memory backend

For every effective query, preserve the intent:

- `query` — the natural-language retrieval request;
- `priority` — importance under context pressure, not a relevance/confidence score;
- `memory_types` — memory-category filter/hint when supported;
- `connection_types` — semantic/temporal/entity/causal emphasis;
- `entities` — retrieval cues, optionally with type and salience;
- `exclude` — material that should not enter current context;
- `bank` — namespace/bank/session hint when the backend has an equivalent;
- `mental_models` / `knowledge_pages` — existing synthesized retrieval targets.

Use native backend controls only when they genuinely exist. Otherwise translate the intent into natural-language retrieval and post-filtering rather than fabricating unsupported API parameters.

See `references/memory-systems/` for worked mappings.

## 4. Keep retrieval bounded

MemHooks is meant to reduce forgotten context, not to maximize prompt size.

A context manager should generally:

1. preserve hard exclusions and host security policy;
2. retrieve enough evidence to answer each applicable recall query;
3. prefer explicitly higher-priority queries when budget forces a choice;
4. use entity salience as a cue-importance hint;
5. remove duplicate/redundant returned memories;
6. stop when further retrieval is unlikely to change the current task.

The protocol intentionally does not prescribe one universal scoring formula.

## 5. Prompt placement remains host-controlled

A `MEMHOOKS.md` file does **not** grant itself system-level prompt privilege. Repositories may be untrusted. The runtime decides where recalled context belongs and how much authority it has.

Safe default:

- treat MemHooks as retrieval metadata;
- treat retrieved memories according to their normal trust/provenance;
- keep actual host policy and security boundaries outside repository-controlled files.

## 6. Optional maintenance

The bundled maintainer can keep retrieval cues fresh without a separate LLM call:

```bash
python3 scripts/memhooks_update.py event
```

for runtime tool events, and:

```bash
python3 scripts/memhooks_update.py note \
  --cwd "$PWD" \
  --query "Why was this retry strategy selected?" \
  --priority 0.8 \
  --role reviewer
```

for same-turn semantic cues.

Maintenance writes routing metadata only. It must not create, rewrite, consolidate, or delete memory-backend facts merely because a hook exists.

## 7. Failure behavior

A robust adapter should fail soft:

- no `MEMHOOKS.md` → continue normally;
- parse/validation error → surface/log it and avoid inventing routing;
- no memory backend → continue without pretending retrieval happened;
- unknown backend feature → use the nearest safe native operation;
- retrieval returns nothing → continue without fabricated continuity;
- conflicting memories → preserve the conflict or use the backend's explicit synthesis operation if one exists.

## Adapter checklist

Before shipping an integration, verify that it:

- uses the reference merge semantics;
- does not silently discard role-restricted queries when role is unknown;
- distinguishes query priority from backend similarity/confidence;
- distinguishes entity salience from query priority;
- respects `exclude`;
- preserves source/provenance when possible;
- keeps retrieval bounded;
- does not allow a hook file to elevate its own prompt privilege;
- does not turn retrieval metadata into automatic memory writes.
