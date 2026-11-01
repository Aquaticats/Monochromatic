//! Controls for the Git reads against real Git in a fixture repository.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::path::PathBuf;

/// The batch parser reads by size, reports a missing name as absent, and rejects malformed output.
#[test]
fn batch_output_is_read_by_size_with_missing_names_absent() {
    let names: Vec<String> = vec![String::from("a.ts"), String::from("gone.ts"), String::from("b.rs")];
    let output: &[u8] = b"oid1 blob 5\nx\ny\nz\nHEAD:gone.ts missing\noid2 blob 0\n\n";
    let texts: BTreeMap<String, String> = parse_batch(output, names.as_slice()).expect("parsed");
    assert_eq!(texts.get("a.ts").map(String::as_str), Some("x\ny\nz"));
    assert_eq!(texts.get("b.rs").map(String::as_str), Some(""));
    assert!(!texts.contains_key("gone.ts"));
    assert!(parse_batch(b"", names.as_slice()).is_err());
    assert!(parse_batch(b"oid1 tree 3\nabc\n", &names[..1]).is_err());
    assert!(parse_batch(b"oid1 blob 9\nabc\n", &names[..1]).is_err());
    assert!(parse_batch(b"oid1 blob x\nabc\n", &names[..1]).is_err());
}

/// A commit that changes, adds and deletes code files reads its own patch and the surviving files' texts.
#[test]
fn a_landed_commit_is_read_in_two_starts_worth_of_data() {
    let parent: PathBuf = fixture("source-read");
    let root: PathBuf = repository(parent.as_path(), "repo");
    std::fs::write(root.join("keep.ts"), "one\ntwo\n").expect("write");
    std::fs::write(root.join("gone.rs"), "bye\n").expect("write");
    git(root.as_path(), &["add", "--", "keep.ts", "gone.rs"]);
    git(root.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(root.join("keep.ts"), "one\nchanged\n").expect("write");
    std::fs::write(root.join("new.mjs"), "fresh\n").expect("write");
    git(root.as_path(), &["rm", "--quiet", "--", "gone.rs"]);
    git(root.as_path(), &["add", "--", "keep.ts", "new.mjs"]);
    git(root.as_path(), &["commit", "--quiet", "--message=change"]);
    let sha: String = git_text(root.as_path(), &["rev-parse", "HEAD"]);
    let names: Vec<String> = git_text(root.as_path(), &["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"]).split('\n').map(String::from).collect();
    let commit: LandedCommit = read_landed_commit(&git_context(), root.as_path(), sha.as_str(), "change", names.as_slice()).expect("read");
    assert_eq!(commit.patch, git_text(root.as_path(), &["show", "--format=", "--patch", "HEAD"]) + "\n");
    assert_eq!(commit.contents.get("keep.ts").map(String::as_str), Some("one\nchanged\n"));
    assert_eq!(commit.contents.get("new.mjs").map(String::as_str), Some("fresh\n"));
    assert!(!commit.contents.contains_key("gone.rs"), "a deleted file has no new-side text");
    assert_eq!(commit.sha, sha);
    remove(parent.as_path());
}

/// A commit name Git does not know is a failure that names the command.
#[test]
fn an_unknown_commit_fails_with_the_command_named() {
    let parent: PathBuf = fixture("source-unknown");
    let root: PathBuf = repository(parent.as_path(), "repo");
    let missing: String = "0".repeat(40);
    let failure: String = read_landed_commit(&git_context(), root.as_path(), missing.as_str(), "s", &[String::from("a.ts")]).expect_err("unknown");
    assert!(failure.contains("git show"), "{failure}");
    remove(parent.as_path());
}
