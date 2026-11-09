//! What:
//!  The sentences a lock's evidence is reported in:
//!  the line that names a holder proven alive,
//!  the line that repeats while it keeps the lock,
//!  the line for an owner nothing proves alive,
//!  the line at a wait's ceiling,
//!  and the diagnostic for a PID number this process table does not have.
//! Why:
//!  Each sentence is built from the evidence and the end of the wait alone,
//!  so one file holds the words a person reads about a lock,
//!  and the file that reads and gathers the evidence stays within its line budget.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // index_lock_evidence_words.ts: one exported sentence per function below.
//! ```

/// The evidence a verdict rests on and the verdict itself.
use crate::index_lock_evidence::{Evidence, PidFile, Verdict, classify_index_lock};
/// The summary of what a scan of open descriptors could not read.
use crate::index_lock_holder_words::partial_summary;
/// `Path` is a borrowed filesystem path.
use std::path::Path;
/// How long a span lasts.
use std::time::Duration;

/// What:
///  `PID <n>` with the command in parentheses when known.
/// Why:
///   Shared by the evidence and the waiting line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `PID ${pid}${command === undefined ? '' : ` (${command})`}`
/// ```
fn holder_name(pid: i64, command: Option<&str>) -> String {
    match command {
        Some(name) => return format!("PID {pid} ({name})"),
        None => return format!("PID {pid}"),
    }
}

/// What:
///  The end of a wait,
/// as a notice line about it says it:
/// the longest the wait lasts,
/// and what becomes of the command when that time is up.
/// Why:
///   The owner's rule is that nothing waits for ever (2026-11-03:
/// "nothing should ever wait forever or
///  indefinitely"),
/// so a line that describes a wait says when the wait ends and what happens then;
/// the ceiling comes from the constant
///  the wait itself runs on,
/// so the line and the wait cannot drift apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface WaitEnd { within: Duration; ending: string }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct WaitEnd<'a> {
    /// The longest the wait lasts.
    pub within: Duration,
    /// What becomes of the command when that time is up.
    pub ending: &'a str,
}

/// What:
///  The one standard-error line naming a proven-alive holder.
/// Why:
///   `provenHolderLine`:
///  the person learns what the wait is for and which process holds the lock;
/// the line names the end of the wait
///  as well,
/// because a holder proven alive is waited for until `end.within` and not for as long as it holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function provenHolderLine({ verdict, lockPath, end }): string;
/// ```
pub fn proven_holder_line(pid: i64, by_descriptor: bool, command: Option<&str>, lock_path: &Path, end: WaitEnd<'_>) -> String {
    let proof: &str = if by_descriptor { "an open descriptor" } else { "its Git lock PID file" };
    return format!(
        "monochromatic-git-policy-cli: waiting for {}, which holds {} (proven by {proof}); this waits until that process lets go, for at most {} seconds; \
         after that, {}, and the lock is left in place.\n",
        holder_name(pid, command),
        lock_path.display(),
        end.within.as_secs(),
        end.ending
    );
}

/// What:
///  The line that repeats while a proven-alive holder keeps the lock.
/// Why:
///   Each repeat names the holder,
/// the proof and how long the wait has lasted,
/// so a reader of the last line alone knows what
///  is going on,
/// and the repeat names the end of the wait too,
/// so the last line alone also says how long the wait can last and what happens then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function provenHolderProgressLine({ verdict, lockPath, waitedSeconds, end }): string;
/// ```
pub fn proven_holder_progress_line(pid: i64, by_descriptor: bool, command: Option<&str>, lock_path: &Path, waited_seconds: u64, end: WaitEnd<'_>) -> String {
    let proof: &str = if by_descriptor { "an open descriptor" } else { "its Git lock PID file" };
    return format!(
        "monochromatic-git-policy-cli: still waiting for {}, which holds {} (proven by {proof}) after {waited_seconds} seconds; this waits until that process \
         lets go, for at most {} seconds in all; after that, {}, and the lock is left in place.\n",
        holder_name(pid, command),
        lock_path.display(),
        end.within.as_secs(),
        end.ending
    );
}

/// What the evidence says about the lock's owner,
/// in words that fit after "found".
fn evidence_words(evidence: &Evidence) -> String {
    let descriptors: String = match &evidence.holders {
        Some(scan) if !scan.unresolvable.is_empty() => format!(
            "a descriptor that names the lock could not be read ({})",
            partial_summary(scan.unresolvable.as_slice())
        ),
        Some(scan) if scan.partial.is_empty() => String::from("no process has the lock open"),
        Some(scan) => format!("no process the scan could see has the lock open (the scan could not see everything: {})", partial_summary(scan.partial.as_slice())),
        None => String::from("the open descriptors were not scanned"),
    };
    let pid_file: String = match &evidence.pid_file {
        PidFile::Absent => String::from("no Git lock PID file beside it (`core.lockfilePid` is off for the program that made it, or it is not Git)"),
        PidFile::Malformed(text) => format!("a Git lock PID file whose text is not `pid <number>` ({text:?})"),
        PidFile::Unreadable(reason) => format!("a Git lock PID file that could not be read ({reason})"),
        PidFile::Owner(pid, process) if process.gone() => format!(
            "a Git lock PID file naming PID {pid}, which this process table does not have (a Git of another PID namespace writes a number this table has never had, \
             and it holds the lock through its editor with no descriptor open)"
        ),
        PidFile::Owner(pid, process) if process.starts_after(&evidence.lock) => format!(
            "a Git lock PID file naming PID {pid} ({}), which runs, and which this host's clock puts after the lock was made, which a reused number would explain and so would a clock that moved",
            process.command.as_deref().unwrap_or("command name unknown")
        ),
        PidFile::Owner(pid, process) => format!("a Git lock PID file naming PID {pid} ({}), which runs", process.command.as_deref().unwrap_or("command name unknown")),
    };
    return format!("{descriptors}, and {pid_file}");
}

/// What:
///  The line that says a lock's owner cannot be proven alive,
/// what is known about it,
/// how long the wait lasts,
/// what becomes of the command then,
/// and what a person can do.
/// Why:
///   An owner nothing proves alive is waited for like one proven alive,
/// until the wait's ceiling (`MERGE_LOCK_CEILING` in `index_holder_wait.rs`,
/// `LANDING_LOCK_CEILING` in `service_index_attempts.rs`),
/// because the person's own `git commit`
///  with its editor open holds the lock exactly so;
/// the line says what was looked at and names the file to remove if no program
///  is using the repository,
/// which is the only case the wait cannot end by itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unprovenWaitLine({ lockPath, evidence, end }): string;
/// ```
pub fn unproven_wait_line(lock_path: &Path, evidence: Option<&Evidence>, end: WaitEnd<'_>) -> String {
    let known: String = match evidence {
        Some(found) => format!(" ({})", evidence_words(found)),
        None => String::from(" (the evidence about its owner could not be read)"),
    };
    return format!(
        "monochromatic-git-policy-cli: waiting for {lock}, whose owner cannot be proven alive{known}; this waits until the lock goes, for at most {} seconds; \
         after that, {}. If no program is using this repository, the lock was left by a program that was killed, and removing the file {lock} \
         lets the command run.\n",
        end.within.as_secs(),
        end.ending,
        lock = lock_path.display()
    );
}

/// What:
///  The line that names who still holds a lock when the wait for it reaches its ceiling,
/// and what becomes of the command.
/// Why:
///   The owner's rule is that nothing waits for ever:
/// at the bound the command stops and says what blocked it,
/// with the evidence
///  there is,
/// and what to run again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ceilingLine({ lockPath, blocker, waitedSeconds, consequence }): string;
/// ```
pub fn ceiling_line(lock_path: &Path, blocker: &str, waited_seconds: u64, consequence: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: {lock} is still held after {waited_seconds} seconds by {blocker}, so {consequence}; nothing was changed and the lock was \
         left in place. Run the command again once that program is done.\n",
        lock = lock_path.display()
    );
}

/// What:
///  The sentence that names the lock file and the fix when a wait ended at its ceiling on an owner nothing proves alive.
/// Why:
///   A lock without its marker is what plain Git leaves when it is killed holding it,
/// and what a kill of the service leaves on a
///  filesystem that places no file whole;
/// nothing here removes it,
/// because it may be a program that is still at work,
/// so the
///  person who knows no program is using the repository needs the file's name and the command to clear it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unprovenRemovalAdvice(lockPath: string): string;
/// ```
pub fn unproven_removal_advice(lock_path: &Path) -> String {
    return format!(
        "monochromatic-git-policy-cli: nothing proves a program is still using {lock}. If no program is using this repository, the lock was left by a program that was killed, \
         and removing the file {lock} lets the command run.\n",
        lock = lock_path.display()
    );
}

/// What:
///  The words for the owner of a lock that is held,
/// from the evidence:
/// a process proven alive,
/// or an owner nothing proves alive
///  with what was looked at.
/// Why:
///   The line at the ceiling names the blocker,
/// whatever the proof.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function blockerWords(evidence, verdict): string;
/// ```
pub fn blocker_words(evidence: &Evidence) -> String {
    return match classify_index_lock(evidence) {
        Verdict::ProvenAlive { pid, by_descriptor, command } => {
            let proof: &str = if by_descriptor { "an open descriptor" } else { "its Git lock PID file" };
            format!("{} (proven by {proof})", holder_name(pid, command.as_deref()))
        }
        Verdict::Dead(pid) => format!("{} (a number this process table does not have)", holder_name(pid, None)),
        Verdict::EvidenceFree => format!("an owner nothing proves alive ({})", evidence_words(evidence)),
    };
}

/// The sentence that ends a dead owner's line when the lock is left where it is,
/// which is what every command but a merge does.
pub const DEAD_OWNER_LOCK_LEFT: &str = "nothing was changed and the lock was left in place. If no other program is using this repository, removing the file lets the command run";

/// What:
///  The diagnostic for a lock whose Git lock PID file names a number this process table does not have,
///  ending in what the command does about it.
/// Why:
///   That owner can never let go,
/// so there is nothing to wait for;
/// a commit service and a merge each say what they do
///  instead,
/// and a removal that only some commands attempt must not be promised by the others.
///  The number is said for what it is:
///  a Git of another PID namespace writes one this table has never had,
///  so the diagnostic claims no proof that the owner is gone
///  (cross-vendor review of 2026-11-09,
/// finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deadOwnerDiagnostic({ lockPath, pid, consequence, ending }): string;
/// ```
pub fn dead_owner_diagnostic(lock_path: &Path, pid: i64, consequence: &str, ending: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: {lock} holds the PID number {pid}, which this process table does not have (a Git of another PID namespace writes a number this table \
         has never had), so no process here will release it. {consequence}; {ending}.\n",
        lock = lock_path.display()
    );
}
