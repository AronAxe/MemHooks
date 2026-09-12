# How MemHooks Works

MemHooks has a deliberately small job:

> **Resolve the retrieval cues that apply to the place an agent is working, then hand those cues to the active memory system.**

## The lifecycle

```text
agent/runtime receives a task
        ↓
determine the real target directory
        ↓
find repository root
        ↓
load MEMHOOKS.md root → leaf
        ↓
apply inherits:false cutoffs
        ↓
merge generic routing + provider namespaces
        ↓
apply known role routing
        ↓
produce one resolved retrieval plan
        ↓
active backend retrieves bounded context
        ↓
agent performs the task
        ↓
agent/runtime may update retrieval cues afterward
```

## 1. Determine the target

The target is the actual project location relevant to the work—not a path guessed from prompt wording.

Examples:

```text
/repo/backend/auth/
/repo/backend/auth/refresh.rs
```

If a file is supplied, the resolver operates from its containing directory.

## 2. Find the root

Inside a Git repository, the `.git` root is the hard project boundary.

MemHooks does not climb above it looking for unrelated hooks. That prevents a stray `~/MEMHOOKS.md` from capturing or influencing an unrelated repository.

Outside Git, the reference resolver uses its documented hook-root fallback semantics.

See [[Inheritance and Scoping]].

## 3. Build the hook chain

Given:

```text
/repo/MEMHOOKS.md
/repo/backend/MEMHOOKS.md
/repo/backend/auth/MEMHOOKS.md
```

and target `/repo/backend/auth`, the resolver reads them in that order.

This creates a natural hierarchy:

- broad project concerns at the root;
- subsystem concerns below;
- highly local concerns nearest the work.

## 4. Apply inheritance

By default, hooks inherit:

```yaml
inherits: true
```

A local hook may intentionally start a new scope:

```yaml
inherits: false
```

That removes ancestors above that hook for the active subtree.

## 5. Merge routing intent

Core fields are backend-neutral:

- recall queries;
- priority;
- roles;
- entities;
- resources;
- tags;
- exclusions;
- scope/sensitivity.

Provider-native controls remain opaque under:

```yaml
backends:
  mem0: {...}
  hindsight: {...}
```

Nested provider mappings structurally merge; more-local scalar/list values replace parent values.

## 6. Resolve query identity

A recall query is identified by its normalized query text across the inheritance chain.

If a child repeats the same query text with different metadata, the **more-local declaration wins** instead of issuing the same retrieval twice with conflicting priority or routing.

That means this root query:

```yaml
- query: "Why did auth change?"
  priority: 0.3
```

can be refined locally:

```yaml
- query: "Why did auth change?"
  priority: 0.9
  when:
    roles: [reviewer]
```

The effective plan contains one local version.

## 7. Apply roles

If the runtime knows active roles, `when.roles` filters applicable queries.

If it does **not** know active roles, restricted queries are preserved rather than silently discarded.

See [[Weighted Retrieval and Roles]].

## 8. Preserve guidance

The optional Markdown body is source-attributed retrieval guidance.

It is not a second structured routing database. Structured metadata belongs in YAML frontmatter.

The reference `explain --format json` handoff includes guidance and provenance so adapters do not have to re-read raw files independently.

## 9. Retrieve through the active backend

MemHooks does not contact memory providers itself.

The runtime chooses the connected/authorized backend and translates the effective routing plan into provider-native retrieval.

For example, a Mem0 adapter may use `filters`, `top_k`, or reranking hints under `backends.mem0`; a Hindsight adapter may use its own memory categories or Reflect controls under `backends.hindsight`.

See [[Memory Backends]].

## 10. Maintain cues through the same data path

The reference maintainer writes to the same frontmatter model the parser and resolver read.

```text
memhooks note/event
      ↓
YAML frontmatter
      ↓
parser/resolver
      ↓
adapter
```

This is intentional. MemHooks should not have one representation for writers and another for readers.

## Design boundary

MemHooks decides **what should be recalled here**.

It does not decide:

- which memory vendor the user must use;
- whether retrieved memory is true;
- which tool permissions an agent has;
- prompt privilege;
- credential scope;
- universal ranking mathematics;
- automatic memory creation/deletion.

Those remain runtime/backend responsibilities.