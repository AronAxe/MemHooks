# Inheritance and Scoping

MemHooks uses the filesystem as a deterministic applicability signal.

The basic rule is:

> **Resolve applicable hooks from the project root toward the place the agent is actually working.**

## The Git root is a hard boundary

Inside a Git repository, MemHooks stops at the repository root.

Example:

```text
/home/aron/MEMHOOKS.md
/home/aron/projects/app/.git/
/home/aron/projects/app/src/auth/
```

When working in `app/src/auth`, the outer `/home/aron/MEMHOOKS.md` does **not** participate.

This is both a semantic and security boundary. An unrelated hook outside the repository must not capture writes or influence resolution simply because it is an ancestor directory.

## Root-to-leaf chain

Given:

```text
/repo/MEMHOOKS.md
/repo/backend/MEMHOOKS.md
/repo/backend/auth/MEMHOOKS.md
```

for target `/repo/backend/auth`, resolution order is:

```text
/repo/MEMHOOKS.md
        ↓
/repo/backend/MEMHOOKS.md
        ↓
/repo/backend/auth/MEMHOOKS.md
```

Broad concerns belong higher; local exceptions/refinements belong lower.

## `inherits: false`

Inheritance defaults to true.

A local hook may declare:

```yaml
inherits: false
```

This cuts off ancestors **above that hook** for the current subtree.

Example:

```text
root hook
  ↓
backend hook
  ↓
auth hook: inherits:false
  ↓
refresh hook
```

Effective chain for `auth/refresh`:

```text
auth hook
  ↓
refresh hook
```

The root/backend hooks are intentionally discarded.

Provider-owned nested keys named `inherits` have no effect on core inheritance:

```yaml
backends:
  example_provider:
    inherits: false   # provider data only
```

Only the top-level core `inherits` field controls MemHooks inheritance.

## Scalar fields

More-local specified scalar values win.

Examples:

```yaml
scope: backend/auth
sensitivity: private
```

A child can replace these values for its subtree.

## Accumulating cues

Entities, resources, tags, exclusions, and distinct recall queries generally accumulate through the active chain with duplicate handling.

The interesting exception is **same-query identity**.

## Same query text: local override

A recall query is identified across the inheritance chain by normalized text.

Root:

```yaml
recall_queries:
  - query: "Why did auth change?"
    priority: 0.3
```

Child:

```yaml
recall_queries:
  - query: "Why did auth change?"
    priority: 0.9
    when:
      roles: [reviewer]
```

The effective result is **one** query: the more-local declaration.

This avoids issuing the same retrieval twice with contradictory metadata.

Different query text remains distinct even if semantically similar. MemHooks deliberately does not perform fuzzy semantic deduplication at the protocol layer.

## Provider namespace inheritance

Provider mappings use deterministic structural merge rules.

Parent:

```yaml
backends:
  mem0:
    filters:
      user_id: alice
    threshold: 0.1
```

Child:

```yaml
backends:
  mem0:
    threshold: 0.2
    rerank: true
```

Resolved:

```yaml
backends:
  mem0:
    filters:
      user_id: alice
    threshold: 0.2
    rerank: true
```

Rules:

- mapping + mapping → recursive merge;
- a more-local scalar replaces the parent value;
- a more-local sequence/list replaces the parent sequence/list.

MemHooks does not concatenate provider-owned lists because it cannot know their semantics.

## Query-local provider overlay

A structured query may add provider configuration:

```yaml
backends:
  mem0:
    filters:
      user_id: alice
    threshold: 0.1

recall_queries:
  - query: "What failed here before?"
    backends:
      mem0:
        top_k: 5
```

The effective query receives the scope-level provider configuration plus the query-local overlay.

## Root semantics must be shared

Writers, readers, validators, and runtime adapters must agree on root discovery.

The reference architecture intentionally centralizes root/inheritance behavior in the Rust engine. Runtime adapters should consume its resolved output rather than independently reimplementing subtly different filesystem semantics.

See [[How MemHooks Works]] and [[Architecture]].