# CLI reference

The `memhooks` binary is the reference command-line resolver and validator for `MEMHOOKS.md` using `memhooks/v2`.

## Commands

```text
memhooks validate [PATH] [--all] [--format human|json|sarif]
memhooks explain [PATH] [--role ROLE ...] [--format human|json]
```

`PATH` defaults to the current directory.

## `memhooks validate`

Validate one file, a subtree, or an entire repository.

```bash
memhooks validate
memhooks validate backend/auth
memhooks validate backend/auth/MEMHOOKS.md
memhooks validate --all
```

With `--all`, MemHooks resolves the repository root and scans below it while respecting Git ignore rules.

### Output formats

```bash
memhooks validate --all --format human
memhooks validate --all --format json
memhooks validate --all --format sarif
```

Human is the default. JSON is suitable for tooling. SARIF is suitable for code-scanning/CI pipelines.

### Exit status

- `0` — no validation errors; warnings may still be present.
- `1` — at least one validation error exists.

The validator checks the v2 **core** plus the structural shape of backend namespaces. It deliberately does not validate arbitrary provider-internal keys inside `backends.<provider>`.

A provider adapter may run additional provider-specific validation separately.

## `memhooks explain`

`explain` resolves the effective root-to-leaf routing configuration for a target path.

```bash
memhooks explain backend/auth
```

Human output includes:

- resolved repository root;
- target path;
- source `MEMHOOKS.md` files in inheritance order;
- active roles when supplied;
- effective recall queries and source provenance;
- explicit query priority when present;
- role applicability;
- resolved backend namespace names;
- entity/resource/exclusion counts.

### Role filtering

```bash
memhooks explain backend/auth --role reviewer
memhooks explain backend/auth --role reviewer --role architect
```

Role matching is exact-string OR matching. A query with no `when.roles` applies universally. If no active roles are supplied, role-restricted queries are preserved rather than discarded.

### JSON explain output

```bash
memhooks explain backend/auth --role reviewer --format json
```

The JSON object exposes:

- `root`
- `target`
- `sources`
- `scope`
- `sensitivity`
- `entities`
- `resources`
- `tags`
- `exclude`
- resolved scope-level `backends`
- `active_roles`
- effective `recall_queries`

Each effective structured query carries its merged generic cues and effective provider namespaces. This makes the JSON form useful as a handoff from the reference resolver to runtime adapters.

## Provider configuration in explain output

The CLI **shows** provider configuration but does not interpret it.

For example:

```yaml
backends:
  mem0:
    top_k: 8
  hindsight:
    memory_types: [experience]
```

will appear in the resolved output as provider-owned data. The CLI does not decide what `top_k` or `memory_types` means.

## CI example

```yaml
- name: Validate MemHooks
  run: memhooks validate --all
```

To emit SARIF:

```bash
memhooks validate --all --format sarif > memhooks.sarif
```

## What the CLI does not do

The CLI does not:

- contact Hindsight, Mem0, OpenViking, Honcho, or another memory backend;
- validate arbitrary provider-native keys inside a provider namespace;
- execute commands declared by hook files;
- create or modify memories;
- decide prompt privilege/injection level;
- start a background daemon.

It is deliberately limited to parsing, core validation, inheritance resolution, role filtering, opaque provider configuration merging, and explanation.
