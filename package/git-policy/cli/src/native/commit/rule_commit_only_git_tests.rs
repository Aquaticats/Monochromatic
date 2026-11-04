//! What:
//!  Real Git 2.56.0 controls for the Git behavior the commit-only decision relies on.
//! Why:
//!  The decision inserts `-o`,
//!  refuses a pathless amend over a dirty index and lets a
//!      pathless commit conclude a merge.
//!  Each rests on what Git does with the rewritten
//!      command line,
//!  so the rewritten arguments are run in a disposable repository.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await git(repo, await commitOnly(['commit', '-m', 'x', '--', 'a.txt']));
//! ```

/// The decision,
///  the fact mappings and the real-Git fixture helpers.
use crate::command_test_support::{
    git, git_output, git_status, os_arguments, output_text, remove, repository_with_tracked_file,
    start_conflicted_merge,
};
use crate::rule_commit_index::{IndexVsHead, index_query_arguments, index_state_from_exit};
use crate::rule_commit_only::{
    CommitOnlyDecision, PendingInjection, decide_commit_only, resolve_index_state,
    resolve_sequencer_state,
};
use crate::rule_commit_sequencer::SequencerState;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Output;

/// Decide and return the rewritten argument list.
fn rewritten(values: &[&str]) -> Vec<OsString> {
    let decision: CommitOnlyDecision =
        decide_commit_only(os_arguments(values).as_slice(), &[]).expect("Git accepts");
    if let CommitOnlyDecision::Rewritten(arguments) = decision {
        return arguments;
    }
    panic!("expected a rewrite of {values:?}");
}

/// Names of the paths staged against `HEAD`.
fn staged_paths(root: &Path) -> String {
    return output_text(&git(root, &["diff", "--cached", "--name-only"]));
}

/// The injected `-o` commits exactly the named path and leaves other staged paths staged.
#[test]
fn injected_only_commits_just_the_named_path() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("only-named");
    std::fs::write(root.join("a.txt"), b"a\n").expect("write a");
    std::fs::write(root.join("b.txt"), b"b\n").expect("write b");
    git(root.as_path(), &["add", "--", "a.txt", "b.txt"]);
    let arguments: Vec<OsString> = rewritten(&["commit", "--quiet", "-m", "only a", "--", "a.txt"]);
    assert_eq!(arguments[1], "-o");
    let committed: Output = git_output(root.as_path(), arguments.as_slice());
    assert!(committed.status.success());
    let changed: Output = git(
        root.as_path(),
        &["show", "--name-only", "--format=", "HEAD"],
    );
    assert_eq!(output_text(&changed), "a.txt");
    assert_eq!(staged_paths(root.as_path()), "b.txt");
    remove(directory.as_path());
}

/// A pathless `-o --amend` reuses `HEAD`'s tree:
///  the staged change stays staged without any
/// warning.
///  This is why the decision refuses the injection while the index differs.
#[test]
fn a_pathless_only_amend_ignores_the_staged_change_in_git() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("only-amend");
    std::fs::write(root.join("tracked.txt"), b"two\n").expect("modify tracked file");
    git(root.as_path(), &["add", "--", "tracked.txt"]);
    let state: IndexVsHead = index_state_from_exit(
        git_output(root.as_path(), index_query_arguments(&[]).as_slice())
            .status
            .code(),
    );
    assert_eq!(state, IndexVsHead::Differs);
    let arguments: Vec<OsString> = os_arguments(&["commit", "--amend", "--no-edit"]);
    let first: CommitOnlyDecision =
        decide_commit_only(arguments.as_slice(), &[]).expect("Git accepts");
    let pending: &PendingInjection = if let CommitOnlyDecision::NeedsIndexState(found) = &first {
        found
    } else {
        panic!("expected the index question, got {first:?}");
    };
    let refused: CommitOnlyDecision = resolve_index_state(arguments.as_slice(), pending, state);
    assert!(matches!(refused, CommitOnlyDecision::Rejected(_)));
    // What Git would have done with the injection: succeed and leave the change staged.
    let injected: CommitOnlyDecision =
        resolve_index_state(arguments.as_slice(), pending, IndexVsHead::Matches);
    assert_eq!(
        injected,
        CommitOnlyDecision::Rewritten(os_arguments(&["commit", "-o", "--amend", "--no-edit"]))
    );
    git(
        root.as_path(),
        &["commit", "--quiet", "-o", "--amend", "--no-edit"],
    );
    assert_eq!(staged_paths(root.as_path()), "tracked.txt");
    remove(directory.as_path());
}

/// During a merge Git forbids a partial commit,
///  so the injected form fails and the
/// pathless form concludes the merge.
///  This is why the decision passes it through.
#[test]
fn a_pathless_commit_concludes_a_merge_in_git() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("only-merge");
    start_conflicted_merge(root.as_path());
    std::fs::write(root.join("tracked.txt"), b"resolved\n").expect("resolve the conflict");
    git(root.as_path(), &["add", "--", "tracked.txt"]);
    let partial: Output = git_status(
        root.as_path(),
        &["commit", "-o", "-m", "partial", "--", "tracked.txt"],
    );
    assert!(!partial.status.success());
    assert!(
        String::from_utf8_lossy(&partial.stderr)
            .contains("cannot do a partial commit during a merge"),
        "{}",
        String::from_utf8_lossy(&partial.stderr)
    );
    let pathless: Vec<OsString> = os_arguments(&["commit", "--quiet", "-m", "conclude"]);
    assert_eq!(
        decide_commit_only(pathless.as_slice(), &[]),
        Ok(CommitOnlyDecision::NeedsSequencerState)
    );
    assert_eq!(
        resolve_sequencer_state(SequencerState::InProgress),
        CommitOnlyDecision::Unchanged
    );
    assert!(
        git_output(root.as_path(), pathless.as_slice())
            .status
            .success()
    );
    assert!(!root.join(".git").join("MERGE_HEAD").exists());
    remove(directory.as_path());
}
