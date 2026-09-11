use memhooks::{parse_hook_str, resolve, validate_parsed, Entity, RecallQuery, Severity};
use std::fs;
use tempfile::tempdir;

#[test]
fn parses_weighted_role_routed_query_and_salient_entity() {
    let source = r#"---
schema: memhooks/v1
recall_queries:
  - query: "What security boundaries apply?"
    priority: 1.0
    when:
      roles: [reviewer, architect]
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
    let entity = &parsed.frontmatter.entities[0];
    assert_eq!(entity.salience(), Some(0.9));
}

#[test]
fn validator_rejects_out_of_range_weights() {
    let source = r#"---
schema: memhooks/v1
recall_queries:
  - query: bad
    priority: 1.5
entities:
  - name: Example
    salience: -0.1
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
schema: memhooks/v1
recall_queries:
  - "root query"
---
"#,
    )
    .unwrap();
    fs::write(
        root.join("src/MEMHOOKS.md"),
        r#"---
schema: memhooks/v1
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
fn legacy_strings_remain_valid() {
    let source = r#"---
schema: memhooks/v1
recall_queries:
  - "What happened before?"
entities:
  - Authentication
---
"#;
    let parsed = parse_hook_str("MEMHOOKS.md", source).unwrap();
    assert!(matches!(
        parsed.frontmatter.recall_queries[0],
        RecallQuery::Simple(_)
    ));
    assert!(matches!(parsed.frontmatter.entities[0], Entity::Simple(_)));
    assert!(!validate_parsed(&parsed)
        .iter()
        .any(|d| d.severity == Severity::Error));
}
