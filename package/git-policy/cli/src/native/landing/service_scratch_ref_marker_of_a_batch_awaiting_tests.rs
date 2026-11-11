//! Controls for the awaited records of a batch's scratch reference:
//! the awaited face writes what the synchronous one writes,
//! a sweep that gives no answer holds no worker,
//! the awaited sweep settles what the synchronous one settles,
//! and a failed landing's record goes only once its reference did.

use super::*;
use crate::service_scratch_ref_of_a_dropped_batch::{RefValue, kept_ref, scratch_ref, value_of};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// The batch every control here names.
const THE_BATCH: u64 = 1;

/// How often a control looks at the flag a task beside the work sets.
const THE_TICK: Duration = Duration::from_millis(50);

/// The most a control waits for that task,
/// far beyond what a free worker needs,
/// so only a held worker reaches it.
const THE_PATIENCE: Duration = Duration::from_secs(10);

/// The most a control waits for the writer that lets a stalled read go.
const THE_WRITER_PATIENCE: Duration = Duration::from_secs(10);

/// What:
///  A repository whose batch `THE_BATCH` left a scratch reference naming its only commit,
/// with a journal of its own.
/// Why:
///  That is the state a landing that ended before its intent reached the journal leaves,
/// and it is what the sweep is asked about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = initRepo(); git(repo, ['update-ref', scratchRef(THE_BATCH), headOf(repo)]);
/// ```
fn a_repository_with_a_dropped_batch(name: &str) -> (PathBuf, PathBuf, PathBuf, GitContext, String) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(journal.as_path()).expect("a journal directory");
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["update-ref", scratch_ref(THE_BATCH).as_str(), head.as_str()]);
    let context: GitContext = GitContext { common_dir: Some(repo.join(".git")), ..git_context() };
    return (root, repo, journal, context, head);
}

/// The context the sweep reads,
/// which is the landing's own with this fixture's journal.
fn the_landing_context(git: &GitContext, journal: &Path) -> LandContext {
    return LandContext { git: git.clone(), journal: journal.to_path_buf(), on_objects_written: None, lock_ceiling: crate::wait_ceilings::LANDING_LOCK_CEILING };
}

/// One request that names the fixture's repository,
/// which is all a failed landing's record decision reads of a batch.
fn a_request_for(repo: &Path) -> CommitRequest {
    let common: PathBuf = repo.join(".git");
    return CommitRequest {
        id: String::from("a-control-request"),
        repository: repo.to_path_buf(),
        common_dir: common.clone(),
        git_dir: common.clone(),
        reference: String::from("refs/heads/main"),
        base: "0".repeat(40),
        changes: Vec::new(),
        message: String::from("a control\n"),
        author: None,
        committer: String::from("A Control <control@example.invalid> 1793535889 +0000"),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: common.join("index"),
        scratch: common.join("monochromatic-git-policy-cli-service-a-control"),
        captured: 0,
        holder: String::new(),
    };
}

/// A runtime of one worker,
/// so a task that does storage itself is the one a control sees holding it.
fn one_worker() -> tokio::runtime::Runtime {
    return tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().expect("a runtime of one worker");
}

/// What:
///  Stand a pipe with no writer at `path`,
/// so a read of it waits in the kernel until a writer appears.
/// Why:
///  The sweep reads every record of the journal,
/// and a record on a mount that stopped answering holds its reader with no end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// execFileSync('mkfifo', [path]);
/// ```
fn stall_the_file(path: &Path) {
    let text: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("a path without a zero byte");
    // SAFETY: `mkfifo` reads the path it is given and writes only the file it makes; the mode is the one a control's own
    //  fixture takes.
    let made: libc::c_int = unsafe { libc::mkfifo(text.as_ptr(), 0o600) };
    assert_eq!(made, 0, "the pipe must stand where the record would be: {}", std::io::Error::last_os_error());
}

/// What:
///  Open the pipe at `path` for writing and let it go,
/// which ends the read that waited for a writer.
/// Why:
///  The read of the pool is abandoned,
/// never cancelled,
/// so the thread that holds it ends only when a writer appears,
/// and an open for writing of a pipe nobody reads yet is refused,
/// so the control asks again until `THE_WRITER_PATIENCE`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (;;) { try { close(openSync(path, 'w' | O_NONBLOCK)); return } catch { await sleep(10) } }
/// ```
fn write_into_the_pipe(path: &Path) {
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    let began: Instant = Instant::now();
    while began.elapsed() < THE_WRITER_PATIENCE {
        match std::fs::OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open(path) {
            Ok(mut writer) => {
                let _written = writer.write_all(b"");
                return;
            }
            Err(_nobody_reads_yet) => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}

/// What:
///  The awaited record names the worktree the synchronous record names.
/// Why:
///   The sweep compares the record's Git directory with the instance it sweeps for,
/// which is a hash of that directory's bytes,
/// so a record of another worktree is another instance's reference and is never removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordScratchRefAwaiting(awaited, 2, repo, gitDir); recordScratchRef(synchronous, 2, repo, gitDir); expect(read(awaited)).toEqual(read(synchronous))
/// ```
#[test]
fn the_awaited_record_names_what_the_synchronous_one_names() {
    let root: PathBuf = fixture("scratch-record-awaiting-agrees");
    let awaited: PathBuf = root.join("awaited");
    let synchronous: PathBuf = root.join("synchronous");
    std::fs::create_dir(&awaited).expect("a journal directory");
    std::fs::create_dir(&synchronous).expect("a journal directory");
    let repository: PathBuf = root.join("repo");
    let git_dir: PathBuf = repository.join(".git");
    let runtime: tokio::runtime::Runtime = one_worker();
    runtime.block_on(record_scratch_ref_awaiting(awaited.as_path(), 2, repository.as_path(), git_dir.as_path()));
    record_scratch_ref(synchronous.as_path(), 2, repository.as_path(), git_dir.as_path());
    assert_eq!(
        std::fs::read_to_string(record_path(awaited.as_path(), 2)).expect("the awaited record"),
        std::fs::read_to_string(record_path(synchronous.as_path(), 2)).expect("the synchronous record"),
        "the two faces write one record"
    );
    runtime.block_on(forget_scratch_ref_awaiting(awaited.as_path(), 2));
    forget_scratch_ref(synchronous.as_path(), 2);
    assert!(!record_path(awaited.as_path(), 2).exists(), "the awaited face removes the record it was asked to forget");
    remove(root.as_path());
}

/// What:
///  A task beside the sweep's listing runs while a record gives no answer,
/// and the start goes on without a worker held.
/// Why:
///   The sweep reads every record of the journal while the service starts,
/// so a record on a mount that stopped answering would hold a worker and the start with it;
/// a listing that gave no answer removes nothing,
/// which leaves every reference to the next restart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(join(journal, '1.scratch')); const sweeping = spawn(() => clearRecordedScratchRefsAwaiting(context, null)); assert(await runsBeside(sweeping))
/// ```
#[test]
fn a_sweep_that_gives_no_answer_holds_no_worker() {
    let (root, repo, journal, context, head) = a_repository_with_a_dropped_batch("scratch-record-awaiting-stalls");
    record_scratch_ref(journal.as_path(), THE_BATCH, repo.as_path(), repo.join(".git").as_path());
    let record: PathBuf = record_path(journal.as_path(), THE_BATCH);
    std::fs::remove_file(record.as_path()).expect("the record a pipe stands in place of");
    stall_the_file(record.as_path());
    let landing: LandContext = the_landing_context(&context, journal.as_path());
    let runtime: tokio::runtime::Runtime = one_worker();
    let ticked: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let tick: Arc<AtomicBool> = Arc::clone(&ticked);
    let _ticking = runtime.spawn(async move {
        tokio::time::sleep(THE_TICK).await;
        tick.store(true, Ordering::SeqCst);
    });
    let sweeping = runtime.spawn(async move { return clear_recorded_scratch_refs_awaiting(&landing, None).await; });
    let began: Instant = Instant::now();
    while !ticked.load(Ordering::SeqCst) && began.elapsed() < THE_PATIENCE {
        std::thread::sleep(THE_TICK);
    }
    let ran_beside: bool = ticked.load(Ordering::SeqCst);
    // The pipe is given a writer before the sweep is read, so a read that held the worker cannot hold this control too.
    write_into_the_pipe(record.as_path());
    runtime.block_on(sweeping).expect("the sweeping task ends");
    assert!(ran_beside, "the task beside a record that gives no answer still runs, so no worker is held by the sweep");
    assert_eq!(value_of(&context, repo.as_path(), scratch_ref(THE_BATCH).as_str()), RefValue::Names(head), "a record that gave no answer sets no removal in motion, so the batch's reference is where it was");
    remove(root.as_path());
}

/// What:
///  The awaited sweep keeps the commit a dropped batch's reference holds on the kept reference and removes both,
/// exactly as the synchronous sweep does.
/// Why:
///   This is the one thing that reaches a reference whose batch was killed between its import and its intent,
/// and the two faces must settle it the same way or a commit survives on one path and not the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await clearRecordedScratchRefsAwaiting(awaitedLanding, null); clearRecordedScratchRefs(synchronousLanding, null); expect(both kept refs).toEqual(head)
/// ```
#[test]
fn the_awaited_sweep_settles_what_the_synchronous_one_settles() {
    let (root, repo, journal, context, head) = a_repository_with_a_dropped_batch("scratch-record-awaiting-sweeps");
    let (other_root, other_repo, other_journal, other_context, other_head) = a_repository_with_a_dropped_batch("scratch-record-awaiting-sweeps-synchronous");
    record_scratch_ref(journal.as_path(), THE_BATCH, repo.as_path(), repo.join(".git").as_path());
    record_scratch_ref(other_journal.as_path(), THE_BATCH, other_repo.as_path(), other_repo.join(".git").as_path());
    let landing: LandContext = the_landing_context(&context, journal.as_path());
    let other_landing: LandContext = the_landing_context(&other_context, other_journal.as_path());
    let runtime: tokio::runtime::Runtime = one_worker();
    runtime.block_on(clear_recorded_scratch_refs_awaiting(&landing, None));
    clear_recorded_scratch_refs(&other_landing, None);
    assert_eq!(value_of(&context, repo.as_path(), kept_ref(THE_BATCH).as_str()), RefValue::Names(head.clone()), "the awaited sweep keeps the commit the reference held");
    assert_eq!(value_of(&other_context, other_repo.as_path(), kept_ref(THE_BATCH).as_str()), RefValue::Names(other_head), "the synchronous sweep keeps the commit of its own batch");
    assert_eq!(value_of(&context, repo.as_path(), scratch_ref(THE_BATCH).as_str()), RefValue::Absent, "the awaited sweep removes the reference it settled");
    assert_eq!(value_of(&other_context, other_repo.as_path(), scratch_ref(THE_BATCH).as_str()), RefValue::Absent, "the synchronous sweep removes the reference it settled");
    assert!(!record_path(journal.as_path(), THE_BATCH).exists(), "the awaited sweep removes the record it settled");
    assert!(!record_path(other_journal.as_path(), THE_BATCH).exists(), "the synchronous sweep removes the record it settled");
    remove(root.as_path());
    remove(other_root.as_path());
}

/// What:
///  A failed landing's record goes through the awaited face once its reference did,
/// and its commit is kept where the person is told to look.
/// Why:
///   A landing that failed keeps the commit its batch built and removes the reference,
/// so the record that names that reference has nothing left to name;
/// a record left behind would make every later start read a reference that is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await endScratchRefRecordAwaiting(landing, [request], THE_BATCH, Some('rejected')); expect(read(keptRef)).toEqual(head); expect(recordExists()).toBe(false)
/// ```
#[test]
fn a_failed_landings_record_goes_once_its_reference_did() {
    let (root, repo, journal, context, head) = a_repository_with_a_dropped_batch("scratch-record-awaiting-ending");
    record_scratch_ref(journal.as_path(), THE_BATCH, repo.as_path(), repo.join(".git").as_path());
    let landing: LandContext = the_landing_context(&context, journal.as_path());
    let requests: Vec<CommitRequest> = vec![a_request_for(repo.as_path())];
    let runtime: tokio::runtime::Runtime = one_worker();
    runtime.block_on(end_scratch_ref_record_awaiting(&landing, requests.as_slice(), THE_BATCH, Some(&FailureCode::Rejected)));
    assert_eq!(value_of(&context, repo.as_path(), kept_ref(THE_BATCH).as_str()), RefValue::Names(head), "the failed landing's commit is kept on the side reference the message names");
    assert_eq!(value_of(&context, repo.as_path(), scratch_ref(THE_BATCH).as_str()), RefValue::Absent, "the failed landing's reference is removed");
    assert!(!record_path(journal.as_path(), THE_BATCH).exists(), "the record goes with the reference it named");
    remove(root.as_path());
}
