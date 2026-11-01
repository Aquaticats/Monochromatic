//! What:
//!  Count the Git processes both private-index sources start,
//!  by a mechanism outside them.
//! Why:
//!  Predicting `git add` and projecting a direct scope must not start a process per
//!      file.
//!  A counting stand-in that every Git start passes through records each start,
//!      and a deliberately per-file reader through the same stand-in shows it sees growth.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // prepare(200 files); readAll(); expect(lines('/fixture/count')).toBe(6);
//! ```
#![cfg(unix)]

/// Import the module under test.
use super::{CandidateRequest, PreparedCandidates, prepare_candidates};
use crate::candidate_version::CandidateVersion;
use crate::test_support::{REAL_GIT, executable, fixture, git, remove, repository};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

/// What:
///  The Git processes predicting one `git add` may start,
///  however many files it
///       stages:
///  the index path query,
///  the listing before and after the replay,
///  the
///       replay,
///  one baseline listing and the long-lived object reader.
/// Why:
///   The bound is a constant;
///  anything per file would make it grow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ADD_PROCESS_BOUND = 6;
/// ```
const ADD_PROCESS_BOUND: usize = 6;

/// What:
///  The Git processes projecting one direct scope may start:
///  the index path query,
///       the private `git add --all`,
///  the scope listing,
///  one baseline listing and the
///       object reader.
/// Why:
///   As for `git add`,
///  a constant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DIRECT_PROCESS_BOUND = 5;
/// ```
const DIRECT_PROCESS_BOUND: usize = 5;

/// A counting `git` that appends one line per start,
///  then becomes real Git.
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

/// A committed repository with `files` new worktree files of distinct content.
fn repository_with_new_files(root: &Path, name: &str, files: usize) -> PathBuf {
    let repo: PathBuf = repository(root, name);
    std::fs::write(repo.join("base.txt"), b"base\n").expect("base");
    git(repo.as_path(), &["add", "base.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::create_dir(repo.join("new")).expect("new directory");
    for index in 0..files {
        std::fs::write(
            repo.join(format!("new/file-{index}.txt")),
            format!("content {index}\n"),
        )
        .expect("new file");
    }
    return repo;
}

/// Prepare through `program` and read every candidate's bytes twice.
fn prepare_and_read(program: &Path, repo: &Path, request: &CandidateRequest, files: usize) {
    let mut prepared: PreparedCandidates = prepare_candidates(
        program,
        &[OsString::from("-C"), repo.as_os_str().to_os_string()],
        &[],
        request,
    )
    .expect("candidates");
    // A direct scope of `:/` also selects the committed base file.
    assert!(prepared.version.candidates().len() >= files);
    for _round in 0..2 {
        let version: Rc<CandidateVersion> = Rc::clone(&prepared.version);
        for candidate in version.candidates() {
            let bytes: Rc<[u8]> = prepared.store.bytes(candidate).expect("bytes");
            assert!(bytes.ends_with(b"\n"));
        }
    }
}

/// One,
///  twenty and two hundred files are predicted and read with the same number of processes.
#[test]
fn both_sources_start_a_fixed_number_of_processes() {
    let root: PathBuf = fixture("prediction-count");
    let (program, count) = counting_git(root.as_path());
    let mut add_counts: Vec<usize> = Vec::new();
    let mut direct_counts: Vec<usize> = Vec::new();
    for files in [1_usize, 20, 200] {
        let repo: PathBuf =
            repository_with_new_files(root.as_path(), format!("repo-{files}").as_str(), files);
        let before_add: usize = starts(count.as_path());
        prepare_and_read(
            program.as_path(),
            repo.as_path(),
            &CandidateRequest::Add(vec![OsString::from("new")]),
            files,
        );
        add_counts.push(starts(count.as_path()) - before_add);
        let before_direct: usize = starts(count.as_path());
        prepare_and_read(
            program.as_path(),
            repo.as_path(),
            &CandidateRequest::Direct(vec![OsString::from(":/")]),
            files,
        );
        direct_counts.push(starts(count.as_path()) - before_direct);
    }
    assert_eq!(add_counts, [ADD_PROCESS_BOUND; 3]);
    assert_eq!(direct_counts, [DIRECT_PROCESS_BOUND; 3]);
    remove(root.as_path());
}

/// More changed paths than one baseline listing takes cost exactly one more listing,
///  and every path is still classified.
#[test]
fn a_second_baseline_listing_starts_after_two_thousand_and_forty_eight_paths() {
    let root: PathBuf = fixture("prediction-count-chunks");
    let (program, count) = counting_git(root.as_path());
    let files: usize = super::PATHSPEC_CHUNK_SIZE + 1;
    let repo: PathBuf = repository_with_new_files(root.as_path(), "repo", files);
    let prepared: PreparedCandidates = prepare_candidates(
        program.as_path(),
        &[OsString::from("-C"), repo.as_os_str().to_os_string()],
        &[],
        &CandidateRequest::Add(vec![OsString::from("new")]),
    )
    .expect("candidates");
    assert_eq!(prepared.version.candidates().len(), files);
    for candidate in prepared.version.candidates() {
        assert_eq!(
            candidate.change,
            crate::candidate_record::CandidateChange::Added,
            "every path of both listings is classified"
        );
    }
    drop(prepared);
    // No bytes were read, so no object reader beyond the one the baseline needs started.
    assert_eq!(starts(count.as_path()), ADD_PROCESS_BOUND + 1);
    remove(root.as_path());
}

/// Positive control:
///  reading the same candidates one `git cat-file` per file through the same stand-in is counted once per file.
#[test]
fn a_per_file_reader_through_the_stand_in_grows_with_the_files() {
    let root: PathBuf = fixture("prediction-count-control");
    let (program, count) = counting_git(root.as_path());
    let mut observed: Vec<usize> = Vec::new();
    for files in [1_usize, 20, 200] {
        let repo: PathBuf =
            repository_with_new_files(root.as_path(), format!("repo-{files}").as_str(), files);
        // The prediction itself runs on real Git directly, outside the count.
        let prepared: PreparedCandidates = prepare_candidates(
            Path::new(REAL_GIT),
            &[OsString::from("-C"), repo.as_os_str().to_os_string()],
            &[],
            &CandidateRequest::Add(vec![OsString::from("new")]),
        )
        .expect("candidates");
        let before: usize = starts(count.as_path());
        for candidate in prepared.version.candidates() {
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
