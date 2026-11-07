//! What:
//!  The reference locks a stopped Git child may have left behind,
//! judged and reported once the child and what it started
//!  are gone.
//! Why:
//!  Git removes its `.lock` files when it ends normally or on a catchable signal,
//! but a child that is killed,
//! or that is
//!  in an uninterruptible call when the stop arrives,
//! cannot.
//! On 2026-11-03 two pushes were stopped at their deadline;
//!  about 37 seconds later,
//! in the same instant,
//! empty `refs/remotes/origin/*.lock` files appeared for exactly their two
//!  tracking refs,
//! with no pid file beside them,
//! and every later `git fetch` of those refs failed with "Unable to create
//!  ... File exists".
//! The service had started those pushes without `core.lockfilePid`,
//! so no pid file was written.
//! That
//!  the creators were the stopped pushes,
//! released by the kernel late,
//! is the inference that fits these facts;
//! nothing
//!  observed the processes themselves.
//!  Releases 0.5.23 to 0.5.26 removed two kinds of file after the stop:
//! a lock whose pid file named an ended member of the
//!  stopped tree,
//! and an empty lock without a pid file that was born after the child started and stayed empty for a settle
//!  time.
//! A cross-vendor review of 2026-11-03T11:33:50-05:00 (finding S-1) showed that neither is an ownership proof:
//! a
//!  live writer of another Git that does not write pid files holds its lock empty with no descriptor open,
//! for as long as
//!  its transaction lasts,
//! and a pid file older than a lock names an earlier holder,
//! so both rules could delete the lock
//!  of a Git that is running,
//! and two writers could then update one reference.
//! The sweep removes nothing on those rules.
//!  It hands the repository to the same judgement that runs before every fetch and push (`stale_ref_locks.rs`),
//! which
//!  removes a reference lock only on a proof and through a conditional removal,
//! and it says what it found and which
//!  command clears a lock it will not remove.
//! The judgement is given one proof of its own,
//! which a container's view of the
//!  machine cannot give it:
//! the numbers of the processes of the stopped tree that have ended.
//! A pid file born at or after a
//!  lock that was itself born after the child started,
//! naming one of them,
//! names a process the wrapper started and saw end,
//! in
//!  its own namespace (the container gate found the sweep leaving its own child's locks without it).
//!  Locks of other protocols (the index,
//! the shallow file,
//! object and pack locks) are not touched or mentioned.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ledger = LockLedger.beforeStart(command); const tree = await stop(child);
//! for (const lock of ledger.sweep(tree)) log(`removed ${lock}`);
//! ```

/// The diagnostic log of the wrapper.
use crate::diagnostic_log::warn;
/// The Git directories a command works in.
use crate::git_directory_roots::roots_of_command;
/// How reference locks are judged,
/// and what the judge knows of the wrapper's own stopped child.
use crate::stale_ref_lock_found::PidFile;
use crate::stale_ref_lock_identity::identity_of;
use crate::stale_ref_lock_judge::{ClaimedLock, Judge, OwnEnded};
use crate::stale_ref_lock_walk::ref_lock_candidates;
/// The processes of the stopped tree and whether each still runs.
use crate::process_tree::{TreeMember, still_running};
/// The judgement and conditional removal of the reference locks below some Git directories.
use crate::stale_ref_locks::clear_stale_ref_locks;
/// Paths of the repository.
use std::path::{Path, PathBuf};
/// The command whose children are stopped.
use std::process::Command;
/// The wall clock the kernel stamps files with.
use std::time::SystemTime;

/// Tag used for diagnostics from this module.
const TAG: &str = "monochromatic-git-policy-cli";

/// What every Git lock file's name ends with.
pub(crate) const LOCK_SUFFIX: &str = ".lock";

/// What the pid file of a lock ends with:
/// the lock's name without `.lock`,
/// then this.
pub(crate) const PID_LOCK_SUFFIX: &str = "~pid.lock";

/// What a pid file's text starts with,
/// before the number.
const PID_TEXT_PREFIX: &str = "pid ";

/// What:
///  Where a command's Git child can leave reference locks.
/// Why:
///   The sweep after a stop judges the locks of this repository,
/// the one the child worked in;
/// the time taken before the
///  child started tells a lock born in its life from an older one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LockLedger = { roots: string[]; since: Date; judge?: Judge };
/// ```
#[derive(Clone, Debug)]
pub struct LockLedger {
    /// The Git directory and the common Git directory of the repository the command runs in;
    /// empty when none is found.
    roots: Vec<PathBuf>,
    /// The wall-clock time read before the child started.
    since: SystemTime,
    /// The judge of this process,
    /// unless a control sets another.
    judge: Option<Judge>,
    /// The locks a member of the tree was seen holding before the stop signalled it.
    claimed: Vec<ClaimedLock>,
}

impl LockLedger {
    /// A ledger that finds nothing:
    /// for a child that holds no Git lock,
    /// such as a reader of objects.
    pub fn none() -> Self {
        return Self { roots: Vec::new(), since: SystemTime::now(), judge: None, claimed: Vec::new() };
    }

    /// The ledger of a command that has not started yet:
    /// its repository comes from `GIT_DIR` in its environment,
    /// or from
    ///  its working directory (the process's own when the command sets none).
    pub fn before_start(command: &Command) -> Self {
        let since: SystemTime = SystemTime::now();
        return Self { roots: roots_of_command(command), since, judge: None, claimed: Vec::new() };
    }

    /// What:
    ///  The same ledger with the reference locks that a still-running member of `tree` holds,
    /// as the pid files beside them say now.
    /// Why:
    ///   Git removes a lock's pid file and then the lock,
    /// so a kill between the two leaves a lock nothing names.
    /// The pid file is read
    ///  before the stop signals the tree,
    /// while it still names its holder,
    /// and the lock is pinned by its file identity;
    /// once the
    ///  member has ended the sweep removes that lock on the wrapper's own observation,
    /// whatever load delayed the holder's cleanup.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const ledger = LockLedger.beforeStart(command).claiming(tree); // before child.kill('SIGTERM')
    /// ```
    pub fn claiming(mut self, tree: &[TreeMember]) -> Self {
        for candidate in ref_lock_candidates(self.roots.as_slice()) {
            if candidate.orphan_staging {
                continue;
            }
            let Some(identity) = identity_of(candidate.lock.as_path()) else { continue };
            let born: Option<SystemTime> = std::fs::symlink_metadata(candidate.lock.as_path()).ok().and_then(created_of);
            let PidFile::Names(pid) = Judge::pid_file_state(candidate.lock.as_path(), born) else { continue };
            if holder_is_running(tree, pid) {
                self.claimed.push(ClaimedLock { lock: candidate.lock, device: identity.device, inode: identity.inode, born_ns: identity.born_ns, pid });
            }
        }
        return self;
    }

    /// The same ledger with another judge,
    /// so a control does not wait out the margin of a real one.
    #[cfg(test)]
    pub fn with_judge(mut self, judge: Judge) -> Self {
        self.judge = Some(judge);
        return self;
    }

    /// What:
    ///  Judge the reference locks of the repository once the child is gone,
    /// remove the ones whose holder is proven gone,
    ///  say what was found about the rest,
    /// and return the paths removed.
    /// Why:
    ///   Nothing ties a lock to the stopped child except a proof about its holder:
    /// the stopped child's tree,
    /// a time of
    ///  birth and an empty file all fit a live writer of another Git as well.
    /// The judgement is the one that runs before
    ///  every fetch and push,
    /// so the proof,
    /// the notice and the conditional removal are the same everywhere.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function sweep(tree: TreeMember[]): string[];
    /// ```
    pub fn sweep(&self, tree: &[TreeMember]) -> Vec<PathBuf> {
        let mut judge: Judge = self.judge.clone().unwrap_or_else(Judge::here);
        // The one proof that is the wrapper's own: the numbers of the stopped tree's processes that have ended.
        let mut pids: Vec<i64> = Vec::new();
        for member in tree {
            if !still_running(member) {
                pids.push(i64::from(member.pid));
            }
        }
        judge.own_ended = Some(OwnEnded { pids, since: self.since, claimed: self.claimed.clone() });
        let outcome = clear_stale_ref_locks(self.roots.as_slice(), &judge);
        let notice: String = outcome.notice();
        if !notice.is_empty() {
            warn(TAG, notice.trim_end());
        }
        let mut removed: Vec<PathBuf> = Vec::new();
        for found in &outcome.cleared {
            removed.push(found.lock.clone());
        }
        return removed;
    }
}

/// The creation time of a file,
/// when its metadata holds one.
fn created_of(metadata: std::fs::Metadata) -> Option<SystemTime> {
    return metadata.created().ok();
}

/// Whether `pid` is a member of `tree` that still runs.
fn holder_is_running(tree: &[TreeMember], pid: i64) -> bool {
    for member in tree {
        if i64::from(member.pid) == pid && still_running(member) {
            return true;
        }
    }
    return false;
}

/// The process number a pid file names,
/// when its text is `pid <number>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Number(readFileSync(path, 'utf8').trim().replace(/^pid /, ''))
/// ```
pub(crate) fn pid_named_by(path: &Path) -> Option<i32> {
    let text: String = std::fs::read_to_string(path).ok()?;
    return text.trim().strip_prefix(PID_TEXT_PREFIX)?.trim().parse::<i32>().ok();
}

/// Lock-sweep behavior is controlled in the test module.
#[cfg(test)]
#[path = "stopped_git_locks_tests.rs"]
mod tests;
