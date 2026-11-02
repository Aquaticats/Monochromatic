//! What:
//!  The attempts and the wait of an acquisition of `index.lock`:
//! placement tried again after each wake,
//! a dead service's lock reclaimed at once,
//! a live or foreign holder waited for on the watch,
//! and the notices that name the holder.
//! Why:
//!  The wait blocks on an inotify watch set before the first attempt,
//! so a release between an attempt and the wait is not missed;
//! the only timers are the limit and the repetition of the holder's line.
//! The module is split from `service_index_lock.rs`,
//! which holds the acquisition's entry points and the lock itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await waitAndPlace(platform, placing, watch, announce, holder)
//! ```

use super::index_lock_evidence::{Verdict, classify_index_lock, gather_index_lock_evidence, proven_holder_line, proven_holder_progress_line};
use super::service_index_lock::{Announce, LockFailure, notice_every};
use super::service_index_marker::is_reclaimable;
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

/// What the evidence says about the holder:
/// its notice line for the person and its words,
/// when a live holder is proven.
fn holder_notice(index: &Path, waited: Duration, limit: Duration, first: bool) -> (Option<String>, Option<String>) {
    let Ok(Some(evidence)) = gather_index_lock_evidence(index) else { return (None, None) };
    let Verdict::ProvenAlive { pid, by_descriptor, command } = classify_index_lock(&evidence) else {
        let line: String = format!(
            "cli-git: waiting for {}, whose owner cannot be proven alive, after {} seconds; giving up after {} seconds.\n",
            evidence.lock_path.display(),
            waited.as_secs(),
            limit.as_secs()
        );
        return (Some(line), None);
    };
    let line: String = if first {
        proven_holder_line(pid, by_descriptor, command.as_deref(), evidence.lock_path.as_path(), limit.as_secs())
    } else {
        proven_holder_progress_line(pid, by_descriptor, command.as_deref(), evidence.lock_path.as_path(), waited.as_secs(), limit.as_secs())
    };
    return (Some(line), Some(holder_words(pid, command.as_deref())));
}

/// What one acquisition places and waits for.
#[derive(Clone, Copy)]
pub(super) struct Placing<'a> {
    pub(super) staged: &'a Path,
    pub(super) held: &'a std::fs::File,
    pub(super) lock: &'a Path,
    pub(super) lock_name: &'a std::ffi::OsStr,
    pub(super) index: &'a Path,
    pub(super) limit: Duration,
}

/// The attempts and the wait of `acquire`,
/// with the staging file already written.
pub(super) fn wait_and_place(platform: &Platform, placing: &Placing, watch: &mut Option<Watch>, announce: &dyn Fn(Announce), holder: &mut Option<String>) -> Result<(), LockFailure> {
    let Placing { staged, held, lock, lock_name, index, limit } = *placing;
    let directory: &Path = lock.parent().unwrap_or(Path::new("."));
    let began: Instant = Instant::now();
    let mut first: bool = true;
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
        let waited: Duration = began.elapsed();
        let exhausted: bool = waited >= limit || watch.is_none();
        // A wake for something else in the directory repeats nothing: the line comes at the first look and then every interval.
        let due: bool = first || last_notice.elapsed() >= notice_every();
        let (line, who) = if due || exhausted { holder_notice(index, waited, limit, first) } else { (None, None) };
        if who.is_some() {
            holder.clone_from(&who);
        }
        if due {
            if let Some(text) = line {
                announce(Announce::Waiting(text));
            }
            first = false;
            last_notice = Instant::now();
        }
        if exhausted {
            announce(Announce::Holder(who.unwrap_or_else(|| { return String::from("Another program"); })));
            return Err(LockFailure { code: FailureCode::IndexInUse });
        }
        if let Some(active) = watch.as_ref()
            && !active.wait(notice_every().saturating_sub(last_notice.elapsed()).max(Duration::from_millis(1)).min(limit.saturating_sub(waited)), lock_name)
        {
            // The watch ended with its directory or by the directory being moved: the ended watch is dropped first, because a
            //  new one may need the descriptor and the quota it holds, then the lock is looked at again with a new watch set.
            drop(watch.take());
            *watch = (platform.watch)(directory);
        }
    }
}
