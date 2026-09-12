# MEMHOOKS File Reference

A `MEMHOOKS.md` file is Markdown with YAML frontmatter.

The frontmatter contains structured retrieval-routing metadata. The optional Markdown body contains source-attributed retrieval guidance.

> **The exact normative contract lives in** [`references/memhooks-format.md`](https://github.com/AronAxe/MemHooks/blob/main/references/memhooks-format.md). **This page is the approachable explanation.**

## Canonical shape

```md
---
schema: memhooks/v2
inherits: true
scope: backend/auth

recall_queries:
  - query: "Why did authentication change after the outage?"
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
---

Remember that this subsystem has unusually strict rollback constraints.
```

## `schema`

Required:

```yaml
schema: memhooks/v2
```

Unsupported schemas fail explicitly during resolution. The package version and protocol schema version are separate concepts.

## `inherits`

Optional boolean. Defaults to `true`.

```yaml
inherits: false
```

starts a new local inheritance scope and cuts off ancestors above this hook for the active subtree.

## `scope`

Optional backend-neutral descriptive label:

```yaml
scope: backend/auth
```

More-local values replace broader values.

## `recall_queries`

The central field: concrete questions worth asking memory in this scope.

Simple:

```yaml
recall_queries:
  - "What failures have happened here before?"
```

Structured:

```yaml
recall_queries:
  - query: "Why was this retry strategy chosen?"
    priority: 0.8
    when:
      roles: [reviewer]
    tags: [reliability]
```

Structured queries may carry:

- `query`
- `priority`
- `when.roles`
- query-local `entities`
- query-local `resources`
- query-local `tags`
- query-local `backends`

Provider-specific fields do **not** belong directly on the query; place them under the provider namespace.

## `priority`

Optional number from `0.0` to `1.0`.

Priority means:

> **How costly is it to omit this recall request when context budget is tight?**

It is not semantic similarity, confidence, truth probability, or a provider search score.

If omitted, MemHooks does not invent a mandatory numeric default.

## `when.roles`

Optional open list of runtime/project role names:

```yaml
when:
  roles: [reviewer, architect]
```

Matching is exact-string OR matching when active roles are known.

## `entities`

Named retrieval cues.

Simple:

```yaml
entities:
  - Authentication
```

Structured:

```yaml
entities:
  - name: OpenAI
    type: ORG
    salience: 0.9
```

`type` is intentionally open; MemHooks does not impose a universal entity taxonomy.

`salience` is optional cue importance from `0.0` to `1.0`.

## `resources`

Named existing resources worth consulting.

Simple:

```yaml
resources:
  - auth-architecture
```

Structured:

```yaml
resources:
  - name: outage-postmortem
    kind: postmortem
    salience: 0.9
```

`kind` is open and backend-neutral. Provider-native resource IDs or modes belong in that provider's namespace.

## `tags`

Generic relevance/routing labels:

```yaml
tags: [security, backend]
```

A provider adapter may map them to native metadata when there is a real equivalent.

## `exclude`

First-class anti-recall:

```yaml
exclude:
  - obsolete OAuth prototype
```

Adapters should keep matching obsolete/misleading material out of working context.

## `sensitivity`

Optional advisory handling metadata:

```yaml
sensitivity: private
```

It does not override host authorization or security policy.

## `backends`

Provider-specific configuration lives here:

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

The generic MemHooks engine knows only that each provider namespace is a mapping. It preserves and structurally merges the contents; provider adapters interpret them.

## Markdown body

Anything after the closing frontmatter delimiter is optional retrieval guidance:

```md
Remember that this area has strict rollback constraints and that previous fixes
failed when they assumed event ordering was stable across failover.
```

The body is **guidance**, not a hidden store for structured queries. The reference resolver preserves it with source provenance and exposes it in the adapter handoff.

## One structured data path

Structured routing belongs in frontmatter. Period.

The reference maintainer, parser, validator, resolver, CLI, and adapters are designed around that single representation.

For inheritance behavior, continue with [[Inheritance and Scoping]].