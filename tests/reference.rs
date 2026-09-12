use memhooks::{
    find_root, parse_hook_str, resolve, validate_parsed, Entity, RecallQuery, Resource, Severity,
};
use serde_json::json;
use std::fs;
use tempfile::tempdir;

#[test]
fn parses_weighted_role_routed_query_with_generic_cues_and_backend_namespaces() {
    let source = r#"---
schema: memhooks/v2
recall_queries:
  - query: "What security boundaries apply?"
    priority: 1.0
    when:
      roles: [reviewer, architect]
    entities:
      - Authentication
    resources:
      - name: auth-design
        kind: architecture-note
        salience: 0.8
    backends:
      hindsight:
        memory_types: [experience]
      mem0:
        top_k: 8
        rerank: true
entities:
  - name: Authentication
    type: CONCEPT
    salience: 0.9
---
"#;
    let parsed = parse_hook_str("MEMHOOKS.md", source).unwrap();
    let query = &parsed.frontmatter.recall_queries[0];
    assert_eq!(query.priority(), Some(1.0));
    assert_eq!(
        query.roles(),
        &["reviewer".to_string(), "architect".to_string()]
    );
    assert!(query.backends().contains_key("hindsight"));
    assert!(query.backends().contains_key("mem0"));
    let entity = &parsed.frontmatter.entities[0];
    assert_eq!(entity.salience(), Some(0.9));
    let resource = &query.resources()[0];
    assert_eq!(resource.salience(), Some(0.8));
}

#[test]
fn validator_rejects_out_of_range_weights_and_provider_fields_in_core() {
    let source = r#"---
schema: memhooks/v2
memory_types: [world]
recall_queries:
  - query: bad
    priority: 1.5
entities:
  - name: Example
    salience: -0.1
resources:
  - name: design
    salience: 2.0
---
"#;
    let parsed = parse_hook_str("MEMHOOKS.md", source).unwrap();
    let diagnostics = validate_parsed(&parsed);
    assert!(diagnostics
        .iter()
        .any(|d| d.code == "MH004" && d.severity == Severity::Error));
    assert!(diagnostics
        .iter()
        .any(|d| d.code == "MH009" && d.severity == Severity::Error));
    assert!(diagnostics
        .iter()
        .any(|d| d.code == "MH016" && d.severity == Severity::Error));
    assert!(diagnostics
        .iter()
        .any(|d| d.code == "MH021" && d.severity == Severity::Error));
}

#[test]
fn diagnostic_span_points_to_the_actual_second_priority() {
    let source = r#"---
schema: memhooks/v2
recall_queries:
  - query: first
    priority: 0.5
  - query: second
    priority: 9.9
---
"#;
    let parsed = parse_hook_str("MEMHOOKS.md", source).unwrap();
    let diagnostics = validate_parsed(&parsed);
    let priority = diagnostics.iter().find(|d| d.code == "MH004").unwrap();
    assert_eq!(priority.line, Some(7));
    assert_eq!(priority.column, Some(5));
}

#[test]
fn misspelled_query_key_is_linted_without_killing_the_file() {
    let source = r#"---
schema: memhooks/v2
recall_queries:
  - quer: "typo in the key"
  - query: "still parsed"
---
"#;
    let parsed = parse_hook_str("MEMHOOKS.md", source).unwrap();
    assert_eq!(parsed.frontmatter.recall_queries.len(), 2);
    let diagnostics = validate_parsed(&parsed);
    assert!(diagnostics
        .iter()
        .any(|d| d.code == "MH003" && d.severity == Severity::Error));
    let unknown = diagnostics.iter().find(|d| d.code == "MH005").unwrap();
    assert_eq!(unknown.line, Some(4));
}

#[test]
fn resolver_honors_inheritance_cut_and_role_filtering() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    fs::create_dir(root.join(".git")).unwrap();
    fs::create_dir_all(root.join("src/auth")).unwrap();

    fs::write(
        root.join("MEMHOOKS.md"),
        r#"---
schema: memhooks/v2
recall_queries:
  - "root query"
---
"#,
    )
    .unwrap();
    fs::write(
        root.join("src/MEMHOOKS.md"),
        r#"---
schema: memhooks/v2
inherits: false
recall_queries:
  - query: "review only"
    priority: 0.8
    when:
      roles: [reviewer]
  - "always"
---
"#,
    )
    .unwrap();

    let resolved = resolve(&root.join("src/auth")).unwrap();
    assert_eq!(resolved.sources.len(), 1);
    assert_eq!(resolved.recall_queries.len(), 2);

    let reviewer = resolved.effective_queries(&["reviewer".into()]);
    assert_eq!(reviewer.len(), 2);
    let feature = resolved.effective_queries(&["feature_dev".into()]);
    assert_eq!(feature.len(), 1);
    assert_eq!(feature[0].query, "always");
}

#[test]
fn local_query_with_same_text_overrides_parent_metadata() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    fs::create_dir(root.join(".git")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("MEMHOOKS.md"),
        r#"---
schema: memhooks/v2
recall_queries:
  - query: "same text"
    priority: 0.2
---
"#,
    )
    .unwrap();
    fs::write(
        root.join("src/MEMHOOKS.md"),
        r#"---
schema: memhooks/v2
recall_queries:
  - query: "same text"
    priority: 0.9
---
"#,
    )
    .unwrap();

    let resolved = resolve(&root.join("src")).unwrap();
    assert_eq!(resolved.recall_queries.len(), 1);
    assert_eq!(resolved.effective_queries(&[])[0].priority, Some(0.9));
    assert!(resolved.recall_queries[0]
        .source
        .ends_with("src/MEMHOOKS.md"));
}

#[test]
fn git_root_is_a_hard_boundary_for_outer_hooks() {
    let temp = tempdir().unwrap();
    let outer = temp.path().join("outer");
    let repo = outer.join("repo");
    let sub = repo.join("sub");
    fs::create_dir_all(&sub).unwrap();
    fs::create_dir(repo.join(".git")).unwrap();
    fs::write(
        outer.join("MEMHOOKS.md"),
        "---\nschema: memhooks/v2\nrecall_queries: [outer]\n---\n",
    )
    .unwrap();

    assert_eq!(find_root(&sub), repo);
    let resolved = resolve(&sub).unwrap();
    assert!(resolved.sources.is_empty());
    assert!(resolved.recall_queries.is_empty());
}

#[test]
fn nonexistent_resolve_target_is_an_error() {
    let temp = tempdir().unwrap();
    let missing = temp.path().join("does-not-exist");
    let error = resolve(&missing).unwrap_err();
    assert_eq!(error.code, "MH024");
}

#[test]
fn backend_namespaces_deep_merge_without_core_interpretation() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    fs::create_dir(root.join(".git")).unwrap();
    fs::create_dir_all(root.join("src/auth")).unwrap();

    fs::write(
        root.join("MEMHOOKS.md"),
        r#"---
schema: memhooks/v2
backends:
  mem0:
    filters:
      user_id: alice
    threshold: 0.2
  hindsight:
    bank: project-memory
recall_queries:
  - "root"
---
"#,
    )
    .unwrap();

    fs::write(
        root.join("src/MEMHOOKS.md"),
        r#"---
schema: memhooks/v2
backends:
  mem0:
    threshold: 0.3
    rerank: true
recall_queries:
  - query: "What changed?"
    backends:
      mem0:
        top_k: 5
      hindsight:
        memory_types: [experience]
---
"#,
    )
    .unwrap();

    let resolved = resolve(&root.join("src/auth")).unwrap();
    let effective = resolved.effective_queries(&[]);
    let query = effective
        .iter()
        .find(|query| query.query == "What changed?")
        .unwrap();
    let provider_json = serde_json::to_value(&query.backends).unwrap();

    assert_eq!(provider_json["mem0"]["filters"]["user_id"], json!("alice"));
    assert_eq!(provider_json["mem0"]["threshold"], json!(0.3));
    assert_eq!(provider_json["mem0"]["rerank"], json!(true));
    assert_eq!(provider_json["mem0"]["top_k"], json!(5));
    assert_eq!(provider_json["hindsight"]["bank"], json!("project-memory"));
    assert_eq!(
        provider_json["hindsight"]["memory_types"],
        json!(["experience"])
    );
}

#[test]
fn guidance_is_part_of_the_resolved_serializable_handoff() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    fs::create_dir(root.join(".git")).unwrap();
    fs::write(
        root.join("MEMHOOKS.md"),
        "---\nschema: memhooks/v2\n---\n\nUse direct recall first.\n",
    )
    .unwrap();
    let resolved = resolve(root).unwrap();
    assert_eq!(resolved.guidance.len(), 1);
    let json = serde_json::to_value(&resolved).unwrap();
    assert_eq!(json["guidance"][0]["value"], "Use direct recall first.");
}

#[test]
fn simple_queries_entities_and_resources_remain_valid_in_v2() {
    let source = r#"---
schema: memhooks/v2
recall_queries:
  - "What happened before?"
entities:
  - Authentication
resources:
  - architecture-notes
---
"#;
    let parsed = parse_hook_str("MEMHOOKS.md", source).unwrap();
    assert!(matches!(
        parsed.frontmatter.recall_queries[0],
        RecallQuery::Simple(_)
    ));
    assert!(matches!(parsed.frontmatter.entities[0], Entity::Simple(_)));
    assert!(matches!(
        parsed.frontmatter.resources[0],
        Resource::Simple(_)
    ));
    assert!(!validate_parsed(&parsed)
        .iter()
        .any(|d| d.severity == Severity::Error));
}

#[test]
fn v1_schema_is_not_accepted_by_v2_validator() {
    let source = r#"---
schema: memhooks/v1
recall_queries:
  - "old schema"
---
"#;
    let parsed = parse_hook_str("MEMHOOKS.md", source).unwrap();
    assert!(validate_parsed(&parsed)
        .iter()
        .any(|d| d.code == "MH001" && d.severity == Severity::Error));
}

#[test]
fn resolver_rejects_v1_schema_instead_of_silently_dropping_provider_fields() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    fs::create_dir(root.join(".git")).unwrap();
    fs::write(
        root.join("MEMHOOKS.md"),
        r#"---
schema: memhooks/v1
memory_types: [experience]
recall_queries:
  - "old schema query"
---
"#,
    )
    .unwrap();

    let error = resolve(root).unwrap_err();
    assert!(error.message.contains("memhooks/v2"));
}
