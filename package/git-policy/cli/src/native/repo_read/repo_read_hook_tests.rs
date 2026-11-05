//! Controls for the runner entry points:
//!  a command line the readers know starts no Git process and prints what Git prints, the switch brings the process
//!  back, a command line the readers do not know still starts Git, and a request's private index is the one read.

use super::*;
use crate::git_metadata::run_metadata_git;
use crate::test_support::{REAL_GIT, executable, fixture, git, git_output, remove, repository};
use crate::transaction_git::{GitContext, GitRequest, run_git};
use std::path::PathBuf;

/// The variables that make the host's own configuration irrelevant, as the fixtures' Git sees it.
fn hermetic(root: &Path) -> Vec<(OsString, OsString)> {
    return vec![
        (OsString::from("GIT_CONFIG_NOSYSTEM"), OsString::from("1")),
        (OsString::from("GIT_CONFIG_GLOBAL"), OsString::from("/nonexistent-global-config")),
        (OsString::from("HOME"), root.join("no-home").into_os_string()),
    ];
}

/// A stand-in Git that records each start and runs the real one, and the file it records into.
fn counting_git(root: &Path) -> (PathBuf, PathBuf) {
    let directory = root.join("counting-bin");
    std::fs::create_dir(&directory).expect("directory");
    let program = directory.join("git");
    let count = root.join("count");
    executable(program.as_path(), format!("#!/bin/sh\nprintf 'start\\n' >> '{}'\nexec {REAL_GIT} \"$@\"\n", count.display()).as_bytes());
    return (program, count);
}

/// How many starts the stand-in recorded.
fn starts(count: &Path) -> usize {
    return std::fs::read_to_string(count).map_or(0, |text| { return text.lines().count(); });
}

/// The arguments of a staged listing run in `repo`.
fn listing(repo: &Path) -> Vec<OsString> {
    return ["-C", repo.to_str().expect("UTF-8"), "ls-files", "--stage", "-z", "--full-name"].iter().map(OsString::from).collect();
}

/// A known query starts nothing and equals Git; the switch, an unknown query and a missing Git behave as described.
#[test]
fn known_queries_start_no_process_and_the_switch_restores_it() {
    let root = fixture("repo-read-hook-metadata");
    let repo = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    git(repo.as_path(), &["add", "a.txt"]);
    let (program, count) = counting_git(root.as_path());
    let overlay = hermetic(root.as_path());
    let expected = git_output(repo.as_path(), &["ls-files", "--stage", "-z", "--full-name"]);
    let found = run_metadata_git(program.as_path(), listing(repo.as_path()).as_slice(), overlay.as_slice()).expect("answer");
    assert_eq!(found.stdout, expected.stdout);
    assert_eq!(starts(count.as_path()), 0, "an in-process answer starts no Git");
    let mut off = overlay.clone();
    off.extend(switch_off());
    let started = run_metadata_git(program.as_path(), listing(repo.as_path()).as_slice(), off.as_slice()).expect("Git runs");
    assert_eq!(started.stdout, expected.stdout);
    assert_eq!(starts(count.as_path()), 1, "the switch starts Git");
    let unknown: Vec<OsString> = ["-C", repo.to_str().expect("UTF-8"), "rev-parse", "HEAD"].iter().map(OsString::from).collect();
    run_metadata_git(program.as_path(), unknown.as_slice(), overlay.as_slice()).expect("Git runs");
    assert_eq!(starts(count.as_path()), 2, "a command line the readers do not know starts Git");
    let missing = run_metadata_git(Path::new("/nonexistent-directory/git"), listing(repo.as_path()).as_slice(), overlay.as_slice());
    assert!(missing.is_ok(), "a known query needs no Git program at all");
    let unstartable = run_metadata_git(Path::new("/nonexistent-directory/git"), unknown.as_slice(), overlay.as_slice());
    assert!(unstartable.is_err(), "a query the readers decline reports that Git could not start");
    remove(root.as_path());
}

/// A transaction request reads the private index it names, not the repository's own.
#[test]
fn a_request_reads_its_private_index() {
    let root = fixture("repo-read-hook-request");
    let repo = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    git(repo.as_path(), &["add", "a.txt"]);
    let private = root.join("private-index");
    std::fs::copy(repo.join(".git/index"), &private).expect("copy");
    std::fs::write(repo.join("b.txt"), "b\n").expect("b");
    let added = std::process::Command::new(REAL_GIT)
        .current_dir(repo.as_path())
        .env_clear()
        .env("GIT_INDEX_FILE", &private)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .args(["add", "b.txt"])
        .output()
        .expect("private add");
    assert!(added.status.success());
    let (program, count) = counting_git(root.as_path());
    // The overlay already names the repository's own index; the request's private index is added later and wins,
    //  as the last assignment does in the child's environment.
    let mut overlay = hermetic(root.as_path());
    overlay.push((OsString::from("GIT_INDEX_FILE"), repo.join(".git/index").into_os_string()));
    let context = GitContext { real_git: program, overlay, global_prefix: Vec::new(), common_dir: None };
    let mut request = GitRequest::new(repo.as_path(), &["ls-files", "--stage", "-z", "--full-name"]);
    request.index = Some(private.clone());
    let found = run_git(&context, &request).expect("answer");
    let own = git_output(repo.as_path(), &["ls-files", "--stage", "-z", "--full-name"]);
    assert_ne!(found.stdout, own.stdout, "the private index holds a path the repository's index does not");
    assert!(found.stdout.windows(5).any(|window| { return window == b"b.txt"; }));
    assert_eq!(starts(count.as_path()), 0);
    remove(root.as_path());
}

/// A reader that panics is answered by Git instead: the guard returns nothing, and a quiet reader passes through.
#[test]
fn a_panicking_reader_is_left_to_git() {
    fn defective(_state: ()) -> Option<u8> {
        panic!("a reader defect");
    }
    fn answering(_state: ()) -> Option<u8> {
        return Some(7u8);
    }
    fn declining(_state: ()) -> Option<u8> {
        return None;
    }
    assert_eq!(guarded((), defective), None);
    assert_eq!(guarded((), answering), Some(7u8));
    assert_eq!(guarded((), declining), None);
}

/// An environment without `GIT_CONFIG_NOSYSTEM` and without the forward-target variable, as a resident process or a caller
///  that holds only a Git path has it: the readers still learn the system file from the Git the caller names, so they answer.
#[test]
fn a_caller_that_names_only_its_git_still_gets_in_process_answers() {
    let root = fixture("repo-read-hook-named-git");
    let repo = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    git(repo.as_path(), &["add", "a.txt"]);
    let named_git = root.join("bin").join("git");
    std::fs::create_dir(named_git.parent().expect("parent")).expect("bin");
    std::fs::copy(REAL_GIT, &named_git).expect("an executable named git");
    let cache = root.join("cache");
    let overlay: Vec<(OsString, OsString)> = vec![
        (OsString::from("GIT_CONFIG_GLOBAL"), OsString::from("/nonexistent-global-config")),
        (OsString::from("HOME"), root.join("no-home").into_os_string()),
        (OsString::from("XDG_CACHE_HOME"), cache.clone().into_os_string()),
    ];
    let context = GitContext { real_git: named_git.clone(), overlay: overlay.clone(), global_prefix: Vec::new(), common_dir: None };
    let request = GitRequest::new(repo.as_path(), &["ls-files", "--stage", "-z", "--full-name"]);
    let expected = git_output(repo.as_path(), &["ls-files", "--stage", "-z", "--full-name"]);
    let found = try_request(&context, &request).expect("the readers answer without the forward-target variable");
    assert_eq!(found.stdout, expected.stdout);
    let learned = std::fs::read_dir(cache.join("cli-git")).expect("the cache directory").count();
    assert_eq!(learned, 1, "the system file path was asked of the named Git once and recorded");
    let metadata = run_metadata_git(named_git.as_path(), listing(repo.as_path()).as_slice(), overlay.as_slice()).expect("answer");
    assert_eq!(metadata.stdout, expected.stdout);
    remove(root.as_path());
}
