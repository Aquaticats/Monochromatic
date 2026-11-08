//! Controls for the stat-data refresh an index gets before the scratch commit:
//! a stale entry is refreshed in the file the refresh names,
//! an unstaged change does not lose the refresh of the other entries,
//! an index another program has locked is left as it was,
//! and a Git that cannot start is answered without a failure.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, remove, renew_inode, repository};
use crate::transaction_git::GitContext;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

/// A repository with `a.txt` and `b.txt` committed,
/// and `a.txt` with a new inode while its bytes and its modification time stay as they were:
/// what the move to another disk left every worktree.
fn stale_repository(root: &Path) -> PathBuf {
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    std::fs::write(repo.join("b.txt"), "b\n").expect("b");
    git(repo.as_path(), &["add", "a.txt", "b.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=files"]);
    renew_inode(repo.join("a.txt").as_path());
    return repo;
}

/// Whether the real index records the inode `name` has now,
/// as Git prints it in `ls-files --debug`.
fn records_the_file_now(repo: &Path, name: &str) -> bool {
    let listing: String = git_text(repo, &["--no-optional-locks", "ls-files", "--debug", "--", name]);
    let inode: u64 = std::fs::metadata(repo.join(name)).expect("the file").ino();
    return listing.contains(format!("ino: {inode}").as_str());
}

/// A stale entry is refreshed in the file the refresh names,
/// so the commit that reads that index does not read the file again.
#[test]
fn a_stale_entry_is_refreshed_in_the_index_it_names() {
    let root: PathBuf = fixture("commit-index-refresh-stale");
    let repo: PathBuf = stale_repository(root.as_path());
    let index: PathBuf = repo.join(".git").join("index");
    assert!(!records_the_file_now(repo.as_path(), "a.txt"), "the fixture's index must be stale, or this control proves nothing");
    assert!(refresh_stat_data(&git_context(), repo.as_path(), index.as_path()), "Git wrote the refresh");
    assert!(records_the_file_now(repo.as_path(), "a.txt"), "the refresh is in the index it named");
    remove(root.as_path());
}

/// An unstaged change makes `git update-index --refresh` exit one,
/// and that answer still carries the refreshed stat data of every other entry:
///  it is a refresh Git kept.
#[test]
fn an_unstaged_change_does_not_lose_the_refresh_of_the_other_entries() {
    let root: PathBuf = fixture("commit-index-refresh-changed");
    let repo: PathBuf = stale_repository(root.as_path());
    std::fs::write(repo.join("b.txt"), "changed\n").expect("an unstaged change");
    let index: PathBuf = repo.join(".git").join("index");
    assert!(refresh_stat_data(&git_context(), repo.as_path(), index.as_path()), "a path that needs an update still keeps the refresh");
    assert!(records_the_file_now(repo.as_path(), "a.txt"), "the other entries keep the refresh");
    remove(root.as_path());
}

/// An index another program has locked is left exactly as it was,
/// and the refresh says so instead of failing the commit.
#[test]
fn an_index_another_program_has_locked_is_left_as_it_was() {
    let root: PathBuf = fixture("commit-index-refresh-locked");
    let repo: PathBuf = stale_repository(root.as_path());
    let index: PathBuf = repo.join(".git").join("index");
    let before: Vec<u8> = std::fs::read(index.as_path()).expect("the index");
    std::fs::write(repo.join(".git").join("index.lock"), b"").expect("a lock another program holds");
    assert!(!refresh_stat_data(&git_context(), repo.as_path(), index.as_path()), "a locked index is not refreshed");
    assert_eq!(std::fs::read(index.as_path()).expect("the index"), before, "the index is as it was");
    remove(root.as_path());
}

/// A Git that cannot start is a refresh that was not kept,
/// not a failure of the commit.
#[test]
fn a_git_that_cannot_start_is_answered_without_a_failure() {
    let root: PathBuf = fixture("commit-index-refresh-no-git");
    let repo: PathBuf = stale_repository(root.as_path());
    let index: PathBuf = repo.join(".git").join("index");
    let mut context: GitContext = git_context();
    context.real_git = PathBuf::from("/nonexistent/git");
    assert!(!refresh_stat_data(&context, repo.as_path(), index.as_path()), "a Git that cannot start keeps nothing");
    remove(root.as_path());
}
