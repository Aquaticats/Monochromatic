//! Controls for the cached stat data of the explicit-path private index:
//!  an entry the base shares with the real index keeps its stat data, so the shadow commit does not read the file again,
//!  and an entry the worktree changed since the real index recorded it does not keep a stale one.

use super::*;
use crate::test_support::{REAL_GIT, fixture, git, git_context, git_text, remove, repository};
use std::path::PathBuf;
use std::process::{Command, Output};

/// The line `git ls-files --debug` prints for the modification time of an entry that was never refreshed.
const UNSET_MODIFICATION_TIME: &str = "mtime: 0:0";

/// A repository with `a.txt` and `b.txt` committed, and the private indexes beside the real index.
fn committed_repository(root: &Path) -> (PathBuf, CaptureIndexes, String) {
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    std::fs::write(repo.join("b.txt"), "b\n").expect("b");
    git(repo.as_path(), &["add", "a.txt", "b.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=files"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: repo.join(".git").join("captured.index"),
        commit_index: repo.join(".git").join("commit.index"),
        object_directory: repo.join(".git").join("objects"),
    };
    return (repo, indexes, base);
}

/// What `git ls-files --debug` prints for `path` in the private index.
fn debug_listing(repo: &Path, index: &Path, path: &str) -> String {
    let output = Command::new(REAL_GIT)
        .current_dir(repo)
        .env("GIT_INDEX_FILE", index)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .args(["ls-files", "--debug", "--", path])
        .output()
        .expect("ls-files");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    return String::from_utf8(output.stdout).expect("UTF-8");
}

/// Initializes the private index for an explicit-path commit of `paths`.
fn initialize_explicit(repo: &Path, indexes: &CaptureIndexes, base: &str, paths: &[OsString]) {
    let choice: IndexSelection = IndexSelection {
        mode: TransactionMode::ExplicitPath,
        pathspecs: paths,
        pathspec_file: None,
        pathspec_file_nul: false,
        stage_into_index: false,
        stage_tracked_changes: false,
        base_revision: base,
    };
    initialize_commit_index(&git_context(), repo, repo.join(".git").join("index").as_path(), indexes, &choice)
        .expect("initialize");
}

/// A path the commit does not name keeps the real index's stat data in the private index.
#[test]
fn an_unchanged_entry_keeps_its_cached_stat_data() {
    let root: PathBuf = fixture("commit-index-stat-kept");
    let (repo, indexes, base) = committed_repository(root.as_path());
    std::fs::write(repo.join("b.txt"), "changed\n").expect("change b");
    initialize_explicit(repo.as_path(), &indexes, base.as_str(), &[OsString::from("b.txt")]);
    let listing: String = debug_listing(repo.as_path(), indexes.commit_index.as_path(), "a.txt");
    assert!(listing.contains("mtime:"), "the listing names the entry: {listing}");
    assert!(!listing.contains(UNSET_MODIFICATION_TIME), "a.txt must keep its stat data: {listing}");
    remove(root.as_path());
}

/// The same control run through the real index shows what "kept" looks like, so the control above can fail:
///  a private index built the old way, by a plain `read-tree` of the base, holds no stat data at all.
#[test]
fn a_plain_read_tree_leaves_no_stat_data() {
    let root: PathBuf = fixture("commit-index-stat-positive-control");
    let (repo, indexes, base) = committed_repository(root.as_path());
    let plain: Output = Command::new(REAL_GIT)
        .current_dir(repo.as_path())
        .env("GIT_INDEX_FILE", indexes.commit_index.as_path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .args(["read-tree", base.as_str()])
        .output()
        .expect("read-tree");
    assert!(plain.status.success());
    let listing: String = debug_listing(repo.as_path(), indexes.commit_index.as_path(), "a.txt");
    assert!(listing.contains(UNSET_MODIFICATION_TIME), "{listing}");
    remove(root.as_path());
}

/// A path the worktree changed after the real index recorded it never keeps a stale stat entry as unchanged:
///  the private index still turns into the tree of the base plus the named path.
#[test]
fn a_named_path_is_staged_from_the_worktree_not_from_the_cached_entry() {
    let root: PathBuf = fixture("commit-index-stat-named");
    let (repo, indexes, base) = committed_repository(root.as_path());
    std::fs::write(repo.join("a.txt"), "changed\n").expect("change a");
    initialize_explicit(repo.as_path(), &indexes, base.as_str(), &[OsString::from("a.txt")]);
    let tree: String = write_private_tree(
        &git_context(),
        repo.as_path(),
        indexes.commit_index.as_path(),
        indexes.object_directory.as_path(),
    )
    .expect("tree");
    let content: String = git_text(repo.as_path(), &["show", format!("{tree}:a.txt").as_str()]);
    assert_eq!(content, "changed");
    remove(root.as_path());
}

/// The content of `path` in the tree the private index produces.
fn private_content(repo: &Path, indexes: &CaptureIndexes, path: &str) -> String {
    let tree: String = write_private_tree(
        &git_context(),
        repo,
        indexes.commit_index.as_path(),
        indexes.object_directory.as_path(),
    )
    .expect("tree");
    return git_text(repo, &["show", format!("{tree}:{path}").as_str()]);
}

/// A repository with `a.txt`, `b.txt` and `d/x.txt` committed.
fn repository_with_directory(root: &Path) -> (PathBuf, CaptureIndexes, String) {
    let (repo, indexes, _base) = committed_repository(root);
    std::fs::create_dir(repo.join("d")).expect("directory");
    std::fs::write(repo.join("d").join("x.txt"), "x\n").expect("x");
    git(repo.as_path(), &["add", "d/x.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=directory"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    return (repo, indexes, base);
}

/// A file marked assume-unchanged that the commit names is staged from the worktree, as Git's own partial commit does;
///  without that, the mark in the copied index makes `git add` skip it and the named file is silently left out.
#[test]
fn a_named_assume_unchanged_file_is_committed() {
    let root: PathBuf = fixture("commit-index-assume-named");
    let (repo, indexes, base) = committed_repository(root.as_path());
    git(repo.as_path(), &["update-index", "--assume-unchanged", "--", "a.txt"]);
    std::fs::write(repo.join("a.txt"), "changed a\n").expect("change a");
    std::fs::write(repo.join("b.txt"), "changed b\n").expect("change b");
    initialize_explicit(repo.as_path(), &indexes, base.as_str(), &[OsString::from("a.txt"), OsString::from("b.txt")]);
    assert_eq!(private_content(repo.as_path(), &indexes, "a.txt"), "changed a");
    assert_eq!(private_content(repo.as_path(), &indexes, "b.txt"), "changed b");
    remove(root.as_path());
}

/// A file marked assume-unchanged that the commit does not name stays as the base has it.
#[test]
fn an_unnamed_assume_unchanged_file_stays_as_the_base_has_it() {
    let root: PathBuf = fixture("commit-index-assume-unnamed");
    let (repo, indexes, base) = committed_repository(root.as_path());
    git(repo.as_path(), &["update-index", "--assume-unchanged", "--", "a.txt"]);
    std::fs::write(repo.join("a.txt"), "changed a\n").expect("change a");
    std::fs::write(repo.join("b.txt"), "changed b\n").expect("change b");
    initialize_explicit(repo.as_path(), &indexes, base.as_str(), &[OsString::from("b.txt")]);
    assert_eq!(private_content(repo.as_path(), &indexes, "a.txt"), "a");
    assert_eq!(private_content(repo.as_path(), &indexes, "b.txt"), "changed b");
    remove(root.as_path());
}

/// A directory named by the commit takes the marked files inside it too.
#[test]
fn a_named_directory_takes_its_assume_unchanged_files() {
    let root: PathBuf = fixture("commit-index-assume-directory");
    let (repo, indexes, base) = repository_with_directory(root.as_path());
    git(repo.as_path(), &["update-index", "--assume-unchanged", "--", "d/x.txt"]);
    std::fs::write(repo.join("d").join("x.txt"), "changed x\n").expect("change x");
    initialize_explicit(repo.as_path(), &indexes, base.as_str(), &[OsString::from("d")]);
    assert_eq!(private_content(repo.as_path(), &indexes, "d/x.txt"), "changed x");
    remove(root.as_path());
}

/// A path marked skip-worktree by hand that the commit names does not fail the commit, as before the copy.
#[test]
fn a_named_skip_worktree_file_does_not_fail_the_commit() {
    let root: PathBuf = fixture("commit-index-skip-named");
    let (repo, indexes, base) = committed_repository(root.as_path());
    git(repo.as_path(), &["update-index", "--skip-worktree", "--", "a.txt"]);
    std::fs::write(repo.join("a.txt"), "changed a\n").expect("change a");
    std::fs::write(repo.join("b.txt"), "changed b\n").expect("change b");
    initialize_explicit(repo.as_path(), &indexes, base.as_str(), &[OsString::from("a.txt"), OsString::from("b.txt")]);
    assert_eq!(private_content(repo.as_path(), &indexes, "a.txt"), "changed a");
    assert_eq!(private_content(repo.as_path(), &indexes, "b.txt"), "changed b");
    remove(root.as_path());
}

/// A tracked empty file that the real index holds as intent-to-add stays in the tree the private index produces when the
///  commit names another path: the seeded copy kept the entry and a tree built from it leaves intent-to-add entries out.
#[test]
fn an_intent_to_add_tracked_empty_file_is_not_dropped_from_the_commit() {
    let root: PathBuf = fixture("commit-index-intent-empty");
    let (repo, indexes, _base) = committed_repository(root.as_path());
    std::fs::write(repo.join("empty"), "").expect("empty file");
    git(repo.as_path(), &["add", "empty"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=empty"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["rm", "--cached", "--quiet", "empty"]);
    git(repo.as_path(), &["add", "--intent-to-add", "empty"]);
    std::fs::write(repo.join("a.txt"), "changed a\n").expect("change a");
    initialize_explicit(repo.as_path(), &indexes, base.as_str(), &[OsString::from("a.txt")]);
    let tree: String = write_private_tree(
        &git_context(),
        repo.as_path(),
        indexes.commit_index.as_path(),
        indexes.object_directory.as_path(),
    )
    .expect("tree");
    let names: String = git_text(repo.as_path(), &["ls-tree", "--name-only", tree.as_str()]);
    assert!(names.lines().any(|name| { return name == "empty"; }), "the tree must still hold the empty file: {names}");
    assert_eq!(private_content(repo.as_path(), &indexes, "a.txt"), "changed a");
    remove(root.as_path());
}

/// An index no mark touches still seeds the private index, so the safeguard costs nothing for it:
///  the copy is used exactly when the reader found the index plain, and an absent file seeds nothing.
#[test]
fn only_a_plain_index_seeds_the_private_index() {
    let root: PathBuf = fixture("commit-index-seeds");
    let (repo, indexes, _base) = committed_repository(root.as_path());
    let real: PathBuf = repo.join(".git").join("index");
    assert!(seeds_private_index(real.as_path()));
    git(repo.as_path(), &["update-index", "--assume-unchanged", "--", "a.txt"]);
    assert!(!seeds_private_index(real.as_path()));
    assert!(!seeds_private_index(indexes.commit_index.as_path()));
    remove(root.as_path());
}
