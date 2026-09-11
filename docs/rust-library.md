# Rust library guide

The `memhooks` crate exposes the same parser, resolver, and validator used by the reference CLI. Agent runtimes can call the library directly instead of spawning the `memhooks` binary or reimplementing the protocol.

> Registry note: the crate is package-tested and crates.io-ready. Until the first authenticated crates.io publication is completed, depend on the GitHub repository/tag rather than the registry.

## Add the dependency

Current GitHub dependency:

```toml
[dependencies]
memhooks = { git = "https://github.com/AronAxe/MemHooks", tag = "v0.4.1" }
```

After the first crates.io publication:

```toml
[dependencies]
memhooks = "0.4.1"
```

## Resolve effective context

The main entry point for runtime integration is [`resolve`]:

```rust
use memhooks::resolve;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resolved = resolve(Path::new("backend/auth"))?;

    println!("root: {}", resolved.root.display());
    for source in &resolved.sources {
        println!("source: {}", source.display());
    }

    Ok(())
}
```

`resolve`:

1. finds the repository/hook root;
2. discovers `MEMHOOKS.md` files root to target;
3. applies `inherits: false` cutoffs;
4. merges fields using the `memhooks/v1` semantics;
5. preserves the source path for queries, entities, and free-form guidance.

## Apply active roles

Once a hook is resolved, obtain the effective query set with [`ResolvedHook::effective_queries`]:

```rust
use memhooks::resolve;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resolved = resolve(Path::new("backend/auth"))?;
    let active_roles = vec!["reviewer".to_string()];

    for query in resolved.effective_queries(&active_roles) {
        println!("{}", query.query);
        println!("priority: {:?}", query.priority);
        println!("roles: {:?}", query.roles);
        println!("memory types: {:?}", query.memory_types);
        println!("connections: {:?}", query.connection_types);
    }

    Ok(())
}
```

If `active_roles` is empty, role-restricted queries are preserved. If active roles are supplied, a restricted query survives when any declared role exactly matches any active role.

`effective_queries` also resolves query-local defaults:

- query-local `memory_types` override scope-wide defaults for that query;
- query-local `connection_types` override scope-wide defaults for that query;
- query-local entities supplement the merged top-level entities.

## Parse without resolving inheritance

Use [`parse_hook`] for a file on disk:

```rust
use memhooks::parse_hook;
use std::path::Path;

let hook = parse_hook(Path::new("MEMHOOKS.md"))?;
println!("schema: {:?}", hook.frontmatter.schema);
```

Use [`parse_hook_str`] when the source text already exists in memory:

```rust
use memhooks::parse_hook_str;
use std::path::Path;

let source = r#"---
schema: memhooks/v1
recall_queries:
  - "What decisions matter here?"
---
"#;

let hook = parse_hook_str(Path::new("virtual/MEMHOOKS.md"), source)?;
assert_eq!(hook.frontmatter.recall_queries.len(), 1);
```

The path argument is retained for provenance and diagnostics even when parsing an in-memory string.

## Validate hooks

Validate one file:

```rust
use memhooks::validate_file;
use std::path::Path;

for diagnostic in validate_file(Path::new("MEMHOOKS.md")) {
    eprintln!("{}: {}", diagnostic.code, diagnostic.message);
}
```

Validate a directory or repository:

```rust
use memhooks::validate_path;
use std::path::Path;

let diagnostics = validate_path(Path::new("."), true);
let has_errors = diagnostics
    .iter()
    .any(|d| d.severity == memhooks::Severity::Error);
```

The second argument to `validate_path` corresponds to CLI `--all` behavior: when `true`, validation starts at the resolved repository root.

## Important exported types and functions

The crate currently re-exports:

### Model

- `HookFrontmatter`
- `RecallQuery`
- `StructuredRecallQuery`
- `QueryCondition`
- `Entity`
- `StructuredEntity`

### Parsing

- `parse_hook`
- `parse_hook_str`
- `ParsedHook`
- `ParseError`

### Resolution

- `find_root`
- `inheritance_chain`
- `resolve`
- `ResolvedHook`
- `EffectiveQuery`
- `Sourced<T>`
- `HOOK_FILENAME`

### Validation

- `discover_hooks`
- `validate_file`
- `validate_parsed`
- `validate_path`
- `Diagnostic`
- `Severity`

## Runtime responsibilities outside the crate

The crate deliberately stops before memory retrieval. A host runtime remains responsible for:

- identifying the configured memory backend;
- translating effective queries into backend-native searches;
- applying trust/access policy;
- budgeting returned context;
- deciding prompt placement;
- retaining provenance from the memory backend;
- deciding whether and how semantic notes are written.

This boundary keeps the reference implementation portable and auditable.
