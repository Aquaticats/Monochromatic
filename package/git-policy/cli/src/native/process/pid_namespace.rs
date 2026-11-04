//! What:
//!  The PID namespace an owner record names, and the rule for judging an owner that lives in another one.
//! Why:
//!  A PID means something only inside its PID namespace. A wrapper in a container that probes a host owner's PID
//!  finds no process (or an unrelated one), so it would judge a live owner dead and recover or retire what the owner
//!  still holds (issues 1781 and 1784). A record therefore names its namespace, and an owner of another namespace is
//!  judged by the age of its record instead of by the PID.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sameNamespace = readlinkSync('/proc/self/ns/pid') === record.ownerPidNamespace;
//! ```

use std::path::Path;
use std::time::{Duration, SystemTime};

/// How long an owner of another PID namespace counts as alive, measured from when its record was written.
///  A commit or a lock hold lasts seconds to minutes; a crashed foreign owner stops blocking after this long.
///  Three kinds of record are judged by this age. A merge's intent record (`merge_isolation_journal.rs`, `write_merge_intent`,
///  written once and synced per merge) and a claim file (`single_owner_claim.rs`, `write_private`, written once) are not
///  rewritten while the owner works, so the age is the owner's whole hold; the item path of a worktree add is judged by its own
///  modification time (`worktree_copy_owner.rs`, `is_alive`), which the add's work can move, so its age is at most the add's
///  length. The patience must
///  outlast the longest such hold. Measured: the longest worktree add in the add timing log (48 adds, 2026-11-03T08:55Z to
///  15:06Z, `totalMs`) took 539.7 s, and the nearest log of a command that waits, the commit timing log (1,051 commits,
///  2026-11-02T09:01:37Z to 2026-11-03T15:10:12Z), has a longest of 750.6 s. No log records the length of a merge, which waits up
///  to the 1,800 s live lock wait (`wait_limit.rs`, `DEFAULT_LIVE_WAIT`) with its record already written, so an hour is twice the
///  longest wait the code allows and 4.8 times the longest command measured. If that wait is lengthened or removed, this value
///  must be revisited with it. The logs hold commands of the host's own PID namespace, so no foreign-namespace hold was observed
///  directly. Kept at an hour as the coordinator's pick under the owner's delegation, open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`, "Foreign-owner patience").
pub const FOREIGN_OWNER_PATIENCE: Duration = Duration::from_secs(3600);

/// What:
///  The identity of this process's PID namespace, such as `pid:[4026531836]`, or nothing where the system has none.
/// Why:
///   The kernel names a namespace by an inode; two processes share a PID space exactly when the names are equal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readlinkSync('/proc/self/ns/pid')
/// ```
pub fn current_pid_namespace() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        return std::fs::read_link("/proc/self/ns/pid")
            .ok()
            .and_then(|target| { return target.into_os_string().into_string().ok(); });
    }
    #[cfg(not(target_os = "linux"))]
    {
        return None;
    }
}

/// What:
///  Whether a recorded namespace and the reader's namespace are both known and differ.
/// Why:
///   Without both names nothing proves the PID space differs, so the PID probe keeps its old meaning; a record
///  written before this field existed carries no name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recorded !== undefined && current !== undefined && recorded !== current
/// ```
pub fn is_other_namespace(recorded: Option<&str>, current: Option<&str>) -> bool {
    return match (recorded, current) {
        (Some(recorded_name), Some(current_name)) => recorded_name != current_name,
        _ => false,
    };
}

/// What:
///  Whether a record written by an owner of another namespace is old enough to stop counting as alive.
/// Why:
///   The record file's modification time stands for when the owner started; a record that cannot be measured keeps
///  its owner alive, because the wrapper never retires what it cannot prove dead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Date.now() - statSync(recordPath).mtimeMs > FOREIGN_OWNER_PATIENCE_MS
/// ```
pub fn foreign_record_has_expired(record_path: &Path) -> bool {
    let Ok(modified) = std::fs::metadata(record_path).and_then(|metadata| { return metadata.modified(); }) else {
        return false;
    };
    return match SystemTime::now().duration_since(modified) {
        Ok(age) => age_outlives_patience(age),
        Err(_) => false,
    };
}

/// What:
///  Whether a record of this age has outlived the patience; exactly the patience still counts as alive.
/// Why:
///   The boundary is a fact of the rule, so it is separated from the clock to be checked exactly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// age > FOREIGN_OWNER_PATIENCE_MS
/// ```
pub fn age_outlives_patience(age: Duration) -> bool {
    return age > FOREIGN_OWNER_PATIENCE;
}

/// What:
///  Whether an owner recorded in `recorded` counts as alive without any PID probe, or nothing when the PID decides.
/// Why:
///   Callers share one rule: an owner of another namespace is alive until its record outlives the patience.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isOtherNamespace(recorded, current) ? !hasExpired(recordPath) : undefined
/// ```
pub fn foreign_owner_standing(recorded: Option<&str>, record_path: &Path) -> Option<bool> {
    if !is_other_namespace(recorded, current_pid_namespace().as_deref()) {
        return None;
    }
    return Some(!foreign_record_has_expired(record_path));
}

/// Namespace controls stay out of the release executable.
#[cfg(test)]
#[path = "pid_namespace_tests.rs"]
mod tests;
