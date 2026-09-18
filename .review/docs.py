from pathlib import Path
r = Path('.')
# Keep exception types accurate, and catch both malformed types and values.
p = r/'hooks/hermes/memhooks_pre_llm.py'
s = p.read_text()
for text in ['f"malformed resolver field: {field}"', '"malformed resolver root/target"', 'f"malformed {field} item"', 'f"malformed {field}"', '"malformed backend map"', '"malformed effective query"', '"hook event must be a JSON object"']:
    s = s.replace('raise ValueError('+text+')', 'raise TypeError('+text+')')
s = s.replace('subprocess.SubprocessError, ValueError, RecursionError', 'subprocess.SubprocessError, ValueError, TypeError, RecursionError')
s = s.replace('for field in indexes:', 'for field, field_index in indexes.items():').replace('indexes[field].get(identity)', 'field_index.get(identity)').replace('indexes[field][identity] = entry', 'field_index[identity] = entry')
p.write_text(s)
# Current version guidance; historical changelog entries are not rewritten.
for name in ['SKILL.md', 'README.md', 'docs/README.md', 'docs/quickstart.md', 'docs/protocol-guide.md', 'docs/rust-library.md', 'docs/publishing.md']:
    p = r/name
    s = p.read_text()
    if name == 'README.md':
        for old, new in [('version-0.5.1','version-0.6.0'), ('cargo install memhooks --version 0.5.1','cargo install memhooks --version 0.6.0'), ('cargo add memhooks@0.5.1','cargo add memhooks@0.6.0'), ('in v0.5.1:', 'in v0.6.0:'), ('**v0.5.1 — backend-neutral `memhooks/v2` + one normative resolver/maintainer data path + hardened runtime/diagnostic/security boundaries.**', '**v0.6.0 — validated routing, consistent project boundaries, active-file recall, bounded context, and generated-cue lifecycle. Protocol: `memhooks/v2`.**')]:
            s = s.replace(old, new)
    else:
        s = s.replace('0.5.1', '0.6.0')
    p.write_text(s)
intro = '''## v0.6.0: reliability and task-aware recall

The file schema stays **`memhooks/v2`**; the machine handoff is now explicitly
versioned as **`memhooks/plan-v1`**. Update the Rust CLI and Hermes adapter together.

- All filesystem entry points share fallible canonical path/root resolution,
  including default `.` invocations from nested directories.
- Semantic validation is required before resolution or persistence. Invalid role
  routing, priorities, entities and backend namespaces fail with diagnostics;
  unknown extension fields remain round-trippable warnings.
- Hook and lock symlinks/special files are rejected. Initialization rechecks file
  state under the same exclusive lock as creation, preserving concurrent notes.
- `MEMHOOKS_MAX_CHARS` caps the **complete emitted context**, including its wrapper.
  Truncation preserves valid JSON and never retains queries without their routing
  constraints. Invalid optional-hook input/configuration produces `{}` plus a diagnostic.
- Hosts can supply session-local `active_files` and `active_roles`. The loader
  resolves at most eight relevant directory scopes under one shared timeout,
  rather than loading every hook in the project. Sibling routing controls remain separate.
- New/refreshed automatic cues use directory-qualified identities. Explicit
  delete/rename file events reconcile local generated resources; `prune` catches
  stale cues from changes made outside those events.

```bash
memhooks prune --all --dry-run
memhooks prune --all
memhooks remove --query "An obsolete local cue" --dry-run
memhooks remove --query "An obsolete local cue"
```

These commands only edit retrieval cues, never backend memories. Dry runs do not
change hook files or create locks. `explain --format json` reports role,
inheritance-cut and override omissions; the bounded handoff reports budget omissions.
See [runtime contract and migration](docs/runtime-plan.md).

'''
p = r/'README.md'
s = p.read_text().replace('## v0.5.1 hardening', intro+'## v0.5.1 foundation (historical)')
s = s.replace('v0.5.1 is an implementation hardening release;', 'v0.5.1 established the shared implementation;')
p.write_text(s)
release = '''## [0.6.0] - 2026-09-18

### Fixed
- Canonical nested/default path handling across initialization, notes, resolution and validation.
- Reject semantically invalid hooks before planning or persistence, including direct library writes.
- Reject hook/lock symlinks and special files; use no-follow opens on Unix/Windows.
- Recheck initialization under the exclusive lock; preserve intervening writes.
- Bound the exact emitted context, including provenance and wrapper; handle malformed events/settings gracefully.
- Accept valid explicit filenames containing parentheses, spaces and Unicode.
- Surface discovery I/O errors instead of reporting a misleading successful scan.

### Added
- Versioned `memhooks/plan-v1` handoff, with role/override/inheritance omission reasons.
- Host-supplied active-file/role routing, bounded scopes, shared timeout and sibling-context isolation.
- Scope-qualified generated query identities and explicit rename/delete reconciliation.
- `prune [PATH] [--all] [--dry-run]` and `remove --query TEXT [--cwd PATH] [--dry-run]`.
- Adversarial regressions, deterministic initialization interleaving, direct-library validation tests,
  real Rust-to-Python adapter integration and platform smoke coverage.

### Migration
- The file schema remains `memhooks/v2`; update CLI and adapter together.
- `find_root`, `inheritance_chain`, `discover_hooks` and `resolver::resolve_parsed` now return `Result`.
- `ParseError.code` is an owned `String`; use `.into()` when constructing it from a literal.
- Hook-file symlinks (including in-root links) are no longer accepted.
- Legacy generated query identities are migrated on the next relevant file event.

'''
p = r/'CHANGELOG.md'
s = p.read_text()
first = s.find('\n## ')
p.write_text(s[:first+1]+release+s[first+1:])
contract = '''# Runtime plan, reliability and 0.6 migration

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
'''
(r/'docs/runtime-plan.md').write_text(contract)
for name in ['docs/rust-library.md', 'docs/integrating-an-agent.md', 'docs/cli.md', 'hooks/hermes/README.md']:
    p = r/name
    p.write_text(p.read_text()+'''\n\n## 0.6 runtime and maintenance additions

See [the runtime contract and migration guide](https://github.com/AronAxe/MemHooks/blob/main/docs/runtime-plan.md)
for `memhooks/plan-v1`, session-local `active_files`/`active_roles`, exact emitted
context limits, `prune`/`remove` previews, fallible Rust APIs and the hook/lock
symlink policy. Install CLI 0.6.0 and the matching adapter together.
''')
p = r/'SECURITY.md'
p.write_text(p.read_text()+'''\n\n## 0.6 filesystem and routing boundary

Resolution and maintenance require semantic validation, not merely a supported
schema string. Hook/lock symlinks and special files are rejected, including
in-root links; Unix/Windows no-follow final-component opens harden the read path.
This is not a filesystem sandbox against concurrent replacement of ancestor
directories/mount points. Hosts must secure the working filesystem and enforce
authorization independently. Runtime budgets retain constraints with their
queries or discard the routing plan; role hints never grant permissions.
''')
p = r/'docs/troubleshooting.md'
p.write_text(p.read_text()+'''\n\n## 0.6 diagnostics and compatibility

An unversioned binary is not compatible with the 0.6 adapter. Update both when
`unsupported plan version` appears. `MH029` rejects hook/lock symlinks and special
files; replace links with ordinary project-local hooks. `MH030` means discovery
could not inspect the requested tree: fix the filesystem error rather than
treating the scan as clean. Invalid optional-hook settings return `{}` with a
stderr diagnostic. See [runtime settings and migration](runtime-plan.md).
''')
(r/'wiki/Release-0.6.0.md').write_text('# MemHooks 0.6.0\n\n'+intro.replace('## v0.6.0: reliability and task-aware recall\n\n','').replace('(docs/runtime-plan.md)','(https://github.com/AronAxe/MemHooks/blob/main/docs/runtime-plan.md)'))
for name in ['Home.md', 'Hermes-Integration.md', 'Agent-Maintenance.md', 'Security-Model.md', 'CLI-Cookbook.md']:
    p = r/'wiki'/name
    s = p.read_text()
    first = s.find('\n')
    note = '\n\n> **0.6.0 update:** See [[Release-0.6.0]] for validated routing, canonical paths,\n> active-file/role input, exact context budgets, generated-cue cleanup and API migration.\n'
    p.write_text(s[:first+1]+note+s[first+1:])
p = r/'wiki/_Sidebar.md'
p.write_text(p.read_text()+'\n- [[Release-0.6.0]]\n')
p = r/'wiki/Hermes-Integration.md'
p.write_text(p.read_text()+'''\n\n### Task-aware pre-LLM events (0.6)

The host may supply `active_files: ["src/auth/login.py"]` and
`active_roles: ["reviewer"]` in the pre-LLM event. The host retains the
session-local activity set. No extra LLM call or global cross-session cache
is introduced. Relevant file scopes are resolved before budgeting, sibling
constraints remain separate, and role/override/budget omissions are observable.
Without these fields cwd routing remains available.
''')
p = r/'wiki/Agent-Maintenance.md'
p.write_text(p.read_text()+'''\n\n### Cleanup commands (0.6)

`memhooks prune --all --dry-run` previews stale generated file cues; remove
`--dry-run` to apply. `memhooks remove --query "Obsolete local cue" --dry-run`
previews removal from the nearest local hook. These operations never delete
backend memories. Explicit rename/delete file events also reconcile generated
resources. Missing files are cleanup inputs only, not newly created anchors.
''')
p = r/'docs/publishing.md'
s = p.read_text().replace('it is an implementation hardening release.', 'it adds reliability fixes, task-aware routing and cue lifecycle support.')
s = s.replace('CI also checks the declared Rust 1.78 MSRV and Python 3.10/3.12 compatibility.', 'CI also checks Rust 1.78, Python 3.10/3.12, real-binary Python integration, and Windows/macOS Rust tests.')
p.write_text(s)
