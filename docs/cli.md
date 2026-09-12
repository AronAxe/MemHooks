# CLI reference

The `memhooks` binary is the reference resolver, validator **and maintainer** for `MEMHOOKS.md` using `memhooks/v2`.

## Commands

```text
memhooks validate [PATH] [--all] [--format human|json|sarif]
memhooks explain [PATH] [--role ROLE ...] [--format human|json]
memhooks init [PATH]
memhooks note --cwd PATH --query TEXT [routing options]
memhooks event
```

## `memhooks init`

Enable MemHooks at the canonical project root:

```bash
memhooks init .
memhooks init /path/to/project
```

The command creates a root `MEMHOOKS.md` with `schema: memhooks/v2`. It uses the same canonical root semantics as the resolver and runtime adapters.

## `memhooks note`

Add or enrich one structured retrieval cue directly in YAML frontmatter:

```bash
memhooks note \
  --cwd "$PWD" \
  --query "Why did authentication change after the outage?" \
  --priority 0.9 \
  --role reviewer \
  --entity '{"name":"Authentication","type":"CONCEPT","salience":0.95}' \
  --resource '{"name":"auth-postmortem","kind":"postmortem","salience":0.9}' \
  --tag security \
  --backends '{"mem0":{"top_k":8}}'
```

Options:

- `--priority 0.0..1.0`
- repeatable `--role`
- repeatable `--entity` as a plain name or JSON object
- repeatable `--resource` as a plain name or JSON object
- repeatable `--tag`
- `--backends` as one JSON object containing opaque provider mappings

If the same trimmed query text already exists in the target hook, the maintainer enriches that query rather than appending a second copy. Unknown/future fields and provider data are preserved.

Maintenance uses an exclusive lock and atomic same-directory replacement.

## `memhooks event`

Consume one runtime `post_tool_call` JSON event from stdin:

```bash
printf '%s' '{"hook_event_name":"post_tool_call","cwd":"/repo","tool_input":{"path":"src/lib.rs"}}' \
  | memhooks event
```

Automatic path anchors are deliberately conservative. Only explicit path-bearing tool-input fields are considered, and candidates must resolve to existing files inside the canonical project root. Free-form file contents are not regex-mined for dotted strings or URLs.

Anchors are stored in YAML frontmatter and are immediately visible through the reference resolver.

## `memhooks validate`

Validate one file, a subtree, or an entire repository:

```bash
memhooks validate
memhooks validate backend/auth
memhooks validate backend/auth/MEMHOOKS.md
memhooks validate --all
```

With `--all`, MemHooks resolves the canonical repository root and scans below it while respecting Git ignore rules.

An explicitly supplied nonexistent path is an **error (`MH024`)**, not an empty success.

### Output formats

```bash
memhooks validate --all --format human
memhooks validate --all --format json
memhooks validate --all --format sarif
```

Human is the default. JSON is suitable for custom tooling. SARIF uses validation-root-relative artifact URIs and includes rule metadata/help for emitted `MHxxx` diagnostics so code-scanning systems can map findings back to repository files.

### Exit status

- `0` — no validation errors; warnings may still be present.
- `1` — at least one validation error exists.

The validator checks the v2 core plus the structural shape of provider namespaces. Provider-internal keys remain provider-owned.

Malformed structured entries are linted as far as possible. A typo such as `quer:` produces targeted query diagnostics instead of collapsing the whole file into a generic enum-deserialization error.

## `memhooks explain`

Resolve the effective root-to-leaf routing configuration for a target path:

```bash
memhooks explain backend/auth
```

Human output includes:

- canonical root and target;
- source hook files in inheritance order;
- active roles when supplied;
- effective recall queries with source, priority, roles, and provider namespace names;
- entity/resource/exclusion counts;
- source-attributed Markdown guidance blocks.

A nonexistent target is an error.

### Role filtering

```bash
memhooks explain backend/auth --role reviewer
memhooks explain backend/auth --role reviewer --role architect
```

Role matching is exact-string OR matching. A query with no `when.roles` applies universally. If no active roles are supplied, restricted queries are preserved rather than discarded.

### JSON adapter handoff

```bash
memhooks explain backend/auth --role reviewer --format json
```

The JSON form serializes the actual resolved structure instead of manually mirroring selected fields. It contains:

- `root`, `target`, `sources`
- `scope`, `sensitivity`
- raw resolved `recall_queries`
- `entities`, `resources`, `tags`, `exclude`
- resolved scope-level `backends`
- source-attributed `guidance`
- `active_roles`
- `effective_queries` after role filtering and provider/core overlays

This is the canonical handoff used by the bundled Hermes adapter.

## Query override behavior

Across inheritance, trimmed query text is the query identity. A more-local query with the same text replaces the parent query and its query-local metadata. The resolved plan therefore does not issue the same question twice with contradictory priorities.

## Canonical root behavior

All reference components share the same root rules:

1. explicit containing `MEMHOOKS_ROOT` when supplied;
2. otherwise nearest Git root as a hard boundary;
3. outside Git only, highest ancestor containing `MEMHOOKS.md`.

## CI examples

Validate:

```yaml
- name: Validate MemHooks
  run: memhooks validate --all
```

Emit SARIF:

```bash
memhooks validate --all --format sarif > memhooks.sarif
```

## What the CLI does not do

The CLI does not:

- contact Hindsight, Mem0, OpenViking, Honcho, or another memory backend;
- interpret arbitrary provider-native keys inside a namespace;
- execute commands declared by hook files;
- create or alter memory-backend facts;
- decide prompt authority;
- start a daemon.

It owns the protocol mechanics—parsing, maintenance, validation, inheritance, role filtering, provider-configuration preservation/merge, explanation, and provenance—not the memory provider itself.