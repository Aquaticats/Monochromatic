//! What:
//!  The worktree work the handlers run after a commit:
//!  emptying the trash a snapshot-made worktree set aside,
//!  filling a new worktree's Cargo build directories,
//!  relinking its copied package state,
//!  deleting the subvolume a removal renamed out of the person's way,
//!  making a snapshot ahead of the next add,
//!  and running the repository's automatic upkeep.
//! Why:
//!   Every one of these starts a child or waits for the kernel,
//!  so each is run from the service's own thread,
//!  with no person waiting for it,
//!  and each awaited face keeps the synchronous one's record and reasons.
//!  They share one file because they are the one step a landed commit leaves behind.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function trashAwaiting(handlers: GitHandlers, item: WorkItem): Promise<Verdict> { /* ... */ }
//! ```

use super::service_work_git_handlers::GitHandlers;
use super::service_work_handlers_and_verdicts::{
    Verdict,
    WorkHandlers,
};
use crate::maintenance_run::{
    AutoMaintenanceOutcome,
    run_auto_maintenance_with_awaiting,
};
use crate::work_item::WorkItem;

/// The thread name one removal of a whole worktree carries in a process listing,
/// so a removal that gave no answer is recognisable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REMOVAL_THREAD_NAME = 'cli-git-worktree-removal';
/// ```
const REMOVAL_THREAD_NAME: &str = "cli-git-worktree-removal";

pub(super) fn trash(item: &WorkItem) -> Verdict {
    return crate::worktree_trash::empty_trash(item);
}

/// What:
///  The same emptying,
/// its entry removals awaited on the runtime.
/// Why:
///   The drain of a round is a task of the runtime,
/// and the synchronous face enters the runtime for
/// `remove_entries`,
/// which is refused there:
/// the trash would stay and the item would settle as failed.
/// The record and the reasons are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// trashAwaiting(item) { return emptyTrashAwaiting(item); }
/// ```
pub(super) fn trash_awaiting<'a>(
    item: &'a WorkItem
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
    return Box::pin(async move {
        return crate::worktree_trash::empty_trash_awaiting(item).await;
    });
}

/// What:
///  Fill the Cargo build directories a new worktree's record names,
/// its clone and its clean awaited on the runtime.
/// Why:
///   The add hands this work over instead of waiting for it
/// (`package_repair_prewarm.rs`),
/// and the round is a task of the runtime,
/// so the tree clone and the Cargo child are awaited here rather than entered from a
///  synchronous face.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// prewarmAwaiting(item) { return prewarmAwaiting(item); }
/// ```
#[cfg(target_os = "linux")]
pub(super) fn prewarm_awaiting<'a>(
    item: &'a WorkItem
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
    return Box::pin(async move {
        return crate::package_repair_prewarm_job::prewarm_awaiting(item).await;
    });
}

/// What:
///  Relink the copied package state a new worktree's record names,
/// each tool's own install awaited on the runtime.
/// Why:
///   The add hands this work over instead of waiting for it
/// (`package_repair_relink.rs`),
/// and the round is a task of the runtime,
/// so each tool is awaited here rather than started from a synchronous face.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// relinkAwaiting(item) { return relinkAwaiting(item); }
/// ```
#[cfg(target_os = "linux")]
pub(super) fn relink_awaiting<'a>(
    item: &'a WorkItem
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
    return Box::pin(async move {
        return crate::package_repair_relink_job::relink_awaiting(item).await;
    });
}

/// What:
///  Delete the subvolume a removal renamed out of the person's way.
/// Why:
///  The removal records the path and returns,
/// so this is where the one kernel call happens,
/// with no person waiting for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// subvolumeDelete(item) { return deleteSubvolumeOf(item); }
/// ```
pub(super) fn subvolume_delete(item: &WorkItem) -> Verdict {
    return crate::worktree_remove_hand_over_service_removal::delete_subvolume_of(item);
}

/// What:
///  The same delete,
///  run on a thread of its own that no runtime owns.
/// Why:
///  The removal of a whole worktree starts Git,
///  and a step the wrapper watches is waited for through the runtime,
///  which refuses a call made from inside itself,
///  while a thread of the blocking pool still carries that runtime with it.
///  So the removal gets a thread of its own,
///  and the round awaits its answer no longer than a bulk Git step may take:
///  a thread the system refuses,
/// and one that gives no answer at the ceiling,
/// are both a removal that did not run,
///  and the record keeps its lease for the next round.
///  Before this,
/// every removal the service made answered "the runtime that runs this command's waits was asked for
///  from inside itself",
///  which read as ignored files nothing could vouch for,
///  and the service kept every worktree it was handed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// subvolumeDeleteAwaiting(item) { return await withAnOwnThreadWithin(OBJECT_STEP_CEILING, () => deleteSubvolumeOf(item)); }
/// ```
pub(super) fn subvolume_delete_awaiting<'a>(
    item: &'a WorkItem
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
    return Box::pin(async move {
        let held: WorkItem = item.clone();
        let (sender, receiver): (
            tokio::sync::oneshot::Sender<Verdict>,
            tokio::sync::oneshot::Receiver<Verdict>,
        ) = tokio::sync::oneshot::channel::<Verdict>();
        let started = std::thread::Builder::new()
            .name(String::from(REMOVAL_THREAD_NAME))
            .spawn(move || {
                // A round that stopped waiting is gone with its receiver, and the verdict goes with the send.
                let _sent = sender.send(
                    crate::worktree_remove_hand_over_service_removal::delete_subvolume_of(&held),
                );
            });
        if started.is_err() {
            return Verdict::Retry(format!(
                "the system refused a thread for the removal of {}",
                item.repository.display()
            ));
        }
        return match tokio::time::timeout(crate::wait_ceilings::OBJECT_STEP_CEILING, receiver).await
        {
            Ok(Ok(verdict)) => verdict,
            // The thread went without a verdict, which a removal cannot do while it runs.
            Ok(Err(_dropped)) => Verdict::Retry(format!(
                "the removal of {} gave no verdict",
                item.repository.display()
            )),
            Err(_expired) => Verdict::Retry(format!(
                "the removal of {} gave no answer within {:?}",
                item.repository.display(),
                crate::wait_ceilings::OBJECT_STEP_CEILING
            )),
        };
    });
}

/// What:
///  Make one snapshot of a worktree ahead of the next add,
/// where the caller has no runtime to hand a job to.
/// Why:
///   The call is one btrfs ioctl and starts no child,
/// so a synchronous caller blocks here itself;
/// the round
///  awaits it on the blocking pool through `snapshot_ahead_awaiting`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// snapshotAhead(item) { return makeOneAhead(item); }
/// ```
#[cfg(target_os = "linux")]
pub(super) fn snapshot_ahead(item: &WorkItem) -> Verdict {
    return crate::worktree_snapshot_ahead_job::snapshot_ahead(item);
}

/// What:
///  Make a snapshot of a worktree ahead of the next add,
/// on a thread of the blocking pool.
/// Why:
///  The ioctl waits for a btrfs transaction commit of the whole filesystem,
/// so a worker of the runtime must not hold it:
/// the
///  commits queued behind this one would wait on it too.
/// The service's own thread is what takes the wait,
/// and the
///  add that asked for the snapshot waits for the answer instead of for the kernel
/// (`worktree_snapshot_ahead.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// snapshotAheadAwaiting(item) { return makeOneAhead(item); }
/// ```
#[cfg(target_os = "linux")]
pub(super) fn snapshot_ahead_awaiting<'a>(
    item: &'a WorkItem
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
    return Box::pin(async move {
        let held: WorkItem = item.clone();
        return match crate::blocking_jobs_within_the_budget::run_a_slow_job_within_the_budget(
            move || {
                return crate::worktree_snapshot_ahead_job::snapshot_ahead(&held);
            },
        )
        .await
        {
            Ok(verdict) => verdict,
            // Nothing was made and nothing is lost: the record keeps its lease and the next round finds it.
            Err(reason) => Verdict::Retry(format!(
                "the snapshot ahead of {} did not run ({reason})",
                item.repository.display()
            )),
        };
    });
}

pub(super) fn maintenance(
    handlers: &GitHandlers,
    item: &WorkItem,
) -> Verdict {
    return match crate::lazy_runtime::run_on_runtime(handlers.maintenance_awaiting(item)) {
        Ok(verdict) => verdict,
        Err(refusal) => Verdict::Failed(format!("the automatic upkeep could not run: {refusal}")),
    };
}

pub(super) fn maintenance_awaiting<'a>(
    handlers: &'a GitHandlers,
    item: &'a WorkItem,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
    return Box::pin(async move {
        let answer: Verdict = match run_auto_maintenance_with_awaiting(
            &handlers.context,
            item.repository.as_path(),
            None,
        )
        .await
        {
            Ok(AutoMaintenanceOutcome::Exited(code)) if code != 0 => {
                Verdict::Failed(format!("it exited with status {code}"))
            }
            Ok(_) => Verdict::Done,
            Err(error) => Verdict::Failed(format!("{error:?}")),
        };
        // The upkeep is where the wrapper prunes the references it keeps built commits on, by age and by size: the
        //  person's commit has returned, and nothing a command waits for is behind this step
        //  (`kept_reference_pass.rs`).
        crate::kept_reference_pass::prune_after_a_commit(&handlers.context, item.git_dir.as_path())
            .await;
        return answer;
    });
}
