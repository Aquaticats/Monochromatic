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
//! which is where the wait names the blocker and ends and the merge is saved on a side ref for a later command to
//!  finish (`SPEC.md`,
//! "Merges and commits never fail");
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
//!  A merge that has nothing of itself in the checkout does not wait at all:
//! one look at the lock decides whether a gone
//!  process left it,
//! and the command goes on either way (`look_at_the_index_lock_once`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await waitForIndexRelease({ realIndex, command: 'merge', cadence, stderr });
//! ```

/// Evidence about the holder of a lock.
use crate::index_lock_evidence::{
    DEAD_OWNER_LOCK_LEFT, Evidence, Verdict, WaitEnd, blocker_words, ceiling_line, classify_index_lock, dead_owner_diagnostic, gather_with, proven_holder_line, proven_holder_progress_line, unproven_wait_line,
};

/// The sentence a merge's ceiling line ends on before its own apply.
/// A held lock is not that merge's failure:
///  it is saved on a side ref and exits 0,
/// and the next command in the worktree finishes the checkout.
const MERGE_CONSEQUENCE: &str = "the merge stops waiting and saves itself on a ref of its own for the next command in this worktree to finish";

/// The sentence the notices before that ceiling end on:
/// the same outcome,
/// said without the stop the ceiling line alone announces.
const MERGE_NOTICE_ENDING: &str = "the merge is saved on a ref of its own for the next command in this worktree to finish";

/// The sentence a command's dead owner's line ends on when nothing else holds the lock:
/// the command clears it and goes on.
pub const CLEARS_A_DEAD_OWNERS_LOCK: &str = "the merge clears that lock when nothing else holds it and goes on; if anything else holds it, nothing was changed and the lock was left in place";

/// The bounded scan of open descriptors.
use crate::index_lock_holders::scan_platform_holders_bounded;
/// The process a PID file names,
/// as this host's process table reports it.
use crate::index_lock_evidence::host_pid_process;
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
    /// The lock's PID file names a process that no longer runs,
    /// so nothing will release it;
    /// the diagnostic was written,
    ///  nothing was changed and the lock was left in place.
    OwnerGone(
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
    /// The PID file names a process that is gone.
    Dead(i64),
    /// Nothing proves a holder alive or dead.
    Nothing(Option<Evidence>),
}

/// What the evidence for `real_index` proves now.
fn look_at(real_index: &Path, gather: &dyn Fn(&Path) -> std::io::Result<Option<Evidence>>) -> Proof {
    let Ok(Some(evidence)) = gather(real_index) else {
        return Proof::Nothing(None);
    };
    return match classify_index_lock(&evidence) {
        Verdict::ProvenAlive { pid, by_descriptor, command } => Proof::Alive(evidence, pid, by_descriptor, command),
        Verdict::Dead(pid) => Proof::Dead(pid),
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
/// and how long the wait lasts before the ending the wait's own caller states happens.
fn notice(command: &str, real_index: &Path, waited: Duration, ceiling: Duration, proof: &Proof, first: bool, ending: &str) -> String {
    let end: WaitEnd<'_> = WaitEnd { within: ceiling, ending };
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
        Proof::Dead(_) => unproven_wait_line(lock_path_of(real_index).as_path(), None, end),
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
/// not forever,
/// because the person's own `git commit` with its editor open holds the lock with no proof;
/// a dead owner ends the wait at once.
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

/// What the one look at the index lock found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexLook = { kind: 'free' } | { kind: 'held' } | { kind: 'left-by-a-gone-process'; pid: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexLook {
    /// Nobody holds it.
    Free,
    /// A program holds it,
    /// or nothing proves which:
    /// either way a merge with nothing of itself in the checkout does not wait.
    Held,
    /// The PID file names a process that no longer runs,
    /// so nothing will release it.
    LeftByAGoneProcess(
        /// The number the PID file holds.
        i64,
    ),
}

/// What:
///  The index lock,
/// looked at once and never waited for.
/// Why:
///  A merge that is saved on its record and its side ref has nothing of itself in the checkout,
/// so a lock cannot be in
///  its way:
/// the next command finishes the merge once the lock is free.
/// Waiting cost every command (a `git rev-parse HEAD`
///  included) the whole ceiling again for as long as the lock stood,
/// and a lock a killed Git left never goes by
///  itself (finding 3 of the merge-and-pull audit of 2026-11-09).
/// The holder is classified exactly as the wait
///  classifies it,
/// so a lock a gone process left is treated the same way by both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const look = lookAtTheIndexLockOnce(realIndex); // 'free' | 'held' | { leftByAGoneProcess: pid }
/// ```
pub fn look_at_the_index_lock_once(real_index: &Path) -> IndexLook {
    if std::fs::symlink_metadata(lock_path_of(real_index)).is_err() {
        return IndexLook::Free;
    }
    return match look_at(real_index, &gather_here) {
        Proof::Dead(pid) => IndexLook::LeftByAGoneProcess(pid),
        Proof::Alive(..) | Proof::Nothing(_) => IndexLook::Held,
    };
}

/// What:
///  The sentence that says what becomes of the command when a recovery's wait for a held index reaches its bound.
/// Why:
///  A recovery of an interrupted merge writes nothing when the index stays held,
///  and a later command finishes the merge instead,
/// unlike the merge that waits for the lock before its own apply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const consequence = 'the interrupted merge stays unfinished and a later command finishes it once that program is done';
/// ```
pub const RECOVERY_CONSEQUENCE: &str = "the interrupted merge stays unfinished, and the next command in this worktree finishes it once that program is done";

/// The sentence the recovery's notices before that ceiling end on:
/// the same outcome,
/// said without the stop the ceiling line alone announces.
const RECOVERY_NOTICE_ENDING: &str = "the interrupted merge stays unfinished, and the next command in this worktree finishes it";

/// What:
///  Wait while any program holds `index.lock` of `real_index`,
/// before a recovery writes the index to finish an interrupted merge.
/// Why:
///  `git update-index` takes the lock and dies when another program holds it,
/// so without this wait one held lock made
///  every command in the worktree exit 2,
/// again and again
///  (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 2).
/// The bound and the notices are the merge's,
/// and only the outcome differs:
///  recovery writes nothing when the wait ends,
/// and leaves the record for the next command.
///  A lock a gone process left is cleared by recovery on the same proof the merge's own path acts on,
/// so the line ends
///  on the same promise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await waitForIndexReleaseWhileFinishingAMerge(realIndex, stderr)
/// ```
pub fn wait_for_index_release_while_finishing_a_merge<W: Write + ?Sized>(real_index: &Path, out: &mut W) -> IndexWait {
    return wait_with_watches_and_consequence(
        real_index,
        "merge",
        RECOVERY_CONSEQUENCE,
        RECOVERY_NOTICE_ENDING,
        CLEARS_A_DEAD_OWNERS_LOCK,
        recovery_ceiling(),
        Cadence::from_environment(),
        out,
        &Watch::on,
    );
}

/// What:
///  The ceiling the recovery's wait for the index lock ends at.
/// Why:
///  A control about a holder that keeps the lock has to reach the ceiling,
/// and waiting the shipping minute for every
///  such control would add a minute to every test run;
/// production reads `MERGE_LOCK_CEILING`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recoveryCeiling = () => SHORTENED?.get() ?? MERGE_LOCK_CEILING;
/// ```
fn recovery_ceiling() -> Duration {
    #[cfg(test)]
    if let Some(shortened) = RECOVERY_LOCK_CEILING_FOR_THIS_THREAD.with(std::cell::Cell::get) {
        return shortened;
    }
    return MERGE_LOCK_CEILING;
}

#[cfg(test)]
thread_local! {
    /// A shortened ceiling for the recovery's wait for the index lock,
    /// so a control reaches it in a moment.
    pub(crate) static RECOVERY_LOCK_CEILING_FOR_THIS_THREAD: std::cell::Cell<Option<Duration>> = const { std::cell::Cell::new(None) };
}

/// What:
///  The wait a merge makes before its own apply,
/// whose dead owner's line says the merge clears that lock,
/// because it does.
/// Why:
///  A merge that waited on a lock a gone process left now removes it on the commit service's proof of disuse and goes
///  on (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 3),
/// so the line must not promise that the lock was
///  left in place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await waitForIndexReleaseBeforeAMergeThatClearsADeadLock(realIndex, stderr)
/// ```
pub fn wait_for_index_release_before_a_merge_that_clears_a_dead_lock<W: Write + ?Sized>(real_index: &Path, out: &mut W) -> IndexWait {
    return wait_with_watches_and_consequence(
        real_index,
        "merge",
        MERGE_CONSEQUENCE,
        MERGE_NOTICE_ENDING,
        CLEARS_A_DEAD_OWNERS_LOCK,
        MERGE_LOCK_CEILING,
        Cadence::from_environment(),
        out,
        &Watch::on,
    );
}

/// Write `bytes` to `out`,
/// then flush it only if the write succeeded;
/// the callers ignore the result,
/// because a closed terminal
///  is not a reason to stop the command the person asked for.
fn write_then_flush<W: Write + ?Sized>(out: &mut W, bytes: &[u8]) -> std::io::Result<()> {
    out.write_all(bytes)?;
    return out.flush();
}

/// `wait_for_index_release` with the function that sets a watch on the lock's directory,
/// so a control can hand it none.
fn wait_with_watches(real_index: &Path, command: &str, ceiling: Duration, cadence: Cadence, out: &mut dyn Write, set_watch: &dyn Fn(&Path) -> Option<Watch>) -> IndexWait {
    let consequence: String = format!("git {command} stops waiting and reports what Git says");
    let notice_ending: String = format!("git {command} meets Git's own refusal");
    return wait_with_watches_and_consequence(real_index, command, consequence.as_str(), notice_ending.as_str(), DEAD_OWNER_LOCK_LEFT, ceiling, cadence, out, set_watch);
}

/// `wait_with_watches` with the sentence the ceiling line ends on given,
/// so a recovery of an interrupted merge can say what it does instead of reporting what Git says.
fn wait_with_watches_and_consequence<W: Write + ?Sized>(
    real_index: &Path,
    command: &str,
    consequence: &str,
    notice_ending: &str,
    dead_ending: &str,
    ceiling: Duration,
    cadence: Cadence,
    out: &mut W,
    set_watch: &dyn Fn(&Path) -> Option<Watch>,
) -> IndexWait {
    let lock = lock_path_of(real_index);
    // The watch comes before the first look: a release between the look and the wait is then seen as an event.
    let mut watch: Option<Watch> = lock.parent().and_then(set_watch);
    let lock_name: std::ffi::OsString = lock.file_name().map(std::ffi::OsStr::to_os_string).unwrap_or_default();
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
        if let Proof::Dead(pid) = &proof {
            let _noted = write_then_flush(out, dead_owner_diagnostic(lock.as_path(), *pid, consequence, dead_ending).as_bytes());
            return IndexWait::OwnerGone(elapsed);
        }
        if due {
            // A closed terminal is not a reason to stop the command the person asked for.
            let _noted = write_then_flush(out, notice(command, real_index, elapsed, ceiling, &proof, !announced, notice_ending).as_bytes());
            announced = true;
        }
        // The owner, 2026-11-03: "nothing should ever wait forever or indefinitely". At the ceiling the wait names the blocker and ends;
        //  the command that waited then decides, and a merge saves itself instead of failing (`MERGE_CONSEQUENCE`).
        if elapsed >= ceiling {
            let blocker: String = match &proof {
                Proof::Alive(evidence, ..) => blocker_words(evidence),
                Proof::Nothing(Some(evidence)) => blocker_words(evidence),
                Proof::Nothing(None) | Proof::Dead(_) => String::from("an owner whose evidence could not be read"),
            };
            let line: String = ceiling_line(lock.as_path(), blocker.as_str(), elapsed.as_secs(), consequence);
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
