#![doc = include_str!("../docs/rust-library.md")]

pub mod model;
pub mod parser;
pub mod resolver;
pub mod validator;

pub use model::{
    BackendMap, Entity, HookFrontmatter, QueryCondition, RecallQuery, Resource, StructuredEntity,
    StructuredRecallQuery, StructuredResource,
};
pub use parser::{parse_hook, parse_hook_str, ParseError, ParsedHook};
pub use resolver::{
    find_root, inheritance_chain, merge_backend_maps, resolve, EffectiveQuery, ResolvedHook, Sourced,
    HOOK_FILENAME,
};
pub use validator::{
    discover_hooks, validate_file, validate_parsed, validate_path, Diagnostic, Severity,
};
