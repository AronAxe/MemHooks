#![doc = include_str!("../docs/rust-library.md")]

mod filesystem;
pub mod maintainer;
pub mod model;
pub mod parser;
pub mod resolver;
pub mod validator;

pub use maintainer::{
    add_note, handle_event, init, prune, remove_note, MaintainerError, NoteInput, PruneReport,
};
pub use model::{
    BackendMap, Entity, HookFrontmatter, QueryCondition, RecallQuery, Resource, StructuredEntity,
    StructuredRecallQuery, StructuredResource,
};
pub use parser::{
    parse_hook, parse_hook_str, render_hook, require_v2_schema, ParseError, ParsedHook,
    SourceLocation,
};
pub use resolver::{
    find_root, inheritance_chain, merge_backend_maps, resolve, EffectiveQuery, QueryOmission,
    ResolvedHook, Sourced, TargetContext, HOOK_FILENAME, PLAN_VERSION,
};
pub use validator::{
    discover_hooks, require_valid, validate_file, validate_parsed, validate_path, Diagnostic,
    Severity,
};
