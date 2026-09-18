use crate::filesystem::{path_error, regular_file_exists};
use crate::model::{BackendMap, Entity, HookFrontmatter, RecallQuery, Resource};
use crate::parser::{parse_hook, ParseError, ParsedHook};
use crate::validator::require_valid;
use serde::{Deserialize, Serialize};
use serde_yaml_ng::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const HOOK_FILENAME: &str = "MEMHOOKS.md";
pub const PLAN_VERSION: &str = "memhooks/plan-v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueryOmission {
    pub source: PathBuf,
    pub query: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Sourced<T> {
    pub source: PathBuf,
    pub value: T,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ResolvedHook {
    pub root: PathBuf,
    pub target: PathBuf,
    pub sources: Vec<PathBuf>,
    pub scope: Option<String>,
    pub sensitivity: Option<String>,
    pub recall_queries: Vec<Sourced<RecallQuery>>,
    pub entities: Vec<Sourced<Entity>>,
    pub resources: Vec<Sourced<Resource>>,
    pub tags: Vec<String>,
    pub exclude: Vec<String>,
    pub backends: BackendMap,
    pub guidance: Vec<Sourced<String>>,
    #[serde(skip)]
    pub query_history: Vec<QueryOmission>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EffectiveQuery {
    pub source: PathBuf,
    pub query: String,
    pub priority: Option<f64>,
    pub roles: Vec<String>,
    pub entities: Vec<Entity>,
    pub resources: Vec<Resource>,
    pub tags: Vec<String>,
    pub backends: BackendMap,
}

impl ResolvedHook {
    pub fn omitted_queries(&self, active_roles: &[String]) -> Vec<QueryOmission> {
        let mut omissions = self.query_history.clone();
        if !active_roles.is_empty() {
            for item in &self.recall_queries {
                let roles = item.value.roles();
                if !roles.is_empty() && !roles.iter().any(|role| active_roles.contains(role)) {
                    omissions.push(QueryOmission {
                        source: item.source.clone(),
                        query: item.value.text().to_string(),
                        reason: "role_mismatch".into(),
                    });
                }
            }
        }
        omissions
    }

    pub fn effective_queries(&self, active_roles: &[String]) -> Vec<EffectiveQuery> {
        let role_filter_active = !active_roles.is_empty();
        self.recall_queries
            .iter()
            .filter_map(|item| {
                let roles = item.value.roles();
                if role_filter_active
                    && !roles.is_empty()
                    && !roles
                        .iter()
                        .any(|role| active_roles.iter().any(|active| active == role))
                {
                    return None;
                }

                let mut entities = self
                    .entities
                    .iter()
                    .map(|entity| entity.value.clone())
                    .collect::<Vec<_>>();
                for entity in item.value.entities() {
                    if !entities.contains(entity) {
                        entities.push(entity.clone());
                    }
                }

                let mut resources = self
                    .resources
                    .iter()
                    .map(|resource| resource.value.clone())
                    .collect::<Vec<_>>();
                for resource in item.value.resources() {
                    if !resources.contains(resource) {
                        resources.push(resource.clone());
                    }
                }

                let mut tags = self.tags.clone();
                extend_unique_strings(&mut tags, item.value.tags().iter().cloned());

                let mut backends = self.backends.clone();
                merge_backend_maps(&mut backends, item.value.backends());

                Some(EffectiveQuery {
                    source: item.source.clone(),
                    query: item.value.text().to_string(),
                    priority: item.value.priority(),
                    roles: roles.to_vec(),
                    entities,
                    resources,
                    tags,
                    backends,
                })
            })
            .collect()
    }
}

pub fn merge_backend_maps(target: &mut BackendMap, overlay: &BackendMap) {
    for (provider, value) in overlay {
        match target.get_mut(provider) {
            Some(existing) => merge_yaml_value(existing, value),
            None => {
                target.insert(provider.clone(), value.clone());
            }
        }
    }
}

fn merge_yaml_value(target: &mut Value, overlay: &Value) {
    match (target, overlay) {
        (Value::Mapping(target_map), Value::Mapping(overlay_map)) => {
            for (key, value) in overlay_map {
                match target_map.get_mut(key) {
                    Some(existing) => merge_yaml_value(existing, value),
                    None => {
                        target_map.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        (target, overlay) => *target = overlay.clone(),
    }
}

/// Canonical paths shared by all filesystem-facing operations.
#[derive(Debug, Clone)]
pub struct TargetContext {
    pub root: PathBuf,
    pub target: PathBuf,
    pub directory: PathBuf,
}

impl TargetContext {
    pub fn new(target: &Path) -> Result<Self, ParseError> {
        if target.file_name().is_some_and(|name| name == HOOK_FILENAME) {
            regular_file_exists(target)?;
        }
        let target = target.canonicalize().map_err(|error| {
            path_error(
                target,
                "MH024",
                format!("could not canonicalize target: {error}"),
            )
        })?;
        let directory = if target.is_file() {
            target.parent().unwrap_or(&target).to_path_buf()
        } else if target.is_dir() {
            target.clone()
        } else {
            return Err(path_error(
                &target,
                "MH029",
                "target is not a regular file or directory",
            ));
        };
        let mut selected = None;
        if let Some(configured) = std::env::var_os("MEMHOOKS_ROOT") {
            let configured = PathBuf::from(configured);
            let root = configured.canonicalize().map_err(|error| {
                path_error(
                    &configured,
                    "MH024",
                    format!("invalid MEMHOOKS_ROOT: {error}"),
                )
            })?;
            if !root.is_dir() {
                return Err(path_error(
                    &root,
                    "MH024",
                    "MEMHOOKS_ROOT must be a directory",
                ));
            }
            if directory.starts_with(&root) {
                selected = Some(root);
            }
        }
        let root = selected
            .or_else(|| {
                directory
                    .ancestors()
                    .find(|dir| dir.join(".git").exists())
                    .map(Path::to_path_buf)
            })
            .or_else(|| {
                directory
                    .ancestors()
                    .filter(|dir| std::fs::symlink_metadata(dir.join(HOOK_FILENAME)).is_ok())
                    .last()
                    .map(Path::to_path_buf)
            })
            .unwrap_or_else(|| directory.clone());
        Ok(Self {
            root,
            target,
            directory,
        })
    }
}

/// Resolve the canonical project boundary, reporting invalid paths/configuration.
pub fn find_root(target: &Path) -> Result<PathBuf, ParseError> {
    Ok(TargetContext::new(target)?.root)
}

pub fn inheritance_chain(target: &Path) -> Result<Vec<PathBuf>, ParseError> {
    let context = TargetContext::new(target)?;
    let mut chain = Vec::new();
    for directory in directories_root_to_target(&context.root, &context.directory) {
        let hook = directory.join(HOOK_FILENAME);
        if regular_file_exists(&hook)? {
            chain.push(hook);
        }
    }
    Ok(chain)
}

pub fn parse_chain(target: &Path) -> Result<Vec<ParsedHook>, ParseError> {
    let mut parsed = Vec::new();
    for path in inheritance_chain(target)? {
        let hook = parse_hook(&path)?;
        require_valid(&hook)?;
        parsed.push(hook);
    }
    Ok(parsed)
}

pub fn resolve(target: &Path) -> Result<ResolvedHook, ParseError> {
    let context = TargetContext::new(target)?;
    let hooks = parse_chain(&context.target)?;
    resolve_parsed(context.root, context.target, &hooks)
}

/// Validate caller-supplied parsed hooks before creating an executable plan.
pub fn resolve_parsed(
    root: PathBuf,
    target: PathBuf,
    hooks: &[ParsedHook],
) -> Result<ResolvedHook, ParseError> {
    let mut resolved = ResolvedHook {
        root,
        target,
        ..ResolvedHook::default()
    };
    for hook in hooks {
        require_valid(hook)?;
        if !hook.frontmatter.inherits() {
            let root = resolved.root.clone();
            let target = resolved.target.clone();
            let mut history = std::mem::take(&mut resolved.query_history);
            history.extend(resolved.recall_queries.iter().map(|item| QueryOmission {
                source: item.source.clone(),
                query: item.value.text().to_string(),
                reason: "inheritance_cut".into(),
            }));
            resolved = ResolvedHook {
                root,
                target,
                query_history: history,
                ..ResolvedHook::default()
            };
        }
        merge_hook(&mut resolved, hook);
    }
    Ok(resolved)
}

fn directories_root_to_target(root: &Path, target: &Path) -> Vec<PathBuf> {
    let Ok(relative) = target.strip_prefix(root) else {
        return vec![target.to_path_buf()];
    };
    let mut result = vec![root.to_path_buf()];
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component);
        result.push(current.clone());
    }
    result
}

fn merge_hook(resolved: &mut ResolvedHook, hook: &ParsedHook) {
    let HookFrontmatter {
        scope,
        sensitivity,
        recall_queries,
        entities,
        resources,
        tags,
        exclude,
        backends,
        ..
    } = &hook.frontmatter;

    resolved.sources.push(hook.path.clone());
    if scope.is_some() {
        resolved.scope = scope.clone();
    }
    if sensitivity.is_some() {
        resolved.sensitivity = sensitivity.clone();
    }

    extend_unique_strings(&mut resolved.tags, tags.iter().cloned());
    extend_unique_strings(&mut resolved.exclude, exclude.iter().cloned());
    merge_backend_maps(&mut resolved.backends, backends);

    for query in recall_queries {
        let sourced = Sourced {
            source: hook.path.clone(),
            value: query.clone(),
        };
        if let Some(existing) = resolved
            .recall_queries
            .iter_mut()
            .find(|existing| existing.value.text().trim() == query.text().trim())
        {
            // Query identity is its trimmed question text. More-local metadata wins.
            resolved.query_history.push(QueryOmission {
                source: existing.source.clone(),
                query: existing.value.text().to_string(),
                reason: "overridden".into(),
            });
            *existing = sourced;
        } else {
            resolved.recall_queries.push(sourced);
        }
    }

    for entity in entities {
        if !resolved
            .entities
            .iter()
            .any(|existing| existing.value == *entity)
        {
            resolved.entities.push(Sourced {
                source: hook.path.clone(),
                value: entity.clone(),
            });
        }
    }

    for resource in resources {
        if !resolved
            .resources
            .iter()
            .any(|existing| existing.value == *resource)
        {
            resolved.resources.push(Sourced {
                source: hook.path.clone(),
                value: resource.clone(),
            });
        }
    }

    let body = hook.body.trim();
    if !body.is_empty() {
        resolved.guidance.push(Sourced {
            source: hook.path.clone(),
            value: body.to_string(),
        });
    }
}

fn extend_unique_strings(target: &mut Vec<String>, values: impl Iterator<Item = String>) {
    let mut seen: HashSet<String> = target.iter().cloned().collect();
    for value in values {
        if seen.insert(value.clone()) {
            target.push(value);
        }
    }
}
