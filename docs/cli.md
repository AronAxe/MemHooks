# CLI reference

The `memhooks` binary is the reference command-line resolver and validator for `MEMHOOKS.md`.

## Commands

```text
memhooks validate [PATH] [--all] [--format human|json|sarif]
memhooks explain [PATH] [--role ROLE ...] [--format human|json]
```

`PATH` defaults to the current directory.

## `memhooks validate`

Validate one file, a subtree, or an entire repository.

### Validate the current subtree

```bash
memhooks validate
```

### Validate a specific file or directory

```bash
memhooks validate backend/auth
memhooks validate backend/auth/MEMHOOKS.md
```

### Validate the whole repository

```bash
memhooks validate --all
```

With `--all`, MemHooks resolves the repository root and scans below it while respecting Git ignore rules.

### Output formats

Human-readable output is the default:

```bash
memhooks validate --all --format human
```

JSON is suitable for custom tooling:

```bash
memhooks validate --all --format json
```

SARIF is suitable for CI/code-scanning pipelines:

```bash
memhooks validate --all --format sarif
```

### Exit status

- `0` — no validation errors. Warnings may still be present.
- `1` — at least one validation error exists.

This makes `memhooks validate --all` safe to use directly as a CI gate.

## `memhooks explain`

`explain` shows the effective root-to-leaf configuration for a target path.

```bash
memhooks explain backend/auth
```

Human output includes:

- resolved repository/root path;
- target path;
- source `MEMHOOKS.md` files in inheritance order;
- active roles, when supplied;
- effective recall queries and their source file;
- explicit query priority when present;
- role applicability;
- entity and exclusion counts.

### Role filtering

```bash
memhooks explain backend/auth --role reviewer
```

Multiple active roles can be supplied:

```bash
memhooks explain backend/auth --role reviewer --role architect
```

Role matching is exact-string OR matching. A query with no `when.roles` applies to every role. If no active role is supplied, role-restricted queries are preserved rather than discarded.

### JSON explain output

```bash
memhooks explain backend/auth --role reviewer --format json
```

The JSON object exposes resolved scope fields, sources, active roles, effective queries, entities, tags, exclusions, mental models, and Knowledge Pages. It is intended as a stable handoff format for adapters and developer tooling.

## CI example

```yaml
- name: Validate MemHooks
  run: memhooks validate --all
```

To emit SARIF for another CI step:

```bash
memhooks validate --all --format sarif > memhooks.sarif
```

## What the CLI does not do

The CLI does not:

- contact Hindsight, OpenViking, Honcho, or another memory backend;
- execute arbitrary commands from a hook;
- create or modify memories;
- decide prompt privilege/injection level;
- start a background daemon.

It is deliberately limited to parsing, validation, inheritance resolution, role filtering, and explanation.
