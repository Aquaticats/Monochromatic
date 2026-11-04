//! What:
//!  Disposable-repository controls for listings that must be refused.
//! Why:
//!  A conflicted index,
//!  a command that fails,
//!  a program that cannot start and output
//!      that is not an object name each leave paths unchecked if they are tolerated.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await expect(store.version(source)).rejects.toMatchObject({ failure: 'git-failed' });
//! ```
#![cfg(unix)]

/// Import the store under test,
///  its sibling fixtures and shared fixtures.
use super::CandidateStore;
use super::tests::{rev_parse, store, store_with, write};
use crate::candidate_error::{CandidateError, CandidateFailure};
use crate::candidate_object::{ObjectId, parse_object_id};
use crate::candidate_version::CandidateSource;
use crate::test_support::{executable, fixture, git, remove, repository};
use std::path::{Path, PathBuf};

/// A conflicted index is refused as a whole.
#[test]
fn unmerged_index_is_refused() {
    let root: PathBuf = fixture("store-unmerged");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), b"f.txt", b"base\n");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["checkout", "--quiet", "-b", "side"]);
    write(repo.as_path(), b"f.txt", b"side\n");
    git(
        repo.as_path(),
        &["commit", "--quiet", "--all", "--message=side"],
    );
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    write(repo.as_path(), b"f.txt", b"main\n");
    git(
        repo.as_path(),
        &["commit", "--quiet", "--all", "--message=main"],
    );
    let merge: std::process::Output =
        crate::test_support::git_output(repo.as_path(), &["merge", "--quiet", "side"]);
    assert!(!merge.status.success(), "the merge must conflict");
    let mut subject: CandidateStore = store(repo.as_path());
    let error: CandidateError = subject
        .version(&CandidateSource::StagedAgainstHead)
        .expect_err("conflicted index");
    assert_eq!(error.failure, CandidateFailure::UnmergedPath);
    remove(root.as_path());
}

/// Listing failures are typed:
///  an unstartable program,
///  a failing command,
///  and a location that is no repository.
#[test]
fn listing_failures_are_typed() {
    let root: PathBuf = fixture("store-failures");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let head: ObjectId = rev_parse(repo.as_path(), "HEAD");
    let absent: ObjectId = parse_object_id(&[b'1'; 40]).expect("name");
    let mut unstartable: CandidateStore =
        store_with(Path::new("/nonexistent-directory/git"), repo.as_path());
    let listing: CandidateError = unstartable
        .version(&CandidateSource::StagedAgainstCommit(head))
        .expect_err("unstartable listing");
    assert_eq!(listing.failure, CandidateFailure::GitNotStarted);
    assert!(
        listing.message.starts_with(
            "cli-git could not list candidate files: git diff-index could not be started: "
        ),
        "{listing}"
    );
    assert_eq!(
        unstartable
            .version(&CandidateSource::StagedAgainstHead)
            .expect_err("unstartable reader")
            .failure,
        CandidateFailure::GitNotStarted
    );
    let mut subject: CandidateStore = store(repo.as_path());
    let failed: CandidateError = subject
        .version(&CandidateSource::Committed(absent.clone()))
        .expect_err("absent commit");
    assert_eq!(failed.failure, CandidateFailure::GitFailed);
    assert!(
        failed
            .message
            .starts_with("cli-git could not list candidate files: git diff-tree failed: fatal: "),
        "{failed}"
    );
    assert_eq!(
        subject
            .version(&CandidateSource::StagedAgainstCommit(absent))
            .expect_err("absent baseline")
            .failure,
        CandidateFailure::GitFailed
    );
    // A failed listing is not remembered as an empty version.
    assert!(subject.versions.is_empty());
    let plain: PathBuf = root.join("plain");
    std::fs::create_dir(&plain).expect("plain directory");
    let mut outside: CandidateStore = store(plain.as_path());
    assert_eq!(
        outside
            .version(&CandidateSource::StagedAgainstHead)
            .expect_err("no repository")
            .failure,
        CandidateFailure::ReaderEnded
    );
    remove(root.as_path());
}

/// An empty-tree name that is not an object name is refused instead of passed to the listing.
#[test]
fn unusable_empty_tree_name_is_refused() {
    let root: PathBuf = fixture("store-empty-tree");
    let program: PathBuf = root.join("git");
    // The stand-in reports an unborn `HEAD`, waits for the reader to be closed, and prints no object name.
    executable(
        program.as_path(),
        b"#!/bin/sh\ncase \"$1\" in\n  cat-file) read -r request; printf 'HEAD missing\\n'; read -r rest ;;\n  hash-object) printf 'not-a-name\\n' ;;\n  *) exit 9 ;;\nesac\nexit 0\n",
    );
    let mut subject: CandidateStore = CandidateStore::new(program.as_path(), &[], &[]);
    let error: CandidateError = subject
        .version(&CandidateSource::StagedAgainstHead)
        .expect_err("unusable empty tree");
    assert_eq!(error.failure, CandidateFailure::ListingMalformed);
    assert!(
        error
            .message
            .contains("git hash-object did not print the empty tree's object name"),
        "{error}"
    );
    remove(root.as_path());
}
