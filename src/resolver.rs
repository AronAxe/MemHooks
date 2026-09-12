use crate::model::{BackendMap, Entity, HookFrontmatter, RecallQuery, Resource};
use crate::parser::{parse_hook, require_v2_schema, ParseError, ParsedHook};
use serde::{Deserialize, Serialize};
use serde_yaml_ng::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const HOOK_FILENAME: &str = "MEMHOOKS.md";

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

/// Resolve the one canonical MemHooks root for a target.
///
/// `MEMHOOKS_ROOT` wins when it contains the target. Otherwise the nearest Git
/// root is the hard boundary. Only outside Git do we fall back to the highest
/// ancestor containing a hook.
pub fn find_root(target: &Path) -> PathBuf {
    let start = if target.is_file() {
        target.parent().unwrap_or(target)
    } else {
        target
    };

    if let Some(configured) = std::env::var_os("MEMHOOKS_ROOT") {
        let configured = PathBuf::from(configured);
        let configured = configured.canonicalize().unwrap_or(configured);
        let absolute_start = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
        if absolute_start.starts_with(&configured) {
            return configured;
        }
    }

    for directory in start.ancestors() {
        if directory.join(".git").exists() {
            return directory.to_path_buf();
        }
    }

    let hooked = start
        .ancestors()
        .filter(|directory| directory.join(HOOK_FILENAME).is_file())
        .last();
    hooked.unwrap_or(start).to_path_buf()
}

pub fn inheritance_chain(target: &Path) -> Vec<PathBuf> {
    let target_dir = if target.is_file() {
        target.parent().unwrap_or(target)
    } else {
        target
    };
    let root = find_root(target_dir);
    directories_root_to_target(&root, target_dir)
        .into_iter()
        .map(|directory| directory.join(HOOK_FILENAME))
        .filter(|path| path.is_file())
        .collect()
}

pub fn parse_chain(target: &Path) -> Result<Vec<ParsedHook>, ParseError> {
    let mut parsed = Vec::new();
    for path in inheritance_chain(target) {
        let hook = parse_hook(&path)?;
        require_v2_schema(&hook)?;
        if !hook.frontmatter.inherits() {
            parsed.clear();
        }
        parsed.push(hook);
    }
    Ok(parsed)
}

pub fn resolve(target: &Path) -> Result<ResolvedHook, ParseError> {
    if !target.exists() {
        return Err(ParseError {
            code: "MH024",
            path: target.to_path_buf(),
            message: format!("target path does not exist: {}", target.display()),
            line: None,
            column: None,
        });
    }

    let target = target
        .canonicalize()
        .unwrap_or_else(|_| target.to_path_buf());
    let root = find_root(&target);
    let hooks = parse_chain(&target)?;
    Ok(resolve_parsed(root, target, &hooks))
}

pub fn resolve_parsed(root: PathBuf, target: PathBuf, hooks: &[ParsedHook]) -> ResolvedHook {
    let mut resolved = ResolvedHook {
        root,
        target,
        ..ResolvedHook::default()
    };

    for hook in hooks {
        merge_hook(&mut resolved, hook);
    }
    resolved
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
