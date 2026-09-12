# Rust library guide

The `memhooks` crate exposes the same parser, resolver, validator, and **frontmatter maintainer** used by the reference CLI. Agent runtimes can call the library directly instead of spawning the CLI or reimplementing root discovery, inheritance, role routing, or file maintenance.

The v0.5.1 library implements **`memhooks/v2`**. The core is backend-neutral; provider-native configuration is carried as opaque YAML under `backends.<provider>`.

## Add the dependency

```toml
[dependencies]
memhooks = "0.5.1"
```

or:

```bash
cargo add memhooks@0.5.1
```

## Resolve effective context

The main runtime entry point is [`resolve`]:

```rust,no_run
use memhooks::resolve;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resolved = resolve(Path::new("backend/auth"))?;

    println!("root: {}", resolved.root.display());
    for source in &resolved.sources {
        println!("source: {}", source.display());
    }
    println!("providers: {:?}", resolved.backends.keys().collect::<Vec<_>>());
    println!("guidance blocks: {}", resolved.guidance.len());
    Ok(())
}
```

`resolve`:

1. rejects nonexistent target paths;
2. uses the canonical MemHooks root semantics;
3. discovers `MEMHOOKS.md` files root → target;
4. rejects unsupported schemas rather than silently reinterpreting them;
5. applies `inherits: false` cutoffs;
6. merges backend-neutral core fields;
7. structurally merges opaque provider namespaces;
8. preserves source provenance for queries, entities, resources, and body guidance.

## Apply active roles

Use [`ResolvedHook::effective_queries`] after resolution:

```rust,no_run
use memhooks::resolve;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resolved = resolve(Path::new("backend/auth"))?;
    let active_roles = vec!["reviewer".to_string()];

    for query in resolved.effective_queries(&active_roles) {
        println!("query: {}", query.query);
        println!("priority: {:?}", query.priority);
        println!("roles: {:?}", query.roles);
        println!("entities: {:?}", query.entities);
        println!("resources: {:?}", query.resources);
        println!("tags: {:?}", query.tags);
        println!("backend namespaces: {:?}", query.backends.keys());
    }

    Ok(())
}
```

Role behavior:

- empty active-role list → preserve role-restricted queries;
- non-empty active-role list → a restricted query survives when any declared role exactly matches any active role;
- unrestricted queries always survive.

When the same trimmed query text appears in a parent and child hook, the **more-local query replaces the parent query and its metadata**. The resolver will not issue the same question twice with contradictory priorities.

## Provider namespaces

Provider data uses [`BackendMap`], a string-keyed map of `serde_yaml_ng::Value`.

Example hook:

```yaml
backends:
  mem0:
    filters:
      user_id: alice
    threshold: 0.1

recall_queries:
  - query: "What changed?"
    backends:
      mem0:
        top_k: 5
        rerank: true
```

The effective query gets a structurally merged Mem0 mapping containing the inherited filter/threshold plus query-local `top_k`/`rerank`. The crate deliberately does not know what those Mem0 keys mean.

[`merge_backend_maps`] is public for adapters/tooling that need exactly the same structural merge behavior:

```rust
use memhooks::{merge_backend_maps, BackendMap};

let mut scope = BackendMap::new();
let query = BackendMap::new();
merge_backend_maps(&mut scope, &query);
```

## Parse without resolving inheritance

Use [`parse_hook`] for a file on disk. This example is `no_run` because it expects a real `MEMHOOKS.md` at runtime:

```rust,no_run
use memhooks::parse_hook;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hook = parse_hook(Path::new("MEMHOOKS.md"))?;
    println!("schema: {:?}", hook.frontmatter.schema);
    Ok(())
}
```

Use [`parse_hook_str`] for in-memory source:

```rust
use memhooks::parse_hook_str;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"---
schema: memhooks/v2
recall_queries:
  - "What decisions matter here?"
backends:
  mem0:
    top_k: 8
---
"#;

    let hook = parse_hook_str(Path::new("virtual/MEMHOOKS.md"), source)?;
    assert_eq!(hook.frontmatter.recall_queries.len(), 1);
    assert!(hook.frontmatter.backends.contains_key("mem0"));
    Ok(())
}
```

Parsing is tolerant enough to keep malformed structured entries available to the validator. A typo such as `quer:` therefore produces a targeted diagnostic instead of aborting the entire file as an internal enum-deserialization error.

## Validate hooks

One file:

```rust,no_run
use memhooks::validate_file;
use std::path::Path;

fn main() {
    for diagnostic in validate_file(Path::new("MEMHOOKS.md")) {
        eprintln!("{}: {}", diagnostic.code, diagnostic.message);
    }
}
```

Repository/tree:

```rust
use memhooks::{validate_path, Severity};
use std::path::Path;

let diagnostics = validate_path(Path::new("."), true);
let has_errors = diagnostics
    .iter()
    .any(|diagnostic| diagnostic.severity == Severity::Error);
assert!(!has_errors);
```

Diagnostics use locations from the YAML frontmatter AST, so repeated keys such as two different `priority:` entries receive the correct line/column instead of the first textual match.

## Maintain hooks through the same data model

The library now owns maintenance as well as reading. This is the important v0.5.1 invariant: **what the maintainer writes is immediately visible through `resolve`.**

Initialize a project:

```rust,no_run
use memhooks::init;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hook = init(Path::new("."))?;
    println!("{}", hook.display());
    Ok(())
}
```

Add or enrich a semantic retrieval cue:

```rust,no_run
use memhooks::{add_note, NoteInput};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    add_note(
        Path::new("."),
        NoteInput {
            query: "Why did authentication change after the outage?".into(),
            priority: Some(0.9),
            roles: vec!["reviewer".into()],
            tags: vec!["security".into()],
            ..NoteInput::default()
        },
    )?;
    Ok(())
}
```

Consume a runtime `post_tool_call` event. This example is `no_run` because it intentionally demonstrates a write-capable maintenance API:

```rust,no_run
use memhooks::handle_event;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event = json!({
        "hook_event_name": "post_tool_call",
        "cwd": ".",
        "tool_input": { "path": "src/lib.rs" }
    });
    let _writes = handle_event(&event)?;
    Ok(())
}
```

The maintainer:

- writes `recall_queries`/resources directly into YAML frontmatter;
- only treats explicit path-bearing tool-input fields as automatic file anchors;
- requires anchored files to exist inside the canonical project root;
- preserves opaque/unknown fields when enriching existing structured queries;
- takes an exclusive sibling lock;
- writes through an atomic same-directory temporary replacement.

The Python `scripts/memhooks_update.py` file is only a compatibility launcher to the `memhooks` binary, not a second parser or persistence implementation.

## Important exported types/functions

### Model

- [`BackendMap`]
- [`HookFrontmatter`]
- [`RecallQuery`]
- [`StructuredRecallQuery`]
- [`QueryCondition`]
- [`Entity`]
- [`StructuredEntity`]
- [`Resource`]
- [`StructuredResource`]

### Parsing

- [`parse_hook`]
- [`parse_hook_str`]
- [`render_hook`]
- [`ParsedHook`]
- [`ParseError`]
- [`SourceLocation`]

### Resolution

- [`find_root`]
- [`inheritance_chain`]
- [`resolve`]
- [`merge_backend_maps`]
- [`ResolvedHook`]
- [`EffectiveQuery`]
- [`Sourced`]
- [`HOOK_FILENAME`]

### Validation

- [`discover_hooks`]
- [`validate_file`]
- [`validate_parsed`]
- [`validate_path`]
- [`Diagnostic`]
- [`Severity`]

### Maintenance

- [`init`]
- [`add_note`]
- [`handle_event`]
- [`NoteInput`]
- [`MaintainerError`]

## Runtime responsibilities outside the crate

The crate deliberately stops before memory retrieval. The host runtime remains responsible for:

- identifying the configured/authorized memory backend;
- interpreting its own `backends.<provider>` namespace;
- optionally validating provider-native configuration;
- translating effective queries into backend-native retrieval;
- applying credentials/access/trust policy;
- budgeting returned context;
- deciding prompt placement;
- retaining memory provenance.

This boundary keeps `memhooks/v2` backend-neutral while still giving runtimes one normative parser/resolver/maintainer implementation.
