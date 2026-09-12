use crate::model::{
    BackendMap, Entity, HookFrontmatter, RecallQuery, Resource, StructuredRecallQuery,
    StructuredResource,
};
use crate::parser::{parse_hook, render_hook, require_v2_schema, ParseError};
use crate::resolver::{find_root, merge_backend_maps, HOOK_FILENAME};
use fs2::FileExt;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
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
    let root = find_root(cwd);
    let hook = root.join(HOOK_FILENAME);
    if hook.exists() {
        return Ok(hook);
    }

    let mut frontmatter = HookFrontmatter::default();
    frontmatter.schema = Some("memhooks/v2".into());
    frontmatter.inherits = Some(true);
    let project_name = root
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("project");
    let body = format!(
        "# MemHooks\n\nRecall prior decisions, constraints, failures, fixes, rejected approaches, and unresolved issues concerning the `{project_name}` project before substantive changes.\n\nThe agent/runtime owns maintenance of concise retrieval cues. Provider-native routing belongs under `backends.<provider>` only when the active backend and control are actually known."
    );
    write_hook_atomic(&hook, &frontmatter, &body)?;
    Ok(hook)
}

pub fn add_note(cwd: &Path, input: NoteInput) -> Result<PathBuf, MaintainerError> {
    let root = enabled_root(cwd)?;
    let target_directory = nearest_hook_directory(cwd, &root).unwrap_or(root.clone());
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
    let cwd = cwd.canonicalize().unwrap_or(cwd);
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
        .filter_map(|candidate| safe_existing_file(&candidate, &cwd, &root))
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

    let max_auto_paths = std::env::var("MEMHOOKS_AUTO_PATHS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(12);
    let mut writes = 0;

    for (relative_directory, additions) in grouped {
        let directory = root.join(relative_directory);
        if !directory.is_dir() {
            continue;
        }
        let hook = directory.join(HOOK_FILENAME);
        with_hook_lock(&hook, || {
            let (mut frontmatter, body) = load_or_default(&hook)?;
            upsert_auto_query(&mut frontmatter.recall_queries, additions, max_auto_paths);
            write_hook_atomic(&hook, &frontmatter, &body)
        })?;
        writes += 1;
    }

    Ok(writes)
}

fn enabled_root(cwd: &Path) -> Result<PathBuf, MaintainerError> {
    let root = find_root(cwd);
    if root.join(HOOK_FILENAME).is_file() {
        Ok(root)
    } else {
        Err(MaintainerError::NotEnabled(root))
    }
}

fn nearest_hook_directory(cwd: &Path, root: &Path) -> Option<PathBuf> {
    let start = if cwd.is_file() { cwd.parent()? } else { cwd };
    for directory in start.ancestors() {
        if !directory.starts_with(root) {
            break;
        }
        if directory.join(HOOK_FILENAME).is_file() {
            return Some(directory.to_path_buf());
        }
        if directory == root {
            break;
        }
    }
    None
}

fn load_or_default(path: &Path) -> Result<(HookFrontmatter, String), MaintainerError> {
    if path.exists() {
        let parsed = parse_hook(path)?;
        require_v2_schema(&parsed)?;
        Ok((parsed.frontmatter, parsed.body))
    } else {
        let mut frontmatter = HookFrontmatter::default();
        frontmatter.schema = Some("memhooks/v2".into());
        frontmatter.inherits = Some(true);
        Ok((frontmatter, String::new()))
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

fn upsert_auto_query(queries: &mut Vec<RecallQuery>, additions: Vec<PathBuf>, max_paths: usize) {
    let mut structured = queries
        .iter()
        .find(|query| query.text().trim() == AUTO_QUERY)
        .cloned()
        .map(|query| match query {
            RecallQuery::Simple(query) => StructuredRecallQuery {
                query,
                ..StructuredRecallQuery::default()
            },
            RecallQuery::Structured(value) => value,
        })
        .unwrap_or_else(|| StructuredRecallQuery {
            query: AUTO_QUERY.into(),
            ..StructuredRecallQuery::default()
        });

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

    if let Some(existing) = queries
        .iter_mut()
        .find(|query| query.text().trim() == AUTO_QUERY)
    {
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
        || candidate.contains(['(', ')'])
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
    let lock = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(lock_path)?;
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
    let content = render_hook(frontmatter, body)?;
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
