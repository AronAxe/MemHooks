use memhooks::{add_note, init, resolve, NoteInput};
use std::fs;
use std::thread;
use tempfile::tempdir;

#[test]
fn concurrent_notes_are_locked_and_all_survive() {
    let repo = tempdir().unwrap();
    fs::create_dir(repo.path().join(".git")).unwrap();
    init(repo.path()).unwrap();
    let root = repo.path().to_path_buf();

    thread::scope(|scope| {
        for index in 0..8 {
            let root = root.clone();
            scope.spawn(move || {
                add_note(
                    &root,
                    NoteInput {
                        query: format!("Concurrent note {index}"),
                        tags: vec!["concurrency-test".into()],
                        ..NoteInput::default()
                    },
                )
                .unwrap();
            });
        }
    });

    let resolved = resolve(&root).unwrap();
    for index in 0..8 {
        let expected = format!("Concurrent note {index}");
        assert!(
            resolved
                .recall_queries
                .iter()
                .any(|query| query.value.text() == expected),
            "missing {expected} after concurrent writes"
        );
    }
}
