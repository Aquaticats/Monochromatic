//! Controls for the selected elements an explicit-path commit names that match nothing:
//! which ones are named,
//! which are left to Git's own text,
//! and what the finding says.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::transaction_journal::TransactionMode;
use std::path::{Path, PathBuf};

/// A repository with `a.txt` and `tree/b.txt` committed,
/// and the revision its index holds.
fn tracked_repository(name: &str) -> (PathBuf, PathBuf, String) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    std::fs::create_dir(repo.join("tree")).expect("tree");
    std::fs::write(repo.join("tree").join("b.txt"), "b\n").expect("b");
    git(repo.as_path(), &["add", "--", "a.txt", "tree/b.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    return (root, repo, base);
}

/// The elements of an explicit-path selection that match nothing,
/// asked the way the preparation asks it.
fn unmatched(repo: &Path, base: &str, elements: &[&str]) -> Vec<Vec<u8>> {
    let specs: Vec<OsString> = elements.iter().map(OsString::from).collect();
    let selection: IndexSelection<'_> = IndexSelection {
        mode: TransactionMode::ExplicitPath,
        pathspecs: specs.as_slice(),
        pathspec_file: None,
        pathspec_file_nul: false,
        stage_into_index: false,
        stage_tracked_changes: false,
        base_revision: base,
    };
    return named_unmatched_elements(&git_context(), repo, repo.join(".git").join("index").as_path(), &selection)
        .expect("the question is answered");
}

/// The words of one element list.
fn shown(elements: &[Vec<u8>]) -> Vec<String> {
    return elements.iter().map(|element| { return String::from_utf8_lossy(element).into_owned(); }).collect();
}

/// A message written after the paths is read as paths,
/// so `-m` and every word after it are named (the recorded shape of
/// `doc/planning/cli-git-commits-that-stopped-after-their-add-steps.md`).
#[test]
fn the_words_of_a_message_written_after_the_paths_are_named() {
    let (root, repo, base) = tracked_repository("unmatched-message-words");
    let found: Vec<Vec<u8>> = unmatched(repo.as_path(), base.as_str(), &["a.txt", "-m", "one two"]);
    assert_eq!(shown(found.as_slice()), vec![String::from("-m"), String::from("one two")]);
    remove(root.as_path());
}

/// A path the worktree does not hold and the index does not hold is named too,
/// because native Git refuses that pathspec as well.
#[test]
fn a_path_the_worktree_and_the_index_do_not_hold_is_named() {
    let (root, repo, base) = tracked_repository("unmatched-missing-path");
    let found: Vec<Vec<u8>> = unmatched(repo.as_path(), base.as_str(), &["a.txt", "gone.txt"]);
    assert_eq!(shown(found.as_slice()), vec![String::from("gone.txt")]);
    remove(root.as_path());
}

/// A selection the index holds names nothing,
/// and a file the worktree holds is left to the staging that asks the worktree about it.
#[test]
fn a_selection_the_index_holds_names_nothing() {
    let (root, repo, base) = tracked_repository("unmatched-none");
    std::fs::write(repo.join("new.txt"), "new\n").expect("new");
    assert!(unmatched(repo.as_path(), base.as_str(), &["a.txt", "tree"]).is_empty(), "the index holds both");
    assert!(unmatched(repo.as_path(), base.as_str(), &["."]).is_empty(), "a directory that holds entries matches");
    assert!(unmatched(repo.as_path(), base.as_str(), &["new.txt"]).is_empty(), "a file the worktree holds is its own question");
    assert!(unmatched(repo.as_path(), base.as_str(), &[]).is_empty(), "an empty selection stages nothing");
    remove(root.as_path());
}

/// A pathspec file this reader does not take is Git's to read,
/// so nothing is named from it.
#[test]
fn a_pathspec_file_the_reader_declines_names_nothing() {
    let (root, repo, base) = tracked_repository("unmatched-pathspec-file");
    let absent: PathBuf = repo.join("no-such-pathspec-file");
    let selection: IndexSelection<'_> = IndexSelection {
        mode: TransactionMode::ExplicitPath,
        pathspecs: &[],
        pathspec_file: Some(absent.as_path()),
        pathspec_file_nul: false,
        stage_into_index: false,
        stage_tracked_changes: false,
        base_revision: base.as_str(),
    };
    assert!(
        named_unmatched_elements(&git_context(), repo.as_path(), repo.join(".git").join("index").as_path(), &selection)
            .expect("the question is answered")
            .is_empty()
    );
    remove(root.as_path());
}

/// The finding names every element and both ways forward,
/// and says where an option belongs,
/// without asking whether the commit landed.
#[test]
fn the_message_names_every_element_and_both_ways_forward() {
    let message: String = named_unmatched_message(&[b"-m".to_vec(), b"one two".to_vec()]);
    assert!(message.contains("[\"-m\",\"one two\"]"), "{message}");
    assert!(message.contains("this commit was not made and nothing was staged by it"), "{message}");
    assert!(message.contains("git commit -m <message> -- <paths>"), "{message}");
    assert!(message.contains("Every element after `--` is a path and never an option"), "{message}");
    assert!(!message.contains("git log --oneline"), "the transaction-failure hint asks about a landing that cannot exist: {message}");
}
