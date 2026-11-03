//! Controls for the wait for any program's hold on `index.lock`: it returns at once for a free index, ends when the lock is
//! removed, names the lock and says it changed nothing, and gives up at its limit leaving the lock in place.
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

/// A lock still held at the limit is left in place, and the wait says so.
#[test]
fn a_lock_held_at_the_limit_is_left_in_place() {
    let root: std::path::PathBuf = fixture("holder-wait-limit");
    let index: std::path::PathBuf = root.join("index");
    let lock: std::path::PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("the other program holds the lock");
    let sink: SharedSink = SharedSink::default();
    let outcome: IndexWait = wait_for_index_release(index.as_path(), "merge", Duration::from_millis(200), quick(), &mut sink.clone());
    assert!(matches!(outcome, IndexWait::GaveUp(waited) if waited >= Duration::from_millis(200)), "{outcome:?}");
    assert!(lock.exists(), "the wait removes no lock");
    assert!(sink.text().contains("nothing was changed and the lock was left in place"), "{}", sink.text());
    remove(root.as_path());
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
