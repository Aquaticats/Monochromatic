//! Controls for the tree questions of the path reset,
//! against real Git's listings.

use super::*;
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_object::{from_hex, to_hex};
use crate::test_support::{fixture, git, git_text, remove, repository};
use std::path::PathBuf;

/// A repository with two commits:
/// the second changes `a`,
/// adds `d/n/m`,
/// removes `d/y` and turns `l` into a file.
fn history(name: &str) -> (PathBuf, PathBuf, String, String) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    for (path, text) in [("a", "1"), ("d/x", "1"), ("d/y", "1"), ("l/inner", "1")] {
        let file: PathBuf = repo.join(path);
        std::fs::create_dir_all(file.parent().expect("parent")).expect("directories");
        std::fs::write(file, text).expect("write");
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=one"]);
    let first: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(repo.join("a"), "2").expect("write");
    std::fs::create_dir_all(repo.join("d/n")).expect("directories");
    std::fs::write(repo.join("d/n/m"), "2").expect("write");
    std::fs::remove_file(repo.join("d/y")).expect("remove");
    std::fs::remove_dir_all(repo.join("l")).expect("remove");
    std::fs::write(repo.join("l"), "now a file").expect("write");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=two"]);
    let second: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    return (root, repo, first, second);
}

/// The entries at named paths equal `git ls-tree -r` for those paths,
/// whether a name is a file,
/// a directory or absent.
#[test]
fn entries_at_names_equal_ls_tree() {
    let (root, repo, _first, second) = history("index-reset-tree-entries");
    let database = ObjectDatabase::open(repo.join(".git").as_path(), None, ObjectFormat::Sha1).expect("database");
    let tree: Vec<u8> = root_tree(&database, second.as_str(), ObjectFormat::Sha1).expect("tree");
    let names: Vec<Vec<u8>> = vec![b"a".to_vec(), b"d".to_vec(), b"l".to_vec(), b"missing".to_vec(), b"d/n/m".to_vec(), b"a/not-a-directory".to_vec()];
    let found: Vec<FlatEntry> = tree_entries_at(&database, tree.as_slice(), names.as_slice(), ObjectFormat::Sha1).expect("entries");
    let listing: String = git_text(repo.as_path(), &["ls-tree", "-r", second.as_str(), "--", "a", "d", "l"]);
    let expected: Vec<String> = listing.lines().map(|line| { return line.to_string(); }).collect();
    let mine: Vec<String> = found.iter().map(|entry| { return format!("{:06o} blob {}\t{}", entry.mode, to_hex(entry.id.as_slice()), String::from_utf8_lossy(entry.path.as_slice())); }).collect();
    assert_eq!(mine, expected);
    remove(root.as_path());
}

/// The changed paths equal `git diff-tree -r --name-only`,
/// including a directory that became a file,
/// and an absent old tree
/// lists everything.
#[test]
fn changed_paths_equal_diff_tree() {
    let (root, repo, first, second) = history("index-reset-tree-changed");
    let database = ObjectDatabase::open(repo.join(".git").as_path(), None, ObjectFormat::Sha1).expect("database");
    let old: Vec<u8> = root_tree(&database, first.as_str(), ObjectFormat::Sha1).expect("old tree");
    let new: Vec<u8> = root_tree(&database, second.as_str(), ObjectFormat::Sha1).expect("new tree");
    let mine: Vec<String> = changed_paths(&database, Some(old.as_slice()), new.as_slice(), ObjectFormat::Sha1).expect("paths").iter().map(|path| { return String::from_utf8_lossy(path).into_owned(); }).collect();
    let mut expected: Vec<String> = git_text(repo.as_path(), &["diff-tree", "-r", "--name-only", "--no-renames", first.as_str(), second.as_str()]).lines().map(str::to_string).collect();
    expected.sort();
    assert_eq!(mine, expected);
    let everything: Vec<Vec<u8>> = changed_paths(&database, None, new.as_slice(), ObjectFormat::Sha1).expect("paths");
    let all: Vec<String> = git_text(repo.as_path(), &["ls-tree", "-r", "--name-only", second.as_str()]).lines().map(str::to_string).collect();
    assert_eq!(everything.iter().map(|path| { return String::from_utf8_lossy(path).into_owned(); }).collect::<Vec<String>>(), all);
    remove(root.as_path());
}

/// A commit name and a tree name both give the root tree;
/// a blob does not.
#[test]
fn a_commit_or_a_tree_names_a_root_tree() {
    let (root, repo, _first, second) = history("index-reset-tree-root");
    let database = ObjectDatabase::open(repo.join(".git").as_path(), None, ObjectFormat::Sha1).expect("database");
    let tree_hex: String = git_text(repo.as_path(), &["rev-parse", format!("{second}^{{tree}}").as_str()]);
    let from_commit = root_tree(&database, second.as_str(), ObjectFormat::Sha1).expect("from commit");
    assert_eq!(from_commit, from_hex(tree_hex.as_bytes(), ObjectFormat::Sha1).expect("hex"));
    assert_eq!(root_tree(&database, tree_hex.as_str(), ObjectFormat::Sha1), Some(from_commit));
    let blob_hex: String = git_text(repo.as_path(), &["rev-parse", format!("{second}:a").as_str()]);
    assert_eq!(root_tree(&database, blob_hex.as_str(), ObjectFormat::Sha1), None);
    remove(root.as_path());
}
