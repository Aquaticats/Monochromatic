//! What:
//!  The wait of a wrapped command that writes the index for the commit service's own hold on the index:
//! event-driven,
//! with a
//!  notice that names what is waited for,
//! and over as soon as the hold is released.
//! Why:
//!  The service holds Git's `index.lock` while it lands a commit and marks it as its own.
//! A wrapped `git add` run by another
//!  process in the same worktree during that time must not fail,
//! and must not be told the hold belongs to a program
//!  nobody can prove alive:
//! the hold is the product's own and ends by itself.
//! A lock without the service's mark is a
//!  foreign program's and keeps the behavior of native Git.
//! The wait blocks on an inotify watch set before the first look,
//!  so a release between the look and the wait is not missed.
//! A service the mark proves to run is waited for until the lock goes or `SERVICE_HOLD_CEILING` passes
//!  (ten minutes,
//! `wait_ceilings.rs`);
//! at the ceiling the command runs and reports what Git says.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await waitForServiceHold({ realIndex, command: 'add', cadence, stderr });
//! ```

use crate::service_index_lock::{LOCK_MARKER, Watch, lock_path_of};
use crate::service_index_watch::POLL_PAUSE;
use crate::wait_ceilings::SERVICE_HOLD_CEILING;
use crate::service_index_marker::{holder_is_running, mark_names_a_gone_holder, marker_of};
use crate::wait_progress::{Cadence, Ticker};
use std::io::{Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};

/// How many times an empty lock is read again before it is taken for a foreign program's,
/// and the pause between the reads.
/// The service places its lock with the whole mark already in it
/// (the lock is the staged file,
/// renamed),
/// so an empty lock is never the service's.
/// The reads are for a lock of a release that wrote its mark after creating the lock,
/// and for a foreign program,
/// which leaves the file empty until it has an index to put in it.
const EMPTY_LOOKS: u32 = 5;
const EMPTY_PAUSE: Duration = Duration::from_millis(2);

/// The age after which a hold is taken for a leftover
/// when nothing else says whether its holder runs:
/// a mark that names no holder (an earlier release's),
/// or a holder this process cannot ask about
/// (another PID namespace,
/// a record it cannot read).
/// A mark that names a holder decides by the holder instead:
/// one that runs is waited for however old the file is,
/// and one that is gone is not waited for.
///
/// Measured on 2026-11-03 in the commit service's journal:
/// of 715 landings that logged the time the service held the lock
///  (`lock_hold_ms`),
/// 13 held it for 58 to 120 seconds,
/// all of it in the sync of the new index to disk
///  (then logged as `sync_ms`,
/// now `index_update_ms`;
/// longest 119,721 ms,
/// 99th percentile 65,787 ms,
/// median 34 ms).
/// A hold of that age is a live service on a slow
///  disk,
/// so the age is the longest measured hold with a quarter more,
/// 150 seconds;
/// the earlier 60 seconds took those live
///  holds for leftovers.
/// A wrong call only lets the forwarded command run into Git's own refusal,
/// which is run again while
///  the service has the index (`service_hold_retry.rs`).
pub const UNASKABLE_HOLD_AGE: Duration = Duration::from_secs(150);

/// Who holds the index.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Holder {
    /// Nobody:
    /// there is no lock.
    Nobody,
    /// The commit service,
    /// by its mark.
    Service,
    /// A program that is not the service,
    /// or a lock that cannot be read.
    Foreign,
}

/// What came of the wait.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Waited {
    /// The service did not hold the index when the command asked.
    NotHeld,
    /// The service held the index and let it go.
    Released,
    /// The service still held the index at the ceiling;
    /// the line saying that the command runs now was written.
    GaveUp,
    /// The hold is older than any measured landing lasts and nothing shows its holder runs,
    /// or names a holder that no longer runs,
    /// so it is a dead service's leftover and was not waited for.
    Stale,
}

/// What:
///  Sleeps for the pause between two reads of an empty lock file.
/// Why:
///  `holder_of` hands it to `holder_after_pauses`,
/// which takes the pause as a parameter so a control can replace it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pauseWhileEmpty = () => sleep(EMPTY_PAUSE);
/// ```
fn pause_while_empty() {
    std::thread::sleep(EMPTY_PAUSE);
}

/// What:
///  Writes a line to the person's terminal and flushes it when the write worked.
/// Why:
///  A closed terminal is not a reason to stop the command the person asked for,
/// so a failure is dropped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { out.write(line); out.flush(); } catch { /* terminal closed */ }
/// ```
fn note_line(out: &mut dyn Write, line: &str) {
    let written: std::io::Result<()> = out.write_all(line.as_bytes());
    if written.is_ok() {
        let _flushed = out.flush();
    }
}

/// Who holds the lock at `lock` now.
fn holder_of(lock: &Path) -> Holder {
    return holder_after_pauses(lock, &pause_while_empty);
}

/// `holder_of` with the pause between the reads of an empty lock given,
/// so a control can write the mark inside a pause instead of racing a timer against it.
///
/// The lock is opened by `open_plain_file`,
/// which opens without blocking and refuses everything that is not a plain file:
/// a named pipe this
///  look would wait on for ever is a foreign program's at once, as a lock of any other shape it cannot read is
/// (`doc/troubleshooting/a-fifo-where-a-file-is-expected-ends-no-command.md`).
fn holder_after_pauses(lock: &Path, pause: &dyn Fn()) -> Holder {
    for look in 0..EMPTY_LOOKS {
        let mut file = match crate::plain_file::open_plain_file(lock) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Holder::Nobody,
            Err(_) => return Holder::Foreign,
        };
        let mut head: [u8; LOCK_MARKER.len()] = [0; LOCK_MARKER.len()];
        let mut filled: usize = 0;
        while filled < head.len() {
            match file.read(&mut head[filled..]) {
                Ok(0) => break,
                Ok(count) => filled += count,
                Err(_) => return Holder::Foreign,
            }
        }
        if filled == head.len() {
            return if head == *LOCK_MARKER.as_bytes() { Holder::Service } else { Holder::Foreign };
        }
        if filled > 0 && !LOCK_MARKER.as_bytes().starts_with(&head[..filled]) {
            return Holder::Foreign;
        }
        if look + 1 < EMPTY_LOOKS {
            pause();
        }
    }
    return Holder::Foreign;
}

/// Whether a program other than the service holds the index of `real_index` now.
pub fn foreign_holds(real_index: &Path) -> bool {
    return holder_of(lock_path_of(real_index).as_path()) == Holder::Foreign;
}

/// The line for the person while the wait lasts:
/// what it waits for,
/// how long it has lasted,
/// and how long it lasts in all before the command runs.
fn waiting_line(command: &str, waited: Duration, ceiling: Duration) -> String {
    return format!(
        "monochromatic-git-policy-cli: git {command} is waiting for the commit service to finish a commit in this repository, {} seconds so far; this \
         waits for at most {} seconds in all, and then git {command} runs and reports what Git says.\n",
        waited.as_secs(),
        ceiling.as_secs()
    );
}

/// How long ago the lock at `lock` was made.
fn age_of(lock: &Path) -> Duration {
    // A lock whose time cannot be read, or is in the future, has no age.
    let Ok(metadata) = std::fs::metadata(lock) else {
        return Duration::ZERO;
    };
    let Ok(made) = metadata.modified() else {
        return Duration::ZERO;
    };
    return std::time::SystemTime::now().duration_since(made).unwrap_or(Duration::ZERO);
}

/// The line for the person when the hold is a leftover.
fn stale_line(command: &str, age: Duration) -> String {
    return format!(
        "monochromatic-git-policy-cli: the commit service's hold on this repository's index is {} seconds old and nothing shows the service still runs, so git {command} does not wait for it.\n",
        age.as_secs()
    );
}

/// The line for the person when the hold names a service process that is gone.
fn gone_line(command: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: the commit service that held this repository's index no longer runs, so git {command} does not wait for it. The next commit made through the service removes the lock it left.\n"
    );
}

/// The line for the person when the wait ran out.
fn gave_up_line(command: &str, waited: Duration) -> String {
    return format!(
        "monochromatic-git-policy-cli: the commit service was still finishing a commit in this repository after {} seconds, so git {command} runs now and reports what Git says.\n",
        waited.as_secs()
    );
}

/// What:
///  Waits while the commit service holds the index of `real_index`,
/// telling `out` what it waits for.
/// Why:
///   Returns at once when nobody holds the index or a foreign program does,
/// so a command pays one file read when there is
///  nothing to wait for.
/// The first line comes after the grace every wait notice has,
/// so a landing that ends at once prints
///  nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await waitForServiceHold(realIndex, 'add', cadence, stderr)
/// ```
pub fn wait_for_service_hold(real_index: &Path, command: &str, cadence: Cadence, out: &mut dyn Write) -> Waited {
    return wait_for_service_hold_within(real_index, command, SERVICE_HOLD_CEILING, cadence, out);
}

/// `wait_for_service_hold` that gives up after `ceiling`.
pub fn wait_for_service_hold_within(real_index: &Path, command: &str, ceiling: Duration, cadence: Cadence, out: &mut dyn Write) -> Waited {
    return wait_with_watches(real_index, command, ceiling, cadence, out, &Watch::on);
}

/// What:
///  `wait_for_service_hold` with the function that sets a watch on the lock's directory,
/// so a control can hand it a watch that is already over.
/// Why:
///   A watch ends when its directory is unmounted or removed,
/// and reports nothing afterwards;
/// the wait sets a new one then,
///  so a release after the end is seen as an event and not at the next timer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await waitWithWatches(realIndex, 'add', cadence, stderr, watchDirectory)
/// ```
fn wait_with_watches(real_index: &Path, command: &str, ceiling: Duration, cadence: Cadence, out: &mut dyn Write, set_watch: &dyn Fn(&Path) -> Option<Watch>) -> Waited {
    let lock = lock_path_of(real_index);
    // The watch comes before the first look: a release between the look and the wait is then seen as an event.
    let mut watch: Option<Watch> = lock.parent().and_then(set_watch);
    // Only the lock's own removal ends the wait; the service's staging files and Git's other files come and go meanwhile.
    let lock_name: std::ffi::OsString = lock.file_name().map(std::ffi::OsStr::to_os_string).unwrap_or_default();
    let began: Instant = Instant::now();
    let mut ticker: Ticker = Ticker::new(cadence);
    let mut held: bool = false;
    loop {
        if holder_of(&lock) != Holder::Service {
            return if held { Waited::Released } else { Waited::NotHeld };
        }
        // A holder the mark names and the system no longer has will never release the lock, however young the lock is.
        if mark_names_a_gone_holder(&lock) {
            note_line(out, gone_line(command).as_str());
            return Waited::Stale;
        }
        let age: Duration = age_of(&lock);
        // A service the marker proves to run is finishing a landing, however long ago its marker was written: the file's age
        //  is only a guide for a hold that proves nothing (a mark without a process, or a process this one cannot see).
        let running: bool = matches!(marker_of(&lock), Some(marked) if holder_is_running(marked.holder.as_ref()));
        if age >= UNASKABLE_HOLD_AGE && !running {
            note_line(out, stale_line(command, age).as_str());
            return Waited::Stale;
        }
        held = true;
        let elapsed: Duration = began.elapsed();
        // The owner, 2026-11-03: "nothing should ever wait forever or indefinitely". At the ceiling the command runs and reports
        //  what Git says.
        if elapsed >= ceiling {
            note_line(out, gave_up_line(command, elapsed).as_str());
            return Waited::GaveUp;
        }
        if ticker.due(elapsed) {
            // A closed terminal is not a reason to stop the command the person asked for.
            note_line(out, waiting_line(command, elapsed, ceiling).as_str());
        }
        let pause: Duration = ticker.next_in(elapsed).min(ceiling.saturating_sub(elapsed)).max(Duration::from_millis(1));
        match watch.as_ref() {
            Some(active) => {
                if !active.wait(pause, lock_name.as_os_str()) {
                    // The watch ended with its directory or by the directory being moved: the ended watch is dropped first, because
                    //  a new one may need the descriptor and the quota it holds, then the lock is looked at again with a new watch set.
                    drop(watch.take());
                    watch = lock.parent().and_then(set_watch);
                }
            }
            // No watch could be set: the lock is looked at again after a poll pause, and a watch is tried again each time.
            None => {
                std::thread::sleep(pause.min(POLL_PAUSE));
                watch = lock.parent().and_then(set_watch);
            }
        }
    }
}

/// `wait_for_service_hold` with the notice cadence this process runs under.
pub fn wait_for_service_hold_here(real_index: &Path, command: &str) -> Waited {
    return wait_for_service_hold(real_index, command, Cadence::from_environment(), &mut std::io::stderr());
}

/// The wait is controlled in the test module.
#[cfg(test)]
#[path = "service_index_wait_tests.rs"]
mod tests;
