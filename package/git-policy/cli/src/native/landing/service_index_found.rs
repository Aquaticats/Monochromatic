//! What:
//!  What an acquisition of the index lock found when it had to wait:
//! the marker on the lock,
//! how old the lock is,
//! whether the holder it names is gone,
//! running or unknown,
//! and which processes have the lock open;
//! the notes an acquisition keeps for the service's log.
//! Why:
//!  A commit that waited five seconds for the index lock left a log that said how long and not for whom.
//! The description is built only after a placement found the lock held,
//! so the fast path of an acquisition pays nothing for it;
//! the scan of `/proc/*/fd` for the processes that have the lock open is made once,
//! at the first look.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const found = describeLock(lockPath, true); // 'service seq=7 pid=42 state=running age_ms=5000 open_by=[42 (monochromatic-git-policy-cli)]'
//! ```

use crate::index_lock_holders::{Holder, HolderEvidence};
use crate::service_index_marker::{HolderState, holder_state, marker_of};
use std::path::Path;
use std::time::{Duration, SystemTime};

/// What an acquisition noted about its wait.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WaitNotes {
    /// The words for the holder the evidence named while the lock was waited for.
    pub holder: Option<String>,
    /// What the first look at the held lock found.
    pub found: Option<String>,
    /// What the last look found,
    /// which is the holder that let go when the wait ended in a success.
    pub last: Option<String>,
    /// How many times the placement found the lock held.
    pub looks: usize,
    /// Time the acquisition spent before its first placement getting a marked file that is flushed to storage:
    /// the whole creation and flush when it staged its own,
    /// the wait for the flush when the file was made ahead and was still being flushed,
    /// and nothing to speak of when it was ready.
    pub staging: Duration,
    /// How the marked file came about:
    /// `pre` when it was made ahead while the service was idle,
    /// `cold` when the acquisition made it.
    pub stage_kind: &'static str,
    /// Time the acquisition spent readying itself before its first placement,
    /// outside the making of the marked file:
    /// the watch on the lock's directory,
    /// and the text the marker names its process with.
    /// Why this is its own figure:
    ///  a wait for the index lock that takes far longer than its own accounting explains is a wait whose time went
    ///  somewhere else,
    /// and these calls are two of the three places
    /// (gate of 2026-11-11T10:29-05:00,
    /// `lock_wait_ms=95277` against `lock_stage_ms=0` and `lock_place_ms=3003`,
    /// which left 92 s of a landing's wait unattributed).
    pub setup: Duration,
    /// Time from the first placement to the lock being taken or given up:
    /// the wait for a holder.
    pub placing: Duration,
    /// Time the acquisition spent once its wait was decided:
    /// removing the marked file it had staged,
    /// which is the third place a wait's time can go without the other figures naming it.
    pub after: Duration,
    /// Whether the wait looked again at the poll pace while another reclaimer held a claim on the stale lock,
    /// because the end of a claim leaves no event on the lock's name to be woken by.
    pub polled: bool,
}

/// The word for what a reader makes of the holder a marker names.
fn state_word(state: HolderState) -> &'static str {
    return match state {
        HolderState::Gone => "gone",
        HolderState::Running => "running",
        HolderState::Unknown => "unknown",
    };
}

/// The processes that a scan saw with the lock open,
/// as `open_by=[pid (comm), ...]`,
/// or nothing when none was seen.
/// Why:
///   The name of a process is its `comm` (at most fifteen bytes,
/// the program's name),
/// never its arguments or its environment,
/// which can hold a token;
/// the scan reads nothing else,
/// and the description is quoted when it is written into a line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// openByText(holders) // 'open_by=[42 (git)]'
/// ```
pub fn open_by_text(seen: Option<&HolderEvidence>) -> Option<String> {
    let holders: &Vec<Holder> = &seen?.holders;
    if holders.is_empty() {
        return None;
    }
    let mut names: Vec<String> = Vec::new();
    for holder in holders.iter() {
        names.push(match holder.command.as_ref() {
            Some(command) => format!("{} ({command})", holder.pid),
            None => holder.pid.to_string(),
        });
    }
    return Some(format!("open_by=[{}]", names.join(", ")));
}

/// What:
///  How many milliseconds ago a file was last changed,
/// zero when that cannot be told.
/// Why:
///  `describe_lock` says how old a lock file is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Date.now() - statSync(lock).mtimeMs
/// ```
fn age_millis(metadata: &std::fs::Metadata) -> u128 {
    let Ok(made) = metadata.modified() else {
        return 0;
    };
    return match SystemTime::now().duration_since(made) {
        Ok(age) => age.as_millis(),
        Err(_in_the_future) => 0,
    };
}

/// What:
///  The description of the lock at `lock` as a waiter finds it now:
/// `vanished` when it is gone,
/// `service seq=.. pid=.. state=.. age_ms=..` for a lock with the service's marker,
/// and `foreign no-marker size=.. age_ms=..` for any other.
/// Why:
///   One line a person can read in the service's log to see what a wait was for.
/// The marker's birth identity (the start tick,
/// the boot ID,
/// the PID namespace) is not in it:
/// the state says what it was used for,
/// and the identifiers are not needed to read the line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// describeLock(lockPath) // string
/// ```
pub fn describe_lock(lock: &Path) -> String {
    let Ok(metadata) = std::fs::metadata(lock) else { return String::from("vanished") };
    let age_ms: u128 = age_millis(&metadata);
    return match marker_of(lock) {
        Some(marked) => match marked.holder {
            Some((pid, birth)) => format!(
                "service seq={} pid={pid} state={} age_ms={age_ms}",
                marked.seq,
                state_word(holder_state(Some(&(pid, birth))))
            ),
            None => format!("service seq={} holder=none (an earlier release's marker) age_ms={age_ms}", marked.seq),
        },
        None => format!("foreign no-marker size={} age_ms={age_ms}", metadata.len()),
    };
}
