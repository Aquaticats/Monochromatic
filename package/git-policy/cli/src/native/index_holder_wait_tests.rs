//! Controls for the wait for any program's hold on `index.lock`: it returns at once for a free index, ends when the lock is
//! removed, names the lock and says it changed nothing, waits for a holder proven alive with no bound, and gives an owner
//! nothing proves alive a short backoff and a diagnostic, leaving the lock in place.
//! Why:
//!  Merges must wait for a transient holder and never remove a lock, so each ending is pinned with a stand-in holder, a
//!  file the control creates and removes, whatever the host's speed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(await waitForIndexRelease({ realIndex, limit })).toEqual({ kind: 'released' });
//! ```

use super::*;
use crate::test_support::{SharedSink, fixture, remove};
use std::time::Duration;

/// A cadence with no grace, so the controls read the notices without waiting for them.
fn quick() -> Cadence {
    return Cadence::from_interval(Duration::from_millis(40));
}

/// A free index is not waited for, and nothing is printed.
#[test]
fn a_free_index_is_not_waited_for() {
    let root: std::path::PathBuf = fixture("holder-wait-free");
    let sink: SharedSink = SharedSink::default();
    let outcome: IndexWait = wait_for_index_release(root.join("index").as_path(), "merge", Duration::from_secs(30), quick(), &mut sink.clone());
    assert_eq!(outcome, IndexWait::NotHeld);
    assert_eq!(sink.text(), "");
    remove(root.as_path());
}

/// A lock that goes while the command waits ends the wait with a release, after a notice that names the lock and says
///  nothing was changed; the lock the other program held is the other program's to remove, and the wait removes nothing.
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
    let outcome: IndexWait = wait_for_index_release(index.as_path(), "merge", Duration::from_secs(60), quick(), &mut writer);
    assert!(matches!(outcome, IndexWait::Released(_)), "{outcome:?}");
    let text: String = sink.text();
    assert!(text.contains("git merge has changed nothing yet and waits for the index lock"), "{text}");
    assert!(text.contains("index.lock"), "the lock is named: {text}");
    assert!(!lock.exists());
    remove(root.as_path());
}

/// A lock nobody can be shown to hold is looked at for the backoff and no longer; the wait then lists what it found, names
///  the file to remove if no program is using the repository, and leaves the lock in place.
#[test]
fn an_owner_nothing_proves_alive_gets_the_backoff_and_a_diagnostic() {
    let root: std::path::PathBuf = fixture("holder-wait-unproven");
    let index: std::path::PathBuf = root.join("index");
    let lock: std::path::PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("a lock nobody holds open");
    let sink: SharedSink = SharedSink::default();
    let outcome: IndexWait = wait_for_index_release(index.as_path(), "merge", Duration::from_millis(200), quick(), &mut sink.clone());
    assert!(matches!(outcome, IndexWait::Unproven(waited) if waited >= Duration::from_millis(200)), "{outcome:?}");
    assert!(lock.exists(), "the wait removes no lock");
    let text: String = sink.text();
    assert!(text.contains("cannot be proven alive"), "{text}");
    assert!(text.contains("has the lock open") && text.contains("no Git lock PID file"), "the evidence is listed: {text}");
    assert!(text.contains("nothing was changed and the lock was left in place"), "{text}");
    assert!(text.contains(format!("removing the file {}", lock.display()).as_str()), "the way out is named: {text}");
    remove(root.as_path());
}

/// An owner nothing proves alive is waited for like one proven alive: the backoff, which used to end the wait, is a twentieth of
///  the time the lock stays, the wait goes on through twenty times it with the line that says what is known and how to remove the
///  lock if no program uses the repository, and it ends when the lock goes.
#[test]
fn an_owner_nothing_proves_alive_is_waited_for_like_a_proven_one() {
    let root: std::path::PathBuf = fixture("holder-wait-unproven-unbounded");
    let index: std::path::PathBuf = root.join("index");
    let lock: std::path::PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("a lock nobody holds open");
    let backoff: Duration = Duration::from_millis(50);
    let holding: Duration = backoff * 20;
    let remover: lock_remover::Remover = lock_remover::Remover::new(lock.clone());
    let letting_go = std::thread::spawn(move || {
        std::thread::sleep(holding);
        remover.remove();
    });
    let sink: SharedSink = SharedSink::default();
    let outcome: IndexWait = wait_for_index_release(index.as_path(), "merge", backoff, quick(), &mut sink.clone());
    letting_go.join().expect("the lock goes");
    assert!(matches!(outcome, IndexWait::Released(waited) if waited >= holding), "{outcome:?}");
    let text: String = sink.text();
    assert!(text.contains("cannot be proven alive") && text.contains("has the lock open"), "what is known is said: {text}");
    assert!(text.contains(format!("removing the file {}", lock.display()).as_str()), "the way out is named: {text}");
    assert!(!text.contains("stops waiting"), "an unproven owner is never given up on: {text}");
    remove(root.as_path());
}

/// The backoff of the dead-owner control: long enough that a wait taken for the backoff of an unproven owner is told apart from
///  one that ended at once, short enough that the control fails in seconds when the dead owner is not recognized.
const DEAD_OWNER_BACKOFF: Duration = Duration::from_secs(20);

/// A PID file that names a process which no longer runs ends the wait at once, however long the backoff is, because nothing
///  will release that lock; the diagnostic names the process.
#[test]
fn a_dead_owner_ends_the_wait_at_once_with_the_process_named() {
    let root: std::path::PathBuf = fixture("holder-wait-dead");
    let index: std::path::PathBuf = root.join("index");
    let lock: std::path::PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("the lock");
    let gone: i64 = i64::from(gone_process_id());
    std::fs::write(crate::recovery_files::lock_pid_path(index.as_path()), format!("pid {gone}\n")).expect("the PID file");
    let sink: SharedSink = SharedSink::default();
    let outcome: IndexWait = wait_for_index_release(index.as_path(), "merge", DEAD_OWNER_BACKOFF, quick(), &mut sink.clone());
    assert!(matches!(outcome, IndexWait::OwnerGone(waited) if waited < DEAD_OWNER_BACKOFF / 2), "{outcome:?}");
    assert!(sink.text().contains(format!("PID {gone}, which no longer runs").as_str()), "{}", sink.text());
    assert!(lock.exists(), "the wait removes no lock");
    remove(root.as_path());
}

/// A holder proven alive is waited for with no bound: the backoff, which is all that bounds an owner nothing proves alive, is
///  a twentieth of the time the holder keeps the lock, the wait goes on through twenty times it, repeats its line naming the
///  holder without ever saying it gives up, and ends when the holder lets go.
#[test]
fn a_holder_proven_alive_is_waited_for_far_beyond_the_backoff() {
    let root: std::path::PathBuf = fixture("holder-wait-alive");
    let index: std::path::PathBuf = root.join("index");
    let lock: std::path::PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("the lock");
    // The PID file names this process, which runs and started before the lock was made.
    std::fs::write(crate::recovery_files::lock_pid_path(index.as_path()), format!("pid {}\n", std::process::id())).expect("the PID file");
    let backoff: Duration = Duration::from_millis(50);
    let holding: Duration = backoff * 20;
    let remover: lock_remover::Remover = lock_remover::Remover::new(lock.clone());
    let letting_go = std::thread::spawn(move || {
        std::thread::sleep(holding);
        remover.remove();
    });
    let sink: SharedSink = SharedSink::default();
    let outcome: IndexWait = wait_for_index_release(index.as_path(), "merge", backoff, quick(), &mut sink.clone());
    letting_go.join().expect("the holder lets go");
    assert!(matches!(outcome, IndexWait::Released(waited) if waited >= holding), "{outcome:?}");
    let text: String = sink.text();
    assert!(text.contains("proven by its Git lock PID file"), "{text}");
    assert!(text.contains("still waiting for"), "the line repeats while the holder holds: {text}");
    assert!(!text.contains("giving up") && !text.contains("stops waiting"), "a proven holder is never given up on: {text}");
    remove(root.as_path());
}

/// The id of a process that has ended, so a PID file can name one that is gone.
fn gone_process_id() -> u32 {
    let mut child = std::process::Command::new("true").spawn().expect("a short process starts");
    let id: u32 = child.id();
    child.wait().expect("it ends");
    return id;
}

/// A small helper owning the lock path, so the sink's action can be `Send`.
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
