//! Controls of the answer a source's own directory gives about a copy that can never be made.

use super::*;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// What:
///  A source that is gone or is a file is provably no worktree,
/// and a directory that stands there is still a question.
/// Why:
///   The registered-root listing cannot start in a path that is not a directory,
/// so that state must be told apart
///  from a refusal that a later command could answer differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(couldBeAWorktree(dir)).toBe(true); expect(couldBeAWorktree(file)).toBe(false); expect(couldBeAWorktree(gone)).toBe(false);
/// ```
#[test]
fn only_a_directory_can_still_be_a_worktree() {
    let root: PathBuf = fixture("owed-copy-source-answer");
    let directory: PathBuf = root.join("a-worktree");
    std::fs::create_dir(directory.as_path()).expect("a directory");
    assert!(source_could_be_a_worktree(directory.as_path()), "a directory a copy may still read is a question left open");
    let file: PathBuf = root.join("a-file");
    std::fs::write(file.as_path(), b"not a worktree\n").expect("a file");
    assert!(!source_could_be_a_worktree(file.as_path()), "a file where a worktree belongs is answered");
    let gone: PathBuf = root.join("removed");
    assert!(!source_could_be_a_worktree(gone.as_path()), "and so is a path nothing stands at");
    remove(root.as_path());
}

/// What:
///  The reason names the source as the worktree it no longer is,
/// with the path in the quoting every other notice uses.
/// Why:
///   The person reads which path to look at,
/// and a path holding a quote or a control byte must not break the line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(reason('/tmp/a"b')).toContain('\\"');
/// ```
#[test]
fn the_reason_names_the_source_and_quotes_its_bytes() {
    let reason: String = source_is_no_worktree_reason(Path::new("/a path with \"quotes\""));
    assert!(reason.starts_with("the worktree it copies from, "), "{reason}");
    assert!(reason.ends_with("is no longer a worktree of this repository"), "{reason}");
    assert!(reason.contains("\\\""), "the quote is escaped as every other notice escapes it: {reason}");
}
