# Rust library guide

The `memhooks` crate exposes the parser, resolver, model, and validator used by the reference CLI. Agent runtimes can call the library directly instead of spawning the CLI or reimplementing filesystem inheritance and role routing.

The v0.5.0 library implements **`memhooks/v2`**, whose core is backend-neutral. Provider-native configuration is carried as opaque YAML under `backends.<provider>`.

## Add the dependency

```toml
[dependencies]
memhooks = "0.5.0"
```

or:

```bash
cargo add memhooks@0.5.0
```

## Resolve effective context

The main runtime entry point is [`resolve`]:

```rust
use memhooks::resolve;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resolved = resolve(Path::new("backend/auth"))?;

    println!("root: {}", resolved.root.display());
    for source in &resolved.sources {
        println!("source: {}", source.display());
    }

    println!("providers: {:?}", resolved.backends.keys().collect::<Vec<_>>());
    Ok(())
}
```

`resolve`:

1. finds the repository/hook root;
2. discovers `MEMHOOKS.md` files root → target;
3. rejects unsupported schemas rather than silently reinterpreting them;
4. applies `inherits: false` cutoffs;
5. merges backend-neutral core fields;
6. structurally merges opaque provider namespaces;
7. preserves source provenance.

## Apply active roles

Use [`ResolvedHook::effective_queries`] after resolution:

```rust
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

`effective_queries` also:

- supplements scope entities with query-local entities;
- supplements scope resources with query-local resources;
- supplements scope tags with query-local tags;
- overlays query-local backend namespaces on the resolved scope-level backend configuration.

## Provider namespaces

Provider data uses [`BackendMap`], which is a string-keyed map of `serde_yaml::Value`.

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

The effective query gets a structurally merged Mem0 mapping containing the inherited filter/threshold plus query-local `top_k`/`rerank`.

The crate deliberately does not know what any of those Mem0 keys mean.

### Structural merge helper

[`merge_backend_maps`] is public for adapters/tooling that need exactly the same merge behavior:

```rust
use memhooks::{merge_backend_maps, BackendMap};

let mut scope = BackendMap::new();
let query = BackendMap::new();
merge_backend_maps(&mut scope, &query);
```

Merge rule:

- mapping + mapping → recursive merge;
- a local scalar/list/other value → replace the parent value.

This avoids inventing additive semantics for provider-owned sequences.

## Parse without resolving inheritance

Use [`parse_hook`] for a file on disk:

```rust
use memhooks::parse_hook;
use std::path::Path;

let hook = parse_hook(Path::new("MEMHOOKS.md"))?;
println!("schema: {:?}", hook.frontmatter.schema);
```

Use [`parse_hook_str`] for in-memory source:

```rust
use memhooks::parse_hook_str;
use std::path::Path;

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
```

Parsing and validation are separate operations. `parse_hook`/`parse_hook_str` deserialize the file; validation reports unsupported schema/core mistakes. `resolve` enforces the supported v2 schema for runtime use.

## Validate hooks

One file:

```rust
use memhooks::validate_file;
use std::path::Path;

for diagnostic in validate_file(Path::new("MEMHOOKS.md")) {
    eprintln!("{}: {}", diagnostic.code, diagnostic.message);
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
```

The second argument to `validate_path` corresponds to CLI `--all` behavior.

The generic validator checks core semantics and that every provider namespace contains a mapping/object. It does **not** validate arbitrary provider-internal keys.

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
- [`ParsedHook`]
- [`ParseError`]

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

## Runtime responsibilities outside the crate

The crate deliberately stops before memory retrieval. The host runtime remains responsible for:

- identifying the configured/authorized memory backend;
- interpreting its own `backends.<provider>` namespace;
- optionally validating provider-native configuration;
- translating effective queries into backend-native retrieval;
- applying credentials/access/trust policy;
- budgeting returned context;
- deciding prompt placement;
- retaining memory provenance;
- maintaining semantic routing cues when appropriate.

This boundary is what keeps `memhooks/v2` genuinely backend-neutral.
