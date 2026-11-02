//! Controls for the marker made ahead of the next batch:
//! it is taken and placed without a flush on the commit's path,
//! a commit that comes while it is still being flushed waits for it and starts no second,
//! a marker for another batch or index is removed and replaced by a cold one,
//! one file at most exists,
//! and nothing is left when the service ends.

use super::*;
use crate::service_index_lock::{acquire_noting_wait, marker_text};
use crate::service_index_marker::own_identity;
use crate::service_index_placement::sync_file;
use crate::test_support::{fixture, remove};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The controls that count flushes run one at a time: the counter is the process's.
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

/// The directory of an index, with the index.
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

/// Waits until the marker made ahead is on its way to being ready: its file exists.
fn until_staged(index: &Path) {
    let began: Instant = Instant::now();
    while leftovers(index).is_empty() {
        assert!(began.elapsed() < Duration::from_secs(10), "no marker was made ahead");
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// A marker made ahead and finished while the service was idle is placed without a flush on the commit's path:
/// the lock is taken in a small part of the time one flush takes,
/// the notes say the file came from the idle time,
/// and its text is the batch's.
#[test]
fn a_marker_made_ahead_is_placed_without_a_flush_at_the_commit() {
    let _serial = ONE_AT_A_TIME.lock().expect("serial");
    let (root, index) = index_in("prestage-ready");
    start_with(slow_flush, &index, 7);
    std::thread::sleep(SLOW + Duration::from_millis(300));
    let began: Instant = Instant::now();
    let mut notes = crate::service_index_lock::WaitNotes::default();
    let lock = acquire_noting_wait(&index, 7, Duration::from_secs(5), &|_| {}, &mut notes).expect("taken");
    let took: Duration = began.elapsed();
    assert!(took < SLOW / 2, "the commit paid for a flush: {took:?}");
    assert_eq!(notes.stage_kind, "pre");
    assert!(notes.staging < SLOW / 2, "{notes:?}");
    assert_eq!(std::fs::read_to_string(crate::service_index_lock::lock_path_of(&index)).expect("marker"), marker_text(7, &own_identity).expect("text"));
    assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
    drop(lock);
    remove(root.as_path());
}

/// A commit that comes while the marker is still being flushed waits for that flush and starts no second one:
/// the staging time it reports is most of the flush,
/// and the flush ran once.
#[test]
fn a_commit_that_comes_during_the_flush_waits_for_it_and_starts_no_second() {
    let _serial = ONE_AT_A_TIME.lock().expect("serial");
    let (root, index) = index_in("prestage-flushing");
    FLUSHES.store(0, Ordering::SeqCst);
    start_with(slow_flush, &index, 8);
    until_staged(&index);
    let mut notes = crate::service_index_lock::WaitNotes::default();
    let lock = acquire_noting_wait(&index, 8, Duration::from_secs(5), &|_| {}, &mut notes).expect("taken");
    assert_eq!(notes.stage_kind, "pre");
    assert!(notes.staging >= SLOW / 2, "the commit reports what it waited for: {notes:?}");
    assert_eq!(FLUSHES.load(Ordering::SeqCst), 1, "one flush, not two");
    assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
    drop(lock);
    remove(root.as_path());
}

/// Starting what is already made or being made for the same batch and index starts nothing:
/// one flush,
/// one file.
#[test]
fn starting_again_for_the_same_batch_makes_no_second_file() {
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
}

/// A marker for another batch is removed when the batch comes,
/// and the commit stages its own: the lock holds the batch's number.
#[test]
fn a_marker_for_another_batch_is_removed_and_the_commit_stages_its_own() {
    let _serial = ONE_AT_A_TIME.lock().expect("serial");
    let (root, index) = index_in("prestage-other-batch");
    start_with(sync_file, &index, 10);
    let mut notes = crate::service_index_lock::WaitNotes::default();
    let lock = acquire_noting_wait(&index, 11, Duration::from_secs(5), &|_| {}, &mut notes).expect("taken");
    assert_eq!(notes.stage_kind, "cold");
    assert_eq!(std::fs::read_to_string(crate::service_index_lock::lock_path_of(&index)).expect("marker"), marker_text(11, &own_identity).expect("text"));
    assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
    drop(lock);
    remove(root.as_path());
}

/// A marker made for one index is not placed as another's lock: it is removed and the commit stages its own.
#[test]
fn a_marker_for_another_index_is_removed() {
    let _serial = ONE_AT_A_TIME.lock().expect("serial");
    let (root, index) = index_in("prestage-other-index");
    let other: PathBuf = root.join("other-git-dir");
    std::fs::create_dir(&other).expect("directory");
    let other_index: PathBuf = other.join("index");
    start_with(sync_file, &other_index, 12);
    let mut notes = crate::service_index_lock::WaitNotes::default();
    let lock = acquire_noting_wait(&index, 12, Duration::from_secs(5), &|_| {}, &mut notes).expect("taken");
    assert_eq!(notes.stage_kind, "cold");
    assert!(std::fs::read_dir(&other).expect("listing").flatten().next().is_none(), "the other directory is empty again");
    drop(lock);
    remove(root.as_path());
}

/// A marker whose file is gone by the time it is taken (the Git directory was replaced) is no marker:
/// the commit stages its own and takes the lock.
#[test]
fn a_marker_whose_file_is_gone_is_staged_again_cold() {
    let _serial = ONE_AT_A_TIME.lock().expect("serial");
    let (root, index) = index_in("prestage-vanished");
    start_with(sync_file, &index, 13);
    until_staged(&index);
    // The flush is quick; the thread has the file open, and the file goes while it is held.
    std::thread::sleep(Duration::from_millis(100));
    for name in leftovers(&index) {
        std::fs::remove_file(index.parent().expect("directory").join(name)).expect("the file goes");
    }
    let mut notes = crate::service_index_lock::WaitNotes::default();
    let lock = acquire_noting_wait(&index, 13, Duration::from_secs(5), &|_| {}, &mut notes).expect("taken cold");
    assert_eq!(notes.stage_kind, "cold");
    drop(lock);
    remove(root.as_path());
}

/// When the service ends (the guard goes out of scope) nothing it made ahead is left in the directory.
#[test]
fn nothing_made_ahead_is_left_when_the_service_ends() {
    let _serial = ONE_AT_A_TIME.lock().expect("serial");
    let (root, index) = index_in("prestage-guard");
    {
        let _guard = PrestageGuard;
        start_with(sync_file, &index, 15);
        until_staged(&index);
    }
    assert!(leftovers(&index).is_empty(), "{:?}", leftovers(&index));
    remove(root.as_path());
}

/// The file made ahead names this process, and its text is this process's identity for that batch,
/// so what a crash leaves is told from a live service's by the name (`remove_dead_staging`) and never read as a lock.
#[test]
fn the_marker_made_ahead_names_this_process_and_the_batch() {
    let _serial = ONE_AT_A_TIME.lock().expect("serial");
    let (root, index) = index_in("prestage-identity");
    start_with(sync_file, &index, 16);
    until_staged(&index);
    let name: String = leftovers(&index).remove(0);
    assert!(name.contains(format!("{}-{}-16-", crate::service_index_staging::STAGED_INFIX, std::process::id()).as_str()), "{name}");
    let text: String = std::fs::read_to_string(index.parent().expect("directory").join(&name)).expect("text");
    assert_eq!(text, marker_text(16, &own_identity).expect("text"));
    discard();
    remove(root.as_path());
}

/// The percentile `p` of the sorted times, in milliseconds.
fn percentile(times: &mut [f64], p: f64) -> f64 {
    times.sort_by(|a, b| { return a.partial_cmp(b).expect("numbers"); });
    return times[((times.len() as f64 - 1.0) * p) as usize];
}

/// Measurement, not a control: what the lock path of a commit costs without and with the marker made ahead, under the load
/// the host has.
/// The arms alternate in one directory on one disk, with a second cold arm as the same-binary control for the spread.
/// Run it by name with `--run-ignored all --no-capture`; it prints counts, medians, 95th percentiles and maxima.
#[test]
#[ignore = "a measurement for the handover, not a control"]
fn measure_the_lock_path_with_and_without_the_marker_made_ahead() {
    let _serial = ONE_AT_A_TIME.lock().expect("serial");
    let (root, index) = index_in("prestage-measure");
    let rounds: usize = std::env::var("PRESTAGE_ROUNDS").ok().and_then(|text| { return text.parse::<usize>().ok(); }).unwrap_or(150);
    let (mut cold, mut cold_again, mut ahead, mut ahead_wait) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut seq: u64 = 1_000;
    let mut once = |kind: &str, cold: &mut Vec<f64>, ahead_wait: &mut Vec<f64>| {
        seq += 1;
        if kind == "pre" {
            start(&index, seq);
            // The idle time between two commits.
            std::thread::sleep(Duration::from_millis(60));
        }
        let began: Instant = Instant::now();
        let mut notes = crate::service_index_lock::WaitNotes::default();
        let lock = acquire_noting_wait(&index, seq, Duration::from_secs(30), &|_| {}, &mut notes).expect("taken");
        cold.push(began.elapsed().as_secs_f64() * 1000.0);
        if kind == "pre" {
            assert_eq!(notes.stage_kind, "pre");
            ahead_wait.push(notes.staging.as_secs_f64() * 1000.0);
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
