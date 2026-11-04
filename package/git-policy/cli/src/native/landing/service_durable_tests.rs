//! Controls for the queue of the landing's syncs: what a publishing landing queues, what a landing that publishes nothing makes
//! at once, and the flush of the index that follows `core.fsync`.

use super::*;
use crate::test_support::{fixture, remove};

/// The names of the syncs a closure's landing queued or waited for, from the events of this thread.
fn events_of(run: impl FnOnce()) -> Vec<String> {
    crate::service_fault::hooks::take_events();
    run();
    return crate::service_fault::hooks::take_events();
}

/// A record file to flush.
fn record(name: &str) -> (std::path::PathBuf, File) {
    let root = fixture(name);
    let path = root.join("record");
    std::fs::write(&path, b"record").expect("record");
    let file = File::open(&path).expect("open");
    return (root, file);
}

/// A landing that publishes queues its flush and the background thread makes it; nothing is made on the caller's thread.
#[test]
fn a_published_landing_queues_and_the_background_thread_flushes() {
    let (root, file) = record("durable-published");
    let before = BACKGROUND_SYNCED.load(std::sync::atomic::Ordering::SeqCst);
    let events = events_of(|| {
        let deferral = begin(false);
        flush_file(&file, "x").expect("queued");
        deferral.settle(true);
    });
    assert_eq!(events, vec![String::from("sync-queued:x")]);
    drain();
    assert!(BACKGROUND_SYNCED.load(std::sync::atomic::Ordering::SeqCst) > before, "the background thread flushed it");
    remove(root.as_path());
}

/// A landing that publishes nothing flushes what it queued before `settle` returns.
#[test]
fn an_unpublished_landing_forces_what_it_queued() {
    let (root, file) = record("durable-unpublished");
    let events = events_of(|| {
        let deferral = begin(false);
        flush_file(&file, "x").expect("queued");
        deferral.settle(false);
    });
    assert_eq!(events, vec![String::from("sync-queued:x"), String::from("sync-waited:x")]);
    remove(root.as_path());
}

/// A deferral dropped without settling (a panic in the landing) forces its queue too, and ends the deferral.
#[test]
fn a_dropped_deferral_forces_and_ends() {
    let (root, file) = record("durable-dropped");
    let events = events_of(|| {
        let deferral = begin(false);
        flush_file(&file, "x").expect("queued");
        drop(deferral);
        flush_file(&file, "y").expect("made now");
    });
    assert_eq!(events, vec![String::from("sync-queued:x"), String::from("sync-waited:x"), String::from("sync-waited:y")]);
    remove(root.as_path());
}

/// Outside a landing a flush is made where it is asked for, as recovery needs it.
#[test]
fn a_flush_outside_a_landing_is_made_now() {
    let (root, file) = record("durable-outside");
    let events = events_of(|| { flush_file(&file, "z").expect("made now"); });
    assert_eq!(events, vec![String::from("sync-waited:z")]);
    remove(root.as_path());
}

/// The index is flushed only when the landing was told it is hardened.
#[test]
fn the_index_is_flushed_only_when_it_is_hardened() {
    let (root, file) = record("durable-index");
    let events = events_of(|| {
        let deferral = begin(false);
        flush_index(&file, "index").expect("skipped");
        deferral.settle(true);
        let hardened = begin(true);
        flush_index(&file, "index").expect("made now");
        hardened.settle(true);
    });
    assert_eq!(events, vec![String::from("sync-waited:index")]);
    remove(root.as_path());
}

/// What:
///  A control that reads the landing's answer to "is the index flushed before it is installed?" after the landing began
///  without `core.fsync` naming the index.
///  `begin(false)` starts a landing and returns a `Deferral` (a handle that must be settled); `index_is_hardened()` reads
///  the answer.
/// Why:
///  The answer must follow what the landing was told, so a landing begun with `false` reports `false`; a function that always
///  said `true` would have the index flushed where native Git does not flush it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const landing = begin(false);
/// assert(indexIsHardened() === false);
/// landing.settle(true);
/// ```
#[test]
fn a_landing_begun_without_the_index_named_reports_it_unhardened() {
    // `let deferral = begin(false);` starts the landing on this thread; `false` says `core.fsync` did not name the index.
    let deferral = begin(false);
    // The answer while the landing runs is the one it was told.
    assert!(!index_is_hardened(), "core.fsync did not name the index, so it is not flushed");
    // `settle(true)` ends the landing as one that published a commit; nothing was queued, so nothing is handed over.
    deferral.settle(true);
}

/// What:
///  A control that reads the same answer after the landing began with `core.fsync` naming the index, again after it
///  settled, and again after a landing that was dropped.
/// Why:
///  A landing told `true` reports `true` (a function that always said `false` would leave the hardened index unflushed), and
///  the thread's answer returns to its default of `true` when a landing ends, however it ends, so the next landing starts
///  from the default and not from the last landing's answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const landing = begin(true);
/// assert(indexIsHardened() === true);
/// landing.settle(true);
/// assert(indexIsHardened() === true);
/// const unhardened = begin(false);
/// unhardened.settle(false);
/// assert(indexIsHardened() === true); // the default is back
/// ```
#[test]
fn a_landing_begun_with_the_index_named_reports_it_hardened_and_the_default_returns_when_it_ends() {
    // `begin(true)` says `core.fsync` named the index.
    let hardened = begin(true);
    assert!(index_is_hardened(), "core.fsync named the index, so it is flushed");
    hardened.settle(true);
    assert!(index_is_hardened(), "no landing runs, so the default (flushed) holds");
    // A landing begun with `false` changes the answer; ending it, here by dropping it unsettled, restores the default.
    let unhardened = begin(false);
    assert!(!index_is_hardened(), "the landing was told the index is not named");
    // `drop(x)` ends `x` early, as a panic in the landing would; the deferral's `Drop` forces its queue and ends it.
    drop(unhardened);
    assert!(index_is_hardened(), "a dropped landing leaves the default behind");
}
