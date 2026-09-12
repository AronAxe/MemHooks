use serde_json::Value;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_memhooks")
}

#[test]
fn explain_json_contains_source_attributed_guidance() {
    let repo = tempdir().unwrap();
    fs::create_dir(repo.path().join(".git")).unwrap();
    fs::write(
        repo.path().join("MEMHOOKS.md"),
        "---\nschema: memhooks/v2\nrecall_queries: [\"What matters?\"]\n---\n\nUse direct recall first.\n",
    )
    .unwrap();

    let output = Command::new(binary())
        .args(["explain", repo.path().to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["guidance"][0]["value"], "Use direct recall first.");
    assert_eq!(value["effective_queries"][0]["query"], "What matters?");
}

#[test]
fn sarif_uses_repository_relative_uri_and_rule_metadata() {
    let repo = tempdir().unwrap();
    fs::create_dir(repo.path().join(".git")).unwrap();
    fs::create_dir_all(repo.path().join("src")).unwrap();
    fs::write(
        repo.path().join("src/MEMHOOKS.md"),
        "---\nschema: memhooks/v2\nrecall_queries:\n  - query: bad\n    priority: 9.9\n---\n",
    )
    .unwrap();

    let output = Command::new(binary())
        .args([
            "validate",
            repo.path().to_str().unwrap(),
            "--all",
            "--format",
            "sarif",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let result = &value["runs"][0]["results"][0];
    assert_eq!(result["ruleId"], "MH004");
    assert_eq!(
        result["locations"][0]["physicalLocation"]["artifactLocation"]["uri"],
        "src/MEMHOOKS.md"
    );
    let rules = value["runs"][0]["tool"]["driver"]["rules"]
        .as_array()
        .unwrap();
    assert!(rules.iter().any(|rule| rule["id"] == "MH004"));
}

#[test]
fn nonexistent_cli_target_is_an_error() {
    let repo = tempdir().unwrap();
    let missing = repo.path().join("missing");

    let validate = Command::new(binary())
        .args(["validate", missing.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!validate.status.success());
    assert!(String::from_utf8_lossy(&validate.stdout).contains("MH024"));

    let explain = Command::new(binary())
        .args(["explain", missing.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!explain.status.success());
    assert!(String::from_utf8_lossy(&explain.stderr).contains("MH024"));
}
