//! Controls of the beat one service keeps and of the reading that tells a working service from a stopped one.

use super::*;

/// A folder of its own for one control.
fn folder(name: &str) -> PathBuf {
    let root: PathBuf = crate::test_support::fixture_root().join(format!("beat-{name}"));
    let _ = std::fs::remove_dir_all(root.as_path());
    std::fs::create_dir_all(root.as_path()).expect("the folder of the beat");
    return root;
}

/// What:
///  A beat that was just moved is a service that is working, and one older than the patience is not.
/// Why:
///  The review of `e347be3115` (part one, finding 2): a service that takes a connection and greets nothing was waited two
///  minutes for. A fresh beat is the fact that says it is working (its runtime being built, its threads running); a stale one
///  says it stopped inside itself.
#[test]
fn a_fresh_beat_is_a_working_service_and_a_stale_one_is_not() {
    let root = folder("fresh-stale");
    let beat: PathBuf = path(root.as_path(), "3f9a");
    let taken: Beat = Beat::take(beat.clone()).expect("the beat is made");
    assert!(is_working(root.as_path(), "3f9a"), "a beat made now is a service that is working");
    // A beat that stopped moving: the file is set back past the patience, as a frozen process leaves it.
    let file: std::fs::File = std::fs::OpenOptions::new().write(true).open(beat.as_path()).expect("the beat");
    file.set_modified(std::time::SystemTime::now() - BEAT_PATIENCE - std::time::Duration::from_secs(1)).expect("an older beat");
    assert!(!is_working(root.as_path(), "3f9a"), "a beat older than the patience is not");
    taken.remove();
    assert!(!beat.exists(), "the file this service made goes with it");
    let _ = std::fs::remove_dir_all(root.as_path());
}

/// What:
///  A service of an earlier release, which keeps no beat, is judged by the log written beside its address.
/// Why:
///  A beat is what this release keeps, and a service of an earlier release still runs for a while after an upgrade
/// (`service_release_order.rs`); its log is written while it serves, so a log that was just written is a service that works,
/// and a log that is silent for the patience is one that stopped, which is the question a command in front of it asks.
#[test]
fn a_service_with_no_beat_is_judged_by_the_log_beside_it() {
    let root = folder("log-only");
    let log: PathBuf = root.join("3f9a.log");
    std::fs::write(log.as_path(), b"started\n").expect("the log");
    assert!(is_working(root.as_path(), "3f9a"), "a log written now is a service that is working");
    let file: std::fs::File = std::fs::OpenOptions::new().write(true).open(log.as_path()).expect("the log");
    file.set_modified(std::time::SystemTime::now() - BEAT_PATIENCE - std::time::Duration::from_secs(1)).expect("an older log");
    assert!(!is_working(root.as_path(), "3f9a"), "a log silent for the patience is not");
    assert!(!is_working(root.as_path(), "nothing-here"), "neither a beat nor a log is no sign of work");
    let _ = std::fs::remove_dir_all(root.as_path());
}

/// What:
///  The thread that beats keeps moving the file while the process runs, and the removal at the service's end takes only the
///  file that service made.
/// Why:
///  A beat that the reader finds fresh has to be moved by the service itself, and a second service that took the name over has
///  a beat of its own that the first one's end must leave alone.
#[test]
fn the_thread_moves_the_beat_and_an_earlier_services_end_leaves_a_newer_beat() {
    let root = folder("thread");
    let beat: PathBuf = path(root.as_path(), "3f9a");
    let mut taken: Beat = Beat::take(beat.clone()).expect("the beat is made");
    let before: std::time::SystemTime = std::fs::metadata(beat.as_path()).expect("the beat").modified().expect("a time");
    taken.start();
    std::thread::sleep(BEAT_EVERY * 3);
    let after: std::time::SystemTime = std::fs::metadata(beat.as_path()).expect("the beat").modified().expect("a time");
    assert!(after > before, "the thread moved the beat: {before:?} to {after:?}");
    // A second service takes the name over: the first one's end must leave the new file.
    let mut newer: Beat = Beat::take(beat.clone()).expect("a newer beat");
    newer.start();
    taken.remove();
    assert!(beat.exists(), "the beat of the service that is there stays when the earlier one ends");
    newer.remove();
    assert!(!beat.exists(), "and the beat of the service that ends goes");
    let _ = std::fs::remove_dir_all(root.as_path());
}
