//! What:
//!  The attempts and the wait of an acquisition of `index.lock`:
//! placement tried again after each wake,
//! a dead service's lock reclaimed at once,
//! a holder proven alive and an owner nothing proves alive both waited for on the watch until the ceiling,
//! a lock that already outlasted one commit's wait refused at once to the commits queued behind it,
//! polled for where no watch can be set,
//! a dead holder reported at once,
//! and the notices that name the holder.
//! Why:
//!  The wait blocks on an inotify watch set before the first attempt,
//! so a release between an attempt and the wait is not missed;
//! the only timer is the repetition of the holder's line,
//! and a poll where no watch can be set.
//! The module is split from `service_index_lock.rs`,
//! which holds the acquisition's entry points and the lock itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await waitAndPlace(platform, placing, watch, announce, holder)
//! ```

use crate::index_lock_evidence::{
    Verdict, WaitEnd, classify_index_lock, gather_with, host_pid_process, proven_holder_line, proven_holder_progress_line, unproven_wait_line, ceiling_line, blocker_words,
    unproven_removal_advice, holder_way_out,
};
use crate::service_index_lock::{Announce, Announcer, LockFailure, notice_every};
use crate::service_index_found::{WaitNotes, describe_lock, open_by_text};
use crate::service_index_refused_lock::{forget, recall, remember};
use crate::service_index_placement::{Placed, Platform, Reclaimed, log_placement_failure, place_lock, reclaim};
use crate::service_index_reclaim::{foreign_words, stale_lock_words, unjudged_words};
use crate::service_index_staging::refresh_age;
use crate::service_index_unheld_removal::{Unheld, remove_if_unheld, told_words};
use crate::service_index_watch::{POLL_PAUSE, Watch};
use crate::service_protocol::FailureCode;
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

/// The line that says this commit did not wait because the same lock already stopped an earlier one.
fn earlier_commit_line(earlier_seconds: u64) -> String {
    return format!(
        "monochromatic-git-policy-cli: the same lock stopped an earlier commit after {earlier_seconds} seconds, so this commit did not wait for it again; once the holder is gone or the lock is cleared, run the commit again.\n"
    );
}

/// What the ceiling's line says becomes of the commit that waited for a held lock,
/// which is withdrawn rather than run into Git's refusal the way a merge is.
const COMMIT_NOT_MADE: &str = "this commit was not made";

/// What the notice of a wait for a held lock says becomes of the commit when the wait reaches its ceiling,
/// where the commit is withdrawn rather than run into Git's refusal the way a merge is:
/// the ceiling's own line (`ceiling_line`) gives the full account.
pub const COMMIT_WITHDRAWN: &str = "this commit is withdrawn and its change stays uncommitted";

/// What one look at a held lock proved.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Proved {
    /// A holder is proven alive;
    /// it is waited for until the ceiling.
    Alive,
    /// Nothing proves a holder alive or dead;
    /// it is waited for like a proven one,
    /// until the ceiling.
    Nothing,
    /// The lock was gone when the evidence was read;
    /// the next attempt will find out.
    Gone,
}

/// What a look at the held lock found:
/// what it proved,
/// the line for the person,
/// the holder's words,
/// the processes that
///  have the lock open,
/// and the diagnostic to give if the look ends the wait.
struct Looked {
    /// The words for the blocker,
    /// for the line at the ceiling.
    blocker: String,
    proved: Proved,
    line: Option<String>,
    who: Option<String>,
    open: Option<String>,
    /// For a holder proven alive:
    /// its number,
    /// program and start time,
    /// and what ends it if it is stuck.
    way_out: Option<String>,
}

/// What the evidence says about the holder:
/// what it proves,
/// its notice line for the person,
/// its words,
/// and the diagnostic for a look that ends the wait.
/// The scan is the platform's,
/// the same one the removal is decided on,
/// so one acquisition reads one host.
fn look_at_holder(platform: &Platform, index: &Path, waited: Duration, first: bool, ceiling: Duration) -> Looked {
    let gathered = gather_with(index, host_pid_process, platform.scan);
    let evidence = match gathered {
        Ok(Some(found)) => found,
        // The lock was released between the placement and this look.
        Ok(None) => return Looked { blocker: String::new(), proved: Proved::Gone, line: None, who: None, open: None, way_out: None },
        Err(_) => {
            let lock: std::path::PathBuf = crate::service_index_lock::lock_path_of(index);
            let end: WaitEnd<'_> = WaitEnd { within: ceiling, ending: COMMIT_WITHDRAWN };
            return Looked { blocker: String::from("an owner whose evidence could not be read"), proved: Proved::Nothing, line: Some(unproven_wait_line(lock.as_path(), None, end)), who: None, open: None, way_out: None };
        }
    };
    let end: WaitEnd<'_> = WaitEnd { within: ceiling, ending: COMMIT_WITHDRAWN };
    // The processes the scan saw with the lock open are the one source of `open_by`: the scan is made once, and bounded in time.
    let open: Option<String> = open_by_text(evidence.holders.as_ref());
    return match classify_index_lock(&evidence) {
        Verdict::ProvenAlive { pid, by_descriptor, command } => {
            let way_out: String = holder_way_out(pid, command.as_deref());
            let line: String = if first {
                proven_holder_line(pid, by_descriptor, command.as_deref(), evidence.lock_path.as_path(), end)
            } else {
                proven_holder_progress_line(pid, by_descriptor, command.as_deref(), evidence.lock_path.as_path(), waited.as_secs(), end)
            };
            Looked { blocker: blocker_words(&evidence), proved: Proved::Alive, line: Some(format!("{line}{way_out}")), who: Some(holder_words(pid, command.as_deref())), open, way_out: Some(way_out) }
        }
        // A PID file naming a number this process table does not have is no proof of a dead owner: a live Git of another
        // PID namespace writes a number this table has never had, and holds the lock through its editor with no descriptor
        // open, so the commit waits for it within its ceiling as it does for an owner nothing proves alive
        // (cross-vendor review of 2026-11-09, finding 2). Only a command that may clear the lock acts on that verdict.
        Verdict::Dead(_) | Verdict::EvidenceFree => Looked {
            blocker: blocker_words(&evidence),
            proved: Proved::Nothing,
            line: Some(unproven_wait_line(evidence.lock_path.as_path(), Some(&evidence), end)),
            who: None,
            open,
            way_out: None,
        },
    };
}

#[cfg(test)]
std::thread_local! {
    /// The waits `until_next_wake` chose on this thread,
    /// in the order they were chosen.
    static WAITS_CHOSEN: std::cell::RefCell<Vec<Duration>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// What:
///  The waits `until_next_wake` has chosen on the calling thread since `forget_waits_chosen`.
/// Why:
///   A control that says how soon a wait looks again cannot read the clock:
/// a thread the scheduler holds makes
///  a wait that asked for the poll pace look slow,
/// and one held just after a sleep makes a wait that asked for the notice
///  interval look quick.
/// The chosen span is the event.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function waitsChosenByThisThread(): number[];
/// ```
#[cfg(test)]
pub(crate) fn waits_chosen_by_this_thread() -> Vec<Duration> {
    return WAITS_CHOSEN.with(|chosen: &std::cell::RefCell<Vec<Duration>>| { return chosen.borrow().clone(); });
}

/// Forget the waits chosen on the calling thread so far;
/// nothing in the release executable.
#[cfg(test)]
pub(crate) fn forget_waits_chosen() {
    WAITS_CHOSEN.with(|chosen: &std::cell::RefCell<Vec<Duration>>| { chosen.borrow_mut().clear() });
}

/// Note for the controls the wait `wake` is about to be taken;
/// nothing in the release executable.
#[cfg(test)]
fn note_the_wait_chosen(wake: Duration) {
    WAITS_CHOSEN.with(|chosen: &std::cell::RefCell<Vec<Duration>>| { chosen.borrow_mut().push(wake) });
}

/// How long the wait blocks before it looks again:
/// the time to the next repetition of the holder's line,
/// and where no watch can be set no longer than the poll pause.
pub(crate) fn until_next_wake(to_notice: Duration, to_ceiling: Duration, watched: bool) -> Duration {
    let wake: Duration = if watched { to_notice.min(to_ceiling) } else { to_notice.min(POLL_PAUSE).min(to_ceiling) };
    let chosen: Duration = wake.max(Duration::from_millis(1));
    #[cfg(test)]
    note_the_wait_chosen(chosen);
    return chosen;
}

/// What one acquisition places and waits for.
#[derive(Clone, Copy)]
pub(crate) struct Placing<'a> {
    pub(crate) staged: &'a Path,
    pub(crate) held: &'a std::fs::File,
    /// The text of the marker,
    /// which the lock is given when it must be created before it can be written.
    pub(crate) marker: &'a str,
    pub(crate) lock: &'a Path,
    pub(crate) lock_name: &'a std::ffi::OsStr,
    pub(crate) index: &'a Path,
    /// How long a holder is waited for.
    pub(crate) ceiling: Duration,
}

/// The attempts and the wait of `acquire`,
/// with the staging file already written.
pub(crate) fn wait_and_place(platform: &Platform, placing: &Placing, watch: &mut Option<Watch>, announce: &dyn Announcer, notes: &mut WaitNotes) -> Result<Placed, LockFailure> {
    let Placing { staged, held, marker, lock, lock_name, index, ceiling } = *placing;
    let directory: &Path = lock.parent().unwrap_or(Path::new("."));
    let began: Instant = Instant::now();
    let mut first: bool = true;
    let mut proof: Proved = Proved::Nothing;
    let mut last_notice: Instant = began;
    // How long earlier commits waited for this very lock, when it already outlasted a wait.
    let mut already_waited: Option<Duration> = None;
    loop {
        // What the reclaim of this round found: a stale lock another reclaimer holds (looked at again at the poll pace, because
        //  the end of a hold leaves no event in the directory), and a stale lock that is not removed here (told to the person).
        let mut busy: bool = false;
        let mut left_stale: Option<(String, &str)> = None;
        // The hold begins at the placement, not at the staging that may have been long ago.
        refresh_age(held);
        match place_lock(platform, staged, marker, lock) {
            Ok(placed) => {
                forget();
                return Ok(placed);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                // A lock with the service's marker whose holder is gone is a dead instance's: it is removed, and the
                //  attempt is made again at once; so is a lock that was replaced while it was judged. A live holder of any
                //  release is waited for like a foreign one, and so is a stale lock another reclaimer is removing (the watch
                //  wakes when it is gone, and the notice timer looks again if that reclaimer died). A lock that cannot be
                //  removed ends the wait: more waiting would not change that.
                match reclaim(platform, lock) {
                    Reclaimed::Removed | Reclaimed::Changed => continue,
                    Reclaimed::Stuck => return Err(LockFailure { code: FailureCode::Unavailable }),
                    Reclaimed::Busy => busy = true,
                    Reclaimed::Unguarded(marked) => left_stale = Some((stale_lock_words(lock, &marked), "A service that is gone")),
                    Reclaimed::Foreign { file, marked } => left_stale = Some((foreign_words(file.as_path(), &marked), "A service on another machine")),
                    Reclaimed::Unjudged { file, marked } => left_stale = Some((unjudged_words(file.as_path(), marked.as_ref()), "A reclaimer that cannot be judged")),
                    Reclaimed::Kept => {
                        // A lock that nothing holds and no Git lock PID file stands behind is removed (owner, 2026-11-07, "Service removes it"):
                        //  the check is made at the first look and at each repeat of the notice, as the evidence is, not at every wake.
                        if first || last_notice.elapsed() >= notice_every() {
                            match remove_if_unheld(platform, index, &platform.scan) {
                                Unheld::Removed(removed) => {
                                    announce.announce(Announce::Waiting(told_words(&removed)));
                                    continue;
                                }
                                Unheld::Changed => continue,
                                Unheld::Kept => {}
                            }
                        }
                    }
                }
            }
            Err(error) => {
                log_placement_failure(lock, &error);
                return Err(LockFailure { code: crate::service_land_follow::storage_failure(&error) });
            }
        }
        // What the placement found held is written down now, after the fast path has been left: the first look with the
        //  processes that have the lock open, every later look without that scan.
        notes.looks += 1;
        // A lock that is the very file an earlier commit already waited a whole ceiling for is not waited for again: the commits queued
        //  behind one blocked landing would otherwise each wait their own ceiling, one after another, for the same file.
        let earlier: Option<Duration> = if notes.looks == 1 { recall(lock) } else { None };
        if earlier.is_some() {
            already_waited = earlier;
        }
        let waited: Duration = began.elapsed();
        // A wake for something else in the directory repeats nothing: the line comes at the first look and then every interval.
        let due: bool = first || last_notice.elapsed() >= notice_every();
        // The evidence is read at the first look and at each repeat of the line, not at every wake or poll.
        let at_ceiling: bool = already_waited.is_some() || waited >= ceiling;
        let mut looked: Looked = if due || at_ceiling { look_at_holder(platform, index, waited, first, ceiling) } else { Looked { blocker: String::new(), proved: proof, line: None, who: None, open: None, way_out: None } };
        if let Some((words, who)) = left_stale {
            // The notice is the person's way to the lock; the holder is named from the marker, not from a process scan.
            looked.line = Some(words);
            looked.who = Some(String::from(who));
        }
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
                announce.announce(Announce::Waiting(text));
            }
            first = false;
            last_notice = Instant::now();
        }
        // A holder proven alive and an owner nothing proves alive are both waited for until the ceiling, because a number
        //  the lock's PID file holds that this process table does not have may be a live Git's of another PID namespace
        //  (cross-vendor review of 2026-11-09, finding 2; owner, 2026-11-03: "nothing should ever wait forever or
        //  indefinitely"); then the wait ends, naming the blocker, and the batch is withdrawn by the caller.
        if at_ceiling && proof != Proved::Gone {
            let held_against_commits: Duration = waited + already_waited.unwrap_or(Duration::ZERO);
            remember(lock, held_against_commits);
            announce.announce(Announce::Waiting(ceiling_line(lock, looked.blocker.as_str(), held_against_commits.as_secs(), COMMIT_NOT_MADE)));
            if let Some(earlier_wait) = already_waited {
                announce.announce(Announce::Waiting(earlier_commit_line(earlier_wait.as_secs())));
            }
            if let Some(way_out) = looked.way_out {
                announce.announce(Announce::Waiting(way_out));
            }
            if proof == Proved::Nothing {
                announce.announce(Announce::Waiting(unproven_removal_advice(lock)));
            }
            announce.announce(Announce::Holder(match looked.who {
                Some(who) => who,
                None => String::from("Another program"),
            }));
            return Err(LockFailure { code: FailureCode::IndexInUse });
        }
        // A claim on the stale lock ends without an event on the lock's name, so the wait looks again at the poll pace while
        //  one is held, and says so for the landing's log and for the controls.
        let watched: bool = !busy;
        let next_wake: Duration = until_next_wake(notice_every().saturating_sub(last_notice.elapsed()), ceiling.saturating_sub(waited), watched);
        if !watched {
            notes.polled = true;
        }
        match watch.as_ref() {
            Some(active) => {
                if !active.wait(next_wake, lock_name) {
                    // The watch ended with its directory or by the directory being moved: the ended watch is dropped first, because a
                    //  new one may need the descriptor and the quota it holds, then the lock is looked at again with a new watch set.
                    drop(watch.take());
                    *watch = (platform.watch)(directory);
                }
            }
            // No watch could be set (the limit is reached, or the file system gives none): the lock is looked at again after a
            //  poll pause, and a watch is tried again each time.
            None => {
                std::thread::sleep(until_next_wake(notice_every().saturating_sub(last_notice.elapsed()), ceiling.saturating_sub(waited), false));
                *watch = (platform.watch)(directory);
            }
        }
    }
}
