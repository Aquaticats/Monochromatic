//! Controls for the merge message pieces.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::path::PathBuf;

/// The description line of each kind of merged name.
#[test]
fn each_kind_of_name_has_its_own_description() {
    let oid: &str = "0123456789012345678901234567890123456789";
    assert_eq!(
        describe_merge_source("feature", "refs/heads/feature", oid),
        Some(format!("{oid}\t\tbranch 'feature' of .\n"))
    );
    assert_eq!(
        describe_merge_source("v1", "refs/tags/v1", oid),
        Some(format!("{oid}\t\ttag 'v1' of .\n"))
    );
    assert_eq!(
        describe_merge_source("origin/main", "refs/remotes/origin/main", oid),
        Some(format!("{oid}\t\tremote-tracking branch 'origin/main' of .\n"))
    );
    assert_eq!(
        describe_merge_source("0123abc", "", oid),
        Some(format!("{oid}\t\tcommit '0123abc'\n"))
    );
}

/// A name that needs another wording is left to native Git.
#[test]
fn names_with_their_own_wording_are_not_described() {
    let oid: &str = "0123456789012345678901234567890123456789";
    for reference in ["feature^", "feature~2", "HEAD@{1}", "main:path", "not hex", ""] {
        assert_eq!(describe_merge_source(reference, "", oid), None, "{reference:?}");
    }
}

/// The default subject is what Git itself writes for the same merge.
#[test]
fn the_default_message_is_the_one_git_writes() {
    let parent: PathBuf = fixture("merge-message-default");
    let repo: PathBuf = repository(parent.as_path(), "repo");
    git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("f.txt"), b"f\n").expect("file");
    git(repo.as_path(), &["add", "--", "f.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=feature work"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("m.txt"), b"m\n").expect("file");
    git(repo.as_path(), &["add", "--", "m.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=main work"]);
    let oid: String = git_text(repo.as_path(), &["rev-parse", "feature"]);
    let line: String = describe_merge_source("feature", "refs/heads/feature", oid.as_str()).expect("branch");
    let mut context = git_context();
    context.global_prefix = vec!["-C".into(), repo.clone().into_os_string()];
    let message: Vec<u8> = default_merge_message(&context, line.as_str(), None).expect("message");
    // The shortlog lists commits the branch does not yet contain, so it is read before the merge.
    let shortlog: Vec<u8> = default_merge_message(&context, line.as_str(), Some(true)).expect("message with log");
    assert!(String::from_utf8_lossy(&shortlog).contains("feature work"), "{shortlog:?}");
    git(repo.as_path(), &["merge", "--no-edit", "feature"]);
    let native: String = git_text(repo.as_path(), &["log", "-1", "--format=%B"]);
    assert_eq!(String::from_utf8(message).expect("text"), native);
    remove(parent.as_path());
}

/// Cleanup strips trailing space, collapses blank lines and ends in one line feed.
#[test]
fn cleanup_matches_a_commit_that_opens_no_editor() {
    let context = git_context();
    let cleaned: Vec<u8> = clean_message(&context, b"\n\nsubject  \n\n\n\nbody\t\n\n\n").expect("cleaned");
    assert_eq!(cleaned, b"subject\n\nbody\n".to_vec());
    assert_eq!(clean_message(&context, b"one").expect("cleaned"), b"one\n".to_vec());
}
