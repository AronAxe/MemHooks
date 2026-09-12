#![doc = include_str!("../docs/rust-library.md")]

pub mod maintainer;
pub mod model;
pub mod parser;
pub mod resolver;
pub mod validator;

pub use maintainer::{add_note, handle_event, init, MaintainerError, NoteInput};
pub use model::{
    BackendMap, Entity, HookFrontmatter, QueryCondition, RecallQuery, Resource, StructuredEntity,
    StructuredRecallQuery, StructuredResource,
};
pub use parser::{
    parse_hook, parse_hook_str, render_hook, require_v2_schema, ParseError, ParsedHook,
    SourceLocation,
};
pub use resolver::{
    find_root, inheritance_chain, merge_backend_maps, resolve, EffectiveQuery, ResolvedHook,
    Sourced, HOOK_FILENAME,
};
pub use validator::{
    discover_hooks, validate_file, validate_parsed, validate_path, Diagnostic, Severity,
};
