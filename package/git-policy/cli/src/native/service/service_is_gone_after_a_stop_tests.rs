//! Controls of the proof that a stopped instance serves nothing any more.

use super::*;

fn folder(name: &str) -> std::path::PathBuf {
    let root: std::path::PathBuf = crate::test_support::fixture_root().join(format!("gone-{name}"));
    std::fs::create_dir_all(&root).expect("fixture folder");
    return root;
}

/// An instance with no note is gone, and so is one whose number carries another birth.
#[test]
fn a_note_that_names_no_living_process_is_gone() {
    let root = folder("stale");
    assert!(service_is_gone(root.as_path(), "3f9a"), "no note at all is no service");
    let note = crate::service_server_pid_note_of_an_instance::path(root.as_path(), "3f9a");
    std::fs::write(note.as_path(), "1 linux:000000\n").expect("a stale note");
    assert!(service_is_gone(root.as_path(), "3f9a"), "a note of another birth names no service");
}

/// An instance whose note names a living process is not gone.
#[test]
fn a_note_that_names_a_living_process_is_not_gone() {
    let root = folder("living");
    let note = crate::service_server_pid_note_of_an_instance::path(root.as_path(), "3f9a");
    crate::service_server_pid_note_of_an_instance::write(note.as_path(), i32::try_from(std::process::id()).expect("a process number")).expect("the note");
    assert!(!service_is_gone(root.as_path(), "3f9a"), "the process the note names is this one");
}
