//! What:
//!  Drains the pending items of one work directory as a task of the runtime:
//! claims them under a lease,
//!  settles the unacceptable ones,
//! groups the pushes by branch,
//! and runs each kind with its awaited handler,
//!  every storage step in a job of the blocking pool.
//! Why:
//!  It is the largest step of the service's work and has a file of its own,
//! so the drains that call it stay short.
//! The round of the service's work is tasks of the runtime (`service_round.rs`),
//! and a task cannot start a Git child
//!  through a synchronous face:
//! the faces that enter the runtime refuse a call from inside it
//! (`lazy_runtime::run_on_runtime`),
//! so each kind is asked through its
//!  awaited face and each step that reads or renames a file is awaited on the pool
//! (`service_work_storage_awaited.rs`),
//! which keeps a worker free while storage or Git waits.
//! The claims are settled by the same task that took them:
//! the set has one owner at a time and travels into each storage job and back,
//! so nothing locks it,
//! and a
//!  job the system refuses a thread for ends the round rather than the set.
//! The renewal is stopped by a wait of the runtime,
//! so nothing this file starts is left running behind it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await drainPending(gitDir, handlers, now, lease, renewal, retrying)
//! ```

use super::service_work_handlers_replanning_and_acceptance::*;
use crate::service_work_storage_awaited::{
    acceptable_awaiting, claim_all_awaiting, clear_push_failures_before_awaiting, record_push_failure_awaiting, record_push_skip_awaiting,
    settle_all_awaiting, still_held_awaiting,
};
use crate::journald_native::Priority;
use crate::push_failure_notice::PushOutcome;
use crate::service_claim_renewal::{HeldClaims, held_claims, renewal_interval};
use crate::service_claim_renewal_running::{ClaimRenewal, Renewal};
use crate::service_timing::log_entry;
use crate::work_item::{Claimed, DONE, FAILED, WorkItem, WorkKind};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// What:
///  Claims and runs every pending item of one work directory until none is left;
/// the claims held are renewed every third of
/// `lease`,
/// so that none of them looks abandoned to another instance while it waits or runs.
/// Why:
///   See the module description.
/// A directory that cannot be listed ends the round where it is,
/// as before,
/// and the records it holds keep their lease.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await drainPendingAwaiting(gitDir, handlers, lease, renewal, replanning)
/// ```
pub(crate) async fn drain_pending_awaiting<H: WorkHandlers + ?Sized>(
    git_dir: &Path,
    handlers: &H,
    lease: Duration,
    renewal: Renewal<'_>,
    replanning: &mut Replanning,
) {
    let work_dir: PathBuf = work_directory(git_dir);
    let (the_claims, renewer) = held_claims(work_dir.as_path());
    let mut held: HeldClaims = the_claims;
    let renewing: ClaimRenewal = match renewal {
        Renewal::Timed => ClaimRenewal::start(renewer, renewal_interval(lease)),
        Renewal::Driven(driver) => ClaimRenewal::driven(renewer, driver),
    };
    // A round that cannot hand its claims back is over: the set went with a job the system refused a thread for, so nothing here
    //  can settle what it claimed, and every record keeps its lease for the next round.
    'round: loop {
        // One listing of `pending` for the whole round: each claim of it carries the reading of the moment it is taken, not of
        //  the start of the round, which a long round makes most of a lease old.
        let claimed: Vec<Claimed> = match claim_all_awaiting(work_dir.as_path()).await {
            Ok(taken) => taken,
            Err(reason) => {
                crate::service_error!("{reason}");
                break;
            }
        };
        for item in &claimed {
            held.hold(item);
        }
        if claimed.is_empty() {
            break;
        }
        // Claims that are no longer acceptable are failed; the others are sorted by kind, in the order they were claimed.
        let (acceptable, refused): (Vec<Claimed>, Vec<Claimed>) = acceptable_awaiting(git_dir, claimed).await;
        match settle_all_awaiting(held, work_dir.as_path(), refused, FAILED).await {
            Some(back) => held = back,
            None => break 'round,
        }
        let mut pushes: Vec<Claimed> = Vec::new();
        let mut trashes: Vec<Claimed> = Vec::new();
        let mut reviews: Vec<Claimed> = Vec::new();
        let mut upkeep: Vec<Claimed> = Vec::new();
        for item in acceptable {
            match item.item.kind {
                WorkKind::Push => pushes.push(item),
                WorkKind::Trash => trashes.push(item),
                WorkKind::Review => reviews.push(item),
                _ => upkeep.push(item),
            }
        }
        let mut groups: Vec<((PathBuf, String), Vec<Claimed>)> = Vec::new();
        for item in pushes {
            let key = (item.item.repository.clone(), item.item.branch_ref.clone());
            let mut joined_at: Option<usize> = None;
            for (at, existing) in groups.iter().enumerate() {
                if existing.0 == key {
                    joined_at = Some(at);
                    break;
                }
            }
            match joined_at {
                Some(at) => groups[at].1.push(item),
                None => groups.push((key, vec![item])),
            }
        }
        for (_, claimed_members) in groups {
            let members: Vec<Claimed> = match still_held_awaiting(held, work_dir.as_path(), claimed_members).await {
                Some((back, mine)) => {
                    held = back;
                    mine
                }
                None => break 'round,
            };
            if members.is_empty() {
                continue;
            }
            let mut items: Vec<WorkItem> = Vec::new();
            for member in &members {
                items.push(member.item.clone());
            }
            // The push carries what existed when it began; a notice written after this moment is about something newer.
            let pushing_since: std::time::SystemTime = std::time::SystemTime::now();
            let pushing: Instant = Instant::now();
            let verdict: Verdict = handlers.push_awaiting(items.as_slice()).await;
            if let Some(newest) = members.last() {
                // One line for every push the service ran, so a branch that was not backed up (or was) is found by its name.
                let (priority, outcome, said): (Priority, &str, &str) = match &verdict {
                    Verdict::Done => (Priority::Info, "done", ""),
                    Verdict::Failed(text) => (Priority::Warning, "failed", text.as_str()),
                    Verdict::Retry(text) => (Priority::Warning, "retrying", text.as_str()),
                    Verdict::Skipped(text) => (Priority::Info, "skipped", text.as_str()),
                };
                let fields: [(&str, String); 3] = [
                    ("CLI_GIT_BRANCH", newest.item.branch_ref.clone()),
                    ("CLI_GIT_OID", newest.item.oid.clone()),
                    ("CLI_GIT_OUTCOME", outcome.to_string()),
                ];
                log_entry(
                    priority,
                    format!(
                        "pushed branch={:?} oid={} items={} outcome={outcome} push_ms={}{}{said}",
                        newest.item.branch_ref,
                        newest.item.oid,
                        members.len(),
                        pushing.elapsed().as_millis(),
                        if said.is_empty() { "" } else { ": " }
                    )
                    .as_str(),
                    &fields,
                );
            }
            match settle_after_the_push(held, work_dir.as_path(), git_dir, &members, verdict, pushing_since, replanning).await {
                Some(back) => held = back,
                None => break 'round,
            }
        }
        let mut upkept: Vec<PathBuf> = Vec::new();
        let still_here: Vec<Claimed> = match still_held_awaiting(held, work_dir.as_path(), upkeep).await {
            Some((back, mine)) => {
                held = back;
                mine
            }
            None => break 'round,
        };
        for item in still_here {
            if !upkept.contains(&item.item.repository) {
                upkept.push(item.item.repository.clone());
                let upkeep_began: Instant = Instant::now();
                match handlers.maintenance_awaiting(&item.item).await {
                    Verdict::Failed(reason) | Verdict::Retry(reason) => crate::service_warn!("automatic maintenance did not run: {reason}"),
                    Verdict::Done | Verdict::Skipped(_) => crate::service_log!("automatic maintenance ran maintenance_ms={}", upkeep_began.elapsed().as_millis()),
                };
            }
            match settle_all_awaiting(held, work_dir.as_path(), vec![item], DONE).await {
                Some(back) => held = back,
                None => break 'round,
            }
        }
        // The left-out files of a snapshot-made worktree are emptied after the person's backup and the upkeep, before the shadow
        //  reviews: freeing them is a bounded, local job.
        let trashed: Vec<Claimed> = match still_held_awaiting(held, work_dir.as_path(), trashes).await {
            Some((back, mine)) => {
                held = back;
                mine
            }
            None => break 'round,
        };
        for item in trashed {
            let verdict: Verdict = handlers.trash_awaiting(&item.item).await;
            match settle_with_the_words(held, work_dir.as_path(), item, verdict, "trash").await {
                Some(back) => held = back,
                None => break 'round,
            }
        }
        // Reviews go last: a push is the person's backup, a review is a shadow record, and each is bounded by its budget.
        let reviewed: Vec<Claimed> = match still_held_awaiting(held, work_dir.as_path(), reviews).await {
            Some((back, mine)) => {
                held = back;
                mine
            }
            None => break 'round,
        };
        for item in reviewed {
            let review_began: Instant = Instant::now();
            let verdict: Verdict = handlers.review_awaiting(&item.item).await;
            if matches!(verdict, Verdict::Done) {
                crate::service_log!("the review of {} is done review_ms={}", item.item.oid, review_began.elapsed().as_millis());
            }
            match settle_with_the_words(held, work_dir.as_path(), item, verdict, "review").await {
                Some(back) => held = back,
                None => break 'round,
            }
        }
    }
    // The renewal is stopped and waited for here, so a drain that returns leaves no task renaming a claim behind it.
    renewing.stop_awaiting().await;
}

/// What:
///  What one push's verdict does to its records and to the retry plan.
/// Why:
///   The four verdicts differ only in what they leave for the person and in what the plan keeps,
/// so they are one function
///  and the drain's loop stays readable.
/// A record goes before its items settle:
/// a service that dies between the
///  two pushes again,
/// and nothing is lost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await settleAfterThePush(held, workDir, gitDir, members, verdict, pushingSince, replanning)
/// ```
async fn settle_after_the_push(
    held: HeldClaims,
    work_dir: &Path,
    git_dir: &Path,
    members: &[Claimed],
    verdict: Verdict,
    pushing_since: std::time::SystemTime,
    replanning: &mut Replanning,
) -> Option<HeldClaims> {
    let newest: Option<&Claimed> = members.last();
    let settling: Vec<Claimed> = members.to_vec();
    match verdict {
        Verdict::Done => {
            // A push of the branch went through, so an earlier failure to push it is no longer true.
            if let Some(latest) = newest {
                clear_push_failures_before_awaiting(git_dir, latest.item.branch_ref.as_str(), pushing_since).await;
                replanning.forget(git_dir, latest.item.branch_ref.as_str());
            }
            return settle_all_awaiting(held, work_dir, settling, DONE).await;
        }
        Verdict::Retry(text) => {
            // Without handlers that retry, a retryable push is a failed one. A push that can pass is not known to
            //  have reached the remote, so its record says that instead of naming what was refused, and it says
            //  whether a plan will try it again.
            let (said, outcome): (String, PushOutcome) = match (newest, replanning.first) {
                (Some(latest), Some(first)) => {
                    let planned: PlannedRetry = planned_retry(git_dir, replanning, first, &latest.item, 1, text.as_str());
                    // The framing promises a retry only when the plan the words speak of exists.
                    let outcome: PushOutcome = if planned.kept { PushOutcome::Unfinished } else { PushOutcome::UnfinishedWithNoRetry };
                    (planned.words, outcome)
                }
                _ => (text, PushOutcome::UnfinishedWithNoRetry),
            };
            if let Some(latest) = newest
                && let Err(reason) =
                    record_push_failure_awaiting(git_dir, latest.item.branch_ref.as_str(), latest.item.oid.as_str(), outcome, said.as_str()).await
            {
                crate::service_error!("the failed push of {} could not be recorded for the person: {reason}", latest.item.oid);
            }
            return settle_all_awaiting(held, work_dir, settling, FAILED).await;
        }
        Verdict::Failed(text) => {
            // A failure the service gives up on is no longer waiting for a new try.
            if let Some(latest) = newest {
                replanning.forget(git_dir, latest.item.branch_ref.as_str());
            }
            // The record goes before the items settle, so an item in `failed` always has its record.
            if let Some(latest) = newest
                && let Err(reason) = record_push_failure_awaiting(
                    git_dir,
                    latest.item.branch_ref.as_str(),
                    latest.item.oid.as_str(),
                    PushOutcome::Refused,
                    text.as_str(),
                )
                .await
            {
                crate::service_error!("the failed push of {} could not be recorded for the person: {reason}", latest.item.oid);
            }
            return settle_all_awaiting(held, work_dir, settling, FAILED).await;
        }
        Verdict::Skipped(text) => {
            // Nothing was pushed, so an earlier failure of the branch is still true and stays. A skip with words leaves
            //  its own record before the items settle; one that could not be recorded settles as failed, so no item
            //  is done without the person having been told.
            let recorded: bool = match newest {
                Some(latest) if !text.is_empty() => {
                    match record_push_skip_awaiting(git_dir, latest.item.branch_ref.as_str(), latest.item.oid.as_str(), text.as_str()).await {
                        Ok(()) => true,
                        Err(reason) => {
                            crate::service_error!("the skipped push of {} could not be recorded for the person: {reason}", latest.item.oid);
                            false
                        }
                    }
                }
                _ => true,
            };
            return settle_all_awaiting(held, work_dir, settling, if recorded { DONE } else { FAILED }).await;
        }
    }
}

/// What:
///  Settle one item whose kind answers with words rather than a push:
/// a skipped one is done and says why,
/// a failed one is failed and says why.
/// Why:
///   A trash and a review answer the same four verdicts and leave the person the same two kinds of line,
/// so the drain
///  keeps one account of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await settleWithTheWords(held, workDir, item, verdict, kind)
/// ```
async fn settle_with_the_words(held: HeldClaims, work_dir: &Path, item: Claimed, verdict: Verdict, kind: &str) -> Option<HeldClaims> {
    match verdict {
        Verdict::Done => {
            return settle_all_awaiting(held, work_dir, vec![item], DONE).await;
        }
        Verdict::Failed(reason) | Verdict::Retry(reason) => {
            match kind {
                "trash" => eprintln!("monochromatic-git-policy-cli-service: the trash {} could not be emptied: {reason}", item.item.parent),
                _ => crate::service_warn!("the review of {} failed: {reason}", item.item.oid),
            }
            return settle_all_awaiting(held, work_dir, vec![item], FAILED).await;
        }
        Verdict::Skipped(reason) => {
            match kind {
                "trash" => eprintln!("monochromatic-git-policy-cli-service: the trash {} was left as it is: {reason}", item.item.parent),
                _ => crate::service_log!("the review of {} was skipped: {reason}", item.item.oid),
            }
            return settle_all_awaiting(held, work_dir, vec![item], DONE).await;
        }
    }
}
