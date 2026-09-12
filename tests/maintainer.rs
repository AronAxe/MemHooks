use memhooks::{
    add_note, handle_event, init, parse_hook, resolve, BackendMap, NoteInput, RecallQuery,
};
use serde_json::json;
use std::fs;
use tempfile::tempdir;

fn make_repo() -> tempfile::TempDir {
    let temp = tempdir().unwrap();
    fs::create_dir(temp.path().join(".git")).unwrap();
    temp
}

#[test]
fn note_written_by_maintainer_is_immediately_visible_to_resolver() {
    let repo = make_repo();
    init(repo.path()).unwrap();
    let backends: BackendMap =
        serde_yaml_ng::from_str("mem0:\n  top_k: 8\n  rerank: true\n").unwrap();

    add_note(
        repo.path(),
        NoteInput {
            query: "Why did auth change?".into(),
            priority: Some(0.9),
            roles: vec!["reviewer".into()],
            tags: vec!["security".into()],
            backends,
            ..NoteInput::default()
        },
    )
    .unwrap();

    let resolved = resolve(repo.path()).unwrap();
    let query = resolved
        .effective_queries(&["reviewer".into()])
        .into_iter()
        .find(|query| query.query == "Why did auth change?")
        .unwrap();
    assert_eq!(query.priority, Some(0.9));
    assert_eq!(query.roles, vec!["reviewer"]);
    assert!(query.tags.contains(&"security".to_string()));
    assert_eq!(
        serde_json::to_value(query.backends).unwrap()["mem0"]["top_k"],
        json!(8)
    );

    let text = fs::read_to_string(repo.path().join("MEMHOOKS.md")).unwrap();
    assert!(text.contains("recall_queries:"));
    assert!(!text.contains("memhooks:notes:start"));
}

#[test]
fn automatic_event_anchors_only_explicit_existing_path_fields() {
    let repo = make_repo();
    init(repo.path()).unwrap();
    fs::create_dir_all(repo.path().join("src")).unwrap();
    fs::write(repo.path().join("src/real.py"), "print('ok')\n").unwrap();

    let payload = json!({
        "hook_event_name": "post_tool_call",
        "cwd": repo.path(),
        "tool_input": {
            "path": "src/real.py",
            "content": "import foo.bar\nrequests.get(\"http://x.com/a.js\")\n"
        }
    });
    assert_eq!(handle_event(&payload).unwrap(), 1);

    let parsed = parse_hook(repo.path().join("src/MEMHOOKS.md")).unwrap();
    let auto = parsed
        .frontmatter
        .recall_queries
        .iter()
        .find(|query| query.tags().iter().any(|tag| tag == "memhooks:auto"))
        .unwrap();
    let names = auto
        .resources()
        .iter()
        .map(|resource| resource.name())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["src/real.py"]);
    assert!(!names.iter().any(|name| name.contains("foo.bar")));
    assert!(!names.iter().any(|name| name.contains("requests.get")));

    let resolved = resolve(repo.path().join("src")).unwrap();
    assert!(resolved.effective_queries(&[]).iter().any(|query| query
        .resources
        .iter()
        .any(|resource| resource.name() == "src/real.py")));
}

#[test]
fn maintainer_never_climbs_above_a_git_root_to_capture_writes() {
    let temp = tempdir().unwrap();
    let outer = temp.path().join("outer");
    let repo = outer.join("repo");
    fs::create_dir_all(repo.join(".git")).unwrap();
    fs::write(outer.join("MEMHOOKS.md"), "---\nschema: memhooks/v2\n---\n").unwrap();

    let error = add_note(
        &repo,
        NoteInput {
            query: "should not escape".into(),
            ..NoteInput::default()
        },
    )
    .unwrap_err();
    assert!(error.to_string().contains(&repo.display().to_string()));
    let outer_text = fs::read_to_string(outer.join("MEMHOOKS.md")).unwrap();
    assert!(!outer_text.contains("should not escape"));
}

#[test]
fn note_round_trip_preserves_unknown_future_fields_and_provider_data() {
    let repo = make_repo();
    fs::write(
        repo.path().join("MEMHOOKS.md"),
        r#"---
schema: memhooks/v2
recall_queries:
  - query: "future compatible"
    x_future:
      enabled: true
    backends:
      mem0:
        filters:
          user_id: alice
---
"#,
    )
    .unwrap();

    add_note(
        repo.path(),
        NoteInput {
            query: "future compatible".into(),
            tags: vec!["updated".into()],
            ..NoteInput::default()
        },
    )
    .unwrap();

    let parsed = parse_hook(repo.path().join("MEMHOOKS.md")).unwrap();
    let RecallQuery::Structured(query) = &parsed.frontmatter.recall_queries[0] else {
        panic!("expected structured query");
    };
    assert!(query.extra.contains_key("x_future"));
    assert!(query.backends.contains_key("mem0"));
    assert!(query.tags.contains(&"updated".to_string()));
}
