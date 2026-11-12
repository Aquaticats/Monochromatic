//! What:
//!  The spellings and the texts of the abort and skip refusals.
//! Why:
//!  A refusal is the whole answer now that nothing is copied,
//!  so every piece of it is checked here:
//!  the command as typed,
//!  its opt-out in the one place the wrapper strips it,
//!  the quit form that keeps the work,
//!  each path's own kind and the count past the bound,
//!  and the two readings that answer for Git itself.
//!  A skip is told apart from an abort:
//!  one restores the state before the operation, the other goes on with the next commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(discardRefusalText({ operation: 'merge', ends: 'abort', paths })).toContain('git merge --abort');
//! ```

use super::{
    abort_command_text, discard_refusal_text, discarded_list, hatch_command_text, ignored_file_refusal_text, quit_command_text, unreadable_refusal_text,
    untracked_file_text,
};
use crate::repository_facts_abort_discard::DiscardBaseline;
use crate::rule_abort_operation::{AbortOperation, OperationEnd};
use crate::rule_worktree_dirty::SHARED_PATHS_NAMED;

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

/// The opt-out is spelled directly after the subcommand,
///  which is the only position the wrapper strips it from.
#[test]
fn every_operation_names_its_opt_out_after_the_subcommand() {
    assert_eq!(hatch_command_text(AbortOperation::Merge, OperationEnd::Abort), "git merge --no-enforce-worktree --abort");
    assert_eq!(hatch_command_text(AbortOperation::Rebase, OperationEnd::Abort), "git rebase --no-enforce-worktree --abort");
    assert_eq!(
        hatch_command_text(AbortOperation::Pick, OperationEnd::Abort),
        "git cherry-pick --no-enforce-worktree --abort"
    );
    assert_eq!(hatch_command_text(AbortOperation::Am, OperationEnd::Abort), "git am --no-enforce-worktree --abort");
    assert_eq!(hatch_command_text(AbortOperation::Rebase, OperationEnd::Skip), "git rebase --no-enforce-worktree --skip");
    assert_eq!(
        hatch_command_text(AbortOperation::Pick, OperationEnd::Skip),
        "git cherry-pick --no-enforce-worktree --skip"
    );
    assert_eq!(hatch_command_text(AbortOperation::Am, OperationEnd::Skip), "git am --no-enforce-worktree --skip");
}

/// Every operation that can be aborted has a quit form that keeps the work,
///  and a skipped operation is stopped with the same one.
#[test]
fn every_operation_names_its_quit_form() {
    assert_eq!(quit_command_text(AbortOperation::Merge), "git merge --quit");
    assert_eq!(quit_command_text(AbortOperation::Rebase), "git rebase --quit");
    assert_eq!(quit_command_text(AbortOperation::Pick), "git cherry-pick --quit or git revert --quit");
    assert_eq!(quit_command_text(AbortOperation::Am), "git am --quit");
}

/// A path is described by the listing that named it:
///  an edit in the worktree, a staged edit, a staged deletion,
///  and, for a merge, a staged resolution whose content stands nowhere else.
#[test]
fn each_path_carries_the_kind_of_work_standing_there() {
    assert_eq!(
        discarded_list(
            DiscardBaseline::RecordedResult,
            &[b"a.txt".to_vec()],
            &[b"a.txt".to_vec()],
            &[b"a.txt".to_vec()]
        ),
        "a.txt (a resolution or edit you made since Git stopped at the conflict; a staged resolution)"
    );
    assert_eq!(
        discarded_list(DiscardBaseline::RecordedResult, &[], &[b"new.txt".to_vec()], &[b"new.txt".to_vec()]),
        "new.txt (content only the index holds)"
    );
    assert_eq!(
        discarded_list(DiscardBaseline::RecordedResult, &[], &[b"gone.txt".to_vec()], &[]),
        "gone.txt (a deletion you staged)"
    );
    assert_eq!(
        discarded_list(DiscardBaseline::OperationStart, &[b"a.txt".to_vec()], &[], &[]),
        "a.txt (an edit you made since the operation began)"
    );
    assert_eq!(
        discarded_list(DiscardBaseline::OperationStart, &[], &[b"b.txt".to_vec()], &[b"b.txt".to_vec()]),
        "b.txt (content the index holds)"
    );
    assert_eq!(
        discarded_list(DiscardBaseline::OperationStart, &[b"c.txt".to_vec()], &[b"c.txt".to_vec()], &[b"c.txt".to_vec()]),
        "c.txt (an edit you made since the operation began; a staged edit)"
    );
}

/// The list names a bounded number of paths and counts the rest.
#[test]
fn the_list_is_bounded_and_counts_the_rest() {
    let mut paths: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for number in 0..SHARED_PATHS_NAMED + 3 {
        paths.push(format!("p{number}.txt").into_bytes());
    }
    let listed: String = discarded_list(DiscardBaseline::OperationStart, paths.as_slice(), &[], &[]);
    assert!(listed.contains("p0.txt"), "{listed}");
    assert!(listed.contains(&format!("and {} more paths", 3)), "{listed}");
    assert!(!listed.contains(&format!("p{}.txt", SHARED_PATHS_NAMED + 2)), "{listed}");
    assert_eq!(listed.matches("an edit you made since the operation began").count(), SHARED_PATHS_NAMED, "{listed}");
}

/// A refusal names the command, each path, the quit form and the opt-out,
///  and never claims a copy was made.
#[test]
fn the_refusal_names_both_ways_out_and_promises_no_copy() {
    let text: String = discard_refusal_text(
        AbortOperation::Rebase,
        OperationEnd::Abort,
        AbortOperation::Rebase,
        DiscardBaseline::OperationStart,
        &[b"a.txt".to_vec()],
        &[],
        &[],
    );
    assert!(text.starts_with("monochromatic-git-policy-cli: git rebase --abort is refused"), "{text}");
    assert!(text.contains("a.txt (an edit you made since the operation began)"), "{text}");
    assert!(text.contains("Nothing was changed, and nothing was copied."), "{text}");
    assert!(text.contains("git rebase --quit"), "{text}");
    assert!(text.contains("git rebase --no-enforce-worktree --abort"), "{text}");
    assert!(!text.contains("saved"), "no sentence claims a copy: {text}");
    assert!(!text.contains('\n'), "the finding is one line: {text}");
}

/// A skip says it goes on with the next commit,
///  never that the state before the operation comes back.
#[test]
fn a_skip_says_it_goes_on() {
    let text: String = discard_refusal_text(
        AbortOperation::Am,
        OperationEnd::Skip,
        AbortOperation::Am,
        DiscardBaseline::OperationStart,
        &[b"a.txt".to_vec()],
        &[],
        &[],
    );
    assert!(text.contains("git am --skip goes on without the commit or patch it stopped at"), "{text}");
    assert!(!text.contains("restores the state before it"), "{text}");
    assert!(text.contains("git am --quit"), "the skip is stopped with the operation's own quit: {text}");
}

/// An untracked file at a path the restored tree tracks is Git's own refusal,
///  and the words say so and name the file.
#[test]
fn an_untracked_file_is_named_as_gits_own_refusal() {
    let text: String = untracked_file_text(
        AbortOperation::Merge,
        OperationEnd::Abort,
        AbortOperation::Merge,
        "HEAD",
        &[String::from("victim.txt")],
    );
    assert!(text.contains("Git itself refuses it here"), "{text}");
    assert!(text.contains("victim.txt"), "{text}");
    assert!(text.contains("Git will not write that tree while it is there"), "{text}");
    assert!(text.contains("mv <path> <somewhere else>"), "the way out: {text}");
    assert!(text.contains("git merge --abort"), "the command as typed: {text}");
    assert!(!text.contains("no-enforce-worktree"), "Git's own refusal is the answer here: {text}");
}

/// A skip's reset refuses that same file,
///  so the words name the skip as typed and the tree it would have written.
#[test]
fn a_skips_untracked_file_is_named_as_gits_refusal_too() {
    let text: String = untracked_file_text(
        AbortOperation::Rebase,
        OperationEnd::Skip,
        AbortOperation::Rebase,
        "HEAD",
        &[String::from("victim.txt")],
    );
    assert!(text.contains("git rebase --skip is refused because Git itself refuses it here"), "{text}");
    assert!(text.contains("writes the tree of HEAD"), "{text}");
    assert!(text.contains("victim.txt"), "{text}");
    assert!(text.contains("The rebase in progress still waits"), "{text}");
}

/// The ignored-file refusal keeps the shared sentence and says the operation still waits.
#[test]
fn the_ignored_file_refusal_names_the_waiting_operation() {
    let text: String = ignored_file_refusal_text(String::from("monochromatic-git-policy-cli: the .env is ignored"), AbortOperation::Rebase);
    assert!(text.starts_with("monochromatic-git-policy-cli: the .env is ignored"), "{text}");
    assert!(text.contains("The rebase in progress still waits, and nothing was changed."), "{text}");
}

/// A reading that failed says which command is refused, why, and how to run it as Git would.
#[test]
fn an_unreadable_reading_names_the_reason_and_the_opt_out() {
    let text: String = unreadable_refusal_text(
        AbortOperation::Rebase,
        OperationEnd::Abort,
        AbortOperation::Rebase,
        "Git records it in /r/.git/rebase-merge/orig-head, which is not a regular file",
    );
    assert!(text.contains("git rebase --abort is refused because the wrapper could not read what it would discard"), "{text}");
    assert!(text.contains("not a regular file"), "{text}");
    assert!(text.contains("git rebase --no-enforce-worktree --abort"), "{text}");
}
