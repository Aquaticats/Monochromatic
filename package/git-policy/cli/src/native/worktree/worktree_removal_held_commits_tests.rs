//! Controls for keeping the commit a reference lock holds in a linked worktree,
//! before a command that may remove that worktree:
//! the commit reaches a reference of the common Git directory,
//! the branch does not move,
//! the person is told,
//! and real Git's own maintenance keeps the commit after the removal.

use super::keep_held_commits;
use crate::held_commit_record::{Held, write};
use crate::test_support::{fixture, git, git_text, remove};
use crate::transaction_git::GitContext;
use std::path::{Path, PathBuf};

/// The branch the linked worktree has checked out.
const BRANCH: &str = "feature";

/// The file the held commit changes.
const EDITED: &str = "edited.txt";

/// The subject of the held commit.
const SUBJECT: &str = "the held edit";

/// The batch number of the held commit.
const SEQ: u64 = 7;

/// A repository,
/// a linked worktree of its own branch with the file the held commit changed,
/// and the commit the branch does not name:
/// the state a commit that met a reference lock leaves in that worktree.
fn a_linked_worktree_with_a_held_commit(label: &str) -> (PathBuf, PathBuf, PathBuf, String) {
    let root: PathBuf = fixture(label);
    let repo: PathBuf = root.join("repo");
    let linked: PathBuf = root.join("linked");
    std::fs::create_dir(repo.as_path()).expect("the repository directory");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(repo.join("base.txt"), b"base\n").expect("the base file");
    git(repo.as_path(), &["add", "--", "base.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let linked_text: String = linked.display().to_string();
    git(repo.as_path(), &["worktree", "add", "--quiet", linked_text.as_str(), "-b", BRANCH]);
    std::fs::write(linked.join(EDITED), b"the edit\n").expect("the edit");
    git(linked.as_path(), &["add", "--", EDITED]);
    git(linked.as_path(), &["commit", "--quiet", "--message", SUBJECT]);
    let held_commit: String = git_text(linked.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    // The reference lock refused the branch write, so the branch still names the commit before it, and no reflog names the
    //  commit either: it was never on the branch, so the reference of the worktree is the only thing holding it.
    git(linked.as_path(), &["reset", "--hard", "--quiet", "HEAD~1"]);
    git(repo.as_path(), &["reflog", "expire", "--expire=now", "--all"]);
    git(linked.as_path(), &["update-ref", "--no-deref", format!("refs/worktree/cli-git-service/{SEQ}").as_str(), held_commit.as_str()]);
    let git_dir: PathBuf = repo.join(".git");
    let worktree_git_dir: PathBuf = git_dir.join("worktrees/linked");
    let held: Held = Held {
        seq: SEQ,
        journal: root.join("journal"),
        repository: linked.clone(),
        common_dir: git_dir.clone(),
        git_dir: worktree_git_dir.clone(),
        reference: format!("refs/heads/{BRANCH}"),
        old: git_text(linked.as_path(), &["rev-parse", "HEAD"]).trim().to_string(),
        new: held_commit.clone(),
        commits: vec![held_commit.clone()],
        index: worktree_git_dir.join("index"),
        reflog: b"commit: the held edit".to_vec(),
        locks: vec![git_dir.join(format!("refs/heads/{BRANCH}.lock"))],
        created: String::from("2026-01-01T00:00:00+00:00"),
    };
    write(worktree_git_dir.as_path(), &held).expect("the record of the hold");
    return (root, repo, linked, held_commit);
}

/// The Git the checks of this crate run real Git through.
fn context(git_dir: &Path) -> GitContext {
    return GitContext { real_git: PathBuf::from("/usr/bin/git"), overlay: Vec::new(), global_prefix: Vec::new(), common_dir: Some(git_dir.to_path_buf()) };
}

/// What:
///  The commit a reference lock holds in a linked worktree is kept on a reference of the common Git directory,
///  the person is told the branch does not move,
///  and the commit survives the two things the removal deletes:
///  the worktree's own Git directory and its per-worktree reference.
/// Why:
///  The record of the hold lives in that Git directory and the commit is held by that reference,
///  so a removal takes both,
///  no command settles that batch afterwards,
///  and real Git's maintenance then removes a commit the wrapper said it kept
///  (a review of `aa7f601c6d`,
///  finding 2).
#[test]
fn a_held_commit_of_a_removed_linked_worktree_survives_on_the_kept_reference() {
    let (root, repo, linked, held_commit) = a_linked_worktree_with_a_held_commit("removal-held-commits");
    let git_dir: PathBuf = repo.join(".git");
    // The state the wrapper's own hold leaves: the worktree's reference is the only one that names the commit, and
    //  per-worktree references are listed only from the worktree they belong to.
    assert_eq!(
        git_text(linked.as_path(), &["for-each-ref", "--contains", held_commit.as_str(), "--format=%(refname)"]).trim(),
        format!("refs/worktree/cli-git-service/{SEQ}"),
        "nothing but the worktree's own reference names the held commit"
    );
    let said: Vec<String> = keep_held_commits(git_dir.as_path(), &context(git_dir.as_path()));
    assert_eq!(said.len(), 1, "the person is told about the one held commit: {said:?}");
    let kept: String = format!("refs/cli-git/kept/{SEQ}");
    assert!(said[0].contains(kept.as_str()), "the line names where the commit is kept: {}", said[0]);
    assert!(said[0].contains(BRANCH), "the line names the branch: {}", said[0]);
    assert!(said[0].contains("stays where it is"), "the line says the branch does not move: {}", said[0]);
    assert_eq!(git_text(repo.as_path(), &["rev-parse", kept.as_str()]).trim(), held_commit, "the kept reference names the held commit");
    // A second command keeps nothing more: the batch's own commit is already there.
    assert_eq!(keep_held_commits(git_dir.as_path(), &context(git_dir.as_path())), Vec::<String>::new(), "nothing is written twice");
    // Git removes the worktree, with the record of the hold and its per-worktree reference inside it, and its own maintenance
    //  then removes what no reference names.
    let linked_text: String = linked.display().to_string();
    git(repo.as_path(), &["worktree", "remove", "--force", linked_text.as_str()]);
    assert!(!git_dir.join("worktrees/linked").exists(), "the removal took the worktree's own Git directory");
    assert_eq!(
        git_text(repo.as_path(), &["for-each-ref", "--contains", held_commit.as_str(), "--format=%(refname)"]).trim(),
        format!("refs/cli-git/kept/{SEQ}"),
        "the commit is held by the reference the removal left alone"
    );
    git(repo.as_path(), &["gc", "--prune=now"]);
    assert_eq!(git_text(repo.as_path(), &["cat-file", "-t", held_commit.as_str()]), "commit", "the commit is not lost with the worktree");
    let branch_reference: String = format!("refs/heads/{BRANCH}");
    assert_ne!(git_text(repo.as_path(), &["rev-parse", branch_reference.as_str()]).trim(), held_commit, "the branch still names the commit before the held one, as a lock keeps it");
    remove(root.as_path());
}

/// What:
///  A kept reference that already names another commit is left where it is,
///  and the person is told so.
/// Why:
///  The kept references are where commits that did not land are kept,
///  so replacing one would forget the commit it holds,
///  which is the owner's rule "never discard user data" again.
#[test]
fn a_kept_reference_that_names_another_commit_is_left_and_said() {
    let (root, repo, _linked, held_commit) = a_linked_worktree_with_a_held_commit("removal-held-commits-clash");
    let git_dir: PathBuf = repo.join(".git");
    let elsewhere: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    let kept: String = format!("refs/cli-git/kept/{SEQ}");
    git(repo.as_path(), &["update-ref", kept.as_str(), elsewhere.as_str()]);
    let said: Vec<String> = keep_held_commits(git_dir.as_path(), &context(git_dir.as_path()));
    assert_eq!(said.len(), 1, "the person is told nothing was written: {said:?}");
    assert!(said[0].contains("already names"), "the line says what stands in the way: {}", said[0]);
    assert_eq!(git_text(repo.as_path(), &["rev-parse", kept.as_str()]).trim(), elsewhere, "the commit an earlier batch was kept under stays named");
    assert_ne!(elsewhere, held_commit, "the control keeps two different commits");
    remove(root.as_path());
}
