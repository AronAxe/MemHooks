# Integrating MemHooks into an agent runtime

MemHooks is intentionally agent-agnostic and backend-neutral. A runtime integration combines the reference resolver with the host's existing memory, trust, and context systems.

The **runtime/agent** owns local hook maintenance. The human user should not have to manually curate hook length, phrasing, or provider metadata during ordinary use.

## Minimal lifecycle

```text
before model call
    ↓
determine real active directory
    ↓
resolve memhooks/v2 root → leaf
    ↓
apply known active role(s)
    ↓
select configured/authorized memory backend
    ↓
translate backend-neutral core intent
    + interpret only that backend's namespace
    ↓
retrieve bounded context
    ↓
inject using host trust/prompt policy


after meaningful work/tool activity (optional)
    ↓
agent/runtime maintains concise retrieval cues
```

The repository includes a Hermes implementation under `hooks/hermes/` and a deterministic maintainer under `scripts/memhooks_update.py`.

## 1. Determine the actual target directory

Use the real working directory or the directory containing the subsystem/file being worked on. Do not infer a fake path from prompt wording.

Rust:

```rust
let resolved = memhooks::resolve(target_path)?;
```

`resolve` handles:

- root discovery;
- `memhooks/v2` enforcement;
- root→leaf inheritance;
- `inherits: false`;
- generic cue merging;
- opaque provider namespace merging;
- source provenance.

## 2. Apply active roles only when known

```rust
let queries = resolved.effective_queries(&active_roles);
```

Rules:

- no role condition → universally applicable;
- restricted query → exact-string OR match against active roles;
- no active-role information → preserve restricted queries;
- normalize host-specific aliases before passing roles to the resolver if the runtime has explicit alias rules.

Do not invent roles just to increase apparent routing precision.

## 3. Select the active backend from runtime configuration

**Namespace presence is not backend selection.**

A hook may contain several namespaces:

```yaml
backends:
  hindsight: {...}
  mem0: {...}
  openviking: {...}
```

That does not mean the agent should call all three systems.

The host runtime should determine which memory backend is configured, connected, authorized, and appropriate for the session. Then interpret only the relevant namespace.

If the configured backend has no namespace in the hook, the generic core retrieval intent still applies.

## 4. Translate backend-neutral core intent

For each effective query preserve:

- `query` — natural-language retrieval request;
- `priority` — importance under context pressure;
- `entities` — named retrieval cues with optional open type/salience;
- `resources` — named existing resources with optional open kind/salience;
- `tags` — generic routing/relevance labels;
- `exclude` — material that should not enter working context;
- `scope` / `sensitivity` — generic scope/advisory handling metadata.

Do not reinterpret one provider's concepts as core fields.

## 5. Interpret the selected provider namespace

Provider-native configuration is available in the effective query's `backends` map.

Example:

```yaml
backends:
  mem0:
    filters:
      user_id: alice
    top_k: 8
    rerank: true
```

A Mem0 adapter may translate that to Mem0's current search API.

Likewise:

```yaml
backends:
  hindsight:
    memory_types: [experience]
    strategy: reflect
```

is interpreted only by a Hindsight adapter.

Reference mappings:

- `references/memory-systems/01-hindsight.md`
- `references/memory-systems/02-openviking.md`
- `references/memory-systems/03-honcho.md`
- `references/memory-systems/04-mem0.md`
- `references/memory-systems/99-generic-or-unknown.md`

### Provider validation

The generic validator checks only that each provider namespace is a mapping/object. Provider adapters may add stricter validation for their own configuration.

Do not make the generic protocol depend on every provider SDK just to validate provider-native keys.

## 6. Understand namespace merge semantics

The reference resolver performs structural merging only:

- mapping + mapping → recursive merge;
- a more local scalar replaces the parent scalar;
- a more local list replaces the parent list;
- query-local provider configuration overlays resolved scope-level configuration.

This is intentionally less opinionated than provider semantics.

If a provider wants additive-list behavior, its adapter can expose a provider-native structure that expresses that explicitly rather than relying on MemHooks to guess.

## 7. Keep retrieval bounded

MemHooks reduces forgotten context; it is not permission to maximize prompt size.

A context manager should generally:

1. preserve host security/authorization and exclusions;
2. retrieve enough evidence for applicable recall queries;
3. prefer higher MemHooks priority when requests compete for budget;
4. use entity/resource salience as cue-importance hints;
5. let the backend rank results within a particular search;
6. remove redundant returned context;
7. stop when additional retrieval is unlikely to affect the task.

Do not confuse MemHooks priority/salience with provider relevance/confidence scores.

## 8. Prompt placement remains host-controlled

A repository-controlled hook cannot grant itself system/developer authority.

Safe model:

- MemHooks is retrieval metadata;
- retrieved memories retain their ordinary provenance/trust;
- host policy decides prompt placement and authority;
- provider credentials and sensitive scopes remain outside untrusted hook files where appropriate.

## 9. Agent/runtime maintenance

This is where “keep hooks concise” belongs: **the runtime/agent should do it automatically.**

Deterministic path maintenance:

```bash
python3 scripts/memhooks_update.py event
```

Same-turn semantic cue already discovered by the active model:

```bash
python3 scripts/memhooks_update.py note \
  --cwd "$PWD" \
  --query "Why was this retry strategy selected?" \
  --priority 0.8 \
  --role reviewer \
  --entity RetryPolicy \
  --resource retry-postmortem \
  --tag reliability \
  --backends '{"mem0":{"top_k":6}}'
```

The maintainer deep-merges repeated provider mappings and enriches generic metadata instead of erasing previous routing context.

### Agent maintenance rules

The agent/runtime should:

- keep cues specific and compact enough to function as an index;
- create local cues when the local subsystem has materially different recall needs;
- prune/replace stale or misleading routing when discovered;
- avoid writing memory contents into the hook;
- avoid guessing semantic metadata or provider controls from file paths alone;
- keep provider-native fields inside `backends.<provider>`.

These are runtime responsibilities, not instructions for the end user to manually curate the repository.

## 10. Failure behavior

A robust adapter should fail soft where safe:

- no `MEMHOOKS.md` → continue normally;
- unsupported schema → surface the error; do not silently reinterpret;
- invalid core routing → surface/log and do not fabricate it;
- no memory backend → continue without claiming retrieval happened;
- unsupported provider hint → ignore/translate safely and report in diagnostics/logging when useful;
- retrieval returns nothing → continue without invented continuity;
- conflicting memories → preserve conflict or use an explicit provider synthesis operation if supported.

## Adapter checklist

Before shipping an integration, verify that it:

- uses reference v2 merge semantics;
- selects the active backend from host configuration rather than namespace presence;
- does not silently discard restricted queries when role is unknown;
- distinguishes priority/salience from backend relevance/confidence;
- interprets provider-native keys only inside the matching namespace;
- respects `exclude`;
- preserves source/provenance where possible;
- keeps retrieval bounded;
- treats hook maintenance as an agent/runtime concern;
- does not allow hook files to elevate their own prompt privilege;
- does not turn retrieval metadata into automatic memory writes.
