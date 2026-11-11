//! Controls for the two findings a preparation ends with before its private index is built:
//! the codes they carry,
//! the exit status,
//! and the selections that end it with neither.

use super::*;
use crate::commit_index_selection::IndexSelection;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::transaction_journal::TransactionMode;
use std::path::PathBuf;

/// A repository with `a.txt` committed and `*.js` ignored,
/// with `lib/_a.js` force-staged and `lib/_c.js` left untracked,
/// and its base revision.
fn ignored_repository(name: &str) -> (PathBuf, PathBuf, String) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(".gitignore"), "*.js\n").expect("ignore file");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    git(repo.as_path(), &["add", ".gitignore", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::create_dir(repo.join("lib")).expect("lib");
    std::fs::write(repo.join("lib").join("_a.js"), "a1\n").expect("_a");
    git(repo.as_path(), &["add", "--force", "lib/_a.js"]);
    std::fs::write(repo.join("lib").join("_c.js"), "c1\n").expect("_c");
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    return (root, repo, base);
}

/// The ending of an explicit-path commit of these elements,
/// asked the way the preparation asks it.
fn named_ending(repo: &Path, base: &str, pathspecs: &[&str]) -> Option<TransactionResult> {
    let specs: Vec<OsString> = pathspecs.iter().map(OsString::from).collect();
    let choice: IndexSelection<'_> = IndexSelection {
        mode: TransactionMode::ExplicitPath,
        pathspecs: specs.as_slice(),
        pathspec_file: None,
        pathspec_file_nul: false,
        stage_into_index: false,
        stage_tracked_changes: false,
        base_revision: base,
    };
    return named_ignored_ending(&git_context(), repo, repo.join(".git").join("index").as_path(), &choice)
        .expect("the question is answered");
}

/// The ending of an explicit-path commit of these elements,
/// asked for a staged deletion.
fn deletion_ending(repo: &Path, base: &str, pathspecs: &[&str]) -> Option<TransactionResult> {
    let specs: Vec<OsString> = pathspecs.iter().map(OsString::from).collect();
    return staged_deletion_ending(&git_context(), repo, repo, repo.join(".git").join("index").as_path(), base, specs.as_slice())
        .expect("the question is answered");
}

/// The ending of an explicit-path commit of these elements,
/// asked for an element that matches nothing.
fn unmatched_ending_of(repo: &Path, base: &str, pathspecs: &[&str]) -> Option<TransactionResult> {
    let specs: Vec<OsString> = pathspecs.iter().map(OsString::from).collect();
    let choice: IndexSelection<'_> = IndexSelection {
        mode: TransactionMode::ExplicitPath,
        pathspecs: specs.as_slice(),
        pathspec_file: None,
        pathspec_file_nul: false,
        stage_into_index: false,
        stage_tracked_changes: false,
        base_revision: base,
    };
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: repo.join(".git").join("index"),
        commit_index: repo.join(".git").join("index"),
        object_directory: repo.join(".git").join("objects"),
    };
    return unmatched_ending(&git_context(), repo, &indexes, &choice).expect("the question is answered");
}

/// A selection whose elements match nothing ends the preparation with its own code,
/// naming each element and where an option belongs.
#[test]
fn an_element_that_matches_nothing_ends_the_preparation_with_its_finding() {
    let (root, repo, base) = ignored_repository("selection-refusal-unmatched");
    let ending: TransactionResult = unmatched_ending_of(repo.as_path(), base.as_str(), &["a.txt", "-m", "one two"]).expect("the refusal");
    assert_eq!(finding(&ending), "explicit-path-commit/named-path-unmatched");
    let PolicyEvent::CoreFinding { message, .. } = &ending.events[0] else {
        panic!("a core finding");
    };
    assert!(message.contains("[\"-m\",\"one two\"]"), "{message}");
    assert!(message.contains("git commit -m <message> -- <paths>"), "{message}");
    remove(root.as_path());
}

/// A selection the private staging can take ends with nothing.
#[test]
fn a_selection_whose_elements_all_match_has_no_unmatched_ending() {
    let (root, repo, base) = ignored_repository("selection-refusal-unmatched-none");
    assert!(unmatched_ending_of(repo.as_path(), base.as_str(), &["a.txt", "lib"]).is_none());
    remove(root.as_path());
}

/// The one core finding an ending carries,
/// and the exit status it ends with.
fn finding(ending: &TransactionResult) -> String {
    assert_eq!(ending.exit_code, 1, "a selection refusal exits 1");
    assert_eq!(ending.landed_oid, None, "a selection refusal lands nothing");
    assert!(!ending.uses_lfs, "nothing landed, so nothing is pushed");
    assert_eq!(ending.events.len(), 1, "one finding explains the refusal");
    let PolicyEvent::CoreFinding { core_id, code, message } = &ending.events[0] else {
        panic!("a selection refusal is a core finding, not {:?}", ending.events[0]);
    };
    assert!(!message.is_empty(), "the finding says why");
    return format!("{core_id}/{code}");
}

/// A named file an ignore rule refuses ends the preparation with its own code,
/// naming the path.
#[test]
fn a_named_ignored_file_ends_the_preparation_with_its_finding() {
    let (root, repo, base) = ignored_repository("selection-refusal-ignored");
    let ending: TransactionResult = named_ending(repo.as_path(), base.as_str(), &["lib/_c.js"]).expect("the refusal");
    assert_eq!(finding(&ending), "explicit-path-commit/named-path-ignored");
    let PolicyEvent::CoreFinding { message, .. } = &ending.events[0] else {
        panic!("a core finding");
    };
    assert!(message.contains("lib/_c.js"), "{message}");
    remove(root.as_path());
}

/// A selection the private staging can take ends with nothing.
#[test]
fn a_selection_an_ignore_rule_leaves_alone_has_no_ending() {
    let (root, repo, base) = ignored_repository("selection-refusal-none");
    assert!(named_ending(repo.as_path(), base.as_str(), &["a.txt", "lib/_a.js"]).is_none());
    assert!(named_ending(repo.as_path(), base.as_str(), &["lib"]).is_none());
    remove(root.as_path());
}

/// A staged deletion whose path stays on disk ends the preparation with its own code.
#[test]
fn a_staged_deletion_whose_path_stays_on_disk_ends_the_preparation() {
    let (root, repo, base) = ignored_repository("selection-refusal-deletion");
    git(repo.as_path(), &["rm", "--quiet", "--cached", "--", "a.txt"]);
    let ending: TransactionResult = deletion_ending(repo.as_path(), base.as_str(), &["a.txt"]).expect("the refusal");
    assert_eq!(finding(&ending), "commit-only/staged-deletion-worktree-present");
    remove(root.as_path());
}

/// A selection with no staged deletion ends with nothing.
#[test]
fn a_selection_without_a_staged_deletion_has_no_ending() {
    let (root, repo, base) = ignored_repository("selection-refusal-no-deletion");
    assert!(deletion_ending(repo.as_path(), base.as_str(), &["a.txt"]).is_none());
    assert!(deletion_ending(repo.as_path(), base.as_str(), &[]).is_none());
    remove(root.as_path());
}
