# MemHooks 0.6.0

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
See [runtime contract and migration](https://github.com/AronAxe/MemHooks/blob/main/docs/runtime-plan.md).

