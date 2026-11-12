//! Controls for the awaited take of the index lock:
//! the holder is named to the clients while the wait runs,
//! both faces take a free lock the same way,
//! the markers made ahead and the lock an earlier commit gave up on come back to the landing,
//! and a refused lock keeps the built commit inside a task.

use super::*;
use crate::service_index_lock::{lock_path_of, marker_text};
use crate::service_index_marker::own_identity;
use crate::service_scratch_ref_of_a_dropped_batch::{RefValue, kept_ref, scratch_ref, value_of};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::sync::{Arc, Mutex};

/// The shortest ceiling a control waits out:
/// a held lock is waited for until it,
/// and the wait's own notices are made on the way.
const A_SHORT_CEILING: Duration = Duration::from_millis(300);

/// A ceiling long enough that a notice the clients did not get while they waited is told apart from one they did:
/// the wait's first look comes after its staging and then waits this long,
/// so a notice forwarded when the wait ran arrives in the first part of the whole take
/// and one forwarded only when the job ended arrives at its end.
const A_HELD_CEILING: Duration = Duration::from_secs(5);

/// The longest a control lets a take run before it calls the wait a hang:
/// a take waits for a place of the process's blocking budget before its own ceiling starts,
/// and the host this suite runs on carries other work at the same time,
/// so this is far beyond what a take on a free host needs.
const THE_PATIENCE: Duration = Duration::from_secs(60);

/// A runtime of two workers:
/// one runs the job that waits for the lock and one polls the task that awaits it.
fn two_workers() -> tokio::runtime::Runtime {
    return tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().expect("a runtime of two workers");
}

/// An index of its own with a database beside it,
/// and the foreign `index.lock` this process is proven to hold.
/// Why:
///  The wait names a holder its evidence proves alive and then waits for it until the ceiling,
/// which is the one way a control sees the holder's lines and ends with `IndexInUse`.
fn an_index_held_by_this_process(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let git_dir: PathBuf = root.join("git-dir");
    std::fs::create_dir(&git_dir).expect("a Git directory");
    let index: PathBuf = git_dir.join("index");
    std::fs::write(&index, "old index").expect("an index");
    std::fs::write(lock_path_of(&index), "foreign").expect("a foreign lock");
    // The PID file names this process, which runs and started before the lock was made.
    std::fs::write(crate::recovery_files::lock_pid_path(&index), format!("pid {}\n", std::process::id())).expect("a PID file");
    return (root, index);
}

/// The request a take is asked for:
/// nothing in it is read once the lock is taken,
/// so only the index,
/// the journal and the scratch tree name real places.
fn request_of(index: &Path, journal: &Path, repository: &Path, git_dir: &Path) -> CommitRequest {
    return CommitRequest {
        id: String::from("control"),
        repository: repository.to_path_buf(),
        common_dir: git_dir.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        reference: String::from("refs/heads/control"),
        base: String::from("0").repeat(40),
        changes: Vec::new(),
        message: String::from("control"),
        author: None,
        committer: String::from("Control <control@example.invalid> 0 +0000"),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: index.to_path_buf(),
        scratch: journal.join("scratch"),
        captured: 0,
        holder: String::new(),
    };
}

/// A context whose Git is the fixture's own.
fn context_of(git_dir: &Path) -> GitContext {
    return GitContext { common_dir: Some(git_dir.to_path_buf()), ..git_context() };
}

/// What:
///  The holder of a held lock is named to the clients while the take waits,
///  and the wait ends at its own ceiling with `IndexInUse`.
/// Why:
///   The wait is a job of the blocking pool and the clients are the task's,
/// so every notice crosses to the task as the job makes it;
/// an announcement that waited for the job's end would reach the person after the lock was taken or refused,
/// and a wait that never ended would hold a commit for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = []; const failure = await takeIndexLockAwaiting(..., (note) => said.push(note), ...); expect(said.some((n) => n.holder)).toBe(true)
/// ```
#[test]
fn the_holder_of_a_held_lock_is_named_while_the_take_waits() {
    let (root, index) = an_index_held_by_this_process("take-lock-awaiting-names-the-holder");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir(&journal).expect("a journal directory");
    let first: CommitRequest = request_of(index.as_path(), journal.as_path(), root.join("repo").as_path(), root.join("git-dir").as_path());
    let said: Arc<Mutex<Vec<(Duration, Announce)>>> = Arc::new(Mutex::new(Vec::new()));
    let heard: Arc<Mutex<Vec<(Duration, Announce)>>> = Arc::clone(&said);
    let heard_since: Instant = Instant::now();
    let announcing = move |note: Announce| { heard.lock().expect("the control's list").push((heard_since.elapsed(), note)); };
    let mut timing: LandTiming = LandTiming::default();
    let runtime: tokio::runtime::Runtime = two_workers();
    let began: Instant = Instant::now();
    let answer: Result<IndexLock, FailureCode> = runtime.block_on(take_index_lock_awaiting(
        &context_of(root.join("git-dir").as_path()),
        journal.as_path(),
        A_HELD_CEILING,
        &first,
        4,
        &announcing,
        &mut timing,
        "new",
        "scratch-ref",
    ));
    let took: Duration = began.elapsed();
    let notes: Vec<(Duration, Announce)> = said.lock().expect("the control's list").clone();
    assert!(took < THE_PATIENCE && took >= A_HELD_CEILING, "the awaited take waited the ceiling it was given: {took:?}");
    assert_eq!(answer.err(), Some(FailureCode::IndexInUse), "a lock its holder still holds is refused at the ceiling");
    assert!(!notes.is_empty(), "the wait said nothing at all, so its notices never reached the clients that await it");
    let (first_at, _first): &(Duration, Announce) = &notes[0];
    assert!(*first_at * 2 < took, "the holder's notice reached the clients while the wait still ran ({first_at:?} of {took:?})");
    assert!(
        notes.iter().any(|(_, note)| { return matches!(note, Announce::Waiting(text) if text.contains(format!("PID {}", std::process::id()).as_str())); }),
        "the wait names the holder to the clients while it runs: {notes:?}"
    );
    assert!(
        notes.iter().any(|(_, note)| { return matches!(note, Announce::Holder(_)); }),
        "the clients learn who held the lock when the wait gives up: {notes:?}"
    );
    assert!(timing.lock_holder.is_some(), "the landing's log line names the holder the evidence found");
    remove(root.as_path());
}

/// What:
///  Both faces take a free lock the same way:
///  each holds a lock that names its own batch,
///  and each stages the marker it takes the lock from the same way.
/// Why:
///   The awaited face is the landing's once it is a task,
/// so a lock either face takes must name the same batch and be held the same way;
/// a difference would be two batches sharing one lock's name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(read(lockOf(awaited))).toEqual(markerOf(5)); expect(read(lockOf(synchronous))).toEqual(markerOf(5))
/// ```
#[test]
fn the_awaited_take_holds_the_lock_the_synchronous_one_holds() {
    let runtime: tokio::runtime::Runtime = two_workers();
    let mut markers: Vec<String> = Vec::new();
    let mut kinds: Vec<&'static str> = Vec::new();
    let mut held: Vec<IndexLock> = Vec::new();
    let mut roots: Vec<PathBuf> = Vec::new();
    for awaited in [true, false] {
        let root: PathBuf = fixture(if awaited { "take-lock-awaiting-free-awaited" } else { "take-lock-awaiting-free-synchronous" });
        let git_dir: PathBuf = root.join("git-dir");
        std::fs::create_dir(&git_dir).expect("a Git directory");
        let index: PathBuf = git_dir.join("index");
        std::fs::write(&index, "old index").expect("an index");
        let journal: PathBuf = root.join("journal");
        std::fs::create_dir(&journal).expect("a journal directory");
        let first: CommitRequest = request_of(index.as_path(), journal.as_path(), root.join("repo").as_path(), git_dir.as_path());
        // A landing counts every pass it makes before it asks for the lock, and the first pass's notes are what its log states.
        let mut timing: LandTiming = LandTiming { attempts: 1, ..LandTiming::default() };
        let taken: Result<IndexLock, FailureCode> = if awaited {
            runtime.block_on(take_index_lock_awaiting(&context_of(git_dir.as_path()), journal.as_path(), A_SHORT_CEILING, &first, 5, &|_| {}, &mut timing, "new", "scratch-ref"))
        } else {
            take_index_lock(&context_of(git_dir.as_path()), journal.as_path(), A_SHORT_CEILING, &first, 5, &|_| {}, &mut timing, "new", "scratch-ref")
        };
        let lock: IndexLock = taken.expect("a free lock is taken");
        markers.push(std::fs::read_to_string(lock_path_of(&index)).expect("the lock's text"));
        kinds.push(timing.lock_notes.stage_kind);
        held.push(lock);
        roots.push(root);
    }
    assert_eq!(markers, vec![marker_text(5, &own_identity).expect("text"), marker_text(5, &own_identity).expect("text")], "both faces hold a lock that names the batch they took it for");
    assert_eq!(kinds, vec!["cold", "cold"], "both faces stage the marker they take the lock from the same way");
    drop(held);
    for root in roots {
        remove(root.as_path());
    }
}

/// What:
///  The awaited take starts the markers of the two batches after it,
///  and they land in the landing that asked for the lock,
///  so the next batch finds one ready instead of staging its own.
/// Why:
///   The markers are started after the lock is taken,
/// while the landing holds it and moves the branch,
/// so the next commit meets a finished marker instead of a flush the previous landing began after it ended
/// (9 to 13 s measured on 2026-11-10).
/// A task starts them itself and their slots are its own,
/// which is what this control reads back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// inALanding(() => { await takeIndexLockAwaiting(...); expect(isMade()).toBe(true); expect(take(index, 6)).not.toBeUndefined() })
/// ```
#[test]
fn the_awaited_take_starts_the_markers_of_the_batches_after_it() {
    let root: PathBuf = fixture("take-lock-awaiting-keeps-the-markers");
    let git_dir: PathBuf = root.join("git-dir");
    std::fs::create_dir(&git_dir).expect("a Git directory");
    let index: PathBuf = git_dir.join("index");
    std::fs::write(&index, "old index").expect("an index");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir(&journal).expect("a journal directory");
    let first: CommitRequest = request_of(index.as_path(), journal.as_path(), root.join("repo").as_path(), git_dir.as_path());
    let runtime: tokio::runtime::Runtime = two_workers();
    let mut timing: LandTiming = LandTiming::default();
    let made: Option<crate::service_index_prestage::Prestaged> = crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = serialize_the_markers();
        let lock: IndexLock = runtime
            .block_on(take_index_lock_awaiting(&context_of(git_dir.as_path()), journal.as_path(), A_SHORT_CEILING, &first, 5, &|_| {}, &mut timing, "new", "scratch-ref"))
            .expect("a free lock is taken");
        let staged: Option<crate::service_index_prestage::Prestaged> = take_a_marker_made_ahead(index.as_path(), 6);
        drop(lock);
        return staged;
    });
    assert!(made.is_some(), "the landing holds the marker the awaited take started for the batch after it");
    crate::service_index_prestage::discard();
    remove(root.as_path());
}

/// What:
///  The marker made ahead for `seq`,
///  waited for while the job that makes it finishes.
/// Why:
///   The marker is made beside the landing,
/// so a take reads the slot before the making has returned and then stages its own;
/// the control waits for the finished making rather than for a clock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// while (!isMade()) await sleep(5); return take(index, seq)
/// ```
fn take_a_marker_made_ahead(index: &Path, seq: u64) -> Option<crate::service_index_prestage::Prestaged> {
    let began: Instant = Instant::now();
    while !crate::service_index_prestage::is_made() {
        assert!(began.elapsed() < THE_PATIENCE, "the markers the take started were not made in time");
        std::thread::sleep(Duration::from_millis(5));
    }
    return crate::service_index_prestage::take(index, seq);
}

/// The controls of the markers made ahead serialize,
/// because the slots and the files are the process's.
fn serialize_the_markers() -> std::sync::MutexGuard<'static, ()> {
    static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());
    return ONE_AT_A_TIME.lock().expect("one control at a time");
}

/// What:
///  The lock an earlier take gave up on is remembered by the landing,
///  so the next take is refused at once instead of waiting a whole ceiling for the same file.
/// Why:
///   The memory is the landing's state,
/// which the job of the pool cannot write;
/// without it four commits of one worktree waited 678 to 1,229 s behind one lock on 2026-11-07,
/// each paying a full ceiling for the file the one ahead had already given up on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// inALanding(async () => { await takeIndexLockAwaiting(..., shortCeiling, ...); expect(recall(lock)).toBeDefined(); const again = await takeIndexLockAwaiting(..., longCeiling, ...); expect(again.code).toBe('index-in-use') })
/// ```
#[test]
fn the_lock_a_take_gave_up_on_is_remembered_by_the_landing() {
    let (root, index) = an_index_held_by_this_process("take-lock-awaiting-remembers");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir(&journal).expect("a journal directory");
    let first: CommitRequest = request_of(index.as_path(), journal.as_path(), root.join("repo").as_path(), root.join("git-dir").as_path());
    let runtime: tokio::runtime::Runtime = two_workers();
    let waited: (Result<IndexLock, FailureCode>, bool, Result<IndexLock, FailureCode>, Duration) = crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let mut timing: LandTiming = LandTiming::default();
        let context: GitContext = context_of(root.join("git-dir").as_path());
        let first_answer: Result<IndexLock, FailureCode> =
            runtime.block_on(take_index_lock_awaiting(&context, journal.as_path(), A_SHORT_CEILING, &first, 4, &|_| {}, &mut timing, "new", "scratch-ref"));
        let remembered: bool = crate::service_index_refused_lock::recall(lock_path_of(&index).as_path()).is_some();
        // The same file, waited for again with a ceiling of a minute: a landing that remembers refuses it at once.
        let began: Instant = Instant::now();
        let second: Result<IndexLock, FailureCode> =
            runtime.block_on(take_index_lock_awaiting(&context, journal.as_path(), Duration::from_secs(60), &first, 5, &|_| {}, &mut timing, "new", "scratch-ref"));
        return (first_answer, remembered, second, began.elapsed());
    });
    assert_eq!(waited.0.err(), Some(FailureCode::IndexInUse), "a lock its holder still holds is refused");
    assert!(waited.1, "the lock the take gave up on is the landing's memory, so the next take does not wait for it again");
    assert_eq!(waited.2.err(), Some(FailureCode::IndexInUse), "the same lock is refused to the take behind it");
    assert!(waited.3 < Duration::from_secs(30), "the second take was refused at once rather than waiting its minute: {:?}", waited.3);
    remove(root.as_path());
}

/// What:
///  A take that is refused keeps the commit the batch built on the side reference the person is told about,
///  removes the batch's scratch reference and forgets its intent,
///  inside a task.
/// Why:
///   A refusal lands nothing,
/// so the commit only the scratch reference held would be lost with it,
/// and an intent left behind would make a later start decide a batch that cannot land.
/// All three are the refusal the synchronous face makes,
/// and a task cannot start the Git that keeps the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await takeIndexLockAwaiting(...); expect(valueOf(keptRef(4))).toEqual(head); expect(valueOf(scratchRef(4))).toBe('absent'); expect(exists(intent)).toBe(false)
/// ```
#[test]
fn a_refused_take_keeps_the_commit_and_forgets_the_intent() {
    let root: PathBuf = fixture("take-lock-awaiting-refusal");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["update-ref", scratch_ref(4).as_str(), head.as_str()]);
    // The lock is held by this process, so the take's wait ends at its ceiling with the holder named.
    std::fs::write(lock_path_of(&git_dir.join("index")), "foreign").expect("a foreign lock");
    std::fs::write(crate::recovery_files::lock_pid_path(&git_dir.join("index")), format!("pid {}\n", std::process::id())).expect("a PID file");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir(&journal).expect("a journal directory");
    std::fs::write(crate::service_journal::batch_path(journal.as_path(), 4, crate::service_journal::OPEN_EXTENSION), "an intent").expect("an intent");
    let first: CommitRequest = request_of(git_dir.join("index").as_path(), journal.as_path(), repo.as_path(), git_dir.as_path());
    let said: Arc<Mutex<Vec<Announce>>> = Arc::new(Mutex::new(Vec::new()));
    let heard: Arc<Mutex<Vec<Announce>>> = Arc::clone(&said);
    let announcing = move |note: Announce| { heard.lock().expect("the control's list").push(note); };
    let mut timing: LandTiming = LandTiming::default();
    let runtime: tokio::runtime::Runtime = two_workers();
    let answer: Result<IndexLock, FailureCode> = runtime.block_on(take_index_lock_awaiting(
        &context_of(git_dir.as_path()),
        journal.as_path(),
        A_SHORT_CEILING,
        &first,
        4,
        &announcing,
        &mut timing,
        head.as_str(),
        scratch_ref(4).as_str(),
    ));
    assert_eq!(answer.err(), Some(FailureCode::IndexInUse), "a lock its holder still holds is refused");
    let context: GitContext = context_of(git_dir.as_path());
    assert_eq!(value_of(&context, repo.as_path(), kept_ref(4).as_str()), RefValue::Names(head), "the refused take keeps the commit its batch built");
    assert_eq!(value_of(&context, repo.as_path(), scratch_ref(4).as_str()), RefValue::Absent, "the refused take removes the batch's scratch reference");
    assert!(!crate::service_journal::batch_path(journal.as_path(), 4, crate::service_journal::OPEN_EXTENSION).exists(), "the refused take forgets the batch's intent");
    let notes: Vec<Announce> = said.lock().expect("the control's list").clone();
    assert!(
        notes.iter().any(|note| { return matches!(note, Announce::Waiting(text) if text.contains(kept_ref(4).as_str())); }),
        "the person is told where the commit is kept: {notes:?}"
    );
    remove(root.as_path());
}
