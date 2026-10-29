//! Controls for the empty-tree table:
//!  every constant equals what real Git hashes in a repository of that object format.

use super::*;
use crate::capture_invocation::capture_invocation_layout;
use crate::test_support::{fixture, git, git_context, git_text, remove};
use std::path::PathBuf;

/// Create an empty repository of `format` and return the name Git gives `tree 0` there,
///  with the format Git reports for it.
fn oracle(name: &str, format: &str) -> (PathBuf, String, String) {
    let root: PathBuf = fixture(name);
    git(root.as_path(), &["init", "--quiet", format!("--object-format={format}").as_str()]);
    let hashed: String = git_text(root.as_path(), &["hash-object", "-t", "tree", "--stdin"]);
    let reported: String = git_text(root.as_path(), &["rev-parse", "--show-object-format"]);
    return (root, hashed, reported);
}

/// The SHA-1 constant is what `git hash-object -t tree --stdin` prints in a SHA-1 repository,
///  and the format name Git reports selects it.
#[test]
fn the_sha1_name_is_what_git_hashes() {
    let (root, hashed, reported): (PathBuf, String, String) = oracle("empty-tree-sha1", "sha1");
    assert_eq!(reported, "sha1");
    assert_eq!(hashed, SHA1_EMPTY_TREE);
    assert_eq!(empty_tree_name(reported.as_str()), Some(hashed.as_str()));
    remove(root.as_path());
}

/// The SHA-256 constant is what Git prints in a SHA-256 repository,
///  and it differs from the SHA-1 one.
#[test]
fn the_sha256_name_is_what_git_hashes() {
    let (root, hashed, reported): (PathBuf, String, String) = oracle("empty-tree-sha256", "sha256");
    assert_eq!(reported, "sha256");
    assert_eq!(hashed, SHA256_EMPTY_TREE);
    assert_eq!(empty_tree_name(reported.as_str()), Some(hashed.as_str()));
    assert_ne!(SHA1_EMPTY_TREE, SHA256_EMPTY_TREE);
    remove(root.as_path());
}

/// A format this table does not know is never answered,
///  so the caller asks Git.
#[test]
fn an_unknown_format_is_not_answered() {
    assert_eq!(empty_tree_name("sha3"), None);
    assert_eq!(empty_tree_name(""), None);
    assert_eq!(empty_tree_name("SHA1"), None);
}

/// The invocation capture of a SHA-256 repository records the SHA-256 empty tree,
///  as the spawned hash did.
#[test]
fn the_capture_of_a_sha256_repository_records_the_sha256_empty_tree() {
    let (root, hashed, _reported): (PathBuf, String, String) = oracle("empty-tree-capture-sha256", "sha256");
    let layout = capture_invocation_layout(&git_context(), root.as_path(), false).expect("layout");
    assert_eq!(layout.object_format, "sha256");
    assert_eq!(layout.empty_tree_oid, hashed);
    remove(root.as_path());
}
