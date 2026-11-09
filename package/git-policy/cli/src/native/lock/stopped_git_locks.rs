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
//! The sweep adds no proof of its own.
//! A number in a pid file is not one:
//! a Git of a sibling PID namespace that shares this repository may have written that number,
//! and a lock this wrapper's child was seen to hold cannot be told from another Git's lock at the same path
//! (cross-vendor review of 2026-11-09,
//! finding 1).
//! A lock the sweep will not remove is named,
//! with the command that clears it.
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
/// How reference locks are judged.
use crate::stale_ref_lock_judge::Judge;
/// The judgement and conditional removal of the reference locks below some Git directories.
use crate::stale_ref_locks::clear_stale_ref_locks;
/// Paths of the repository.
use std::path::{Path, PathBuf};
/// The command whose children are stopped.
use std::process::Command;

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
/// the one the child worked in,
/// and removes one only on a proof about its holder.
/// It adds none of its own:
/// a number in a pid file says nothing about which namespace wrote it,
/// and a lock a member of the tree was seen to hold is a lock another Git may hold at the same path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LockLedger = { roots: string[]; judge?: Judge };
/// ```
#[derive(Clone, Debug)]
pub struct LockLedger {
    /// The Git directory and the common Git directory of the repository the command runs in;
    /// empty when none is found.
    roots: Vec<PathBuf>,
    /// The judge of this process,
    /// unless a control sets another.
    judge: Option<Judge>,
}

impl LockLedger {
    /// A ledger that finds nothing:
    /// for a child that holds no Git lock,
    /// such as a reader of objects.
    pub fn none() -> Self {
        return Self { roots: Vec::new(), judge: None };
    }

    /// The ledger of a command that has not started yet:
    /// its repository comes from `GIT_DIR` in its environment,
    /// or from
    ///  its working directory (the process's own when the command sets none).
    pub fn before_start(command: &Command) -> Self {
        return Self { roots: roots_of_command(command), judge: None };
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
    pub fn sweep(&self) -> Vec<PathBuf> {
        let judge: Judge = self.judge.clone().unwrap_or_else(Judge::here);
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
