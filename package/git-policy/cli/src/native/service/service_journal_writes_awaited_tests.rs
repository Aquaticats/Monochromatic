//! Controls for the awaited journal writes:
//! a journal record that gives no answer holds no worker,
//! the awaited faces write what the synchronous ones write,
//! and a landing's deferral and its sync figures survive the job that does the write.

use super::*;
use crate::blocking_jobs_within_the_budget::a_controls_answer::{AStatesAnswer, state_the_answer};
use crate::service_journal::{DONE_EXTENSION, OPEN_EXTENSION, START_EXTENSION, Started, TEMPORARY_EXTENSION, batch_path, mark_done, read_start, remove_start, write_intent, write_start};
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

/// An intent of a repository no test writes to:
/// the write of it only makes the journal's own file.
fn intent(seq: u64) -> Intent {
    return Intent {
        seq,
        repository: PathBuf::from("/work/repo"),
        common_dir: PathBuf::from("/work/repo/.git"),
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
        old: String::from("a").repeat(40),
        new: String::from("b").repeat(40),
        ids: vec![String::from("id-1")],
        commits: vec![String::from("c").repeat(40)],
        holders: Vec::new(),
        paths: vec![b"src/a b.txt".to_vec()],
        derived: false,
        cleanup: None,
        index: PathBuf::from("/work/repo/.git/index"),
        git_dir: PathBuf::from("/work/repo/.git"),
    };
}

/// What:
///  Stand a pipe with no reader at `path`,
/// so a write that opens it waits in the kernel until a reader appears.
/// Why:
///  This is the one way to make the write of a record hold its thread with no end,
/// which is what a disk that stopped answering does.
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
    assert_eq!(made, 0, "the pipe must stand where the record's file would be: {}", std::io::Error::last_os_error());
}

/// What:
///  Open the pipe at `path` for reading,
/// which ends the wait of a write that opened it,
/// and hold it open until the control lets it go.
/// Why:
///  The open of a pipe for writing returns once a reader has it,
/// so the record's own write goes on from there and the control reads what it answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = openSync(path, constants.O_RDONLY | constants.O_NONBLOCK);
/// ```
fn open_the_reader(path: &Path) -> std::fs::File {
    use std::os::unix::fs::OpenOptionsExt as _;
    return std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK).open(path).expect("a reader opens the pipe the record waited for");
}

/// A runtime of one worker,
/// so a task that does storage itself is the one a control sees holding it.
fn one_worker() -> tokio::runtime::Runtime {
    return tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().expect("a runtime of one worker");
}

/// What:
///  A task beside the intent's write runs while the write waits,
/// and the landing is answered instead of waiting without end.
/// Why:
///   The intent's write is a file and its flush,
/// which wait in the kernel on a disk that stopped answering;
/// a task that did the write itself would hold a worker the runtime needs,
/// and no timer on that worker ends it.
/// The batch is answered with the failure the write gave,
/// so its branch does not move and the next service decides the batch from the branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(join(journal, '4.tmp')); const writing = spawn(() => writeIntentAwaiting(journal, intent(4))); assert(await runsBeside(writing))
/// ```
#[test]
fn an_intent_write_that_gives_no_answer_does_not_hold_a_worker() {
    let root: PathBuf = fixture("journal-awaiting-stalls");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir(&journal).expect("a journal directory");
    let temporary: PathBuf = batch_path(journal.as_path(), 4, TEMPORARY_EXTENSION);
    stall_the_file(temporary.as_path());
    let runtime: tokio::runtime::Runtime = one_worker();
    let ticked: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let tick: Arc<AtomicBool> = Arc::clone(&ticked);
    let _ticking = runtime.spawn(async move {
        tokio::time::sleep(THE_TICK).await;
        tick.store(true, Ordering::SeqCst);
    });
    let writing_journal: PathBuf = journal.clone();
    let writing = runtime.spawn(async move { return write_intent_awaiting(writing_journal.as_path(), &intent(4)).await; });
    let began: Instant = std::time::Instant::now();
    while !ticked.load(Ordering::SeqCst) && began.elapsed() < THE_PATIENCE {
        std::thread::sleep(THE_TICK);
    }
    let ran_beside: bool = ticked.load(Ordering::SeqCst);
    // The pipe is given a reader before the answer is read, so a write that held the worker cannot hold this control too.
    let reading: std::fs::File = open_the_reader(temporary.as_path());
    let answered: std::io::Result<()> = runtime.block_on(writing).expect("the writing task ends");
    drop(reading);
    assert!(ran_beside, "the task beside an intent write that gives no answer still runs, so no worker is held by that write");
    assert!(answered.is_err(), "a record whose file could not be flushed is answered as a failure, so the landing ends instead of landing a batch nothing recorded");
    remove(root.as_path());
}

/// What:
///  The awaited intent write writes the same file as the synchronous one,
///  and the landing's deferred flushes stay the landing's through the job.
/// Why:
///   The job of the pool cannot read or write the landing's state
/// (`landing_state_carried_by_its_task.rs`),
/// so the deferral travels into it and the syncs it made travel back;
/// a job that flushed the intent itself would make a person's commit wait for the disk,
/// which is what `service_durable.rs` keeps off the commit's path,
/// and one that recorded nothing would lose the figures its answer states.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// inALanding(() => { beginDeferral(); await writeIntentAwaiting(dir, intent); expect(syncsWord(takeSyncs())).toBe('intent-file=queued;intent-dir=queued') })
/// ```
#[test]
fn the_awaited_intent_write_keeps_the_flushes_the_landing_defers() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let root: PathBuf = fixture("journal-awaiting-defers");
        let journal: PathBuf = root.join("journal");
        std::fs::create_dir(&journal).expect("a journal directory");
        let deferral = crate::service_durable::begin(false);
        let runtime: tokio::runtime::Runtime = one_worker();
        let written: std::io::Result<()> = runtime.block_on(write_intent_awaiting(journal.as_path(), &intent(4)));
        assert!(written.is_ok(), "the intent of a free journal is written: {written:?}");
        let kept: String = std::fs::read_to_string(batch_path(journal.as_path(), 4, OPEN_EXTENSION)).expect("the intent's file");
        // The record also carries the reading of the clocks it was written by (`service_journal_record_age.rs`), which no two
        //  writes share; the intent the two faces write is compared as the record reads back.
        assert_eq!(crate::service_journal::parse_intent(without_the_age(kept.as_str()).as_str()), Some(intent(4)), "the awaited face writes the intent the synchronous one writes");
        assert!(kept.lines().any(|line| { return line.starts_with("stamp="); }), "and the reading of the clocks it was written by: {kept}");
        let figures: Vec<crate::service_durable_syncs::SyncFigure> = crate::service_durable::take_syncs();
        let named: String = crate::service_durable_syncs::syncs_word(figures.as_slice());
        assert_eq!(named, "intent-file=queued;intent-dir=queued", "the flushes of the intent are the landing's to settle, so nothing waited for the disk");
        deferral.settle(true);
        remove(root.as_path());
    });
}

/// What:
///  The awaited start record is the record the synchronous face writes,
///  and a job that gave no answer is answered as a record that could not be written.
/// Why:
///   A landing writes the start record while it holds the index lock,
/// so the write waits on the pool and the landing goes on with the same file;
/// a batch whose record is missing is a batch no replay is made for,
/// which is the answer a record that could not be written gives already.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeStartAwaiting(dir, 5, witness); expect(readStart(dir, 5)).toEqual(syncAnswer)
/// ```
#[test]
fn the_awaited_start_record_is_the_one_the_synchronous_face_writes() {
    let root: PathBuf = fixture("journal-awaiting-start");
    let awaited: PathBuf = root.join("awaited");
    let synchronous: PathBuf = root.join("synchronous");
    std::fs::create_dir(&awaited).expect("a journal directory");
    std::fs::create_dir(&synchronous).expect("a journal directory");
    let witness: IndexWitness = IndexWitness::File { inode: 7, size: 96, modified: (1_700_000_000, 4), changed: (1_700_000_001, 5), trailer: String::from("ab") };
    let runtime: tokio::runtime::Runtime = one_worker();
    let written: std::io::Result<()> = runtime.block_on(write_start_awaiting(awaited.as_path(), 5, &witness));
    assert!(written.is_ok(), "the start record of a free journal is written: {written:?}");
    write_start(synchronous.as_path(), 5, &witness).expect("the synchronous record");
    assert_eq!(read_start(awaited.as_path(), 5), Started::Recorded(witness.clone()), "the awaited face records the witness the synchronous one records");
    assert_eq!(
        std::fs::read_to_string(batch_path(awaited.as_path(), 5, START_EXTENSION)).expect("the awaited record"),
        std::fs::read_to_string(batch_path(synchronous.as_path(), 5, START_EXTENSION)).expect("the synchronous record"),
        "the two faces write one record"
    );
    for answer in [AStatesAnswer::NotRun, AStatesAnswer::NoAnswer] {
        let _stated = state_the_answer(answer);
        let refused: std::io::Result<()> = runtime.block_on(write_start_awaiting(root.join("unwritten").as_path(), 6, &witness));
        assert!(refused.is_err(), "a start record no job wrote in time is answered as one that could not be written");
    }
    remove(root.as_path());
}

/// What:
///  The completion of a batch the awaited face makes is the one the synchronous face makes:
/// the same record files are left,
/// with the same contents,
/// and the syncs the landing defers are the job's.
/// Why:
///   A restart decides a batch from the records a completion leaves,
/// so a face that renamed or removed another set of them would decide another batch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// markDone(one, 3); await markDoneAwaiting(other, 3); expect(records(one)).toEqual(records(other))
/// ```
#[test]
fn the_awaited_completion_leaves_what_the_synchronous_one_leaves() {
    let root: PathBuf = fixture("journal-awaiting-completion");
    let awaited: PathBuf = root.join("awaited");
    let synchronous: PathBuf = root.join("synchronous");
    let witness: IndexWitness = IndexWitness::File { inode: 9, size: 120, modified: (1_700_000_002, 6), changed: (1_700_000_003, 7), trailer: String::from("cd") };
    let mut started: Vec<PathBuf> = Vec::new();
    for dir in [awaited.as_path(), synchronous.as_path()] {
        std::fs::create_dir(dir).expect("a journal directory");
        write_intent(dir, &intent(3)).expect("the intent");
        write_start(dir, 3, &witness).expect("the start record");
        // What a landing that installed its index leaves beside the intent before the batch is marked done.
        std::fs::write(batch_path(dir, 3, "indexed"), b"aa".repeat(20)).expect("the indexed record");
        started.push(dir.to_path_buf());
    }
    let runtime: tokio::runtime::Runtime = one_worker();
    let marked: std::io::Result<()> = runtime.block_on(mark_done_awaiting(awaited.as_path(), 3));
    assert!(marked.is_ok(), "the completion of a free journal is written: {marked:?}");
    mark_done(synchronous.as_path(), 3).expect("the synchronous completion");
    assert_eq!(records_of(awaited.as_path(), 3), records_of(synchronous.as_path(), 3), "the two faces leave one set of records");
    assert_eq!(
        crate::service_journal::parse_intent(without_the_age(finished_of(awaited.as_path()).as_str()).as_str()),
        crate::service_journal::parse_intent(without_the_age(finished_of(synchronous.as_path()).as_str()).as_str()),
        "the finished record of the two faces reads back as one intent"
    );
    assert!(batch_path(awaited.as_path(), 3, "done").exists(), "the batch's open record became its finished one");
    assert!(!batch_path(awaited.as_path(), 3, OPEN_EXTENSION).exists(), "the open record is gone");
    assert!(!batch_path(awaited.as_path(), 3, START_EXTENSION).exists(), "the start record is gone");
    assert!(!batch_path(awaited.as_path(), 3, "indexed").exists(), "the record of the installed index is gone");
    for answer in [AStatesAnswer::NotRun, AStatesAnswer::NoAnswer] {
        let _stated = state_the_answer(answer);
        let refused: std::io::Result<()> = runtime.block_on(mark_done_awaiting(root.join("unwritten").as_path(), 4));
        assert!(refused.is_err(), "a completion no job made in time is answered as one that could not be written");
    }
    remove(root.as_path());
}

/// What:
///  The start record the awaited face takes away is the one the synchronous face takes away,
/// and neither answers anything.
/// Why:
///   The record is only an obligation to replay,
/// and a replay over a record that is gone is the answer both faces give a commit that owes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeStart(one, 2); await removeStartAwaiting(other, 2); expect(records(one)).toEqual(records(other))
/// ```
#[test]
fn the_awaited_removal_takes_away_what_the_synchronous_one_takes_away() {
    let root: PathBuf = fixture("journal-awaiting-removal");
    let awaited: PathBuf = root.join("awaited");
    let synchronous: PathBuf = root.join("synchronous");
    let witness: IndexWitness = IndexWitness::File { inode: 11, size: 64, modified: (1_700_000_004, 8), changed: (1_700_000_005, 9), trailer: String::from("ef") };
    for dir in [awaited.as_path(), synchronous.as_path()] {
        std::fs::create_dir(dir).expect("a journal directory");
        write_start(dir, 2, &witness).expect("the start record");
        std::fs::write(batch_path(dir, 2, "indexed"), b"bb".repeat(20)).expect("a record that is not the start's");
    }
    let runtime: tokio::runtime::Runtime = one_worker();
    runtime.block_on(remove_start_awaiting(awaited.as_path(), 2));
    remove_start(synchronous.as_path(), 2);
    assert_eq!(records_of(awaited.as_path(), 2), records_of(synchronous.as_path(), 2), "the two faces leave one set of records");
    assert!(!batch_path(awaited.as_path(), 2, START_EXTENSION).exists(), "the start record is gone");
    assert!(batch_path(awaited.as_path(), 2, "indexed").exists(), "the record that is not the start's is left where it is");
    remove(root.as_path());
}

/// The records of one batch that a directory holds:
/// every file whose name begins with the batch's number.
fn records_of(dir: &Path, seq: u64) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(dir).expect("the journal directory").flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(format!("{seq}.").as_str()) {
            found.push(name);
        }
    }
    found.sort();
    return found;
}

/// The finished record of one batch,
/// as the file reads.
fn finished_of(dir: &Path) -> String {
    return std::fs::read_to_string(batch_path(dir, 3, DONE_EXTENSION)).expect("the finished record");
}

/// A record without the line that carries the reading of the clocks it was written by,
/// which no two writes share.
fn without_the_age(text: &str) -> String {
    return text.lines().filter(|line| { return !line.starts_with("stamp="); }).collect::<Vec<&str>>().join("\n");
}
