//! What:
//!  The name a forced `checkout`, `switch`, `reset` or `restore` writes the tree of,
//!  and what the reading answers when Git cannot list that tree.
//! Why:
//!  `git ls-tree topic` fails for a branch that exists only as `refs/remotes/origin/topic`,
//!  while `git checkout -f topic` creates the local branch from it and writes that tree
//!  (measured on this host with native Git 2.56.0,
//!  2026-11-13);
//!  a reading that took the failed listing for "this form writes nothing" let the forced form
//!  overwrite an untracked file it never looked at,
//!  which is the loss these controls keep closed.
//!  The other half is the opposite mistake:
//!  a tree Git knows and cannot list is a refusal,
//!  never an empty answer,
//!  because what such a command writes is unknown rather than nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // pathsTheTreeWrites(facts, { kind: 'tree', revision: 'topic', guesses: true }) === ['b.txt']
//! // pathsTheTreeWrites(facts, { kind: 'tree', revision: 'topic', guesses: false }) === []
//! ```

use super::paths_the_tree_writes;
use crate::command_test_support::{git, repository_with_tracked_file};
use crate::command_written_paths::{WrittenEntries, WrittenPaths};
use crate::repo_read::repository_facts::{GitFacts, git_facts};
use crate::test_support::{REAL_GIT, isolated_overlay, remove};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// A provider for real Git after `-C <repository>`.
fn facts_in(repository: &Path) -> GitFacts {
    let prefix: Vec<OsString> = vec![OsString::from("-C"), repository.as_os_str().to_os_string()];
    return git_facts(Path::new(REAL_GIT), prefix.as_slice(), isolated_overlay().as_slice());
}

/// The written-path reading of a whole tree the form names,
/// with the guess of a remote branch set the way a `checkout` or a `reset` sets it.
fn tree_of(repository: &Path, revision: &str, guesses: bool) -> Result<Vec<Vec<u8>>, String> {
    let mut facts: GitFacts = facts_in(repository);
    return paths_the_tree_writes(
        &mut facts,
        &WrittenPaths::Tree {
            revision: Some(OsString::from(revision)),
            entries: WrittenEntries::EveryEntry,
            guesses_a_remote_branch: guesses,
        },
    );
}

/// What:
///  A repository whose `topic` commit holds `b.txt` and whose only ref to it is a remote-tracking one,
/// with a linked worktree on `main`.
/// Why:
///  That is the state a forced `checkout topic` guesses the local branch from,
///  and the state `git ls-tree topic` cannot read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = repositoryWithOnlyARemoteTrackingBranch('remote-only-branch', 'origin');
/// ```
fn repository_with_only_a_remote_tracking_branch(name: &str, remote: &str) -> (PathBuf, PathBuf) {
    let (directory, repository): (PathBuf, PathBuf) = repository_with_tracked_file(name);
    git(repository.as_path(), &["checkout", "--quiet", "-b", "topic"]);
    std::fs::write(repository.join("b.txt"), b"from the topic tree\n").expect("the topic's file");
    git(repository.as_path(), &["add", "--", "b.txt"]);
    git(repository.as_path(), &["commit", "--quiet", "--message=topic", "--", "b.txt"]);
    let topic: String = String::from_utf8_lossy(&git(repository.as_path(), &["rev-parse", "topic"]).stdout).trim().to_owned();
    git(repository.as_path(), &["checkout", "--quiet", "main"]);
    git(repository.as_path(), &["branch", "--delete", "--force", "topic"]);
    let reference: String = format!("refs/remotes/{remote}/topic");
    git(repository.as_path(), &["update-ref", reference.as_str(), topic.as_str()]);
    // Git guesses only from the remote-tracking branch of a configured remote: with no
    //  `remote.<name>` at all it refuses the short name
    //  (`error: pathspec 'topic' did not match any file(s) known to git`, measured on this host
    //  with native Git 2.56.0, 2026-11-13).
    git(repository.as_path(), &["remote", "add", remote, "/nonexistent-remote"]);
    return (directory, repository);
}

/// The paths of the `topic` commit's tree:
/// the file it adds and the one it inherited from `main`.
fn the_topics_tree() -> Vec<Vec<u8>> {
    return vec![b"b.txt".to_vec(), b"tracked.txt".to_vec()];
}

/// A forced `checkout` or `switch` writes the tree of the branch it creates from a remote-tracking
/// branch of that name,
/// so the reading names that tree's paths.
///  `git ls-tree topic` itself exits nonzero in this state,
///  which is the control that this test needs the guess.
#[test]
fn a_remote_tracking_branch_of_the_name_is_the_tree_a_checkout_writes() {
    let (directory, repository): (PathBuf, PathBuf) = repository_with_only_a_remote_tracking_branch("written-remote-only", "origin");
    // The control: the listing Git does without the guess fails, so an empty answer would be wrong.
    let direct = crate::command_test_support::git_status(repository.as_path(), &["ls-tree", "-r", "--name-only", "topic"]);
    assert!(!direct.status.success(), "the control needs `git ls-tree topic` to fail here: {direct:?}");
    assert_eq!(tree_of(repository.as_path(), "topic", true), Ok(the_topics_tree()));
    // A reset names a revision and never the branch a short name stands for.
    assert_eq!(tree_of(repository.as_path(), "topic", false), Ok(Vec::<Vec<u8>>::new()));
    // A name Git resolves without any guess is read with and without it.
    assert_eq!(tree_of(repository.as_path(), "origin/topic", true), Ok(the_topics_tree()));
    assert_eq!(tree_of(repository.as_path(), "origin/topic", false), Ok(the_topics_tree()));
    remove(directory.as_path());
}

/// A remote's own name may hold a slash,
/// and Git guesses the branch from the ref that begins with that name.
#[test]
fn a_remote_whose_name_holds_a_slash_is_read_whole() {
    let (directory, repository): (PathBuf, PathBuf) = repository_with_only_a_remote_tracking_branch("written-slashed-remote", "a/b");
    assert_eq!(tree_of(repository.as_path(), "topic", true), Ok(the_topics_tree()));
    remove(directory.as_path());
}

/// Two remotes carrying the branch are an ambiguity Git refuses,
/// and `checkout.defaultRemote` names the one Git then writes.
#[test]
fn two_remotes_are_ambiguous_unless_the_default_remote_names_one() {
    let (directory, repository): (PathBuf, PathBuf) = repository_with_only_a_remote_tracking_branch("written-two-remotes", "origin");
    let topic: String = String::from_utf8_lossy(&git(repository.as_path(), &["rev-parse", "refs/remotes/origin/topic"]).stdout).trim().to_owned();
    git(repository.as_path(), &["update-ref", "refs/remotes/second/topic", topic.as_str()]);
    git(repository.as_path(), &["remote", "add", "second", "/nonexistent-second"]);
    // The control: real Git refuses the ambiguous name and writes nothing.
    let direct = crate::command_test_support::git_status(repository.as_path(), &["ls-tree", "-r", "--name-only", "topic"]);
    assert!(!direct.status.success());
    assert_eq!(tree_of(repository.as_path(), "topic", true), Ok(Vec::<Vec<u8>>::new()));
    git(repository.as_path(), &["config", "checkout.defaultRemote", "second"]);
    assert_eq!(tree_of(repository.as_path(), "topic", true), Ok(the_topics_tree()));
    remove(directory.as_path());
}

/// `checkout.guess=false` makes Git refuse the short name,
/// so the reading resolves nothing and leaves the command to Git.
#[test]
fn a_repository_that_turns_the_guess_off_resolves_nothing() {
    let (directory, repository): (PathBuf, PathBuf) = repository_with_only_a_remote_tracking_branch("written-guess-off", "origin");
    git(repository.as_path(), &["config", "checkout.guess", "false"]);
    let direct = crate::command_test_support::git_status(repository.as_path(), &["ls-tree", "-r", "--name-only", "topic"]);
    assert!(!direct.status.success());
    assert_eq!(tree_of(repository.as_path(), "topic", true), Ok(Vec::<Vec<u8>>::new()));
    remove(directory.as_path());
}

/// A name Git knows nothing of writes no path:
/// Git refuses the form itself,
/// and the guard must not refuse a command Git would not have run.
#[test]
fn a_name_git_does_not_know_writes_no_path() {
    let (directory, repository): (PathBuf, PathBuf) = repository_with_only_a_remote_tracking_branch("written-unknown-name", "origin");
    assert_eq!(tree_of(repository.as_path(), "nosuchbranch", true), Ok(Vec::<Vec<u8>>::new()));
    remove(directory.as_path());
}

/// What:
///  A tree Git knows and cannot list is a failure of the reading,
///  never "this form writes nothing".
/// Why:
///  The two answers mean opposite things for a guard:
///  a name Git does not know is a form Git refuses itself,
///  while an object it knows and cannot read says nothing about what the command would write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // remove the tree object of `other`, then pathsTheTreeWrites(facts, 'other') throws
/// ```
#[test]
fn a_tree_that_cannot_be_listed_is_a_failure() {
    let (directory, repository): (PathBuf, PathBuf) = repository_with_only_a_remote_tracking_branch("written-unreadable-tree", "origin");
    let tree: String = String::from_utf8_lossy(&git(repository.as_path(), &["rev-parse", "refs/remotes/origin/topic^{tree}"]).stdout).trim().to_owned();
    let object: PathBuf = repository
        .join(".git")
        .join("objects")
        .join(&tree[..2])
        .join(&tree[2..]);
    std::fs::remove_file(object.as_path()).expect("the tree object");
    let answer: Result<Vec<Vec<u8>>, String> = tree_of(repository.as_path(), "origin/topic", false);
    match answer {
        Err(cause) => assert!(cause.contains("could not read the tree `origin/topic` names"), "{cause}"),
        Ok(found) => panic!("an unreadable tree was read as {found:?}"),
    }
    remove(directory.as_path());
}
