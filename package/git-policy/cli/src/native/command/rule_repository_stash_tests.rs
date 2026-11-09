//! What:
//!  The stash query's arguments,
//!  its three answers,
//!  and a real-Git control of both.
//! Why:
//!  A question that read a failure as "no stash" would let `git stash clear` empty the repository.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // stashFromOutput({ success: true, code: 0, stdout: 'sha', stderr: '' }) === true
//! ```

/// The query builder,
///  its reader and the real-Git fixture helpers.
use super::{STASH_REFERENCE, stash_from_output, stash_query_arguments};
use crate::command_test_support::{fixture, git, git_output, os_arguments, remove, repository};
use crate::git_metadata::MetadataOutput;
use std::path::PathBuf;
use std::process::Output;

/// Build the output of one query run.
fn output(success: bool, code: i32, stderr: &[u8]) -> MetadataOutput {
    return MetadataOutput {
        success,
        code: Some(code),
        stdout: Vec::<u8>::new(),
        stderr: stderr.to_vec(),
    };
}

/// The global options come first,
///  then the one reference question.
#[test]
fn the_query_names_the_stash_reference_after_the_global_options() {
    assert_eq!(STASH_REFERENCE, "refs/stash");
    assert_eq!(
        stash_query_arguments(os_arguments(&["-C", "dir"]).as_slice()),
        os_arguments(&["-C", "dir", "rev-parse", "--verify", "--quiet", "refs/stash"])
    );
    assert_eq!(
        stash_query_arguments(&[]),
        os_arguments(&["rev-parse", "--verify", "--quiet", "refs/stash"])
    );
}

/// Status 0 is an entry,
///  status 1 is none,
///  and every other ending is a question Git could not answer.
#[test]
fn a_failed_question_is_never_read_as_no_stash() {
    assert_eq!(stash_from_output(&output(true, 0, b"")), Ok(true));
    assert_eq!(stash_from_output(&output(false, 1, b"")), Ok(false));
    assert_eq!(
        stash_from_output(&output(false, 128, b"fatal: not a git repository\nmore\n")),
        Err(String::from(
            "Git could not read the repository stash: fatal: not a git repository"
        ))
    );
    assert_eq!(
        stash_from_output(&output(false, 128, b"")),
        Err(String::from("Git could not read the repository stash: "))
    );
}

/// Real Git:
///  no stash entry answers 1,
///  and a pushed stash answers 0 with the entry's commit.
#[test]
fn real_git_answers_where_the_repository_holds_a_stash() {
    let root: PathBuf = fixture("repository-stash");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
    git(repo.as_path(), &["add", "--", "tracked.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let empty: Output = git_output(repo.as_path(), stash_query_arguments(&[]).as_slice());
    assert_eq!(empty.status.code(), Some(1), "no stash yet");
    std::fs::write(repo.join("tracked.txt"), b"two\n").expect("edit");
    git(repo.as_path(), &["stash", "push", "--quiet"]);
    let held: Output = git_output(repo.as_path(), stash_query_arguments(&[]).as_slice());
    assert!(held.status.success(), "a stash entry answers 0: {}", String::from_utf8_lossy(held.stderr.as_slice()));
    assert!(!held.stdout.is_empty(), "the entry's commit is printed");
    remove(root.as_path());
}
