//! What:
//!  Which `git worktree` invocations create or move a worktree,
//!  with a real-Git control.
//! Why:
//!  The refusal of ignored-state synchronization that is not ported yet hangs on this
//!      one fact;
//!  reading `add` where Git reads something else would forward a creation.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(changesWorktreeRegistrations(['add', '../topic'])).toBe(true);
//! ```

/// The fact under test and the real-Git fixture helpers.
use super::{WorktreeRemoval, changes_worktree_registrations, worktree_removal};
use crate::command_test_support::os_arguments;
/// The owned operating-system text a removal's directory is held in.
use std::ffi::OsString;
use crate::test_support::{fixture, git, git_output, remove, repository};
use std::path::PathBuf;
use std::process::Output;

/// Only `add` and `move` directly after `worktree` change registrations.
#[test]
fn only_the_first_word_selects_the_action() {
    for values in [
        vec!["add"],
        vec!["add", "../topic"],
        vec!["add", "-b", "topic", "../topic"],
        vec!["move", "a", "b"],
        vec!["move", "a", "b", "--force"],
        vec!["move", "--force", "a", "b"],
        vec!["add", "../topic", "--quiet"],
        vec!["add", "--", "list"],
    ] {
        assert!(
            changes_worktree_registrations(os_arguments(values.as_slice()).as_slice()),
            "{values:?}"
        );
    }
    for values in [
        vec![],
        vec!["list"],
        vec!["list", "add"],
        vec!["remove", "move"],
        vec!["prune"],
        vec!["lock", "add"],
        vec!["repair"],
        vec!["ad"],
        vec!["added"],
        vec!["--add"],
        vec!["-f", "add"],
        vec!["ADD"],
        vec![""],
    ] {
        assert!(
            !changes_worktree_registrations(os_arguments(values.as_slice()).as_slice()),
            "{values:?}"
        );
    }
}

/// Count the worktrees real Git has registered for a repository.
fn registered(repo: &std::path::Path) -> usize {
    let listing: Output = git(repo, &["worktree", "list", "--porcelain"]);
    let mut count: usize = 0;
    // Each registered worktree starts one porcelain record with a `worktree <path>` line.
    for line in String::from_utf8_lossy(&listing.stdout).lines() {
        if line.starts_with("worktree ") {
            count += 1;
        }
    }
    return count;
}

/// Real Git 2.56.0 takes the word after `worktree` as the action,
///  never an abbreviation or a later word.
#[test]
fn real_git_reads_the_word_after_worktree() {
    let root: PathBuf = fixture("worktree-word");
    let repo: PathBuf = repository(root.as_path(), "repo");
    assert_eq!(registered(repo.as_path()), 1);
    git(repo.as_path(), &["worktree", "add", "--quiet", "../one"]);
    assert_eq!(registered(repo.as_path()), 2);
    git(repo.as_path(), &["worktree", "move", "../one", "../moved"]);
    assert!(root.join("moved").is_dir());
    assert!(!root.join("one").exists());
    // An abbreviated word, an option before the word and a later `add` create nothing.
    for arguments in [
        vec!["worktree", "ad", "../two"],
        vec!["worktree", "--force", "add", "../two"],
        vec!["worktree", "list", "add", "../two"],
    ] {
        let refused: Output = git_output(repo.as_path(), arguments.as_slice());
        assert!(!refused.status.success(), "{arguments:?}");
        assert_eq!(registered(repo.as_path()), 2, "{arguments:?}");
        assert!(!root.join("two").exists(), "{arguments:?}");
    }
    remove(root.as_path());
}

/// What:
///  An option written after the directory is still the option the audit reads,
///  and the directory it names is the same one.
/// Why:
///   Git's `parse_options` reads the whole argument list,
///  so `git worktree remove <path> --force` is the forced removal of that path, spelled the other way round;
///  the record the audit writes must say that, and the reading of the same command by the guard did not
///  (a review of `d351b65746`, finding 1).
#[test]
fn an_option_after_the_directory_is_the_same_removal() {
    for values in [vec!["remove", "topic", "--force"], vec!["remove", "--force", "topic"], vec!["remove", "topic", "-f"]] {
        assert_eq!(
            worktree_removal(os_arguments(values.as_slice()).as_slice()),
            Some(WorktreeRemoval { target: OsString::from("topic"), force: true }),
            "{values:?}"
        );
    }
    assert_eq!(
        worktree_removal(os_arguments(&["remove", "topic", "--no-force"]).as_slice()),
        Some(WorktreeRemoval { target: OsString::from("topic"), force: false }),
        "and the negated form carries no force"
    );
    assert_eq!(
        worktree_removal(os_arguments(&["remove", "--", "topic", "--force"]).as_slice()),
        Some(WorktreeRemoval { target: OsString::from("topic"), force: false }),
        "a word after `--` is a directory, so the option there is never read as one"
    );
}
