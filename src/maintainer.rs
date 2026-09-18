use crate::filesystem::{open_regular_file, path_error, regular_file_exists};
use crate::model::{
    BackendMap, Entity, HookFrontmatter, RecallQuery, Resource, StructuredRecallQuery,
    StructuredResource,
};
use crate::parser::{parse_hook, parse_hook_str, render_hook, ParseError};
use crate::resolver::{find_root, merge_backend_maps, TargetContext, HOOK_FILENAME};
use crate::validator::{discover_hooks, require_valid};
use fs2::FileExt;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use thiserror::Error;

const AUTO_QUERY: &str = "What prior decisions, constraints, failures, fixes, rejected approaches, and unresolved issues involve files in this directory?";
const AUTO_TAG: &str = "memhooks:auto";
const PATH_KEYS: &[&str] = &[
    "path",
    "paths",
    "file",
    "files",
    "filename",
    "file_path",
    "filepath",
    "target",
    "source",
    "destination",
    "dest",
    "output",
    "output_path",
    "workdir",
    "cwd",
];
const SKIP_PARTS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    ".venv",
    "venv",
    "node_modules",
    "dist",
    "build",
    "target",
    "__pycache__",
    ".idea",
    ".vscode",
    ".pytest_cache",
    ".mypy_cache",
];

#[derive(Debug, Error)]
pub enum MaintainerError {
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("MemHooks is not enabled at the project root `{0}`")]
    NotEnabled(PathBuf),
    #[error("could not atomically replace `{path}`: {message}")]
    Persist { path: PathBuf, message: String },
}

#[derive(Debug, Clone, Default)]
pub struct NoteInput {
    pub query: String,
    pub priority: Option<f64>,
    pub roles: Vec<String>,
    pub entities: Vec<Entity>,
    pub resources: Vec<Resource>,
    pub tags: Vec<String>,
    pub backends: BackendMap,
}

pub fn init(cwd: &Path) -> Result<PathBuf, MaintainerError> {
    if !cwd.exists() {
        return Err(MaintainerError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("path does not exist: {}", cwd.display()),
        )));
    }
    let root = find_root(cwd)?;
    let hook = root.join(HOOK_FILENAME);

    let project_name = root
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("project");
    let body = format!(
        "# MemHooks\n\nRecall prior decisions, constraints, failures, fixes, rejected approaches, and unresolved issues concerning the `{project_name}` project before substantive changes.\n\nThe agent/runtime owns maintenance of concise retrieval cues. Provider-native routing belongs under `backends.<provider>` only when the active backend and control are actually known."
    );
    let frontmatter = HookFrontmatter {
        schema: Some("memhooks/v2".into()),
        inherits: Some(true),
        ..HookFrontmatter::default()
    };
    initialize_hook(&hook, &frontmatter, &body)?;
    Ok(hook)
}

pub fn add_note(cwd: &Path, input: NoteInput) -> Result<PathBuf, MaintainerError> {
    let context = TargetContext::new(cwd)?;
    let cwd = &context.directory;
    let root = enabled_root(cwd)?;
    if normalize_text(&input.query).is_empty() {
        return Err(path_error(cwd, "MH003", "recall query must not be empty").into());
    }
    let target_directory = nearest_hook_directory(cwd, &root)?.unwrap_or(root.clone());
    let hook = target_directory.join(HOOK_FILENAME);
    with_hook_lock(&hook, || {
        let (mut frontmatter, body) = load_or_default(&hook)?;
        upsert_query(&mut frontmatter.recall_queries, input);
        write_hook_atomic(&hook, &frontmatter, &body)
    })?;
    Ok(hook)
}

pub fn handle_event(payload: &JsonValue) -> Result<usize, MaintainerError> {
    if payload.get("hook_event_name").and_then(JsonValue::as_str) != Some("post_tool_call") {
        return Ok(0);
    }

    let cwd = payload
        .get("cwd")
        .and_then(JsonValue::as_str)
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir()?);
    let cwd = TargetContext::new(&cwd)?.directory;
    let root = match enabled_root(&cwd) {
        Ok(root) => root,
        Err(MaintainerError::NotEnabled(_)) => return Ok(0),
        Err(error) => return Err(error),
    };

    let mut candidates = Vec::new();
    collect_explicit_paths(
        payload.get("tool_input").unwrap_or(&JsonValue::Null),
        &mut candidates,
    );
    let mut relative_files = candidates
        .into_iter()
        .filter_map(|candidate| safe_event_file(&candidate, &cwd, &root))
        .collect::<Vec<_>>();
    relative_files.sort();
    relative_files.dedup();
    if relative_files.is_empty() {
        return Ok(0);
    }

    let mut grouped = BTreeMap::<PathBuf, Vec<PathBuf>>::new();
    for relative in relative_files {
        grouped
            .entry(relative.parent().unwrap_or(Path::new("")).to_path_buf())
            .or_default()
            .push(relative);
    }

    let max_auto_paths = match std::env::var("MEMHOOKS_AUTO_PATHS") {
        Ok(value) => value
            .parse::<usize>()
            .ok()
            .filter(|n| (1..=256).contains(n))
            .ok_or_else(|| {
                path_error(
                    &root,
                    "MH028",
                    "MEMHOOKS_AUTO_PATHS must be an integer between 1 and 256",
                )
            })?,
        Err(std::env::VarError::NotPresent) => 12,
        Err(error) => return Err(path_error(&root, "MH028", error.to_string()).into()),
    };
    let mut writes = 0;

    for (relative_directory, additions) in grouped {
        let directory = root.join(&relative_directory);
        if !directory.is_dir() {
            continue;
        }
        let hook = directory.join(HOOK_FILENAME);
        let additions = additions
            .into_iter()
            .filter(|path| root.join(path).is_file())
            .collect::<Vec<_>>();
        if additions.is_empty() && !regular_file_exists(&hook)? {
            continue;
        }
        let changed = with_hook_lock(&hook, || {
            let (mut frontmatter, body) = load_or_default(&hook)?;
            let before = frontmatter.clone();
            prune_auto_queries(&mut frontmatter.recall_queries, &root)?;
            if !additions.is_empty() {
                upsert_auto_query(
                    &mut frontmatter.recall_queries,
                    additions,
                    max_auto_paths,
                    &relative_directory,
                );
            }
            if frontmatter == before {
                return Ok(false);
            }
            write_hook_atomic(&hook, &frontmatter, &body)?;
            Ok(true)
        })?;
        writes += usize::from(changed);
    }

    Ok(writes)
}

fn enabled_root(cwd: &Path) -> Result<PathBuf, MaintainerError> {
    let root = find_root(cwd)?;
    let hook = root.join(HOOK_FILENAME);
    if regular_file_exists(&hook)? {
        require_valid(&parse_hook(&hook)?)?;
        Ok(root)
    } else {
        Err(MaintainerError::NotEnabled(root))
    }
}

fn nearest_hook_directory(cwd: &Path, root: &Path) -> Result<Option<PathBuf>, ParseError> {
    let start = if cwd.is_file() {
        cwd.parent().unwrap_or(cwd)
    } else {
        cwd
    };
    for directory in start.ancestors() {
        if !directory.starts_with(root) {
            break;
        }
        if regular_file_exists(&directory.join(HOOK_FILENAME))? {
            return Ok(Some(directory.to_path_buf()));
        }
        if directory == root {
            break;
        }
    }
    Ok(None)
}

fn load_or_default(path: &Path) -> Result<(HookFrontmatter, String), MaintainerError> {
    if regular_file_exists(path)? {
        let parsed = parse_hook(path)?;
        require_valid(&parsed)?;
        Ok((parsed.frontmatter, parsed.body))
    } else {
        Ok((
            HookFrontmatter {
                schema: Some("memhooks/v2".into()),
                inherits: Some(true),
                ..HookFrontmatter::default()
            },
            String::new(),
        ))
    }
}

fn upsert_query(queries: &mut Vec<RecallQuery>, input: NoteInput) {
    let normalized_query = normalize_text(&input.query);
    if normalized_query.is_empty() {
        return;
    }

    if let Some(existing) = queries
        .iter_mut()
        .find(|query| normalize_text(query.text()) == normalized_query)
    {
        let mut structured = match existing.clone() {
            RecallQuery::Simple(query) => StructuredRecallQuery {
                query,
                ..StructuredRecallQuery::default()
            },
            RecallQuery::Structured(value) => value,
        };
        merge_note_metadata(&mut structured, input);
        *existing = RecallQuery::Structured(structured);
    } else {
        let mut structured = StructuredRecallQuery {
            query: normalized_query,
            ..StructuredRecallQuery::default()
        };
        merge_note_metadata(&mut structured, input);
        queries.push(RecallQuery::Structured(structured));
    }
}

fn merge_note_metadata(target: &mut StructuredRecallQuery, input: NoteInput) {
    if let Some(priority) = input.priority {
        target.priority = Some(priority);
    }

    if !input.roles.is_empty() {
        let when = target.when.get_or_insert_with(Default::default);
        extend_unique(
            &mut when.roles,
            input.roles.into_iter().map(|role| normalize_text(&role)),
        );
    }
    extend_unique(&mut target.entities, input.entities.into_iter());
    extend_unique(&mut target.resources, input.resources.into_iter());
    extend_unique(
        &mut target.tags,
        input.tags.into_iter().map(|tag| normalize_text(&tag)),
    );
    merge_backend_maps(&mut target.backends, &input.backends);
}

fn upsert_auto_query(
    queries: &mut Vec<RecallQuery>,
    additions: Vec<PathBuf>,
    max_paths: usize,
    directory: &Path,
) {
    let scope = if directory.as_os_str().is_empty() {
        ".".to_string()
    } else {
        directory.to_string_lossy().replace('\\', "/")
    };
    let scoped_query = format!("{AUTO_QUERY} [scope: {scope}]");
    let owned = |query: &&RecallQuery| {
        query.tags().iter().any(|tag| tag == AUTO_TAG)
            && (query.text().trim() == AUTO_QUERY || query.text().trim() == scoped_query)
    };
    let mut structured = queries
        .iter()
        .find(owned)
        .cloned()
        .map(|query| match query {
            RecallQuery::Simple(query) => StructuredRecallQuery {
                query,
                ..StructuredRecallQuery::default()
            },
            RecallQuery::Structured(value) => value,
        })
        .unwrap_or_else(|| StructuredRecallQuery {
            query: scoped_query.clone(),
            ..StructuredRecallQuery::default()
        });

    structured.query = scoped_query;
    if !structured.tags.iter().any(|tag| tag == AUTO_TAG) {
        structured.tags.push(AUTO_TAG.into());
    }

    let mut resources = structured
        .resources
        .iter()
        .filter(|resource| matches!(resource, Resource::Structured(value) if value.kind.as_deref() == Some("file")))
        .cloned()
        .collect::<Vec<_>>();
    for addition in additions {
        let resource = Resource::Structured(StructuredResource {
            name: addition.to_string_lossy().replace('\\', "/"),
            kind: Some("file".into()),
            ..StructuredResource::default()
        });
        if !resources.contains(&resource) {
            resources.push(resource);
        }
    }
    if resources.len() > max_paths {
        resources.drain(0..resources.len() - max_paths);
    }

    structured
        .resources
        .retain(|resource| !matches!(resource, Resource::Structured(value) if value.kind.as_deref() == Some("file")));
    structured.resources.extend(resources);

    if let Some(existing) = queries.iter_mut().find(|query| {
        query.tags().iter().any(|tag| tag == AUTO_TAG)
            && (query.text().trim() == AUTO_QUERY || query.text().trim() == structured.query)
    }) {
        *existing = RecallQuery::Structured(structured);
    } else {
        queries.push(RecallQuery::Structured(structured));
    }
}

fn collect_explicit_paths(value: &JsonValue, output: &mut Vec<String>) {
    match value {
        JsonValue::Object(mapping) => {
            for (key, nested) in mapping {
                if PATH_KEYS.contains(&key.to_ascii_lowercase().as_str()) {
                    collect_path_values(nested, output);
                } else if nested.is_object() || nested.is_array() {
                    collect_explicit_paths(nested, output);
                }
            }
        }
        JsonValue::Array(values) => {
            for nested in values {
                collect_explicit_paths(nested, output);
            }
        }
        _ => {}
    }
}

fn collect_path_values(value: &JsonValue, output: &mut Vec<String>) {
    match value {
        JsonValue::String(value) => output.push(value.clone()),
        JsonValue::Array(values) => {
            for value in values {
                collect_path_values(value, output);
            }
        }
        _ => {}
    }
}

fn safe_existing_file(candidate: &str, cwd: &Path, root: &Path) -> Option<PathBuf> {
    if candidate.is_empty()
        || candidate.starts_with("http://")
        || candidate.starts_with("https://")
        || candidate.starts_with("git@")
    {
        return None;
    }

    let path = PathBuf::from(candidate);
    let path = if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    };
    let path = path.canonicalize().ok()?;
    if !path.is_file() || path.file_name()?.to_str()? == HOOK_FILENAME {
        return None;
    }
    let relative = path.strip_prefix(root).ok()?.to_path_buf();
    if relative
        .components()
        .any(|component| SKIP_PARTS.contains(&component.as_os_str().to_string_lossy().as_ref()))
    {
        return None;
    }
    Some(relative)
}

fn with_hook_lock<T>(
    hook: &Path,
    operation: impl FnOnce() -> Result<T, MaintainerError>,
) -> Result<T, MaintainerError> {
    let parent = hook.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let filename = hook
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(HOOK_FILENAME);
    let lock_path = parent.join(format!(".{filename}.lock"));
    let lock = open_regular_file(&lock_path, true)?;
    lock.lock_exclusive()?;
    let result = operation();
    let _ = FileExt::unlock(&lock);
    result
}

fn write_hook_atomic(
    hook: &Path,
    frontmatter: &HookFrontmatter,
    body: &str,
) -> Result<(), MaintainerError> {
    let parent = hook.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    regular_file_exists(hook)?;
    let content = render_hook(frontmatter, body)?;
    require_valid(&parse_hook_str(hook, &content)?)?;
    let mut temporary = NamedTempFile::new_in(parent)?;
    temporary.write_all(content.as_bytes())?;
    temporary.flush()?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(hook)
        .map_err(|error| MaintainerError::Persist {
            path: hook.to_path_buf(),
            message: error.error.to_string(),
        })?;

    #[cfg(unix)]
    {
        if let Ok(directory) = File::open(parent) {
            let _ = directory.sync_all();
        }
    }
    Ok(())
}

fn normalize_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extend_unique<T: PartialEq>(target: &mut Vec<T>, values: impl Iterator<Item = T>) {
    for value in values {
        if !target.contains(&value) {
            target.push(value);
        }
    }
}

// Existence and validation belong in the same critical section as creation.
fn initialize_hook(
    hook: &Path,
    frontmatter: &HookFrontmatter,
    body: &str,
) -> Result<(), MaintainerError> {
    with_hook_lock(hook, || {
        if regular_file_exists(hook)? {
            require_valid(&parse_hook(hook)?)?;
            return Ok(());
        }
        write_hook_atomic(hook, frontmatter, body)
    })
}

// Missing leaf paths let explicit delete/rename events reconcile existing local cues.
fn safe_event_file(candidate: &str, cwd: &Path, root: &Path) -> Option<PathBuf> {
    if let Some(path) = safe_existing_file(candidate, cwd, root) {
        return Some(path);
    }
    let path = cwd.join(candidate);
    if fs::symlink_metadata(&path).is_ok() {
        return None;
    }
    let name = path.file_name()?;
    if name == HOOK_FILENAME {
        return None;
    }
    let canonical = path.parent()?.canonicalize().ok()?.join(name);
    let relative = canonical.strip_prefix(root).ok()?.to_path_buf();
    if relative
        .components()
        .any(|part| SKIP_PARTS.contains(&part.as_os_str().to_string_lossy().as_ref()))
    {
        return None;
    }
    Some(relative)
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct PruneReport {
    pub hooks_changed: usize,
    pub resources_removed: usize,
    pub queries_removed: usize,
    pub dry_run: bool,
}

/// Remove one locally authored query by normalized text, never backend memories.
/// Dry runs do not create locks or change files.
pub fn remove_note(cwd: &Path, query: &str, dry_run: bool) -> Result<bool, MaintainerError> {
    let context = TargetContext::new(cwd)?;
    let root = enabled_root(&context.directory)?;
    let directory = nearest_hook_directory(&context.directory, &root)?.unwrap_or(root);
    let hook = directory.join(HOOK_FILENAME);
    let operation = || {
        let (mut fm, body) = load_or_default(&hook)?;
        let count = fm.recall_queries.len();
        fm.recall_queries
            .retain(|item| normalize_text(item.text()) != normalize_text(query));
        let changed = fm.recall_queries.len() != count;
        if changed && !dry_run {
            write_hook_atomic(&hook, &fm, &body)?;
        }
        Ok(changed)
    };
    if dry_run {
        operation()
    } else {
        with_hook_lock(&hook, operation)
    }
}

/// Reconcile generated file cues after deletion/rename. Semantic notes are untouched.
pub fn prune(cwd: &Path, all: bool, dry_run: bool) -> Result<PruneReport, MaintainerError> {
    let context = TargetContext::new(cwd)?;
    let root = enabled_root(&context.directory)?;
    let mut report = PruneReport {
        dry_run,
        ..PruneReport::default()
    };
    for hook in discover_hooks(&context.target, all)? {
        let operation = || {
            let (mut fm, body) = load_or_default(&hook)?;
            let (resources, queries) = prune_auto_queries(&mut fm.recall_queries, &root)?;
            if resources + queries > 0 && !dry_run {
                write_hook_atomic(&hook, &fm, &body)?;
            }
            Ok((resources, queries))
        };
        let (resources, queries) = if dry_run {
            operation()?
        } else {
            with_hook_lock(&hook, operation)?
        };
        report.hooks_changed += usize::from(resources + queries > 0);
        report.resources_removed += resources;
        report.queries_removed += queries;
    }
    Ok(report)
}

fn prune_auto_queries(
    queries: &mut Vec<RecallQuery>,
    root: &Path,
) -> Result<(usize, usize), MaintainerError> {
    let mut removed = 0;
    for query in queries.iter_mut() {
        let RecallQuery::Structured(query) = query else {
            continue;
        };
        if !query.tags.iter().any(|tag| tag == AUTO_TAG) {
            continue;
        }
        let mut retained = Vec::new();
        for resource in std::mem::take(&mut query.resources) {
            let is_file = matches!(&resource, Resource::Structured(value) if value.kind.as_deref() == Some("file"));
            let keep = if is_file {
                match root.join(resource.name()).canonicalize() {
                    Ok(path) => path.starts_with(root) && path.is_file(),
                    Err(error) if error.kind() == io::ErrorKind::NotFound => false,
                    Err(error) => return Err(error.into()),
                }
            } else {
                true
            };
            if keep {
                retained.push(resource);
            } else {
                removed += 1;
            }
        }
        query.resources = retained;
    }
    let before = queries.len();
    queries.retain(|query| {
        !(query.tags().iter().any(|tag| tag == AUTO_TAG) && query.resources().is_empty())
    });
    Ok((removed, before - queries.len()))
}

#[cfg(test)]
mod initialization_tests {
    use super::*;
    use std::sync::{Arc, Barrier};

    #[test]
    fn stale_initializer_preserves_an_intervening_note() {
        let repo = tempfile::tempdir().unwrap();
        fs::create_dir(repo.path().join(".git")).unwrap();
        let hook = repo.path().join(HOOK_FILENAME);
        let prepared = Arc::new(Barrier::new(2));
        let resume = Arc::new(Barrier::new(2));
        std::thread::scope(|scope| {
            let prepared_worker = prepared.clone();
            let resume_worker = resume.clone();
            let hook = &hook;
            scope.spawn(move || {
                assert!(!hook.exists()); // the stale observation made by the old implementation
                let fm = HookFrontmatter {
                    schema: Some("memhooks/v2".into()),
                    ..Default::default()
                };
                prepared_worker.wait();
                resume_worker.wait();
                initialize_hook(hook, &fm, "must not replace the other initializer").unwrap();
            });
            prepared.wait();
            init(repo.path()).unwrap();
            add_note(
                repo.path(),
                NoteInput {
                    query: "Keep this note".into(),
                    ..Default::default()
                },
            )
            .unwrap();
            resume.wait();
        });
        let text = fs::read_to_string(hook).unwrap();
        assert!(text.contains("Keep this note"));
        assert!(!text.contains("must not replace"));
    }
}
