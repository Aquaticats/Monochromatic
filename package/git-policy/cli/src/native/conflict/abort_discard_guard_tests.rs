//! What:
//!  The abort carve-out's verdicts:
//!  left to Git where nothing waits,
//!  allowed with the person's own work saved,
//!  and rejected when a copy cannot be written.
//! Why:
//!  Each verdict is a promise about the person's work,
//!  and the copies are cheap to check when the repository is a directory this control makes itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(judgeAbortDiscard(facts, GuardedCommand.AbortMerge, identity, 0, Limits)).toEqual({ kind: 'left-to-git' });
//! ```

use super::{AbortVerdict, judge_abort_discard};
use crate::conflict_sides::SAVED_DIRECTORY;
use crate::discarded_prune::Limits;
use crate::policy_test_support::{ScriptedFacts, scripted_facts};
use crate::repository_facts_abort_discard::AbortDiscard;
use crate::rule_abort_operation::AbortOperation;
use crate::rule_linked_worktree::GuardedCommand;
use crate::test_support::{fixture, remove};
use crate::worktree_identity::WorktreeIdentity;
use std::path::{Path, PathBuf};

/// One disposable repository directory with a linked worktree layout,
///  and the identity that names it.
struct Scene {
    root: PathBuf,
}

impl Scene {
    /// A fixture directory holding a `.git` directory and a worktree.
    fn new(name: &str) -> Scene {
        let root: PathBuf = fixture(name).join("repo");
        std::fs::create_dir_all(root.join(".git").join("worktrees/linked").as_path()).expect("git directory");
        std::fs::create_dir_all(root.as_path()).expect("worktree");
        return Scene { root };
    }

    /// The identity of a linked worktree of this repository.
    fn identity(&self) -> WorktreeIdentity {
        return WorktreeIdentity::LinkedWorktree {
            common_dir: self.root.join(".git"),
            git_dir: self.root.join(".git/worktrees/linked"),
            worktree_root: self.root.clone(),
        };
    }
}

impl Drop for Scene {
    fn drop(&mut self) {
        // The fixture root is `repo`'s parent; removing it removes the worktree and the Git directory together.
        if let Some(root) = self.root.parent() {
            remove(root);
        }
    }
}

/// The limits of the saved copies:
/// no test waits on the pass.
fn limits() -> Limits {
    return Limits::default();
}

/// A form that is not an abort form is never this carve-out's business.
#[test]
fn every_other_guarded_form_is_not_applicable() {
    let scene: Scene = Scene::new("not-an-abort");
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(AbortDiscard::Waiting { operation: AbortOperation::Merge, paths: vec![b"a.txt".to_vec()] });
    for command in [
        GuardedCommand::Stash,
        GuardedCommand::Clean,
        GuardedCommand::ResetHard,
        GuardedCommand::RestoreWorktree,
        GuardedCommand::CheckoutPaths,
        GuardedCommand::ReadTree,
    ] {
        assert_eq!(
            judge_abort_discard(&mut facts, command, &scene.identity(), 0, limits()),
            AbortVerdict::NotApplicable,
            "{command:?}"
        );
    }
    assert!(!facts.asked.iter().any(|asked| { return asked.starts_with("abort-discard"); }), "{:?}", facts.asked);
}

/// No operation waiting means Git refuses the abort itself:
/// the command is left to Git,
///  nothing is measured and nothing is saved.
#[test]
fn an_abort_with_nothing_waiting_is_left_to_git_without_a_copy() {
    let scene: Scene = Scene::new("nothing-waiting");
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(AbortDiscard::NothingWaiting);
    assert_eq!(
        judge_abort_discard(&mut facts, GuardedCommand::AbortMerge, &scene.identity(), 0, limits()),
        AbortVerdict::LeftToGit
    );
    assert!(!scene.root.join(".git").join(SAVED_DIRECTORY).exists(), "no copy directory");
}

/// An answer that cannot be read keeps the usual rejection,
///  which loses nothing.
#[test]
fn an_unreadable_answer_is_not_applicable() {
    let scene: Scene = Scene::new("unreadable");
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Err(String::from("Git could not read the operation state"));
    assert_eq!(
        judge_abort_discard(&mut facts, GuardedCommand::AbortMerge, &scene.identity(), 0, limits()),
        AbortVerdict::NotApplicable
    );
}

/// An operation whose own state is all there is runs with no copy and no notice.
#[test]
fn the_operations_own_state_alone_runs_without_a_copy() {
    let scene: Scene = Scene::new("own-state");
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(AbortDiscard::Waiting { operation: AbortOperation::Merge, paths: Vec::<Vec<u8>>::new() });
    assert_eq!(
        judge_abort_discard(&mut facts, GuardedCommand::AbortMerge, &scene.identity(), 0, limits()),
        AbortVerdict::Allowed { notice: String::new(), copies: Vec::new() }
    );
    assert!(!scene.root.join(".git").join(SAVED_DIRECTORY).exists(), "no copy directory");
}

/// A path the person wrote is copied under the common Git directory,
///  and the notice names the copy.
#[test]
fn the_persons_own_work_is_copied_and_named() {
    let scene: Scene = Scene::new("save");
    std::fs::write(scene.root.join("a.txt"), b"resolved by hand\n").expect("the file");
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(AbortDiscard::Waiting { operation: AbortOperation::Merge, paths: vec![b"a.txt".to_vec()] });
    match judge_abort_discard(&mut facts, GuardedCommand::AbortMerge, &scene.identity(), 7, limits()) {
        AbortVerdict::Allowed { notice, copies } => {
            assert_eq!(copies.len(), 1, "{copies:?}");
            assert_eq!(copies[0].path, "a.txt");
            let place: &Path = copies[0].saved_to.as_deref().expect("a copy");
            assert_eq!(std::fs::read(place).expect("the copy"), b"resolved by hand\n");
            assert!(notice.contains("a.txt"), "{notice}");
            assert!(notice.contains("git merge --abort"), "{notice}");
            assert!(notice.contains("copies are under"), "{notice}");
        }
        other => panic!("expected an allowed command, got {other:?}"),
    }
}

/// A path the worktree no longer holds is named as having nothing to copy,
///  and the command still runs.
#[test]
fn a_path_deleted_in_the_worktree_is_named_as_having_no_copy() {
    let scene: Scene = Scene::new("deleted");
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(AbortDiscard::Waiting { operation: AbortOperation::Rebase, paths: vec![b"gone.txt".to_vec()] });
    match judge_abort_discard(&mut facts, GuardedCommand::AbortRebase, &scene.identity(), 11, limits()) {
        AbortVerdict::Allowed { notice, copies } => {
            assert_eq!(copies.len(), 1, "{copies:?}");
            assert_eq!(copies[0].saved_to, None);
            assert!(notice.contains("gone.txt"), "{notice}");
            assert!(notice.contains("nothing to copy"), "{notice}");
            assert!(notice.contains("git rebase --abort"), "{notice}");
        }
        other => panic!("expected an allowed command, got {other:?}"),
    }
}

/// An ignored file at a path the restored tree tracks is refused before anything is saved,
///  and the refusal names the path and the operation that still waits.
#[test]
fn an_ignored_file_the_restored_tree_tracks_keeps_the_refusal() {
    let scene: Scene = Scene::new("ignored");
    std::fs::write(scene.root.join("a.txt"), b"resolved by hand\n").expect("the file");
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(AbortDiscard::Waiting { operation: AbortOperation::Merge, paths: vec![b"a.txt".to_vec()] });
    facts.ignored_and_tracked = Ok(vec![String::from(".env")]);
    match judge_abort_discard(&mut facts, GuardedCommand::AbortMerge, &scene.identity(), 0, limits()) {
        AbortVerdict::CopyFailed(text) => {
            assert!(text.contains(".env"), "{text}");
            assert!(text.contains("merge in progress still waits"), "{text}");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
    assert!(!scene.root.join(".git").join(SAVED_DIRECTORY).exists(), "nothing was saved");
}

/// An ignored-file answer that cannot be read keeps the refusal rather than writing a tree over an unknown file.
#[test]
fn an_unreadable_ignored_file_answer_keeps_the_refusal() {
    let scene: Scene = Scene::new("ignored-unreadable");
    let mut facts: ScriptedFacts = scripted_facts();
    facts.abort_discard = Ok(AbortDiscard::Waiting { operation: AbortOperation::Am, paths: Vec::<Vec<u8>>::new() });
    facts.ignored_and_tracked = Err(String::from("the tree of HEAD could not be read"));
    match judge_abort_discard(&mut facts, GuardedCommand::AbortAm, &scene.identity(), 0, limits()) {
        AbortVerdict::CopyFailed(text) => {
            assert!(text.contains("the tree of HEAD could not be read"), "{text}");
            assert!(text.contains("am in progress still waits"), "{text}");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}
