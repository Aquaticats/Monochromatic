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

use crate::service_claim_renewal_running::Renewal;
use crate::service_work_handlers_replanning_and_acceptance::{
    Drained, PlannedRetry, Replanning, Verdict, WorkHandlers, common_directory_of, end_the_plans_of_a_gone_repository_awaiting, is_acceptable, is_bare_common_directory,
    keep_plans_while_unreachable, planned_retry, work_directory,
};
use crate::service_work_storage_awaited::{
    clear_push_failures_before_awaiting, instance_of_awaiting, next_claim_expiry_awaiting, record_push_failure_awaiting, record_push_skip_awaiting,
    requeue_expired_awaiting,
};
use crate::claim_time::ClaimTime;
use crate::push_failure_notice::PushOutcome;
use crate::push_retry::{Due, PROTECTED_BRANCH_TRIES, Retrying, out_of_tries, out_of_tries_words};
use crate::service_work_drain_pending_claims::drain_pending_awaiting;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// What:
///  Drain one directory for a service round,
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await drainPlannedAwaiting(dir, handlers, now, retrying)
/// ```
pub async fn drain_planned_awaiting<H: WorkHandlers + ?Sized>(git_dir: &Path, handlers: &H, now: &ClaimTime, lease: Duration, retrying: &Retrying) -> Drained {
    return drain_planned_with_awaiting(git_dir, handlers, now, lease, Renewal::Timed, retrying).await;
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
pub async fn drain_planned_with_awaiting<H: WorkHandlers + ?Sized>(
    git_dir: &Path,
    handlers: &H,
    now: &ClaimTime,
    lease: Duration,
    renewal: Renewal<'_>,
    retrying: &Retrying,
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
        return match crate::unreachable_git_dir_proven_gone::prove(git_dir, worktree.as_deref(), &crate::unreachable_git_dir_proven_gone::TheMountTable::default()) {
            crate::unreachable_git_dir_proven_gone::Proof::MayReturn(_) => keep_plans_while_unreachable(git_dir, retrying, &unreachable),
            crate::unreachable_git_dir_proven_gone::Proof::Gone(gone) => end_the_plans_of_a_gone_repository_awaiting(git_dir, retrying, &unreachable, &gone).await,
        };
    }
    let work_dir: PathBuf = work_directory(git_dir);
    let mut replanning: Replanning = Replanning { first: retrying.first, changes: Vec::new(), plans: retrying.plans.clone() };
    // The notice is written before the item moves to `failed`: nothing scans `failed`, so the order is the only thing that
    //  keeps a kill in between from abandoning the branch in silence.
    requeue_expired_awaiting(work_dir.as_path(), git_dir, now, lease).await;
    run_due_push_retries_awaiting(git_dir, handlers, retrying.due.as_slice(), &mut replanning).await;
    drain_pending_awaiting(git_dir, handlers, lease, renewal, &mut replanning).await;
    let wait: Option<Duration> = next_claim_expiry_awaiting(work_dir.as_path(), &ClaimTime::read(), lease).await;
    return Drained { wait, changes: replanning.changes };
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
fn worktree_named_by(retrying: &Retrying, git_dir: &Path) -> Option<PathBuf> {
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
pub async fn drain_common_records_planned_awaiting<H: WorkHandlers + ?Sized>(
    git_dir: &Path,
    handlers: &H,
    now: &ClaimTime,
    lease: Duration,
    retrying: &Retrying,
) -> Drained {
    let Some(common) = common_directory_of(git_dir) else {
        return Drained::default();
    };
    if common == git_dir || !is_bare_common_directory(common.as_path()) || !work_directory(common.as_path()).is_dir() {
        return Drained::default();
    }
    let mut replanning: Replanning = Replanning { first: retrying.first, changes: Vec::new(), plans: retrying.plans.clone() };
    requeue_expired_awaiting(work_directory(common.as_path()).as_path(), common.as_path(), now, lease).await;
    drain_pending_awaiting(common.as_path(), handlers, lease, Renewal::Timed, &mut replanning).await;
    let wait: Option<Duration> = next_claim_expiry_awaiting(work_directory(common.as_path()).as_path(), &ClaimTime::read(), lease).await;
    return Drained { wait, changes: replanning.changes };
}

/// What:
///  Run the pushes that are due again,
/// awaited by a caller that is itself a task of the runtime:
/// one push per due record,
/// a notice
///  for the person on each way it can end,
/// and the plan the next try is kept in.
/// Why:
///   A push starts Git,
/// so it is asked through the handlers' awaited face;
/// what it leaves for the person is a file,
///  so each record is a job of the pool;
/// and the words of a retry are planned by one function the synchronous face uses too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runDuePushRetriesAwaiting(gitDir, handlers, candidates, replanning)
/// ```
pub(crate) async fn run_due_push_retries_awaiting<H: WorkHandlers + ?Sized>(git_dir: &Path, handlers: &H, candidates: &[Due], replanning: &mut Replanning) {
    for due in candidates {
        if due.git_dir != git_dir {
            continue;
        }
        let branch: &str = due.item.branch_ref.as_str();
        let commit: &str = due.item.oid.as_str();
        if !is_acceptable(&due.item, git_dir) {
            crate::service_warn!("the push of {branch} is not tried again: its repository or branch record is no longer acceptable");
            replanning.forget(git_dir, branch);
            continue;
        }
        let pushing_since: std::time::SystemTime = std::time::SystemTime::now();
        let began: Instant = Instant::now();
        let verdict: Verdict = handlers.push_awaiting(std::slice::from_ref(&due.item)).await;
        match verdict {
            Verdict::Done => {
                clear_push_failures_before_awaiting(git_dir, branch, pushing_since).await;
                replanning.forget(git_dir, branch);
                crate::service_log!("the push of {branch:?} landed on try {} push_ms={}", due.attempt + 1, began.elapsed().as_millis());
            }
            Verdict::Retry(text) => {
                // What:
                //  A push the service has given every try it gives a protected-branch refusal is reported as failed,
                //  with what the server
                //  said and how long the tries covered.
                // Why:
                //  That refusal can clear by itself, so it is retried;
                // and a rule of the branch that refuses every push (an unsigned commit on a
                //  branch that requires signatures) refuses every try as well,
                // so the tries end and the branch waits for the person
                //  instead of for ever.
                if out_of_tries(text.as_str(), due.attempt + 1)
                    && let Some(first) = replanning.first
                {
                    let said: String = format!("{}\n{}", out_of_tries_words(first), text);
                    crate::service_warn!(
                        "the push of {branch:?} failed on try {} of {PROTECTED_BRANCH_TRIES} and needs you: {}",
                        due.attempt + 1,
                        text.lines().next().unwrap_or_default()
                    );
                    replanning.forget(git_dir, branch);
                    if let Err(reason) = record_push_failure_awaiting(git_dir, branch, commit, PushOutcome::Refused, said.as_str()).await {
                        crate::service_error!("the failed push of {commit} could not be recorded for the person: {reason}");
                    }
                    continue;
                }
                // The notice says what happens next, so it has to be what this round did: a plan kept, or none at all.
                let (said, outcome): (String, PushOutcome) = match replanning.first {
                    Some(first) => {
                        let planned: PlannedRetry = planned_retry(git_dir, replanning, first, &due.item, due.attempt + 1, text.as_str());
                        // The framing promises a retry only when the plan the words speak of exists.
                        let outcome: PushOutcome = if planned.kept { PushOutcome::Unfinished } else { PushOutcome::UnfinishedWithNoRetry };
                        (planned.words, outcome)
                    }
                    None => {
                        replanning.forget(git_dir, branch);
                        (text, PushOutcome::UnfinishedWithNoRetry)
                    }
                };
                if let Err(reason) = record_push_failure_awaiting(git_dir, branch, commit, outcome, said.as_str()).await {
                    crate::service_error!("the failed push of {commit} could not be recorded for the person: {reason}");
                }
            }
            Verdict::Failed(text) => {
                crate::service_warn!("the push of {branch:?} failed on try {} and needs you: {}", due.attempt + 1, text.lines().next().unwrap_or_default());
                replanning.forget(git_dir, branch);
                if let Err(reason) = record_push_failure_awaiting(git_dir, branch, commit, PushOutcome::Refused, text.as_str()).await {
                    crate::service_error!("the failed push of {commit} could not be recorded for the person: {reason}");
                }
            }
            Verdict::Skipped(text) => {
                replanning.forget(git_dir, branch);
                if !text.is_empty()
                    && let Err(reason) = record_push_skip_awaiting(git_dir, branch, commit, text.as_str()).await
                {
                    crate::service_error!("the skipped push of {commit} could not be recorded for the person: {reason}");
                }
            }
        }
    }
    return;
}
