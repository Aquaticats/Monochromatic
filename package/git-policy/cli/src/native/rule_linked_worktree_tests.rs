//! What:
//!  Which commands the linked-worktree policy guards,
//!  and every rejection text.
//! Why:
//!  A harmless form read as destructive blocks an inspection;
//!  a destructive form read
//!      as harmless lets `git reset --hard` run in the primary checkout.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(decideLinkedWorktree(['clean', '-n'])).toEqual({ kind: 'pass' });
//! ```

/// The decision,
///  its texts and the argument builder.
use super::{
    GuardedCommand, LINKED_WORKTREE_REQUIRED_CODE, LinkedWorktreeDecision, decide_linked_worktree,
    dirty_scope, dirty_worktree_message, main_worktree_message, outside_worktree_message,
    resolve_linked_worktree,
};
use crate::command_test_support::os_arguments;
use crate::effective_target::EffectiveTarget;
use crate::rule_worktree_dirty::DirtyScope;

/// Every guarded form.
const ALL_COMMANDS: [GuardedCommand; 18] = [
    GuardedCommand::Stash,
    GuardedCommand::Clean,
    GuardedCommand::ResetHard,
    GuardedCommand::ResetMixed,
    GuardedCommand::RestoreStaged,
    GuardedCommand::RestoreWorktree,
    GuardedCommand::CheckoutPaths,
    GuardedCommand::CheckoutMove,
    GuardedCommand::SwitchForce,
    GuardedCommand::SwitchMove,
    GuardedCommand::Remove,
    GuardedCommand::Move,
    GuardedCommand::CheckoutIndex,
    GuardedCommand::ReadTree,
    GuardedCommand::AbortMerge,
    GuardedCommand::AbortRebase,
    GuardedCommand::AbortPick,
    GuardedCommand::AbortAm,
];

/// Decide one invocation written as text.
fn decide(values: &[&str]) -> LinkedWorktreeDecision {
    return decide_linked_worktree(os_arguments(values).as_slice());
}

/// Every stash form,
///  a deleting clean and a file-rewriting reset need the worktree kind.
#[test]
fn destructive_forms_need_the_worktree_kind() {
    assert_eq!(LINKED_WORKTREE_REQUIRED_CODE, "linked-worktree-required");
    for (values, command) in [
        (vec!["stash"], GuardedCommand::Stash),
        (vec!["stash", "drop"], GuardedCommand::Stash),
        (vec!["stash", "pop"], GuardedCommand::Stash),
        (vec!["stash", "clear"], GuardedCommand::Stash),
        (vec!["stash", "push", "-m", "note"], GuardedCommand::Stash),
        (vec!["stash", "--no-such-option"], GuardedCommand::Stash),
        (vec!["clean"], GuardedCommand::Clean),
        (vec!["clean", "-fd"], GuardedCommand::Clean),
        (vec!["clean", "-n", "--no-dry-run"], GuardedCommand::Clean),
        (vec!["clean", "-i"], GuardedCommand::Clean),
        (vec!["reset"], GuardedCommand::ResetMixed),
        (vec!["reset", "--mixed"], GuardedCommand::ResetMixed),
        (vec!["reset", "file"], GuardedCommand::ResetMixed),
        (vec!["reset", "--", "--hard"], GuardedCommand::ResetMixed),
        (vec!["reset", "--hard"], GuardedCommand::ResetHard),
        (vec!["reset", "--merge", "HEAD~1"], GuardedCommand::ResetHard),
        (vec!["reset", "--keep"], GuardedCommand::ResetHard),
        (vec!["reset", "--soft", "--hard"], GuardedCommand::ResetHard),
        (
            vec!["-C", "dir", "--no-pager", "reset", "--hard"],
            GuardedCommand::ResetHard,
        ),
        (vec!["restore", "a.txt"], GuardedCommand::RestoreWorktree),
        (vec!["restore", "--staged", "--", "."], GuardedCommand::RestoreStaged),
        (vec!["restore", "-S", "a.txt"], GuardedCommand::RestoreStaged),
        (vec!["restore", "--staged", "--worktree", "a.txt"], GuardedCommand::RestoreWorktree),
        (vec!["checkout", "--", "a.txt"], GuardedCommand::CheckoutPaths),
        (vec!["checkout", "-f"], GuardedCommand::CheckoutPaths),
        (vec!["switch", "--discard-changes", "topic"], GuardedCommand::SwitchForce),
        (vec!["switch", "-f", "topic"], GuardedCommand::SwitchForce),
        (vec!["rm", "-rf", "dir"], GuardedCommand::Remove),
        (vec!["mv", "--force", "a", "b"], GuardedCommand::Move),
        (vec!["checkout-index", "-f", "-a"], GuardedCommand::CheckoutIndex),
        (vec!["read-tree", "--reset", "HEAD"], GuardedCommand::ReadTree),
        (vec!["merge", "--abort"], GuardedCommand::AbortMerge),
        (vec!["rebase", "--abort"], GuardedCommand::AbortRebase),
        (vec!["cherry-pick", "--abort"], GuardedCommand::AbortPick),
        (vec!["revert", "--abort"], GuardedCommand::AbortPick),
        (vec!["am", "--abort"], GuardedCommand::AbortAm),
    ] {
        assert_eq!(
            decide(values.as_slice()),
            LinkedWorktreeDecision::NeedsEffectiveTarget(command),
            "{values:?}"
        );
    }
}

/// A named target moves `HEAD` and is rejected only while the worktree is dirty.
#[test]
fn head_moves_need_the_dirtiness() {
    for (values, command) in [
        (vec!["checkout", "topic"], GuardedCommand::CheckoutMove),
        (vec!["checkout", "-"], GuardedCommand::CheckoutMove),
        (vec!["switch", "topic"], GuardedCommand::SwitchMove),
        (vec!["switch", "--detach", "HEAD~1"], GuardedCommand::SwitchMove),
    ] {
        assert_eq!(
            decide(values.as_slice()),
            LinkedWorktreeDecision::WhenDirty(command),
            "{values:?}"
        );
    }
}

/// Only a deleting clean counts untracked files as uncommitted state.
#[test]
fn only_clean_counts_untracked_files() {
    for command in ALL_COMMANDS {
        let expected: DirtyScope = if command == GuardedCommand::Clean {
            DirtyScope::TrackedAndUntracked
        } else {
            DirtyScope::Tracked
        };
        assert_eq!(dirty_scope(command), expected, "{command:?}");
    }
}

/// Other commands,
///  dry runs,
///  index-only resets and regions Git refuses pass without a measurement.
#[test]
fn harmless_forms_pass() {
    for values in [
        vec![],
        vec!["--version"],
        vec!["status"],
        vec!["add", "stash"],
        vec!["stash", "list"],
        vec!["stash", "list", "--stat"],
        vec!["stash", "show"],
        vec!["stash", "show", "-p", "stash@{1}"],
        vec!["stash", "create"],
        vec!["--no-pager", "stash", "list"],
        vec!["clean", "-n"],
        vec!["clean", "--dry-run", "-fd"],
        vec!["clean", "-i", "-n"],
        vec!["reset", "--soft", "HEAD~1"],
        vec!["reset", "--hard", "--soft"],
        vec!["restore", "-h"],
        vec!["checkout"],
        vec!["checkout", "-b", "topic"],
        vec!["switch", "-c", "topic"],
        vec!["rm", "a.txt"],
        vec!["rm", "--", "-f"],
        vec!["mv", "a", "b"],
        vec!["merge", "topic"],
        vec!["rebase", "main"],
        vec!["read-tree", "HEAD"],
        vec!["checkout", "--no-such-option"],
        vec!["switch", "--no-such-option"],
        vec!["clean", "--no-such-option"],
        vec!["reset", "--hard", "--no-such-option"],
        vec!["--no-such-global", "reset", "--hard"],
    ] {
        assert_eq!(
            decide(values.as_slice()),
            LinkedWorktreeDecision::Pass,
            "{values:?}"
        );
    }
}

/// Only the main worktree and a place without a worktree are rejected,
///  each with its own text.
#[test]
fn only_main_and_absent_worktrees_are_rejected() {
    for command in ALL_COMMANDS {
        assert_eq!(
            resolve_linked_worktree(command, EffectiveTarget::MainWorktree),
            Some(main_worktree_message(command))
        );
        assert_eq!(
            resolve_linked_worktree(command, EffectiveTarget::OutsideWorktree),
            Some(outside_worktree_message(command))
        );
        assert_eq!(
            resolve_linked_worktree(command, EffectiveTarget::LinkedWorktree),
            None
        );
        assert_eq!(
            resolve_linked_worktree(command, EffectiveTarget::Allowlisted),
            None
        );
        assert_ne!(
            main_worktree_message(command),
            outside_worktree_message(command)
        );
    }
}

/// Every text names the form, what it destroys, a sanctioned alternative, and the deliberate way to discard.
#[test]
fn every_text_names_the_loss_an_alternative_and_the_hatch() {
    for command in ALL_COMMANDS {
        let main = main_worktree_message(command);
        let dirty = dirty_worktree_message(command);
        let outside = outside_worktree_message(command);
        assert!(main.starts_with("cli-git: git "), "{command:?}: {main}");
        assert!(dirty.starts_with("cli-git: git "), "{command:?}: {dirty}");
        assert!(outside.starts_with("cli-git: git "), "{command:?}: {outside}");
        assert!(main.contains("in the main git worktree"), "{command:?}");
        assert!(dirty.contains("has uncommitted"), "{command:?}");
        assert!(outside.contains("-C <linked-worktree-root>"), "{command:?}");
        for text in [&main, &dirty] {
            assert!(text.contains("--no-enforce-worktree"), "{command:?}: {text}");
            assert!(text.contains("git diff > backup.patch"), "{command:?}: {text}");
            assert!(text.contains(" (allowed)") || text.contains("git commit --message")
                || text.contains("git rm <destination>") || text.contains("git rebase --continue")
                || text.contains("--continue") || text.contains("git diff -- <path>"),
                "{command:?} names no alternative: {text}");
        }
        assert_ne!(main, dirty);
        assert_ne!(main, outside);
    }
}

/// Each intent gets the alternative that fits it.
#[test]
fn each_intent_gets_its_own_alternative() {
    let unstage = main_worktree_message(GuardedCommand::RestoreStaged);
    assert!(unstage.contains("git commit --message <message> -- <paths>"));
    assert!(unstage.contains("git diff --cached --name-only"));
    let revert = main_worktree_message(GuardedCommand::RestoreWorktree);
    assert!(revert.contains("git show HEAD:<path>"));
    assert!(revert.contains("git diff -- <path> > backup.patch"));
    assert!(main_worktree_message(GuardedCommand::ResetHard).contains("git reset --soft <commit>"));
    assert!(main_worktree_message(GuardedCommand::Stash).contains("git worktree add -b <branch> <path>"));
    assert!(main_worktree_message(GuardedCommand::Clean).contains("git clean -n"));
    assert!(dirty_worktree_message(GuardedCommand::Clean).contains("untracked files"));
    assert!(!dirty_worktree_message(GuardedCommand::RestoreWorktree).contains("untracked"));
    assert!(main_worktree_message(GuardedCommand::AbortMerge).contains("git merge --quit"));
    assert!(main_worktree_message(GuardedCommand::AbortRebase).contains("git rebase --quit"));
    assert!(main_worktree_message(GuardedCommand::AbortPick).contains("--quit"));
    assert!(main_worktree_message(GuardedCommand::AbortAm).contains("git am --quit"));
    assert!(main_worktree_message(GuardedCommand::CheckoutIndex).contains("--prefix=<directory>/"));
    assert!(main_worktree_message(GuardedCommand::Remove).contains("git rm --cached <path>"));
    assert!(dirty_worktree_message(GuardedCommand::SwitchMove).contains("git worktree add <path> <branch>"));
}

/// The README documents the alternative that each rejection text names, in the same words.
#[test]
fn the_readme_documents_what_the_texts_recommend() {
    let readme: &str = include_str!("../../README.md");
    assert!(readme.contains("### Discarding commands"));
    for (command, phrase) in [
        (GuardedCommand::Stash, "git worktree add -b <branch> <path>"),
        (GuardedCommand::Clean, "git clean -n"),
        (GuardedCommand::ResetHard, "git reset --soft <commit>"),
        (GuardedCommand::ResetMixed, "git diff --cached --name-only"),
        (GuardedCommand::RestoreStaged, "git commit --message <message> -- <paths>"),
        (GuardedCommand::RestoreWorktree, "git show HEAD:<path>"),
        (GuardedCommand::CheckoutPaths, "git show HEAD:<path>"),
        (GuardedCommand::CheckoutMove, "git worktree add <path> <branch>"),
        (GuardedCommand::SwitchForce, "git worktree add <path> <branch>"),
        (GuardedCommand::SwitchMove, "git worktree add <path> <branch>"),
        (GuardedCommand::Remove, "git rm --cached <path>"),
        (GuardedCommand::Move, "git rm <destination>"),
        (GuardedCommand::CheckoutIndex, "git checkout-index --prefix=<directory>/ --all"),
        (GuardedCommand::ReadTree, "git read-tree -m"),
        (GuardedCommand::AbortMerge, "--quit"),
        (GuardedCommand::AbortRebase, "--quit"),
        (GuardedCommand::AbortPick, "--quit"),
        (GuardedCommand::AbortAm, "--quit"),
    ] {
        assert!(
            main_worktree_message(command).contains(phrase),
            "{command:?} text lacks {phrase}"
        );
        assert!(readme.contains(phrase), "{command:?}: README lacks {phrase}");
    }
    assert!(readme.contains("git diff > backup.patch"));
    assert!(readme.contains("--no-enforce-worktree"));
}
