use clap::{Parser, Subcommand, ValueEnum};
use memhooks::{resolve, validate_path, Diagnostic, Severity};
use serde_json::json;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "memhooks",
    version,
    about = "Reference resolver and linter for MEMHOOKS.md"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate one file, a subtree, or an entire repository.
    Validate {
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Resolve the repository root and validate every MEMHOOKS.md beneath it.
        #[arg(long)]
        all: bool,
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
    /// Show the effective inherited MemHooks context for a path.
    Explain {
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Filter role-targeted queries using one or more active roles.
        #[arg(long = "role")]
        roles: Vec<String>,
        #[arg(long, value_enum, default_value_t = ExplainFormat::Human)]
        format: ExplainFormat,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
    Sarif,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ExplainFormat {
    Human,
    Json,
}

fn main() {
    let cli = Cli::parse();
    let exit_code = match cli.command {
        Command::Validate { path, all, format } => run_validate(path, all, format),
        Command::Explain {
            path,
            roles,
            format,
        } => run_explain(path, roles, format),
    };
    std::process::exit(exit_code);
}

fn run_validate(path: PathBuf, all: bool, format: OutputFormat) -> i32 {
    let diagnostics = validate_path(&path, all);
    match format {
        OutputFormat::Human => print_human_diagnostics(&diagnostics),
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&diagnostics).expect("serialize diagnostics")
        ),
        OutputFormat::Sarif => println!(
            "{}",
            serde_json::to_string_pretty(&to_sarif(&diagnostics)).expect("serialize SARIF")
        ),
    }

    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
    {
        1
    } else {
        0
    }
}

fn run_explain(path: PathBuf, roles: Vec<String>, format: ExplainFormat) -> i32 {
    let resolved = match resolve(&path) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("error[MH000]: {}", error.message);
            eprintln!(
                "  --> {}:{}:{}",
                error.path.display(),
                error.line.unwrap_or(1),
                error.column.unwrap_or(1)
            );
            return 1;
        }
    };
    let queries = resolved.effective_queries(&roles);

    match format {
        ExplainFormat::Json => {
            let output = json!({
                "root": resolved.root,
                "target": resolved.target,
                "sources": resolved.sources,
                "scope": resolved.scope,
                "sensitivity": resolved.sensitivity,
                "entities": resolved.entities,
                "resources": resolved.resources,
                "tags": resolved.tags,
                "exclude": resolved.exclude,
                "backends": resolved.backends,
                "active_roles": roles,
                "recall_queries": queries,
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&output).expect("serialize explain output")
            );
        }
        ExplainFormat::Human => {
            println!("MemHooks effective context");
            println!("root:   {}", resolved.root.display());
            println!("target: {}", resolved.target.display());
            println!("sources:");
            for source in &resolved.sources {
                println!("  - {}", source.display());
            }
            if !roles.is_empty() {
                println!("active roles: {}", roles.join(", "));
            }
            if !resolved.backends.is_empty() {
                println!(
                    "backend namespaces: {}",
                    resolved
                        .backends
                        .keys()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            println!("recall queries ({}):", queries.len());
            for query in queries {
                let priority = query
                    .priority
                    .map(|value| format!("{value:.2}"))
                    .unwrap_or_else(|| "default".into());
                let role_text = if query.roles.is_empty() {
                    "all".to_string()
                } else {
                    query.roles.join(",")
                };
                println!("  - [{} | roles:{}] {}", priority, role_text, query.query);
                println!("      source: {}", query.source.display());
                if !query.backends.is_empty() {
                    println!(
                        "      backends: {}",
                        query
                            .backends
                            .keys()
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
            }
            println!("entities: {}", resolved.entities.len());
            println!("resources: {}", resolved.resources.len());
            println!("exclusions: {}", resolved.exclude.len());
        }
    }

    0
}

fn print_human_diagnostics(diagnostics: &[Diagnostic]) {
    if diagnostics.is_empty() {
        println!("MemHooks validation passed with no diagnostics.");
        return;
    }

    for diagnostic in diagnostics {
        let level = match diagnostic.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        println!("{level}[{}]: {}", diagnostic.code, diagnostic.message);
        let line = diagnostic.line.unwrap_or(1);
        let column = diagnostic.column.unwrap_or(1);
        println!("  --> {}:{line}:{column}", diagnostic.path.display());
        if let Some(help) = &diagnostic.help {
            println!("  help: {help}");
        }
        println!();
    }

    let errors = diagnostics
        .iter()
        .filter(|item| item.severity == Severity::Error)
        .count();
    let warnings = diagnostics.len() - errors;
    println!("{errors} error(s), {warnings} warning(s)");
}

fn to_sarif(diagnostics: &[Diagnostic]) -> serde_json::Value {
    let results = diagnostics
        .iter()
        .map(|diagnostic| {
            json!({
                "ruleId": diagnostic.code,
                "level": match diagnostic.severity { Severity::Error => "error", Severity::Warning => "warning" },
                "message": { "text": diagnostic.message },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": diagnostic.path.to_string_lossy() },
                        "region": {
                            "startLine": diagnostic.line.unwrap_or(1),
                            "startColumn": diagnostic.column.unwrap_or(1)
                        }
                    }
                }]
            })
        })
        .collect::<Vec<_>>();

    json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "memhooks",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/AronAxe/MemHooks"
                }
            },
            "results": results
        }]
    })
}
