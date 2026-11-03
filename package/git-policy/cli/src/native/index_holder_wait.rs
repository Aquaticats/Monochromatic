//! What:
//!  The wait of a command that is about to change the checkout for any program's hold on Git's `index.lock`: the commit
//!  service's, a foreign `git`'s, or a lock nothing proves held. Event-driven, with a notice that names what is waited for
//!  and the holder, and over as soon as the lock is gone. A holder proven alive is waited for with no bound; an owner that
//!  cannot be proven alive gets the short backoff of `unproven_backoff.rs`; a PID file naming a process that is gone ends the
//!  wait at once. The last two end with a diagnostic that lists the evidence.
//! Why:
//!  Native Git never waits for `index.lock`: a merge that finds it held dies (`Unable to create ... index.lock` or, from the
//!  stash that every merge commit takes first, `could not write index` and `stash failed`). Other sessions commit into a
//!  shared checkout all the time, so a held lock is ordinary and passes by itself. This wait is the sibling of the wait for
//!  the commit service's own hold (`service_index_wait.rs`), which returns at once for a foreign holder because a plain
//!  forwarded writer keeps the behavior of native Git; a merge is held to the owner's rule that merges never fail
//!  (`SPEC.md`, "Merges and commits never fail"). The wait blocks on an inotify watch set before the first look, so a
//!  release between the look and the wait is not missed; where no watch can be set it looks again after a short pause.
//!  It never removes a lock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await waitForIndexRelease({ realIndex, command: 'merge', limit, cadence, stderr });
//! ```

/// Evidence about the holder of a lock.
use super::index_lock_evidence::{
    Evidence, Verdict, classify_index_lock, dead_owner_diagnostic, gather_with, proven_holder_line, proven_holder_progress_line, unproven_diagnostic,
    unproven_wait_line, unreadable_diagnostic,
};
/// The bounded scan of open descriptors.
use super::index_lock_holders::scan_platform_holders_bounded;
/// The start time of a process.
use super::process_start::resolve_process_start;
/// The place of the lock and the watch on its directory.
use super::service_index_lock::{Watch, lock_path_of};
/// The backoff for an owner nothing proves alive, and the pace of the notices.
use super::unproven_backoff::UNPROVEN_OWNER_BACKOFF;
use super::wait_progress::{Cadence, Ticker};
/// Writing the notices.
use std::io::Write;
/// `Path` is a borrowed filesystem path.
use std::path::Path;
/// Time spans and the monotonic clock.
use std::time::{Duration, Instant};

/// How long the wait sleeps between looks where no watch can be set.
const POLL_PAUSE: Duration = Duration::from_millis(50);

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
    /// The lock was still there after the backoff and nothing proved a program held it; the diagnostic was written, nothing
    ///  was changed and the lock was left in place.
    Unproven(
        /// How long the wait lasted.
        Duration,
    ),
    /// The lock's PID file names a process that no longer runs, so nothing will release it; the diagnostic was written,
    ///  nothing was changed and the lock was left in place.
    OwnerGone(
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

/// The evidence of the host: the Git lock PID file and the bounded scan of open descriptors.
fn gather_here(real_index: &Path) -> std::io::Result<Option<Evidence>> {
    return gather_with(real_index, resolve_process_start, scan_platform_holders_bounded);
}

/// The notice: what the command waits for, why it is safe to wait, then the holder.
fn notice(command: &str, real_index: &Path, waited: Duration, proof: &Proof, backoff: Duration, first: bool) -> String {
    let tail: String = match proof {
        Proof::Alive(evidence, pid, by_descriptor, holder_command) => {
            let line: String = if first {
                proven_holder_line(*pid, *by_descriptor, holder_command.as_deref(), evidence.lock_path.as_path())
            } else {
                proven_holder_progress_line(*pid, *by_descriptor, holder_command.as_deref(), evidence.lock_path.as_path(), waited.as_secs())
            };
            line
        }
        Proof::Dead(_) | Proof::Nothing(_) => unproven_wait_line(lock_path_of(real_index).as_path(), backoff),
    };
    return format!(
        "cli-git: git {command} has changed nothing yet and waits for the index lock of this repository; {}",
        tail.strip_prefix("cli-git: ").unwrap_or(tail.as_str())
    );
}

/// What:
///  Wait while any program holds `index.lock` of `real_index`, telling `out` what is waited for.
/// Why:
///   Returns after one metadata read when nothing holds the lock. The first line comes at once for a holder that is already
///  known when the wait begins and then every notice interval, so a queued command can be told from a hung one. A holder
///  proven alive is waited for with no bound; anything else is waited for `backoff` and then reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await waitForIndexRelease(realIndex, 'merge', backoff, cadence, stderr)
/// ```
pub fn wait_for_index_release(real_index: &Path, command: &str, backoff: Duration, cadence: Cadence, out: &mut dyn Write) -> IndexWait {
    let lock = lock_path_of(real_index);
    // The watch comes before the first look: a release between the look and the wait is then seen as an event.
    let mut watch: Option<Watch> = lock.parent().and_then(Watch::on);
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
        // The evidence is read when the first line is due, when a repeat is due, and when the backoff of an owner nothing
        //  proves ends, so a scan of the open descriptors is not made for every wake of the watch.
        let backoff_over: bool = elapsed >= backoff && !matches!(proof, Proof::Alive(..));
        let due: bool = !announced || ticker.due(elapsed);
        if due || backoff_over {
            proof = look_at(real_index, &gather_here);
        }
        match &proof {
            Proof::Dead(pid) => {
                let _noted = out.write_all(dead_owner_diagnostic(lock.as_path(), *pid, consequence.as_str()).as_bytes()).and_then(|()| { return out.flush(); });
                return IndexWait::OwnerGone(elapsed);
            }
            Proof::Nothing(found) if elapsed >= backoff => {
                // The lock may have gone between the check above and the look: that is a release, not an unproven owner.
                if std::fs::symlink_metadata(&lock).is_err() {
                    return IndexWait::Released(began.elapsed());
                }
                let text: String = match found {
                    Some(evidence) => unproven_diagnostic(evidence, elapsed.as_secs(), consequence.as_str()),
                    None => unreadable_diagnostic(lock.as_path(), elapsed.as_secs(), consequence.as_str()),
                };
                let _noted = out.write_all(text.as_bytes()).and_then(|()| { return out.flush(); });
                return IndexWait::Unproven(elapsed);
            }
            Proof::Alive(..) | Proof::Nothing(_) => {}
        }
        if due {
            // A closed terminal is not a reason to stop the command the person asked for.
            let _noted = out.write_all(notice(command, real_index, elapsed, &proof, backoff, !announced).as_bytes()).and_then(|()| { return out.flush(); });
            announced = true;
        }
        // Only an owner nothing proves alive has a time at which the wait ends; a proven holder is waited for by the notices.
        let until_backoff: Duration = if matches!(proof, Proof::Alive(..)) { Duration::MAX } else { backoff.saturating_sub(elapsed) };
        let pause: Duration = ticker.next_in(elapsed).min(until_backoff).max(Duration::from_millis(1));
        match watch.as_ref() {
            Some(active) => {
                if !active.wait(pause, lock_name.as_os_str()) {
                    // The watch ended with its directory: the ended watch is dropped first, because a new one may need the
                    //  descriptor and the quota it holds, then the lock is looked at again with a new watch set.
                    drop(watch.take());
                    watch = lock.parent().and_then(Watch::on);
                }
            }
            None => std::thread::sleep(pause.min(POLL_PAUSE)),
        }
    }
}

/// `wait_for_index_release` with the backoff and the notice cadence this process runs under, on standard error.
pub fn wait_for_index_release_here(real_index: &Path, command: &str) -> IndexWait {
    return wait_for_index_release(real_index, command, UNPROVEN_OWNER_BACKOFF, Cadence::from_environment(), &mut std::io::stderr());
}

/// The wait is controlled in the test module.
#[cfg(test)]
#[path = "index_holder_wait_tests.rs"]
mod tests;
