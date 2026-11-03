//! Controls for the second pass over a merge from objects: the attribute source follows the caller's own, and a tree comes
//!  back only when every conflicted file resolved, with the other files of the tree untouched and no scratch index left.

use super::*;
use crate::merge_second_pass::ContextRunner;
use crate::merge_second_pass::tests::{Change, World, corpus, merge_tree_of, runner, world};
use crate::test_support::{git_context, git_text, remove};
use crate::transaction_git::GitContext;
use std::path::PathBuf;

/// The variable is added when the environment has none, and left alone when the person set one.
#[test]
fn the_attribute_source_is_added_only_where_none_is_set() {
    let none: Vec<(OsString, OsString)> = vec![(OsString::from("HOME"), OsString::from("/home/x"))];
    assert_eq!(attribute_source_variables("abc123", none.as_slice()), vec![(OsString::from(ATTRIBUTE_SOURCE_VARIABLE), OsString::from("abc123"))]);
    let set: Vec<(OsString, OsString)> = vec![(OsString::from(ATTRIBUTE_SOURCE_VARIABLE), OsString::from("main"))];
    assert!(attribute_source_variables("abc123", set.as_slice()).is_empty());
}

/// Every conflicted file resolves: the tree holds the merged bytes, every other path is as merge-tree wrote it, and the
/// scratch index is gone.
#[test]
fn a_tree_comes_back_with_the_merged_files_and_nothing_else_changed() {
    let world: World = world(
        "second-pass-tree",
        &[("a.md", corpus("md1b", "base").as_slice()), ("clean.txt", b"one\n"), ("sub/deep.md", corpus("md1b", "base").as_slice())],
        &[("a.md", Change::Write(corpus("md1b", "ours").as_slice())), ("sub/deep.md", Change::Write(corpus("md1b", "ours").as_slice()))],
        &[("a.md", Change::Write(corpus("md1b", "theirs").as_slice())), ("clean.txt", Change::Write(b"two\n")), ("sub/deep.md", Change::Write(corpus("md1b", "theirs").as_slice()))],
    );
    let (tree, conflicts) = merge_tree_of(&world);
    let context: GitContext = git_context();
    let common: PathBuf = world.repo.join(".git");
    let resolved: Resolved = resolve_tree(&runner(&context, world.repo.as_path()), common.as_path(), tree.as_str(), conflicts.as_slice()).expect("every file resolves");
    assert_eq!(resolved.merged.len(), 2);
    for path in ["a.md", "sub/deep.md"] {
        let blob: String = git_text(world.repo.as_path(), &["show", format!("{}:{path}", resolved.tree).as_str()]) + "\n";
        assert_eq!(blob.as_bytes(), corpus("md1b", "word.out").as_slice(), "{path}");
    }
    assert_eq!(git_text(world.repo.as_path(), &["show", format!("{}:clean.txt", resolved.tree).as_str()]), "two");
    let leftovers: Vec<String> = std::fs::read_dir(common.join("cli-git-scratch")).expect("the scratch directory").filter_map(|entry| { return entry.ok()?.file_name().to_str().map(String::from); }).filter(|name| { return name.starts_with("second-pass-"); }).collect();
    assert_eq!(leftovers, Vec::<String>::new());
    remove(world.root.as_path());
}

/// One file that does not resolve keeps the whole merge conflicted, although the other resolves.
#[test]
fn one_file_left_over_returns_nothing() {
    let world: World = world(
        "second-pass-tree-left",
        &[("a.md", corpus("md1b", "base").as_slice()), ("b.md", corpus("md2", "base").as_slice())],
        &[("a.md", Change::Write(corpus("md1b", "ours").as_slice())), ("b.md", Change::Write(corpus("md2", "ours").as_slice()))],
        &[("a.md", Change::Write(corpus("md1b", "theirs").as_slice())), ("b.md", Change::Write(corpus("md2", "theirs").as_slice()))],
    );
    let (tree, conflicts) = merge_tree_of(&world);
    assert_eq!(conflicts.len(), 2);
    let context: GitContext = git_context();
    let common: PathBuf = world.repo.join(".git");
    assert_eq!(resolve_tree(&runner(&context, world.repo.as_path()), common.as_path(), tree.as_str(), conflicts.as_slice()), None);
    remove(world.root.as_path());
}

/// A runner that cannot start Git ends in nothing, not in a panic.
#[test]
fn a_git_that_cannot_start_returns_nothing() {
    let world: World = world(
        "second-pass-tree-nogit",
        &[("a.md", corpus("md1b", "base").as_slice())],
        &[("a.md", Change::Write(corpus("md1b", "ours").as_slice()))],
        &[("a.md", Change::Write(corpus("md1b", "theirs").as_slice()))],
    );
    let (tree, conflicts) = merge_tree_of(&world);
    let mut context: GitContext = git_context();
    context.real_git = PathBuf::from("/nonexistent/git");
    let common: PathBuf = world.repo.join(".git");
    let broken: ContextRunner<'_> = runner(&context, world.repo.as_path());
    assert_eq!(resolve_tree(&broken, common.as_path(), tree.as_str(), conflicts.as_slice()), None);
    remove(world.root.as_path());
}
