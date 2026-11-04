//! What:
//!  Count the Git processes the candidate store starts,
//!  by a mechanism outside the store.
//! Why:
//!  The store exists so that reading N files does not start N processes.
//!  The count is
//!      taken by a counting stand-in every Git start must pass through,
//!  and a
//!      deliberately per-file reader proves that stand-in does see one start per file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // PATH=/fixture/bin:$PATH; readAll(200 files); expect(lines('/fixture/count')).toBe(2);
//! ```
#![cfg(unix)]

/// Import the store under test and shared fixtures.
use super::CandidateStore;
use super::tests::{at, store_with};
use crate::candidate_version::{CandidateSource, CandidateVersion};
use crate::test_support::{REAL_GIT, executable, fixture, git, remove, repository};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

/// Git processes one staged version may start,
///  however many files it lists and reads:
/// the long-lived object reader and one listing.
const STAGED_PROCESS_BOUND: usize = 2;

/// A counting `git`:
///  it appends one line per start to the count file,
///  then becomes real Git.
/// Every Git start that goes through this path is recorded by the operating system's
/// append,
///  whatever the code under test believes it started.
fn counting_git(root: &Path) -> (PathBuf, PathBuf) {
    let directory: PathBuf = root.join("counting-bin");
    std::fs::create_dir(&directory).expect("counting directory");
    let program: PathBuf = directory.join("git");
    let count: PathBuf = root.join("count");
    executable(
        program.as_path(),
        format!(
            "#!/bin/sh\nprintf 'start\\n' >> '{}'\nexec {REAL_GIT} \"$@\"\n",
            count.display()
        )
        .as_bytes(),
    );
    return (program, count);
}

/// The number of Git starts recorded so far.
fn starts(count: &Path) -> usize {
    match std::fs::read_to_string(count) {
        Ok(text) => return text.lines().count(),
        Err(error) => {
            assert_eq!(error.kind(), std::io::ErrorKind::NotFound, "{error}");
            return 0;
        }
    }
}

/// A repository with `files` staged files of distinct content.
fn staged_repository(root: &Path, name: &str, files: usize) -> PathBuf {
    let repo: PathBuf = repository(root, name);
    for index in 0..files {
        std::fs::write(
            repo.join(format!("file-{index}.txt")),
            format!("content {index}\n"),
        )
        .expect("staged file");
    }
    git(repo.as_path(), &["add", "--all"]);
    return repo;
}

/// Read every candidate of the staged version through a store that starts Git only via the counting stand-in.
fn read_all(store: &mut CandidateStore, files: usize) {
    let version: Rc<CandidateVersion> = store
        .version(&CandidateSource::StagedAgainstHead)
        .expect("staged version");
    assert_eq!(version.candidates().len(), files);
    for index in 0..files {
        let path: String = format!("file-{index}.txt");
        let bytes: Rc<[u8]> = store.bytes(at(&version, path.as_bytes())).expect("bytes");
        assert_eq!(&bytes[..], format!("content {index}\n").as_bytes());
    }
}

/// One,
///  twenty and two hundred staged files are each listed and read with the same two Git processes.
#[test]
fn staged_candidates_are_read_with_a_fixed_number_of_processes() {
    let root: PathBuf = fixture("count-bound");
    let (program, count) = counting_git(root.as_path());
    let mut observed: Vec<usize> = Vec::new();
    for files in [1_usize, 20, 200] {
        let repo: PathBuf =
            staged_repository(root.as_path(), format!("repo-{files}").as_str(), files);
        let before: usize = starts(count.as_path());
        let mut store: CandidateStore = store_with(program.as_path(), repo.as_path());
        read_all(&mut store, files);
        // Listing and reading everything again within the version starts nothing.
        read_all(&mut store, files);
        drop(store);
        observed.push(starts(count.as_path()) - before);
    }
    assert_eq!(
        observed,
        [
            STAGED_PROCESS_BOUND,
            STAGED_PROCESS_BOUND,
            STAGED_PROCESS_BOUND
        ]
    );
    remove(root.as_path());
}

/// Positive control:
///  a per-file reader through the same stand-in is counted once per file,
///  so the stand-in can show growth.
#[test]
fn per_file_reader_is_counted_once_per_file() {
    let root: PathBuf = fixture("count-control");
    let (program, count) = counting_git(root.as_path());
    let mut observed: Vec<usize> = Vec::new();
    for files in [1_usize, 20, 200] {
        let repo: PathBuf =
            staged_repository(root.as_path(), format!("repo-{files}").as_str(), files);
        // The listing itself is taken outside the count, with real Git directly.
        let mut listing: CandidateStore = store_with(Path::new(REAL_GIT), repo.as_path());
        let version: Rc<CandidateVersion> = listing
            .version(&CandidateSource::StagedAgainstHead)
            .expect("staged version");
        let before: usize = starts(count.as_path());
        for candidate in version.candidates() {
            // The naive reader: one `git cat-file blob <oid>` process per candidate.
            let output: std::process::Output = Command::new(program.as_path())
                .current_dir(&repo)
                .args([
                    "cat-file",
                    "blob",
                    candidate.object.as_ref().expect("object").as_str(),
                ])
                .output()
                .expect("per-file read");
            assert!(output.status.success());
            assert!(output.stdout.starts_with(b"content "));
        }
        observed.push(starts(count.as_path()) - before);
    }
    assert_eq!(observed, [1, 20, 200]);
    remove(root.as_path());
}

/// A new listing after invalidation costs one more process,
///  and a repository without commits one more for the empty tree.
#[test]
fn invalidation_and_unborn_head_have_fixed_extra_costs() {
    let root: PathBuf = fixture("count-extra");
    let (program, count) = counting_git(root.as_path());
    let repo: PathBuf = staged_repository(root.as_path(), "repo", 50);
    let mut store: CandidateStore = store_with(program.as_path(), repo.as_path());
    read_all(&mut store, 50);
    assert_eq!(starts(count.as_path()), STAGED_PROCESS_BOUND);
    store.invalidate();
    // The reader and its remembered blobs survive; only the listing is run again.
    read_all(&mut store, 50);
    assert_eq!(starts(count.as_path()), STAGED_PROCESS_BOUND + 1);
    drop(store);
    let unborn: PathBuf = root.join("unborn");
    std::fs::create_dir(&unborn).expect("unborn directory");
    git(
        unborn.as_path(),
        &["init", "--quiet", "--initial-branch=main"],
    );
    for index in 0..50 {
        std::fs::write(
            unborn.join(format!("file-{index}.txt")),
            format!("content {index}\n"),
        )
        .expect("staged file");
    }
    git(unborn.as_path(), &["add", "--all"]);
    let before: usize = starts(count.as_path());
    let mut unborn_store: CandidateStore = CandidateStore::new(
        program.as_path(),
        &[OsString::from("-C"), unborn.clone().into_os_string()],
        &[],
    );
    read_all(&mut unborn_store, 50);
    drop(unborn_store);
    assert_eq!(starts(count.as_path()) - before, STAGED_PROCESS_BOUND + 1);
    remove(root.as_path());
}
