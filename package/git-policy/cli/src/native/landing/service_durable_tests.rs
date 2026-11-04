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
