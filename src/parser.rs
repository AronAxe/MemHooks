use crate::model::HookFrontmatter;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct ParsedHook {
    pub path: PathBuf,
    pub frontmatter: HookFrontmatter,
    pub body: String,
    pub source: String,
    pub frontmatter_start_line: usize,
}

impl ParsedHook {
    pub fn line_for_key(&self, key: &str) -> Option<usize> {
        let needle = format!("{key}:");
        self.source
            .lines()
            .enumerate()
            .find_map(|(index, line)| {
                let trimmed = line.trim_start();
                if trimmed.starts_with(&needle) {
                    Some(index + 1)
                } else {
                    None
                }
            })
    }
}

#[derive(Debug, Error, Clone)]
#[error("{message}")]
pub struct ParseError {
    pub path: PathBuf,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

pub fn parse_hook(path: impl AsRef<Path>) -> Result<ParsedHook, ParseError> {
    let path = path.as_ref();
    let source = fs::read_to_string(path).map_err(|error| ParseError {
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
            path: path.clone(),
            message: "unterminated YAML frontmatter; expected closing `---`".into(),
            line: Some(all_lines.len().max(1)),
            column: Some(1),
        })?;

    let yaml = all_lines[1..end_index].join("\n");
    let frontmatter = serde_yaml::from_str::<HookFrontmatter>(&yaml).map_err(|error| {
        let location = error.location();
        ParseError {
            path: path.clone(),
            message: format!("invalid YAML frontmatter: {error}"),
            line: location.map(|loc| loc.line() + 1),
            column: location.map(|loc| loc.column()),
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
    })
}
