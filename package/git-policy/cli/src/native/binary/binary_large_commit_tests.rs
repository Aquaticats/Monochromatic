//! Binary-level controls for a commit that changes many files:
//! the tree it lands equals the tree real Git writes from the same index,
//! for edits,
//! new files,
//! deletions and renames.
//! Why:
//!  A landing of thousands of paths updates the worktree's index in process
//! (`index_reset_file.rs`,
//! `index_reset.rs`,
//! `index_reset_tree.rs`),
//! and a shortcut there that selected the wrong entries or read the wrong trees
//! would still land a commit,
//! with a tree or an index that differs from Git's.
//!  The size is large enough that a per-path child or a per-path tree walk would
//! show its cost,
//! and the comparison is Git's own `write-tree` over the staged index,
//! so nothing here depends on this machine's speed.

use super::commit::{assert_nothing_left, git_text, keep_maintenance_in_the_foreground, remove_settled, run_commit};
use super::support::{Fixture, fixture, git, porcelain, repository, stderr_of};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// How many files the commit changes.
///  Large enough that a walk of the tree for every name would be visible in the
/// object reads the guard beside the tree reader counts,
/// and small enough that four controls run in a few seconds.
const CHANGED_FILES: usize = 1_000;

/// How many directories the files are spread over.
const DIRECTORIES: usize = 10;

/// The repository-relative path of file `number`.
fn file_at(number: usize) -> String {
    return format!("d{:02}/f{:04}.txt", number % DIRECTORIES, number);
}

/// The repository-relative path of the new file `number`.
fn new_file_at(number: usize) -> String {
    return format!("n{:02}/new-{:04}.txt", number % DIRECTORIES, number);
}

/// A repository whose `CHANGED_FILES` files are committed,
/// with automatic maintenance kept in the foreground.
fn many_files(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    for number in 0..CHANGED_FILES {
        let file: PathBuf = repo.join(file_at(number));
        std::fs::create_dir_all(file.parent().expect("parent")).expect("directory");
        std::fs::write(file, format!("base {number}\n")).expect("file");
    }
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    keep_maintenance_in_the_foreground(fixture, repo.as_path());
    return repo;
}

/// Apply one kind of change to every file,
/// and stage it with real Git.
fn change_and_stage(fixture: &Fixture, repo: &Path, kind: &str) {
    for number in 0..CHANGED_FILES {
        let path: PathBuf = repo.join(file_at(number));
        match kind {
            "edit" => std::fs::write(path, format!("edit {number}\n")).expect("edit"),
            "new" => {
                let new: PathBuf = repo.join(new_file_at(number));
                std::fs::create_dir_all(new.parent().expect("parent")).expect("directory");
                std::fs::write(new, format!("new {number}\n")).expect("new file");
            }
            "delete" => std::fs::remove_file(path).expect("delete"),
            "rename" => std::fs::rename(path, repo.join(format!("{}.renamed", file_at(number)))).expect("rename"),
            _ => panic!("unknown change kind {kind}"),
        }
    }
    git(fixture, repo, &["add", "--all"]);
}

/// A commit of every file changed by `kind` lands the tree real Git writes from the staged index,
/// and leaves nothing behind.
fn a_large_commit_of(fixture_name: &str, kind: &str) {
    let fixture: Fixture = fixture(fixture_name);
    let repo: PathBuf = many_files(&fixture, "repo");
    change_and_stage(&fixture, repo.as_path(), kind);
    // Git's own tree of the index this commit starts from: the tree the commit must land.
    let expected: String = git_text(&fixture, repo.as_path(), &["write-tree"]).trim().to_string();
    let landed = run_commit(&fixture, repo.as_path(), &["commit", "--no-only", "--quiet", "--message=large"]);
    assert_eq!(landed.code, Some(0), "a large {kind} commit: {}", stderr_of(&landed));
    let landed_tree: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD^{tree}"]).trim().to_string();
    assert_eq!(landed_tree, expected, "a large {kind} commit lands the tree Git writes from the same index");
    assert_eq!(porcelain(&fixture, repo.as_path()), "", "a large {kind} commit leaves no staged or unstaged change");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A commit of a thousand edits to tracked files lands Git's tree.
#[test]
fn a_large_commit_of_edits_lands_gits_tree() {
    a_large_commit_of("commit-large-edits", "edit");
}

/// A commit of a thousand new files lands Git's tree.
#[test]
fn a_large_commit_of_new_files_lands_gits_tree() {
    a_large_commit_of("commit-large-new", "new");
}

/// A commit of a thousand deletions lands Git's tree.
#[test]
fn a_large_commit_of_deletions_lands_gits_tree() {
    a_large_commit_of("commit-large-delete", "delete");
}

/// A commit of a thousand renames lands Git's tree.
#[test]
fn a_large_commit_of_renames_lands_gits_tree() {
    a_large_commit_of("commit-large-rename", "rename");
}

/// A commit of a thousand files that lack their final newline:
/// the policy corrects every one of them,
/// and the commit and the worktree both hold the corrected bytes.
/// Why:
///  This is the shape a pass that corrects many files has:
/// one blob and one index entry per correction,
/// and the worktree copy proven unchanged before it is replaced.
/// The batch that writes them must not start a Git process per file,
/// and the tree it lands must be the corrected one.
#[test]
fn a_large_commit_of_files_missing_their_final_newline_lands_the_corrected_tree() {
    let fixture: Fixture = fixture("commit-large-correct");
    let repo: PathBuf = many_files(&fixture, "repo");
    for number in 0..CHANGED_FILES {
        std::fs::write(repo.join(file_at(number)), format!("edit {number}")).expect("edit without a newline");
    }
    git(&fixture, repo.as_path(), &["add", "--all"]);
    let landed = run_commit(&fixture, repo.as_path(), &["commit", "--no-only", "--quiet", "--message=corrected"]);
    assert_eq!(landed.code, Some(0), "a large corrected commit: {}", stderr_of(&landed));
    let landed_tree: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD^{tree}"]).trim().to_string();
    let installed_tree: String = git_text(&fixture, repo.as_path(), &["write-tree"]).trim().to_string();
    assert_eq!(landed_tree, installed_tree, "the commit lands the corrected tree the installed index holds");
    assert_eq!(porcelain(&fixture, repo.as_path()), "", "the correction reaches the worktree and leaves nothing staged");
    for number in [0, CHANGED_FILES / 2, CHANGED_FILES - 1] {
        let shown: String = git_text(&fixture, repo.as_path(), &["show", format!("HEAD:{}", file_at(number)).as_str()]);
        assert!(shown.ends_with('\n'), "the corrected file {number} ends with a newline: {shown:?}");
    }
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}
