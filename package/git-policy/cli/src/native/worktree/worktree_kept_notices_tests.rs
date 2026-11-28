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

use super::{
    kept_notice_directory,
    kept_worktree_notices,
    record_kept_worktree,
};
use crate::push_failure_notice_shown_then_removed::Notice;
use crate::test_support::{
    fixture,
    remove,
    shown_notices,
};
use std::path::{
    Path,
    PathBuf,
};

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
///  A kept worktree is named with its path,
///  its branch and its reason,
///  and a second command finds nothing.
/// Why:
///  The person has to be able to act on the line,
///  and a record that was shown once must not come back on every command.
#[test]
fn a_kept_worktree_is_named_once_with_its_path_its_branch_and_its_reason() {
    let root: PathBuf = fixture("kept-notice-once");
    let worktree: PathBuf = root.join("topic");
    record_kept_worktree(
        root.as_path(),
        worktree.as_path(),
        "refs/heads/topic",
        "another worktree this repository registers is inside it (`/x`)",
    )
    .expect("the record of a worktree a removal kept");
    let texts: Vec<String> = shown(root.as_path());
    assert_eq!(texts.len(), 1, "one record, one line: {texts:?}");
    assert!(
        texts[0].contains(worktree.display().to_string().as_str()),
        "the line names the worktree: {texts:?}"
    );
    assert!(
        texts[0].contains("refs/heads/topic"),
        "the line names the branch: {texts:?}"
    );
    assert!(
        texts[0].contains("another worktree this repository registers is inside it"),
        "the line names the reason: {texts:?}"
    );
    assert!(
        texts[0].contains("git worktree remove"),
        "the line names the command that removes it: {texts:?}"
    );
    assert!(shown(root.as_path()).is_empty(), "a record is shown once");
    assert!(
        !kept_notice_directory(root.as_path())
            .join("kept-nothing.txt")
            .exists(),
        "no other record was written"
    );
}

/// What:
///  A record whose words could not be written goes back to its own name,
///  and the next command shows it.
/// Why:
///  A record removed before its words reach the person is gone with nothing shown,
///  and a closed terminal is not a reason to lose the statement.
///  This is the writer the notice is handed directly;
///  the stream every command uses answers the same way from the thread that writes it,
///  which the control below shows.
#[test]
fn a_record_whose_words_could_not_be_written_stays_for_the_next_command() {
    let root: PathBuf = fixture("kept-notice-unsaid");
    let worktree: PathBuf = root.join("topic");
    record_kept_worktree(
        root.as_path(),
        worktree.as_path(),
        "refs/heads/topic",
        "it is locked, and a lock is a promise to keep it",
    )
    .expect("the record of a worktree a removal kept");
    let notices: Vec<Notice> = kept_worktree_notices(root.as_path());
    assert_eq!(notices.len(), 1, "one record to show");
    let refused: std::io::Result<()> =
        notices[0].show(&mut crate::test_support::AStreamThatRefusesEveryByte);
    assert!(
        refused.is_err(),
        "a standard error that refuses the words says so"
    );
    let again: Vec<String> = shown(root.as_path());
    assert_eq!(
        again.len(),
        1,
        "the record is found by the next command: {again:?}"
    );
    assert!(
        again[0].contains("it is locked"),
        "and it still says why: {again:?}"
    );
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
    std::fs::write(foreign.as_path(), b"a note of another program\n")
        .expect("a file this program did not write");
    assert!(
        shown(root.as_path()).is_empty(),
        "nothing of another program's is shown as this one's line"
    );
    assert!(
        foreign.is_file(),
        "and nothing of it is removed: {}",
        foreign.display()
    );
}

/// A kept-notice record that is a named pipe says nothing and holds no command,
/// at the bound:
/// opening a named pipe for reading waits for a writer that may never come,
/// and every command of the repository shows these records (issue 6415).
#[test]
fn a_kept_notice_that_is_a_named_pipe_is_answered_at_once() {
    let root: PathBuf = fixture("kept-notice-named-pipe");
    let directory: PathBuf = kept_notice_directory(root.as_path());
    std::fs::create_dir_all(directory.as_path()).expect("the kept-notice folder");
    crate::subvolume_sync_attrs::make_fifo(
        directory.join("kept-0123456789abcdef.txt").as_path(),
        0o600,
    )
    .expect("a named pipe where a notice goes");
    let inside: PathBuf = root.clone();
    let answered = crate::test_support::finished_within(move || return shown(inside.as_path()));
    let Some(texts) = answered else {
        panic!("a kept notice that is a named pipe held the command")
    };
    assert!(texts.is_empty(), "a pipe is no notice");
    remove(root.as_path());
}

/// What:
///  A kept worktree's record whose line the thread that writes standard error could not put in the
///  stream stays for the next command.
/// Why:
///   The record is removed only once its line is known written,
///  and the write itself answers before the line is written:
///  the thread's answer is what tells the two apart
///  (`foundation/standard_error_off_the_calling_thread.rs`).
///  The kept worktree stands until a person decides about it,
///  so a line nobody could read is the whole of what that decision has.
#[test]
fn a_record_whose_line_the_stream_could_not_take_stays_for_the_next_command() {
    crate::process_wide_state_child_test_support::run_the_body_in_a_child_of_its_own(
        module_path!(),
        "a_record_whose_line_the_stream_could_not_take_stays_for_the_next_command",
        || {
            let root: PathBuf = fixture("kept-notice-line-not-taken");
            let worktree: PathBuf = root.join("topic");
            record_kept_worktree(
                root.as_path(),
                worktree.as_path(),
                "refs/heads/topic",
                "it is locked, and a lock is a promise to keep it",
            )
            .expect("the record of a worktree a removal kept");
            let notices: Vec<Notice> = kept_worktree_notices(root.as_path());
            assert_eq!(notices.len(), 1, "one record to show");
            let mut notes = crate::test_support::standard_error_whose_writer_refuses_every_line();
            let refused: std::io::Result<()> = notices[0].show(&mut notes);
            assert!(
                refused.is_err(),
                "the thread that writes standard error says the line did not reach the person"
            );
            let again: Vec<String> = shown(root.as_path());
            assert_eq!(
                again.len(),
                1,
                "the record is found by the next command: {again:?}"
            );
            assert!(
                again[0].contains("it is locked"),
                "and it still says why: {again:?}"
            );
            remove(root.as_path());
        },
    );
}
