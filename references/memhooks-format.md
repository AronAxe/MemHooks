# `MEMHOOKS.md` format — memhooks/v2

A `MEMHOOKS.md` file is Markdown with YAML frontmatter. **YAML frontmatter is the sole normative store for structured retrieval routing.** The optional Markdown body is source-attributed free-form retrieval guidance; it is not a second location for structured queries or machine-maintained routing records.

`memhooks/v2` deliberately separates **backend-neutral retrieval intent** from **provider-native controls**.

The core protocol does not define Hindsight memory types, Mem0 filters, Honcho controls, OpenViking hierarchy controls, or equivalent provider vocabulary. Those belong under `backends.<provider>` and are interpreted only by the corresponding authorized runtime adapter.

The MemHooks package version and protocol schema version are separate. MemHooks package **v0.5.1** implements `memhooks/v2`.

## Canonical shape

```md
---
schema: memhooks/v2
inherits: true
scope: backend/auth

recall_queries:
  - "What architectural decisions govern this subsystem?"

  - query: "Why did the authentication design change after the outage?"
    priority: 1.0
    when:
      roles: [reviewer, architect]
    entities:
      - Authentication
    resources:
      - name: auth-postmortem
        kind: postmortem
        salience: 0.9
    tags: [security]
    backends:
      mem0:
        top_k: 8
        rerank: true
      hindsight:
        memory_types: [experience]
        connection_types: [causal, temporal]
        strategy: reflect

entities:
  - name: Authentication
    type: CONCEPT
    salience: 0.95

resources:
  - auth-architecture

tags: [backend]
exclude:
  - obsolete OAuth prototype

sensitivity: private

backends:
  mem0:
    filters:
      user_id: auth-agent
  hindsight:
    bank: project-memory
---

Use concrete recall first. Use provider-specific synthesis only when the active
backend exposes it and the task actually requires it.
```

## Required field

### `schema`

```yaml
schema: memhooks/v2
```

The v0.5.x reference resolver accepts `memhooks/v2`. Unsupported schemas must fail clearly rather than being silently reinterpreted by either the resolver or bundled runtime adapters.

## Core fields

### `inherits`

Optional boolean, default `true`.

```yaml
inherits: false
```

A hook with `inherits: false` cuts off ancestor hooks above it for the target subtree. Provider-native keys named `inherits` inside `backends.<provider>` have no effect on core inheritance.

### `scope`

Optional backend-neutral descriptive string. More-local values replace parent values.

### `recall_queries`

A list of concrete questions worth asking memory when work occurs in this scope.

Simple form:

```yaml
recall_queries:
  - "What failures have happened in this subsystem before?"
```

Structured form:

```yaml
recall_queries:
  - query: "Why was token rotation split into two stages?"
    priority: 0.9
    when:
      roles: [reviewer]
    entities:
      - Authentication
    resources:
      - auth-postmortem
    tags: [security]
    backends:
      mem0:
        top_k: 6
```

A structured query may contain:

| Field | Type | Meaning |
|---|---|---|
| `query` | string | required natural-language retrieval request |
| `priority` | number `0.0..1.0` | optional importance under context pressure |
| `when.roles` | list of strings | optional role applicability |
| `entities` | entity list | query-local entity cues |
| `resources` | resource list | query-local existing resources |
| `tags` | string list | query-local neutral routing labels |
| `backends` | mapping | query-local provider-native hints |

#### Query identity and local override

The **trimmed query text is the query identity across an inheritance chain**.

If a parent and child declare the same trimmed query text, the more-local declaration **replaces the parent declaration and all of its query-local metadata**. The resolved plan contains that question once.

This prevents an adapter from issuing the same retrieval twice with contradictory priorities, roles, or provider hints. Within one file, repeated query text remains a validation warning.

### `priority`

`priority` belongs to a recall request. It means: **if context cannot satisfy every applicable request equally, preserve higher-priority requests first.**

It does not mean semantic similarity, backend score, truth probability, memory confidence, or a mandated universal ranking multiplier. If omitted, no numeric default is imposed by the protocol.

### `when.roles`

Role names are open project/runtime-defined strings.

When active roles are known, a restricted query applies if any listed role exactly matches an active role. A query without `when.roles` applies universally. If the runtime has no active-role concept, role-restricted queries are preserved rather than silently discarded.

### `entities`

Entities are named retrieval cues.

```yaml
entities:
  - Authentication
  - name: OpenAI
    type: ORG
    salience: 0.9
```

`type` is an optional open string. `salience`, when present, is `0.0..1.0` and expresses importance of that entity as a retrieval cue. Both remain distinct from provider relevance/confidence scores.

### `resources`

Resources are named existing things that may deserve retrieval/read access.

```yaml
resources:
  - auth-architecture
  - name: outage-postmortem
    kind: postmortem
    salience: 0.9
```

`kind` is an optional open string. `salience`, when present, is `0.0..1.0` and expresses cue importance. Provider-native resource identifiers/modes remain provider-owned under `backends.<provider>`.

### `tags`

Backend-neutral routing/relevance labels. An adapter may map them to real native metadata filters when one exists; otherwise they remain generic hints.

### `exclude`

Material that should not enter current working context when matched by retrieval. Adapters may use native negative filters or post-filter results.

### `sensitivity`

Optional advisory handling metadata. It does not override host authorization/security policy.

## Provider namespaces: `backends`

`backends` is a mapping from provider/adapter name to an **opaque mapping/object**.

```yaml
backends:
  mem0:
    filters:
      user_id: alice
    top_k: 8
    threshold: 0.15
    rerank: true

  hindsight:
    bank: project-memory
    memory_types: [experience]
    connection_types: [causal, temporal]
```

The MemHooks core preserves provider namespaces, validates only their outer mapping shape, structurally merges them, and exposes them to adapters. It does not validate or interpret provider-internal keys.

### Provider namespace merge semantics

1. mapping/object + mapping/object → recursive merge;
2. a more-local scalar/sequence/other value replaces the parent value;
3. query-local `backends.<provider>` overlays resolved scope-level provider configuration using the same rules.

MemHooks does not concatenate provider lists because the generic core cannot know provider semantics.

## Canonical root and containment

The resolver, maintainer, and bundled runtime adapters must use **one root semantic**:

1. if the host explicitly supplies `MEMHOOKS_ROOT` and the target is contained by it, that root wins;
2. otherwise the nearest ancestor containing `.git` is a **hard project boundary**;
3. only when the target is outside Git may the implementation fall back to the highest ancestor containing `MEMHOOKS.md`;
4. the maintainer must never climb above a Git root to capture a repository that has not explicitly enabled MemHooks at that root.

Therefore an unrelated `~/MEMHOOKS.md` cannot receive writes from a Git repository whose own root is not MemHooks-enabled.

All maintained file/resource anchors must canonicalize to existing files contained by the chosen root.

## Root-to-leaf resolution

For:

```text
/repo/MEMHOOKS.md
/repo/backend/MEMHOOKS.md
/repo/backend/auth/MEMHOOKS.md
```

an agent working in `/repo/backend/auth/` resolves hooks root → leaf.

Core semantics:

- `inherits: false` clears ancestors above that hook;
- `scope` and `sensitivity` use the most-local specified value;
- entities/resources/tags/exclusions accumulate with exact duplicate suppression;
- a same-text more-local recall query replaces its parent query as defined above;
- priority and role conditions remain attached to their query;
- query-local entities/resources/tags supplement resolved top-level cues;
- role filtering happens after resolution when active roles are known;
- `backends` follows the structural merge rules above;
- free-form bodies accumulate as **source-attributed guidance**.

## Structured routing versus Markdown guidance

There is deliberately one structured routing data model:

```text
YAML frontmatter → parser/resolver → explain/adapter handoff
```

Machine-maintained semantic cues and deterministic file anchors **must be written into frontmatter**, normally as `recall_queries` plus generic `resources`/tags. A maintainer must not create a second JSON/Markdown routing store in the body.

The Markdown body is guidance only. It remains part of `ResolvedHook.guidance`, retains source provenance, and must be included in complete resolver/adapter handoffs such as `memhooks explain --format json`.

## Reference maintenance semantics

The reference CLI/library provides the normative maintenance path:

```bash
memhooks init .
memhooks note --cwd . --query "Why did auth change?" --priority 0.9
# one post_tool_call JSON event on stdin:
memhooks event
```

Maintenance requirements:

- write structured cues to YAML frontmatter through the same model used by the resolver;
- preserve unknown/future structured fields and opaque backend configuration when enriching a cue;
- automatic path anchors may inspect explicit path-bearing tool-input fields, not arbitrary file contents/free text;
- automatic anchors must refer only to existing files inside the canonical root;
- concurrent read-modify-write operations must use a lock;
- writes must use atomic same-directory replacement so interruption cannot truncate the hook.

The bundled Python maintainer script may exist as a compatibility launcher, but must not implement an independent parser or persistence format.

## Validation and source locations

The reference validator checks core syntax/semantics such as:

- schema is `memhooks/v2`;
- query/entity/resource names are non-empty;
- priority/salience are within `0.0..1.0`;
- role names are non-empty;
- backend namespace names are non-empty;
- backend namespaces contain mappings/objects;
- duplicate core cues and recall/exclude contradictions;
- unknown/malformed core fields;
- target paths exist when explicitly requested.

Diagnostics for structured YAML entries should point to the actual node that caused the finding, not the first textual occurrence of a key elsewhere in the file. This is required for trustworthy human diagnostics and SARIF annotations.

The validator deliberately does not validate arbitrary provider-native configuration inside `backends.<provider>`.

## SARIF handoff

SARIF emitted by the reference CLI uses repository/validation-root-relative artifact URIs so code-scanning systems can map alerts back to files. The tool driver should expose diagnostic rule metadata/help for emitted `MHxxx` findings.

## Runtime adapter handoff

A complete adapter handoff includes:

- resolved root/target/sources;
- effective queries after optional role filtering;
- entities/resources/tags/exclusions;
- resolved provider namespaces;
- source-attributed Markdown guidance.

The bundled Hermes adapter delegates root discovery, schema validation, inheritance, and handoff construction to the reference resolver. It does not maintain a second YAML parser.

## Security and trust boundary

A hook file is repository-controlled retrieval metadata. It does not grant itself execution or prompt privilege.

Loading a hook must not by itself:

- execute arbitrary repository-declared commands;
- write, alter, consolidate, or delete memories;
- elevate repository text to system/developer authority;
- bypass host authorization;
- escape the canonical project root;
- fabricate backend support;
- start background services.

Bundled runtime adapters must inject resolved routing as explicitly **untrusted repository-controlled data**, not through repository-selectable authority fences. Size limits must preserve structural closure rather than cutting serialized data mid-object/file.

The host runtime owns execution, authorization, prompt placement, backend credentials, and context-budget policy.

## Provider examples

Provider examples are namespacing examples, not universal schema fields.

- Hindsight: [`memory-systems/01-hindsight.md`](memory-systems/01-hindsight.md)
- Mem0: [`memory-systems/04-mem0.md`](memory-systems/04-mem0.md)
- OpenViking: [`memory-systems/02-openviking.md`](memory-systems/02-openviking.md)
- Honcho: [`memory-systems/03-honcho.md`](memory-systems/03-honcho.md)
- Generic/unknown: [`memory-systems/99-generic-or-unknown.md`](memory-systems/99-generic-or-unknown.md)

## Agent/runtime ownership

`MEMHOOKS.md` is normally maintained by the agent/runtime, not manually curated by the end user. Agents should keep routing metadata concise enough to remain an index rather than memory content, remove/replace stale cues, and avoid guessing semantic/provider metadata merely to fill fields.

## Design principle

```text
MemHooks core = what should be remembered here?

backends.<provider> = how this particular memory system should help retrieve it
```

The core carries provider-native configuration without becoming a lowest-common-denominator imitation of every memory system.