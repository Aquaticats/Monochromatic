//! Controls for taking the index lock of a landing:
//! the lock names the batch that took it,
//! and the markers of the two batches after it begin flushing while this landing holds the lock,
//! so the next commit finds its own on the disk and the one after it finds the spare,
//! instead of waiting for a flush the previous landing started after it ended,
//! and a marker another index's landing started is let go without waiting for its flush.

use super::*;
use crate::service_index_lock::{lock_path_of, marker_text};
use crate::service_index_marker::own_identity;
use crate::service_index_prestage;
use crate::test_support::{fixture, remove};
use std::sync::Mutex;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The controls run one at a time:
/// the marker made ahead is kept in one slot per landing thread,
/// and the slot is the process's.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

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

/// The request a landing takes the lock for:
/// nothing in it is read once the lock is taken,
/// so only the index and the scratch tree name real places.
fn request_of(index: &Path, scratch: &Path, git_dir: &Path) -> CommitRequest {
    return CommitRequest {
        id: String::from("control"),
        repository: git_dir.parent().expect("repository").to_path_buf(),
        common_dir: git_dir.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        reference: String::from("refs/heads/control"),
        base: String::from("0".repeat(40)),
        changes: Vec::new(),
        message: String::from("control"),
        author: None,
        committer: String::from("Control <control@example.invalid> 0 +0000"),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: index.to_path_buf(),
        scratch: scratch.to_path_buf(),
        captured: 0,
        holder: String::new(),
    };
}

/// A context whose Git is never started here:
/// taking the lock runs no child.
fn context_of(git_dir: &Path) -> GitContext {
    return GitContext { real_git: PathBuf::from("/usr/bin/git"), overlay: Vec::new(), global_prefix: Vec::new(), common_dir: Some(git_dir.to_path_buf()) };
}

/// The markers made ahead are written by jobs of the blocking pool,
/// so the files appear a moment after the landing asks for them:
/// waits for all the service keeps while the landing still holds the lock.
fn until_the_next_batchs_markers_are_being_made(index: &Path) -> Vec<String> {
    let began: Instant = Instant::now();
    loop {
        let staged: Vec<String> = leftovers(index);
        if staged.len() >= service_index_prestage::MARKERS_MADE_AHEAD {
            return staged;
        }
        assert!(began.elapsed() < Duration::from_secs(60), "the markers of the two batches after this one were not started while the landing held the lock: {staged:?}");
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// The markers of the two batches after the one that took the lock begin flushing while this landing holds the lock:
/// their files are there before the landing ends,
/// and their texts are those batches'.
#[test]
fn the_batch_after_this_one_begins_flushing_when_the_landing_takes_the_lock() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let root: PathBuf = fixture("take-index-lock-starts-next-batchs-marker");
        let git_dir: PathBuf = root.join("git-dir");
        std::fs::create_dir(&git_dir).expect("directory");
        let index: PathBuf = git_dir.join("index");
        std::fs::write(&index, "old index").expect("index");
        let scratch: PathBuf = root.join("scratch");
        std::fs::create_dir(&scratch).expect("scratch");
        let journal: PathBuf = root.join("journal");
        std::fs::create_dir(&journal).expect("journal");
        let first: CommitRequest = request_of(index.as_path(), scratch.as_path(), git_dir.as_path());
        let mut timing: LandTiming = LandTiming::default();
        let held: IndexLock = take_index_lock(&context_of(git_dir.as_path()), journal.as_path(), Duration::from_secs(30), &first, 7, &|_| {}, &mut timing, "new", "scratch-ref").expect("the lock is taken");
        // The lock is the batch's own marker, and the next batch's marker is being made beside it while this one is held.
        assert_eq!(std::fs::read_to_string(lock_path_of(&index)).expect("the lock's text"), marker_text(7, &own_identity).expect("text"));
        let staged: Vec<String> = until_the_next_batchs_markers_are_being_made(index.as_path());
        assert_eq!(staged.len(), service_index_prestage::MARKERS_MADE_AHEAD, "the next batch's marker and the spare after it are being made: {staged:?}");
        let mut texts: Vec<String> = staged.iter().map(|name| { return std::fs::read_to_string(git_dir.join(name)).expect("the staged marker's text"); }).collect();
        texts.sort();
        let mut expected: Vec<String> = vec![marker_text(8, &own_identity).expect("text"), marker_text(9, &own_identity).expect("text")];
        expected.sort();
        assert_eq!(texts, expected);
        drop(held);
        service_index_prestage::discard();
        remove(root.as_path());
    });
}

/// How many of the control's held flushes have entered.
static HELD_SYNCS_ENTERED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// How many of the control's held flushes have ended.
static HELD_SYNCS_ENDED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Whether a held flush may end.
static FLUSH_MAY_END: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The longest a flush is held:
/// a product that waits for it ends it at this ceiling,
/// and the control then reads the wait from `HELD_SYNCS_ENDED` instead of hanging the run.
const THE_CEILING_OF_A_HELD_SYNC: Duration = Duration::from_secs(2);

/// A flush held until the control lets it go,
/// which says so when it has ended.
fn sync_held_by_the_control(file: &std::fs::File) -> std::io::Result<()> {
    HELD_SYNCS_ENTERED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let held: Instant = Instant::now();
    while !FLUSH_MAY_END.load(std::sync::atomic::Ordering::SeqCst) && held.elapsed() < THE_CEILING_OF_A_HELD_SYNC {
        std::thread::sleep(Duration::from_millis(1));
    }
    HELD_SYNCS_ENDED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    return file.sync_all();
}

/// Waits until `want` held flushes have entered.
fn until_the_held_flushes_are_running(want: usize) {
    let began: Instant = Instant::now();
    while HELD_SYNCS_ENTERED.load(std::sync::atomic::Ordering::SeqCst) < want {
        assert!(began.elapsed() < Duration::from_secs(10), "{} of {want} held flushes entered", HELD_SYNCS_ENTERED.load(std::sync::atomic::Ordering::SeqCst));
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// What:
///  Taking this batch's index lock returns without waiting for another index's marker made ahead,
/// though both of that index's markers are still flushing.
/// Why:
///   The markers of the two batches after this one are started under the lock this landing just took
/// (`take_index_lock`),
/// and a marker another index's landing started is let go there;
/// a wait there holds this index's lock while another batch's flush finishes,
/// which is the owner's "nothing should ever wait forever or indefinitely",
/// and the agent rule that no commit waits on another's work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// start(held(otherIndex, 8)); start(held(otherIndex, 9)); const held = await takeIndexLock(index, 20); expect(heldSyncsEnded).toBe(0);
/// ```
#[test]
fn taking_this_batchs_lock_does_not_wait_for_another_indexs_flush() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let _serial = ONE_AT_A_TIME.lock().expect("serial");
        let root: PathBuf = fixture("take-index-lock-other-index-flushing");
        let other_git: PathBuf = root.join("other-git-dir");
        std::fs::create_dir(&other_git).expect("directory");
        let other_index: PathBuf = other_git.join("index");
        std::fs::write(&other_index, "old index").expect("other index");
        let git_dir: PathBuf = root.join("git-dir");
        std::fs::create_dir(&git_dir).expect("directory");
        let index: PathBuf = git_dir.join("index");
        std::fs::write(&index, "old index").expect("index");
        let scratch: PathBuf = root.join("scratch");
        std::fs::create_dir(&scratch).expect("scratch");
        let journal: PathBuf = root.join("journal");
        std::fs::create_dir(&journal).expect("journal");
        HELD_SYNCS_ENTERED.store(0, std::sync::atomic::Ordering::SeqCst);
        HELD_SYNCS_ENDED.store(0, std::sync::atomic::Ordering::SeqCst);
        FLUSH_MAY_END.store(false, std::sync::atomic::Ordering::SeqCst);
        service_index_prestage::start_with(sync_held_by_the_control, other_index.as_path(), 8);
        service_index_prestage::start_with(sync_held_by_the_control, other_index.as_path(), 9);
        until_the_held_flushes_are_running(2);
        let first: CommitRequest = request_of(index.as_path(), scratch.as_path(), git_dir.as_path());
        let mut timing: LandTiming = LandTiming::default();
        let held: IndexLock = take_index_lock(&context_of(git_dir.as_path()), journal.as_path(), Duration::from_secs(30), &first, 20, &|_| {}, &mut timing, "new", "scratch-ref").expect("this index's lock is free and is taken");
        assert_eq!(HELD_SYNCS_ENDED.load(std::sync::atomic::Ordering::SeqCst), 0, "taking this batch's lock waited for another index's marker to finish flushing");
        let staged_elsewhere: Vec<String> = std::fs::read_dir(&other_git)
            .expect("listing")
            .flatten()
            .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
            .filter(|name| { return name.contains(crate::service_index_staging::STAGED_INFIX); })
            .collect();
        assert!(staged_elsewhere.is_empty(), "the other index's markers go without waiting for their flushes: {staged_elsewhere:?}");
        drop(held);
        FLUSH_MAY_END.store(true, std::sync::atomic::Ordering::SeqCst);
        let ended: Instant = Instant::now();
        while HELD_SYNCS_ENDED.load(std::sync::atomic::Ordering::SeqCst) < 2 {
            assert!(ended.elapsed() < Duration::from_secs(10), "{} of 2 held flushes ended", HELD_SYNCS_ENDED.load(std::sync::atomic::Ordering::SeqCst));
            std::thread::sleep(Duration::from_millis(1));
        }
        service_index_prestage::discard();
        remove(root.as_path());
    });
}
