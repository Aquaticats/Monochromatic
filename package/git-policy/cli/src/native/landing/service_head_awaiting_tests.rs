//! Controls for the awaited faces of the head questions:
//! a `HEAD` file or a `commondir` that gives no answer holds no worker,
//! both faces answer the same thing,
//! and a task
//!  whose Git must be asked can ask it.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// The shortest a wait can be and still leave a task beside it time to run on this host.
const SHORT_CEILING: Duration = Duration::from_millis(100);

/// How often a control looks at the flag a task beside the work sets.
const THE_TICK: Duration = Duration::from_millis(50);

/// The most a control waits for that task,
/// far beyond what a free worker needs,
/// so only a held worker reaches it.
const THE_PATIENCE: Duration = Duration::from_secs(10);

/// What:
///  Stand a pipe with no writer at `path`,
/// so a read of it waits in the kernel until a writer opens it.
/// Why:
///  This is the one way to make a read of a file hold its thread with no end,
/// which is what a mount that stopped answering does
///  (`foundation/work_awaited_within_a_ceiling.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// execFileSync('mkfifo', [path]);
/// ```
fn stall_a_file(path: &Path) {
    let text: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("a path without a zero byte");
    // SAFETY: `mkfifo` reads the path it is given and writes only the file it makes; the mode is the one a control's own
    //  fixture takes.
    let made: libc::c_int = unsafe { libc::mkfifo(text.as_ptr(), 0o600) };
    assert_eq!(made, 0, "the pipe must stand where the file would be: {}", std::io::Error::last_os_error());
}

/// What:
///  Open the pipe at `path` for writing and close it again,
/// which ends the read a control left behind.
/// Why:
///  The read of the pool is abandoned,
/// never cancelled,
/// so the thread that holds it ends only when a writer appears;
/// the open does
///  not wait for a reader,
///  and the control asks again until `THE_PATIENCE`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (;;) { try { close(openSync(path, 'w')); return } catch { await sleep(10) } }
/// ```
fn release_the_pipe(path: &Path) {
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    let began: std::time::Instant = std::time::Instant::now();
    while began.elapsed() < THE_PATIENCE {
        match std::fs::OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open(path) {
            Ok(mut writer) => {
                let _written = writer.write_all(b"");
                return;
            }
            Err(_not_yet) => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}

/// What:
///  A repository with a linked worktree whose own Git directory holds a `commondir`.
/// Why:
///  The head question reads that file to prove the directory is the asking worktree's,
/// so a pipe standing where it
///  belongs is a read of a file on a mount that stopped answering.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = initRepo(); const linked = addWorktree(repo); return { repo, common: join(repo, '.git'), own: join(repo, '.git/worktrees/linked') };
/// ```
fn a_linked_worktree(name: &str) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["branch", "linked-branch"]);
    let linked: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", linked.to_str().expect("a path"), "linked-branch"]);
    let own: PathBuf = repo.join(".git").join("worktrees").join("linked");
    return (root, repo, linked, own);
}

/// What:
///  A head file that gives no answer does not hold a worker:
/// a task beside the read runs while the read waits,
/// and the read ends at its ceiling.
/// Why:
///  The read is `canonicalize` and two file reads of the worktree's state,
/// which wait in the kernel on a mount that stopped
///  answering;
/// a task that did them itself would hold a worker the runtime needs,
/// and no timer on that worker can end
///  it.
/// The answer of a read that gave nothing in time is the one an unreadable file gives,
/// so the caller asks Git
///  and nothing waits on the file for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [repo, linked, own] = aLinkedWorktree(); mkfifo(join(own, 'commondir')); assert(await runsBeside(() => headFileWithin(100, own, common)) === 'unreadable')
/// ```
#[test]
fn a_head_file_that_gives_no_answer_does_not_hold_a_worker() {
    let (root, _repo, _linked, own) = a_linked_worktree("head-awaiting-stalls");
    let common: PathBuf = root.join("repo").join(".git");
    let stalled: PathBuf = own.join("commondir");
    std::fs::remove_file(stalled.as_path()).expect("the pointer a pipe stands in place of");
    stall_a_file(stalled.as_path());
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().expect("a runtime of one worker");
    let ticked: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let tick: Arc<AtomicBool> = Arc::clone(&ticked);
    let _ticking = runtime.spawn(async move {
        tokio::time::sleep(THE_TICK).await;
        tick.store(true, Ordering::SeqCst);
    });
    let reading_directory: PathBuf = own.clone();
    let common_directory: PathBuf = common.clone();
    let reading = runtime.spawn(async move { return head_file_within(SHORT_CEILING, reading_directory.as_path(), common_directory.as_path()).await; });
    let began: std::time::Instant = std::time::Instant::now();
    while !ticked.load(Ordering::SeqCst) && began.elapsed() < THE_PATIENCE {
        std::thread::sleep(THE_TICK);
    }
    let ran_beside: bool = ticked.load(Ordering::SeqCst);
    // The pipe is let go before the answer is read, so a read that held the worker cannot hold this control too.
    release_the_pipe(stalled.as_path());
    let answered: HeadFile = runtime.block_on(async { return reading.await.expect("the reading task ends"); });
    assert!(ran_beside, "the task beside a head file that gives no answer still runs, so no worker is held by that read");
    assert_eq!(answered, HeadFile::Unreadable, "a head file that gave nothing in time is answered as an unreadable one, so Git is asked instead");
    assert!(began.elapsed() < THE_PATIENCE, "the read ended at its ceiling and never waited on the file without end: {} ms", began.elapsed().as_millis());
    remove(root.as_path());
}

/// What:
///  The answers one question gives through both faces,
/// so a control compares them without asking which is which.
/// Why:
///  The awaited face must answer what the synchronous one answers;
/// a difference here is a landing that reads another
///  worktree's head once it is a task.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [sync, awaited] = bothAnswers(git, repo, own, common, reference);
/// ```
fn both_answers(git: &GitContext, repository: &Path, own: &Path, common: &Path, reference: &crate::ref_name::RefName) -> (Result<bool, FailureCode>, Result<bool, FailureCode>) {
    let synchronous: Result<bool, FailureCode> = still_checked_out(git, repository, own, common, reference);
    let awaited: Result<bool, FailureCode> =
        crate::lazy_runtime::run_on_runtime(still_checked_out_awaiting(git, repository, own, common, reference)).expect("the runtime");
    return (synchronous, awaited);
}

/// What:
///  Both faces answer the same thing about a branch,
/// about another branch,
/// about a detached head and about a
///  linked worktree's own Git directory.
/// Why:
///   The synchronous face is what the landing calls today and the awaited one is what it will call as a task,
/// so the two
///  must agree everywhere the answer is read from a file and nowhere near Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const reference of ['refs/heads/main', 'refs/heads/other', 'HEAD']) assert.deepEqual(...bothAnswers(reference));
/// ```
#[test]
fn the_awaited_head_questions_answer_what_the_synchronous_ones_answer() {
    let root: PathBuf = fixture("head-awaiting-agrees");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let common: PathBuf = repo.join(".git");
    let context: GitContext = git_context();
    for reference in ["refs/heads/main", "refs/heads/other", "HEAD"] {
        let (synchronous, awaited) = both_answers(&context, repo.as_path(), common.as_path(), common.as_path(), &crate::ref_name::RefName::from_text(reference));
        assert_eq!(synchronous, awaited, "the two faces answer one thing about {reference} in a worktree on a branch");
    }
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["checkout", "--quiet", "--detach", head.as_str()]);
    for reference in ["refs/heads/main", "HEAD"] {
        let (synchronous, awaited) = both_answers(&context, repo.as_path(), common.as_path(), common.as_path(), &crate::ref_name::RefName::from_text(reference));
        assert_eq!(synchronous, awaited, "the two faces answer one thing about {reference} in a worktree with a detached head");
    }
    // A linked worktree reads its own Git directory, and the two faces read the same one.
    git(repo.as_path(), &["branch", "linked-branch"]);
    let linked: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", linked.to_str().expect("a path"), "linked-branch"]);
    let own: PathBuf = common.join("worktrees").join("linked");
    for reference in ["refs/heads/linked-branch", "refs/heads/main", "HEAD"] {
        let (synchronous, awaited) = both_answers(&context, linked.as_path(), own.as_path(), common.as_path(), &crate::ref_name::RefName::from_text(reference));
        assert_eq!(synchronous, awaited, "the two faces answer one thing about {reference} in a linked worktree");
    }
    remove(root.as_path());
}

/// What:
///  A head question whose Git must be asked is answered inside a task,
/// and the synchronous face cannot answer there at
///  all.
/// Why:
///   The synchronous face runs its child by entering the runtime,
/// which a task may not do,
/// so a landing that becomes a task
///  needs the awaited face;
/// the answer it gives is the one Git gives,
/// which is what pins that the child ran and was awaited rather than
///  abandoned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answered = await runtime.spawn(() => followsMoveAwaiting(...)); const refused = await runtime.spawn(() => followsMove(...)); assert(answered.ok && refused.unavailable)
/// ```
#[test]
fn a_head_question_whose_git_must_be_asked_is_answered_inside_a_task() {
    let root: PathBuf = fixture("head-awaiting-in-a-task");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let common: PathBuf = repo.join(".git");
    let context: GitContext = git_context();
    // A Git directory that is no worktree of the repository cannot answer from a file, so Git is asked.
    let elsewhere: PathBuf = root.join("no-git-dir");
    std::fs::create_dir_all(elsewhere.as_path()).expect("a directory that is no Git directory");
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().expect("a runtime of one worker");
    let awaiting_context: GitContext = context.clone();
    let awaiting_repository: PathBuf = repo.clone();
    let awaiting_own: PathBuf = elsewhere.clone();
    let awaiting_common: PathBuf = common.clone();
    let awaiting_head: String = head.clone();
    let answered: Result<bool, FailureCode> = runtime.block_on(async move {
        return follows_move_awaiting(&awaiting_context, awaiting_repository.as_path(), awaiting_own.as_path(), awaiting_common.as_path(), &crate::ref_name::RefName::from_text("HEAD"), awaiting_head.as_str())
            .await;
    });
    assert_eq!(answered, Ok(true), "the awaited face asks Git for a head no file can answer and the commit it names is the one that moved the branch");
    let waiting: GitContext = context.clone();
    let waiting_repository: PathBuf = repo.clone();
    let waiting_own: PathBuf = elsewhere.clone();
    let waiting_common: PathBuf = common.clone();
    let waiting_head: String = head.clone();
    let refused: Result<bool, FailureCode> = runtime.block_on(async move {
        let asking = tokio::spawn(async move {
            return follows_move(&waiting, waiting_repository.as_path(), waiting_own.as_path(), waiting_common.as_path(), &crate::ref_name::RefName::from_text("HEAD"), waiting_head.as_str());
        });
        return asking.await.expect("the task that asks ends");
    });
    assert_eq!(refused, Err(FailureCode::Unavailable), "the synchronous face cannot start its Git from inside the runtime, which is why a task asks the awaited face");
    remove(root.as_path());
}
