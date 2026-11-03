//! What:
//!  The attempts and the wait of an acquisition of `index.lock`:
//! placement tried again after each wake,
//! a dead service's lock reclaimed at once,
//! a holder proven alive waited for on the watch with no bound,
//! a holder nothing proves alive waited for the short backoff and then reported,
//! a dead holder reported at once,
//! and the notices that name the holder.
//! Why:
//!  The wait blocks on an inotify watch set before the first attempt,
//! so a release between an attempt and the wait is not missed;
//! the only timers are the backoff of an unproven holder and the repetition of the holder's line.
//! The module is split from `service_index_lock.rs`,
//! which holds the acquisition's entry points and the lock itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await waitAndPlace(platform, placing, watch, announce, holder)
//! ```

use super::index_lock_evidence::{
    Verdict, classify_index_lock, dead_owner_diagnostic, gather_with, proven_holder_line, proven_holder_progress_line, unproven_diagnostic, unproven_wait_line,
    unreadable_diagnostic,
};
use super::index_lock_holders::scan_platform_holders_bounded;
use super::process_start::resolve_process_start;
use super::service_index_lock::{Announce, LockFailure, notice_every};
use super::service_index_marker::is_reclaimable;
use super::service_index_found::{WaitNotes, describe_lock, open_by_text};
use super::service_index_placement::{Platform, log_placement_failure, place_marked, reclaim};
use super::service_index_staging::refresh_age;
use super::service_index_watch::Watch;
use super::service_protocol::FailureCode;
use std::path::Path;
use std::time::{Duration, Instant};

/// The words for the process holding the lock,
/// from the evidence Git's lock leaves.
fn holder_words(pid: i64, command: Option<&str>) -> String {
    return match command {
        Some(name) => format!("Process {pid} ({name})"),
        None => format!("Process {pid}"),
    };
}

/// What the diagnostics say becomes of the commit that was waiting.
const COMMIT_NOT_MADE: &str = "this commit was not made";

/// What one look at a held lock proved.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Proved {
    /// A holder is proven alive; it is waited for with no bound.
    Alive,
    /// The PID file names a process that is gone; nothing will release the lock.
    Dead,
    /// Nothing proves a holder alive or dead; it is waited for the backoff.
    Nothing,
    /// The lock was gone when the evidence was read; the next attempt will find out.
    Gone,
}

/// What a look at the held lock found: what it proved, the line for the person, the holder's words, the processes that
///  have the lock open, and the diagnostic to give if the look ends the wait.
struct Looked {
    proved: Proved,
    line: Option<String>,
    who: Option<String>,
    open: Option<String>,
    ending: Option<String>,
}

/// What the evidence says about the holder:
/// what it proves,
/// its notice line for the person,
/// its words,
/// and the diagnostic for a look that ends the wait.
fn look_at_holder(index: &Path, waited: Duration, backoff: Duration, first: bool) -> Looked {
    let gathered = gather_with(index, resolve_process_start, scan_platform_holders_bounded);
    let evidence = match gathered {
        Ok(Some(found)) => found,
        // The lock was released between the placement and this look.
        Ok(None) => return Looked { proved: Proved::Gone, line: None, who: None, open: None, ending: None },
        Err(_) => {
            let lock: std::path::PathBuf = super::service_index_lock::lock_path_of(index);
            let ending: String = unreadable_diagnostic(lock.as_path(), waited.as_secs(), COMMIT_NOT_MADE);
            return Looked { proved: Proved::Nothing, line: Some(unproven_wait_line(lock.as_path(), backoff)), who: None, open: None, ending: Some(ending) };
        }
    };
    // The processes the scan saw with the lock open are the one source of `open_by`: the scan is made once, and bounded in time.
    let open: Option<String> = open_by_text(evidence.holders.as_ref());
    return match classify_index_lock(&evidence) {
        Verdict::ProvenAlive { pid, by_descriptor, command } => {
            let line: String = if first {
                proven_holder_line(pid, by_descriptor, command.as_deref(), evidence.lock_path.as_path())
            } else {
                proven_holder_progress_line(pid, by_descriptor, command.as_deref(), evidence.lock_path.as_path(), waited.as_secs())
            };
            Looked { proved: Proved::Alive, line: Some(line), who: Some(holder_words(pid, command.as_deref())), open, ending: None }
        }
        Verdict::Dead(pid) => Looked {
            proved: Proved::Dead,
            line: None,
            who: Some(holder_words(pid, None)),
            open,
            ending: Some(dead_owner_diagnostic(evidence.lock_path.as_path(), pid, COMMIT_NOT_MADE)),
        },
        Verdict::EvidenceFree => Looked {
            proved: Proved::Nothing,
            line: Some(unproven_wait_line(evidence.lock_path.as_path(), backoff)),
            who: None,
            open,
            ending: Some(unproven_diagnostic(&evidence, waited.as_secs(), COMMIT_NOT_MADE)),
        },
    };
}

/// How long the wait blocks before it looks again:
/// the time to the next repetition of the holder's line,
/// and for a holder nothing proves alive no longer than the backoff has left.
fn until_next_wake(to_notice: Duration, to_backoff: Duration, proof: Proved) -> Duration {
    let wake: Duration = if proof == Proved::Alive { to_notice } else { to_notice.min(to_backoff) };
    return wake.max(Duration::from_millis(1));
}

/// What one acquisition places and waits for.
#[derive(Clone, Copy)]
pub(super) struct Placing<'a> {
    pub(super) staged: &'a Path,
    pub(super) held: &'a std::fs::File,
    pub(super) lock: &'a Path,
    pub(super) lock_name: &'a std::ffi::OsStr,
    pub(super) index: &'a Path,
    /// How long a holder nothing proves alive is waited for; a holder proven alive is waited for with no bound.
    pub(super) backoff: Duration,
}

/// The attempts and the wait of `acquire`,
/// with the staging file already written.
pub(super) fn wait_and_place(platform: &Platform, placing: &Placing, watch: &mut Option<Watch>, announce: &dyn Fn(Announce), notes: &mut WaitNotes) -> Result<(), LockFailure> {
    let Placing { staged, held, lock, lock_name, index, backoff } = *placing;
    let directory: &Path = lock.parent().unwrap_or(Path::new("."));
    let began: Instant = Instant::now();
    let mut first: bool = true;
    let mut proof: Proved = Proved::Nothing;
    let mut last_notice: Instant = began;
    loop {
        // The hold begins at the placement, not at the staging that may have been long ago.
        refresh_age(held);
        match place_marked(platform, staged, lock) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                // A lock with the service's marker whose holder is gone is a dead instance's: it is removed, and the
                //  attempt is made again at once. A live holder of any release is waited for like a foreign one. A lock
                //  that cannot be removed ends the wait: more waiting would not change that.
                if is_reclaimable(lock) {
                    if !reclaim(platform, lock) {
                        return Err(LockFailure { code: FailureCode::Unavailable });
                    }
                    continue;
                }
            }
            Err(error) => {
                log_placement_failure(lock, &error);
                return Err(LockFailure { code: super::service_land_follow::storage_failure(&error) });
            }
        }
        // What the placement found held is written down now, after the fast path has been left: the first look with the
        //  processes that have the lock open, every later look without that scan.
        notes.looks += 1;
        let waited: Duration = began.elapsed();
        // A wake for something else in the directory repeats nothing: the line comes at the first look and then every interval.
        let due: bool = first || last_notice.elapsed() >= notice_every();
        // Where no watch can be set there is nothing to block on, so the wait counts as over at once, as it always did.
        let no_watch: bool = watch.is_none();
        let backoff_over: bool = (waited >= backoff && proof != Proved::Alive) || no_watch;
        let looked: Looked = if due || backoff_over { look_at_holder(index, waited, backoff, first) } else { Looked { proved: proof, line: None, who: None, open: None, ending: None } };
        proof = looked.proved;
        let mut seen: String = describe_lock(lock);
        if let Some(text) = looked.open {
            seen.push(' ');
            seen.push_str(text.as_str());
        }
        if notes.found.is_none() {
            notes.found = Some(seen.clone());
        }
        notes.last = Some(seen);
        if looked.who.is_some() {
            notes.holder.clone_from(&looked.who);
        }
        if due {
            if let Some(text) = looked.line {
                announce(Announce::Waiting(text));
            }
            first = false;
            last_notice = Instant::now();
        }
        // Nothing the lock's holder does is waited for beyond what the look proved: a dead owner at once, an unproven one
        //  after the backoff, a proven one never.
        let ends: bool = match proof {
            Proved::Dead => true,
            Proved::Nothing => backoff_over,
            Proved::Alive => no_watch,
            Proved::Gone => false,
        };
        if ends {
            if let Some(text) = looked.ending {
                announce(Announce::Waiting(text));
            }
            announce(Announce::Holder(looked.who.unwrap_or_else(|| { return String::from("Another program"); })));
            return Err(LockFailure { code: FailureCode::IndexInUse });
        }
        if let Some(active) = watch.as_ref()
            && !active.wait(until_next_wake(notice_every().saturating_sub(last_notice.elapsed()), backoff.saturating_sub(waited), proof), lock_name)
        {
            // The watch ended with its directory or by the directory being moved: the ended watch is dropped first, because a
            //  new one may need the descriptor and the quota it holds, then the lock is looked at again with a new watch set.
            drop(watch.take());
            *watch = (platform.watch)(directory);
        }
    }
}
