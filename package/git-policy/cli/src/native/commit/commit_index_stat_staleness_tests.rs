//! Controls for the question whether an index still records the stat data its worktree has:
//! an index Git has just written needs no refresh,
//! a file whose inode changed does,
//! and a path the commit stages from the worktree does not.

use super::{WorktreeStaging, index_stat_data_needs_refresh};
use crate::test_support::{fixture, git, git_context, git_text, remove, renew_inode, repository};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

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

/// The real index of `repo`.
fn real_index(repo: &Path) -> PathBuf {
    return repo.join(".git").join("index");
}

/// The answer for a commit of `paths`,
/// as the command line types them.
fn needs_refresh_of(repo: &Path, paths: &[&str]) -> bool {
    let typed: Vec<OsString> = paths.iter().map(|path| { return OsString::from(*path); }).collect();
    return index_stat_data_needs_refresh(&git_context(), repo, real_index(repo).as_path(), WorktreeStaging::Named(typed.as_slice()));
}

/// The answer for a commit whose content comes from the index alone.
fn needs_refresh(repo: &Path) -> bool {
    return index_stat_data_needs_refresh(&git_context(), repo, real_index(repo).as_path(), WorktreeStaging::None);
}

/// An index Git has just written describes the worktree,
/// so the commit pays no refresh:
/// the positive control for this is the inode case below,
/// which this answer must not swallow.
#[test]
fn an_index_that_records_the_worktrees_stat_data_needs_no_refresh() {
    let root: PathBuf = fixture("index-staleness-fresh");
    let repo: PathBuf = committed_repository(root.as_path());
    assert!(!needs_refresh(repo.as_path()), "a commit the worktree has not moved starts no Git for the stat data");
    remove(root.as_path());
}

/// A file the move to another disk gave a new inode,
/// with its bytes and its modification time kept,
/// is the staleness the refresh exists for:
/// the index records an inode the file no longer has.
#[test]
fn a_file_whose_inode_changed_needs_the_refresh() {
    let root: PathBuf = fixture("index-staleness-inode");
    let repo: PathBuf = committed_repository(root.as_path());
    renew_inode(repo.join("a.txt").as_path());
    assert!(needs_refresh(repo.as_path()), "a moved file leaves stat data the worktree no longer has");
    remove(root.as_path());
}

/// A change the commit stages from the worktree is written into the private index afresh,
/// so its stale record cannot cost the scratch commit a read,
/// and a change it does not stage can:
/// one index,
/// two selections,
/// two answers.
#[test]
fn only_a_change_the_commit_does_not_stage_needs_the_refresh() {
    let root: PathBuf = fixture("index-staleness-named");
    let repo: PathBuf = committed_repository(root.as_path());
    std::fs::write(repo.join("a.txt"), "changed\n").expect("change a");
    assert!(!needs_refresh_of(repo.as_path(), &["a.txt"]), "the commit stages a.txt from the worktree, so its record is replaced");
    assert!(needs_refresh_of(repo.as_path(), &["b.txt"]), "the commit leaves a.txt alone, so its record costs the scratch commit a read");
    assert!(needs_refresh(repo.as_path()), "an index commit takes what the index holds, so the change is not restaged");
    remove(root.as_path());
}

/// An entry a landing left without stat data,
/// which is what `git reset --no-refresh` writes,
/// is stale even though the bytes it names are the worktree's.
#[test]
fn an_entry_without_stat_data_needs_the_refresh() {
    let root: PathBuf = fixture("index-staleness-unstatted");
    let repo: PathBuf = committed_repository(root.as_path());
    let blob: String = git_text(repo.as_path(), &["rev-parse", "HEAD:a.txt"]);
    let cache_info: String = format!("100644,{blob},a.txt");
    git(repo.as_path(), &["update-index", "--add", "--cacheinfo", cache_info.as_str()]);
    assert!(needs_refresh(repo.as_path()), "an entry with no stat data is one the scratch commit hashes");
    remove(root.as_path());
}

/// A file marked assume-unchanged is one Git's own refresh never re-reads,
/// so the refresh here would change nothing and is not asked for.
#[test]
fn an_assume_unchanged_entry_needs_no_refresh() {
    let root: PathBuf = fixture("index-staleness-assume");
    let repo: PathBuf = committed_repository(root.as_path());
    git(repo.as_path(), &["update-index", "--assume-unchanged", "--", "a.txt"]);
    renew_inode(repo.join("a.txt").as_path());
    assert!(!needs_refresh(repo.as_path()), "Git's refresh skips an assume-unchanged entry, so no Git is started for it");
    remove(root.as_path());
}

/// An index this reader cannot understand is left to Git's own refresh.
#[test]
fn an_index_that_cannot_be_read_needs_the_refresh() {
    let root: PathBuf = fixture("index-staleness-unreadable");
    let repo: PathBuf = committed_repository(root.as_path());
    std::fs::write(real_index(repo.as_path()), b"not an index").expect("a file no reader understands");
    assert!(needs_refresh(repo.as_path()), "a declined read asks Git");
    remove(root.as_path());
}

/// A repository nothing was ever staged in has no index,
/// and no stat data to refresh,
/// so the refresh is not asked for either.
#[test]
fn a_missing_index_needs_no_refresh() {
    let root: PathBuf = fixture("index-staleness-missing");
    let repo: PathBuf = committed_repository(root.as_path());
    std::fs::remove_file(real_index(repo.as_path())).expect("remove the index");
    assert!(!needs_refresh(repo.as_path()), "there is no stat data to refresh");
    remove(root.as_path());
}
