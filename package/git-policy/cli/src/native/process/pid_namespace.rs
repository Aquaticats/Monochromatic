//! What:
//!  The PID namespace an owner record names,
//! and the rule for judging an owner that lives in another one.
//! Why:
//!  A PID means something only inside its PID namespace.
//! A wrapper in a container that probes a host owner's PID
//!  finds no process (or an unrelated one),
//! so it would judge a live owner dead and recover or retire what the owner
//!  still holds (issues 1781 and 1784).
//! A record therefore names its namespace,
//! and an owner of another namespace is
//!  judged by the age of its record instead of by the PID.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sameNamespace = readlinkSync('/proc/self/ns/pid') === record.ownerPidNamespace;
//! ```

/// The longest a merge waits for a foreign program's `index.lock`,
/// which the patience is derived from.
use crate::wait_ceilings::LOCK_CEILING;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// How long an owner of another PID namespace counts as alive,
/// measured from when its record was written.
///  A commit or a lock hold lasts seconds to minutes.
/// Past this age a record is neither alive nor dead:
/// the command that finds it stops,
///  changes nothing and names it (`ExpiredForeignRecord`;
/// owner,
/// 2026-11-04,
/// "Refuse and name it (Recommended)"),
/// because the age proves no
///  death,
/// and a crashed foreign owner's record is removed by the person with the `rm` the error gives.
///  Three kinds of record are judged by this age.
/// A merge's intent record (`merge_isolation_journal.rs`,
/// `write_merge_intent`,
///  written once and synced per merge) and a claim file (`single_owner_claim.rs`,
/// `write_private`,
/// written once) are not
///  rewritten while the owner works,
/// so the age is the owner's whole hold;
/// the item path of a worktree add is judged by its own
///  modification time (`worktree_copy_owner.rs`,
/// `is_alive`),
/// which the add's work can move,
/// so its age is at most the add's
///  length.
/// The patience must outlast the longest such hold.
///  Basis:
/// twice `LOCK_CEILING` (10 minutes,
/// `wait_ceilings.rs`),
/// the longest a merge waits for a foreign `index.lock` with its
///  record already written,
/// so 20 minutes.
/// It is derived,
/// not written as a number,
/// so the two cannot drift apart again:
/// it was
///  an hour because it was twice the 30 minute live lock wait of `wait_limit.rs`,
/// a file that no longer exists,
/// and the owner's
///  rule (2026-11-03,
/// "nothing should ever wait forever or indefinitely") bounded that wait at `LOCK_CEILING`.
/// Measured:
/// the
///  longest worktree add in the add timing log (79 adds,
/// 2026-11-03T08:55:46Z to 2026-11-04T13:53:17Z,
/// `totalMs`) took 539.7 s
///  and the longest commit in the commit timing log (1,838 commits to 2026-11-04T14:08:47Z) 750.6 s,
/// so the patience is 1.6
///  times the longest command measured and 2.2 times the longest add.
/// The logs hold commands of the host's own PID namespace,
/// so
///  no foreign-namespace hold was observed directly.
/// If `LOCK_CEILING` changes,
/// this value follows it;
/// revisit the margin then
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Foreign-owner patience").
/// Read again on 2026-11-08T18:43-05:00:
/// the longest service-side hold in the journal is the `lock_hold_ms` of 119,721 ms,
/// a tenth of this patience,
/// and the longest whole command in the timing log is 1,228,797 ms,
/// 28,797 ms past it.
/// The log times a whole command and not a record's hold,
/// so that figure says nothing about the records this patience judges.
/// The merge it was stated against waits one minute now (`MERGE_LOCK_CEILING`),
/// which would make a patience derived the same way two minutes;
/// the value stays at twice `LOCK_CEILING` and another change owns it.
pub const FOREIGN_OWNER_PATIENCE: Duration = Duration::from_secs(2 * LOCK_CEILING.as_secs());

/// What:
///  The identity of this process's PID namespace,
/// such as `pid:[4026531836]`,
/// or nothing where the system has none.
/// Why:
///   The kernel names a namespace by an inode;
/// two processes share a PID space exactly when the names are equal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readlinkSync('/proc/self/ns/pid')
/// ```
pub fn current_pid_namespace() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        // `?` in an `Option` function returns `None` early, where the old `.ok().and_then(..)` chain produced `None` too.
        let target: std::path::PathBuf = std::fs::read_link("/proc/self/ns/pid").ok()?;
        return target.into_os_string().into_string().ok();
    }
    #[cfg(not(target_os = "linux"))]
    {
        return None;
    }
}

/// What:
///  Whether a recorded namespace and the reader's namespace are both known and differ.
/// Why:
///   Without both names nothing proves the PID space differs,
/// so the PID probe keeps its old meaning;
/// a record
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
///  The age of a record file,
/// or nothing when it cannot be measured.
/// Why:
///   The record file's modification time stands for when the owner started;
/// a record that cannot be measured keeps its owner alive,
///  because the wrapper never retires what it cannot prove dead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Date.now() - statSync(recordPath).mtimeMs
/// ```
pub fn foreign_record_age(record_path: &Path) -> Option<Duration> {
    let metadata: std::fs::Metadata = std::fs::metadata(record_path).ok()?;
    let modified: SystemTime = metadata.modified().ok()?;
    return SystemTime::now().duration_since(modified).ok();
}

/// What:
///  Whether a record of this age has outlived the patience;
/// exactly the patience still counts as alive.
/// Why:
///   The boundary is a fact of the rule,
/// so it is separated from the clock to be checked exactly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// age > FOREIGN_OWNER_PATIENCE_MS
/// ```
pub fn age_outlives_patience(age: Duration) -> bool {
    return age > FOREIGN_OWNER_PATIENCE;
}

/// What:
///  A record of another PID namespace that is older than the patience:
/// neither alive nor dead.
/// Why:
///   Owner,
/// 2026-11-04,
/// "Refuse and name it (Recommended)":
/// expiry no longer authorizes a command to treat a foreign owner as dead
///  and to recover,
/// take over or remove what it holds.
/// The age is a length that nothing bounds (a write that cannot be run twice may
///  run past any number,
/// and a step the wrapper left running outlives the wrapper),
/// so it proves no death.
/// The command stops,
/// changes
///  nothing,
/// and says which record it found and what a person checks before removing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ExpiredForeignRecord = { record: string; namespace: string; age: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpiredForeignRecord {
    /// The record file.
    pub record: std::path::PathBuf,
    /// The PID namespace the record names.
    pub namespace: String,
    /// How old the record file is.
    pub age: Duration,
}

impl ExpiredForeignRecord {
    /// What:
    ///  The error a command stops with:
    /// the record file,
    /// the owner's PID and namespace,
    /// the age,
    /// that the owner may be live in
    ///  another container or a Git process left running,
    /// and the exact `rm`.
    /// Why:
    ///   The person must be able to decide,
    /// and the condition for removing the record covers the Git processes the wrapper leaves
    ///  running (a step that cannot be run twice survives the wrapper that started it),
    /// not only a running monochromatic-git-policy-cli command.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// `monochromatic-git-policy-cli: the record ${record} was written by process ${pid} ... rm -- ${quoted(record)}`
    /// ```
    pub fn describe(&self, owner_pid: i64) -> String {
        return format!(
            "monochromatic-git-policy-cli: the record {record} was written by process {owner_pid} in the PID namespace {namespace}, which is not this one, and it is {minutes} \
             minutes old, past the {patience} minutes after which monochromatic-git-policy-cli no longer takes such a record for a live owner's. Nothing was \
             changed. The owner may still be alive in another container, and a Git process that monochromatic-git-policy-cli left running (a write it could not \
             safely end, such as a merge, a reset or a branch update) may still be working for it. Look in every container and on this host \
             that use this repository for a monochromatic-git-policy-cli command and for a Git process monochromatic-git-policy-cli left running (`ps` shows them); when none runs, \
             remove the record with `rm -- {path}` and run the command again.",
            record = self.record.display(),
            namespace = self.namespace,
            minutes = self.age.as_secs() / 60,
            patience = FOREIGN_OWNER_PATIENCE.as_secs() / 60,
            path = crate::stale_ref_lock_message::shell_quote(self.record.as_path())
        );
    }
}

/// What:
///  How a record is judged when it names another PID namespace:
/// alive while it is younger than the patience,
/// refused after it,
/// and
///  nothing for a record of this namespace or of none,
/// which the PID decides.
/// Why:
///   Callers share one rule:
/// an owner of another namespace is alive until its record outlives the patience,
/// and then the command
///  stops (`ExpiredForeignRecord`) and changes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isOtherNamespace(recorded, current) ? (ageOutlivesPatience(age(recordPath)) ? refused : alive) : undefined
/// ```
pub fn foreign_owner_standing(recorded: Option<&str>, record_path: &Path) -> Option<Result<(), ExpiredForeignRecord>> {
    #[cfg(test)]
    STANDING_LOOKUPS.with(|count| { count.set(count.get() + 1); });
    if !is_other_namespace(recorded, current_pid_namespace().as_deref()) {
        return None;
    }
    let namespace: String = recorded.map_or_else(String::new, String::from);
    return Some(match foreign_record_age(record_path) {
        Some(age) if age_outlives_patience(age) => Err(ExpiredForeignRecord { record: record_path.to_path_buf(), namespace, age }),
        Some(_) | None => Ok(()),
    });
}

#[cfg(test)]
thread_local! {
    /// How many times this thread asked for a standing,
    /// so a control can see that a decision asked once.
    pub(crate) static STANDING_LOOKUPS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Namespace controls stay out of the release executable.
#[cfg(test)]
#[path = "pid_namespace_tests.rs"]
mod tests;
