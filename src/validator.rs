use crate::model::{BackendMap, Entity, RecallQuery, Resource};
use crate::parser::{parse_hook, ParseError, ParsedHook};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use serde_yaml_ng::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const PROVIDER_SPECIFIC_V1_FIELDS: &[&str] = &[
    "bank",
    "memory_types",
    "connection_types",
    "mental_models",
    "knowledge_pages",
];
const QUERY_FIELDS: &[&str] = &[
    "query",
    "priority",
    "when",
    "entities",
    "resources",
    "tags",
    "backends",
];
const ENTITY_FIELDS: &[&str] = &["name", "type", "salience"];
const RESOURCE_FIELDS: &[&str] = &["name", "kind", "salience"];

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
        location_path: Option<&str>,
    ) -> Self {
        Self::new(Severity::Error, code, parsed, message, location_path)
    }

    pub fn warning(
        code: &str,
        parsed: &ParsedHook,
        message: impl Into<String>,
        location_path: Option<&str>,
    ) -> Self {
        Self::new(Severity::Warning, code, parsed, message, location_path)
    }

    fn new(
        severity: Severity,
        code: &str,
        parsed: &ParsedHook,
        message: impl Into<String>,
        location_path: Option<&str>,
    ) -> Self {
        let location = location_path.and_then(|path| parsed.location(path));
        Self {
            code: code.into(),
            severity,
            message: message.into(),
            path: parsed.path.clone(),
            line: location.map(|location| location.line),
            column: location.map(|location| location.column),
            help: None,
        }
    }

    pub fn from_parse(error: ParseError) -> Self {
        Self {
            code: error.code.into(),
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
    for (index, query) in fm.recall_queries.iter().enumerate() {
        let path = format!("recall_queries[{index}]");
        validate_query(parsed, query, &path, &mut diagnostics);
        let key = query.text().trim().to_string();
        if !key.is_empty() && !query_keys.insert(key.clone()) {
            diagnostics.push(Diagnostic::warning(
                "MH010",
                parsed,
                format!("duplicate recall query `{key}` in one hook"),
                Some(&path),
            ));
        }
    }

    let mut entity_keys = HashSet::new();
    for (index, entity) in fm.entities.iter().enumerate() {
        let path = format!("entities[{index}]");
        validate_entity(parsed, entity, &path, &mut diagnostics);
        let key = entity.name().trim().to_string();
        if !key.is_empty() && !entity_keys.insert(key.clone()) {
            diagnostics.push(Diagnostic::warning(
                "MH011",
                parsed,
                format!("duplicate top-level entity `{key}`"),
                Some(&path),
            ));
        }
    }

    let mut resource_keys = HashSet::new();
    for (index, resource) in fm.resources.iter().enumerate() {
        let path = format!("resources[{index}]");
        validate_resource(parsed, resource, &path, &mut diagnostics);
        let key = resource.name().trim().to_string();
        if !key.is_empty() && !resource_keys.insert(key.clone()) {
            diagnostics.push(Diagnostic::warning(
                "MH015",
                parsed,
                format!("duplicate top-level resource `{key}`"),
                Some(&path),
            ));
        }
    }

    let excludes = fm
        .exclude
        .iter()
        .map(|value| value.trim())
        .collect::<HashSet<_>>();
    for (index, query) in fm.recall_queries.iter().enumerate() {
        if excludes.contains(query.text().trim()) {
            let path = format!("recall_queries[{index}]");
            diagnostics.push(Diagnostic::warning(
                "MH012",
                parsed,
                format!("recall query is also excluded: `{}`", query.text().trim()),
                Some(&path),
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
    if !path.exists() {
        return vec![Diagnostic {
            code: "MH024".into(),
            severity: Severity::Error,
            message: format!("target path does not exist: {}", path.display()),
            path: path.to_path_buf(),
            line: None,
            column: None,
            help: Some("check the path spelling before validating".into()),
        }];
    }

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
                "run `memhooks init` to enable the project, or validate a MemHooks-enabled path"
                    .into(),
            ),
        }];
    }
    hooks.iter().flat_map(|hook| validate_file(hook)).collect()
}

fn validate_query(
    parsed: &ParsedHook,
    query: &RecallQuery,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if query.text().trim().is_empty() {
        diagnostics.push(Diagnostic::error(
            "MH003",
            parsed,
            "recall query must contain a non-empty `query` string",
            Some(path),
        ));
    }

    if let Some(priority) = query.priority() {
        if !(0.0..=1.0).contains(&priority) || !priority.is_finite() {
            diagnostics.push(Diagnostic::error(
                "MH004",
                parsed,
                format!("query priority must be between 0.0 and 1.0; got {priority:?}"),
                Some(&format!("{path}.priority")),
            ));
        }
    }

    if let RecallQuery::Structured(value) = query {
        for key in value.extra.keys() {
            let field_path = if key == "_invalid_value" {
                path.to_string()
            } else {
                format!("{path}.{key}")
            };
            if key == "_invalid_value" || QUERY_FIELDS.contains(&key.as_str()) {
                diagnostics.push(Diagnostic::error(
                    "MH025",
                    parsed,
                    if key == "_invalid_value" {
                        "recall query entries must be strings or mappings".into()
                    } else {
                        format!("recall-query field `{key}` has an invalid value/type")
                    },
                    Some(&field_path),
                ));
            } else if PROVIDER_SPECIFIC_V1_FIELDS.contains(&key.as_str()) {
                diagnostics.push(Diagnostic::error(
                    "MH017",
                    parsed,
                    format!(
                        "provider-specific query field `{key}` is not part of the memhooks/v2 core; place it under `backends.<provider>`"
                    ),
                    Some(&field_path),
                ));
            } else {
                diagnostics.push(Diagnostic::warning(
                    "MH005",
                    parsed,
                    format!("unknown recall-query field `{key}`"),
                    Some(&field_path),
                ));
            }
        }
        if let Some(when) = &value.when {
            for (index, role) in when.roles.iter().enumerate() {
                if role.trim().is_empty() {
                    diagnostics.push(Diagnostic::error(
                        "MH006",
                        parsed,
                        "role names must not be empty",
                        Some(&format!("{path}.when.roles[{index}]")),
                    ));
                }
            }
            for key in when.extra.keys() {
                diagnostics.push(Diagnostic::warning(
                    "MH007",
                    parsed,
                    format!("unknown routing condition `{key}`"),
                    Some(&format!("{path}.when.{key}")),
                ));
            }
        }
        for (index, entity) in value.entities.iter().enumerate() {
            validate_entity(
                parsed,
                entity,
                &format!("{path}.entities[{index}]"),
                diagnostics,
            );
        }
        for (index, resource) in value.resources.iter().enumerate() {
            validate_resource(
                parsed,
                resource,
                &format!("{path}.resources[{index}]"),
                diagnostics,
            );
        }
        validate_backends(
            parsed,
            &format!("{path}.backends"),
            &value.backends,
            diagnostics,
        );
    }
}

fn validate_entity(
    parsed: &ParsedHook,
    entity: &Entity,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if entity.name().trim().is_empty() {
        diagnostics.push(Diagnostic::error(
            "MH008",
            parsed,
            "entity must contain a non-empty `name` string",
            Some(path),
        ));
    }
    if let Some(salience) = entity.salience() {
        if !(0.0..=1.0).contains(&salience) || !salience.is_finite() {
            diagnostics.push(Diagnostic::error(
                "MH009",
                parsed,
                format!("entity salience must be between 0.0 and 1.0; got {salience:?}"),
                Some(&format!("{path}.salience")),
            ));
        }
    }
    if let Entity::Structured(value) = entity {
        for key in value.extra.keys() {
            let field_path = if key == "_invalid_value" {
                path.to_string()
            } else {
                format!("{path}.{key}")
            };
            if key == "_invalid_value" || ENTITY_FIELDS.contains(&key.as_str()) {
                diagnostics.push(Diagnostic::error(
                    "MH026",
                    parsed,
                    if key == "_invalid_value" {
                        "entity entries must be strings or mappings".into()
                    } else {
                        format!("entity field `{key}` has an invalid value/type")
                    },
                    Some(&field_path),
                ));
            } else {
                diagnostics.push(Diagnostic::warning(
                    "MH014",
                    parsed,
                    format!("unknown entity field `{key}`"),
                    Some(&field_path),
                ));
            }
        }
    }
}

fn validate_resource(
    parsed: &ParsedHook,
    resource: &Resource,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if resource.name().trim().is_empty() {
        diagnostics.push(Diagnostic::error(
            "MH020",
            parsed,
            "resource must contain a non-empty `name` string",
            Some(path),
        ));
    }
    if let Some(salience) = resource.salience() {
        if !(0.0..=1.0).contains(&salience) || !salience.is_finite() {
            diagnostics.push(Diagnostic::error(
                "MH021",
                parsed,
                format!("resource salience must be between 0.0 and 1.0; got {salience:?}"),
                Some(&format!("{path}.salience")),
            ));
        }
    }
    if let Resource::Structured(value) = resource {
        for key in value.extra.keys() {
            let field_path = if key == "_invalid_value" {
                path.to_string()
            } else {
                format!("{path}.{key}")
            };
            if key == "_invalid_value" || RESOURCE_FIELDS.contains(&key.as_str()) {
                diagnostics.push(Diagnostic::error(
                    "MH027",
                    parsed,
                    if key == "_invalid_value" {
                        "resource entries must be strings or mappings".into()
                    } else {
                        format!("resource field `{key}` has an invalid value/type")
                    },
                    Some(&field_path),
                ));
            } else {
                diagnostics.push(Diagnostic::warning(
                    "MH022",
                    parsed,
                    format!("unknown resource field `{key}`"),
                    Some(&field_path),
                ));
            }
        }
    }
}

fn validate_backends(
    parsed: &ParsedHook,
    path: &str,
    backends: &BackendMap,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (provider, config) in backends {
        let provider_path = format!("{path}.{provider}");
        if provider.trim().is_empty() {
            diagnostics.push(Diagnostic::error(
                "MH018",
                parsed,
                "backend namespace names must not be empty",
                Some(path),
            ));
        }
        if !matches!(config, Value::Mapping(_)) {
            diagnostics.push(Diagnostic::error(
                "MH019",
                parsed,
                format!("backend namespace `{provider}` must contain a mapping/object"),
                Some(&provider_path),
            ));
        }
    }
}
