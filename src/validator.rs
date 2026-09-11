use crate::model::{BackendMap, Entity, RecallQuery, Resource};
use crate::parser::{parse_hook, ParseError, ParsedHook};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const PROVIDER_SPECIFIC_V1_FIELDS: &[&str] = &[
    "bank",
    "memory_types",
    "connection_types",
    "mental_models",
    "knowledge_pages",
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn error(
        code: &str,
        parsed: &ParsedHook,
        message: impl Into<String>,
        key: Option<&str>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: Severity::Error,
            message: message.into(),
            path: parsed.path.clone(),
            line: key.and_then(|key| parsed.line_for_key(key)),
            column: None,
            help: None,
        }
    }

    pub fn warning(
        code: &str,
        parsed: &ParsedHook,
        message: impl Into<String>,
        key: Option<&str>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: Severity::Warning,
            message: message.into(),
            path: parsed.path.clone(),
            line: key.and_then(|key| parsed.line_for_key(key)),
            column: None,
            help: None,
        }
    }

    pub fn from_parse(error: ParseError) -> Self {
        Self {
            code: "MH000".into(),
            severity: Severity::Error,
            message: error.message,
            path: error.path,
            line: error.line,
            column: error.column,
            help: None,
        }
    }
}

pub fn validate_file(path: &Path) -> Vec<Diagnostic> {
    match parse_hook(path) {
        Ok(parsed) => validate_parsed(&parsed),
        Err(error) => vec![Diagnostic::from_parse(error)],
    }
}

pub fn validate_parsed(parsed: &ParsedHook) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let fm = &parsed.frontmatter;

    match fm.schema.as_deref() {
        Some("memhooks/v2") => {}
        Some(other) => diagnostics.push(Diagnostic::error(
            "MH001",
            parsed,
            format!("unsupported schema `{other}`; expected `memhooks/v2`"),
            Some("schema"),
        )),
        None => diagnostics.push(Diagnostic::error(
            "MH001",
            parsed,
            "missing required `schema: memhooks/v2`",
            Some("schema"),
        )),
    }

    for key in fm.extra.keys() {
        if PROVIDER_SPECIFIC_V1_FIELDS.contains(&key.as_str()) {
            diagnostics.push(Diagnostic::error(
                "MH016",
                parsed,
                format!(
                    "provider-specific field `{key}` is not part of the memhooks/v2 core; place provider-native controls under `backends.<provider>`"
                ),
                Some(key),
            ));
        } else {
            diagnostics.push(Diagnostic::warning(
                "MH002",
                parsed,
                format!("unknown top-level field `{key}`"),
                Some(key),
            ));
        }
    }

    validate_backends(parsed, "backends", &fm.backends, &mut diagnostics);

    let mut query_keys = HashSet::new();
    for query in &fm.recall_queries {
        validate_query(parsed, query, &mut diagnostics);
        let key = query.text().trim().to_string();
        if !key.is_empty() && !query_keys.insert(key.clone()) {
            diagnostics.push(Diagnostic::warning(
                "MH010",
                parsed,
                format!("duplicate recall query `{key}`"),
                Some("recall_queries"),
            ));
        }
    }

    let mut entity_keys = HashSet::new();
    for entity in &fm.entities {
        validate_entity(parsed, entity, &mut diagnostics);
        let key = entity.name().trim().to_string();
        if !key.is_empty() && !entity_keys.insert(key.clone()) {
            diagnostics.push(Diagnostic::warning(
                "MH011",
                parsed,
                format!("duplicate top-level entity `{key}`"),
                Some("entities"),
            ));
        }
    }

    let mut resource_keys = HashSet::new();
    for resource in &fm.resources {
        validate_resource(parsed, resource, &mut diagnostics);
        let key = resource.name().trim().to_string();
        if !key.is_empty() && !resource_keys.insert(key.clone()) {
            diagnostics.push(Diagnostic::warning(
                "MH015",
                parsed,
                format!("duplicate top-level resource `{key}`"),
                Some("resources"),
            ));
        }
    }

    let excludes = fm
        .exclude
        .iter()
        .map(|value| value.trim())
        .collect::<HashSet<_>>();
    for query in &fm.recall_queries {
        if excludes.contains(query.text().trim()) {
            diagnostics.push(Diagnostic::warning(
                "MH012",
                parsed,
                format!("recall query is also excluded: `{}`", query.text().trim()),
                Some("exclude"),
            ));
        }
    }

    diagnostics
}

pub fn discover_hooks(path: &Path, all: bool) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path.to_path_buf()];
    }

    let scan_root = if all {
        crate::resolver::find_root(path)
    } else {
        path.to_path_buf()
    };
    let mut hooks = WalkBuilder::new(&scan_root)
        .hidden(false)
        .git_ignore(true)
        .git_exclude(true)
        .git_global(true)
        .build()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_some_and(|kind| kind.is_file()))
        .filter(|entry| entry.file_name() == crate::resolver::HOOK_FILENAME)
        .map(|entry| entry.into_path())
        .collect::<Vec<_>>();
    hooks.sort();
    hooks
}

pub fn validate_path(path: &Path, all: bool) -> Vec<Diagnostic> {
    let hooks = discover_hooks(path, all);
    if hooks.is_empty() {
        return vec![Diagnostic {
            code: "MH013".into(),
            severity: Severity::Warning,
            message: "no MEMHOOKS.md files found".into(),
            path: path.to_path_buf(),
            line: None,
            column: None,
            help: Some(
                "run `memhooks validate --all` from a repository containing MemHooks files".into(),
            ),
        }];
    }
    hooks.iter().flat_map(|hook| validate_file(hook)).collect()
}

fn validate_query(parsed: &ParsedHook, query: &RecallQuery, diagnostics: &mut Vec<Diagnostic>) {
    if query.text().trim().is_empty() {
        diagnostics.push(Diagnostic::error(
            "MH003",
            parsed,
            "recall query must not be empty",
            Some("recall_queries"),
        ));
    }

    if let Some(priority) = query.priority() {
        if !(0.0..=1.0).contains(&priority) || !priority.is_finite() {
            diagnostics.push(Diagnostic::error(
                "MH004",
                parsed,
                format!("query priority must be between 0.0 and 1.0; got {priority}"),
                Some("priority"),
            ));
        }
    }

    if let RecallQuery::Structured(value) = query {
        for key in value.extra.keys() {
            if PROVIDER_SPECIFIC_V1_FIELDS.contains(&key.as_str()) {
                diagnostics.push(Diagnostic::error(
                    "MH017",
                    parsed,
                    format!(
                        "provider-specific query field `{key}` is not part of the memhooks/v2 core; place it under `backends.<provider>`"
                    ),
                    Some(key),
                ));
            } else {
                diagnostics.push(Diagnostic::warning(
                    "MH005",
                    parsed,
                    format!("unknown recall-query field `{key}`"),
                    Some(key),
                ));
            }
        }
        if let Some(when) = &value.when {
            for role in &when.roles {
                if role.trim().is_empty() {
                    diagnostics.push(Diagnostic::error(
                        "MH006",
                        parsed,
                        "role names must not be empty",
                        Some("roles"),
                    ));
                }
            }
            for key in when.extra.keys() {
                diagnostics.push(Diagnostic::warning(
                    "MH007",
                    parsed,
                    format!("unknown routing condition `{key}`"),
                    Some(key),
                ));
            }
        }
        for entity in &value.entities {
            validate_entity(parsed, entity, diagnostics);
        }
        for resource in &value.resources {
            validate_resource(parsed, resource, diagnostics);
        }
        validate_backends(parsed, "backends", &value.backends, diagnostics);
    }
}

fn validate_entity(parsed: &ParsedHook, entity: &Entity, diagnostics: &mut Vec<Diagnostic>) {
    if entity.name().trim().is_empty() {
        diagnostics.push(Diagnostic::error(
            "MH008",
            parsed,
            "entity name must not be empty",
            Some("entities"),
        ));
    }
    if let Some(salience) = entity.salience() {
        if !(0.0..=1.0).contains(&salience) || !salience.is_finite() {
            diagnostics.push(Diagnostic::error(
                "MH009",
                parsed,
                format!("entity salience must be between 0.0 and 1.0; got {salience}"),
                Some("salience"),
            ));
        }
    }
    if let Entity::Structured(value) = entity {
        for key in value.extra.keys() {
            diagnostics.push(Diagnostic::warning(
                "MH014",
                parsed,
                format!("unknown entity field `{key}`"),
                Some(key),
            ));
        }
    }
}

fn validate_resource(parsed: &ParsedHook, resource: &Resource, diagnostics: &mut Vec<Diagnostic>) {
    if resource.name().trim().is_empty() {
        diagnostics.push(Diagnostic::error(
            "MH020",
            parsed,
            "resource name must not be empty",
            Some("resources"),
        ));
    }
    if let Some(salience) = resource.salience() {
        if !(0.0..=1.0).contains(&salience) || !salience.is_finite() {
            diagnostics.push(Diagnostic::error(
                "MH021",
                parsed,
                format!("resource salience must be between 0.0 and 1.0; got {salience}"),
                Some("salience"),
            ));
        }
    }
    if let Resource::Structured(value) = resource {
        for key in value.extra.keys() {
            diagnostics.push(Diagnostic::warning(
                "MH022",
                parsed,
                format!("unknown resource field `{key}`"),
                Some(key),
            ));
        }
    }
}

fn validate_backends(
    parsed: &ParsedHook,
    field: &str,
    backends: &BackendMap,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (provider, config) in backends {
        if provider.trim().is_empty() {
            diagnostics.push(Diagnostic::error(
                "MH018",
                parsed,
                "backend namespace names must not be empty",
                Some(field),
            ));
        }
        if !matches!(config, Value::Mapping(_)) {
            diagnostics.push(Diagnostic::error(
                "MH019",
                parsed,
                format!("backend namespace `{provider}` must contain a mapping/object"),
                Some(field),
            ));
        }
    }
}
