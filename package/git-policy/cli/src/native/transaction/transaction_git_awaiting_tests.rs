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
use crate::parallel_test_support::ONE_PLACE;
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

/// What:
///  A branch write of the landing asked of the **synchronous** face from inside
///  the runtime still lands.
/// Why:
///  This is why the synchronous face was not turned into a bridge over the
///  asynchronous one:
/// a step with an `Unbounded` bound makes no pipes,
/// and the pipes are the only reason the
///  synchronous face enters the runtime,
/// so a bridge would have turned a working path into `RuntimeError::Nested`.
/// The control fails if a later change
///  makes the synchronous face enter the runtime for every bound.
#[test]
fn a_branch_write_of_the_landing_asked_of_the_synchronous_face_inside_the_runtime_still_lands() {
    let root: PathBuf = fixture("transaction-git-awaiting-sync-branch-write");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let head: GitOutput = crate::lazy_runtime::run_on_runtime(async {
        return run_git_checked_awaiting(&context, &GitRequest::new(repo.as_path(), &["rev-parse", "HEAD"])).await;
    })
    .expect("the runtime")
    .expect("rev-parse HEAD");
    let commit: String = String::from_utf8_lossy(head.stdout.as_slice()).trim().to_string();
    let moving: GitRequest = GitRequest::new(repo.as_path(), &["update-ref", "refs/heads/landed-synchronously", commit.as_str()]);
    let written: GitOutput = crate::lazy_runtime::run_on_runtime(async {
        return in_a_landing_task(async { return crate::transaction_git::run_git(&context, &moving); }).await;
    })
    .expect("the runtime")
    .expect("the synchronous face of a branch write runs from inside the runtime");
    assert!(written.succeeded(), "{}", written.error_text());
    let moved: GitOutput = crate::lazy_runtime::run_on_runtime(async {
        return run_git_checked_awaiting(&context, &GitRequest::new(repo.as_path(), &["rev-parse", "refs/heads/landed-synchronously"])).await;
    })
    .expect("the runtime")
    .expect("rev-parse of the ref the write moved");
    assert_eq!(String::from_utf8_lossy(moved.stdout.as_slice()).trim(), commit);
    remove(root.as_path());
}

/// The word the stand-in Git of the control below answers with,
/// which no reader ever writes.
const ANSWERED_BY_GIT: &str = "/answered-by-git";

/// The stand-in Git of the control below:
/// it answers the one question the readers ask a real Git with a system configuration path of its own,
/// and every other command line with `ANSWERED_BY_GIT`.
const GIT_THAT_ANSWERS_WITH_A_MARKER: &str = "#!/bin/sh\nif [ \"$1\" = \"var\" ]; then printf '/nonexistent/etc/gitconfig\\n'; exit 0; fi\nprintf '/answered-by-git\\n'\nexit 0\n";

/// What:
///  The in-process answer is read as a job of the process's blocking pool,
///  so a pool with no free place leaves the answer to Git,
///  and the request still ends.
/// Why:
///   The readers walk the file system and read the repository's real path,
/// which waits in the kernel on a store that stopped answering,
/// and a worker of the runtime inside that read cannot be ended by any timer:
/// the request never returns at all
/// (the review of `259f83cbab`,
/// the route's fallback repeating its own reading in process,
/// on the worker,
/// outside every ceiling).
/// The answer is a saving rather than the only way to it,
/// so no commit may wait for a place of the pool to have it.
/// The one place of this process's bound is held here,
/// and the Git the request names is a stand-in whose answer is a word no reader writes,
/// so the two answers can be told apart.
#[test]
fn a_busy_pool_leaves_the_in_process_answer_to_git() {
    let root: PathBuf = fixture("transaction-git-awaiting-in-process-off-the-worker");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let stand_in: PathBuf = root.join("git-that-answers-with-a-marker");
    crate::test_support::executable(stand_in.as_path(), GIT_THAT_ANSWERS_WITH_A_MARKER.as_bytes());
    let mut context: GitContext = git_context();
    context.real_git = stand_in;
    let request: GitRequest = GitRequest::new(repo.as_path(), &["rev-parse", "--verify", "--quiet", "refs/heads/main^{commit}"]);
    // The budget is made here, with the one place this control then holds.
    crate::blocking_jobs_within_the_budget::hold_blocking_jobs_to(ONE_PLACE);
    // Positive control: with the place free the readers answer, so only a reader can answer this command line.
    let read: GitOutput = crate::lazy_runtime::run_on_runtime(async {
        return run_git_awaiting(&context, &request).await;
    })
    .expect("the runtime")
    .expect("the readers answer the commit's name");
    assert_ne!(
        answer(read.stdout.as_slice()),
        ANSWERED_BY_GIT,
        "positive control: the readers answer this line, so a started Git is the only other answer there is"
    );
    // The place is held, so the read cannot be a job, and the same request goes to Git.
    let (started, has_started) = std::sync::mpsc::channel::<()>();
    let (release, waits) = std::sync::mpsc::channel::<()>();
    let holder = std::thread::spawn(move || {
        let answer = crate::lazy_runtime::run_on_runtime(crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget(move || {
            let _told = started.send(());
            let _held = waits.recv();
        }))
        .expect("the holder's runtime");
        answer.expect("the holder's job");
    });
    assert!(has_started.recv_timeout(crate::parallel_test_support::PATIENCE).is_ok(), "the one place of the process's bound is held");
    let busy: std::io::Result<GitOutput> = crate::lazy_runtime::run_on_runtime(async {
        return run_git_awaiting(&context, &request).await;
    })
    .expect("the runtime");
    let _let_go = release.send(());
    holder.join().expect("the holder");
    let answered: GitOutput = busy.expect("Git answers a command line the readers may answer, and this Git starts");
    assert_eq!(
        answer(answered.stdout.as_slice()),
        ANSWERED_BY_GIT,
        "the in-process read ran on this worker, where a store that did not answer would hold it for good"
    );
    remove(root.as_path());
}
