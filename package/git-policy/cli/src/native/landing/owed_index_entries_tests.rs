//! What:
//!  What the landing reads out of a tree or an index for the paths one record names.
//! Why:
//!  The pathspecs are the wrapper's own,
//! so a pathspec mode the service inherited must not widen them:
//!  `GIT_ICASE_PATHSPECS=1` makes `:(top,literal)a.txt` match `A.txt` too,
//!  and a read that reports the twin's entry is a read about a path the batch never selected.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect([...(await indexEntries(index, ['a.txt'])).keys()]).toEqual(['a.txt']);
//! ```

use super::{head_entries, index_entries, tree_entries};
use crate::test_support::{fixture, git, git_context, remove, repository};
use std::ffi::OsString;
use std::path::PathBuf;

/// The service's context when the command that first started it came from a shell that turned pathspec magic off and left matching
/// case-insensitive,
/// which every Git the service starts inherits.
fn inherited_literal_and_case_insensitive_pathspec_context() -> crate::transaction_git::GitContext {
    let mut context: crate::transaction_git::GitContext = git_context();
    context.overlay.push((OsString::from("GIT_LITERAL_PATHSPECS"), OsString::from("1")));
    context.overlay.push((OsString::from("GIT_ICASE_PATHSPECS"), OsString::from("1")));
    return context;
}

/// A repository holding two names that differ only in case,
/// both committed.
fn repository_with_a_case_twin(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("A.txt"), b"upper\n").expect("write A.txt");
    std::fs::write(repo.join("a.txt"), b"lower\n").expect("write a.txt");
    git(repo.as_path(), &["add", "--", "A.txt", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base", "--", "A.txt", "a.txt"]);
    return (root, repo);
}

/// Every read names the path it was asked about and nothing else,
/// so no entry of its case twin reaches the record the repair writes.
#[test]
fn the_entries_of_a_path_are_read_without_its_case_twin() {
    let (root, repo) = repository_with_a_case_twin("owed-entries-case-twin");
    let mut context: crate::transaction_git::GitContext = git_context();
    context.overlay.push((OsString::from("GIT_ICASE_PATHSPECS"), OsString::from("1")));
    let asked: [Vec<u8>; 1] = [b"a.txt".to_vec()];
    let from_the_tree = tree_entries(&context, repo.as_path(), "HEAD", asked.as_slice()).expect("the tree reads");
    let from_the_index = index_entries(&context, repo.as_path(), repo.join(".git/index").as_path(), asked.as_slice()).expect("the index reads");
    let from_the_head = head_entries(&context, repo.as_path(), asked.as_slice()).expect("the head reads");
    assert_eq!(from_the_tree.keys().collect::<Vec<&Vec<u8>>>(), vec![&asked[0]], "the tree is read at the path asked about");
    assert_eq!(from_the_index.keys().collect::<Vec<&Vec<u8>>>(), vec![&asked[0]], "the index is read at the path asked about");
    assert_eq!(from_the_head.keys().collect::<Vec<&Vec<u8>>>(), vec![&asked[0]], "the head is read at the path asked about");
    remove(root.as_path());
}

/// A mode the caller wrote as a global option reaches no read of the wrapper's own pathspecs either:
/// under `--literal-pathspecs` the generated `:(top,literal)a.txt` is a file name that matches nothing,
/// and every read answers an empty map although the entry is there,
/// which leaves the repair excluding nothing.
#[test]
fn the_entries_of_a_path_are_read_under_the_callers_literal_option() {
    let (root, repo) = repository_with_a_case_twin("owed-entries-literal-option");
    let mut context: crate::transaction_git::GitContext = git_context();
    context.global_prefix.push(OsString::from("--literal-pathspecs"));
    let asked: [Vec<u8>; 1] = [b"a.txt".to_vec()];
    let from_the_tree = tree_entries(&context, repo.as_path(), "HEAD", asked.as_slice()).expect("the tree reads");
    let from_the_index = index_entries(&context, repo.as_path(), repo.join(".git/index").as_path(), asked.as_slice()).expect("the index reads");
    let from_the_head = head_entries(&context, repo.as_path(), asked.as_slice()).expect("the head reads");
    assert_eq!(from_the_tree.keys().collect::<Vec<&Vec<u8>>>(), vec![&asked[0]], "the tree is read at the path asked about");
    assert_eq!(from_the_index.keys().collect::<Vec<&Vec<u8>>>(), vec![&asked[0]], "the index is read at the path asked about");
    assert_eq!(from_the_head.keys().collect::<Vec<&Vec<u8>>>(), vec![&asked[0]], "the head is read at the path asked about");
    remove(root.as_path());
}

/// The same for `--icase-pathspecs` in the caller's prefix,
/// which makes the reads name the case twin or refuse the magic outright.
#[test]
fn the_entries_of_a_path_are_read_under_the_callers_case_insensitive_option() {
    let (root, repo) = repository_with_a_case_twin("owed-entries-icase-option");
    let mut context: crate::transaction_git::GitContext = git_context();
    context.global_prefix.push(OsString::from("--icase-pathspecs"));
    let asked: [Vec<u8>; 1] = [b"a.txt".to_vec()];
    let from_the_tree = tree_entries(&context, repo.as_path(), "HEAD", asked.as_slice()).expect("the tree reads");
    let from_the_index = index_entries(&context, repo.as_path(), repo.join(".git/index").as_path(), asked.as_slice()).expect("the index reads");
    let from_the_head = head_entries(&context, repo.as_path(), asked.as_slice()).expect("the head reads");
    assert_eq!(from_the_tree.keys().collect::<Vec<&Vec<u8>>>(), vec![&asked[0]], "the tree is read at the path asked about");
    assert_eq!(from_the_index.keys().collect::<Vec<&Vec<u8>>>(), vec![&asked[0]], "the index is read at the path asked about");
    assert_eq!(from_the_head.keys().collect::<Vec<&Vec<u8>>>(), vec![&asked[0]], "the head is read at the path asked about");
    remove(root.as_path());
}

/// A shell that turned pathspec magic off as well leaves Git refusing both settings together,
/// and Git 2.56.0 answers "global 'literal' pathspec setting is incompatible with all other global pathspec settings":
/// the reads must still answer,
/// or a landing started from that shell cannot settle the record it owes.
#[test]
fn the_entries_of_a_path_are_read_under_an_inherited_literal_mode() {
    let (root, repo) = repository_with_a_case_twin("owed-entries-literal-mode");
    let context: crate::transaction_git::GitContext = inherited_literal_and_case_insensitive_pathspec_context();
    let asked: [Vec<u8>; 1] = [b"a.txt".to_vec()];
    let from_the_tree = tree_entries(&context, repo.as_path(), "HEAD", asked.as_slice()).expect("the tree reads");
    let from_the_index = index_entries(&context, repo.as_path(), repo.join(".git/index").as_path(), asked.as_slice()).expect("the index reads");
    let from_the_head = head_entries(&context, repo.as_path(), asked.as_slice()).expect("the head reads");
    assert_eq!(from_the_tree.keys().collect::<Vec<&Vec<u8>>>(), vec![&asked[0]], "the tree is read at the path asked about");
    assert_eq!(from_the_index.keys().collect::<Vec<&Vec<u8>>>(), vec![&asked[0]], "the index is read at the path asked about");
    assert_eq!(from_the_head.keys().collect::<Vec<&Vec<u8>>>(), vec![&asked[0]], "the head is read at the path asked about");
    remove(root.as_path());
}
