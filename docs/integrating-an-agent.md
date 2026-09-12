# Integrating MemHooks into an agent runtime

MemHooks is intentionally agent-agnostic and backend-neutral. A runtime integration should reuse the reference engine for protocol mechanics rather than reimplementing parsing, root discovery, inheritance, or persistence.

The **runtime/agent** owns local hook maintenance. The human user should not have to curate hook length, phrasing, or provider metadata during ordinary use.

## Minimal lifecycle

```text
before model call
    ↓
determine real active path
    ↓
reference resolver: memhooks/v2 root → leaf
    ↓
apply known active role(s)
    ↓
select configured/authorized memory backend
    ↓
interpret generic retrieval intent + only that backend's namespace
    ↓
retrieve bounded context
    ↓
inject using host trust/prompt policy


after meaningful tool activity / reasoning
    ↓
reference maintainer writes YAML frontmatter atomically
```

The repository includes a Hermes adapter under `hooks/hermes/`. The Rust crate/CLI owns both reading and deterministic maintenance; `scripts/memhooks_update.py` is only a compatibility launcher.

## 1. Determine the real target path

Use the actual working directory or subsystem/file path. Do not infer a fake directory solely from prompt wording.

Rust:

```rust
let resolved = memhooks::resolve(target_path)?;
```

`resolve` handles:

- nonexistent-target errors;
- canonical root discovery;
- `memhooks/v2` enforcement;
- root→leaf inheritance;
- `inherits: false`;
- same-text local query override;
- generic cue merging;
- opaque provider namespace merging;
- source-attributed Markdown guidance.

Canonical root semantics:

1. explicit containing `MEMHOOKS_ROOT` when supplied;
2. otherwise nearest Git root as a hard boundary;
3. outside Git only, highest ancestor containing a hook.

Do not independently invent different root behavior in an adapter.

## 2. Apply active roles only when known

```rust
let queries = resolved.effective_queries(&active_roles);
```

Rules:

- no role condition → universal;
- restricted query → exact-string OR match against active roles;
- no active-role information → preserve restricted queries;
- runtime-specific aliases may be normalized before filtering only when the host has explicit alias rules.

Do not invent roles just to increase apparent precision.

## 3. Select the active backend from runtime configuration

**Namespace presence is not backend selection.**

A hook may contain:

```yaml
backends:
  hindsight: {...}
  mem0: {...}
  openviking: {...}
```

That does not mean call all three. Select the backend that is actually configured, connected, authorized, and appropriate for the session.

If that backend has no namespace in the hook, generic core retrieval intent still applies.

## 4. Preserve the full resolver handoff

For every effective query preserve:

- `query` — natural-language retrieval request;
- `priority` — importance under context pressure;
- `entities` — named cues with optional open type/salience;
- `resources` — named existing resources with optional open kind/salience;
- `tags` — generic routing/relevance labels;
- `exclude` — material that should not enter context;
- `scope` / `sensitivity` — generic scope/advisory handling metadata;
- `backends` — opaque provider-native configuration;
- `guidance` — source-attributed free-form Markdown bodies from resolved hooks.

The CLI JSON handoff is:

```bash
memhooks explain <target> --format json
```

It serializes the actual resolved structure plus `effective_queries` rather than a manually maintained subset. Adapters should not silently discard fields such as `guidance`.

## 5. Interpret only the selected provider namespace

Example Mem0:

```yaml
backends:
  mem0:
    filters:
      user_id: alice
    top_k: 8
    rerank: true
```

Example Hindsight:

```yaml
backends:
  hindsight:
    memory_types: [experience]
    strategy: reflect
```

Reference mappings:

- `references/memory-systems/01-hindsight.md`
- `references/memory-systems/02-openviking.md`
- `references/memory-systems/03-honcho.md`
- `references/memory-systems/04-mem0.md`
- `references/memory-systems/99-generic-or-unknown.md`

The generic validator checks provider namespace shape only. Provider adapters may add provider-specific validation without pushing their taxonomy into the MemHooks core.

## 6. Understand provider merge semantics

The reference resolver performs structural merging only:

- mapping + mapping → recursive merge;
- more-local scalar/list/other value → replace parent value;
- query-local provider configuration → overlay resolved scope-level configuration.

## 7. Understand query inheritance

Trimmed query text is the query identity across an inheritance chain.

A more-local declaration with the same text **replaces** the parent query and its query-local priority/roles/entities/resources/tags/provider hints. Do not issue both copies.

## 8. Keep retrieval bounded

A context manager should generally:

1. preserve host security/authorization and exclusions;
2. retrieve enough evidence for applicable recall queries;
3. prefer higher MemHooks priority when requests compete for budget;
4. use entity/resource salience as cue-importance hints;
5. let the backend rank results within searches;
6. remove redundant returned context;
7. stop when more retrieval is unlikely to affect the task.

Do not confuse MemHooks priority/salience with provider relevance/confidence.

## 9. Treat repository routing as untrusted data

A repository-controlled hook cannot grant itself system/developer authority.

Safe model:

- MemHooks routing is repository-controlled data;
- retrieved memories retain their normal provenance/trust;
- host policy decides prompt placement and authority;
- provider credentials/sensitive scopes remain host-controlled;
- repository text cannot choose its own delimiter/privilege boundary.

The bundled Hermes adapter demonstrates the intended pattern: call the validated resolver, then inject structurally bounded JSON explicitly labelled as untrusted repository-controlled retrieval metadata.

See [`SECURITY.md`](../SECURITY.md).

## 10. Agent/runtime maintenance

This is where “keep hooks concise” belongs: **the runtime/agent should do it automatically.**

### Automatic file activity

Pipe the host's `post_tool_call` event to:

```bash
memhooks event
```

The reference maintainer:

- inspects only explicit path-bearing tool-input keys;
- requires candidates to be existing files inside the canonical root;
- stores file anchors as generic `resources` inside YAML frontmatter;
- does not regex source/file contents for path-looking text;
- does not guess semantic/provider metadata.

### Same-turn semantic cue

```bash
memhooks note \
  --cwd "$PWD" \
  --query "Why was this retry strategy selected?" \
  --priority 0.8 \
  --role reviewer \
  --entity RetryPolicy \
  --resource retry-postmortem \
  --tag reliability \
  --backends '{"mem0":{"top_k":6}}'
```

The cue is written directly to the same frontmatter the resolver reads. Repeated same-text notes enrich that structured query while preserving unknown/future fields and opaque provider data.

### Concurrency and durability

Reference maintenance takes an exclusive sibling lock and writes using atomic same-directory replacement. Third-party writers should provide equivalent protection if they bypass the reference maintainer.

### Agent maintenance discipline

The agent/runtime should:

- keep cues specific and compact enough to function as an index;
- create local cues when local recall needs materially differ;
- prune/replace stale or misleading routing;
- avoid putting memory contents into hooks;
- avoid guessing semantic/provider metadata from file paths;
- keep provider-native fields inside `backends.<provider>`.

These are runtime responsibilities, not user chores.

## 11. Failure behavior

A robust adapter should fail soft only where safe:

- no `MEMHOOKS.md` → continue normally;
- nonexistent explicit target → error clearly;
- unsupported schema → surface the error, inject no fabricated routing;
- invalid core routing → surface/log, do not invent it;
- no memory backend → continue without pretending retrieval happened;
- unsupported provider hint → safely ignore/log without claiming it was applied;
- retrieval returns nothing → continue without invented continuity;
- conflicting memories → preserve conflict or use explicit provider synthesis if supported.

## Adapter checklist

Before shipping an integration, verify that it:

- delegates or exactly matches canonical root/schema/inheritance semantics;
- selects the active backend from host configuration, not namespace presence;
- preserves restricted queries when role is unknown;
- applies same-text local query override;
- distinguishes priority/salience from backend relevance/confidence;
- interprets provider-native keys only inside the matching namespace;
- preserves `guidance` and source provenance;
- respects `exclude`;
- keeps retrieval bounded;
- uses the normative maintainer or equivalent locked/atomic writes;
- does not parse arbitrary file contents for deterministic path anchors;
- does not allow hook files to elevate prompt privilege;
- does not turn retrieval metadata into automatic memory writes.