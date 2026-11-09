//! Controls for the wait on the commit service's hold of the index:
//! who counts as the holder,
//! the event-driven release,
//! the
//!  notice,
//! and the ceiling that ends a wait on a holder proven to run.

use super::*;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// A cadence with no grace and a short interval,
/// so a short wait still prints.
const QUICK: Cadence = Cadence { grace: Duration::from_millis(20), interval: Duration::from_millis(20) };

/// A directory with an index path in it.
fn index_in(root: &Path) -> PathBuf {
    return root.join("index");
}

/// The mark the service writes,
/// for batch `seq`.
fn marked(seq: u64) -> String {
    return format!("{LOCK_MARKER}{seq}\n");
}

/// A notice sink that makes the holder let go when the first notice is written,
/// and keeps what was written.
/// Why:
///   The notice is the wait's own report that it has looked at the held lock and is waiting,
///   so a release made there happens after the look at the point the control means,
/// with no clock deciding it.
struct ReleaseOnNotice {
    lock: PathBuf,
    text: Vec<u8>,
    released: bool,
}

impl ReleaseOnNotice {
    fn holding(lock: &Path) -> ReleaseOnNotice {
        return ReleaseOnNotice { lock: lock.to_path_buf(), text: Vec::new(), released: false };
    }

    fn written(&self) -> String {
        return String::from_utf8_lossy(&self.text).into_owned();
    }
}

impl Write for ReleaseOnNotice {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.text.extend_from_slice(bytes);
        if !self.released {
            self.released = true;
            std::fs::remove_file(&self.lock).expect("release");
        }
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// Starts a thread that removes `lock` as soon as anything opens it,
/// which the wait does at its first look.
/// Why:
///   The release follows the look because the look is what triggers it,
/// so no clock decides whether the wait finds the lock
///   held.
/// The watch is set before this returns,
/// so an open that follows is not missed.
fn release_when_opened(lock: &Path) -> std::thread::JoinHandle<()> {
    use std::os::unix::ffi::OsStrExt;
    let directory = std::ffi::CString::new(lock.parent().expect("a directory").as_os_str().as_bytes()).expect("a path");
    // SAFETY: plain system calls with valid arguments; the descriptor is closed by the thread.
    let descriptor: i32 = unsafe { libc::inotify_init1(libc::IN_CLOEXEC) };
    assert!(descriptor >= 0, "inotify is available");
    // SAFETY: `directory` is a valid NUL-terminated string for the call.
    let added: i32 = unsafe { libc::inotify_add_watch(descriptor, directory.as_ptr(), libc::IN_OPEN) };
    assert!(added >= 0, "the directory is watched");
    let target: PathBuf = lock.to_path_buf();
    return std::thread::spawn(move || {
        let name: Vec<u8> = target.file_name().expect("a name").as_bytes().to_vec();
        loop {
            let mut buffer: [u8; 4096] = [0; 4096];
            // SAFETY: the buffer is valid for its length.
            let read: isize = unsafe { libc::read(descriptor, buffer.as_mut_ptr().cast(), buffer.len()) };
            let count: usize = usize::try_from(read).expect("events were read");
            if buffer[..count].windows(name.len()).any(|window| { return window == name.as_slice(); }) {
                break;
            }
        }
        // SAFETY: the descriptor belongs to this thread and is closed once.
        unsafe { libc::close(descriptor) };
        std::fs::remove_file(&target).expect("release");
    });
}

/// Nobody,
/// the service by its mark,
/// and anything else are told apart;
/// an empty lock is a foreign program's.
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

/// A lock written in two steps,
/// the mark arriving a moment after the file exists,
/// is still the service's.
/// The mark is written inside the first pause between two reads,
/// so the read that follows it is the one that finds it,
/// whatever the host's speed.
#[test]
fn a_mark_that_arrives_a_moment_late_is_still_the_services() {
    let root: PathBuf = fixture("index-wait-late");
    let lock: PathBuf = lock_path_of(index_in(root.as_path()).as_path());
    std::fs::write(&lock, "").expect("lock");
    let pauses: std::cell::Cell<u32> = std::cell::Cell::new(0);
    let writing = || {
        pauses.set(pauses.get() + 1);
        std::fs::write(&lock, marked(1)).expect("mark");
    };
    assert_eq!(holder_after_pauses(lock.as_path(), &writing), Holder::Service);
    assert_eq!(pauses.get(), 1, "the mark was found by the read that followed the first pause");
    remove(root.as_path());
}

/// A lock that stays empty through every read is a foreign program's,
/// after the reads are used up.
#[test]
fn a_lock_that_stays_empty_is_foreign_after_the_reads_are_used_up() {
    let root: PathBuf = fixture("index-wait-empty");
    let lock: PathBuf = lock_path_of(index_in(root.as_path()).as_path());
    std::fs::write(&lock, "").expect("lock");
    let pauses: std::cell::Cell<u32> = std::cell::Cell::new(0);
    let counting = || { pauses.set(pauses.get() + 1); };
    assert_eq!(holder_after_pauses(lock.as_path(), &counting), Holder::Foreign);
    assert_eq!(pauses.get(), EMPTY_LOOKS - 1, "one pause between each two reads");
    remove(root.as_path());
}

/// No lock and a foreign lock end the wait at once,
/// and print nothing.
#[test]
fn nothing_is_waited_for_when_nobody_or_a_foreign_program_holds_the_index() {
    let root: PathBuf = fixture("index-wait-none");
    let index: PathBuf = index_in(root.as_path());
    let mut out: Vec<u8> = Vec::new();
    assert_eq!(wait_for_service_hold(index.as_path(), "add", QUICK, &mut out), Waited::NotHeld);
    std::fs::write(lock_path_of(index.as_path()), b"not the service\n").expect("lock");
    assert_eq!(wait_for_service_hold(index.as_path(), "add", QUICK, &mut out), Waited::NotHeld);
    assert!(out.is_empty(), "{}", String::from_utf8_lossy(&out));
    remove(root.as_path());
}

/// The service's hold is waited for until it is released,
/// the release is seen as an event,
/// and the notice names the command
///  and the service.
/// The holder lets go when the first notice is written,
/// so the release follows the wait's look at the held lock.
#[test]
fn the_services_hold_is_waited_for_until_it_is_released() {
    let root: PathBuf = fixture("index-wait-release");
    let index: PathBuf = index_in(root.as_path());
    let lock: PathBuf = lock_path_of(index.as_path());
    std::fs::write(&lock, marked(3)).expect("lock");
    let mut out: ReleaseOnNotice = ReleaseOnNotice::holding(lock.as_path());
    let outcome: Waited = wait_for_service_hold(index.as_path(), "rebase", QUICK, &mut out);
    assert_eq!(outcome, Waited::Released);
    let text: String = out.written();
    assert!(text.contains("git rebase") && text.contains("commit service"), "{text}");
    assert!(!text.contains("index.lock") && !text.contains("lock"), "no internal name reaches the person: {text}");
    remove(root.as_path());
}

/// A hold the mark proves to run,
/// kept past the ceiling,
/// is given up on:
/// the line says the command runs now and reports what Git
/// says,
/// and the lock is not touched.
#[test]
fn a_hold_that_outlasts_the_ceiling_ends_the_wait_and_stays() {
    let root: PathBuf = fixture("index-wait-ceiling");
    let index: PathBuf = index_in(root.as_path());
    let lock: PathBuf = lock_path_of(index.as_path());
    let identity: String = crate::service_index_marker::own_identity().expect("identity");
    std::fs::write(&lock, format!("{LOCK_MARKER}9 {} {identity}\n", std::process::id())).expect("lock");
    let mut out: Vec<u8> = Vec::new();
    let outcome: Waited = wait_for_service_hold_within(index.as_path(), "add", Duration::from_millis(300), QUICK, &mut out);
    let text: String = String::from_utf8_lossy(&out).into_owned();
    assert_eq!(outcome, Waited::GaveUp);
    assert!(text.contains("still finishing a commit") && text.contains("git add runs now"), "{text}");
    assert!(lock.exists(), "the hold is not the command's to remove");
    remove(root.as_path());
}

/// Where no watch can be set,
/// a hold the mark proves to run is polled for until it is released,
/// where the wait used to return at
///  once and let the command run into Git's refusal.
#[test]
fn where_no_watch_can_be_set_the_services_hold_is_polled_for() {
    let root: PathBuf = fixture("index-wait-no-watch-poll");
    let index: PathBuf = index_in(root.as_path());
    let lock: PathBuf = lock_path_of(index.as_path());
    let identity: String = crate::service_index_marker::own_identity().expect("identity");
    std::fs::write(&lock, format!("{LOCK_MARKER}9 {} {identity}\n", std::process::id())).expect("lock");
    let remover_lock: PathBuf = lock.clone();
    let letting_go = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(300));
        std::fs::remove_file(&remover_lock).expect("the service lets go");
    });
    let no_watch = |_: &Path| -> Option<Watch> { return None; };
    let mut out: Vec<u8> = Vec::new();
    let outcome: Waited = wait_with_watches(index.as_path(), "add", SERVICE_HOLD_CEILING, QUICK, &mut out, &no_watch);
    letting_go.join().expect("the service lets go");
    assert_eq!(outcome, Waited::Released, "{}", String::from_utf8_lossy(&out));
    remove(root.as_path());
}

/// A holder the mark proves to run is waited for until it lets go:
/// the lock is looked at,
/// notices repeat over many intervals
///  without ever saying the wait gives up,
/// and the wait ends when the holder lets go.
#[test]
fn a_holder_the_mark_proves_to_run_is_waited_for_until_it_lets_go() {
    let root: PathBuf = fixture("index-wait-no-bound");
    let index: PathBuf = index_in(root.as_path());
    let lock: PathBuf = lock_path_of(index.as_path());
    let identity: String = crate::service_index_marker::own_identity().expect("identity");
    std::fs::write(&lock, format!("{LOCK_MARKER}9 {} {identity}\n", std::process::id())).expect("lock");
    let holding: Duration = QUICK.interval * 20;
    let remover_lock: PathBuf = lock.clone();
    let letting_go = std::thread::spawn(move || {
        std::thread::sleep(holding);
        std::fs::remove_file(&remover_lock).expect("the service lets go");
    });
    let mut out: Vec<u8> = Vec::new();
    assert_eq!(wait_for_service_hold(index.as_path(), "add", QUICK, &mut out), Waited::Released);
    letting_go.join().expect("the service lets go");
    let text: String = String::from_utf8_lossy(&out).into_owned();
    assert!(text.matches("waiting for the commit service").count() > 1, "the line repeats while the service holds: {text}");
    assert!(text.contains(format!("for at most {} seconds in all", SERVICE_HOLD_CEILING.as_secs()).as_str()), "the line says how long the wait lasts before the command runs: {text}");
    assert!(!text.contains("giving up") && !text.contains("runs now"), "a notice while the wait goes on never says the command gave up or ran; only the ceiling's line does: {text}");
    remove(root.as_path());
}

/// A hold older than a landing can last is a dead service's leftover:
/// it is not waited for,
/// it says so,
/// and it stays.
#[test]
fn a_hold_older_than_any_landing_is_not_waited_for() {
    let root: PathBuf = fixture("index-wait-stale");
    let index: PathBuf = index_in(root.as_path());
    let lock: PathBuf = lock_path_of(index.as_path());
    std::fs::write(&lock, marked(5)).expect("lock");
    let long_ago: std::time::SystemTime = std::time::SystemTime::now() - UNASKABLE_HOLD_AGE - Duration::from_secs(60);
    std::fs::File::options().write(true).open(&lock).expect("open").set_modified(long_ago).expect("age");
    let mut out: Vec<u8> = Vec::new();
    assert_eq!(wait_for_service_hold(index.as_path(), "add", QUICK, &mut out), Waited::Stale);
    assert!(String::from_utf8_lossy(&out).contains("does not wait for it"), "{}", String::from_utf8_lossy(&out));
    assert!(lock.exists());
    remove(root.as_path());
}

/// A landing that ends within the grace prints nothing.
/// The holder lets go when the wait first opens the lock,
/// so the release always follows the look.
#[test]
fn a_hold_released_within_the_grace_prints_nothing() {
    let root: PathBuf = fixture("index-wait-grace");
    let index: PathBuf = index_in(root.as_path());
    let lock: PathBuf = lock_path_of(index.as_path());
    std::fs::write(&lock, marked(4)).expect("lock");
    let releaser = release_when_opened(lock.as_path());
    let mut out: Vec<u8> = Vec::new();
    let slow: Cadence = Cadence { grace: Duration::from_secs(30), interval: Duration::from_secs(30) };
    assert_eq!(wait_for_service_hold(index.as_path(), "add", slow, &mut out), Waited::Released);
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
            // The holder lets go as the new watch is set, after the wait found the lock held and the first watch over.
            std::fs::remove_file(&lock).expect("release");
            return Watch::on(directory);
        }
        let ending: Option<Watch> = Watch::on(decoy.as_path());
        std::fs::remove_dir(&decoy).expect("the watched directory goes");
        return ending;
    };
    let began: Instant = Instant::now();
    let mut out: Vec<u8> = Vec::new();
    let slow: Cadence = Cadence { grace: Duration::from_secs(30), interval: Duration::from_secs(30) };
    let outcome: Waited = wait_with_watches(index.as_path(), "add", SERVICE_HOLD_CEILING, slow, &mut out, &set_watch);
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
    let long_ago: std::time::SystemTime = std::time::SystemTime::now() - UNASKABLE_HOLD_AGE - Duration::from_secs(60);
    std::fs::File::options().write(true).open(&lock).expect("open").set_modified(long_ago).expect("age");
    // The holder lets go when the first notice is written: the checks of age and of the holder have found the hold current.
    let mut waiting: ReleaseOnNotice = ReleaseOnNotice::holding(lock.as_path());
    assert_eq!(wait_for_service_hold(index.as_path(), "add", QUICK, &mut waiting), Waited::Released);
    assert!(!waiting.written().contains("does not wait for it"), "{}", waiting.written());
    assert!(waiting.written().contains("waiting for the commit service"), "the hold was waited for: {}", waiting.written());
    let mut out: Vec<u8> = Vec::new();
    let parts: Vec<&str> = identity.split('#').collect();
    std::fs::write(&lock, format!("{LOCK_MARKER}8 {} {}#{}#{}\n", std::process::id(), parts[0], "0".repeat(32), parts[2])).expect("lock of another boot");
    std::fs::File::options().write(true).open(&lock).expect("open").set_modified(long_ago).expect("age");
    assert_eq!(wait_for_service_hold(index.as_path(), "add", QUICK, &mut out), Waited::Stale);
    remove(root.as_path());
}

/// The wait that sets a new watch after the first ended drops the first before it asks for the next:
/// with the quota used up by the first,
/// the second is created only then.
/// The holder lets go inside the second call,
/// once the quota has been checked,
/// so no clock decides whether the lock is still held when the wait looks at it.
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
        if crate::service_index_watch::live_watches_on_this_thread() > 0 {
            return None;
        }
        std::fs::remove_file(&lock).expect("release");
        return Watch::on(directory);
    };
    let began: Instant = Instant::now();
    let mut out: Vec<u8> = Vec::new();
    let slow: Cadence = Cadence { grace: Duration::from_secs(30), interval: Duration::from_secs(30) };
    let outcome: Waited = wait_with_watches(index.as_path(), "add", SERVICE_HOLD_CEILING, slow, &mut out, &quota_of_one);
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
    assert_eq!(wait_for_service_hold(index.as_path(), "add", QUICK, &mut out), Waited::Stale);
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
    // A young hold of that kind is waited for: the holder lets go when the first notice is written.
    let mut waiting: ReleaseOnNotice = ReleaseOnNotice::holding(lock.as_path());
    assert_eq!(wait_for_service_hold(index.as_path(), "add", QUICK, &mut waiting), Waited::Released);
    let said: String = waiting.written();
    assert!(!said.contains("no longer runs"), "{said}");
    std::fs::write(&lock, format!("{LOCK_MARKER}9 2147483646 {}#{}#1\n", parts[0], parts[1])).expect("lock again");
    let long_ago: std::time::SystemTime = std::time::SystemTime::now() - UNASKABLE_HOLD_AGE - Duration::from_secs(60);
    std::fs::File::options().write(true).open(&lock).expect("open").set_modified(long_ago).expect("age");
    let mut old: Vec<u8> = Vec::new();
    assert_eq!(wait_for_service_hold(index.as_path(), "add", QUICK, &mut old), Waited::Stale);
    assert!(String::from_utf8_lossy(old.as_slice()).contains("does not wait for it"), "{}", String::from_utf8_lossy(old.as_slice()));
    remove(root.as_path());
}

/// What:
///  A lock that holds a named pipe, or a link to one, is a foreign program's at once.
/// Why:
///  Opening a named pipe for reading waits for a writer that may never come, so this look held a
/// wrapped `git add` for ever and every commit behind it
/// (`doc/troubleshooting/a-fifo-where-a-file-is-expected-ends-no-command.md`).
#[test]
fn a_lock_that_is_not_a_plain_file_is_foreign_at_once() {
    let root: PathBuf = fixture("index-wait-pipe");
    let index: PathBuf = index_in(root.as_path());
    let lock: PathBuf = lock_path_of(index.as_path());
    crate::subvolume_sync_attrs::make_fifo(lock.as_path(), 0o600).expect("named pipe");
    let named_pipe: PathBuf = lock.clone();
    let looked: Option<Holder> = crate::test_support::finished_within(move || { return holder_of(named_pipe.as_path()); });
    let Some(holder) = looked else { panic!("looking at a named pipe lock did not finish within five seconds") };
    assert_eq!(holder, Holder::Foreign);
    std::fs::remove_file(lock.as_path()).expect("the pipe goes");
    let pipe: PathBuf = root.join("pipe");
    crate::subvolume_sync_attrs::make_fifo(pipe.as_path(), 0o600).expect("named pipe");
    std::os::unix::fs::symlink(pipe.as_path(), lock.as_path()).expect("a link to the pipe");
    let linked: PathBuf = lock.clone();
    let looked: Option<Holder> = crate::test_support::finished_within(move || { return holder_of(linked.as_path()); });
    let Some(holder) = looked else { panic!("looking at a link to a named pipe did not finish within five seconds") };
    assert_eq!(holder, Holder::Foreign, "a link to a pipe is not the service's lock");
    remove(root.as_path());
}

/// What:
///  A command waits for no hold of a lock that is a named pipe:
/// the wait ends at once,
/// saying nothing.
/// Why:
///  The look that tells the holders apart is the one that must not wait, so the wait itself is
/// held to the same bound here.
#[test]
fn a_command_waits_for_no_hold_of_a_named_pipe() {
    let root: PathBuf = fixture("index-wait-pipe-command");
    let index: PathBuf = index_in(root.as_path());
    let lock: PathBuf = lock_path_of(index.as_path());
    crate::subvolume_sync_attrs::make_fifo(lock.as_path(), 0o600).expect("named pipe");
    let bounded: PathBuf = index.clone();
    let waited: Option<(Waited, Vec<u8>)> = crate::test_support::finished_within(move || {
        let mut out: Vec<u8> = Vec::new();
        let waited: Waited = wait_for_service_hold(bounded.as_path(), "add", QUICK, &mut out);
        return (waited, out);
    });
    let Some((waited, out)) = waited else { panic!("waiting over a named pipe lock did not finish within five seconds") };
    assert_eq!(waited, Waited::NotHeld);
    assert!(out.is_empty(), "{}", String::from_utf8_lossy(out.as_slice()));
    remove(root.as_path());
}
