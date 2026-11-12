//! Controls for the wait for any program's hold on `index.lock`:
//! it returns at once for a free index,
//! ends when the lock is
//! removed,
//! names the lock and says it changed nothing,
//! and waits for a holder proven alive,
//! and for an owner nothing proves
//! alive,
//! until the ceiling (`MERGE_LOCK_CEILING`).
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
    assert!(text.contains(format!("for at most {} seconds", MERGE_LOCK_CEILING.as_secs()).as_str()), "the line says how long the wait lasts before it ends: {text}");
    assert!(!text.contains("stops waiting"), "a notice while the wait goes on never says the command stopped; only the ceiling's line does: {text}");
    remove(root.as_path());
}

/// How long the controls keep a lock:
/// far more than the second an owner nothing proves alive used to be waited for.
const LONG_HOLD: Duration = Duration::from_millis(2500);

/// The shortest ceiling a wait can be given:
/// the wait's clock has already run out when it begins,
/// so an ending that comes before the ceiling is proven to take no time over it,
/// however long the host stalls.
const NO_CEILING: Duration = Duration::ZERO;

/// A PID file that holds a number this process table does not have ends the wait at once,
/// because no process here will release that lock;
/// the
///  diagnostic names the number,
/// says that a Git of another PID namespace writes one this table has never had,
/// and says what the command that reads it does about the lock.
/// A merge then clears the lock on its own proof of disuse or leaves it where it is
/// (`service_index_unheld_removal.rs`),
/// so no commit is failed by this ending
/// (cross-vendor review of 2026-11-09,
/// finding 2).
/// The ceiling is already out when the wait begins,
/// so a number waited for like a live holder's ends in the ceiling's words instead,
/// at once and without a stopwatch to read.
#[test]
fn a_number_this_table_does_not_have_ends_the_wait_at_once_with_the_number_named() {
    let root: std::path::PathBuf = fixture("holder-wait-dead");
    let index: std::path::PathBuf = root.join("index");
    let lock: std::path::PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("the lock");
    let gone: i64 = i64::from(gone_process_id());
    std::fs::write(crate::recovery_files::lock_pid_path(index.as_path()), format!("pid {gone}\n")).expect("the PID file");
    let sink: SharedSink = SharedSink::default();
    let outcome: IndexWait = wait_for_index_release_within(index.as_path(), "merge", NO_CEILING, quick(), &mut sink.clone());
    assert!(matches!(outcome, IndexWait::OwnerGone(_)), "{outcome:?}");
    let text: String = sink.text();
    assert!(text.contains(format!("PID number {gone}").as_str()), "the number the PID file holds is said: {text}");
    assert!(text.contains("which this process table does not have"), "the number is said for what it is: {text}");
    assert!(text.contains("another PID namespace"), "why this table does not have it is said: {text}");
    assert!(!text.contains("still held after"), "such an owner is not waited for: {text}");
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

/// The ceiling the controls of the ceiling give the wait.
/// The wait ends by itself at this ceiling,
/// so the control reads the ending rather than timing it,
/// and a wait that never ends is the whole run's watchdog to report.
const SHORT_CEILING: Duration = Duration::from_millis(300);

/// Runs the wait with the short ceiling on the control's own thread,
/// and returns what it ended with and said.
fn wait_at_the_ceiling(index: &std::path::Path) -> (IndexWait, String) {
    let sink: SharedSink = SharedSink::default();
    let outcome: IndexWait = wait_for_index_release_within(index, "merge", SHORT_CEILING, quick(), &mut sink.clone());
    return (outcome, sink.text());
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
    let (outcome, text) = wait_at_the_ceiling(index.as_path());
    assert!(matches!(outcome, IndexWait::GaveUp(waited) if waited >= SHORT_CEILING), "{outcome:?}");
    assert!(text.contains("still held after") && text.contains(format!("PID {}", std::process::id()).as_str()), "{text}");
    assert!(lock.exists(), "the wait removes no lock");
    remove(root.as_path());
}

/// An owner nothing proves alive is given up on at the ceiling too,
/// after the whole ceiling has passed,
/// with what was looked at.
#[test]
fn an_owner_nothing_proves_alive_is_given_up_on_at_the_ceiling() {
    let root: std::path::PathBuf = fixture("holder-wait-ceiling-unproven");
    let index: std::path::PathBuf = root.join("index");
    let lock: std::path::PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("the lock");
    let (outcome, text) = wait_at_the_ceiling(index.as_path());
    assert!(matches!(outcome, IndexWait::GaveUp(waited) if waited >= SHORT_CEILING), "{outcome:?}");
    assert!(text.contains("still held after") && text.contains("an owner nothing proves alive"), "{text}");
    assert!(lock.exists());
    remove(root.as_path());
}

/// A holder proven alive is waited for until it lets go:
/// the wait goes on through the whole hold,
/// repeats its line naming the
///  holder,
/// and ends when the holder lets go,
/// well inside the one-minute ceiling it takes (`MERGE_LOCK_CEILING`,
/// the control of that ceiling is `a_holder_proven_alive_is_given_up_on_at_the_ceiling`).
#[test]
fn a_holder_proven_alive_is_waited_for_until_the_lock_is_released() {
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
    assert!(text.contains(format!("for at most {} seconds", MERGE_LOCK_CEILING.as_secs()).as_str()), "the line says how long the wait lasts before it ends: {text}");
    assert!(!text.contains("giving up") && !text.contains("stops waiting"), "a notice while the wait goes on never says the wait ended; only the ceiling's line does: {text}");
    remove(root.as_path());
}

/// What:
///  The wait a merge makes before its own apply says the merge is saved for a later command,
///  not that Git refuses it.
/// Why:
///  A held lock is not the merge's failure (`SPEC.md`,
///  "Merges and commits never fail"):
/// the merge is saved on a side ref and exits 0,
///  so a line promising another command's refusal sends the person to a command that does not fail and leaves the
///  saved merge unnamed.
#[test]
fn the_merge_wait_says_the_merge_is_saved_not_that_git_refuses() {
    let root: std::path::PathBuf = fixture("holder-wait-merge-saved");
    let index: std::path::PathBuf = root.join("index");
    let lock: std::path::PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("the other program holds the lock");
    let sink: SharedSink = SharedSink::default();
    let remover: lock_remover::Remover = lock_remover::Remover::new(lock.clone());
    let mut writer = crate::test_support::ActionSink::new(&sink, 1, move || { remover.remove(); });
    let outcome: IndexWait = wait_for_index_release_before_a_merge_that_clears_a_dead_lock(index.as_path(), &mut writer);
    assert!(matches!(outcome, IndexWait::Released(_)), "{outcome:?}");
    let text: String = sink.text();
    assert!(text.contains("the merge is saved"), "the saved merge is named: {text}");
    assert!(!text.contains("reports what Git says"), "the line promises no report the merge does not make: {text}");
    assert!(!text.contains("Git's own refusal"), "the line promises no refusal the merge does not make: {text}");
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

/// What:
///  A merge's one look at the index names the program that holds it and gives up at once,
///  and names a dead owner whose lock the merge may clear.
/// Why:
///   A merge has an answer that needs no wait,
/// so sixty seconds of waiting on another agent's command bought
///  nothing (a cross-vendor audit of alpha.12,
/// "A merge waits for another agent's live index holder").
///  The clock is read
///  around the look,
/// so a look that waited would be the control's failure rather than a slow host's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writePidFile({ pid: process.pid }); const began = performance.now(); expect(lookAtTheIndexBeforeAMerge(index, sink).kind).toBe('held-by-another-command'); expect(performance.now() - began).toBeLessThan(2000);
/// ```
#[test]
fn a_merge_looks_once_at_the_index_and_never_waits_for_a_holder() {
    let root: std::path::PathBuf = fixture("holder-look-merge");
    let index: std::path::PathBuf = root.join("index");
    let lock: std::path::PathBuf = root.join("index.lock");
    let sink: SharedSink = SharedSink::default();
    assert_eq!(look_at_the_index_before_a_merge(index.as_path(), &mut sink.clone()), IndexWait::NotHeld, "a free index ends the look");
    assert_eq!(sink.text(), "", "a free index prints nothing");
    std::fs::write(&lock, b"").expect("the other program's lock");
    std::fs::write(crate::recovery_files::lock_pid_path(index.as_path()), format!("pid {}\n", std::process::id())).expect("the PID file");
    let began: std::time::Instant = std::time::Instant::now();
    let looked: IndexWait = look_at_the_index_before_a_merge(index.as_path(), &mut sink.clone());
    assert!(began.elapsed() < Duration::from_secs(2), "the look does not wait: {:?}", began.elapsed());
    match looked {
        IndexWait::HeldByAnotherCommand(holder) => {
            assert!(holder.contains(format!("PID {}", std::process::id()).as_str()), "the holder is named: {holder}");
        }
        other => panic!("a holder ends the look at once: {other:?}"),
    }
    assert!(lock.exists(), "the look removes no lock");
    remove(root.as_path());
}

/// A lock a gone process left is named by the same look,
/// so the merge clears it on the proof a commit acts on.
#[test]
fn a_merge_looks_once_at_the_index_and_names_a_dead_owner() {
    let root: std::path::PathBuf = fixture("holder-look-merge-dead");
    let index: std::path::PathBuf = root.join("index");
    let lock: std::path::PathBuf = root.join("index.lock");
    std::fs::write(&lock, b"").expect("the lock a killed Git left");
    let gone: i64 = i64::from(gone_process_id());
    std::fs::write(crate::recovery_files::lock_pid_path(index.as_path()), format!("pid {gone}\n")).expect("the PID file");
    let sink: SharedSink = SharedSink::default();
    let looked: IndexWait = look_at_the_index_before_a_merge(index.as_path(), &mut sink.clone());
    assert!(matches!(looked, IndexWait::OwnerGone(_)), "{looked:?}");
    let text: String = sink.text();
    assert!(text.contains(format!("PID number {gone}").as_str()), "the dead owner is named: {text}");
    assert!(lock.exists(), "the look removes no lock");
    remove(root.as_path());
}
