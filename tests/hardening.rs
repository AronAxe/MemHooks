use memhooks::{add_note, init, parse_hook_str, resolve, NoteInput};
use std::fs;
use tempfile::tempdir;

#[test]
fn library_rejects_invalid_note_inputs_without_mutation() {
    let repo = tempdir().unwrap();
    fs::create_dir(repo.path().join(".git")).unwrap();
    let hook = init(repo.path()).unwrap();
    let before = fs::read(&hook).unwrap();
    let invalid = vec![
        NoteInput {
            query: "".into(),
            ..Default::default()
        },
        NoteInput {
            query: "bad".into(),
            priority: Some(f64::NAN),
            ..Default::default()
        },
        NoteInput {
            query: "bad".into(),
            priority: Some(2.0),
            ..Default::default()
        },
        NoteInput {
            query: "bad".into(),
            roles: vec![" ".into()],
            ..Default::default()
        },
        NoteInput {
            query: "bad".into(),
            backends: serde_yaml_ng::from_str("mem0: 42").unwrap(),
            ..Default::default()
        },
        NoteInput {
            query: "bad".into(),
            entities: serde_yaml_ng::from_str("[{name: '', salience: 2}]").unwrap(),
            ..Default::default()
        },
    ];
    for input in invalid {
        assert!(add_note(repo.path(), input).is_err());
        assert_eq!(fs::read(&hook).unwrap(), before);
    }
}

#[test]
fn parsed_resolution_cannot_bypass_semantic_validation() {
    let hook = parse_hook_str("MEMHOOKS.md", "---\nschema: memhooks/v2\nrecall_queries:\n  - query: bad\n    when:\n      roles: reviewer\n---\n").unwrap();
    assert!(memhooks::resolver::resolve_parsed("/repo".into(), "/repo".into(), &[hook]).is_err());
}

#[test]
fn inheritance_cut_is_observable() {
    let repo = tempdir().unwrap();
    fs::create_dir(repo.path().join(".git")).unwrap();
    fs::create_dir(repo.path().join("src")).unwrap();
    fs::write(
        repo.path().join("MEMHOOKS.md"),
        "---\nschema: memhooks/v2\nrecall_queries: [parent]\n---\n",
    )
    .unwrap();
    fs::write(
        repo.path().join("src/MEMHOOKS.md"),
        "---\nschema: memhooks/v2\ninherits: false\nrecall_queries: [child]\n---\n",
    )
    .unwrap();
    let resolved = resolve(&repo.path().join("src")).unwrap();
    assert_eq!(resolved.effective_queries(&[])[0].query, "child");
    assert!(resolved
        .omitted_queries(&[])
        .iter()
        .any(|q| q.query == "parent" && q.reason == "inheritance_cut"));
}

#[cfg(unix)]
#[test]
fn hook_symlink_inside_root_is_also_rejected() {
    let repo = tempdir().unwrap();
    fs::create_dir(repo.path().join(".git")).unwrap();
    fs::write(
        repo.path().join("real.md"),
        "---\nschema: memhooks/v2\n---\n",
    )
    .unwrap();
    std::os::unix::fs::symlink("real.md", repo.path().join("MEMHOOKS.md")).unwrap();
    assert!(resolve(repo.path()).is_err());
    assert!(init(repo.path()).is_err());
}
