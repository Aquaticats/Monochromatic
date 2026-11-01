//! What: Disposable-repository controls for the `HEAD` tree listing.
//! Why: The listing is what a policy compares the candidate state with; it must name every
//!      file of `HEAD` with its mode, from any directory, and nothing before the first commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const records = await store.headTreeRecords(); // every file of HEAD
//! ```
#![cfg(unix)]

/// Import the store under test and its sibling fixtures.
use super::CandidateStore;
/// The store's fixtures: a store over real Git, an object name, and a worktree writer.
use super::tests::{rev_parse, store, write};
/// The mode a record carries.
use crate::candidate_object::{CandidateMode, ObjectId};
/// The record a pushed tree lists.
use crate::candidate_record::CandidateRecord;
/// The record under test.
use crate::candidate_tree::TreeRecord;
/// Shared fixture directories and real Git.
use crate::test_support::{fixture, git, remove, repository};
/// Unix permission bits, for the executable file.
use std::os::unix::fs::PermissionsExt;
/// Owned filesystem paths.
use std::path::PathBuf;

/// Every file of `HEAD` is listed with its mode and object, sorted as Git sorts, nested
/// directories included; staged and worktree changes after the commit are not.
#[test]
fn lists_every_file_of_head_with_its_mode() {
    let root: PathBuf = fixture("store-head-tree");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::create_dir_all(repo.join("deep/er")).expect("directories");
    write(repo.as_path(), b"deep/er/file.txt", b"nested\n");
    write(repo.as_path(), b"run.sh", b"#!/bin/sh\n");
    std::fs::set_permissions(repo.join("run.sh"), std::fs::Permissions::from_mode(0o755))
        .expect("executable");
    std::os::unix::fs::symlink("run.sh", repo.join("link")).expect("link");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=files"]);
    write(repo.as_path(), b"later.txt", b"after the commit\n");
    git(repo.as_path(), &["add", "later.txt"]);
    let mut subject: CandidateStore = store(repo.join("deep").as_path());
    let records: Vec<TreeRecord> = subject.head_tree_records().expect("tree");
    assert_eq!(
        records,
        vec![
            TreeRecord {
                mode: CandidateMode::Regular,
                object: rev_parse(repo.as_path(), "HEAD:deep/er/file.txt"),
                path: b"deep/er/file.txt".to_vec(),
            },
            TreeRecord {
                mode: CandidateMode::Symlink,
                object: rev_parse(repo.as_path(), "HEAD:link"),
                path: b"link".to_vec(),
            },
            TreeRecord {
                mode: CandidateMode::Executable,
                object: rev_parse(repo.as_path(), "HEAD:run.sh"),
                path: b"run.sh".to_vec(),
            },
        ]
    );
    remove(root.as_path());
}

/// Before the first commit there is no `HEAD`, so there are no records and no listing runs.
#[test]
fn unborn_head_has_no_records() {
    let root: PathBuf = fixture("store-head-tree-unborn");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("repository directory");
    git(
        repo.as_path(),
        &["init", "--quiet", "--initial-branch=main"],
    );
    write(repo.as_path(), b"staged.txt", b"staged\n");
    git(repo.as_path(), &["add", "staged.txt"]);
    let mut subject: CandidateStore = store(repo.as_path());
    assert_eq!(subject.head_tree_records().expect("tree"), Vec::new());
    remove(root.as_path());
}

/// A listing Git refuses is reported with Git's message.
#[test]
fn a_failed_listing_is_reported() {
    let root: PathBuf = fixture("store-head-tree-failed");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), b"file.txt", b"content\n");
    git(repo.as_path(), &["add", "file.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=file"]);
    let head: String =
        String::from_utf8(git(repo.as_path(), &["rev-parse", "HEAD:"]).stdout).expect("tree name");
    // Removing the commit's tree object makes `ls-tree` fail while `HEAD` still resolves.
    let tree: &str = head.trim();
    let object: PathBuf = repo.join(".git/objects").join(&tree[..2]).join(&tree[2..]);
    std::fs::remove_file(object).expect("remove the tree object");
    let mut subject: CandidateStore = store(repo.as_path());
    let error = subject.head_tree_records().expect_err("listing fails");
    assert!(error.message.contains("ls-tree"), "{}", error.message);
    remove(root.as_path());
}

/// A pushed tree lists a name that is not UTF-8 by its exact bytes.
#[test]
fn a_pushed_tree_lists_a_name_that_is_not_utf8_exactly() {
    let root: PathBuf = fixture("store-pushed-tree-bytes");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), b"bad\xff.txt", b"bytes\n");
    write(repo.as_path(), b"plain.txt", b"text\n");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=files"]);
    let tree: ObjectId = rev_parse(repo.as_path(), "HEAD^{tree}");
    let mut subject: CandidateStore = store(repo.as_path());
    let records: Vec<CandidateRecord> = subject.tree_records(&tree).expect("tree records");
    let paths: Vec<&[u8]> = records.iter().map(|record| { return record.path.as_slice(); }).collect();
    assert_eq!(paths, vec![b"bad\xff.txt".as_slice(), b"plain.txt".as_slice()]);
    remove(root.as_path());
}
