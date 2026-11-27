//! What:
//!  The shapes one round of retries is handed:
//!  a branch waiting for a new try,
//!  the tries of one Git directory that are due again,
//!  the request a worker is given,
//!  and the change a worker asks of the plan.
//! Why:
//!   The plan belongs to the service loop alone,
//!  so a worker is handed its own copy of what is due and returns what it did;
//!  the loop applies those changes in the order they were made,
//!  which is how the last plan of a branch wins.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Due = { gitDir: string; item: WorkItem; attempt: number };
//! type RetryChange = { plan: Due } | { forget: { gitDir: string; branchRef: string } };
//! ```

/// Import the push item a plan schedules.
use crate::work_item::WorkItem;
/// Import the borrowed and owned path types a plan names its Git directory with.
use std::path::{
    Path,
    PathBuf,
};
/// Import the pause a request names,
/// and the clock reading a plan is due at.
use std::time::{
    Duration,
    Instant,
};

/// One branch waiting for a new try.
#[derive(Clone, Debug)]
pub(super) struct Waiting {
    /// The Git directory of the worktree whose service pushes.
    pub(super) git_dir: PathBuf,
    /// The newest push item of the branch;
    /// its branch is what is pushed again.
    pub(super) item: WorkItem,
    /// The number of the try that is due.
    pub(super) attempt: u32,
    /// When it is due.
    pub(super) due: Instant,
}

/// The waiting plans without the one for `branch_ref` of `git_dir`,
/// in their order.
pub(super) fn without_branch(
    waiting: Vec<Waiting>,
    git_dir: &Path,
    branch_ref: &str,
) -> Vec<Waiting> {
    let mut kept: Vec<Waiting> = Vec::new();
    for entry in waiting {
        if !(entry.git_dir == git_dir && entry.item.branch_ref == branch_ref) {
            kept.push(entry);
        }
    }
    return kept;
}

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
        return Self {
            first: None,
            due: Vec::new(),
            plans: None,
        };
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
/// type RetryChange = { plan: { gitDir; item; attempt; due } } | { forget: { gitDir; branchRef; onlyWhileUnreachable; taken } };
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
        /// Whether this forget rests on a reading that can go stale:
        /// the proof that the repository is gone,
        /// whose plans are dropped only while it still is.
        only_while_unreachable: bool,
        /// The plan the reading ended,
        /// which the round took out of the list before the reading,
        /// so that a repository that is back when this change lands keeps it.
        /// Nothing for a forget that is the final word.
        taken: Option<Due>,
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
    STOP_AFTER.with(|slot| {
        slot.set(Some(changes));
    });
    return StopAfterGuard;
}

/// Ends the control's bound when dropped.
#[cfg(test)]
pub(crate) struct StopAfterGuard;

#[cfg(test)]
impl Drop for StopAfterGuard {
    fn drop(&mut self) {
        STOP_AFTER.with(|slot| {
            slot.set(None);
        });
    }
}

/// Whether a control's bound is reached after `applied` changes.
#[cfg(test)]
pub(super) fn stopped_after(applied: usize) -> bool {
    return STOP_AFTER.with(|slot| {
        return slot.get().is_some_and(|bound| {
            return applied >= bound;
        });
    });
}
