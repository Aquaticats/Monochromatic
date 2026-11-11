//! Controls for the markers made ahead of the next batches:
//! one is taken and placed without a flush on the commit's path,
//! a commit that comes while one is still being made never waits for it and stages its own,
//! a marker for another batch or index is removed and replaced by a cold one,
//! a marker that will not be used is let go without waiting for its making,
//! whether it is still flushing,
//! still queued behind another job of the pool,
//! or made for another index,
//! the files kept are bounded by `MARKERS_MADE_AHEAD` and the spare of the next batch survives a take,
//! the service's last sweep leaves no marker the making had not recorded yet,
//! and nothing is left when the service ends.

use super::*;
use crate::service_index_lock::{acquire_noting_wait, marker_text};
use crate::service_index_marker::own_identity;
use crate::service_index_placement::sync_file;
use crate::test_support::{fixture, remove};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The controls that count flushes run one at a time:
/// the counter is the process's.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// How many flushes `slow_flush` was asked for.
static FLUSHES: AtomicUsize = AtomicUsize::new(0);

/// How long a slow flush takes in the controls.
const SLOW: Duration = Duration::from_millis(400);

/// A flush that takes as long as a loaded disk makes it take.
fn slow_flush(file: &std::fs::File) -> std::io::Result<()> {
    FLUSHES.fetch_add(1, Ordering::SeqCst);
    std::thread::sleep(SLOW);
    return file.sync_all();
}

/// The directory of an index,
/// with the index.
fn index_in(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let git_dir: PathBuf = root.join("git-dir");
    std::fs::create_dir(&git_dir).expect("directory");
    let index: PathBuf = git_dir.join("index");
    std::fs::write(&index, "old index").expect("index");
    return (root, index);
}

/// The names in the index's directory other than the index and its lock.
fn leftovers(index: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(index.parent().expect("directory"))
        .expect("listing")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
        .filter(|name| { return name != "index" && name != "index.lock"; })
        .collect();
    names.sort();
    return names;
}

/// Waits until the marker made ahead is on its way to being ready:
/// its file exists.
fn until_staged(index: &Path) {
    let began: Instant = Instant::now();
    while leftovers(index).is_empty() {
        assert!(began.elapsed() < Duration::from_secs(10), "no marker was made ahead");
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// Waits until `want` markers made ahead are made and flushed.
fn until_finished(index: &Path, want: usize) {
    let began: Instant = Instant::now();
    while finished_markers() < want {
        assert!(began.elapsed() < Duration::from_secs(60), "{} of {want} markers were never finished: {:?}", finished_markers(), leftovers(index));
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// The texts of the markers waiting in the index's directory,
/// which name the batch each was made for.
fn texts_of(index: &Path) -> Vec<String> {
    let directory: &Path = index.parent().expect("directory");
    let mut texts: Vec<String> = leftovers(index).iter().map(|name| { return std::fs::read_to_string(directory.join(name)).expect("the marker's text"); }).collect();
    texts.sort();
    return texts;
}

/// A marker made ahead and finished while the service was idle is placed without a flush on the commit's path:
/// the commit asked for no flush (the count of flushes,
/// not a clock,
/// says so),
/// the notes say the file came from the idle time,
/// and its text is the batch's.
#[test]
fn a_marker_made_ahead_is_placed_without_a_flush_at_the_commit() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let (root, index) = index_in("prestage-ready");
        FLUSHES.store(0, Ordering::SeqCst);
        start_with(slow_flush, &index, 7);
        // The idle time: until the flush is over, whatever the disk makes of it.
        let idle_began: Instant = Instant::now();
        while !is_made() {
            assert!(idle_began.elapsed() < Duration::from_secs(60), "the flush never ended");
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(FLUSHES.load(Ordering::SeqCst), 1, "the flush ran while the service was idle");
        FLUSHES.store(0, Ordering::SeqCst);
        let mut notes = crate::service_index_lock::WaitNotes::default();
        let lock = acquire_noting_wait(&index, 7, &|_| {}, &mut notes).expect("taken");
        assert_eq!(FLUSHES.load(Ordering::SeqCst), 0, "the commit flushed nothing: the flush was the idle time's");
        assert_eq!(notes.stage_kind, "pre");
        assert_eq!(std::fs::read_to_string(crate::service_index_lock::lock_path_of(&index)).expect("marker"), marker_text(7, &own_identity).expect("text"));
        assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
        drop(lock);
        remove(root.as_path());
    });
}

/// Set when the gated flush has finished.
static GATED_FLUSH_DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Holds the flush until a commit is waiting for the marker,
/// so the commit comes during the flush on any host.
fn until_a_commit_waits() {
    let began: Instant = Instant::now();
    while !COMMIT_WAITING.load(Ordering::SeqCst) {
        // The bound only ends a failing run: the commit that waits is what opens the gate.
        assert!(began.elapsed() < Duration::from_secs(60), "no commit came to wait for the flush");
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Set when a held flush may finish.
static FLUSH_MAY_FINISH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The longest a held flush waits to be let go.
/// Why:
///  A control that expects a landing to wait for the flush lets it go only after the lock is taken,
/// and a landing that waits
///  would hold the flush's gate forever;
/// the ceiling ends such a run instead of hanging it,
/// and it is far past the
///  cold staging a landing does on this disk (a flush of its own marker,
/// which waits on the whole filesystem's dirty data),
/// so
///  a control that reads the held marker's file as still being written is reading a fact and not a race.
const HOLD_CEILING: Duration = Duration::from_secs(120);

/// What:
///  Waits for the commit,
/// then for the control to let this flush go,
/// which is what makes "the flush was still running when the lock was
///  taken" a fact and not a race against a clock.
/// Why:
///   The flush is not asked of this disk:
/// a flush here waits on the whole filesystem's dirty data and was measured past
///  a minute under this host's load,
/// which no control's bound can hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// untilACommitWaits(); const held = now(); while (!FLUSH_MAY_FINISH && now() - held < HOLD_CEILING) sleep(1);
/// ```
fn hold_until_let_go() {
    FLUSHES.fetch_add(1, Ordering::SeqCst);
    until_a_commit_waits();
    let held: Instant = Instant::now();
    while !FLUSH_MAY_FINISH.load(Ordering::SeqCst) && held.elapsed() < HOLD_CEILING {
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// The flushes the control's own held flush has ended,
/// so a control waits for the last write of a job it let go before it removes the directory.
static HELD_FLUSHES_ENDED: AtomicUsize = AtomicUsize::new(0);

/// The longest a control holds a flush whose marker another batch or another index lets go:
/// a product that waits for that flush ends it at this ceiling,
/// and the control then reads the wait from `HELD_FLUSHES_ENDED` instead of hanging the run.
const THE_CEILING_OF_A_HELD_FLUSH: Duration = Duration::from_secs(2);

/// A flush the control holds until it lets it go,
/// which says so when it has ended.
/// Why:
///  A control that proves nothing waits for a marker still being made needs the making to be held while it looks:
/// the marker's file exists and its batch is passed,
/// and no clock decides that.
fn flush_held_by_the_control(file: &std::fs::File) -> std::io::Result<()> {
    FLUSHES.fetch_add(1, Ordering::SeqCst);
    let held: Instant = Instant::now();
    while !FLUSH_MAY_FINISH.load(Ordering::SeqCst) && held.elapsed() < THE_CEILING_OF_A_HELD_FLUSH {
        std::thread::sleep(Duration::from_millis(1));
    }
    HELD_FLUSHES_ENDED.fetch_add(1, Ordering::SeqCst);
    return file.sync_all();
}

/// Waits until `want` flushes are held.
fn until_flushes_are_held(want: usize) {
    let began: Instant = Instant::now();
    while FLUSHES.load(Ordering::SeqCst) < want {
        assert!(began.elapsed() < Duration::from_secs(10), "{} of {want} flushes were held", FLUSHES.load(Ordering::SeqCst));
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Lets the held flushes go and waits until `want` of them have ended,
/// so nothing still writes into a directory a control removes.
fn let_the_held_flushes_go(want: usize) {
    FLUSH_MAY_FINISH.store(true, Ordering::SeqCst);
    let began: Instant = Instant::now();
    while HELD_FLUSHES_ENDED.load(Ordering::SeqCst) < want {
        assert!(began.elapsed() < Duration::from_secs(10), "{} of {want} held flushes ended", HELD_FLUSHES_ENDED.load(Ordering::SeqCst));
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// A flush held until the control lets it go.
fn held_flush(_: &std::fs::File) -> std::io::Result<()> {
    hold_until_let_go();
    GATED_FLUSH_DONE.store(true, Ordering::SeqCst);
    return Ok(());
}

/// A flush held until the control lets it go,
/// which then fails.
fn held_failing_flush(_: &std::fs::File) -> std::io::Result<()> {
    hold_until_let_go();
    GATED_FLUSH_DONE.store(true, Ordering::SeqCst);
    return Err(std::io::Error::from_raw_os_error(libc::EIO));
}

/// What:
///  A commit that comes while the marker is still being flushed takes the lock the ordinary way and never waits for that
/// flush,
/// and the marker that was still being flushed is removed once it is finished,
/// so nothing is left in the index's directory.
/// Why:
///   The flush is seconds long on a loaded disk,
/// and a landing waited for it on the commit's path
///  (`lock_stage_kind=pre`,
/// 9 to 13 s measured 2026-11-10);
/// the owner's bar is that a commit through the wrapper is not five
///  seconds slower than native Git,
/// so a landing stands aside from a flush still running instead.
/// The flush is held until the commit
///  is waiting for it,
/// so the commit provably came during the flush.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// start(gatedSlowFlush, index, 8); untilStaged(index); const lock = await acquire(index, 8);
/// expect(notes.stageKind).toBe('cold'); discard(); expect(leftovers(index)).toEqual([]);
/// ```
#[test]
fn a_commit_that_comes_during_the_flush_takes_the_lock_without_waiting_for_it() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let (root, index) = index_in("prestage-flushing");
        FLUSHES.store(0, Ordering::SeqCst);
        COMMIT_WAITING.store(false, Ordering::SeqCst);
        GATED_FLUSH_DONE.store(false, Ordering::SeqCst);
        FLUSH_MAY_FINISH.store(false, Ordering::SeqCst);
        start_with(held_flush, &index, 8);
        until_staged(&index);
        assert!(!GATED_FLUSH_DONE.load(Ordering::SeqCst), "the premise: the flush is still held when the commit comes");
        let mut notes = crate::service_index_lock::WaitNotes::default();
        let lock = acquire_noting_wait(&index, 8, &|_| {}, &mut notes).expect("taken");
        assert_eq!(notes.stage_kind, "cold", "the commit staged its own marker instead of waiting for the flush");
        assert_eq!(std::fs::read_to_string(crate::service_index_lock::lock_path_of(&index)).expect("marker"), marker_text(8, &own_identity).expect("text"), "the lock holds this batch's marker");
        // The marker that was still being flushed is not lost: it finishes beside the landing, and the idle time after the
        //  landing removes it.
        assert!(!GATED_FLUSH_DONE.load(Ordering::SeqCst), "the lock was taken while the flush was still held");
        FLUSH_MAY_FINISH.store(true, Ordering::SeqCst);
        let waited: Instant = Instant::now();
        while !GATED_FLUSH_DONE.load(Ordering::SeqCst) {
            assert!(waited.elapsed() < Duration::from_secs(60), "the flush never ended");
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(FLUSHES.load(Ordering::SeqCst), 1, "one flush, the idle time's");
        drop(lock);
        discard();
        assert!(leftovers(&index).is_empty(), "the marker that was flushing is removed once it is finished: {:?}", leftovers(&index));
        remove(root.as_path());
    });
}

/// Starting what is already made or being made for the same batch and index starts nothing:
/// one flush,
/// one file.
#[test]
fn starting_again_for_the_same_batch_makes_no_second_file() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let (root, index) = index_in("prestage-same");
        FLUSHES.store(0, Ordering::SeqCst);
        start_with(slow_flush, &index, 9);
        start_with(slow_flush, &index, 9);
        start_with(slow_flush, &index, 9);
        discard();
        assert_eq!(FLUSHES.load(Ordering::SeqCst), 1);
        assert!(leftovers(&index).is_empty(), "discarded: {:?}", leftovers(&index));
        remove(root.as_path());
    });
}

/// What:
///  One more marker is asked for than the service keeps,
/// and the oldest goes:
/// its file leaves the directory as well as the slots,
/// so the files waiting are the newest ones.
/// Why:
///   `MARKERS_MADE_AHEAD` is the bound on the files the service leaves in the Git directory,
/// and a marker dropped from the slots while its file stays would name a batch no commit takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// prestage.start(index, 17); prestage.start(index, 18); prestage.start(index, 19); /* the file of 17 is gone */
/// ```
#[test]
fn the_oldest_marker_goes_when_more_are_asked_for_than_the_service_keeps() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let (root, index) = index_in("prestage-oldest-goes");
        let kept: u64 = u64::try_from(MARKERS_MADE_AHEAD).expect("a count of files");
        let oldest: u64 = 17;
        for seq in oldest..=oldest + kept {
            start_with(sync_file, &index, seq);
        }
        until_finished(&index, MARKERS_MADE_AHEAD);
        let mut expected: Vec<String> = (oldest + 1..=oldest + kept).map(|seq| { return marker_text(seq, &own_identity).expect("text"); }).collect();
        expected.sort();
        assert_eq!(texts_of(&index), expected, "the oldest marker's file goes with it");
        discard();
        remove(root.as_path());
    });
}

/// What:
///  A commit takes the marker of its own batch,
/// and the marker of the batch after it stays:
/// it is the spare the next landing takes.
/// Why:
///   `take` removes only a marker of a passed batch or of another index,
/// and a marker of a later batch is what keeps the next commit's lock path a rename.
/// A marker still being made is no marker for a landing either (`take`),
/// so the idle time's making is waited for here,
/// which is never a commit's wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// prestage.start(index, 19); untilFinished(1); prestage.start(index, 20); untilFinished(2);
/// const taken = await prestage.take(index, 19); /* the file of 20 stays */
/// ```
#[test]
fn taking_this_batchs_marker_keeps_the_spare_of_the_next_one() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let (root, index) = index_in("prestage-spare-kept");
        let mine: u64 = 19;
        start_with(sync_file, &index, mine);
        until_finished(&index, 1);
        start_with(sync_file, &index, mine + 1);
        until_finished(&index, MARKERS_MADE_AHEAD);
        let taken: Prestaged = take(&index, mine).expect("this batch's marker is taken");
        assert_eq!(taken.marker, marker_text(mine, &own_identity).expect("text"));
        // The taken file is the commit's to place; here it goes, so what waits is the spare alone.
        remove_if_ours(taken);
        assert_eq!(texts_of(&index), vec![marker_text(mine + 1, &own_identity).expect("text")], "the spare of the next batch waits");
        discard();
        remove(root.as_path());
    });
}

/// A marker for another batch is removed when the batch comes,
/// and the commit stages its own:
/// the lock holds the batch's number.
#[test]
fn a_marker_for_another_batch_is_removed_and_the_commit_stages_its_own() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let (root, index) = index_in("prestage-other-batch");
        start_with(sync_file, &index, 10);
        until_made();
        let mut notes = crate::service_index_lock::WaitNotes::default();
        let lock = acquire_noting_wait(&index, 11, &|_| {}, &mut notes).expect("taken");
        assert_eq!(notes.stage_kind, "cold");
        assert_eq!(std::fs::read_to_string(crate::service_index_lock::lock_path_of(&index)).expect("marker"), marker_text(11, &own_identity).expect("text"));
        assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
        drop(lock);
        remove(root.as_path());
    });
}

/// A marker made for one index is not placed as another's lock:
/// it is removed and the commit stages its own.
#[test]
fn a_marker_for_another_index_is_removed() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let (root, index) = index_in("prestage-other-index");
        let other: PathBuf = root.join("other-git-dir");
        std::fs::create_dir(&other).expect("directory");
        let other_index: PathBuf = other.join("index");
        start_with(sync_file, &other_index, 12);
        until_made();
        let mut notes = crate::service_index_lock::WaitNotes::default();
        let lock = acquire_noting_wait(&index, 12, &|_| {}, &mut notes).expect("taken");
        assert_eq!(notes.stage_kind, "cold");
        assert!(std::fs::read_dir(&other).expect("listing").flatten().next().is_none(), "the other directory is empty again");
        drop(lock);
        remove(root.as_path());
    });
}

/// A marker whose file is gone by the time it is taken (the Git directory was replaced) is no marker:
/// the commit stages its own and takes the lock.
#[test]
fn a_marker_whose_file_is_gone_is_staged_again_cold() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let (root, index) = index_in("prestage-vanished");
        start_with(sync_file, &index, 13);
        until_made();
        // The flush is over; the service holds the file open, and the file goes while it is held.
        for name in leftovers(&index) {
            std::fs::remove_file(index.parent().expect("directory").join(name)).expect("the file goes");
        }
        let mut notes = crate::service_index_lock::WaitNotes::default();
        let lock = acquire_noting_wait(&index, 13, &|_| {}, &mut notes).expect("taken cold");
        assert_eq!(notes.stage_kind, "cold");
        drop(lock);
        remove(root.as_path());
    });
}

/// When the service ends (the guard goes out of scope) nothing it made ahead is left in the directory.
#[test]
fn nothing_made_ahead_is_left_when_the_service_ends() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let (root, index) = index_in("prestage-guard");
        {
            let _guard = PrestageGuard;
            start_with(sync_file, &index, 15);
            until_staged(&index);
        }
        assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
        remove(root.as_path());
    });
}

/// The file made ahead names this process,
/// and its text is this process's identity for that batch,
/// so what a crash leaves is told from a live service's by the name (`remove_dead_staging`) and never read as a lock.
#[test]
fn the_marker_made_ahead_names_this_process_and_the_batch() {
    let _serial = ONE_AT_A_TIME.lock().expect("serial");
    let (root, index) = index_in("prestage-identity");
    start_with(sync_file, &index, 16);
    until_staged(&index);
    let name: String = leftovers(&index).remove(0);
    let scope: &str = crate::service_index_marker::current_pid_scope().expect("a PID namespace");
    assert!(name.contains(format!("{}-{}.{scope}-16-", crate::service_index_staging::STAGED_INFIX, std::process::id()).as_str()), "{name}");
    let text: String = std::fs::read_to_string(index.parent().expect("directory").join(&name)).expect("text");
    assert_eq!(text, marker_text(16, &own_identity).expect("text"));
    discard();
    remove(root.as_path());
}

/// Waits until the marker made ahead is finished.
fn until_made() {
    let began: Instant = Instant::now();
    while !is_made() {
        assert!(began.elapsed() < Duration::from_secs(60), "the marker was never finished");
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// The only file in the index's directory besides the index,
/// by its name.
fn the_staged_file(index: &Path) -> PathBuf {
    let mut names: Vec<String> = leftovers(index);
    assert_eq!(names.len(), 1, "{names:?}");
    return index.parent().expect("directory").join(names.remove(0));
}

/// A pathname that came to name another file while the service was idle is that file's:
/// the commit stages its own marker and the other file stays where it is.
#[test]
fn a_pathname_that_names_another_file_is_left_alone_by_the_commit() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let (root, index) = index_in("prestage-replaced");
        start_with(sync_file, &index, 20);
        until_made();
        let staged: PathBuf = the_staged_file(&index);
        std::fs::remove_file(&staged).expect("the marker's file goes");
        std::fs::write(&staged, "someone else's file").expect("another file takes the name");
        let mut notes = crate::service_index_lock::WaitNotes::default();
        let lock = acquire_noting_wait(&index, 20, &|_| {}, &mut notes).expect("taken cold");
        assert_eq!(notes.stage_kind, "cold");
        assert_eq!(std::fs::read_to_string(&staged).expect("the other file stays"), "someone else's file");
        drop(lock);
        remove(root.as_path());
    });
}

/// Discarding the marker made ahead removes the file it made and nothing else:
/// a pathname that names another file by then keeps that file.
#[test]
fn discarding_leaves_a_file_that_took_the_name_alone() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let (root, index) = index_in("prestage-discard-replaced");
        start_with(sync_file, &index, 21);
        until_made();
        let staged: PathBuf = the_staged_file(&index);
        std::fs::remove_file(&staged).expect("the marker's file goes");
        std::fs::write(&staged, "another file again").expect("another file takes the name");
        discard();
        assert_eq!(std::fs::read_to_string(&staged).expect("discard leaves it"), "another file again");
        remove(root.as_path());
    });
}

/// A link to the flushed file in its place is not the file that was flushed:
/// placing it would place the link.
/// The commit stages its own marker,
/// the lock is a regular file,
/// and the link stays.
#[test]
fn a_link_in_the_place_of_the_flushed_file_is_not_placed() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let (root, index) = index_in("prestage-link");
        start_with(sync_file, &index, 23);
        until_made();
        let staged: PathBuf = the_staged_file(&index);
        let moved: PathBuf = root.join("moved-marker");
        std::fs::rename(&staged, &moved).expect("the file moves");
        std::os::unix::fs::symlink(&moved, &staged).expect("a link takes its place");
        let mut notes = crate::service_index_lock::WaitNotes::default();
        let lock = acquire_noting_wait(&index, 23, &|_| {}, &mut notes).expect("taken cold");
        assert_eq!(notes.stage_kind, "cold");
        assert!(std::fs::symlink_metadata(crate::service_index_lock::lock_path_of(&index)).expect("lock").file_type().is_file(), "the lock is a regular file, not a link");
        assert!(std::fs::symlink_metadata(&staged).expect("the link stays").file_type().is_symlink());
        drop(lock);
        remove(root.as_path());
    });
}

/// What:
///  A commit that comes to a flush which then fails takes the lock the ordinary way,
/// without waiting for that flush,
/// and the failed flush removes
///  its own file beside the landing.
/// Why:
///   The lock is taken while the flush is still running,
/// which the marker's file in the directory shows;
/// a landing that
///  waited for the flush would find the file already gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lock = await acquire(index, 24); expect(kind).toBe('cold'); expect(leftovers(index)).not.toEqual([]);
/// ```
#[test]
fn a_commit_that_comes_to_a_failing_flush_stages_its_own_without_joining_it() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let (root, index) = index_in("prestage-failing");
        FLUSHES.store(0, Ordering::SeqCst);
        COMMIT_WAITING.store(false, Ordering::SeqCst);
        FLUSH_MAY_FINISH.store(false, Ordering::SeqCst);
        start_with(held_failing_flush, &index, 24);
        until_staged(&index);
        let mut notes = crate::service_index_lock::WaitNotes::default();
        let lock = acquire_noting_wait(&index, 24, &|_| {}, &mut notes).expect("taken cold beside the flush");
        assert_eq!(notes.stage_kind, "cold");
        assert_eq!(leftovers(&index).len(), 1, "the flush that will fail is still running: {:?}", leftovers(&index));
        // It fails, and its own file goes with it.
        FLUSH_MAY_FINISH.store(true, Ordering::SeqCst);
        let waited: Instant = Instant::now();
        while !leftovers(&index).is_empty() {
            assert!(waited.elapsed() < Duration::from_secs(60), "the failing flush left its file: {:?}", leftovers(&index));
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(FLUSHES.load(Ordering::SeqCst), 1);
        drop(lock);
        discard();
        remove(root.as_path());
    });
}

/// What:
///  A marker for another batch that is still being flushed is left to finish rather than waited for,
/// and it is removed once it is
///  finished.
/// Why:
///   The commit's own path may not wait for the disk,
/// and the marker is not the commit's;
/// the file stays until it is safe to
///  remove,
/// which is after its flush.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// start(gatedSlowFlush, index, 25); const lock = await acquire(index, 26);
/// expect(kind).toBe('cold'); expect(leftovers(index)).toHaveLength(1); discard(); expect(leftovers(index)).toEqual([]);
/// ```
#[test]
fn a_marker_of_another_batch_that_is_still_flushing_is_left_to_finish() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let (root, index) = index_in("prestage-mismatch-wait");
        COMMIT_WAITING.store(false, Ordering::SeqCst);
        GATED_FLUSH_DONE.store(false, Ordering::SeqCst);
        FLUSH_MAY_FINISH.store(false, Ordering::SeqCst);
        start_with(held_flush, &index, 25);
        until_staged(&index);
        let mut notes = crate::service_index_lock::WaitNotes::default();
        let lock = acquire_noting_wait(&index, 26, &|_| {}, &mut notes).expect("taken cold");
        assert_eq!(notes.stage_kind, "cold");
        assert_eq!(leftovers(&index).len(), 1, "the other batch's marker is still being flushed: {:?}", leftovers(&index));
        FLUSH_MAY_FINISH.store(true, Ordering::SeqCst);
        drop(lock);
        // The idle time after the landing removes it, once its flush is over.
        discard();
        assert!(leftovers(&index).is_empty(), "the marker of the other batch is removed once it is finished: {:?}", leftovers(&index));
        remove(root.as_path());
    });
}

/// The percentile `p` of the sorted times,
/// in milliseconds.
fn percentile(times: &mut [f64], p: f64) -> f64 {
    times.sort_by(|a, b| { return a.partial_cmp(b).expect("numbers"); });
    return times[((times.len() as f64 - 1.0) * p) as usize];
}

/// Measurement,
/// not a control:
/// what the lock path of a commit costs without and with the marker made ahead,
/// under the load
/// the host has.
/// The arms alternate in one directory on one disk,
/// with a second cold arm as the same-binary control for the spread.
/// Run it by name with `--run-ignored all --no-capture`;
/// it prints counts,
/// medians,
/// 95th percentiles and maxima.
#[test]
#[ignore = "a measurement for the handover, not a control"]
fn measure_the_lock_path_with_and_without_the_marker_made_ahead() {
    let _serial = ONE_AT_A_TIME.lock().expect("serial");
    let (root, index) = index_in("prestage-measure");
    let rounds: usize = std::env::var("PRESTAGE_ROUNDS").ok().and_then(|text| { return text.parse::<usize>().ok(); }).unwrap_or(150);
    let (mut cold, mut cold_again, mut ahead, mut ahead_wait) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut seq: u64 = 1_000;
    let mut once = |kind: &str, times: &mut Vec<f64>, waits: &mut Vec<f64>| {
        seq += 1;
        if kind == "pre" {
            start(&index, seq);
            // The idle time between two commits.
            std::thread::sleep(Duration::from_millis(60));
        }
        let began: Instant = Instant::now();
        let mut notes = crate::service_index_lock::WaitNotes::default();
        let lock = acquire_noting_wait(&index, seq, &|_| {}, &mut notes).expect("taken");
        times.push(began.elapsed().as_secs_f64() * 1000.0);
        if kind == "pre" {
            assert_eq!(notes.stage_kind, "pre");
            waits.push(notes.staging.as_secs_f64() * 1000.0);
        }
        drop(lock);
    };
    let mut unused: Vec<f64> = Vec::new();
    for _ in 0..rounds {
        once("cold", &mut cold, &mut unused);
        once("pre", &mut ahead, &mut ahead_wait);
        once("cold", &mut cold_again, &mut unused);
    }
    for (name, times) in [("cold", &mut cold), ("cold again (spread)", &mut cold_again), ("made ahead", &mut ahead), ("made ahead, waited for flush", &mut ahead_wait)] {
        let over_20: usize = times.iter().filter(|time| { return **time > 20.0; }).count();
        let over_100: usize = times.iter().filter(|time| { return **time > 100.0; }).count();
        eprintln!(
            "MEASURE {name:30} n={} p50={:.3} ms p95={:.3} ms max={:.3} ms over20={over_20} over100={over_100}",
            times.len(),
            percentile(times, 0.5),
            percentile(times, 0.95),
            percentile(times, 1.0)
        );
    }
    remove(root.as_path());
}

/// A file a killed service left under the name of a process number that another process has now is told from a running
/// writer's by its text:
/// a whole marker that names a holder that is gone is removed,
/// while an empty file,
/// a half written one and a running writer's own marker stay.
/// This is what keeps what a crash leaves from growing when process numbers come round.
#[test]
fn a_file_left_under_a_reused_process_number_is_removed_by_its_marker() {
    let _serial = ONE_AT_A_TIME.lock().expect("serial");
    let (root, index) = index_in("prestage-reused-number");
    let lock_path: PathBuf = crate::service_index_lock::lock_path_of(&index);
    let alive: u32 = std::process::id();
    let named = |tag: &str| -> PathBuf { return PathBuf::from(format!("{}{}-{alive}-{tag}", lock_path.display(), crate::service_index_staging::STAGED_INFIX)); };
    let reused: PathBuf = named("1-reused");
    std::fs::write(&reused, format!("{}{}1 {alive} linux:1#{}#1{}\n", crate::service_index_lock::LOCK_MARKER, crate::service_index_marker::MARKER_VERSION, "0".repeat(32), Some(crate::claim_time::current_machine_id()).filter(|id| { return !id.is_empty(); }).map_or_else(String::new, |id| { return format!("#{id}"); }))).expect("a killed service's file");
    let empty: PathBuf = named("2-empty");
    std::fs::write(&empty, "").expect("a writer between creating and writing");
    let half: PathBuf = named("3-half");
    std::fs::write(&half, "cli-git-ser").expect("a writer part way");
    let running: PathBuf = named("4-running");
    std::fs::write(&running, marker_text(4, &own_identity).expect("text")).expect("a running writer's marker");
    crate::service_index_staging::remove_dead_staging(&lock_path);
    assert!(!reused.exists(), "the file of a gone holder under a reused number goes");
    assert!(empty.exists() && half.exists() && running.exists(), "{:?}", leftovers(&index));
    remove(root.as_path());
}

/// The names of the threads this process has,
/// read from `/proc` (the kernel keeps the first fifteen characters of a thread's name).
fn thread_names_of_this_process() -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for task in std::fs::read_dir("/proc/self/task").expect("the tasks of this process").flatten() {
        if let Ok(name) = std::fs::read_to_string(task.path().join("comm")) {
            names.push(name.trim_end().to_string());
        }
    }
    return names;
}

/// The marker made ahead is made by a job of the runtime's blocking pool,
/// not by a thread the module started for it:
/// while its flush is running no thread of the process is named for it.
#[test]
fn the_marker_made_ahead_is_made_without_a_thread_of_its_own() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let (root, index) = index_in("prestage-no-own-thread");
        FLUSHES.store(0, Ordering::SeqCst);
        start_with(slow_flush, &index, 11);
        until_staged(&index);
        let own: Vec<String> = thread_names_of_this_process().into_iter().filter(|name| return "cli-git-prestage".starts_with(name.as_str()) && name.len() >= 14).collect();
        assert!(own.is_empty(), "the marker is being made by a thread of its own: {own:?}");
        discard();
        remove(root.as_path());
    });
}

/// What:
///  A start that asks for one more marker than the service keeps lets go of the oldest,
/// which is still flushing,
/// and returns for the commit without waiting for that flush.
/// Why:
///   This runs on the commit's path,
/// after the lock is taken (`service_land_take_index_lock.rs`),
/// so a wait here holds an index lock while it waits,
/// and the pool jobs that hold it up are other commands' work:
/// the owner's rules are that nothing waits indefinitely and that a commit never waits on another agent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// start(held(index, 30)); start(held(index, 31)); start(syncFile, index, 32); expect(heldFlushesEnded).toBe(0);
/// ```
#[test]
fn a_start_that_evicts_a_marker_still_flushing_does_not_wait_for_it() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let (root, index) = index_in("prestage-evict-flushing");
        FLUSHES.store(0, Ordering::SeqCst);
        HELD_FLUSHES_ENDED.store(0, Ordering::SeqCst);
        FLUSH_MAY_FINISH.store(false, Ordering::SeqCst);
        let oldest: u64 = 30;
        start_with(flush_held_by_the_control, &index, oldest);
        until_flushes_are_held(1);
        start_with(flush_held_by_the_control, &index, oldest + 1);
        until_flushes_are_held(2);
        // One more than the service keeps: the oldest goes, and its flush is still held.
        start_with(sync_file, &index, oldest + 2);
        assert_eq!(HELD_FLUSHES_ENDED.load(Ordering::SeqCst), 0, "the start waited for the oldest marker's flush to end");
        let_the_held_flushes_go(2);
        discard();
        assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
        remove(root.as_path());
    });
}

/// What:
///  A start for this index lets go of a marker made for another index,
/// and returns without waiting for its flush.
/// Why:
///   Two worktrees of one repository share the service,
/// and a landing on one index reaches this step (`start`) while the other index's marker is still being made;
/// the lock of this batch is taken already,
/// so a wait there blocks a command that holds a lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// start(held(otherIndex, 33)); start(syncFile, index, 33); expect(heldFlushesEnded).toBe(0);
/// ```
#[test]
fn a_start_for_this_index_does_not_wait_for_another_indexs_flush() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let (root, index) = index_in("prestage-other-index-making");
        let other: PathBuf = root.join("other-git-dir");
        std::fs::create_dir(&other).expect("directory");
        let other_index: PathBuf = other.join("index");
        FLUSHES.store(0, Ordering::SeqCst);
        HELD_FLUSHES_ENDED.store(0, Ordering::SeqCst);
        FLUSH_MAY_FINISH.store(false, Ordering::SeqCst);
        start_with(flush_held_by_the_control, &other_index, 33);
        until_flushes_are_held(1);
        start_with(sync_file, &index, 33);
        assert_eq!(HELD_FLUSHES_ENDED.load(Ordering::SeqCst), 0, "the start waited for another index's flush to end");
        assert!(std::fs::read_dir(&other).expect("listing").flatten().next().is_none(), "the other index's marker goes without waiting for its flush");
        let_the_held_flushes_go(1);
        discard();
        assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
        remove(root.as_path());
    });
}

/// Controls that occupy the pool hold its one thread only this long,
/// for the same reason as `THE_CEILING_OF_A_HELD_FLUSH`.
const THE_CEILING_OF_AN_OCCUPIED_POOL: Duration = Duration::from_secs(2);

/// Whether the occupying job of a control has the pool's one thread.
static THE_POOL_IS_OCCUPIED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether the occupying job of a control may end.
static THE_OCCUPATION_IS_OVER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// What:
///  A start that lets go of a marker whose job has not got a thread yet returns at once,
/// and that marker makes no file at all.
/// Why:
///   A marker queued behind other work of the pool is the case the owner's rule "nothing should ever wait forever"
/// is about:
/// the job's own wait is not this commit's,
/// and a file that no batch takes must not appear later either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// capWorkers(1); occupy(); start(index, 36); start(index, 37); start(syncFile, index, 38); expect(flushes).toBe(0);
/// ```
#[test]
fn evicting_a_marker_that_never_got_a_thread_makes_no_file_and_waits_for_nothing() {
    crate::lazy_runtime::cap_workers(1);
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let (root, index) = index_in("prestage-evict-queued");
        FLUSHES.store(0, Ordering::SeqCst);
        THE_POOL_IS_OCCUPIED.store(false, Ordering::SeqCst);
        THE_OCCUPATION_IS_OVER.store(false, Ordering::SeqCst);
        let occupied = crate::lazy_runtime::spawn_on_runtime(crate::blocking_job::run_blocking_job(|| {
            THE_POOL_IS_OCCUPIED.store(true, Ordering::SeqCst);
            let held: Instant = Instant::now();
            while !THE_OCCUPATION_IS_OVER.load(Ordering::SeqCst) && held.elapsed() < THE_CEILING_OF_AN_OCCUPIED_POOL {
                std::thread::sleep(Duration::from_millis(1));
            }
        }))
        .expect("the runtime runs the occupying job");
        let began: Instant = Instant::now();
        while !THE_POOL_IS_OCCUPIED.load(Ordering::SeqCst) {
            assert!(began.elapsed() < Duration::from_secs(10), "the occupying job never started");
            std::thread::sleep(Duration::from_millis(1));
        }
        let oldest: u64 = 36;
        start_with(slow_flush, &index, oldest);
        start_with(slow_flush, &index, oldest + 1);
        // The pool has one thread and it is occupied, so neither marker has been made: the oldest is let go before its
        //  making began.
        start_with(sync_file, &index, oldest + 2);
        assert_eq!(FLUSHES.load(Ordering::SeqCst), 0, "the start waited for a marker queued behind the occupying job");
        THE_OCCUPATION_IS_OVER.store(true, Ordering::SeqCst);
        let _ended = crate::lazy_runtime::run_on_runtime(occupied);
        discard();
        assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
        remove(root.as_path());
    });
}

/// What:
///  A marker the service has stopped keeping is not made at all,
/// even though its making was started after the last sweep,
/// so a stop leaves no file behind.
/// Why:
///   The service's last sweep ran before this marker's job reached its file,
/// and a file created after the sweep is a file nothing removes;
/// `SPEC.md` "Removing the lock and what is made durable" says the markers "are removed when the service ends,
///  whichever way its loop ends".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeMadeAhead(); start(index, 34); untilFinished(1); expect(leftovers(index)).toEqual([]); expect(flushes).toBe(0);
/// ```
#[test]
fn a_marker_the_service_has_stopped_keeping_is_not_made() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let (root, index) = index_in("prestage-service-ends");
        FLUSHES.store(0, Ordering::SeqCst);
        // The service's last sweep, before this marker was even started.
        remove_the_file_made_ahead();
        start_with(slow_flush, &index, 34);
        until_finished(&index, 1);
        assert_eq!(FLUSHES.load(Ordering::SeqCst), 0, "the marker was made after the service stopped keeping them");
        assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
        discard();
        remove(root.as_path());
    });
}

/// A marker the service recorded before its last sweep is removed by that sweep,
/// and the sweep waits for no flush.
#[test]
fn the_last_sweep_removes_a_recorded_marker() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let (root, index) = index_in("prestage-last-sweep");
        start_with(sync_file, &index, 35);
        until_made();
        remove_the_file_made_ahead();
        assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
        discard();
        remove(root.as_path());
    });
}
