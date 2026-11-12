//! Controls for the question whether an index still records the stat data its worktree has:
//! an index Git has just written needs no refresh,
//! a file whose inode changed does,
//! a path the commit stages from the worktree does not,
//! and the paths the walk names are exactly the stale ones,
//! so a refresh can look at those and no others.

use super::{EntrySlice, STAT_WALK_BREAK_EVEN_ENTRIES, StaleStatData, WorktreeStaging, stale_paths_in_slice, stale_stat_data, walk_in_slices, workers_for};
use crate::test_support::{age, fixture, git, git_context, git_text, remove, renew_inode, repository};
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
fn verdict_of(repo: &Path, paths: &[&str]) -> StaleStatData {
    let typed: Vec<OsString> = paths.iter().map(|path| { return OsString::from(*path); }).collect();
    return stale_stat_data(&git_context(), repo, real_index(repo).as_path(), WorktreeStaging::Named(typed.as_slice()));
}

/// The answer for a commit whose content comes from the index alone.
fn verdict(repo: &Path) -> StaleStatData {
    return stale_stat_data(&git_context(), repo, real_index(repo).as_path(), WorktreeStaging::None);
}

/// Whether an answer says a refresh is needed at all:
/// the paths a refresh has to look at,
/// or a question for Git.
fn needs_refresh(answer: &StaleStatData) -> bool {
    return !matches!(answer, StaleStatData::Current);
}

/// The answer for a commit of `paths`,
/// as the command line types them,
/// asked as whether the refresh is needed.
fn needs_refresh_of(repo: &Path, paths: &[&str]) -> bool {
    return needs_refresh(&verdict_of(repo, paths));
}

/// The answer for a commit whose content comes from the index alone,
/// asked as whether the refresh is needed.
fn needs_refresh_here(repo: &Path) -> bool {
    return needs_refresh(&verdict(repo));
}

/// An index Git has just written describes the worktree,
/// so the commit pays no refresh:
/// the positive control for this is the inode case below,
/// which this answer must not swallow.
#[test]
fn an_index_that_records_the_worktrees_stat_data_needs_no_refresh() {
    let root: PathBuf = fixture("index-staleness-fresh");
    let repo: PathBuf = committed_repository(root.as_path());
    assert!(!needs_refresh_here(repo.as_path()), "a commit the worktree has not moved starts no Git for the stat data");
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
    assert!(needs_refresh_here(repo.as_path()), "a moved file leaves stat data the worktree no longer has");
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
    assert!(needs_refresh_here(repo.as_path()), "an index commit takes what the index holds, so the change is not restaged");
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
    assert!(needs_refresh_here(repo.as_path()), "an entry with no stat data is one the scratch commit hashes");
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
    assert!(!needs_refresh_here(repo.as_path()), "Git's refresh skips an assume-unchanged entry, so no Git is started for it");
    remove(root.as_path());
}

/// An index this reader cannot understand is left to Git's own refresh.
#[test]
fn an_index_that_cannot_be_read_needs_the_refresh() {
    let root: PathBuf = fixture("index-staleness-unreadable");
    let repo: PathBuf = committed_repository(root.as_path());
    std::fs::write(real_index(repo.as_path()), b"not an index").expect("a file no reader understands");
    assert_eq!(verdict(repo.as_path()), StaleStatData::Unknown, "a declined read asks Git for the whole index");
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
    assert!(!needs_refresh_here(repo.as_path()), "there is no stat data to refresh");
    remove(root.as_path());
}

/// A repository whose `files` files are committed,
/// each of them aged so its entry is not racily clean.
fn many_files_repository(root: &Path, files: usize) -> PathBuf {
    let repo: PathBuf = repository(root, "repo");
    for number in 0..files {
        let path: PathBuf = repo.join(format!("f{number}.txt"));
        std::fs::write(path.as_path(), format!("{number}\n")).expect("write");
        age(path.as_path());
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=files"]);
    return repo;
}

/// The entries of `repo`'s index,
/// and the walk of all of them.
fn slice_of(repo: &Path) -> EntrySlice<'static> {
    let bytes: Vec<u8> = std::fs::read(real_index(repo)).expect("index");
    let parsed: crate::index_full::FullIndex = crate::index_full::parse_full_index(bytes.as_slice(), 20).expect("parsed");
    // The control holds them for the length of its walk, and leaking them names that once.
    let index: &'static crate::index_full::FullIndex = Box::leak(Box::new(parsed));
    return EntrySlice { entries: index.entries.as_slice(), worktree_root: Box::leak(repo.to_path_buf().into_boxed_path()), restaged: &[] };
}

/// The slices answer exactly what the one linear walk answers,
/// for every thread count and both answers:
/// this is the control that the parallel walk changed no verdict.
#[test]
fn the_sliced_walk_answers_as_the_one_linear_walk_does() {
    let root: PathBuf = fixture("index-staleness-slices");
    let repo: PathBuf = many_files_repository(root.as_path(), 40);
    assert!(stale_paths_in_slice(slice_of(repo.as_path())).is_empty(), "a clean index names no path on one thread");
    for workers in 1..=8 {
        assert!(
            walk_in_slices(slice_of(repo.as_path()), workers).is_some_and(|stale| { return stale.is_empty(); }),
            "a clean index names no path on {workers} threads"
        );
    }
    renew_inode(repo.join("f10.txt").as_path());
    assert_eq!(stale_paths_in_slice(slice_of(repo.as_path())), vec![b"f10.txt".to_vec()], "the renewed inode is named on one thread");
    for workers in 1..=8 {
        assert_eq!(
            walk_in_slices(slice_of(repo.as_path()), workers),
            Some(vec![b"f10.txt".to_vec()]),
            "the renewed inode is named on {workers} threads"
        );
    }
    remove(root.as_path());
}

/// The walk names the entries that lost their stat data and no others:
/// the whole of what the refresh has to look at,
/// where a yes-or-no answer left the refresh looking at every entry of the index.
#[test]
fn the_walk_names_every_stale_path_and_no_other() {
    let root: PathBuf = fixture("index-staleness-names");
    let repo: PathBuf = many_files_repository(root.as_path(), 40);
    renew_inode(repo.join("f3.txt").as_path());
    renew_inode(repo.join("f31.txt").as_path());
    assert_eq!(
        verdict(repo.as_path()),
        StaleStatData::Stale { paths: vec![b"f3.txt".to_vec(), b"f31.txt".to_vec()], entries: 40 },
        "the two entries whose inodes moved are the ones the refresh is told about, and the index's size comes with them"
    );
    remove(root.as_path());
}

/// Below the break-even of the neighbouring file-tree jobs the walk keeps one thread.
#[test]
fn a_walk_below_the_break_even_uses_one_thread() {
    assert_eq!(workers_for(STAT_WALK_BREAK_EVEN_ENTRIES - 1), 1, "a short walk pays no thread");
}
