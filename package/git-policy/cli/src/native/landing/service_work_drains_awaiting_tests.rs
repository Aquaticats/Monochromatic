//! Controls for the drains awaited by a task of the runtime:
//! a `commondir` that never answers does not hold the round,
//! and a retry whose record for the person could not be written keeps its try.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const drained = await drainCommonRecordsAwaiting(gitDir, ...); expect(drained.wait).toBeDefined();
//! ```

use super::*;
use crate::parallel_test_support::{ONE_PLACE, PATIENCE};
use crate::push_retry::RetryChange;
use crate::service_work::{CLAIM_LEASE, PUSH_KINDS};
use crate::test_support::{fixture, remove};
use crate::work_item::WorkItem;
use std::os::unix::fs::OpenOptionsExt as _;
use std::path::Path;
use std::time::Duration;

/// The longest this control lets the drain it started run before it ends the read itself,
/// which is a moment and not the state step's own ceiling.
const THE_CONTROLS_WAIT: Duration = Duration::from_millis(500);

/// The branch every item of these controls names.
const BRANCH: &str = "refs/heads/main";

/// The commit every item of these controls names.
const OID: &str = "0123456789abcdef0123456789abcdef01234567";

/// Handlers that run nothing and answer done,
/// so a drain reaches its own storage steps.
struct NothingToRun;

impl WorkHandlers for NothingToRun {
    fn push(&self, _items: &[WorkItem]) -> Verdict {
        return Verdict::Done;
    }

    fn maintenance(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn review(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }
}

/// Handlers whose push the person must act on.
struct TerminalFailure;

impl WorkHandlers for TerminalFailure {
    fn push(&self, _items: &[WorkItem]) -> Verdict {
        return Verdict::Failed(String::from("the remote refused the update"));
    }

    fn maintenance(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }

    fn review(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Done;
    }
}

/// What:
///  Stand a named pipe where a file the drain reads must be,
///  so the read of it answers only when a writer arrives.
/// Why:
///   A store that stops answering holds a read in the kernel,
/// and a pipe with no writer is the one file that does so without a filesystem this host would have to break.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(join(gitDir, 'commondir'), 0o600)
/// ```
fn stall_a_file(path: &Path) {
    let text: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("a path without a zero byte");
    // SAFETY: `mkfifo` reads the path it is given and writes only the file it makes; the mode is the one a control's own
    //  directory takes.
    let made: libc::c_int = unsafe { libc::mkfifo(text.as_ptr(), 0o600) };
    assert_eq!(made, 0, "the pipe must stand where the file is read: {}", std::io::Error::last_os_error());
}

/// What:
///  Open the pipe for writing and close it again,
/// which ends the read the drain left behind when its ceiling passed.
/// Why:
///  That read is abandoned,
/// never cancelled, so the thread that holds it ends only when a writer appears;
/// a control that left it would keep one thread of the test binary, and no more.
/// The open does not wait for a reader:
///  a pipe with no reader is reported instead,
/// and the control tries again until `PATIENCE`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// while (elapsed < PATIENCE) { try { const w = open(path, O_WRONLY | O_NONBLOCK); w.write(''); return } catch {} }
/// ```
fn release_the_pipe(path: &Path) {
    let began: Instant = Instant::now();
    while began.elapsed() < PATIENCE {
        match std::fs::OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open(path) {
            Ok(mut writer) => {
                let _written = std::io::Write::write_all(&mut writer, b"");
                return;
            }
            Err(_not_yet) => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}

/// The records the person has been told about below `git_dir`.
fn notices(git_dir: &Path) -> usize {
    return std::fs::read_dir(crate::push_failure_notice::notice_directory(git_dir)).map_or(0, |listing| { return listing.flatten().count(); });
}

/// What:
///  A drain of the common directory of a linked worktree whose `commondir` gives no answer ends,
///  and asks to be looked at again,
/// instead of reading that file on the runtime's own task.
/// Why:
///   The read is storage,
/// and a filesystem that stopped answering holds the thread that makes it for good:
/// the round that awaits this drain then never ends,
/// so no later push or upkeep round runs and the service never exits idle
/// (`AGENTS.md`,
/// "nothing should ever wait forever or indefinitely").
/// Whether a common directory is there is not known after such a read,
/// so the answer is a look again and never "this repository has none".
#[test]
fn a_commondir_that_gives_no_answer_does_not_hold_the_drain() {
    let root: PathBuf = fixture("drains-common-no-answer");
    let git_dir: PathBuf = root.join("worktree").join(".git");
    std::fs::create_dir_all(git_dir.as_path()).expect("Git directory");
    let stalled: PathBuf = git_dir.join("commondir");
    stall_a_file(stalled.as_path());
    let read_by: PathBuf = git_dir.clone();
    let (finished, has_finished) = std::sync::mpsc::channel::<Drained>();
    let draining = std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime for the control");
        let handlers: NothingToRun = NothingToRun;
        let drained: Drained = runtime.block_on(drain_common_records_planned_awaiting(read_by.as_path(), &handlers, &ClaimTime::read(), CLAIM_LEASE, &Retrying::none(), &PUSH_KINDS));
        let _sent = finished.send(drained);
    });
    let returned: Result<Drained, std::sync::mpsc::RecvTimeoutError> = has_finished.recv_timeout(THE_CONTROLS_WAIT);
    // The read the drain left behind is ended before this control judges, so a failing control leaves no thread reading.
    release_the_pipe(stalled.as_path());
    let drained: Drained = returned.expect("the drain waited inside its own read of `commondir`, so the round that awaits it never ends");
    assert!(
        drained.wait.is_some(),
        "whether there is a common directory is not known, so the drain asks to be looked at again"
    );
    draining.join().expect("the draining task");
    remove(root.as_path());
}

/// What:
///  A retry the person must hear about keeps its try when the step that writes their record could not run,
///  and the next round that can write the record drops the try.
/// Why:
///   Nothing scans the records of `failed`,
/// so the record is the person's only word about a push that did not land;
/// a try dropped before that record exists leaves a commit unpushed with neither a retry nor a word,
/// which no command would ever finish (`AGENTS.md`,
/// "never discard user data").
/// Every place of the process's bound is held here,
/// which is the state a store that answers slowly leaves behind,
/// so the record's step cannot start and only the try can keep the commit from being abandoned.
#[test]
fn a_retry_whose_record_could_not_be_written_keeps_its_try() {
    let root: PathBuf = fixture("drains-terminal-retry-keeps-its-try");
    let repository: PathBuf = root.join("repository");
    let git_dir: PathBuf = repository.join(".git");
    std::fs::create_dir_all(git_dir.as_path()).expect("Git directory");
    let plans: PathBuf = root.join(crate::push_retry::PLAN_DIRECTORY);
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: repository.clone(),
        git_dir: git_dir.clone(),
        branch_ref: String::from(BRANCH),
        oid: String::from(OID),
        parent: String::new(),
        created: String::from("2026-01-01T00:00:00.000Z"),
        tries: 0,
    };
    let due: Due = Due { git_dir: git_dir.clone(), item: item.clone(), attempt: 0 };
    crate::push_retry_store::save(plans.as_path(), &crate::push_retry_store::StoredPlan { git_dir: git_dir.clone(), item: item.clone(), attempt: 0 }).expect("the plan the round took");
    assert_eq!(crate::push_retry_store::load(plans.as_path()).len(), 1, "positive control: the durable try is there before the round");
    crate::blocking_jobs_within_the_budget::hold_blocking_jobs_to(ONE_PLACE);
    let (started, has_started) = std::sync::mpsc::channel::<()>();
    let (release, waiting) = std::sync::mpsc::channel::<()>();
    let holder = std::thread::spawn(move || {
        let answer = crate::lazy_runtime::run_on_runtime(crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget(move || {
            let _told = started.send(());
            let _held = waiting.recv();
        }))
        .expect("the holder's runtime");
        answer.expect("the holder's job");
    });
    assert!(has_started.recv_timeout(PATIENCE).is_ok(), "the only place of the process's bound is held");
    let mut replanning: Replanning = Replanning { first: Some(FIRST_PAUSE), changes: Vec::new(), plans: Some(plans.clone()) };
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime for the control");
    runtime.block_on(run_due_push_retries_awaiting(git_dir.as_path(), &TerminalFailure, std::slice::from_ref(&due), &mut replanning));
    let kept: usize = crate::push_retry_store::load(plans.as_path()).len();
    let asked: bool = replanning.changes.iter().any(|change: &RetryChange| { return matches!(change, RetryChange::Plan { .. }); });
    let told: usize = notices(git_dir.as_path());
    let _let_go = release.send(());
    holder.join().expect("the holder");
    assert_eq!(told, 0, "no place came within the ceiling, so nothing was written and the person has not been told yet");
    assert_eq!(kept, 1, "the try the record was to replace is still on disk, so a later round tries the branch and tells the person");
    assert!(asked, "the loop is asked to try the branch again: {:?}", replanning.changes);
    // The bound is free now, so the next round writes the record and only then drops the try.
    let mut later: Replanning = Replanning { first: Some(FIRST_PAUSE), changes: Vec::new(), plans: Some(plans.clone()) };
    runtime.block_on(run_due_push_retries_awaiting(git_dir.as_path(), &TerminalFailure, std::slice::from_ref(&due), &mut later));
    drop(runtime);
    assert_eq!(notices(git_dir.as_path()), 1, "the person has the record of the push that needs them");
    assert!(crate::push_retry_store::load(plans.as_path()).is_empty(), "the record exists, so the try it replaces is dropped");
    remove(root.as_path());
}
