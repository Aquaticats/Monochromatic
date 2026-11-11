//! What:
//!  The values the service's work loop hands a worker about a branch's new try,
//! and what the worker hands back:
//! the pushes due again,
//! the request a drain is given,
//! and one change to the plan.
//! Why:
//!  The plan itself belongs to the loop alone (`push_retry.rs`),
//! so a worker gets its own copy of what is due and returns what it did,
//! and nothing is shared or waited for.
//!  The helpers that read a round's changes,
//! and the control that stops `apply` halfway,
//! read these values alone,
//! so they live here and the schedule's own file stays a page long.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // push/pushRetryWorkerValues.ts: export type Due; export type Retrying; export type RetryChange
//! ```

use crate::push_retry_store::{StoredPlan, name_of};
use crate::work_item::WorkItem;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// A push that is due again.
#[derive(Clone, Debug)]
pub struct Due {
    /// The Git directory whose service pushes;
    /// the plan is keyed by it.
    pub git_dir: PathBuf,
    /// The item to push again.
    pub item: WorkItem,
    /// The number of this try.
    pub attempt: u32,
}

/// What:
///  What the worker of one directory is handed about retries:
/// the first pause (nothing when its handlers do not retry) and the
///  pushes of that directory that are due again.
/// Why:
///   The plan belongs to the service loop alone.
/// A worker never reads it:
/// it gets its own copy of what is due for its directory,
///  as an owned value inside its request,
/// so no worker shares the plan with another and nothing waits for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Retrying = { first?: number; due: Due[] };
/// ```
#[derive(Clone, Debug, Default)]
pub struct Retrying {
    /// The pause before the first new try;
    /// nothing when a retryable failure is to be reported as a failed push.
    pub first: Option<Duration>,
    /// The pushes due again,
    /// each naming the directory it belongs to.
    pub due: Vec<Due>,
    /// The directory a plan is kept in,
    /// so that a drain can keep one on disk before it tells the person a retry is coming;
    /// nothing for a plan kept in memory only.
    pub plans: Option<PathBuf>,
}

/// Methods over a request.
impl Retrying {
    /// A request for a drain that keeps no plan:
    /// a push that could pass is reported as failed,
    /// as it is for handlers that do not retry.
    pub fn none() -> Self {
        return Self { first: None, due: Vec::new(), plans: None };
    }
}

/// What:
///  One change a worker asks of the plan:
/// plan a try,
/// or drop the plan of a branch.
/// Why:
///   A worker cannot change the plan (it does not hold it),
/// so it returns what it did and the service loop applies the changes
///  on its own thread,
/// in the order they were made,
/// which is how the last plan of a branch wins.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RetryChange = { plan: { gitDir; item; attempt; due } } | { forget: { gitDir; branchRef; asked } };
/// ```
#[derive(Clone, Debug)]
pub enum RetryChange {
    /// Try the branch of `item` again as try number `attempt`,
    /// at `due`,
    /// replacing any plan of the same branch.
    Plan {
        /// The Git directory whose service pushes.
        git_dir: PathBuf,
        /// The newest push item of the branch.
        item: WorkItem,
        /// The number of the try that is planned.
        attempt: u32,
        /// When it is due.
        due: Instant,
    },
    /// The branch needs no new try:
    /// its push landed,
    /// or the person must act.
    Forget {
        /// The Git directory whose service pushes.
        git_dir: PathBuf,
        /// The branch reference.
        branch_ref: String,
        /// When the worker asked for the branch's plans to end,
        /// in microseconds since the epoch.
        /// Why:
        ///   A forget is applied by a job that is never cancelled,
        /// so it may be written long after the round that asked for it ended;
        /// this instant is what tells the plans it read from the ones a later round wrote,
        /// which stay.
        asked: u64,
    },
}

// The number of changes a control lets `apply` apply before it stops, as a killed service leaves the round half applied;
// nothing outside a control.
#[cfg(test)]
thread_local! {
    /// The bound of the control running on this thread.
    static STOP_AFTER: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
}

/// Stops `apply` after `changes` changes on this thread until the guard drops.
#[cfg(test)]
pub(crate) fn stop_after_in_a_control(changes: usize) -> StopAfterGuard {
    STOP_AFTER.with(|slot| { slot.set(Some(changes)); });
    return StopAfterGuard;
}

/// Ends the control's bound when dropped.
#[cfg(test)]
pub(crate) struct StopAfterGuard;

#[cfg(test)]
impl Drop for StopAfterGuard {
    fn drop(&mut self) {
        STOP_AFTER.with(|slot| { slot.set(None); });
    }
}

/// Whether a control's bound is reached after `applied` changes.
#[cfg(test)]
pub(crate) fn stopped_after(applied: usize) -> bool {
    return STOP_AFTER.with(|slot| { return slot.get().is_some_and(|bound| { return applied >= bound; }); });
}

/// What:
///  Whether the round plans the same branch of the same Git directory again after `at`.
/// Why:
///   The last plan of a branch is the round's statement about it;
/// publishing an earlier one would remove the
///  file the worker of the last one already wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// changes.slice(at + 1).some((c) => c.plan && c.plan.gitDir === gitDir && c.plan.item.branchRef === branchRef)
/// ```
pub(crate) fn planned_again_after(changes: &[RetryChange], at: usize, git_dir: &Path, branch_ref: &str) -> bool {
    for change in &changes[at + 1..] {
        if let RetryChange::Plan { git_dir: planned_dir, item, .. } = change
            && planned_dir == git_dir
            && item.branch_ref == branch_ref
        {
            return true;
        }
    }
    return false;
}

/// What:
///  The file name of the last plan the changes after `at` write for one branch of one Git directory,
/// or nothing.
/// Why:
///   A forget replayed before that plan must not remove it,
/// and the name is known before the plan is written;
///  only the last plan of a branch is ever published,
/// so its name is the one to keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// changes.slice(at + 1).filter((c) => c.plan && sameBranch(c.plan)).map(planName).at(-1)
/// ```
pub(crate) fn last_plan_after(changes: &[RetryChange], at: usize, git_dir: &Path, branch_ref: &str) -> Option<String> {
    let mut name: Option<String> = None;
    for change in &changes[at + 1..] {
        if let RetryChange::Plan { git_dir: planned_dir, item, attempt, .. } = change
            && planned_dir == git_dir
            && item.branch_ref == branch_ref
        {
            name = Some(name_of(&StoredPlan { git_dir: planned_dir.clone(), item: item.clone(), attempt: *attempt }));
        }
    }
    return name;
}
