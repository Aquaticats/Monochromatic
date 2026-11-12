//! Controls for the awaited copy of a batch's conclusion state:
//! a copy that gives no answer holds no worker,
//! the awaited face keeps what the synchronous one keeps,
//! and a copy no job promised is answered as one that removes nothing.

use super::*;
use crate::blocking_jobs_within_the_budget::a_controls_answer::{AStatesAnswer, state_the_answer};
use crate::test_support::{fixture, remove};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// How often a control looks at the flag a task beside the work sets.
const THE_TICK: Duration = Duration::from_millis(50);

/// The most a control waits for that task,
/// far beyond what a free worker needs,
/// so only a held worker reaches it.
const THE_PATIENCE: Duration = Duration::from_secs(10);

/// The most a control waits for the writer that lets a stalled read go.
const THE_WRITER_PATIENCE: Duration = Duration::from_secs(10);

/// What:
///  A scratch directory with the conclusion state a landing compares:
/// a copied directory beside the shadow's state files.
/// Why:
///  The copy reads both,
/// so both must be there for the awaited face to keep what the synchronous one keeps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scratch = mkdtemp(); writeFile(join(scratch, 'conclusion/MERGE_RR'), 'rr'); writeFile(join(scratch, 'shadow/MERGE_HEAD'), 'head')
/// ```
fn a_scratch_directory(root: &Path) -> PathBuf {
    let scratch: PathBuf = root.join("scratch");
    let copies: PathBuf = scratch.join(CONCLUSION_COPY_DIRECTORY);
    std::fs::create_dir_all(&copies).expect("the copy directory of a preparation");
    std::fs::write(copies.join("MERGE_RR"), "a resolution cache").expect("a file of the copied directory");
    let shadow: PathBuf = scratch.join("shadow");
    std::fs::create_dir_all(&shadow).expect("the shadow directory of a preparation");
    std::fs::write(shadow.join("MERGE_HEAD"), "a merge head").expect("a state file of the shadow");
    std::fs::write(shadow.join("MERGE_MSG"), "a merge message").expect("a state file of the shadow");
    return scratch;
}

/// The cleanup a batch lands with:
/// the state a merge,
/// a cherry-pick or a revert left in the person's worktree.
fn cleanup_of(root: &Path, scratch: PathBuf) -> Cleanup {
    return Cleanup { git_dir: root.join("git-dir"), scratch, reftable: false };
}

/// Every file under `root`,
/// by its name relative to `root`,
/// with its bytes as text,
/// so two copies are compared whole.
fn tree_of(root: &Path) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return found;
    };
    for entry in entries.flatten() {
        let path: PathBuf = entry.path();
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            for (deeper, text) in tree_of(path.as_path()) {
                found.push((format!("{name}/{deeper}"), text));
            }
            continue;
        }
        found.push((name, std::fs::read_to_string(path.as_path()).unwrap_or_default()));
    }
    found.sort();
    return found;
}

/// What:
///  Stand a pipe with no writer where a state file of the shadow belongs,
/// so the copy of it waits in the kernel until a writer appears.
/// Why:
///  `std::fs::copy` opens its source for reading,
/// and an open of a pipe for reading waits for a writer,
/// which is what a disk that stopped answering does to every read.
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
    assert_eq!(made, 0, "the pipe must stand where the state file would be: {}", std::io::Error::last_os_error());
}

/// What:
///  Open the pipe at `path` for writing,
/// which ends the open for reading that waited for a writer,
/// write one line and let it go.
/// Why:
///  The read of the pool is abandoned,
/// never cancelled,
/// so the thread that holds it ends only when a writer appears,
/// and an open for writing of a pipe nobody reads yet is refused,
/// so the control asks again until `THE_WRITER_PATIENCE`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (;;) { try { const writer = openSync(path, 'w' | O_NONBLOCK); writeSync(writer, 'x'); closeSync(writer); return } catch { await sleep(10) } }
/// ```
fn write_into_the_pipe(path: &Path) {
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    let began: Instant = Instant::now();
    while began.elapsed() < THE_WRITER_PATIENCE {
        match std::fs::OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open(path) {
            Ok(mut writer) => {
                let _written = writer.write_all(b"a merge head");
                return;
            }
            Err(_nobody_reads_yet) => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}

/// A runtime of one worker,
/// so a task that does storage itself is the one a control sees holding it.
fn one_worker() -> tokio::runtime::Runtime {
    return tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().expect("a runtime of one worker");
}

/// What:
///  A task beside the copy runs while the copy waits for a state file that gives no answer,
///  and the landing is answered instead of waiting without end.
/// Why:
///   The copy reads a tree and writes another,
/// which waits in the kernel on a disk that stopped answering;
/// a task that made the copy itself would hold a worker the runtime needs,
/// and the branch may not move before the copy is there.
/// The copy of a state file that gives no answer is not promised to the landing,
/// so it never takes the name the cleanup reads and removes nothing of the person's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(join(scratch, 'shadow/MERGE_HEAD')); const keeping = spawn(() => keepOfIntentAwaiting(...)); assert(await runsBeside(keeping))
/// ```
#[test]
fn a_copy_that_gives_no_answer_does_not_hold_a_worker() {
    let root: PathBuf = fixture("conclusion-copy-awaiting-holds");
    let scratch: PathBuf = a_scratch_directory(root.as_path());
    let refused: PathBuf = scratch.join("shadow").join("MERGE_HEAD");
    std::fs::remove_file(refused.as_path()).expect("the state file a pipe stands in place of");
    stall_the_file(refused.as_path());
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir(&journal).expect("a journal directory");
    let cleanup: Cleanup = cleanup_of(root.as_path(), scratch);
    let runtime: tokio::runtime::Runtime = one_worker();
    let ticked: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let tick: Arc<AtomicBool> = Arc::clone(&ticked);
    let _ticking = runtime.spawn(async move {
        tokio::time::sleep(THE_TICK).await;
        tick.store(true, Ordering::SeqCst);
    });
    let keeping_journal: PathBuf = journal.clone();
    let keeping = runtime.spawn(async move { return keep_of_intent_awaiting(Some(&cleanup), keeping_journal.as_path(), 4).await; });
    let began: Instant = Instant::now();
    while !ticked.load(Ordering::SeqCst) && began.elapsed() < THE_PATIENCE {
        std::thread::sleep(THE_TICK);
    }
    let ran_beside: bool = ticked.load(Ordering::SeqCst);
    // The pipe is given a writer before the answer is read, so a copy that held the worker cannot hold this control too.
    write_into_the_pipe(refused.as_path());
    let kept: Option<PathBuf> = runtime.block_on(keeping).expect("the keeping task ends");
    assert!(ran_beside, "the task beside a copy that gives no answer still runs, so no worker is held by that copy");
    assert_eq!(kept, None, "a copy of a state file that gives no answer is not promised to the landing, so nothing of the person's is removed");
    assert!(!path_of(journal.as_path(), 4).exists(), "a copy that failed never takes the name the cleanup reads");
    assert!(!journal.join("4.cleanup.tmp").exists(), "the staging of a copy that failed goes with it");
    remove(root.as_path());
}

/// What:
///  The awaited copy keeps what the synchronous one keeps,
///  and the batch with no cleanup keeps nothing.
/// Why:
///   The two faces must keep one copy,
/// because the cleanup a landed commit makes compares what is in it,
/// and a face that kept another tree would remove a state file the person still has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(treeOf(await keepOfIntentAwaiting(cleanup, awaitedJournal, 5))).toEqual(treeOf(keepOfIntent(cleanup, synchronousJournal, 5)))
/// ```
#[test]
fn the_awaited_copy_keeps_what_the_synchronous_one_keeps() {
    let root: PathBuf = fixture("conclusion-copy-awaiting-agrees");
    let scratch: PathBuf = a_scratch_directory(root.as_path());
    let cleanup: Cleanup = cleanup_of(root.as_path(), scratch);
    let awaited: PathBuf = root.join("awaited");
    let synchronous: PathBuf = root.join("synchronous");
    std::fs::create_dir(&awaited).expect("a journal directory");
    std::fs::create_dir(&synchronous).expect("a journal directory");
    let runtime: tokio::runtime::Runtime = one_worker();
    let kept: Option<PathBuf> = runtime.block_on(keep_of_intent_awaiting(Some(&cleanup), awaited.as_path(), 5));
    assert_eq!(kept, Some(path_of(awaited.as_path(), 5)), "the awaited face keeps the copy under the batch's name");
    let made: Option<PathBuf> = keep_of_intent(Some(&cleanup), synchronous.as_path(), 5);
    assert_eq!(made, Some(path_of(synchronous.as_path(), 5)), "the synchronous face keeps the copy under the batch's name");
    let theirs: Vec<(String, String)> = tree_of(path_of(synchronous.as_path(), 5).as_path());
    assert!(!theirs.is_empty(), "the synchronous copy holds the conclusion state: {theirs:?}");
    assert_eq!(tree_of(path_of(awaited.as_path(), 5).as_path()), theirs, "the two faces keep one tree of conclusion state");
    let nothing: Option<PathBuf> = runtime.block_on(keep_of_intent_awaiting(None, awaited.as_path(), 6));
    assert_eq!(nothing, None, "a batch with no cleanup keeps no copy");
    remove(root.as_path());
}

/// What:
///  A copy the system gave no thread for,
///  or one that gave no answer in time,
///  is not promised to the landing,
///  so the cleanup that reads it removes nothing.
/// Why:
///   The state a landed merge would remove is the person's,
/// and a copy a landing cannot prove is the one it made is no proof;
/// the safe side is to leave the state where it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const answer of ['notRun', 'noAnswer']) { stateTheAnswer(answer); expect(await keepOfIntentAwaiting(cleanup, journal, 7)).toBeUndefined() }
/// ```
#[test]
fn a_copy_no_job_promised_is_answered_as_one_that_removes_nothing() {
    let root: PathBuf = fixture("conclusion-copy-awaiting-no-answer");
    let scratch: PathBuf = a_scratch_directory(root.as_path());
    let cleanup: Cleanup = cleanup_of(root.as_path(), scratch);
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir(&journal).expect("a journal directory");
    let runtime: tokio::runtime::Runtime = one_worker();
    for answer in [AStatesAnswer::NotRun, AStatesAnswer::NoAnswer] {
        let _stated = state_the_answer(answer);
        let kept: Option<PathBuf> = runtime.block_on(keep_of_intent_awaiting(Some(&cleanup), journal.as_path(), 7));
        assert_eq!(kept, None, "a copy no job promised is not promised to the landing");
    }
    remove(root.as_path());
}
