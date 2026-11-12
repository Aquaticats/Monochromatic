//! Controls for the stat-data refresh an index gets before the scratch commit:
//! a stale entry is refreshed in the file the refresh names,
//! an unstaged change does not lose the refresh of the other entries,
//! an index another program has locked is left as it was,
//! and a Git that cannot start is answered without a failure.

use super::*;
use crate::commit_index_stat_staleness::WorktreeStaging;
use crate::test_support::{fixture, git, git_context, git_text, remove, renew_inode, repository};
use crate::transaction_git::GitContext;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::time::Instant;

/// A repository with `a.txt` and `b.txt` committed,
/// so both index entries record the stat data their files have.
fn committed_repository(root: &Path) -> PathBuf {
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    std::fs::write(repo.join("b.txt"), "b\n").expect("b");
    git(repo.as_path(), &["add", "a.txt", "b.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=files"]);
    return repo;
}

/// A repository with `a.txt` and `b.txt` committed,
/// and `a.txt` with a new inode while its bytes and its modification time stay as they were:
/// what the move to another disk left every worktree.
fn stale_repository(root: &Path) -> PathBuf {
    let repo: PathBuf = committed_repository(root);
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

/// The refresh the commit asks for looks at the entries the walk named and no others:
/// a path the commit stages from the worktree is written afresh by `git add`,
/// so its record costs the commit nothing and the refresh leaves it alone.
/// Why:
///   Refreshing the whole index asked the file system about every entry of the index,
/// tens of thousands of questions on a worktree of this repository,
/// where the entries that lost their stat data are the files the person changed since the landing before.
#[test]
fn the_refresh_looks_at_the_entries_the_walk_named_and_no_other() {
    let root: PathBuf = fixture("commit-index-refresh-named-only");
    let repo: PathBuf = committed_repository(root.as_path());
    // Both files lose their recorded stat data, and the commit stages `a.txt` from the worktree, so the walk names
    // `b.txt` alone.
    renew_inode(repo.join("a.txt").as_path());
    renew_inode(repo.join("b.txt").as_path());
    let index: PathBuf = repo.join(".git").join("index");
    let staged: Vec<std::ffi::OsString> = vec![std::ffi::OsString::from("a.txt")];
    assert_eq!(
        refresh_stat_data_when_stale(&git_context(), repo.as_path(), index.as_path(), WorktreeStaging::Named(staged.as_slice())),
        StatRefresh::Kept,
        "the entry the walk named is refreshed"
    );
    assert!(records_the_file_now(repo.as_path(), "b.txt"), "the entry the walk named is refreshed in the index");
    assert!(
        !records_the_file_now(repo.as_path(), "a.txt"),
        "the entry the commit stages from the worktree is left to `git add`, which writes its stat data there"
    );
    remove(root.as_path());
}

/// A file the worktree no longer holds is refreshed without removing its entry:
/// a refresh reports a path that needs an update and never records a deletion nobody asked for.
#[test]
fn a_named_refresh_keeps_the_entry_of_a_file_the_worktree_lost() {
    let root: PathBuf = fixture("commit-index-refresh-gone");
    let repo: PathBuf = committed_repository(root.as_path());
    std::fs::remove_file(repo.join("b.txt")).expect("the worktree loses b.txt");
    let index: PathBuf = repo.join(".git").join("index");
    let before: String = git_text(repo.as_path(), &["--no-optional-locks", "ls-files", "--stage"]);
    assert!(refresh_named_stat_data(&git_context(), repo.as_path(), index.as_path(), &[b"b.txt".to_vec()]), "the refresh is kept");
    assert_eq!(
        git_text(repo.as_path(), &["--no-optional-locks", "ls-files", "--stage"]),
        before,
        "every entry is still there, the lost file's included"
    );
    remove(root.as_path());
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

/// An index that already records the worktree's stat data costs no Git at all:
/// the context names a Git that cannot start,
/// so a refresh that had run would answer that the refresh was not kept.
#[test]
fn an_index_that_records_the_stat_data_is_answered_without_git() {
    let root: PathBuf = fixture("commit-index-refresh-up-to-date");
    let repo: PathBuf = committed_repository(root.as_path());
    let index: PathBuf = repo.join(".git").join("index");
    let mut context: GitContext = git_context();
    context.real_git = PathBuf::from("/nonexistent/git");
    assert_eq!(
        refresh_stat_data_when_stale(&context, repo.as_path(), index.as_path(), WorktreeStaging::None),
        StatRefresh::NotNeeded,
        "an up-to-date index costs no Git start and takes no lock"
    );
    remove(root.as_path());
}

/// The positive control for the answer above:
/// the same Git that cannot start is asked for an index whose file has a new inode,
/// so the answer says the refresh was asked for and not kept.
#[test]
fn an_index_that_lost_the_worktrees_stat_data_asks_git() {
    let root: PathBuf = fixture("commit-index-refresh-needed");
    let repo: PathBuf = stale_repository(root.as_path());
    let index: PathBuf = repo.join(".git").join("index");
    let mut context: GitContext = git_context();
    context.real_git = PathBuf::from("/nonexistent/git");
    assert_eq!(
        refresh_stat_data_when_stale(&context, repo.as_path(), index.as_path(), WorktreeStaging::None),
        StatRefresh::NotKept,
        "a stale index is refreshed, so this Git was asked"
    );
    remove(root.as_path());
}

/// A refresh that meets another program's `index.lock` is not kept and does not wait for it:
/// the lock is held for the whole call and Git answers it at once,
/// where a step that waited would sit at `LANDING_LOCK_CEILING` (a minute) instead of returning.
#[test]
fn a_refresh_that_meets_another_programs_lock_does_not_wait_for_it() {
    let root: PathBuf = fixture("commit-index-refresh-held-lock");
    let repo: PathBuf = stale_repository(root.as_path());
    let index: PathBuf = repo.join(".git").join("index");
    let before: Vec<u8> = std::fs::read(index.as_path()).expect("the index");
    let lock: PathBuf = repo.join(".git").join("index.lock");
    std::fs::write(lock.as_path(), b"").expect("a lock another program holds");
    let began: Instant = Instant::now();
    let outcome: StatRefresh = refresh_stat_data_when_stale(&git_context(), repo.as_path(), index.as_path(), WorktreeStaging::None);
    let spent: std::time::Duration = began.elapsed();
    assert_eq!(outcome, StatRefresh::NotKept, "a held lock is a refresh Git did not write");
    assert_eq!(std::fs::read(index.as_path()).expect("the index"), before, "the real index is as it was");
    assert!(
        spent < crate::wait_ceilings::LANDING_LOCK_CEILING,
        "the refresh waited {spent:?} for a lock it never gets, where the longest a landing waits for one is {:?}",
        crate::wait_ceilings::LANDING_LOCK_CEILING
    );
    remove(root.as_path());
}
