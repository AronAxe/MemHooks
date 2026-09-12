use clap::{Parser, Subcommand, ValueEnum};
use memhooks::{
    find_root, handle_event, init as maintainer_init, resolve, validate_path, BackendMap, Diagnostic,
    EffectiveQuery, Entity, NoteInput, ResolvedHook, Resource, Severity,
};
use serde::Serialize;
use serde_json::json;
use std::collections::BTreeSet;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(
    name = "memhooks",
    version,
    about = "Reference resolver, validator, and maintainer for MEMHOOKS.md"
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
    /// Enable MemHooks at the canonical project root.
    Init {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Add or enrich one semantic retrieval cue in YAML frontmatter.
    Note {
        #[arg(long, default_value = ".")]
        cwd: PathBuf,
        #[arg(long)]
        query: String,
        #[arg(long)]
        priority: Option<f64>,
        #[arg(long = "role")]
        roles: Vec<String>,
        #[arg(long = "entity")]
        entities: Vec<String>,
        #[arg(long = "resource")]
        resources: Vec<String>,
        #[arg(long = "tag")]
        tags: Vec<String>,
        /// JSON object containing opaque provider-native settings.
        #[arg(long, default_value = "{}")]
        backends: String,
    },
    /// Consume one post_tool_call event from stdin and maintain file anchors.
    Event,
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

#[derive(Serialize)]
struct ExplainOutput<'a> {
    #[serde(flatten)]
    resolved: &'a ResolvedHook,
    active_roles: &'a [String],
    effective_queries: &'a [EffectiveQuery],
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
        Command::Init { path } => run_init(path),
        Command::Note {
            cwd,
            query,
            priority,
            roles,
            entities,
            resources,
            tags,
            backends,
        } => run_note(
            cwd, query, priority, roles, entities, resources, tags, backends,
        ),
        Command::Event => run_event(),
    };
    std::process::exit(exit_code);
}

fn run_validate(path: PathBuf, all: bool, format: OutputFormat) -> i32 {
    let sarif_root = validation_root(&path, all);
    let diagnostics = validate_path(&path, all);
    match format {
        OutputFormat::Human => print_human_diagnostics(&diagnostics),
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&diagnostics).expect("serialize diagnostics")
        ),
        OutputFormat::Sarif => println!(
            "{}",
            serde_json::to_string_pretty(&to_sarif(&diagnostics, &sarif_root))
                .expect("serialize SARIF")
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
            eprintln!("error[{}]: {}", error.code, error.message);
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
            let output = ExplainOutput {
                resolved: &resolved,
                active_roles: &roles,
                effective_queries: &queries,
            };
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
            for query in &queries {
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
            println!("guidance blocks ({}):", resolved.guidance.len());
            for guidance in &resolved.guidance {
                println!("  - source: {}", guidance.source.display());
                for line in guidance.value.lines() {
                    println!("      {line}");
                }
            }
        }
    }

    0
}

fn run_init(path: PathBuf) -> i32 {
    match maintainer_init(&path) {
        Ok(hook) => {
            println!("MemHooks enabled: {}", hook.display());
            0
        }
        Err(error) => {
            eprintln!("error: {error}");
            1
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_note(
    cwd: PathBuf,
    query: String,
    priority: Option<f64>,
    roles: Vec<String>,
    entity_args: Vec<String>,
    resource_args: Vec<String>,
    tags: Vec<String>,
    backends_json: String,
) -> i32 {
    if let Some(priority) = priority {
        if !priority.is_finite() || !(0.0..=1.0).contains(&priority) {
            eprintln!("error: --priority must be between 0.0 and 1.0");
            return 2;
        }
    }

    let entities = match entity_args
        .iter()
        .map(|value| parse_entity_arg(value))
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(values) => values,
        Err(error) => {
            eprintln!("error: invalid --entity: {error}");
            return 2;
        }
    };
    let resources = match resource_args
        .iter()
        .map(|value| parse_resource_arg(value))
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(values) => values,
        Err(error) => {
            eprintln!("error: invalid --resource: {error}");
            return 2;
        }
    };
    let backends = match serde_json::from_str::<BackendMap>(&backends_json) {
        Ok(backends) => backends,
        Err(error) => {
            eprintln!("error: --backends must be a JSON object of provider mappings: {error}");
            return 2;
        }
    };

    let input = NoteInput {
        query,
        priority,
        roles,
        entities,
        resources,
        tags,
        backends,
    };
    match memhooks::add_note(&cwd, input) {
        Ok(hook) => {
            println!("Updated {}", hook.display());
            0
        }
        Err(error) => {
            eprintln!("error: {error}");
            1
        }
    }
}

fn run_event() -> i32 {
    let mut input = String::new();
    if let Err(error) = io::stdin().read_to_string(&mut input) {
        eprintln!("error: could not read event from stdin: {error}");
        return 1;
    }
    let payload = match serde_json::from_str(&input) {
        Ok(payload) => payload,
        Err(error) => {
            eprintln!("error: invalid event JSON: {error}");
            return 2;
        }
    };
    match handle_event(&payload) {
        Ok(writes) => {
            if writes > 0 {
                eprintln!("MemHooks maintained {writes} hook file(s).");
            }
            0
        }
        Err(error) => {
            eprintln!("error: {error}");
            1
        }
    }
}

fn parse_entity_arg(value: &str) -> Result<Entity, String> {
    if value.trim_start().starts_with('{') {
        serde_json::from_str::<Entity>(value).map_err(|error| error.to_string())
    } else {
        Ok(Entity::Simple(value.trim().to_string()))
    }
}

fn parse_resource_arg(value: &str) -> Result<Resource, String> {
    if value.trim_start().starts_with('{') {
        serde_json::from_str::<Resource>(value).map_err(|error| error.to_string())
    } else {
        Ok(Resource::Simple(value.trim().to_string()))
    }
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

fn validation_root(path: &Path, all: bool) -> PathBuf {
    if !path.exists() {
        return path.parent().unwrap_or(Path::new(".")).to_path_buf();
    }
    if all {
        find_root(path)
    } else if path.is_file() {
        path.parent().unwrap_or(Path::new(".")).to_path_buf()
    } else {
        path.to_path_buf()
    }
}

fn to_sarif(diagnostics: &[Diagnostic], root: &Path) -> serde_json::Value {
    let codes = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect::<BTreeSet<_>>();
    let rules = codes
        .iter()
        .map(|code| {
            let (title, help) = diagnostic_rule(code);
            json!({
                "id": code,
                "name": title,
                "shortDescription": { "text": title },
                "help": { "text": help },
                "helpUri": "https://github.com/AronAxe/MemHooks/blob/main/docs/troubleshooting.md"
            })
        })
        .collect::<Vec<_>>();

    let results = diagnostics
        .iter()
        .map(|diagnostic| {
            json!({
                "ruleId": diagnostic.code,
                "level": match diagnostic.severity { Severity::Error => "error", Severity::Warning => "warning" },
                "message": { "text": diagnostic.message },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": sarif_uri(&diagnostic.path, root) },
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
                    "informationUri": "https://github.com/AronAxe/MemHooks",
                    "rules": rules
                }
            },
            "results": results
        }]
    })
}

fn sarif_uri(path: &Path, root: &Path) -> String {
    let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let relative = canonical_path
        .strip_prefix(&canonical_root)
        .unwrap_or(&canonical_path);
    let value = relative.to_string_lossy().replace('\\', "/");
    value.trim_start_matches("./").to_string()
}

fn diagnostic_rule(code: &str) -> (&'static str, &'static str) {
    match code {
        "MH000" => ("Invalid MemHooks file", "The file or YAML frontmatter could not be parsed."),
        "MH001" => ("Unsupported schema", "Use the supported `schema: memhooks/v2`."),
        "MH002" => ("Unknown top-level field", "Remove the field or move provider-native configuration under `backends.<provider>`."),
        "MH003" => ("Missing recall query", "A structured recall entry needs a non-empty `query` string."),
        "MH004" => ("Invalid query priority", "Priority must be finite and between 0.0 and 1.0."),
        "MH005" => ("Unknown recall-query field", "Check the field name or move provider-native settings under `backends.<provider>`."),
        "MH006" => ("Empty role", "Role names must be non-empty strings."),
        "MH007" => ("Unknown routing condition", "Only supported routing conditions belong under `when`."),
        "MH008" => ("Invalid entity", "An entity needs a non-empty name."),
        "MH009" => ("Invalid entity salience", "Entity salience must be finite and between 0.0 and 1.0."),
        "MH010" => ("Duplicate query", "Duplicate query text appears within one hook."),
        "MH011" => ("Duplicate entity", "Duplicate entity name appears within one hook."),
        "MH012" => ("Recall/exclude conflict", "The same text is both recalled and excluded."),
        "MH013" => ("No hooks found", "No MEMHOOKS.md files were found in the requested scope."),
        "MH014" => ("Unknown entity field", "Check the entity field name."),
        "MH015" => ("Duplicate resource", "Duplicate resource name appears within one hook."),
        "MH016" => ("Provider field in core", "Provider-native fields belong under `backends.<provider>`."),
        "MH017" => ("Provider query field in core", "Provider-native query fields belong under `backends.<provider>`."),
        "MH018" => ("Empty backend namespace", "Backend namespace names must be non-empty."),
        "MH019" => ("Invalid backend namespace", "A backend namespace must contain a mapping/object."),
        "MH020" => ("Invalid resource", "A resource needs a non-empty name."),
        "MH021" => ("Invalid resource salience", "Resource salience must be finite and between 0.0 and 1.0."),
        "MH022" => ("Unknown resource field", "Check the resource field name."),
        "MH024" => ("Path does not exist", "Correct the target path before running MemHooks."),
        "MH025" => ("Malformed recall query", "A recall-query entry or one of its known fields has the wrong YAML type."),
        "MH026" => ("Malformed entity", "An entity entry or one of its known fields has the wrong YAML type."),
        "MH027" => ("Malformed resource", "A resource entry or one of its known fields has the wrong YAML type."),
        _ => ("MemHooks diagnostic", "See the MemHooks troubleshooting documentation."),
    }
}
