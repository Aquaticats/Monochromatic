//! Controls of the note that names an instance's service:
//! its text, its reading, and the birth identity that tells a reused number apart.

use super::*;

/// A folder of this test's own under the build directory.
fn folder(name: &str) -> std::path::PathBuf {
    let root: std::path::PathBuf = crate::test_support::fixture_root().join(format!("pid-note-{name}"));
    std::fs::create_dir_all(&root).expect("fixture folder");
    return root;
}

/// What the note of this process names is this process.
#[test]
fn the_note_names_the_process_that_wrote_it() {
    let root = folder("written");
    let note = path(root.as_path(), "3f9a");
    write(note.as_path(), i32::try_from(std::process::id()).expect("a process number")).expect("the note is written");
    let written = read(note.as_path()).expect("the note reads back");
    assert_eq!(written.pid, i32::try_from(std::process::id()).expect("a process number"));
    assert!(is_still_the_process(&written), "the identity the note holds is this process's");
    remove(note.as_path());
    assert!(read(note.as_path()).is_none(), "a removed note names nothing");
}

/// A note whose identity is another birth names a process that is gone,
/// although its number is this one's.
#[test]
fn a_note_with_another_birth_names_nothing() {
    let root = folder("reused");
    let note = path(root.as_path(), "3f9a");
    std::fs::write(note.as_path(), format!("{} linux:000000\n", std::process::id())).expect("a note of another birth");
    let written = read(note.as_path()).expect("the note reads back");
    assert!(!is_still_the_process(&written), "the same number with another birth is another process");
}

/// A text that is not a note names nothing,
/// so a file someone else put there is never a reason to signal a process.
#[test]
fn text_that_is_not_a_note_names_nothing() {
    assert!(parse("").is_none());
    assert!(parse("nonsense\n").is_none());
    assert!(parse("12 linux:99\n").is_some());
}
