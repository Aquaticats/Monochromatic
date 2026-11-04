//! What:
//!  Disposable-repository controls for what a staged version is compared with and which index it reads.
//! Why:
//!  A repository without commits,
//!  a SHA-256 repository and a private index file each
//!      change the listing command's inputs,
//!  and each must still list exactly what is staged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const version = await store.version({ kind: 'staged-against-head' }); // in a repository with no commit
//! ```
#![cfg(unix)]

/// Import the store under test,
///  its sibling fixtures and shared fixtures.
use super::CandidateStore;
use super::tests::{at, paths, store, write};
use crate::candidate_record::CandidateChange;
use crate::candidate_version::{Candidate, CandidateSource, CandidateVersion};
use crate::test_support::{REAL_GIT, fixture, git, remove, repository};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Without any commit every index entry is an addition,
///  listed against the empty tree.
#[test]
fn unborn_head_lists_every_index_entry_as_added() {
    let root: PathBuf = fixture("store-unborn");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("repository directory");
    git(
        repo.as_path(),
        &["init", "--quiet", "--initial-branch=main"],
    );
    write(repo.as_path(), b"first.txt", b"first\n");
    write(repo.as_path(), b"second\xff.txt", b"");
    git(repo.as_path(), &["add", "--all"]);
    let mut subject: CandidateStore = store(repo.as_path());
    let version: Rc<CandidateVersion> = subject
        .version(&CandidateSource::StagedAgainstHead)
        .expect("staged version");
    assert_eq!(
        paths(&version),
        [b"first.txt".to_vec(), b"second\xff.txt".to_vec()]
    );
    for candidate in version.candidates() {
        assert_eq!(candidate.change, CandidateChange::Added);
    }
    assert_eq!(
        &subject.bytes(at(&version, b"first.txt")).expect("bytes")[..],
        b"first\n"
    );
    remove(root.as_path());
}

/// A SHA-256 repository works the same way:
///  64-digit names,
///  and its own empty tree when unborn.
#[test]
fn sha256_repository_is_listed_and_read() {
    let root: PathBuf = fixture("store-sha256");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("repository directory");
    git(
        repo.as_path(),
        &[
            "init",
            "--quiet",
            "--initial-branch=main",
            "--object-format=sha256",
        ],
    );
    write(repo.as_path(), b"q.txt", b"sha256 content\n");
    git(repo.as_path(), &["add", "--all"]);
    let mut subject: CandidateStore = store(repo.as_path());
    let version: Rc<CandidateVersion> = subject
        .version(&CandidateSource::StagedAgainstHead)
        .expect("staged version");
    let candidate: &Candidate = at(&version, b"q.txt");
    assert_eq!(
        candidate.object.as_ref().expect("object").as_str().len(),
        64
    );
    assert_eq!(
        &subject.bytes(candidate).expect("bytes")[..],
        b"sha256 content\n"
    );
    remove(root.as_path());
}

/// The environment overlay reaches the listing:
///  a private index file is the one that is listed.
#[test]
fn overlay_selects_a_private_index() {
    let root: PathBuf = fixture("store-private-index");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), b"real.txt", b"in the real index\n");
    git(repo.as_path(), &["add", "real.txt"]);
    let private_index: PathBuf = root.join("private-index");
    std::fs::copy(repo.join(".git/index"), &private_index).expect("copy index");
    write(
        repo.as_path(),
        b"private.txt",
        b"only in the private index\n",
    );
    let add: std::process::Output = std::process::Command::new(REAL_GIT)
        .current_dir(&repo)
        .env("GIT_INDEX_FILE", &private_index)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .args(["add", "private.txt"])
        .output()
        .expect("stage into the private index");
    assert!(
        add.status.success(),
        "{}",
        String::from_utf8_lossy(&add.stderr)
    );
    let mut private_store: CandidateStore = CandidateStore::new(
        Path::new(REAL_GIT),
        &[OsString::from("-C"), repo.clone().into_os_string()],
        &[(
            OsString::from("GIT_INDEX_FILE"),
            private_index.clone().into_os_string(),
        )],
    );
    let private_version: Rc<CandidateVersion> = private_store
        .version(&CandidateSource::StagedAgainstHead)
        .expect("private version");
    assert_eq!(
        paths(&private_version),
        [b"private.txt".to_vec(), b"real.txt".to_vec()]
    );
    assert_eq!(
        &private_store
            .bytes(at(&private_version, b"private.txt"))
            .expect("bytes")[..],
        b"only in the private index\n"
    );
    let mut real_store: CandidateStore = store(repo.as_path());
    let real_version: Rc<CandidateVersion> = real_store
        .version(&CandidateSource::StagedAgainstHead)
        .expect("real version");
    assert_eq!(paths(&real_version), [b"real.txt".to_vec()]);
    remove(root.as_path());
}

/// An intent-to-add entry is listed as an addition of the empty blob,
///  which Git stored when the intent was recorded.
#[test]
fn intent_to_add_entry_is_an_empty_addition() {
    let root: PathBuf = fixture("store-intent");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), b"intent.txt", b"not staged yet\n");
    git(repo.as_path(), &["add", "--intent-to-add", "intent.txt"]);
    let mut subject: CandidateStore = store(repo.as_path());
    let version: Rc<CandidateVersion> = subject
        .version(&CandidateSource::StagedAgainstHead)
        .expect("staged version");
    assert_eq!(paths(&version), [b"intent.txt".to_vec()]);
    let candidate: &Candidate = at(&version, b"intent.txt");
    assert_eq!(candidate.change, CandidateChange::Added);
    // The worktree content is not what the index names: the entry's object is the empty blob.
    assert!(subject.bytes(candidate).expect("empty blob").is_empty());
    remove(root.as_path());
}
