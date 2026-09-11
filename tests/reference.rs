use memhooks::{
    parse_hook_str, resolve, validate_parsed, Entity, RecallQuery, Resource, Severity,
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
    assert!(matches!(parsed.frontmatter.resources[0], Resource::Simple(_)));
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
