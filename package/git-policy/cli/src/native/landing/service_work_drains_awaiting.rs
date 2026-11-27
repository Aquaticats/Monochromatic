//! What:
//!  The drains of one directory and of the common directory of a bare repository,
//! awaited by a caller that is itself a task of the runtime:
//! the claims whose holder
//!  stopped are taken back,
//! the pushes due again are tried,
//! the pending records are drained,
//!  and the wait for another instance's claim is given back.
//! Why:
//!  The round of the service's work is tasks (`service_round.rs`),
//! and a task cannot enter the runtime a synchronous face
//!  enters:
//! a push reads its child through the runtime and refuses a call from inside it
//! (`lazy_runtime::run_on_runtime`),
//! so the
//!  drain of a directory has a face a task may call,
//! and the face outside the runtime keeps the name and signature its callers have.
//! Every storage step is a job of the pool with the claim set carried with it
//!  (`service_work_storage_awaited.rs`),
//! so a worker is free while storage waits,
//! and the renewal is stopped by a wait of the runtime.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const drained = await drainPlannedAwaiting(dir, handlers, now, retrying)
//! ```

use crate::claim_time::ClaimTime;
use crate::housekeeping_of_one_round::HOUSEKEEPING_OF_ONE_ROUND;
use crate::parallel_budget::HostResources;
use crate::push_retry::Retrying;
use crate::service_claim_renewal_running::Renewal;
use crate::service_work_common_directory::{
    common_directory_of,
    is_bare_common_directory,
};
use crate::service_work_drain_pending_claims::drain_pending_awaiting;
use crate::service_work_handlers_replanning_and_acceptance::{
    Drained,
    Replanning,
    WorkHandlers,
    end_the_plans_of_a_gone_repository_awaiting,
    keep_plans_while_unreachable,
    work_directory,
};
use crate::service_work_storage_awaited::instance_of_awaiting;
use crate::work_item::WorkKind;
use std::path::{
    Path,
    PathBuf,
};
use std::time::Duration;

/// The pushes a round runs again because their retry is due
/// are declared beside this file
/// (`service_work_drains_awaiting_the_due_push_retries.rs`),
/// because the round that drains the queue is at its line bound.
pub(crate) use crate::service_work_drains_awaiting_the_due_push_retries::run_due_push_retries_awaiting;

/// What:
///  Drain one directory of the kinds `kinds` names for a service round,
/// awaited by a caller that is itself a task of the runtime:
/// the pushes due again come with the request,
/// and the changes the
///  drain asks of the retry plan are given back with the wait,
/// for the service loop to apply.
/// Why:
///   A directory that cannot be reached runs nothing and keeps its push plans,
/// and the wait for another instance's claim is
///  read after the drain,
/// as the synchronous face does.
/// The kinds are the caller's:
/// the round that answers a wake claims the pushes alone (`PUSH_KINDS`),
/// so a step that can take minutes never stands between a landing and its push,
/// and a drain outside the service claims every kind (`SERVICE_KINDS`),
///  because nothing else would run them there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await drainPlannedAwaiting(dir, handlers, now, retrying, kinds, host)
/// ```
pub async fn drain_planned_awaiting<H: WorkHandlers + Sync + ?Sized>(
    git_dir: &Path,
    handlers: &H,
    now: &ClaimTime,
    lease: Duration,
    retrying: &Retrying,
    kinds: &'static [WorkKind],
    host: &HostResources,
) -> Drained {
    return drain_planned_with_awaiting(
        git_dir,
        handlers,
        now,
        lease,
        Renewal::Timed,
        retrying,
        kinds,
        host,
    )
    .await;
}

/// What:
///  `drain_planned_awaiting` with the lease and the renewal given.
/// Why:
///   A control states a short lease and drives the renewal itself,
/// and the round names the production lease explicitly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await drainPlannedWithAwaiting(dir, handlers, now, lease, renewal, retrying)
/// ```
pub async fn drain_planned_with_awaiting<H: WorkHandlers + Sync + ?Sized>(
    git_dir: &Path,
    handlers: &H,
    now: &ClaimTime,
    lease: Duration,
    renewal: Renewal<'_>,
    retrying: &Retrying,
    kinds: &'static [WorkKind],
    host: &HostResources,
) -> Drained {
    // A directory that cannot be reached runs nothing,
    // and what becomes of its push plans is what the filesystem still proves about it:
    // a
    //  repository that may return keeps them (plan item W3),
    // and one that is gone ends them (`unreachable_git_dir_proven_gone.rs`).
    if let Err(unreachable) = instance_of_awaiting(git_dir).await {
        // The worktree the plans of this directory name, which is how a Git directory that moved is told from a repository
        //  that was deleted.
        let worktree: Option<PathBuf> = worktree_named_by(retrying, git_dir);
        return match crate::unreachable_git_dir_proven_gone::prove(
            git_dir,
            worktree.as_deref(),
            &crate::unreachable_git_dir_proven_gone::the_mounts(),
        ) {
            crate::unreachable_git_dir_proven_gone::Proof::MayReturn(_) => {
                keep_plans_while_unreachable(git_dir, retrying, &unreachable).await
            }
            crate::unreachable_git_dir_proven_gone::Proof::Gone(gone) => {
                end_the_plans_of_a_gone_repository_awaiting(git_dir, retrying, &unreachable, &gone)
                    .await
            }
        };
    }
    let mut replanning: Replanning = Replanning {
        first: retrying.first,
        changes: Vec::new(),
        plans: retrying.plans.clone(),
    };
    run_due_push_retries_awaiting(git_dir, handlers, retrying.due.as_slice(), &mut replanning)
        .await;
    // What this look answers is when to look at this directory again: the housekeeping it left for a later look and the end of
    //  the earliest claim it keeps, whichever comes first (`drain_pending_awaiting`).
    let wait: Option<Duration> = drain_pending_awaiting(
        git_dir,
        handlers,
        now,
        lease,
        renewal,
        &mut replanning,
        kinds,
        HOUSEKEEPING_OF_ONE_ROUND,
        host,
    )
    .await;
    return Drained {
        wait,
        changes: replanning.changes,
    };
}

/// What:
///  The worktree the plans of `git_dir` name,
/// which is the repository a missing Git directory may belong to.
/// Why:
///   A worktree that is still there is a repository whose own Git directory moved or is away,
/// and the proof reads that
///  difference (`unreachable_git_dir_proven_gone.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const worktree = retrying.due.find((due) => due.gitDir === dir)?.item.repository;
/// ```
fn worktree_named_by(
    retrying: &Retrying,
    git_dir: &Path,
) -> Option<PathBuf> {
    for due in &retrying.due {
        if due.git_dir == git_dir {
            return Some(due.item.repository.clone());
        }
    }
    return None;
}

/// What:
///  The records the releases before one service per worktree left in the common directory of a bare repository,
/// drained
///  from the instance of a linked worktree,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   Nothing for a repository with a main worktree,
/// for a main worktree and for a directory with no `commondir`,
/// which
///  is the synchronous face's answer,
/// and its lease is the caller's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await drainCommonRecordsPlannedAwaiting(dir, handlers, now, lease, retrying)
/// ```
pub async fn drain_common_records_planned_awaiting<H: WorkHandlers + Sync + ?Sized>(
    git_dir: &Path,
    handlers: &H,
    now: &ClaimTime,
    lease: Duration,
    retrying: &Retrying,
    kinds: &'static [WorkKind],
    host: &HostResources,
) -> Drained {
    let Some(common) = common_directory_of(git_dir) else {
        return Drained::default();
    };
    if common == git_dir
        || !is_bare_common_directory(common.as_path())
        || !work_directory(common.as_path()).is_dir()
    {
        return Drained::default();
    }
    let mut replanning: Replanning = Replanning {
        first: retrying.first,
        changes: Vec::new(),
        plans: retrying.plans.clone(),
    };
    let wait: Option<Duration> = drain_pending_awaiting(
        common.as_path(),
        handlers,
        now,
        lease,
        Renewal::Timed,
        &mut replanning,
        kinds,
        HOUSEKEEPING_OF_ONE_ROUND,
        host,
    )
    .await;
    return Drained {
        wait,
        changes: replanning.changes,
    };
}
