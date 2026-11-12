//! What:
//!  The notice that names the saved copies and the text of a refused save.
//! Why:
//!  A copy nobody is told about is a copy the person will not find,
//!  and the notice must name the command,
//!  the copies and the paths without repeating a long path once per file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(savedBeforeAbortNotice('merge', copies)).toContain('git merge --abort');
//! ```

use super::{abort_command_text, copy_failed_text, saved_before_abort_notice};
use crate::conflict_sides::SavedCopy;
use crate::rule_abort_operation::AbortOperation;
use crate::rule_worktree_dirty::SHARED_PATHS_NAMED;
use std::path::PathBuf;

/// A saved copy at a place under one directory.
fn saved(path: &str, place: &str) -> SavedCopy {
    return SavedCopy {
        path: path.to_string(),
        saved_to: Some(PathBuf::from(place)),
    };
}

/// Every operation names its command in the spelling a person typed.
#[test]
fn every_operation_names_its_command() {
    assert_eq!(abort_command_text(AbortOperation::Merge), "git merge --abort");
    assert_eq!(abort_command_text(AbortOperation::Rebase), "git rebase --abort");
    assert_eq!(abort_command_text(AbortOperation::Pick), "git cherry-pick --abort or git revert --abort");
    assert_eq!(abort_command_text(AbortOperation::Am), "git am --abort");
}

/// The notice names the directory once,
///  each copy below it,
///  and the command that runs next.
#[test]
fn the_notice_names_the_directory_each_copy_and_the_command() {
    let copies: Vec<SavedCopy> = vec![
        saved("a.txt", "/repo/.git/monochromatic-git-policy-cli/discarded/2026-11-10T10-00-00/a.txt"),
        saved("b/c.txt", "/repo/.git/monochromatic-git-policy-cli/discarded/2026-11-10T10-00-00/b/c.txt"),
    ];
    let notice: String = saved_before_abort_notice(AbortOperation::Merge, copies.as_slice());
    assert!(notice.contains("copies are under /repo/.git/monochromatic-git-policy-cli/discarded/2026-11-10T10-00-00"), "{notice}");
    assert!(notice.contains("a.txt => "), "{notice}");
    assert!(notice.contains("b/c.txt => "), "{notice}");
    assert!(notice.contains("git merge --abort runs now"), "{notice}");
    assert!(notice.contains("before the merge"), "{notice}");
    // The long directory is not repeated for the second copy.
    assert_eq!(notice.matches("discarded/2026-11-10T10-00-00/b/c.txt").count(), 1, "{notice}");
}

/// A path the worktree no longer holds says there was nothing to copy.
#[test]
fn a_deleted_path_says_there_was_nothing_to_copy() {
    let copies: Vec<SavedCopy> = vec![SavedCopy { path: String::from("gone.txt"), saved_to: None }];
    let notice: String = saved_before_abort_notice(AbortOperation::Rebase, copies.as_slice());
    assert!(notice.contains("gone.txt was deleted in the worktree, so there was nothing to copy"), "{notice}");
    assert!(notice.contains("git rebase --abort runs now"), "{notice}");
    assert!(!notice.contains("copies are under"), "{notice}");
}

/// Past the shared number of names the notice counts the rest instead of listing them.
#[test]
fn the_notice_counts_the_paths_beyond_the_shared_number() {
    let mut copies: Vec<SavedCopy> = Vec::new();
    for number in 0..=SHARED_PATHS_NAMED {
        copies.push(saved(format!("f{number}.txt").as_str(), format!("/repo/discarded/2026-11-10T10-00-00/f{number}.txt").as_str()));
    }
    let notice: String = saved_before_abort_notice(AbortOperation::Am, copies.as_slice());
    assert!(notice.contains("and 1 more in the same directory"), "{notice}");
    assert!(!notice.contains(&format!("f{SHARED_PATHS_NAMED}.txt")), "{notice}");
}

/// A copy that could not be written refuses the command and names what failed.
#[test]
fn the_refused_save_names_the_command_the_operation_and_the_reason() {
    let text: String = copy_failed_text(AbortOperation::Pick, "a.txt", "no space left on device");
    assert!(text.contains("a.txt"), "{text}");
    assert!(text.contains("no space left on device"), "{text}");
    assert!(text.contains("git cherry-pick --abort or git revert --abort"), "{text}");
    assert!(text.contains("--no-enforce-worktree"), "{text}");
    assert!(text.contains("nothing was changed"), "{text}");
}
