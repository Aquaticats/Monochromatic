//! Controls for the wait on the commit service's hold of the index: who counts as the holder, the event-driven release, the
//!  notice, and the limit.

use super::*;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// A cadence with no grace and a short interval, so a short wait still prints.
const QUICK: Cadence = Cadence { grace: Duration::from_millis(20), interval: Duration::from_millis(20) };

/// A directory with an index path in it.
fn index_in(root: &Path) -> PathBuf {
    return root.join("index");
}

/// The mark the service writes, for batch `seq`.
fn marked(seq: u64) -> String {
    return format!("{LOCK_MARKER}{seq}\n");
}

/// Nobody, the service by its mark, and anything else are told apart; an empty lock is a foreign program's.
#[test]
fn the_holder_is_told_by_the_mark() {
    let root: PathBuf = fixture("index-wait-holder");
    let lock: PathBuf = lock_path_of(index_in(root.as_path()).as_path());
    assert_eq!(holder_of(lock.as_path()), Holder::Nobody);
    std::fs::write(&lock, marked(7)).expect("lock");
    assert_eq!(holder_of(lock.as_path()), Holder::Service);
    std::fs::write(&lock, "").expect("lock");
    assert_eq!(holder_of(lock.as_path()), Holder::Foreign, "an empty lock is a program that has not written its index yet");
    std::fs::write(&lock, "DIRC\0\0\0\u{2}").expect("lock");
    assert_eq!(holder_of(lock.as_path()), Holder::Foreign);
    std::fs::write(&lock, &LOCK_MARKER.as_bytes()[..LOCK_MARKER.len() / 2]).expect("lock");
    assert_eq!(holder_of(lock.as_path()), Holder::Foreign, "half a mark is no mark");
    remove(root.as_path());
}

/// A lock written in two steps, the mark arriving a moment after the file exists, is still the service's.
#[test]
fn a_mark_that_arrives_a_moment_late_is_still_the_services() {
    let root: PathBuf = fixture("index-wait-late");
    let lock: PathBuf = lock_path_of(index_in(root.as_path()).as_path());
    std::fs::write(&lock, "").expect("lock");
    let writer: PathBuf = lock.clone();
    let late = std::thread::spawn(move || {
        std::thread::sleep(EMPTY_PAUSE);
        std::fs::write(writer, marked(1)).expect("mark");
    });
    assert_eq!(holder_of(lock.as_path()), Holder::Service);
    late.join().expect("writer");
    remove(root.as_path());
}

/// No lock and a foreign lock end the wait at once, and print nothing.
#[test]
fn nothing_is_waited_for_when_nobody_or_a_foreign_program_holds_the_index() {
    let root: PathBuf = fixture("index-wait-none");
    let index: PathBuf = index_in(root.as_path());
    let mut out: Vec<u8> = Vec::new();
    assert_eq!(wait_for_service_hold(index.as_path(), "add", Duration::from_secs(5), QUICK, &mut out), Waited::NotHeld);
    std::fs::write(lock_path_of(index.as_path()), b"not the service\n").expect("lock");
    assert_eq!(wait_for_service_hold(index.as_path(), "add", Duration::from_secs(5), QUICK, &mut out), Waited::NotHeld);
    assert!(out.is_empty(), "{}", String::from_utf8_lossy(&out));
    remove(root.as_path());
}

/// The service's hold is waited for until it is released, the release is seen as an event, and the notice names the command
///  and the service.
#[test]
fn the_services_hold_is_waited_for_until_it_is_released() {
    let root: PathBuf = fixture("index-wait-release");
    let index: PathBuf = index_in(root.as_path());
    let lock: PathBuf = lock_path_of(index.as_path());
    std::fs::write(&lock, marked(3)).expect("lock");
    let released: PathBuf = lock.clone();
    let releaser = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(150));
        std::fs::remove_file(released).expect("release");
    });
    let began: Instant = Instant::now();
    let mut out: Vec<u8> = Vec::new();
    let outcome: Waited = wait_for_service_hold(index.as_path(), "rebase", Duration::from_secs(30), QUICK, &mut out);
    releaser.join().expect("releaser");
    assert_eq!(outcome, Waited::Released);
    assert!(began.elapsed() < Duration::from_secs(5), "the release woke the wait: {:?}", began.elapsed());
    let text: String = String::from_utf8_lossy(&out).into_owned();
    assert!(text.contains("git rebase") && text.contains("commit service"), "{text}");
    assert!(!text.contains("index.lock") && !text.contains("lock"), "no internal name reaches the person: {text}");
    remove(root.as_path());
}

/// A hold that outlasts the limit ends the wait with a plain line, and the lock is not touched.
#[test]
fn a_hold_that_outlasts_the_limit_ends_the_wait_and_stays() {
    let root: PathBuf = fixture("index-wait-limit");
    let index: PathBuf = index_in(root.as_path());
    let lock: PathBuf = lock_path_of(index.as_path());
    std::fs::write(&lock, marked(9)).expect("lock");
    let mut out: Vec<u8> = Vec::new();
    assert_eq!(wait_for_service_hold(index.as_path(), "add", Duration::from_millis(100), QUICK, &mut out), Waited::GaveUp);
    let text: String = String::from_utf8_lossy(&out).into_owned();
    assert!(text.contains("still finishing a commit") && text.contains("git add runs now"), "{text}");
    assert!(lock.exists(), "the hold is not the command's to remove");
    remove(root.as_path());
}

/// A hold older than a landing can last is a dead service's leftover: it is not waited for, it says so, and it stays.
#[test]
fn a_hold_older_than_any_landing_is_not_waited_for() {
    let root: PathBuf = fixture("index-wait-stale");
    let index: PathBuf = index_in(root.as_path());
    let lock: PathBuf = lock_path_of(index.as_path());
    std::fs::write(&lock, marked(5)).expect("lock");
    let long_ago: std::time::SystemTime = std::time::SystemTime::now() - HOLD_LIMIT - Duration::from_secs(60);
    std::fs::File::options().write(true).open(&lock).expect("open").set_modified(long_ago).expect("age");
    let began: Instant = Instant::now();
    let mut out: Vec<u8> = Vec::new();
    assert_eq!(wait_for_service_hold(index.as_path(), "add", Duration::from_secs(60), QUICK, &mut out), Waited::Stale);
    assert!(began.elapsed() < Duration::from_secs(5), "{:?}", began.elapsed());
    assert!(String::from_utf8_lossy(&out).contains("does not wait for it"), "{}", String::from_utf8_lossy(&out));
    assert!(lock.exists());
    remove(root.as_path());
}

/// A landing that ends within the grace prints nothing.
#[test]
fn a_hold_released_within_the_grace_prints_nothing() {
    let root: PathBuf = fixture("index-wait-grace");
    let index: PathBuf = index_in(root.as_path());
    let lock: PathBuf = lock_path_of(index.as_path());
    std::fs::write(&lock, marked(4)).expect("lock");
    let released: PathBuf = lock.clone();
    let releaser = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        std::fs::remove_file(released).expect("release");
    });
    let mut out: Vec<u8> = Vec::new();
    let slow: Cadence = Cadence { grace: Duration::from_secs(30), interval: Duration::from_secs(30) };
    assert_eq!(wait_for_service_hold(index.as_path(), "add", Duration::from_secs(60), slow, &mut out), Waited::Released);
    releaser.join().expect("releaser");
    assert!(out.is_empty(), "{}", String::from_utf8_lossy(&out));
    remove(root.as_path());
}

/// A mark that names a holder which is gone is a dead service's leftover whatever its age: the command does not wait out a
///  landing that no process is making, and tells the person what became of the hold and what removes it.
#[test]
fn a_mark_whose_holder_is_gone_is_not_waited_for() {
    let root: PathBuf = fixture("index-wait-gone-holder");
    let index: PathBuf = index_in(root.as_path());
    let lock: PathBuf = lock_path_of(index.as_path());
    // A process number no system hands out, with a birth identity nothing has.
    std::fs::write(&lock, format!("{LOCK_MARKER}7 2147483646 no-such-birth\n")).expect("lock");
    let mut out: Vec<u8> = Vec::new();
    let began: Instant = Instant::now();
    assert_eq!(wait_for_service_hold(index.as_path(), "add", Duration::from_secs(30), QUICK, &mut out), Waited::Stale);
    assert!(began.elapsed() < Duration::from_secs(10), "{:?}", began.elapsed());
    let said: String = String::from_utf8_lossy(out.as_slice()).into_owned();
    assert!(said.contains("no longer runs"), "{said}");
    assert!(said.contains("next commit"), "{said}");
    assert!(lock.exists(), "the leftover is not the command's to remove");
    remove(root.as_path());
}
