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
//! const found = describeLock(lockPath, true); // 'service seq=7 pid=42 state=running age_ms=5000 open_by=[42 (cli-git-service)]'
//! ```

use super::index_lock_holders::scan_platform_holders;
use super::service_index_marker::{HolderState, holder_state, marker_of};
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
    /// Time to create the marked file and flush it to storage,
    /// which the acquisition spends before its first placement and which a slow disk makes long.
    pub staging: Duration,
    /// Time from the first placement to the lock being taken or given up:
    /// the wait for a holder.
    pub placing: Duration,
}

/// The word for what a reader makes of the holder a marker names.
fn state_word(state: HolderState) -> &'static str {
    return match state {
        HolderState::Gone => "gone",
        HolderState::Running => "running",
        HolderState::Unknown => "unknown",
    };
}

/// The processes that have the lock open,
/// as `open_by=[pid (command), ...]`,
/// or nothing when none was seen.
fn open_by(lock: &Path, device: u64, inode: u64) -> Option<String> {
    let seen = scan_platform_holders(lock, device, inode);
    if seen.holders.is_empty() {
        return None;
    }
    let names: Vec<String> = seen
        .holders
        .iter()
        .map(|holder| {
            return holder.command.as_ref().map_or_else(|| { return holder.pid.to_string(); }, |command| { return format!("{} ({command})", holder.pid); });
        })
        .collect();
    return Some(format!("open_by=[{}]", names.join(", ")));
}

/// What:
///  The description of the lock at `lock` as a waiter finds it now:
/// `vanished` when it is gone,
/// `service seq=.. pid=.. state=.. age_ms=..` for a lock with the service's marker,
/// and `foreign no-marker size=.. age_ms=..` for any other,
/// with the processes that have it open when `scan` is set.
/// Why:
///   One line a person can read in the service's log to see what a wait was for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// describeLock(lockPath, scan) // string
/// ```
pub fn describe_lock(lock: &Path, scan: bool) -> String {
    use std::os::unix::fs::MetadataExt;
    let Ok(metadata) = std::fs::metadata(lock) else { return String::from("vanished") };
    let age_ms: u128 = metadata.modified().ok().and_then(|made| { return SystemTime::now().duration_since(made).ok(); }).map_or(0, |age| { return age.as_millis(); });
    let mut text: String = match marker_of(lock) {
        Some(marked) => match marked.holder {
            Some((pid, birth)) => format!(
                "service seq={} pid={pid} state={} age_ms={age_ms} birth={birth:?}",
                marked.seq,
                state_word(holder_state(Some(&(pid, birth.clone()))))
            ),
            None => format!("service seq={} holder=none (an earlier release's marker) age_ms={age_ms}", marked.seq),
        },
        None => format!("foreign no-marker size={} age_ms={age_ms}", metadata.len()),
    };
    if scan && let Some(open) = open_by(lock, metadata.dev(), metadata.ino()) {
        text.push(' ');
        text.push_str(open.as_str());
    }
    return text;
}
