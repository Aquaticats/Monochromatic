//! What:
//!  The service's consumer of after-commit work:
//! pending `push` and `maintenance` work items of one repository are
//!  claimed,
//! coalesced,
//! run,
//! settled,
//! and a push that failed leaves a record that the person's next wrapper command shows.
//! Why:
//!  A commit must not wait for the network or for upkeep,
//! so the wrapper leaves a record and the resident service does
//!  the work on its own thread (`doc/planning/cli-git-resident-service.md`,
//! "After-commit work").
//! The service is the
//!  only pusher of the repositories it serves,
//! so no lock is needed:
//! several pending pushes of one branch become one
//!  push,
//! and a service that died mid-push is noticed by the lease of its claims running out,
//! never by a process id:
//! the
//!  holder renews the lease while it works,
//! so a successor that starts beside a holder that is still running leaves its claims
//!  alone,
//! and one that starts after the holder died takes them back as soon as the lease is over.
//! Other kinds of work
//!  (the standards review) are never claimed here.
//!
//! The drains are here,
//! with the trait the runtime asks of the real handlers;
//! the handlers themselves and the review they run are `service_work_git_handlers.rs`,
//! a push's verdict is `service_work_pushes.rs`,
//! and the worktree work a landed commit leaves is `service_work_worktree_upkeep.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const group of coalesce(await claimAll(workDir, ['push', 'maintenance']))) settle(group, await handler(group));
//! ```

/// The real handlers of the service's work,
/// whose settings and review are `service_work_git_handlers.rs`.
pub use super::service_work_git_handlers::GitHandlers;
use crate::claim_time::ClaimTime;
use crate::parallel_budget::HostResources;
use crate::push_retry::Retrying;
use crate::service_claim_renewal_running::Renewal;
pub(crate) use crate::service_work_common_directory::common_directory_of;
pub use crate::service_work_drains_awaiting::{
    drain_common_records_planned_awaiting,
    drain_planned_awaiting,
    drain_planned_with_awaiting,
};
pub use crate::service_work_handlers_replanning_and_acceptance::*;
use crate::work_item::WorkItem;
use std::path::Path;
use std::time::Duration;

/// How long a claim goes without a renewal before another instance runs its item again;
/// the holder renews it three times in this
/// span while it works (`service_claim_renewal.rs`),
/// so a holder that is slow keeps its claim and one that died loses it
/// after this long.
///  Measured by killing a throwaway service during a claim:
/// the service binary in a fixture repository claimed a push held by the
///  remote's hook,
/// was killed with SIGKILL 1.5 s after the claim (with its process group),
/// and a new service was started and woken;
///  the push was made again 119.57,
/// 119.56 and 119.55 s after the kill in three runs (2026-11-03),
/// so the cost of a kill is the lease
///  to within half a second.
/// No kill of a commit service appears in the user journal of this host (19 boots),
/// so the kills a person
///  meets were not counted.
/// The renewal thread is independent of the landing (`service_claim_renewal.rs`),
/// so a landing that stalled for
///  119.8 s (the longest `total_ms` in the service journal) does not stop renewals;
/// a stall of the whole process longer than the
///  80 s a renewal may be late would.
/// Kept at 120 s as the coordinator's pick under the owner's delegation,
/// open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Picked for these follow-ups",
/// batch size and claim lease,
/// option B).
pub const CLAIM_LEASE: Duration = Duration::from_secs(120);

/// What:
///  Runs everything pending for one repository:
/// takes back the claims whose holder stopped renewing them,
/// claims every pending
///  push,
/// maintenance and review item,
/// runs one push per branch and one upkeep per repository,
/// settles the items,
/// and leaves a
///  notice for each failed or skipped push.
/// Gives how long until a claim another instance still holds runs out its lease,
/// or
///  nothing when there is none.
/// Why:
///   Coalescing keeps a burst of commits to one push;
/// a handler that fails leaves the items in `failed`,
/// not in a loop,
///  and one that answers `Retry` leaves its item claimed,
/// so the lease running out queues it once more and no round spins on it.
/// A
///  claim found in this directory may belong to a live instance (a restarted service starts beside the one it replaces,
/// and the
///  linked worktrees of a bare repository share a directory),
/// so only its lease says it was abandoned;
/// the caller is told when
///  to look again,
/// because nothing else would wake it once the lease is over.
/// This keeps no retry plan:
/// a push that could pass
///  is reported as failed,
/// as for handlers that do not retry.
/// The service's rounds use `drain_planned` instead.
/// It claims every kind the service runs (`SERVICE_KINDS`),
/// because no lane runs beside it as it does beside a round
///  (`landing/service_work_slow_lane.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wait = await drain(gitDir, handlers, readClaimTime())
/// ```
pub fn drain(
    git_dir: &Path,
    handlers: &(dyn WorkHandlers + Sync),
    now: &ClaimTime,
) -> Option<Duration> {
    return drain_with_lease(git_dir, handlers, now, CLAIM_LEASE);
}

/// `drain` with the lease given,
/// so a test can use a short one.
pub fn drain_with_lease(
    git_dir: &Path,
    handlers: &(dyn WorkHandlers + Sync),
    now: &ClaimTime,
    lease: Duration,
) -> Option<Duration> {
    return drain_with_renewal(git_dir, handlers, now, lease, Renewal::Timed);
}

/// `drain_with_lease` with the way its claims are kept fresh given,
/// so a control can drive the renewal itself.
pub fn drain_with_renewal(
    git_dir: &Path,
    handlers: &(dyn WorkHandlers + Sync),
    now: &ClaimTime,
    lease: Duration,
    renewal: Renewal<'_>,
) -> Option<Duration> {
    return drain_planned_with(git_dir, handlers, now, lease, renewal, &Retrying::none()).wait;
}

/// `drain` for a service round:
/// the pushes due again come with the request,
/// and the changes the drain asks of the retry plan are
/// given back with the wait,
/// for the service loop to apply.
pub fn drain_planned(
    git_dir: &Path,
    handlers: &(dyn WorkHandlers + Sync),
    now: &ClaimTime,
    retrying: &Retrying,
) -> Drained {
    return drain_planned_with(
        git_dir,
        handlers,
        now,
        CLAIM_LEASE,
        Renewal::Timed,
        retrying,
    );
}

/// What:
///  `drain_planned` with the lease and the renewal given,
/// its waits made by entering the runtime once.
/// Why:
///   The body is the awaited one (`service_work_drains_awaiting.rs`),
/// so the two faces cannot drain differently;
/// a caller inside the
///  runtime cannot enter it and is told so,
/// which leaves every record it holds to its lease.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runOnRuntime(drainPlannedWithAwaiting(gitDir, handlers, now, lease, renewal, retrying, readHost()))
/// ```
pub fn drain_planned_with(
    git_dir: &Path,
    handlers: &(dyn WorkHandlers + Sync),
    now: &ClaimTime,
    lease: Duration,
    renewal: Renewal<'_>,
    retrying: &Retrying,
) -> Drained {
    // What: the CPUs and memory every batch of this drain is sized by, read once for the drain.
    // Why:  a drain outside the round has no caller stating a host, and its batches must not each read the machine.
    let host: HostResources = HostResources::read();
    return match crate::lazy_runtime::run_on_runtime(drain_planned_with_awaiting(
        git_dir,
        handlers,
        now,
        lease,
        renewal,
        retrying,
        SERVICE_KINDS,
        &host,
    )) {
        Ok(drained) => drained,
        Err(refusal) => {
            crate::diagnostic_log::debug(
                "drainPlanned",
                format!("the drain of {} was not run: {refusal}", git_dir.display()).as_str(),
            );
            Drained::default()
        }
    };
}

/// What:
///  Runs the records the releases before one service per worktree left in the common directory of a bare repository,
/// from the
///  instance of a linked worktree:
/// nothing for a repository with a main worktree,
/// for a main worktree and for a directory with
///  no `commondir`.
/// Why:
///   Those releases kept every worktree's records in the common directory.
/// A repository with a main worktree has an instance
///  whose own directory that is,
/// and it runs them (the main worktree's current records live there too),
/// so a linked instance
///  must not.
/// A repository whose worktrees are all linked has no such instance,
/// and its pushes would sit there with no notice.
///   Several linked instances may drain the directory at once,
/// so a claim is taken back only after its lease,
/// which the holder
///  renews while it works.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wait = await drainCommonRecords(linkedGitDir, handlers, now)
/// ```
pub fn drain_common_records(
    git_dir: &Path,
    handlers: &(dyn WorkHandlers + Sync),
    now: &ClaimTime,
) -> Option<Duration> {
    return drain_common_records_with_lease(git_dir, handlers, now, CLAIM_LEASE);
}

/// `drain_common_records` with the lease given,
/// so a test can use a short one.
pub fn drain_common_records_with_lease(
    git_dir: &Path,
    handlers: &(dyn WorkHandlers + Sync),
    now: &ClaimTime,
    lease: Duration,
) -> Option<Duration> {
    return drain_common_records_planned(git_dir, handlers, now, lease, &Retrying::none()).wait;
}

/// `drain_common_records` for a service round:
/// `first` is the pause before the first new try of a push that could pass (nothing
/// keeps no plan),
/// and the plans the drain asks for are keyed by the common directory,
/// which the service loop wakes by itself
/// when one is due;
/// that wake drains the common directory as a directory of its own,
/// which has no `commondir`.
/// What:
///  The same drain,
/// its waits made by entering the runtime once.
/// Why:
///   The body is the awaited one (`service_work_drains_awaiting.rs`);
/// a caller inside the runtime cannot enter it and is told so,
/// which leaves
///  the records where they are until their lease ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runOnRuntime(drainCommonRecordsPlannedAwaiting(gitDir, handlers, now, lease, retrying, readHost()))
/// ```
pub fn drain_common_records_planned(
    git_dir: &Path,
    handlers: &(dyn WorkHandlers + Sync),
    now: &ClaimTime,
    lease: Duration,
    retrying: &Retrying,
) -> Drained {
    // What: the CPUs and memory every batch of this drain is sized by, read once for the drain.
    // Why:  a drain outside the round has no caller stating a host, and its batches must not each read the machine.
    let host: HostResources = HostResources::read();
    return match crate::lazy_runtime::run_on_runtime(drain_common_records_planned_awaiting(
        git_dir,
        handlers,
        now,
        lease,
        retrying,
        SERVICE_KINDS,
        &host,
    )) {
        Ok(drained) => drained,
        Err(refusal) => {
            crate::diagnostic_log::debug(
                "drainCommonRecords",
                format!(
                    "the drain of the common records of {} was not run: {refusal}",
                    git_dir.display()
                )
                .as_str(),
            );
            Drained::default()
        }
    };
}

/// Trait implementation for the real Git.
///
/// Each method is one line of this file:
/// the steps of the kind it names live in the file named for that kind,
/// so a reader takes the kinds the runtime asks for here
/// and the work each one does where that work is described.
impl WorkHandlers for GitHandlers {
    fn push(
        &self,
        items: &[WorkItem],
    ) -> Verdict {
        return super::service_work_pushes::push(self, items);
    }

    fn push_awaiting<'a>(
        &'a self,
        items: &'a [WorkItem],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return super::service_work_pushes::push_awaiting(self, items);
    }

    fn trash(
        &self,
        item: &WorkItem,
    ) -> Verdict {
        return super::service_work_worktree_upkeep::trash(item);
    }

    fn trash_awaiting<'a>(
        &'a self,
        item: &'a WorkItem,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return super::service_work_worktree_upkeep::trash_awaiting(item);
    }

    #[cfg(target_os = "linux")]
    fn prewarm_awaiting<'a>(
        &'a self,
        item: &'a WorkItem,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return super::service_work_worktree_upkeep::prewarm_awaiting(item);
    }

    #[cfg(target_os = "linux")]
    fn relink_awaiting<'a>(
        &'a self,
        item: &'a WorkItem,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return super::service_work_worktree_upkeep::relink_awaiting(item);
    }

    fn subvolume_delete(
        &self,
        item: &WorkItem,
    ) -> Verdict {
        return super::service_work_worktree_upkeep::subvolume_delete(item);
    }

    fn subvolume_delete_awaiting<'a>(
        &'a self,
        item: &'a WorkItem,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return super::service_work_worktree_upkeep::subvolume_delete_awaiting(item);
    }

    #[cfg(target_os = "linux")]
    fn snapshot_ahead(
        &self,
        item: &WorkItem,
    ) -> Verdict {
        return super::service_work_worktree_upkeep::snapshot_ahead(item);
    }

    #[cfg(target_os = "linux")]
    fn snapshot_ahead_awaiting<'a>(
        &'a self,
        item: &'a WorkItem,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return super::service_work_worktree_upkeep::snapshot_ahead_awaiting(item);
    }

    fn maintenance(
        &self,
        item: &WorkItem,
    ) -> Verdict {
        return super::service_work_worktree_upkeep::maintenance(self, item);
    }

    fn maintenance_awaiting<'a>(
        &'a self,
        item: &'a WorkItem,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return super::service_work_worktree_upkeep::maintenance_awaiting(self, item);
    }

    fn review(
        &self,
        item: &WorkItem,
    ) -> Verdict {
        return super::service_work_git_handlers::review(self, item);
    }

    fn review_awaiting<'a>(
        &'a self,
        item: &'a WorkItem,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return super::service_work_git_handlers::review_awaiting(self, item);
    }
}

/// Work consumer controls stay out of the release executable.
#[cfg(test)]
#[path = "service_work_tests.rs"]
mod tests;

/// A push the service stopped is tried again on its own:
/// the controls with a remote that stalls once.
#[cfg(test)]
#[path = "service_push_retry_tests.rs"]
mod push_retry_tests;
