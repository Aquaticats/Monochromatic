//! What:
//!  The abort and skip carve-out's verdicts:
//!  left to Git where nothing waits,
//!  allowed where the operation's own state is all there is,
//!  and refused where the person's own work would go,
//!  with each path and the kind of work standing there named.
//! Why:
//!  Each verdict is a promise about the person's work,
//!  and a refusal is the whole answer now that nothing is copied:
//!  a sentence that named no path,
//!  no way out or no opt-out would leave the person guessing.
//!  Nothing here touches a repository, because nothing is written any more.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(judgeAbortDiscard(facts, GuardedCommand.AbortMerge)).toEqual({ kind: 'allowed' });
//! ```

use super::{AbortVerdict, judge_abort_discard};
use crate::policy_test_support::{ScriptedFacts, scripted_facts};
use crate::repository_facts_abort_discard::{AbortDiscard, DiscardBaseline};
use crate::rule_abort_operation::{AbortOperation, OperationEnd};
use crate::rule_linked_worktree::GuardedCommand;
use std::ffi::OsString;

/// A waiting abort whose restored tree is `HEAD`,
///  with the person's own work at these paths,
///  read against the merge result Git recorded.
fn waiting(operation: AbortOperation, worktree: Vec<Vec<u8>>, index: Vec<Vec<u8>>, staged: Vec<Vec<u8>>) -> AbortDiscard {
    return AbortDiscard::Waiting {
        in_progress: operation,
        ends: OperationEnd::Abort,
        restored: OsString::from("HEAD"),
        baseline: DiscardBaseline::RecordedResult,
        worktree,
        index,
        staged,
    };
}

/// A waiting operation read the way a rebase, a pick or an `am` is read:
///  the worktree against the index and the index against `HEAD`,
///  because none of them records a result.
fn waiting_from_the_operation_start(operation: AbortOperation, ends: OperationEnd, worktree: Vec<Vec<u8>>, index: Vec<Vec<u8>>) -> AbortDiscard {
    return AbortDiscard::Waiting {
        in_progress: operation,
        ends,
        restored: OsString::from("HEAD"),
        baseline: DiscardBaseline::OperationStart,
        worktree,
        staged: index.clone(),
        index,
    };
}

/// A form that is not an abort or skip form is never this carve-out's business.
#[test]
fn every_other_guarded_form_is_not_applicable() {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(waiting(AbortOperation::Merge, vec![b"a.txt".to_vec()], Vec::new(), Vec::new()));
    for command in [
        GuardedCommand::Stash,
        GuardedCommand::Clean,
        GuardedCommand::ResetHard,
        GuardedCommand::RestoreWorktree,
        GuardedCommand::CheckoutPaths,
        GuardedCommand::ReadTree,
    ] {
        assert_eq!(judge_abort_discard(&mut facts, command), AbortVerdict::NotApplicable, "{command:?}");
    }
    assert!(!facts.asked.iter().any(|asked| { return asked.starts_with("abort-discard"); }), "{:?}", facts.asked);
}

/// No operation waiting means Git refuses the abort itself:
///  the command is left to Git.
#[test]
fn an_abort_with_nothing_waiting_is_left_to_git() {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(AbortDiscard::NothingWaiting);
    assert_eq!(judge_abort_discard(&mut facts, GuardedCommand::AbortMerge), AbortVerdict::LeftToGit);
}

/// An answer that cannot be read is a refusal that names the reason,
///  and never a quiet run and never a sentence about a dirty worktree.
#[test]
fn an_unreadable_answer_is_refused_naming_the_reason() {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Err(String::from("could not read the operation state: fatal: not a git repository"));
    match judge_abort_discard(&mut facts, GuardedCommand::AbortMerge) {
        AbortVerdict::Refused(text) => {
            assert!(text.contains("git merge --abort is refused because the wrapper could not read"), "{text}");
            assert!(text.contains("not a git repository"), "{text}");
            assert!(text.contains("git merge --no-enforce-worktree --abort"), "{text}");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// An operation whose own state is all there is runs exactly as Git runs it.
#[test]
fn the_operations_own_state_alone_runs_without_a_line() {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(waiting(AbortOperation::Merge, Vec::new(), Vec::new(), Vec::new()));
    assert_eq!(judge_abort_discard(&mut facts, GuardedCommand::AbortMerge), AbortVerdict::Allowed);
}

/// A resolution written by hand during a merge is refused,
///  and the refusal names the path, what stands there and the two ways out.
#[test]
fn a_hand_resolution_is_refused_and_named() {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(waiting(AbortOperation::Merge, vec![b"a.txt".to_vec()], Vec::new(), Vec::new()));
    match judge_abort_discard(&mut facts, GuardedCommand::AbortMerge) {
        AbortVerdict::Refused(text) => {
            assert!(text.contains("a.txt (a resolution or edit you made since Git stopped at the conflict)"), "{text}");
            assert!(text.contains("the merge in progress still waits"), "{text}");
            assert!(text.contains("Nothing was changed, and nothing was copied."), "{text}");
            assert!(text.contains("git merge --quit"), "the way out that keeps the work: {text}");
            assert!(text.contains("git merge --no-enforce-worktree --abort"), "the opt-out: {text}");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// Content that stands only in the index is refused as such:
///  a resolution staged while the worktree still holds the merge's own result.
#[test]
fn content_only_the_index_holds_is_named_that_way() {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(waiting(AbortOperation::Merge, Vec::new(), vec![b"new.txt".to_vec()], vec![b"new.txt".to_vec()]));
    match judge_abort_discard(&mut facts, GuardedCommand::AbortMerge) {
        AbortVerdict::Refused(text) => assert!(text.contains("new.txt (content only the index holds)"), "{text}"),
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// A staged resolution and a later edit in the worktree are two things a person made,
///  and the refusal names both of them at one path.
#[test]
fn a_staged_resolution_beside_a_later_worktree_edit_names_both() {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(waiting(AbortOperation::Merge, vec![b"a.txt".to_vec()], vec![b"a.txt".to_vec()], vec![b"a.txt".to_vec()]));
    match judge_abort_discard(&mut facts, GuardedCommand::AbortMerge) {
        AbortVerdict::Refused(text) => assert!(
            text.contains("a.txt (a resolution or edit you made since Git stopped at the conflict; a staged resolution)"),
            "{text}"
        ),
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// A rebase records no result, so an edit in the worktree is named as one since the operation began.
#[test]
fn an_edit_since_a_rebase_began_is_named_that_way() {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(waiting_from_the_operation_start(
        AbortOperation::Rebase,
        OperationEnd::Abort,
        vec![b"a.txt".to_vec()],
        Vec::new(),
    ));
    match judge_abort_discard(&mut facts, GuardedCommand::AbortRebase) {
        AbortVerdict::Refused(text) => {
            assert!(text.contains("a.txt (an edit you made since the operation began)"), "{text}");
            assert!(text.contains("git rebase --abort"), "{text}");
            assert!(text.contains("git rebase --quit"), "{text}");
            assert!(text.contains("git rebase --no-enforce-worktree --abort"), "{text}");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// An edit that was staged is content the index holds,
///  and the worktree holds it too, which the refusal may not deny.
#[test]
fn a_staged_edit_is_named_as_staged_content() {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(waiting_from_the_operation_start(
        AbortOperation::Am,
        OperationEnd::Abort,
        Vec::new(),
        vec![b"b.txt".to_vec()],
    ));
    match judge_abort_discard(&mut facts, GuardedCommand::AbortAm) {
        AbortVerdict::Refused(text) => {
            assert!(text.contains("b.txt (content the index holds)"), "{text}");
            assert!(!text.contains("only the index holds"), "the worktree holds that content too: {text}");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// A path whose index entry was erased holds no content,
///  and the refusal says the deletion is the work at risk.
#[test]
fn a_staged_deletion_is_named_as_a_deletion() {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(waiting(AbortOperation::Merge, Vec::new(), vec![b"gone.txt".to_vec()], Vec::new()));
    match judge_abort_discard(&mut facts, GuardedCommand::AbortMerge) {
        AbortVerdict::Refused(text) => assert!(text.contains("gone.txt (a deletion you staged)"), "{text}"),
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// A refusal names a bounded number of paths and counts the rest.
#[test]
fn every_path_beyond_the_bound_is_counted() {
    let bound: usize = crate::rule_worktree_dirty::SHARED_PATHS_NAMED;
    let mut paths: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for number in 0..bound + 2 {
        paths.push(format!("p{number}.txt").into_bytes());
    }
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(waiting(AbortOperation::Merge, paths, Vec::new(), Vec::new()));
    match judge_abort_discard(&mut facts, GuardedCommand::AbortMerge) {
        AbortVerdict::Refused(text) => {
            assert!(text.contains("p0.txt"), "the first paths are named: {text}");
            assert!(text.contains(&format!("and {} more paths", 2)), "the rest are counted: {text}");
            assert!(!text.contains(&format!("p{}.txt", bound + 1)), "no path past the bound is named: {text}");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// A skip names its own operation and its own quit form:
///  the skip goes on instead of restoring what stood before.
#[test]
fn a_skip_names_the_quit_form_of_its_own_operation() {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(waiting_from_the_operation_start(
        AbortOperation::Pick,
        OperationEnd::Skip,
        vec![b"a.txt".to_vec()],
        Vec::new(),
    ));
    match judge_abort_discard(&mut facts, GuardedCommand::SkipPick) {
        AbortVerdict::Refused(text) => {
            assert!(text.contains("git cherry-pick --skip or git revert --skip"), "{text}");
            assert!(text.contains("git cherry-pick --quit or git revert --quit"), "{text}");
            assert!(text.contains("git cherry-pick --no-enforce-worktree --skip"), "{text}");
            assert!(text.contains("goes on without the commit or patch it stopped at"), "{text}");
            assert!(!text.contains("restores the state before it"), "a skip restores nothing: {text}");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// An ignored file at a path the restored tree tracks is refused and named.
#[test]
fn an_ignored_file_the_restored_tree_tracks_is_refused_and_named() {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(waiting(AbortOperation::Merge, Vec::new(), Vec::new(), Vec::new()));
    facts.ignored_and_tracked = Ok(vec![String::from(".env")]);
    match judge_abort_discard(&mut facts, GuardedCommand::AbortMerge) {
        AbortVerdict::Refused(text) => {
            assert!(text.contains(".env"), "{text}");
            assert!(text.contains("The merge in progress still waits"), "{text}");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// An ignored-file answer that cannot be read keeps the refusal rather than writing a tree over an unknown file.
#[test]
fn an_unreadable_ignored_file_answer_is_refused() {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(waiting(AbortOperation::Am, Vec::new(), Vec::new(), Vec::new()));
    facts.ignored_and_tracked = Err(String::from("the tree of HEAD could not be read"));
    match judge_abort_discard(&mut facts, GuardedCommand::AbortAm) {
        AbortVerdict::Refused(text) => {
            assert!(text.contains("the tree of HEAD could not be read"), "{text}");
            assert!(text.contains("The am in progress still waits"), "{text}");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// The ignored-file question asks for the revision the abort actually restores,
///  so a rebase's own recorded head is read and no fixed name stands in for it.
#[test]
fn the_ignored_file_question_reads_the_restored_revision() {
    let record: &str = "0123456789abcdef0123456789abcdef01234567";
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(AbortDiscard::Waiting {
        in_progress: AbortOperation::Rebase,
        ends: OperationEnd::Abort,
        restored: OsString::from(record),
        baseline: DiscardBaseline::OperationStart,
        worktree: Vec::<Vec<u8>>::new(),
        index: Vec::<Vec<u8>>::new(),
        staged: Vec::<Vec<u8>>::new(),
    });
    facts.ignored_and_tracked = Ok(Vec::<String>::new());
    assert_eq!(judge_abort_discard(&mut facts, GuardedCommand::AbortRebase), AbortVerdict::Allowed);
    assert!(
        facts.asked.iter().any(|asked| { return *asked == format!("ignored-and-tracked:{record}"); }),
        "the record the abort restores is the tree asked about: {:?}",
        facts.asked
    );
}

/// An untracked file at a path the restored tree tracks is refused with Git's own reason for refusing,
///  and the file is named.
#[test]
fn an_untracked_file_the_restored_tree_tracks_is_named_as_gits_own_refusal() {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(waiting_from_the_operation_start(
        AbortOperation::Rebase,
        OperationEnd::Abort,
        Vec::new(),
        Vec::new(),
    ));
    facts.untracked_and_tracked = Ok(vec![String::from("victim.txt")]);
    match judge_abort_discard(&mut facts, GuardedCommand::AbortRebase) {
        AbortVerdict::Refused(text) => {
            assert!(text.contains("victim.txt"), "{text}");
            assert!(text.contains("Git itself refuses it here"), "the words say it is Git that refuses: {text}");
            assert!(text.contains("Git will not write that tree"), "{text}");
            assert!(!text.contains("nothing was copied"), "no copy was owed: {text}");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// A skip's reset refuses that same untracked file,
///  so the same words answer it and name the command as typed.
#[test]
fn a_skip_is_refused_for_the_untracked_file_its_reset_will_not_write_over() {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(waiting_from_the_operation_start(
        AbortOperation::Rebase,
        OperationEnd::Skip,
        Vec::new(),
        Vec::new(),
    ));
    facts.untracked_and_tracked = Ok(vec![String::from("victim.txt")]);
    match judge_abort_discard(&mut facts, GuardedCommand::SkipRebase) {
        AbortVerdict::Refused(text) => {
            assert!(text.contains("victim.txt"), "{text}");
            assert!(text.contains("git rebase --skip is refused because Git itself refuses it here"), "{text}");
            assert!(text.contains("Git will not write that tree"), "{text}");
            assert!(text.contains("The rebase in progress still waits"), "{text}");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// An `am` abort that ends a rebase names the rebase in its refusal,
///  while the command stays the one the person typed.
#[test]
fn an_am_abort_that_ends_a_rebase_names_the_rebase() {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(waiting(AbortOperation::Rebase, Vec::new(), Vec::new(), Vec::new()));
    facts.ignored_and_tracked = Ok(vec![String::from(".env")]);
    match judge_abort_discard(&mut facts, GuardedCommand::AbortAm) {
        AbortVerdict::Refused(text) => {
            assert!(text.contains("git am --abort"), "the command is named as typed: {text}");
            assert!(text.contains("rebase in progress still waits"), "the operation it ends is named: {text}");
            assert!(!text.contains("the am in progress"), "no sentence calls the rebase an am: {text}");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}
