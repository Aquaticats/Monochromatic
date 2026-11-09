//! Controls for the Git runner a task of the runtime awaits:
//! a real Git child answers with its output,
//! an input reaches Git through the private file,
//! a failure names the command,
//! and the same query asked of the synchronous face from inside the runtime is refused,
//! which is why
//!  this face exists.
#![cfg(unix)]

use super::*;
use crate::landing_state_carried_by_its_task::in_a_landing_task;
use crate::test_support::{fixture, git_context, remove, repository};
use std::path::PathBuf;

/// The text of a Git answer,
/// without the newline Git closes a line of output with.
fn answer(bytes: &[u8]) -> String {
    return String::from_utf8_lossy(bytes).trim().to_string();
}

/// A question put to Git is answered on the runtime,
/// and its output is the whole of what Git wrote.
#[test]
fn a_query_awaited_on_the_runtime_gives_gits_answer() {
    let root: PathBuf = fixture("transaction-git-awaiting-query");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let request: GitRequest = GitRequest::new(repo.as_path(), &["rev-parse", "--show-toplevel"]);
    let output: GitOutput = crate::lazy_runtime::run_on_runtime(async {
        return run_git_awaiting(&context, &request).await;
    })
    .expect("the runtime")
    .expect("rev-parse runs");
    assert!(output.succeeded(), "{}", output.error_text());
    assert_eq!(answer(output.stdout.as_slice()), repo.display().to_string());
    remove(root.as_path());
}

/// What:
///  The same query asked of the synchronous face from inside the runtime is refused.
/// Why:
///   The control above would pass for the wrong reason if the synchronous face also worked there:
/// this shows the two faces differ
///  in exactly the way the asynchronous one is written for.
#[test]
fn the_synchronous_face_of_the_same_query_is_refused_from_inside_the_runtime() {
    let root: PathBuf = fixture("transaction-git-awaiting-sync-refused");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let request: GitRequest = GitRequest::new(repo.as_path(), &["rev-parse", "--show-toplevel"]);
    let refused: std::io::Result<GitOutput> = crate::lazy_runtime::run_on_runtime(async {
        return crate::transaction_git::run_git(&context, &request);
    })
    .expect("the runtime");
    let error: std::io::Error = refused.expect_err("a step that is ended at its ceiling registers its pipes by entering the runtime");
    assert!(
        error.to_string().contains("inside itself"),
        "the refusal names the runtime that was asked for from inside itself: {error}"
    );
    remove(root.as_path());
}

/// Bytes on standard input reach Git through the private input file,
/// and the object it writes comes back by name:
/// an input larger than a pipe buffer never deadlocks the run.
#[test]
fn an_input_awaited_on_the_runtime_reaches_git_and_comes_back_hashed() {
    let root: PathBuf = fixture("transaction-git-awaiting-input");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let mut hashing: GitRequest = GitRequest::new(repo.as_path(), &["hash-object", "-w", "--stdin"]);
    hashing.input = Some(b"content\n".to_vec());
    let hashed: GitOutput = crate::lazy_runtime::run_on_runtime(async {
        return run_git_checked_awaiting(&context, &hashing).await;
    })
    .expect("the runtime")
    .expect("hash-object");
    assert_eq!(answer(hashed.stdout.as_slice()), "d95f3ad14dee633a758d2e331151e950dd13e4ed");
    let mut large: GitRequest = GitRequest::new(repo.as_path(), &["hash-object", "--stdin"]);
    large.input = Some(vec![b'x'; 4 * 1024 * 1024]);
    let big: GitOutput = crate::lazy_runtime::run_on_runtime(async {
        return run_git_checked_awaiting(&context, &large).await;
    })
    .expect("the runtime")
    .expect("hash-object on a large input");
    assert!(big.succeeded());
    remove(root.as_path());
}

/// A non-zero exit is an answer,
/// and a failure names the command and quotes Git's standard error,
/// as the synchronous face does.
#[test]
fn a_failure_awaited_on_the_runtime_names_the_command() {
    let root: PathBuf = fixture("transaction-git-awaiting-failure");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let missing: GitRequest = GitRequest::new(repo.as_path(), &["rev-parse", "--verify", "--quiet", "refs/heads/none"]);
    let output: GitOutput = crate::lazy_runtime::run_on_runtime(async {
        return run_git_awaiting(&context, &missing).await;
    })
    .expect("the runtime")
    .expect("rev-parse runs");
    assert_eq!(output.code, Some(1));
    let failure: GitFailure = crate::lazy_runtime::run_on_runtime(async {
        return run_git_checked_awaiting(&context, &GitRequest::new(repo.as_path(), &["cat-file", "-t", "0000000000000000000000000000000000000001"])).await;
    })
    .expect("the runtime")
    .expect_err("missing object");
    assert!(
        failure.to_string().starts_with("git cat-file -t 0000000000000000000000000000000000000001 failed: fatal:"),
        "{failure}"
    );
    remove(root.as_path());
}

/// A branch write made by a landing task is waited for without a bound,
/// as the owner's "Never cut a branch write (Recommended)"
///  says,
/// and the ref it moves is moved.
#[test]
fn a_branch_write_awaited_in_a_landing_task_moves_the_ref() {
    let root: PathBuf = fixture("transaction-git-awaiting-branch-write");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let head: GitOutput = crate::lazy_runtime::run_on_runtime(async {
        return run_git_checked_awaiting(&context, &GitRequest::new(repo.as_path(), &["rev-parse", "HEAD"])).await;
    })
    .expect("the runtime")
    .expect("rev-parse HEAD");
    let commit: String = answer(head.stdout.as_slice());
    let moving: GitRequest = GitRequest::new(repo.as_path(), &["update-ref", "refs/heads/landed-here", commit.as_str()]);
    let written: GitOutput = crate::lazy_runtime::run_on_runtime(async {
        return in_a_landing_task(async { return run_git_checked_awaiting(&context, &moving).await; }).await;
    })
    .expect("the runtime")
    .expect("update-ref");
    assert!(written.succeeded(), "{}", written.error_text());
    let moved: GitOutput = crate::lazy_runtime::run_on_runtime(async {
        return run_git_checked_awaiting(&context, &GitRequest::new(repo.as_path(), &["rev-parse", "refs/heads/landed-here"])).await;
    })
    .expect("the runtime")
    .expect("rev-parse of the ref the write moved");
    assert_eq!(answer(moved.stdout.as_slice()), commit);
    remove(root.as_path());
}

/// The runner leaves no child of its own behind and no descriptor open,
/// which a piped step that was never read would show.
#[test]
fn a_run_leaves_no_descriptor_open() {
    let root: PathBuf = fixture("transaction-git-awaiting-descriptors");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    // The first run builds the runtime and its reactor, whose own descriptors stay for the life of the process.
    let warm: GitOutput = crate::lazy_runtime::run_on_runtime(async {
        return run_git_awaiting(&context, &GitRequest::new(repo.as_path(), &["status", "--short"])).await;
    })
    .expect("the runtime")
    .expect("the warm-up runs");
    assert!(warm.succeeded(), "{}", warm.error_text());
    let before: usize = std::fs::read_dir("/proc/self/fd").expect("the descriptor table").count();
    for _run in 0..2 {
        let status: GitOutput = crate::lazy_runtime::run_on_runtime(async {
            return run_git_awaiting(&context, &GitRequest::new(repo.as_path(), &["status", "--short"])).await;
        })
        .expect("the runtime")
        .expect("status runs");
        assert!(status.succeeded(), "{}", status.error_text());
    }
    let after: usize = std::fs::read_dir("/proc/self/fd").expect("the descriptor table").count();
    assert_eq!(after, before, "the runs left {} descriptors of their own open", after.saturating_sub(before));
    remove(root.as_path());
}
