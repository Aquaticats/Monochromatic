//! What:
//!  Controls of the record a removal leaves for a worktree it kept:
//!  the words the person reads,
//!  that they are read once,
//!  and that nothing is removed before they are out.
//! Why:
//!   The worktree a service keeps stays in the repository with no command of the merge left to say so,
//!  so the record is the only statement a person gets,
//!  and a record read twice or never is either noise or silence.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recordKeptWorktree(commonDir, { worktree, branch, why }); const texts = shown(keptWorktreeNotices(commonDir)); expect(texts[0]).toContain(...)
//! ```

use super::{kept_notice_directory, kept_worktree_notices, record_kept_worktree};
use crate::push_failure_notice_shown_then_removed::Notice;
use crate::test_support::{fixture, shown_notices};
use std::path::{Path, PathBuf};

/// What:
///  The words one command shows for the records this repository holds.
/// Why:
///   A record goes only once its words are written out,
/// so a control reads them through the step the person's terminal receives them by.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const texts = shownNotices(keptWorktreeNotices(commonDir));
/// ```
fn shown(common_dir: &Path) -> Vec<String> {
    return shown_notices(kept_worktree_notices(common_dir));
}

/// What:
///  A kept worktree is named with its path, its branch and its reason,
///  and a second command finds nothing.
/// Why:
///  The person has to be able to act on the line,
///  and a record that was shown once must not come back on every command.
#[test]
fn a_kept_worktree_is_named_once_with_its_path_its_branch_and_its_reason() {
    let root: PathBuf = fixture("kept-notice-once");
    let worktree: PathBuf = root.join("topic");
    record_kept_worktree(root.as_path(), worktree.as_path(), "refs/heads/topic", "another worktree this repository registers is inside it (`/x`)")
        .expect("the record of a worktree a removal kept");
    let texts: Vec<String> = shown(root.as_path());
    assert_eq!(texts.len(), 1, "one record, one line: {texts:?}");
    assert!(texts[0].contains(worktree.display().to_string().as_str()), "the line names the worktree: {texts:?}");
    assert!(texts[0].contains("refs/heads/topic"), "the line names the branch: {texts:?}");
    assert!(texts[0].contains("another worktree this repository registers is inside it"), "the line names the reason: {texts:?}");
    assert!(texts[0].contains("git worktree remove"), "the line names the command that removes it: {texts:?}");
    assert!(shown(root.as_path()).is_empty(), "a record is shown once");
    assert!(!kept_notice_directory(root.as_path()).join("kept-nothing.txt").exists(), "no other record was written");
}

/// What:
///  A record whose words could not be written goes back to its own name,
///  and the next command shows it.
/// Why:
///  A record removed before its words reach the person is gone with nothing shown,
///  and a closed terminal is not a reason to lose the statement.
#[test]
fn a_record_whose_words_could_not_be_written_stays_for_the_next_command() {
    let root: PathBuf = fixture("kept-notice-unsaid");
    let worktree: PathBuf = root.join("topic");
    record_kept_worktree(root.as_path(), worktree.as_path(), "refs/heads/topic", "it is locked, and a lock is a promise to keep it")
        .expect("the record of a worktree a removal kept");
    let notices: Vec<Notice> = kept_worktree_notices(root.as_path());
    assert_eq!(notices.len(), 1, "one record to show");
    let refused: std::io::Result<()> = notices[0].show(&mut Failing);
    assert!(refused.is_err(), "a standard error that refuses the words says so");
    let again: Vec<String> = shown(root.as_path());
    assert_eq!(again.len(), 1, "the record is found by the next command: {again:?}");
    assert!(again[0].contains("it is locked"), "and it still says why: {again:?}");
}

/// A writer that refuses every byte,
/// as a closed terminal does.
struct Failing;

impl std::io::Write for Failing {
    fn write(&mut self, _bytes: &[u8]) -> std::io::Result<usize> {
        return Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe));
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe));
    }
}

/// What:
///  A file in the notice directory that this program did not write is neither shown nor removed.
/// Why:
///   The directory is one a person can see,
///  and printing another program's text as this program's sentence would be worse than silence,
///  while deleting it would be taking something of theirs.
#[test]
fn a_file_that_is_no_record_of_this_program_is_left_where_it_is() {
    let root: PathBuf = fixture("kept-notice-foreign");
    let directory: PathBuf = kept_notice_directory(root.as_path());
    std::fs::create_dir_all(directory.as_path()).expect("the notice directory");
    let foreign: PathBuf = directory.join("kept-foreign.txt");
    std::fs::write(foreign.as_path(), b"a note of another program\n").expect("a file this program did not write");
    assert!(shown(root.as_path()).is_empty(), "nothing of another program's is shown as this one's line");
    assert!(foreign.is_file(), "and nothing of it is removed: {}", foreign.display());
}
