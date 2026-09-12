# Why MemHooks?

Most memory systems answer a question like:

> **Given a query, what should I retrieve?**

MemHooks addresses the problem one step earlier:

> **How does the agent know what to ask for in the first place?**

That gap is easy to miss because a memory system can be working perfectly while the agent still behaves as if it forgot something.

## The retrieval-cue problem

Imagine an agent returns to an authentication subsystem three weeks later.

The memory backend already contains:

- the reason token rotation was split into two stages;
- a previous race condition;
- an outage caused by a rejected implementation;
- a security constraint agreed with the team;
- the fact that a seemingly obvious refactor was already tried and rolled back.

Nothing is missing from storage.

But the current prompt says only:

> Refactor `refresh.rs`.

A generic semantic memory search may retrieve recent or superficially similar material. The agent may never formulate the question:

> Why was refresh-token rotation split into two stages, and what approaches were rejected?

That is the problem MemHooks targets.

## Memory versus cue

```text
memory backend
    stores what happened

MemHooks
    stores what should be recalled here
```

A `MEMHOOKS.md` near the authentication code can contain a compact retrieval cue:

```yaml
recall_queries:
  - query: "Why was refresh-token rotation split into two stages, and what alternatives were rejected?"
    priority: 0.9
    tags: [security, auth]
```

The hook does **not** need to contain the answer. It points the agent toward the answer already stored elsewhere.

## Why filesystem scope?

Codebases and projects already have a useful hierarchy:

```text
project
├── backend
│   └── auth
├── frontend
└── billing
```

Different areas have different historical context. A billing agent should not automatically receive authentication postmortems, and an auth agent should not need every decision in the entire company memory.

Filesystem scope gives MemHooks a deterministic, cheap, understandable signal for **where** a cue applies.

## Why not just put everything in AGENTS.md / README / CLAUDE.md?

Because those files are usually instructions or documentation, not a retrieval-routing layer.

Putting durable memory contents directly into prompt files creates other problems:

- prompt size grows continually;
- stale facts remain present even when irrelevant;
- project instructions and historical evidence get mixed together;
- every model call pays for context that may not matter;
- one provider's memory features leak into project-level instructions.

MemHooks stays narrow: keep a compact index of what should be recalled, then retrieve the actual evidence only when relevant.

## Why not let vector search figure it out?

Vector search is useful after a query exists. It does not guarantee the right query will be generated.

For example:

```text
current task: optimize cache invalidation
```

A similarity search might retrieve documents mentioning caches or invalidation. But the critical context may be:

> A previous optimization violated ordering guarantees under failover.

If nothing cues the agent to ask about prior failures or ordering constraints, good storage and good embeddings are not enough.

## Why backend-neutral?

Different systems have different strengths:

- Hindsight has its own memory categories, relationships, Reflect, and Mental Models.
- Mem0 has filters, reranking, graph retrieval, and scope identifiers.
- OpenViking has its own hierarchical/resource model.
- Honcho has its own session/peer/reasoning concepts.

Those concepts are valuable—but they are not universal.

MemHooks therefore separates:

```text
core retrieval intent
        +
provider-specific hints
```

The same project cue can survive a change of memory backend without pretending all backends expose the same API.

## What success looks like

Without MemHooks:

```text
memory contains answer ✅
agent has access ✅
agent asks wrong/no question ❌
useful memory is never surfaced ❌
```

With MemHooks:

```text
agent enters relevant scope
        ↓
local hook supplies retrieval cue
        ↓
active backend retrieves evidence
        ↓
agent sees relevant prior context
        ↓
current decision benefits from project history
```

MemHooks is intentionally not a giant memory framework. Its value comes from solving one small but important missing step:

> **Cue the right memory before the agent needs to rediscover why it mattered.**