//! What:
//!  The wait of a command that is about to change the checkout for any program's hold on Git's `index.lock`:
//! the commit
//!  service's,
//! a foreign `git`'s,
//! or a lock nothing proves held.
//! Event-driven,
//! with a notice that names what is waited for
//!  and the holder,
//! and over as soon as the lock is gone.
//! A holder proven alive and an owner that cannot be proven alive are
//!  both waited for until `MERGE_LOCK_CEILING` (one minute,
//! `wait_ceilings.rs`),
//! which is where the wait names the blocker and ends and the merge goes on to meet Git's own refusal;
//!  the line for the second says what is known and how to remove the lock if no program uses the repository.
//! A PID file naming
//!  a process that is gone ends the wait at once with a diagnostic.
//! Why:
//!  Native Git never waits for `index.lock`:
//! a merge that finds it held dies (`Unable to create ... index.lock` or,
//! from the
//!  stash that every merge commit takes first,
//! `could not write index` and `stash failed`).
//! Other sessions commit into a
//!  shared checkout all the time,
//! so a held lock is ordinary and passes by itself.
//! This wait is the sibling of the wait for
//!  the commit service's own hold (`service_index_wait.rs`),
//! which returns at once for a foreign holder because a plain
//!  forwarded writer keeps the behavior of native Git;
//! a merge is held to the owner's rule that merges never fail
//!  (`SPEC.md`,
//! "Merges and commits never fail").
//! The wait blocks on an inotify watch set before the first look,
//! so a
//!  release between the look and the wait is not missed;
//! where no watch can be set it looks again after a short pause.
//!  It never removes a lock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await waitForIndexRelease({ realIndex, command: 'merge', cadence, stderr });
//! ```

/// Evidence about the holder of a lock.
use crate::index_lock_evidence::{
    Evidence, Verdict, WaitEnd, blocker_words, ceiling_line, classify_index_lock, gather_with, host_pid_process, proven_holder_line, proven_holder_progress_line, unproven_wait_line,
};
/// The bounded scan of open descriptors.
use crate::index_lock_holders::scan_platform_holders_bounded;
/// The place of the lock and the watch on its directory.
use crate::service_index_lock::{Watch, lock_path_of};
/// The longest a wait lasts.
use crate::wait_ceilings::MERGE_LOCK_CEILING;
/// How long a wait without a watch sleeps between looks.
use crate::service_index_watch::POLL_PAUSE;
use crate::wait_progress::{Cadence, Ticker};
/// Writing the notices.
use std::io::Write;
/// `Path` is a borrowed filesystem path.
use std::path::Path;
/// Time spans and the monotonic clock.
use std::time::{Duration, Instant};

/// What came of the wait.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexWait {
    /// Nobody held the lock when the command asked.
    NotHeld,
    /// A holder had the lock and let it go.
    Released(
        /// How long the wait lasted.
        Duration,
    ),
    /// The lock was still held at the ceiling;
    /// the line naming the blocker was written,
    /// nothing was changed and the lock was left
    ///  in place.
    GaveUp(
        /// How long the wait lasted.
        Duration,
    ),
}

/// What the last look at the lock found.
enum Proof {
    /// A holder is proven alive.
    Alive(Evidence, i64, bool, Option<String>),
    /// Nothing proves a holder alive.
    Nothing(Option<Evidence>),
}

/// What the evidence for `real_index` proves now.
fn look_at(real_index: &Path, gather: &dyn Fn(&Path) -> std::io::Result<Option<Evidence>>) -> Proof {
    let Ok(Some(evidence)) = gather(real_index) else {
        return Proof::Nothing(None);
    };
    return match classify_index_lock(&evidence) {
        Verdict::ProvenAlive { pid, by_descriptor, command } => Proof::Alive(evidence, pid, by_descriptor, command),
        Verdict::EvidenceFree => Proof::Nothing(Some(evidence)),
    };
}

/// The evidence of the host:
/// the Git lock PID file and the bounded scan of open descriptors.
fn gather_here(real_index: &Path) -> std::io::Result<Option<Evidence>> {
    return gather_with(real_index, host_pid_process, scan_platform_holders_bounded);
}

/// The notice:
/// what the command waits for,
/// why it is safe to wait,
/// then the holder,
/// and how long the wait lasts before the command meets Git's own refusal.
fn notice(command: &str, real_index: &Path, waited: Duration, ceiling: Duration, proof: &Proof, first: bool) -> String {
    let ending: String = format!("git {command} meets Git's own refusal");
    let end: WaitEnd<'_> = WaitEnd { within: ceiling, ending: ending.as_str() };
    let tail: String = match proof {
        Proof::Alive(evidence, pid, by_descriptor, holder_command) => {
            let line: String = if first {
                proven_holder_line(*pid, *by_descriptor, holder_command.as_deref(), evidence.lock_path.as_path(), end)
            } else {
                proven_holder_progress_line(*pid, *by_descriptor, holder_command.as_deref(), evidence.lock_path.as_path(), waited.as_secs(), end)
            };
            line
        }
        Proof::Nothing(found) => unproven_wait_line(lock_path_of(real_index).as_path(), found.as_ref(), end),
    };
    return format!(
        "monochromatic-git-policy-cli: git {command} has changed nothing yet and waits for the index lock of this repository; {}",
        tail.strip_prefix("monochromatic-git-policy-cli: ").unwrap_or(tail.as_str())
    );
}

/// What:
///  Wait while any program holds `index.lock` of `real_index`,
/// telling `out` what is waited for.
/// Why:
///   Returns after one metadata read when nothing holds the lock.
/// The first line comes at once for a holder that is already
///  known when the wait begins and then every notice interval,
/// so a queued command can be told from a hung one.
/// A holder
///  proven alive and an owner nothing proves alive are waited for until `MERGE_LOCK_CEILING`,
/// not forever:
///  the person's own `git commit` with its editor open holds the lock with no proof,
/// and a number in the lock's PID file that this process table does not have may be a live Git's of another
///  PID namespace,
///  which holds the lock through its editor with no descriptor open
///  (cross-vendor review of 2026-11-09,
/// finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await waitForIndexRelease(realIndex, 'merge', cadence, stderr)
/// ```
pub fn wait_for_index_release(real_index: &Path, command: &str, cadence: Cadence, out: &mut dyn Write) -> IndexWait {
    return wait_for_index_release_within(real_index, command, MERGE_LOCK_CEILING, cadence, out);
}

/// `wait_for_index_release` that gives up after `ceiling`.
pub fn wait_for_index_release_within(real_index: &Path, command: &str, ceiling: Duration, cadence: Cadence, out: &mut dyn Write) -> IndexWait {
    return wait_with_watches(real_index, command, ceiling, cadence, out, &Watch::on);
}

/// Write `bytes` to `out`,
/// then flush it only if the write succeeded;
/// the callers ignore the result,
/// because a closed terminal
///  is not a reason to stop the command the person asked for.
fn write_then_flush(out: &mut dyn Write, bytes: &[u8]) -> std::io::Result<()> {
    out.write_all(bytes)?;
    return out.flush();
}

/// `wait_for_index_release` with the function that sets a watch on the lock's directory,
/// so a control can hand it none.
fn wait_with_watches(real_index: &Path, command: &str, ceiling: Duration, cadence: Cadence, out: &mut dyn Write, set_watch: &dyn Fn(&Path) -> Option<Watch>) -> IndexWait {
    let lock = lock_path_of(real_index);
    // The watch comes before the first look: a release between the look and the wait is then seen as an event.
    let mut watch: Option<Watch> = lock.parent().and_then(set_watch);
    let lock_name: std::ffi::OsString = lock.file_name().map(std::ffi::OsStr::to_os_string).unwrap_or_default();
    let consequence: String = format!("git {command} stops waiting and reports what Git says");
    let began: Instant = Instant::now();
    let mut ticker: Ticker = Ticker::announced_at_start(cadence);
    let mut held: bool = false;
    let mut announced: bool = false;
    let mut proof: Proof = Proof::Nothing(None);
    loop {
        if std::fs::symlink_metadata(&lock).is_err() {
            return if held { IndexWait::Released(began.elapsed()) } else { IndexWait::NotHeld };
        }
        held = true;
        let elapsed: Duration = began.elapsed();
        // The evidence is read when the first line is due and when a repeat is due, so a scan of the open descriptors is not made
        //  for every wake of the watch.
        let due: bool = !announced || ticker.due(elapsed);
        if due || elapsed >= ceiling {
            proof = look_at(real_index, &gather_here);
        }
        if due {
            // A closed terminal is not a reason to stop the command the person asked for.
            let _noted = write_then_flush(out, notice(command, real_index, elapsed, ceiling, &proof, !announced).as_bytes());
            announced = true;
        }
        // The owner, 2026-11-03: "nothing should ever wait forever or indefinitely". At the ceiling the wait names the blocker and ends;
        //  the merge then runs into Git's own refusal and exits non-zero, with nothing changed.
        if elapsed >= ceiling {
            let blocker: String = match &proof {
                Proof::Alive(evidence, ..) => blocker_words(evidence),
                Proof::Nothing(Some(evidence)) => blocker_words(evidence),
                Proof::Nothing(None) => String::from("an owner whose evidence could not be read"),
            };
            let line: String = ceiling_line(lock.as_path(), blocker.as_str(), elapsed.as_secs(), consequence.as_str());
            let _noted = write_then_flush(out, line.as_bytes());
            return IndexWait::GaveUp(elapsed);
        }
        let pause: Duration = ticker.next_in(elapsed).min(ceiling.saturating_sub(elapsed)).max(Duration::from_millis(1));
        match watch.as_ref() {
            Some(active) => {
                if !active.wait(pause, lock_name.as_os_str()) {
                    // The watch ended with its directory: the ended watch is dropped first, because a new one may need the
                    //  descriptor and the quota it holds, then the lock is looked at again with a new watch set.
                    drop(watch.take());
                    watch = lock.parent().and_then(set_watch);
                }
            }
            None => std::thread::sleep(pause.min(POLL_PAUSE)),
        }
    }
}

/// `wait_for_index_release` with the notice cadence this process runs under,
/// on standard error.
pub fn wait_for_index_release_here(real_index: &Path, command: &str) -> IndexWait {
    return wait_for_index_release(real_index, command, Cadence::from_environment(), &mut std::io::stderr());
}

/// The wait is controlled in the test module.
#[cfg(test)]
#[path = "index_holder_wait_tests.rs"]
mod tests;
