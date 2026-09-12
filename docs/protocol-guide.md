# Protocol guide

This document explains the practical semantics of **`memhooks/v2`**. The normative specification is [`references/memhooks-format.md`](../references/memhooks-format.md).

## Mental model

```text
core fields
    = provider-neutral retrieval intent

backends.<provider>
    = opaque provider-native retrieval controls
```

The core never promotes a provider's ontology into universal fields.

There is also deliberately **one structured routing store**:

```text
YAML frontmatter
    ↓
reference parser / resolver / maintainer
```

The Markdown body is optional source-attributed guidance, not a second JSON/query database.

## Core shape

```md
---
schema: memhooks/v2
inherits: true
scope: backend/auth

recall_queries:
  - "What decisions matter here?"
  - query: "What security boundaries must never be violated?"
    priority: 1.0
    when:
      roles: [reviewer, architect]
    entities: [Authentication]
    resources:
      - name: auth-postmortem
        kind: postmortem
        salience: 0.9
    tags: [security]
    backends:
      mem0:
        top_k: 8
      hindsight:
        memory_types: [experience]

entities:
  - name: Authentication
    type: CONCEPT
    salience: 0.95
resources: [auth-architecture]
tags: [backend]
exclude: [obsolete OAuth prototype]
sensitivity: private

backends:
  mem0:
    filters:
      user_id: auth-agent
  hindsight:
    bank: project-memory
---

Optional free-form retrieval guidance goes here.
```

## `schema`

Required:

```yaml
schema: memhooks/v2
```

The package version and schema version are separate. v0.5.1 remains `memhooks/v2` and hard-rejects unsupported schemas during resolution.

## Canonical root

Resolver, maintainer, and bundled runtime adapters share one root definition:

1. explicit containing `MEMHOOKS_ROOT` if supplied;
2. otherwise nearest Git root as a hard boundary;
3. only outside Git, highest ancestor containing `MEMHOOKS.md`.

A hook above a Git repository cannot silently capture that repository's maintenance writes.

## `inherits`

Optional boolean, default `true`.

```yaml
inherits: false
```

starts a new local hook scope. Ancestors above that hook are discarded for the active subtree.

Only the top-level core field controls inheritance. A provider-native key named `inherits` under `backends.<provider>` is opaque and has no effect.

## `recall_queries`

The central field. Each query describes context whose answer could materially change current work.

Simple:

```yaml
recall_queries:
  - "Why was the current queue architecture selected?"
```

Structured:

```yaml
recall_queries:
  - query: "Which previous failures involved queue ordering?"
    priority: 0.9
    when:
      roles: [reviewer]
    entities: [Queue]
    resources: [queue-postmortem]
    tags: [reliability]
    backends:
      mem0:
        rerank: true
```

### Query identity and local override

Across an inheritance chain, **trimmed query text is the query identity**.

If a child repeats the same question as a parent, the child's declaration replaces the parent query and all query-local metadata. The final plan contains one retrieval request, not two contradictory copies.

Within one file, duplicate query text is still a validation warning.

### `priority`

Optional float `0.0..1.0`. It means importance of the recall request under context pressure. It does not mean truth, confidence, semantic similarity, or provider relevance.

### `when.roles`

Optional open role strings. A restricted query applies if any declared role exactly matches an active role. If the runtime has no active-role information, restricted queries remain present rather than being guessed away.

### Query-local cues

Structured queries can add local `entities`, `resources`, `tags`, and `backends`. Generic cues supplement resolved scope cues; query-local provider mappings overlay scope provider mappings structurally.

## `entities`

Named retrieval cues:

```yaml
entities:
  - Authentication
  - name: OpenAI
    type: ORG
    salience: 0.9
```

`type` is intentionally open. `salience` is optional `0.0..1.0` cue importance, separate from query priority/provider scores.

## `resources`

Named existing resources:

```yaml
resources:
  - auth-architecture
  - name: outage-postmortem
    kind: postmortem
    salience: 0.9
```

`kind` is an open string. Provider-native resource identifiers remain provider-owned where exact semantics matter.

Automatic deterministic file anchors are represented as generic resources with `kind: file` on a structured auto-query.

## `tags`

Backend-neutral routing/relevance labels. Adapters may map them to native metadata filters where a real equivalent exists.

## `exclude`

First-class anti-recall. Use native negative filtering or post-filtering to keep known obsolete/misleading context out of the active task.

## `scope` and `sensitivity`

`scope` is descriptive backend-neutral metadata. `sensitivity` is advisory handling metadata. Neither overrides host filesystem state, authorization, or security policy.

## `backends`

Opaque provider mappings:

```yaml
backends:
  mem0:
    filters:
      user_id: alice
    top_k: 8

  hindsight:
    bank: project-memory
    memory_types: [experience]
```

The core knows only that provider names are non-empty and provider values are mappings. It does not know what `user_id`, `top_k`, `bank`, or `memory_types` means.

### Provider merge semantics

- mapping + mapping → recursive merge;
- more-local scalar/list/other value → replace parent value;
- query-local provider mapping → overlay resolved scope provider mapping.

Lists are replaced rather than concatenated because generic MemHooks cannot know provider list semantics.

## Markdown body guidance

Free-form Markdown after the frontmatter remains source-attributed retrieval guidance.

It is accumulated root → leaf in `ResolvedHook.guidance` and included in the complete `memhooks explain --format json` adapter handoff.

Structured/machine-maintained routing belongs in YAML frontmatter, not in hidden JSON code fences in the body.

## Reference maintenance

The reference engine writes the same model it reads:

```bash
memhooks init .
memhooks note --cwd . --query "Why was this design chosen?" --priority 0.9
printf '%s' '<post_tool_call JSON>' | memhooks event
```

Maintenance requirements:

- semantic cues and deterministic anchors are written to YAML frontmatter;
- unknown/future structured fields and provider mappings survive enrichment;
- automatic file anchors inspect explicit path-bearing tool-input fields only;
- candidate anchors must be existing files inside the canonical root;
- concurrent updates use an exclusive lock;
- writes use atomic same-directory replacement.

The Python helper is compatibility glue to the canonical binary, not a separate persistence implementation.

## Root-to-leaf resolution

For a target:

1. choose the canonical root;
2. discover hooks root → leaf;
3. reject unsupported schemas;
4. apply `inherits: false` cutoffs;
5. merge remaining hooks;
6. make same-text child queries replace parent queries;
7. accumulate generic list cues with exact duplicate suppression;
8. use local scalar core values;
9. structurally merge provider mappings;
10. accumulate source-attributed body guidance;
11. apply known role filtering.

## Diagnostics

The v0.5.1 validator uses YAML AST source positions rather than textual first-match searching. Repeated fields therefore point at the actual offending node.

Malformed structured query/entity/resource entries are preserved far enough to produce targeted lint diagnostics when possible. Truly syntactically invalid YAML can still produce a file-level parse error.

SARIF uses validation-root-relative artifact paths and includes rule metadata/help.

## Provider mappings

- [Hindsight](../references/memory-systems/01-hindsight.md)
- [OpenViking](../references/memory-systems/02-openviking.md)
- [Honcho](../references/memory-systems/03-honcho.md)
- [Mem0](../references/memory-systems/04-mem0.md)
- [Generic / unknown](../references/memory-systems/99-generic-or-unknown.md)

## Trust boundary

Repository-controlled hooks are untrusted routing data. They cannot grant themselves system/developer authority, execute arbitrary commands, authorize memory writes, bypass access controls, or escape the canonical root.

Bundled adapters should inject validated routing structurally and with an explicit trust label. Size limits must preserve structural closure.

See [`SECURITY.md`](../SECURITY.md).

## Agent/runtime ownership

The agent/runtime normally maintains hook files, not the end user. Operational rules like keeping cues concise, pruning stale routing, or choosing provider hints are instructions to the maintainer/runtime.

## Non-goals

`MEMHOOKS.md` is not a memory database, transcript dump, README replacement, prompt-privilege declaration, retention policy, arbitrary command manifest, universal provider API, or directive store.