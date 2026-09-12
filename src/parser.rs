use crate::model::HookFrontmatter;
use saphyr::{LoadableYamlNode, MarkedYamlOwned, YamlDataOwned};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct ParsedHook {
    pub path: PathBuf,
    pub frontmatter: HookFrontmatter,
    pub body: String,
    pub source: String,
    pub frontmatter_start_line: usize,
    locations: BTreeMap<String, SourceLocation>,
}

impl ParsedHook {
    pub fn location(&self, path: &str) -> Option<SourceLocation> {
        self.locations.get(path).copied()
    }

    pub fn line_for_key(&self, key: &str) -> Option<usize> {
        self.location(key).map(|location| location.line)
    }
}

#[derive(Debug, Error, Clone)]
#[error("{message}")]
pub struct ParseError {
    pub code: &'static str,
    pub path: PathBuf,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

pub fn parse_hook(path: impl AsRef<Path>) -> Result<ParsedHook, ParseError> {
    let path = path.as_ref();
    let source = fs::read_to_string(path).map_err(|error| ParseError {
        code: "MH000",
        path: path.to_path_buf(),
        message: format!("could not read file: {error}"),
        line: None,
        column: None,
    })?;
    parse_hook_str(path, &source)
}

pub fn parse_hook_str(path: impl AsRef<Path>, source: &str) -> Result<ParsedHook, ParseError> {
    let path = path.as_ref().to_path_buf();
    let mut lines = source.lines();
    let first = lines.next().unwrap_or_default();
    if first.trim() != "---" {
        return Err(ParseError {
            code: "MH000",
            path,
            message: "MEMHOOKS.md must begin with YAML frontmatter (`---`)".into(),
            line: Some(1),
            column: Some(1),
        });
    }

    let all_lines: Vec<&str> = source.lines().collect();
    let end_index = all_lines
        .iter()
        .enumerate()
        .skip(1)
        .find_map(|(index, line)| (line.trim() == "---").then_some(index))
        .ok_or_else(|| ParseError {
            code: "MH000",
            path: path.clone(),
            message: "unterminated YAML frontmatter; expected closing `---`".into(),
            line: Some(all_lines.len().max(1)),
            column: Some(1),
        })?;

    let yaml = all_lines[1..end_index].join("\n");
    let frontmatter = serde_yaml_ng::from_str::<HookFrontmatter>(&yaml).map_err(|error| {
        let location = error.location();
        ParseError {
            code: "MH000",
            path: path.clone(),
            message: format!("invalid YAML frontmatter: {error}"),
            line: location.as_ref().map(|location| location.line() + 1),
            column: location.as_ref().map(|location| location.column()),
        }
    })?;

    let body = if end_index + 1 < all_lines.len() {
        all_lines[end_index + 1..].join("\n")
    } else {
        String::new()
    };

    Ok(ParsedHook {
        path,
        frontmatter,
        body,
        source: source.to_string(),
        frontmatter_start_line: 2,
        locations: build_location_index(&yaml),
    })
}

pub fn render_hook(frontmatter: &HookFrontmatter, body: &str) -> Result<String, ParseError> {
    let mut yaml = serde_yaml_ng::to_string(frontmatter).map_err(|error| ParseError {
        code: "MH000",
        path: PathBuf::from("MEMHOOKS.md"),
        message: format!("could not serialize MemHooks frontmatter: {error}"),
        line: None,
        column: None,
    })?;
    if let Some(stripped) = yaml.strip_prefix("---\n") {
        yaml = stripped.to_string();
    }
    if !yaml.ends_with('\n') {
        yaml.push('\n');
    }

    let body = body.trim_matches('\n');
    if body.is_empty() {
        Ok(format!("---\n{yaml}---\n"))
    } else {
        Ok(format!("---\n{yaml}---\n\n{body}\n"))
    }
}

pub fn require_v2_schema(parsed: &ParsedHook) -> Result<(), ParseError> {
    if parsed.frontmatter.schema.as_deref() == Some("memhooks/v2") {
        return Ok(());
    }
    let location = parsed.location("schema");
    Err(ParseError {
        code: "MH001",
        path: parsed.path.clone(),
        message: format!(
            "unsupported MemHooks schema `{}`; expected `memhooks/v2`",
            parsed.frontmatter.schema.as_deref().unwrap_or("<missing>")
        ),
        line: location.map(|location| location.line),
        column: location.map(|location| location.column),
    })
}

fn build_location_index(yaml: &str) -> BTreeMap<String, SourceLocation> {
    let mut locations = BTreeMap::new();
    let Ok(documents) = MarkedYamlOwned::load_from_str(yaml) else {
        return locations;
    };
    let Some(root) = documents.first() else {
        return locations;
    };
    walk_marked_yaml(root, "", &mut locations);
    locations
}

fn walk_marked_yaml(
    node: &MarkedYamlOwned,
    path: &str,
    locations: &mut BTreeMap<String, SourceLocation>,
) {
    match &node.data {
        YamlDataOwned::Mapping(mapping) => {
            for (key, value) in mapping {
                let Some(key_text) = key.data.as_str() else {
                    continue;
                };
                let child_path = if path.is_empty() {
                    key_text.to_string()
                } else {
                    format!("{path}.{key_text}")
                };
                locations.insert(child_path.clone(), location_from_mark(key));
                walk_marked_yaml(value, &child_path, locations);
            }
        }
        YamlDataOwned::Sequence(sequence) => {
            for (index, value) in sequence.iter().enumerate() {
                let child_path = format!("{path}[{index}]");
                locations.insert(child_path.clone(), location_from_mark(value));
                walk_marked_yaml(value, &child_path, locations);
            }
        }
        YamlDataOwned::Tagged(_, value) => walk_marked_yaml(value, path, locations),
        _ => {}
    }
}

fn location_from_mark(node: &MarkedYamlOwned) -> SourceLocation {
    SourceLocation {
        // Saphyr line markers are 1-indexed, columns are 0-indexed; YAML starts on source line 2.
        line: node.span.start.line() + 1,
        column: node.span.start.col() + 1,
    }
}
