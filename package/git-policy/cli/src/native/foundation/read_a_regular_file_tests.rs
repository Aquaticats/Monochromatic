//! Controls of the reading that a named pipe cannot hold.

use super::*;

/// A folder of its own for one control.
fn folder(name: &str) -> std::path::PathBuf {
    let root: std::path::PathBuf = crate::test_support::fixture_root().join(format!("regular-{name}"));
    let _ = std::fs::remove_dir_all(root.as_path());
    std::fs::create_dir_all(root.as_path()).expect("the folder");
    return root;
}

/// What:
///  A regular file reads as its text, and a named pipe at the same kind of path reads as nothing, at once.
/// Why:
///  The review of `e347be3115` (part one, finding 3): a pipe where the service's note, log or worktree note goes held a commit,
///  `service stop` and `git worktree remove` for ever. Every reader of this program's own files answers nothing for a pipe now,
///  which the callers already handle as "the file is not there".
#[test]
fn a_regular_file_reads_and_a_named_pipe_answers_nothing_at_once() {
    let root = folder("pipe");
    let file: std::path::PathBuf = root.join("a-note");
    std::fs::write(file.as_path(), "12 linux:34\n").expect("a file");
    assert_eq!(read_regular_file(file.as_path()).as_deref(), Some("12 linux:34\n"), "a regular file reads as its text");
    assert!(is_regular_file(file.as_path()), "and is a regular file");
    let pipe: std::path::PathBuf = root.join("a-pipe");
    let made: std::process::ExitStatus = std::process::Command::new("/usr/bin/mkfifo").arg(pipe.as_path()).status().expect("mkfifo");
    assert!(made.success(), "the pipe is made");
    let began: std::time::Instant = std::time::Instant::now();
    assert_eq!(read_regular_file(pipe.as_path()), None, "a pipe answers nothing, with no writer and no bound");
    assert!(began.elapsed() < std::time::Duration::from_secs(1), "the read of a pipe ends at once: {:?}", began.elapsed());
    assert!(!is_regular_file(pipe.as_path()), "a pipe is not the regular file this program writes");
    assert_eq!(read_regular_file(root.join("not-there").as_path()), None, "a path that is there for nothing reads nothing");
    let _ = std::fs::remove_dir_all(root.as_path());
}
