//! What:
//!  The notice that names the saved copies and the text of a refused save.
//! Why:
//!  A copy nobody is told about is a copy the person will not find,
//!  and the notice must name the command,
//!  the copies and the paths without repeating a long path once per file.
//!  A skip is told apart from an abort:
//!  one restores the state before the operation, the other goes on with the next commit,
//!  and the notice says which.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(savedBeforeTheCommandNotice('merge', copies)).toContain('git merge --abort');
//! ```

use super::{abort_command_text, copy_failed_text, saved_before_the_command_notice, untracked_file_text};
use crate::conflict_side_source::FailureKind;
use crate::conflict_sides::SavedCopy;
use crate::rule_abort_operation::{AbortOperation, OperationEnd};
use crate::rule_worktree_dirty::SHARED_PATHS_NAMED;
use std::path::PathBuf;

/// The folder every copy of these controls is in.
fn folder() -> std::path::PathBuf {
    return PathBuf::from("/repo/.git/monochromatic-git-policy-cli/discarded/2026-11-10T10-00-00");
}

/// A saved copy at a place under one directory.
fn saved(path: &str, place: &str) -> SavedCopy {
    return SavedCopy {
        path: path.to_string(),
        saved_to: Some(PathBuf::from(place)),
    };
}

/// Every operation names its command in the spelling a person typed,
///  and a skip names its own command.
#[test]
fn every_operation_names_its_command() {
    assert_eq!(abort_command_text(AbortOperation::Merge, OperationEnd::Abort), "git merge --abort");
    assert_eq!(abort_command_text(AbortOperation::Rebase, OperationEnd::Abort), "git rebase --abort");
    assert_eq!(
        abort_command_text(AbortOperation::Pick, OperationEnd::Abort),
        "git cherry-pick --abort or git revert --abort"
    );
    assert_eq!(abort_command_text(AbortOperation::Am, OperationEnd::Abort), "git am --abort");
    assert_eq!(abort_command_text(AbortOperation::Rebase, OperationEnd::Skip), "git rebase --skip");
    assert_eq!(
        abort_command_text(AbortOperation::Pick, OperationEnd::Skip),
        "git cherry-pick --skip or git revert --skip"
    );
    assert_eq!(abort_command_text(AbortOperation::Am, OperationEnd::Skip), "git am --skip");
}

/// The notice names the directory once,
///  each copy below it,
///  the command that runs next,
///  and, in plain words, the comparison the wrapper makes before Git starts.
#[test]
fn the_notice_names_the_directory_each_copy_and_the_command() {
    let copies: Vec<SavedCopy> = vec![
        saved("a.txt", "/repo/.git/monochromatic-git-policy-cli/discarded/2026-11-10T10-00-00/a.txt"),
        saved("b/c.txt", "/repo/.git/monochromatic-git-policy-cli/discarded/2026-11-10T10-00-00/b/c.txt"),
    ];
    let notice: String = saved_before_the_command_notice(AbortOperation::Merge, OperationEnd::Abort, AbortOperation::Merge, copies.as_slice(), folder().as_path());
    assert!(notice.contains("copies are under /repo/.git/monochromatic-git-policy-cli/discarded/2026-11-10T10-00-00"), "{notice}");
    assert!(notice.contains("a.txt => "), "{notice}");
    assert!(notice.contains("b/c.txt => "), "{notice}");
    assert!(notice.contains("git merge --abort restores the state before the merge"), "{notice}");
    assert!(notice.contains("before the merge"), "{notice}");
    // The long directory is not repeated for the second copy.
    assert_eq!(notice.matches("discarded/2026-11-10T10-00-00/b/c.txt").count(), 1, "{notice}");
    // The window no program outside Git can close is said in plain words.
    assert!(notice.contains("compares each of these files with its copy once more"), "{notice}");
    assert!(notice.contains("a write that lands after that last comparison is not saved"), "{notice}");
    assert!(notice.contains("nothing you wrote before that is lost"), "{notice}");
}

/// A skip says the operation goes on with the next commit,
///  never that the state before the operation comes back.
#[test]
fn a_skip_says_the_operation_goes_on() {
    let copies: Vec<SavedCopy> = vec![saved("a.txt", "/repo/.git/discarded/2026-11-10T10-00-00/a.txt")];
    let notice: String = saved_before_the_command_notice(AbortOperation::Rebase, OperationEnd::Skip, AbortOperation::Rebase, copies.as_slice(), folder().as_path());
    assert!(notice.contains("git rebase --skip discards them"), "{notice}");
    assert!(notice.contains("git rebase --skip goes on with the next commit or patch"), "{notice}");
    assert!(!notice.contains("restores the state before"), "{notice}");
}

/// A path the worktree no longer holds says there was nothing to copy.
#[test]
fn a_deleted_path_says_there_was_nothing_to_copy() {
    let copies: Vec<SavedCopy> = vec![SavedCopy { path: String::from("gone.txt"), saved_to: None }];
    let notice: String = saved_before_the_command_notice(AbortOperation::Rebase, OperationEnd::Abort, AbortOperation::Rebase, copies.as_slice(), folder().as_path());
    assert!(notice.contains("gone.txt was not in the worktree, so there was nothing to copy"), "{notice}");
    assert!(notice.contains("git rebase --abort restores the state before the rebase"), "{notice}");
    assert!(notice.contains("copies are under"), "the folder the command's copies are in is named: {notice}");
}

/// Past the shared number of names the notice counts the rest instead of listing them.
#[test]
fn the_notice_counts_the_paths_beyond_the_shared_number() {
    let mut copies: Vec<SavedCopy> = Vec::new();
    for number in 0..=SHARED_PATHS_NAMED {
        copies.push(saved(format!("f{number}.txt").as_str(), format!("/repo/discarded/2026-11-10T10-00-00/f{number}.txt").as_str()));
    }
    let notice: String = saved_before_the_command_notice(AbortOperation::Am, OperationEnd::Abort, AbortOperation::Am, copies.as_slice(), folder().as_path());
    assert!(notice.contains("and 1 more in the same directory"), "{notice}");
    assert!(!notice.contains(&format!("f{SHARED_PATHS_NAMED}.txt")), "{notice}");
}

/// A copy that could not be written refuses the command and names what failed,
///  and the advice fits what went wrong.
#[test]
fn the_refused_save_names_the_command_the_operation_and_the_reason() {
    let text: String = copy_failed_text(
        AbortOperation::Pick,
        OperationEnd::Abort,
        AbortOperation::Pick,
        "a.txt",
        "no space left on device",
        FailureKind::Io,
    );
    assert!(text.contains("a.txt"), "{text}");
    assert!(text.contains("no space left on device"), "{text}");
    assert!(text.contains("git cherry-pick --abort or git revert --abort"), "{text}");
    assert!(text.contains("--no-enforce-worktree"), "{text}");
    assert!(text.contains("nothing was changed"), "{text}");
    assert!(text.contains("Free the space or fix the permissions"), "the advice fits a system failure: {text}");
    // A source that changed under the copy needs quiet, not free space.
    let changed: String = copy_failed_text(
        AbortOperation::Rebase,
        OperationEnd::Skip,
        AbortOperation::Rebase,
        "big.bin",
        "changed while it was copied",
        FailureKind::Changed,
    );
    assert!(changed.contains("git rebase --skip is rejected"), "{changed}");
    assert!(changed.contains("once nothing is writing to that file"), "{changed}");
    assert!(!changed.contains("Free the space"), "{changed}");
}

/// An `am` abort that ends a rebase names the rebase in the notice,
///  while the command stays the one the person typed.
#[test]
fn an_am_abort_that_ends_a_rebase_names_the_rebase() {
    let copies: Vec<SavedCopy> = vec![saved("a.txt", "/repo/.git/discarded/2026-11-10T10-00-00/a.txt")];
    let notice: String = saved_before_the_command_notice(AbortOperation::Am, OperationEnd::Abort, AbortOperation::Rebase, copies.as_slice(), folder().as_path());
    assert!(notice.contains("besides the rebase in progress"), "{notice}");
    assert!(notice.contains("git am --abort restores the state before the rebase"), "{notice}");
    assert!(notice.contains("before the rebase"), "{notice}");
    assert!(!notice.contains("the am in progress"), "{notice}");
    let text: String = copy_failed_text(
        AbortOperation::Am,
        OperationEnd::Abort,
        AbortOperation::Rebase,
        "a.txt",
        "no space left on device",
        FailureKind::Io,
    );
    assert!(text.contains("the rebase in progress holds changes"), "{text}");
    assert!(!text.contains("the am in progress"), "{text}");
}

/// An untracked file that stops Git's own reset is named with the way forward,
///  and the refusal says Git will not abort while it is there.
#[test]
fn the_untracked_refusal_names_the_file_and_what_moves_the_person_forward() {
    let text: String = untracked_file_text(
        AbortOperation::Rebase,
        OperationEnd::Abort,
        AbortOperation::Rebase,
        [String::from("victim.txt")].as_slice(),
    );
    assert!(text.contains("git rebase --abort is rejected"), "{text}");
    assert!(text.contains("the untracked file victim.txt"), "{text}");
    assert!(text.contains("Git will not restore that tree while it is there"), "{text}");
    assert!(text.contains("the rebase in progress still waits"), "{text}");
    assert!(text.contains("mv <path>"), "{text}");
    assert!(text.contains("nothing was saved or changed"), "{text}");
    let many: String = untracked_file_text(
        AbortOperation::Am,
        OperationEnd::Abort,
        AbortOperation::Am,
        [String::from("a.txt"), String::from("b.txt")].as_slice(),
    );
    assert!(many.contains("untracked files a.txt, b.txt"), "{many}");
    assert!(many.contains("while they are there"), "{many}");
}
