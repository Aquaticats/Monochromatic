//! Controls for capture history on a real repository with real capture records:
//!  first-parent listing with and without a base, the decisions per record outcome, and malformed records.

use super::*;
use crate::capture_records::{
    CAPTURED_FILENAME, CapturedRecord, LandedCaptureRecord, encode_captured_record, encode_landed_capture_record,
};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::collections::BTreeSet;
use std::path::PathBuf;

/// Revision text of a commit or blob, trimmed.
fn rev(repo: &Path, spec: &str) -> String {
    return git_text(repo, &["rev-parse", spec]).trim().to_string();
}

/// A repository with a base commit `a.txt` and one landed commit changing `a.txt`; returns the base and landed IDs.
fn history(root: &Path) -> (PathBuf, String, String) {
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(repo.join("a.txt"), "1\n").expect("a");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = rev(repo.as_path(), "HEAD");
    std::fs::write(repo.join("a.txt"), "2\n").expect("a landed");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=landed"]);
    let landed: String = rev(repo.as_path(), "HEAD");
    return (repo, base, landed);
}

/// Writes the transaction's `captured.json` for worktree `w` at `sequence` that read `paths`.
fn write_capture(transaction: &Path, sequence: i64, paths: &[&str]) {
    std::fs::create_dir_all(transaction).expect("transaction directory");
    let record: CapturedRecord = CapturedRecord {
        worktree_id: String::from("w"),
        sequence,
        next_sequence_before_base: sequence,
        worktree_paths: paths.iter().map(|path| { return String::from(*path); }).collect(),
    };
    std::fs::write(transaction.join(CAPTURED_FILENAME), encode_captured_record(&record)).expect("captured.json");
}

/// Writes the landed-capture record of `commit` for worktree `w` at `sequence` that read `paths`.
fn write_landed(git_dir: &Path, commit: &str, sequence: i64, paths: &[&str]) {
    let directory: PathBuf = landed_record_directory(git_dir);
    std::fs::create_dir_all(&directory).expect("landed directory");
    let record: LandedCaptureRecord = LandedCaptureRecord {
        commit: String::from(commit),
        transaction_id: String::from("0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10"),
        worktree_id: String::from("w"),
        sequence,
        next_sequence_after_landing: sequence + 1,
        worktree_paths: paths.iter().map(|path| { return String::from(*path); }).collect(),
    };
    std::fs::write(directory.join(format!("{commit}.json")), encode_landed_capture_record(&record)).expect("landed record");
}

/// The first-parent listing lists the landed commit with its path, and an unborn base lists every commit.
#[test]
fn the_listing_follows_the_first_parent_line() {
    let root: PathBuf = fixture("replay-capture-history-listing");
    let (repo, base, landed) = history(root.as_path());
    let since_base: Vec<LandedChange> = list_landed_changes(&git_context(), repo.join(".git").as_path(), repo.as_path(), Some(base.as_str()), landed.as_str()).expect("since base");
    assert_eq!(since_base.len(), 1);
    assert_eq!(since_base[0].commit, landed);
    assert_eq!(since_base[0].paths, BTreeSet::from([String::from("a.txt")]));
    let unborn: Vec<LandedChange> = list_landed_changes(&git_context(), repo.join(".git").as_path(), repo.as_path(), None, landed.as_str()).expect("unborn");
    // The empty `initial` commit of the fixture changes nothing, so diff-tree prints no record and it is left out.
    assert_eq!(unborn.len(), 2);
    remove(root.as_path());
}

/// Each outcome of the decision: prepared later, landed later, no capture record, and a path the transaction did not read.
#[test]
fn the_decision_follows_the_recorded_sequences() {
    let root: PathBuf = fixture("replay-capture-history-decision");
    let (repo, base, landed) = history(root.as_path());
    let git_dir: PathBuf = repo.join(".git");
    let transaction: PathBuf = root.join("transaction");
    let paths: Vec<String> = vec![String::from("a.txt"), String::from("b.txt")];
    let decide = |current: &str| -> BTreeMap<String, CaptureOrderDecision> {
        return decide_shared_paths(
            &git_context(),
            git_dir.as_path(),
            repo.as_path(),
            git_dir.as_path(),
            transaction.as_path(),
            Some(base.as_str()),
            current,
            &paths,
        )
        .expect("decision");
    };
    assert!(decide(landed.as_str()).is_empty(), "no captured.json decides nothing");
    write_capture(transaction.as_path(), 5, &["a.txt", "b.txt"]);
    assert_eq!(decide(landed.as_str()).get("a.txt"), Some(&CaptureOrderDecision::Unordered), "no landed record");
    write_landed(git_dir.as_path(), landed.as_str(), 2, &["a.txt"]);
    let later: BTreeMap<String, CaptureOrderDecision> = decide(landed.as_str());
    assert_eq!(later.get("a.txt"), Some(&CaptureOrderDecision::Prepared));
    assert_eq!(later.get("b.txt"), Some(&CaptureOrderDecision::Unordered));
    write_capture(transaction.as_path(), 1, &["a.txt", "b.txt"]);
    assert_eq!(decide(landed.as_str()).get("a.txt"), Some(&CaptureOrderDecision::Landed));
    remove(root.as_path());
}

/// A landed-capture record that does not parse fails the decision rather than leaving the path unordered.
#[test]
fn a_malformed_landed_record_fails_the_decision() {
    let root: PathBuf = fixture("replay-capture-history-malformed");
    let (repo, base, landed) = history(root.as_path());
    let git_dir: PathBuf = repo.join(".git");
    let transaction: PathBuf = root.join("transaction");
    write_capture(transaction.as_path(), 5, &["a.txt"]);
    let directory: PathBuf = landed_record_directory(git_dir.as_path());
    std::fs::create_dir_all(&directory).expect("landed directory");
    std::fs::write(directory.join(format!("{landed}.json")), b"not json").expect("malformed record");
    let outcome = decide_shared_paths(
        &git_context(),
        git_dir.as_path(),
        repo.as_path(),
        git_dir.as_path(),
        transaction.as_path(),
        Some(base.as_str()),
        landed.as_str(),
        &[String::from("a.txt")],
    );
    assert!(matches!(outcome, Err(CaptureHistoryError::Record(_))));
    remove(root.as_path());
}

/// The revision helper and the blob-free listing agree with Git's own answer for the landed commit.
#[test]
fn the_fixture_commits_are_the_expected_revisions() {
    let root: PathBuf = fixture("replay-capture-history-revisions");
    let (repo, base, landed) = history(root.as_path());
    assert_eq!(rev(repo.as_path(), "HEAD^"), base);
    assert_eq!(rev(repo.as_path(), "HEAD"), landed);
    remove(root.as_path());
}
