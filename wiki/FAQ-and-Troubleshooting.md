# FAQ and Troubleshooting

## Is MemHooks a memory database?

No.

MemHooks stores **retrieval cues and routing metadata**. The actual memories remain in the configured memory backend.

## Does MemHooks require Hindsight?

No.

`memhooks/v2` is backend-neutral. Hindsight, Mem0, OpenViking, Honcho, and other systems live behind provider namespaces.

See [[Memory Backends]].

## Does MemHooks call every backend mentioned in a hook?

No.

Namespace presence is not backend selection. The host runtime chooses the configured/authorized backend and interprets only that namespace.

## Do users have to maintain all these files manually?

No—and the normal design assumes they do not.

The agent/runtime should create, refine, and prune routing cues automatically. See [[Agent Maintenance]].

## Why use files at all?

Because the filesystem already encodes project locality. A cue next to `backend/auth` is a deterministic signal that it matters when work happens there.

The hook is not the memory; it is a local pointer toward relevant memory.

## What is the difference between `priority` and `salience`?

- query `priority` → importance of satisfying that recall request under context pressure;
- entity/resource `salience` → importance of that cue;
- backend relevance/confidence → provider-native score for a returned result.

They are deliberately separate.

See [[Weighted Retrieval and Roles]].

## Why did my parent hook disappear?

Check for:

```yaml
inherits: false
```

in a more-local hook. It intentionally cuts off ancestors above that hook.

## Why did my parent query disappear?

If a child declares the **same normalized query text**, the local declaration overrides the parent version.

This prevents duplicate retrieval of the same question with contradictory metadata.

See [[Inheritance and Scoping]].

## Why is a role-restricted query still visible?

If you do not supply active roles, the resolver preserves role-restricted queries.

Unknown role information is not the same as a known non-match.

Try:

```bash
memhooks explain . --role reviewer
```

## Can `MEMHOOKS.md` execute commands?

No.

Hook files are retrieval metadata, not an execution manifest.

See [[Security Model]].

## Can a hook grant itself system prompt authority?

No.

Repository-controlled text cannot elevate its own trust level. Prompt placement and authority belong to the host runtime.

## Why does `memhooks explain` reject my old file?

The current reference resolver expects:

```yaml
schema: memhooks/v2
```

Unsupported schemas fail explicitly instead of being silently reinterpreted.

## Where do Hindsight `memory_types` or Mem0 `top_k` go?

Under their provider namespace:

```yaml
backends:
  hindsight:
    memory_types: [experience]

  mem0:
    top_k: 8
```

They are not universal MemHooks fields.

## Why is a provider list replacing the parent list?

Because provider namespaces are opaque to the core.

MemHooks knows how to recursively merge mappings, but it cannot know whether a provider-owned list is additive, ordered, exclusive, or something else. Therefore a more-local list replaces the parent list.

## Why does MemHooks ignore a touched-looking path in source code?

Automatic maintenance intentionally considers explicit path-bearing tool fields rather than scraping arbitrary source contents.

A string such as:

```text
requests.get("http://x.com/a.js")
```

should not become a file anchor merely because it contains dots and slashes.

## Why must automatic path anchors point to existing files?

Because otherwise arbitrary dotted identifiers and malformed tokens can pollute routing. The deterministic maintainer verifies actual filesystem paths inside the project boundary.

## Can a hook outside my Git repo affect my project?

Not inside a Git repository. The `.git` root is a hard containment boundary for the reference resolver and maintainer.

## `memhooks validate --all` says no hooks were found

Make sure MemHooks is initialized in the intended project:

```bash
memhooks init .
memhooks validate --all
```

If you passed a path explicitly, verify it exists and points at the project you intended.

## Why does a nonexistent path fail now?

Because silently treating a typo as an empty/clean project is dangerous. Explicit nonexistent targets should be errors.

## Where did the Markdown note block go?

Structured notes belong in YAML frontmatter now.

The Markdown body is guidance only. This gives the writer and resolver one shared structured representation.

## Does `explain --format json` include body guidance?

Yes. The resolved handoff includes source-attributed guidance so runtime adapters do not need to independently re-read raw hook files.

## Can concurrent tool calls corrupt a hook?

The reference maintainer uses locking plus atomic replacement to avoid lost updates/truncated files during concurrent maintenance.

## Why is `Cargo.lock` committed?

The project ships a binary as well as a library. Committing the lockfile makes repository/CI builds reproducible and allows CI to use `--locked`.

## Why does CI test Rust 1.78 specifically?

Because `Cargo.toml` declares a Rust 1.78 minimum. A minimum version claim is only useful if CI actually builds against it.

## Where is the exact protocol definition?

Not this Wiki.

The normative specification is:

[references/memhooks-format.md](https://github.com/AronAxe/MemHooks/blob/main/references/memhooks-format.md)

If a Wiki explanation and that file conflict, the specification wins.

## Still stuck?

Useful diagnostics:

```bash
memhooks validate --all
memhooks explain . --format json
```

For implementation-level diagnostic codes, see the versioned repository documentation:

[docs/troubleshooting.md](https://github.com/AronAxe/MemHooks/blob/main/docs/troubleshooting.md)