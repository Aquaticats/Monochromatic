//! What:
//!  What one look at the held index lock proves,
//! and everything the person is told about it:
//! whether a holder is proven alive,
//! whether nothing proves one alive or dead,
//! whether the lock was gone,
//! the words for the process that holds it,
//! the line for a lock that already outlasted an earlier commit's wait,
//! and the one look that reads Git's evidence about the holder.
//! Why:
//!  The attempts and the wait (`service_index_attempts.rs`) read this once at the first look and at each
//! repetition of the holder's line,
//! so the evidence and its words sit in a file that waits for nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const looked = lookAtHolder(platform, index, waited, first, ceiling); // {proved, line, who, wayOut}
//! ```

use crate::index_lock_evidence::{
    Verdict,
    classify_index_lock,
    gather_with,
    holder_way_out,
    host_pid_process,
};
/// The sentences a holder look reports the evidence in,
/// and the two ends a line names.
use crate::index_lock_evidence_words::{
    WaitEnd,
    blocker_words,
    proven_holder_line,
    proven_holder_progress_line,
    unproven_wait_line,
};
use crate::service_index_found::open_by_text;
/// A name another program chose,
/// shown so no byte of it can act on the line it lands in,
/// and the parentheses that line puts around it.
use crate::shown_so_it_cannot_rewrite_the_line::{
    TheWrappingAroundTheName,
    shown_so_it_cannot_rewrite_the_line,
};
use std::path::Path;
use std::time::Duration;

/// What:
///  The holder a service's answer names,
///  from the evidence Git's lock leaves:
///  its number and the program it runs,
///  the program shown so no byte of it can act on the line.
/// Why:
///  The name is the one the program chose for itself (`/proc/<pid>/comm`),
///  and a person reads it to find what holds the lock:
///  an escape in it would move the cursor or erase the line,
///  and a `)` would end the parentheses that say whose name it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = (pid, command) => `Process ${pid} (${shownSoItCannotRewriteTheLine(command, TheWrappingAroundTheName.Parentheses)})`;
/// ```
pub(crate) fn holder_words(
    pid: i64,
    command: Option<&str>,
) -> String {
    return match command {
        Some(name) => format!(
            "Process {pid} ({})",
            shown_so_it_cannot_rewrite_the_line(name, TheWrappingAroundTheName::Parentheses)
        ),
        None => format!("Process {pid}"),
    };
}

/// The line that says this commit did not wait because the same lock already stopped an earlier one.
pub(crate) fn earlier_commit_line(earlier_seconds: u64) -> String {
    return format!(
        "monochromatic-git-policy-cli: the same lock stopped an earlier commit after {earlier_seconds} seconds, so this commit did not wait for it again; once the holder is gone or the lock is cleared, run the commit again.\n"
    );
}

/// What the ceiling's line says becomes of the commit that waited for a held lock,
/// which is withdrawn rather than run into Git's refusal the way a merge is.
pub(crate) const COMMIT_NOT_MADE: &str = "this commit was not made";

/// What the notice of a wait for a held lock says becomes of the commit when the wait reaches its ceiling,
/// where the commit is withdrawn rather than run into Git's refusal the way a merge is:
/// the ceiling's own line (`ceiling_line`) gives the full account.
pub const COMMIT_WITHDRAWN: &str = "this commit is withdrawn and its change stays uncommitted";

/// What one look at a held lock proved.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Proved {
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
pub(crate) struct Looked {
    /// The words for the blocker,
    /// for the line at the ceiling.
    pub(crate) blocker: String,
    pub(crate) proved: Proved,
    pub(crate) line: Option<String>,
    pub(crate) who: Option<String>,
    pub(crate) open: Option<String>,
    /// For a holder proven alive:
    /// its number,
    /// program and start time,
    /// and what ends it if it is stuck.
    pub(crate) way_out: Option<String>,
}

/// What the evidence says about the holder:
/// what it proves,
/// its notice line for the person,
/// its words,
/// and the diagnostic for a look that ends the wait.
/// The scan is the platform's,
/// the same one the removal is decided on,
/// so one acquisition reads one host;
/// it is given as the platform's own scan,
/// which keeps this file from naming the platform the caller holds.
pub(crate) fn look_at_holder(
    scan: fn(&Path, u64, u64) -> crate::index_lock_holders::HolderEvidence,
    index: &Path,
    lock: &Path,
    waited: Duration,
    first: bool,
    ceiling: Duration,
) -> Looked {
    let gathered = gather_with(index, host_pid_process, scan);
    let evidence = match gathered {
        Ok(Some(found)) => found,
        // The lock was released between the placement and this look.
        Ok(None) => {
            return Looked {
                blocker: String::new(),
                proved: Proved::Gone,
                line: None,
                who: None,
                open: None,
                way_out: None,
            };
        }
        Err(_) => {
            let end: WaitEnd<'_> = WaitEnd {
                within: ceiling,
                ending: COMMIT_WITHDRAWN,
            };
            return Looked {
                blocker: String::from("an owner whose evidence could not be read"),
                proved: Proved::Nothing,
                line: Some(unproven_wait_line(lock, None, end)),
                who: None,
                open: None,
                way_out: None,
            };
        }
    };
    let end: WaitEnd<'_> = WaitEnd {
        within: ceiling,
        ending: COMMIT_WITHDRAWN,
    };
    // The processes the scan saw with the lock open are the one source of `open_by`: the scan is made once, and bounded in time.
    let open: Option<String> = open_by_text(evidence.holders.as_ref());
    return match classify_index_lock(&evidence) {
        Verdict::ProvenAlive {
            pid,
            by_descriptor,
            command,
        } => {
            let way_out: String = holder_way_out(pid, command.as_deref());
            let line: String = if first {
                proven_holder_line(
                    pid,
                    by_descriptor,
                    command.as_deref(),
                    evidence.lock_path.as_path(),
                    end,
                )
            } else {
                proven_holder_progress_line(
                    pid,
                    by_descriptor,
                    command.as_deref(),
                    evidence.lock_path.as_path(),
                    waited.as_secs(),
                    end,
                )
            };
            Looked {
                blocker: blocker_words(&evidence),
                proved: Proved::Alive,
                line: Some(format!("{line}{way_out}")),
                who: Some(holder_words(pid, command.as_deref())),
                open,
                way_out: Some(way_out),
            }
        }
        // A PID file naming a number this process table does not have is no proof of a dead owner: a live Git of another
        // PID namespace writes a number this table has never had, and holds the lock through its editor with no descriptor
        // open, so the commit waits for it within its ceiling as it does for an owner nothing proves alive
        // (cross-vendor review of 2026-11-09, finding 2). Only a command that may clear the lock acts on that verdict.
        Verdict::Dead(_) | Verdict::EvidenceFree => Looked {
            blocker: blocker_words(&evidence),
            proved: Proved::Nothing,
            line: Some(unproven_wait_line(
                evidence.lock_path.as_path(),
                Some(&evidence),
                end,
            )),
            who: None,
            open,
            way_out: None,
        },
    };
}

/// Controls of the holder field stay out of the release executable.
#[cfg(test)]
#[path = "service_index_holder_look_tests.rs"]
mod tests;
