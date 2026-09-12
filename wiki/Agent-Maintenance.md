# Agent Maintenance

This page exists partly to make one thing unmistakable:

> **Normal users are not expected to manually curate `MEMHOOKS.md`.**

MemHooks routing files are agent/runtime infrastructure.

## Ownership model

### User

The human user normally:

- installs/enables MemHooks;
- chooses/configures the host runtime;
- may choose a memory backend;
- may inspect or override routing when they deliberately want to.

### Agent/runtime

The agent/runtime normally:

- creates local hooks when a subsystem develops distinct recall needs;
- writes durable recall queries after meaningful decisions/failures/constraints are discovered;
- maintains conservative file-resource anchors after relevant tool activity;
- replaces stale routing;
- avoids turning hooks into memory dumps;
- supplies provider-native hints only when they are genuinely known.

## One structured persistence path

The reference maintainer writes the same YAML frontmatter the resolver reads.

```text
memhooks note / memhooks event
        ↓
MEMHOOKS.md YAML frontmatter
        ↓
reference parser
        ↓
reference resolver
        ↓
adapter
```

There is no second JSON note database hidden in the Markdown body.

This matters because a writer and resolver that persist different representations can both look correct in isolation while failing end to end.

## `memhooks note`

When the current agent has already learned a durable retrieval cue during ordinary reasoning, it can preserve it without another model call.

```bash
memhooks note \
  --cwd "$PWD" \
  --query "Why did authentication change after the outage?" \
  --priority 0.9 \
  --role reviewer \
  --entity Authentication \
  --resource auth-postmortem \
  --tag security
```

Provider-specific hints are allowed when actually known:

```bash
memhooks note \
  --cwd "$PWD" \
  --query "Why did authentication change after the outage?" \
  --backends '{"mem0":{"top_k":8,"rerank":true}}'
```

The core preserves provider mappings without interpreting them.

## Repeated notes

If the same normalized query already exists locally, maintenance enriches/updates that structured query rather than blindly appending duplicates.

Unknown future fields and opaque provider data should survive maintenance rather than being deleted by a lossy round-trip.

## `memhooks event`

A runtime can send structured post-tool-call activity to the deterministic maintainer.

The maintainer extracts **explicit path fields**, verifies that they resolve to real files inside the repository boundary, and uses them as conservative file-resource anchors.

It does not scrape arbitrary file contents looking for strings that resemble paths.

That distinction prevents junk such as:

```text
foo.bar
requests.get
mangled URL fragments
```

from becoming routing anchors merely because they appeared inside source code.

## Deterministic maintenance must stay conservative

A touched file path can tell the runtime:

> This file was involved in recent work.

It cannot reliably tell the runtime:

- the semantic importance of the work;
- which future role should retrieve it;
- which named entity matters;
- whether a Hindsight Mental Model should be queried;
- which Mem0 filter/reranker should be used;
- what the memory itself should contain.

So automatic file anchors must not guess those things.

## Atomic writes and locking

Maintenance is a read-modify-write operation and may happen from parallel tool calls.

The reference maintainer therefore uses:

- an exclusive sibling lock;
- a temporary file in the same directory;
- flush/sync;
- atomic replacement.

The goal is simple: parallel events should not silently erase one another, and an interrupted write should not leave a truncated `MEMHOOKS.md`.

## Root containment

Maintenance follows the same project-root semantics as resolution.

Inside Git, the repository root is the hard boundary. The maintainer must not climb above `.git` looking for an outer hook to capture a project that was never enabled.

## Keep hooks concise—who exactly should do that?

The **agent/runtime**.

“Keep routing cues concise” is an implementation/maintenance rule, not a chore for the human user.

A good maintainer should automatically prefer:

- concrete questions over vague reminders;
- compact entities/resources over copied memory content;
- local scope when a cue only matters locally;
- replacing stale cues over endlessly appending history;
- bounded auto-generated path anchors.

## What belongs in the hook?

Good:

```yaml
recall_queries:
  - "Why was this locking strategy chosen?"
resources:
  - concurrency-postmortem
```

Bad:

```text
A 2,000-word transcript of the entire concurrency incident.
```

MemHooks should remain an **index into memory**, not become the memory itself.

## Human overrides

Humans are still free to edit hooks. The point is not to prohibit that—it is to avoid designing the normal workflow around it.

If the user explicitly adds or changes routing, the agent/runtime should treat that as intentional project input and preserve it unless a clear conflict/error requires attention.