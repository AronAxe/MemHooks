# Runtime plan, reliability and 0.6 migration

MemHooks files remain `memhooks/v2`. `memhooks explain --format json` adds
`plan_version: "memhooks/plan-v1"`, the selected `effective_queries`, and
`omitted_queries` with `source`, `query`, and `reason`. Reasons are `overridden`,
`inheritance_cut`, or `role_mismatch`. The full explain output is diagnostic data,
not the size-bounded runtime representation. Semantic errors stop planning with
source-positioned diagnostics instead of becoming unrestricted queries.

## Host input

The reference Python adapter accepts this event object:

```json
{
  "hook_event_name": "pre_llm_call",
  "cwd": "/absolute/project",
  "active_files": ["src/auth/login.py", "src/api/routes.py"],
  "active_roles": ["reviewer"]
}
```

`active_files` and `active_roles` are optional. The host must supply its own
**session-local** file activity; the adapter does not invent or persist a global
session history. At most 64 nonempty file strings and 32 role strings are accepted.
File targets must exist inside the base canonical root. Nested repositories with
a different root are not silently merged. Ancestor targets are redundant when a
descendant is selected; at most eight resulting directories are resolved under
one shared timeout. If no usable files are supplied, ordinary cwd routing remains.
An invalid active child is not replaced with broader root cues.

The runtime view drops raw duplicate queries, retains source attribution, and
uses `recall_queries` for the selected effective queries. With multiple sibling
scopes it includes `contexts`, keyed by string IDs. Each contains that scope's
`target`, `scope`, `sensitivity`, `exclude`, top-level cues and backend defaults.
Queries/guidance reference these using a `contexts` ID list. Identical inherited
items with matching routing controls are deduplicated; differing sibling
constraints must not be flattened. Effective queries already carry their own
merged entities, resources, tags and backend settings. Host authorization remains
separate and authoritative.

## Exact budgets and graceful failures

| Setting | Default | Accepted range |
| --- | --- | --- |
| `MEMHOOKS_MAX_CHARS` | 24000 | 1–1000000 characters |
| `MEMHOOKS_MAX_GUIDANCE_CHARS` | 6000 | 1–100000 characters |
| `MEMHOOKS_RESOLVE_TIMEOUT` | 5 | 0.01–120 finite seconds |
| `MEMHOOKS_AUTO_PATHS` (Rust maintainer) | 12 | 1–256 files per generated cue |

The total cap includes the trust wrapper and the exact compact JSON emitted;
it is **not a tokenizer-dependent token count**. Oversized guidance is marked.
Further reduction removes whole items, preferring routing over prose and higher
priorities over lower ones. `budget_omissions` counts removed items by field;
full query-by-query explanations remain available through `explain`.
Provenance itself is bounded. Scope, sensitivity, exclusions and backend controls
are never removed while leaving their queries executable: a minimal no-routing
fallback is used when those controls cannot fit. A budget too small even for the
wrapper and `{}` causes no context injection.

Malformed event objects, fields, environment settings, resolver output, unsupported
plan versions or a missing executable produce a normal `{}` optional-hook response
and a stderr diagnostic. Install the CLI and adapter together; an older unversioned
resolver is deliberately not treated as compatible. `MEMHOOKS_BIN`, when set,
is authoritative; a missing configured file does not silently select another binary.

## Generated cue lifecycle

Generated cues carry the reserved `memhooks:auto` tag. Their question text now
includes `[scope: <root-relative-directory>]` so ancestor resources are not lost
through same-text child overrides. Existing legacy generated cues are upgraded
when the relevant directory is next maintained. Ordinary query identity and
inheritance rules are unchanged.

Explicit post-tool `source`, `destination`, `path` and other supported path fields
can reconcile renamed/deleted leaf files. Missing paths are used only to clean
existing cues, never to create file anchors. Changes without these events can be
reconciled using:

```bash
memhooks prune --all --dry-run
memhooks prune --all
memhooks remove --cwd src/auth --query "Obsolete local cue" --dry-run
```

Pruning only edits generated file resources/empty generated queries, not ordinary
semantic notes or backend memories. Removal targets the nearest local hook;
removing a child override may reveal an inherited parent query. A preview does not
create lock files or modify hooks. Applied changes are locked/atomic **per hook**,
not a cross-file transaction. Validation or filesystem errors are surfaced.

## Rust API migration

`find_root`, `inheritance_chain`, `discover_hooks` and `resolver::resolve_parsed`
now return `Result<_, ParseError>`. Propagate `?` or handle errors explicitly.
`TargetContext::new` exposes the common fallible canonical root/target/directory
interpretation. `ParseError.code` is now `String` so semantic validator diagnostics
retain their original codes and locations. `require_valid` rejects errors while
allowing forward-compatible warnings. New public maintenance APIs are `prune`,
`remove_note`, and `PruneReport`.

Canonicalization errors are no longer silently reinterpreted as relative paths.
A configured `MEMHOOKS_ROOT` must be an existing directory; as before it overrides
Git-root discovery only when it contains the target. `validate --all` scans the
whole selected project even when invoked on a file or from a nested cwd.

Hook and lock files must be ordinary files, not symlinks or special files. This
includes symlinks whose destinations are still inside the repository. Unix uses
`O_NOFOLLOW` and nonblocking opens; Windows opens the final reparse point rather
than following it. The design is not a sandbox against a hostile process replacing
ancestor directories or mount points concurrently: the host must secure its working
filesystem and enforce tool permissions. Untrusted guidance remains data.

Additional diagnostics: `MH028` invalid runtime configuration, `MH029` unsafe
hook/lock/target type, and `MH030` hook-discovery I/O failure.

## Evidence and the next evaluation boundary

Regression tests cover malformed routing, nested paths, hook/lock symlinks,
initialization interleavings, exact Unicode-inclusive budgets, lifecycle previews,
active-file routing and real Rust-to-Python handoffs. Rust CI requires the binary
integration cases to execute; Python-only compatibility jobs explicitly skip them.

These are correctness tests, **not a demonstrated agent recall improvement**.
For a real evaluation, pair MemHooks-on/off runs with the same agent, backend,
stored memories and task set. Record expected/retrieved context IDs, repeated known
mistakes, retrieval calls, injected tokens and latency. Report per-task paired
changes, aggregate counts and uncertainty, with fixed model/backend settings and
seeds where supported. Do not substitute synthetic routing success for task outcomes.
