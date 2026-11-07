//! Controls for the wait for any program's hold on `index.lock`:
//! it returns at once for a free index,
//! ends when the lock is
//! removed,
//! names the lock and says it changed nothing,
//! and waits for a holder proven alive,
//! and for an owner nothing proves
//! alive,
//! with no bound.
//! Why:
//!  Merges must wait for a transient holder and never remove a lock,
//! so each ending is pinned with a stand-in holder,
//! a
//!  file the control creates and removes,
//! whatever the host's speed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(await waitForIndexRelease({ realIndex })).toEqual({ kind: 'released' });
//! ```

use super::*;
use crate::test_support::{SharedSink, fixture, remove};
use std::time::Duration;

/// A cadence with no grace,
/// so the controls read the notices without waiting for them.
fn quick() -> Cadence {
    return Cadence::from_interval(Duration::from_millis(40));
}

/// What:
///  Holds a lock for `holding` counted from the moment the wait has begun,
/// which its first notice shows.
/// Why:
///   A hold counted from the start of the holding thread begins before the wait's own clock does,
/// so the wait could read a little less
///  than `holding` when the thread that starts the wait was late (`Released(2.49917834s)` for a hold of 2.5 s in the gate of the
///  branch merged with main at `c23b85309c`);
/// the wait speaks only after it began,
/// so the hold counted from its first notice is never
///  longer than what the wait measured.
fn hold_from_the_first_notice_of_the_wait(watching: &SharedSink, holding: Duration) {
    while watching.text().is_empty() {
        std::thread::sleep(Duration::from_millis(5));
    }
    std::thread::sleep(holding);
}

/// A free index is not waited for,
/// and nothing is printed.
#[test]
fn a_free_index_is_not_waited_for() {
    let root: std::path::PathBuf = fixture("holder-wait-free");
    let sink: SharedSink = SharedSink::default();
    let outcome: IndexWait = wait_for_index_release(root.join("index").as_path(), "merge", quick(), &mut sink.clone());
    assert_eq!(outcome, IndexWait::NotHeld);
    assert_eq!(sink.text(), "");
    remove(root.as_path());
}

/// A lock that goes while the command waits ends the wait with a release,
/// after a notice that names the lock and says
///  nothing was changed;
/// the lock the other program held is the other program's to remove,
/// and the wait removes nothing.
#[test]
fn a_released_lock_ends_the_wait_after_a_notice() {
    let root: std::path::PathBuf = fixture("holder-wait-released");
    let index: std::path::PathBuf = root.join("index");
    let lock: std::path::PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("the other program holds the lock");
    let sink: SharedSink = SharedSink::default();
    // The lock goes when the first line has been written, so the control depends on the notice and not on a timer.
    let remover: lock_remover::Remover = lock_remover::Remover::new(lock.clone());
    let mut writer = crate::test_support::ActionSink::new(&sink, 1, move || { remover.remove(); });
    let outcome: IndexWait = wait_for_index_release(index.as_path(), "merge", quick(), &mut writer);
    assert!(matches!(outcome, IndexWait::Released(_)), "{outcome:?}");
    let text: String = sink.text();
    assert!(text.contains("git merge has changed nothing yet and waits for the index lock"), "{text}");
    assert!(text.contains("index.lock"), "the lock is named: {text}");
    assert!(!lock.exists());
    remove(root.as_path());
}

/// An owner nothing proves alive is waited for like one proven alive:
/// the wait goes on through a hold far longer than the second
///  it used to end at,
/// with the line that says what is known and how to remove the lock if no program uses the repository,
/// and it
///  ends when the lock goes.
#[test]
fn an_owner_nothing_proves_alive_is_waited_for_like_a_proven_one() {
    let root: std::path::PathBuf = fixture("holder-wait-unproven-unbounded");
    let index: std::path::PathBuf = root.join("index");
    let lock: std::path::PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("a lock nobody holds open");
    let holding: Duration = LONG_HOLD;
    let remover: lock_remover::Remover = lock_remover::Remover::new(lock.clone());
    let sink: SharedSink = SharedSink::default();
    let watching: SharedSink = sink.clone();
    let letting_go = std::thread::spawn(move || {
        hold_from_the_first_notice_of_the_wait(&watching, holding);
        remover.remove();
    });
    let outcome: IndexWait = wait_for_index_release(index.as_path(), "merge", quick(), &mut sink.clone());
    letting_go.join().expect("the lock goes");
    assert!(matches!(outcome, IndexWait::Released(waited) if waited >= holding), "{outcome:?}");
    let text: String = sink.text();
    assert!(text.contains("cannot be proven alive") && text.contains("has the lock open"), "what is known is said: {text}");
    assert!(text.contains(format!("removing the file {}", lock.display()).as_str()), "the way out is named: {text}");
    assert!(!text.contains("stops waiting"), "an unproven owner is never given up on: {text}");
    remove(root.as_path());
}

/// How long the controls keep a lock:
/// far more than the second an owner nothing proves alive used to be waited for.
const LONG_HOLD: Duration = Duration::from_millis(2500);

/// The time a control gives a wait that must end at once before it calls it hung.
const DEAD_OWNER_PATIENCE: Duration = Duration::from_secs(10);

/// A PID file that names a process which no longer runs ends the wait at once,
/// because nothing will release that lock;
/// the
///  diagnostic names the process.
/// The wait runs on a thread and the control gives it a few seconds,
/// so a dead owner waited for
///  like a live one fails the control instead of hanging it.
#[test]
fn a_dead_owner_ends_the_wait_at_once_with_the_process_named() {
    let root: std::path::PathBuf = fixture("holder-wait-dead");
    let index: std::path::PathBuf = root.join("index");
    let lock: std::path::PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("the lock");
    let gone: i64 = i64::from(gone_process_id());
    std::fs::write(crate::recovery_files::lock_pid_path(index.as_path()), format!("pid {gone}\n")).expect("the PID file");
    let sink: SharedSink = SharedSink::default();
    let (finished, ended) = std::sync::mpsc::channel::<IndexWait>();
    let waiter_index: std::path::PathBuf = index.clone();
    let waiter_sink: SharedSink = sink.clone();
    std::thread::spawn(move || {
        let outcome: IndexWait = wait_for_index_release(waiter_index.as_path(), "merge", quick(), &mut waiter_sink.clone());
        let _sent = finished.send(outcome);
    });
    let outcome: IndexWait = ended.recv_timeout(DEAD_OWNER_PATIENCE).expect("a dead owner was waited for like a live one");
    assert!(matches!(outcome, IndexWait::OwnerGone(_)), "{outcome:?}");
    assert!(sink.text().contains(format!("PID {gone}, which no longer runs").as_str()), "{}", sink.text());
    assert!(lock.exists(), "the wait removes no lock");
    remove(root.as_path());
}

/// Where no watch can be set,
/// the wait polls:
/// a lock that goes after a while ends it,
/// as it does with a watch.
#[test]
fn where_no_watch_can_be_set_the_merge_wait_polls_until_the_lock_goes() {
    let root: std::path::PathBuf = fixture("holder-wait-no-watch");
    let index: std::path::PathBuf = root.join("index");
    let lock: std::path::PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("the lock");
    let remover: lock_remover::Remover = lock_remover::Remover::new(lock.clone());
    let letting_go = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(300));
        remover.remove();
    });
    let sink: SharedSink = SharedSink::default();
    let none = |_: &std::path::Path| -> Option<Watch> { return None; };
    let outcome: IndexWait = wait_with_watches(index.as_path(), "merge", LONG_CEILING, quick(), &mut sink.clone(), &none);
    letting_go.join().expect("the lock goes");
    assert!(matches!(outcome, IndexWait::Released(_)), "{outcome:?}");
    remove(root.as_path());
}

/// The ceiling the long controls give the wait,
/// so only the controls of the ceiling itself reach it.
const LONG_CEILING: Duration = Duration::from_secs(3600);

/// The ceiling the controls of the ceiling give the wait,
/// and the time they give it before they call it hung.
const SHORT_CEILING: Duration = Duration::from_millis(300);
const CEILING_PATIENCE: Duration = Duration::from_secs(10);

/// Runs the wait with the short ceiling on a thread,
/// and returns what it ended with and said,
/// or nothing when it was still waiting
///  after `CEILING_PATIENCE`.
fn wait_at_the_ceiling(index: &std::path::Path) -> Option<(IndexWait, String)> {
    let (finished, ended) = std::sync::mpsc::channel();
    let sink: SharedSink = SharedSink::default();
    let reader: SharedSink = sink.clone();
    let index_path: std::path::PathBuf = index.to_path_buf();
    std::thread::spawn(move || {
        let outcome: IndexWait = wait_for_index_release_within(index_path.as_path(), "merge", SHORT_CEILING, quick(), &mut sink.clone());
        let _sent = finished.send(outcome);
    });
    let outcome: IndexWait = ended.recv_timeout(CEILING_PATIENCE).ok()?;
    return Some((outcome, reader.text()));
}

/// A holder proven alive that keeps the lock past the ceiling is given up on:
/// the line names the process and says the command
/// stops;
/// the lock is left in place.
#[test]
fn a_holder_proven_alive_is_given_up_on_at_the_ceiling() {
    let root: std::path::PathBuf = fixture("holder-wait-ceiling-proven");
    let index: std::path::PathBuf = root.join("index");
    let lock: std::path::PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("the lock");
    std::fs::write(crate::recovery_files::lock_pid_path(index.as_path()), format!("pid {}\n", std::process::id())).expect("the PID file");
    let (outcome, text) = wait_at_the_ceiling(index.as_path()).expect("the wait ended at its ceiling");
    assert!(matches!(outcome, IndexWait::GaveUp(waited) if waited >= SHORT_CEILING), "{outcome:?}");
    assert!(text.contains("still held after") && text.contains(format!("PID {}", std::process::id()).as_str()), "{text}");
    assert!(lock.exists(), "the wait removes no lock");
    remove(root.as_path());
}

/// An owner nothing proves alive is given up on at the ceiling too,
/// with what was looked at.
#[test]
fn an_owner_nothing_proves_alive_is_given_up_on_at_the_ceiling() {
    let root: std::path::PathBuf = fixture("holder-wait-ceiling-unproven");
    let index: std::path::PathBuf = root.join("index");
    let lock: std::path::PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("the lock");
    let (outcome, text) = wait_at_the_ceiling(index.as_path()).expect("the wait ended at its ceiling");
    assert!(matches!(outcome, IndexWait::GaveUp(_)), "{outcome:?}");
    assert!(text.contains("still held after") && text.contains("an owner nothing proves alive"), "{text}");
    assert!(lock.exists());
    remove(root.as_path());
}

/// A holder proven alive is waited for with no bound:
/// the wait goes on through the whole hold,
/// repeats its line naming the
///  holder without ever saying it gives up,
/// and ends when the holder lets go.
#[test]
fn a_holder_proven_alive_is_waited_for_with_no_bound() {
    let root: std::path::PathBuf = fixture("holder-wait-alive");
    let index: std::path::PathBuf = root.join("index");
    let lock: std::path::PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("the lock");
    // The PID file names this process, which runs and started before the lock was made.
    std::fs::write(crate::recovery_files::lock_pid_path(index.as_path()), format!("pid {}\n", std::process::id())).expect("the PID file");
    let holding: Duration = LONG_HOLD;
    let remover: lock_remover::Remover = lock_remover::Remover::new(lock.clone());
    let sink: SharedSink = SharedSink::default();
    let watching: SharedSink = sink.clone();
    let letting_go = std::thread::spawn(move || {
        hold_from_the_first_notice_of_the_wait(&watching, holding);
        remover.remove();
    });
    let outcome: IndexWait = wait_for_index_release(index.as_path(), "merge", quick(), &mut sink.clone());
    letting_go.join().expect("the holder lets go");
    assert!(matches!(outcome, IndexWait::Released(waited) if waited >= holding), "{outcome:?}");
    let text: String = sink.text();
    assert!(text.contains("proven by its Git lock PID file"), "{text}");
    assert!(text.contains("still waiting for"), "the line repeats while the holder holds: {text}");
    assert!(!text.contains("giving up") && !text.contains("stops waiting"), "a proven holder is never given up on: {text}");
    remove(root.as_path());
}

/// The id of a process that has ended,
/// so a PID file can name one that is gone.
fn gone_process_id() -> u32 {
    let mut child = std::process::Command::new("true").spawn().expect("a short process starts");
    let id: u32 = child.id();
    child.wait().expect("it ends");
    return id;
}

/// A small helper owning the lock path,
/// so the sink's action can be `Send`.
mod lock_remover {
    /// Removes one file.
    pub(super) struct Remover(std::path::PathBuf);

    impl Remover {
        /// A remover of `path`.
        pub(super) fn new(path: std::path::PathBuf) -> Self {
            return Self(path);
        }

        /// Remove the file.
        pub(super) fn remove(&self) {
            std::fs::remove_file(&self.0).expect("the other program lets go");
        }
    }
}
