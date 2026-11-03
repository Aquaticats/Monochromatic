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
    let mut out: Vec<u8> = Vec::new();
    let outcome: Waited = wait_for_service_hold(index.as_path(), "rebase", Duration::from_secs(30), QUICK, &mut out);
    releaser.join().expect("releaser");
    assert_eq!(outcome, Waited::Released);
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

/// A limit no test outlasts:
/// a wait that is wrongly taken for a leftover's hold runs into the per-test limit and is reported as
/// a hang,
/// whatever the host's load,
/// where a wall-clock assertion would fail a correct wait on a slow host.
const LIMIT_NO_TEST_OUTLASTS: Duration = Duration::from_secs(24 * 60 * 60);

/// A hold older than a landing can last is a dead service's leftover: it is not waited for, it says so, and it stays.
#[test]
fn a_hold_older_than_any_landing_is_not_waited_for() {
    let root: PathBuf = fixture("index-wait-stale");
    let index: PathBuf = index_in(root.as_path());
    let lock: PathBuf = lock_path_of(index.as_path());
    std::fs::write(&lock, marked(5)).expect("lock");
    let long_ago: std::time::SystemTime = std::time::SystemTime::now() - HOLD_LIMIT - Duration::from_secs(60);
    std::fs::File::options().write(true).open(&lock).expect("open").set_modified(long_ago).expect("age");
    let mut out: Vec<u8> = Vec::new();
    assert_eq!(wait_for_service_hold(index.as_path(), "add", LIMIT_NO_TEST_OUTLASTS, QUICK, &mut out), Waited::Stale);
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

/// A watch that ends while the service holds the index (its directory unmounted or removed) does not leave the wait asleep to its timer:
/// the wait sets a new watch,
/// and the release that follows is seen as an event.
/// The first watch here is one on a directory that is removed as the watch is set,
/// so it reports its end at once;
/// the cadence's interval is far longer than the test lasts,
/// so only an event can end the wait in time.
#[test]
fn a_watch_that_ended_is_set_again_and_the_release_that_follows_is_seen() {
    let root: PathBuf = fixture("index-wait-watch-ended");
    let index: PathBuf = index_in(root.as_path());
    let lock: PathBuf = lock_path_of(index.as_path());
    std::fs::write(&lock, marked(6)).expect("lock");
    let decoy: PathBuf = root.join("decoy");
    std::fs::create_dir(&decoy).expect("decoy directory");
    let set: std::cell::Cell<u32> = std::cell::Cell::new(0);
    let set_watch = |directory: &Path| -> Option<Watch> {
        set.set(set.get() + 1);
        if set.get() > 1 {
            return Watch::on(directory);
        }
        let ending: Option<Watch> = Watch::on(decoy.as_path());
        std::fs::remove_dir(&decoy).expect("the watched directory goes");
        return ending;
    };
    let released: PathBuf = lock.clone();
    let releaser = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(300));
        std::fs::remove_file(released).expect("release");
    });
    let began: Instant = Instant::now();
    let mut out: Vec<u8> = Vec::new();
    let slow: Cadence = Cadence { grace: Duration::from_secs(30), interval: Duration::from_secs(30) };
    let outcome: Waited = wait_with_watches(index.as_path(), "add", Duration::from_secs(60), slow, &mut out, &set_watch);
    releaser.join().expect("releaser");
    assert_eq!(outcome, Waited::Released);
    assert_eq!(set.get(), 2, "a new watch was set once the first had ended");
    // The poll is 30 seconds apart: a release seen long before the next poll was seen as an event, and the wall time against
    // the interval is the property under test.
    assert!(began.elapsed() < slow.interval, "the release woke the wait, not the slow poll: {:?}", began.elapsed());
    remove(root.as_path());
}

/// A hold whose marker names a process that runs now,
/// in this boot,
/// is a landing in progress however old the file is:
/// it is waited for and its release is seen,
/// not called a leftover.
/// A marker of another boot with the same text is a leftover.
#[test]
fn a_hold_whose_process_runs_is_not_stale_by_age() {
    let root: PathBuf = fixture("index-wait-live-old");
    let index: PathBuf = index_in(root.as_path());
    let lock: PathBuf = lock_path_of(index.as_path());
    let identity: String = crate::service_index_marker::own_identity().expect("identity");
    std::fs::write(&lock, format!("{LOCK_MARKER}8 {} {identity}\n", std::process::id())).expect("lock");
    let long_ago: std::time::SystemTime = std::time::SystemTime::now() - HOLD_LIMIT - Duration::from_secs(60);
    std::fs::File::options().write(true).open(&lock).expect("open").set_modified(long_ago).expect("age");
    let released: PathBuf = lock.clone();
    let releaser = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(200));
        std::fs::remove_file(released).expect("release");
    });
    let mut out: Vec<u8> = Vec::new();
    assert_eq!(wait_for_service_hold(index.as_path(), "add", Duration::from_secs(30), QUICK, &mut out), Waited::Released);
    releaser.join().expect("releaser");
    assert!(!String::from_utf8_lossy(&out).contains("does not wait for it"), "{}", String::from_utf8_lossy(&out));
    let parts: Vec<&str> = identity.split('#').collect();
    std::fs::write(&lock, format!("{LOCK_MARKER}8 {} {}#{}#{}\n", std::process::id(), parts[0], "0".repeat(32), parts[2])).expect("lock of another boot");
    std::fs::File::options().write(true).open(&lock).expect("open").set_modified(long_ago).expect("age");
    assert_eq!(wait_for_service_hold(index.as_path(), "add", LIMIT_NO_TEST_OUTLASTS, QUICK, &mut out), Waited::Stale);
    remove(root.as_path());
}

/// The wait that sets a new watch after the first ended drops the first before it asks for the next:
/// with the quota used up by the first,
/// the second is created only then.
#[test]
fn the_wait_drops_an_ended_watch_before_it_sets_the_next() {
    let root: PathBuf = fixture("index-wait-watch-quota");
    let index: PathBuf = index_in(root.as_path());
    let lock: PathBuf = lock_path_of(index.as_path());
    std::fs::write(&lock, marked(6)).expect("lock");
    let decoy: PathBuf = root.join("decoy");
    std::fs::create_dir(&decoy).expect("decoy");
    let calls: std::cell::Cell<u32> = std::cell::Cell::new(0);
    let quota_of_one = |directory: &Path| -> Option<Watch> {
        calls.set(calls.get() + 1);
        if calls.get() == 1 {
            let ending: Watch = Watch::on(decoy.as_path()).expect("watch");
            std::fs::remove_dir(&decoy).expect("the watched directory goes");
            return Some(ending);
        }
        return if crate::service_index_watch::live_watches_on_this_thread() > 0 { None } else { Watch::on(directory) };
    };
    let released: PathBuf = lock.clone();
    let releaser = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(300));
        std::fs::remove_file(released).expect("release");
    });
    let began: Instant = Instant::now();
    let mut out: Vec<u8> = Vec::new();
    let slow: Cadence = Cadence { grace: Duration::from_secs(30), interval: Duration::from_secs(30) };
    let outcome: Waited = wait_with_watches(index.as_path(), "add", Duration::from_secs(60), slow, &mut out, &quota_of_one);
    releaser.join().expect("releaser");
    assert_eq!(outcome, Waited::Released);
    assert_eq!(calls.get(), 2);
    // The release is seen as an event, long before the poll that is 30 seconds away.
    assert!(began.elapsed() < slow.interval, "{:?}", began.elapsed());
    remove(root.as_path());
}

/// A mark that names a holder which is gone is a dead service's leftover whatever its age:
/// the command does not wait out a landing that no process is making,
/// and tells the person what became of the hold and what removes it.
#[test]
fn a_mark_whose_holder_is_gone_is_not_waited_for() {
    let root: PathBuf = fixture("index-wait-gone-holder");
    let index: PathBuf = index_in(root.as_path());
    let lock: PathBuf = lock_path_of(index.as_path());
    // A process number no system hands out, with a birth identity nothing has.
    std::fs::write(&lock, format!("{LOCK_MARKER}7 2147483646 no-such-birth\n")).expect("lock");
    let mut out: Vec<u8> = Vec::new();
    assert_eq!(wait_for_service_hold(index.as_path(), "add", LIMIT_NO_TEST_OUTLASTS, QUICK, &mut out), Waited::Stale);
    let said: String = String::from_utf8_lossy(out.as_slice()).into_owned();
    assert!(said.contains("no longer runs"), "{said}");
    assert!(said.contains("next commit"), "{said}");
    assert!(lock.exists(), "the leftover is not the command's to remove");
    remove(root.as_path());
}

/// A hold whose marker names a PID namespace this reader is not in says nothing about the holder:
/// the number may be a process of the reader's own namespace or none.
/// The wrapper neither calls the live service gone nor waits for it as a proven holder;
/// it falls back to the age of the lock,
/// as for a mark that names no holder.
#[test]
fn a_hold_of_another_pid_namespace_is_neither_gone_nor_proven() {
    let root: PathBuf = fixture("index-wait-foreign-namespace");
    let index: PathBuf = index_in(root.as_path());
    let lock: PathBuf = lock_path_of(index.as_path());
    let identity: String = crate::service_index_marker::own_identity().expect("identity");
    let parts: Vec<&str> = identity.split('#').collect();
    // A process number nothing runs under here, in a namespace this process is not in.
    std::fs::write(&lock, format!("{LOCK_MARKER}9 2147483646 {}#{}#1\n", parts[0], parts[1])).expect("lock");
    let mut out: Vec<u8> = Vec::new();
    assert_eq!(wait_for_service_hold(index.as_path(), "add", Duration::from_millis(150), QUICK, &mut out), Waited::GaveUp);
    let said: String = String::from_utf8_lossy(out.as_slice()).into_owned();
    assert!(!said.contains("no longer runs"), "{said}");
    let long_ago: std::time::SystemTime = std::time::SystemTime::now() - HOLD_LIMIT - Duration::from_secs(60);
    std::fs::File::options().write(true).open(&lock).expect("open").set_modified(long_ago).expect("age");
    let mut old: Vec<u8> = Vec::new();
    assert_eq!(wait_for_service_hold(index.as_path(), "add", LIMIT_NO_TEST_OUTLASTS, QUICK, &mut old), Waited::Stale);
    assert!(String::from_utf8_lossy(old.as_slice()).contains("does not wait for it"), "{}", String::from_utf8_lossy(old.as_slice()));
    remove(root.as_path());
}
