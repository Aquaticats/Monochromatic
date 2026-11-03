//! What:
//!  The wait of a command that is about to change the checkout for any program's hold on Git's `index.lock`: the commit
//!  service's, a foreign `git`'s, or a lock nothing proves held. Event-driven, with a notice that names what is waited for
//!  and the holder, bounded by the wait limit, and over as soon as the lock is gone.
//! Why:
//!  Native Git never waits for `index.lock`: a merge that finds it held dies (`Unable to create ... index.lock` or, from the
//!  stash that every merge commit takes first, `could not write index` and `stash failed`). Other sessions commit into a
//!  shared checkout all the time, so a held lock is ordinary and passes by itself. This wait is the sibling of the wait for
//!  the commit service's own hold (`service_index_wait.rs`), which returns at once for a foreign holder because a plain
//!  forwarded writer keeps the behavior of native Git; a merge is held to the owner's rule that merges never fail
//!  (`SPEC.md`, "Merges and commits never fail"). The wait blocks on an inotify watch set before the first look, so a
//!  release between the look and the wait is not missed; where no watch can be set it looks again after a short pause.
//!  It never removes a lock: an unproven holder is waited for to the limit, as the commit service waits.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await waitForIndexRelease({ realIndex, command: 'merge', limit, cadence, stderr });
//! ```

/// Evidence about the holder of a lock.
use super::index_lock_evidence::{
    Verdict, classify_index_lock, gather_with, proven_holder_line, proven_holder_progress_line,
};
/// The bounded scan of open descriptors.
use super::index_lock_holders::scan_platform_holders_bounded;
/// The start time of a process.
use super::process_start::resolve_process_start;
/// The place of the lock and the watch on its directory.
use super::service_index_lock::{Watch, lock_path_of};
/// The limit and the pace of the notices.
use super::wait_limit::live_wait_limit;
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
    /// The lock was still held at the limit; nothing was changed and the lock was left in place.
    GaveUp(
        /// How long the wait lasted.
        Duration,
    ),
}

/// The words for what the evidence says about the holder, as the tail of a notice.
fn holder_tail(real_index: &Path, waited: Duration, limit: Duration, first: bool) -> String {
    let unproven = || {
        return format!(
            "waiting for {}, whose owner cannot be proven alive, after {} seconds; giving up after {} seconds",
            lock_path_of(real_index).display(),
            waited.as_secs(),
            limit.as_secs()
        );
    };
    let Ok(Some(evidence)) = gather_with(real_index, resolve_process_start, scan_platform_holders_bounded) else {
        return unproven();
    };
    let Verdict::ProvenAlive { pid, by_descriptor, command } = classify_index_lock(&evidence) else {
        return unproven();
    };
    let line: String = if first {
        proven_holder_line(pid, by_descriptor, command.as_deref(), evidence.lock_path.as_path(), limit.as_secs())
    } else {
        proven_holder_progress_line(pid, by_descriptor, command.as_deref(), evidence.lock_path.as_path(), waited.as_secs(), limit.as_secs())
    };
    return line.strip_prefix("cli-git: ").unwrap_or(line.as_str()).trim_end().to_owned();
}

/// The notice: what the command waits for, why it is safe to wait, then the holder.
fn notice(command: &str, real_index: &Path, waited: Duration, limit: Duration, first: bool) -> String {
    return format!(
        "cli-git: git {command} has changed nothing yet and waits for the index lock of this repository; {}\n",
        holder_tail(real_index, waited, limit, first)
    );
}

/// What:
///  Wait while any program holds `index.lock` of `real_index`, telling `out` what is waited for.
/// Why:
///   Returns after one metadata read when nothing holds the lock. The first line comes at once for a holder that is already
///  known when the wait begins and then every notice interval, so a queued command can be told from a hung one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await waitForIndexRelease(realIndex, 'merge', limit, cadence, stderr)
/// ```
pub fn wait_for_index_release(real_index: &Path, command: &str, limit: Duration, cadence: Cadence, out: &mut dyn Write) -> IndexWait {
    let lock = lock_path_of(real_index);
    // The watch comes before the first look: a release between the look and the wait is then seen as an event.
    let mut watch: Option<Watch> = lock.parent().and_then(Watch::on);
    let lock_name: std::ffi::OsString = lock.file_name().map(std::ffi::OsStr::to_os_string).unwrap_or_default();
    let began: Instant = Instant::now();
    let mut ticker: Ticker = Ticker::announced_at_start(cadence);
    let mut held: bool = false;
    let mut announced: bool = false;
    loop {
        if std::fs::symlink_metadata(&lock).is_err() {
            return if held { IndexWait::Released(began.elapsed()) } else { IndexWait::NotHeld };
        }
        held = true;
        let elapsed: Duration = began.elapsed();
        if elapsed >= limit {
            let _noted = out
                .write_all(
                    format!(
                        "cli-git: the index lock of this repository was still held after {} seconds, so git {command} stops waiting and reports what Git says; nothing was changed and the lock was left in place.\n",
                        elapsed.as_secs()
                    )
                    .as_bytes(),
                )
                .and_then(|()| { return out.flush(); });
            return IndexWait::GaveUp(elapsed);
        }
        // The holder is known when the wait begins, so the first line comes at once and the repeats follow the cadence.
        if !announced || ticker.due(elapsed) {
            // A closed terminal is not a reason to stop the command the person asked for.
            let _noted = out.write_all(notice(command, real_index, elapsed, limit, !announced).as_bytes()).and_then(|()| { return out.flush(); });
            announced = true;
        }
        let pause: Duration = ticker.next_in(elapsed).min(limit.saturating_sub(elapsed)).max(Duration::from_millis(1));
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

/// `wait_for_index_release` with the limit and the notice cadence this process runs under, on standard error.
pub fn wait_for_index_release_here(real_index: &Path, command: &str) -> IndexWait {
    return wait_for_index_release(real_index, command, live_wait_limit(), Cadence::from_environment(), &mut std::io::stderr());
}

/// The wait is controlled in the test module.
#[cfg(test)]
#[path = "index_holder_wait_tests.rs"]
mod tests;
