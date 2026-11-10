//! What:
//!  Drains the pending items of one work directory:
//! claims them under a lease,
//! settles the unacceptable ones,
//! groups the pushes
//!  by branch,
//! and runs each kind with its handler.
//! Why:
//!  It is the largest step of the service's work and has a file of its own,
//! so the drains that call it stay short.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await drainPending(gitDir, handlers, now, lease, renewal, retrying)
//! ```

use crate::service_work_handlers_replanning_and_acceptance::*;
use crate::journald_native::Priority;
use crate::push_failure_notice::{PushOutcome, clear_push_failures_before, record_push_failure, record_push_skip};
use crate::service_claim_renewal::{held_claims, renewal_interval};
use crate::service_claim_renewal_running::{ClaimRenewal, Renewal};
use crate::service_timing::log_entry;
use crate::work_item::{Claimed, DONE, FAILED, WorkItem, WorkKind, claim_all_of};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
/// Claims and runs every pending item of one work directory until none is left;
/// the claims held are renewed every third of
/// `lease`,
/// so that none of them looks abandoned to another instance while it waits or runs.
pub(crate) fn drain_pending(git_dir: &Path, handlers: &dyn WorkHandlers, lease: Duration, renewal: Renewal<'_>, replanning: &mut Replanning) {
    let work_dir: PathBuf = work_directory(git_dir);
    let (held, renewer) = held_claims(work_dir.as_path());
    let renewing: ClaimRenewal = match renewal {
        Renewal::Timed => ClaimRenewal::start(renewer, renewal_interval(lease)),
        Renewal::Driven(driver) => ClaimRenewal::driven(renewer, driver),
    };
    loop {
        // One listing of `pending` for the whole round: each claim of it carries the reading of the moment it is taken, not of
        //  the start of the round, which a long round makes most of a lease old.
        let claimed: Vec<Claimed> = match claim_all_of(work_dir.as_path(), &SERVICE_KINDS) {
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
        let mut acceptable: Vec<Claimed> = Vec::new();
        let mut refused: Vec<Claimed> = Vec::new();
        for item in claimed {
            if is_acceptable(&item.item, git_dir) {
                acceptable.push(item);
            } else {
                refused.push(item);
            }
        }
        settle_all(work_dir.as_path(), &held, &refused, FAILED);
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
            let members: Vec<Claimed> = still_held(&held, work_dir.as_path(), claimed_members);
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
            let verdict: Verdict = handlers.push(items.as_slice());
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
                    format!("pushed branch={:?} oid={} items={} outcome={outcome} push_ms={}{}{said}", newest.item.branch_ref, newest.item.oid, members.len(), pushing.elapsed().as_millis(), if said.is_empty() { "" } else { ": " }).as_str(),
                    &fields,
                );
            }
            match verdict {
                Verdict::Done => {
                    // A push of the branch went through, so an earlier failure to push it is no longer true. The records go
                    //  before the items settle: a service that dies between the two pushes again, and nothing is lost.
                    if let Some(newest) = members.last() {
                        clear_push_failures_before(git_dir, newest.item.branch_ref.as_str(), pushing_since);
                        replanning.forget(git_dir, newest.item.branch_ref.as_str());
                    }
                    settle_all(work_dir.as_path(), &held, &members, DONE);
                }
                Verdict::Retry(text) => {
                    // Without handlers that retry, a retryable push is a failed one. A push that can pass is not known to
                    //  have reached the remote, so its record says that instead of naming what was refused, and it says
                    //  whether a plan will try it again.
                    let (said, outcome): (String, PushOutcome) = match (members.last(), replanning.first) {
                        (Some(newest), Some(first)) => {
                            let planned: PlannedRetry = planned_retry(git_dir, replanning, first, &newest.item, 1, text.as_str());
                            // The framing promises a retry only when the plan the words speak of exists.
                            let outcome: PushOutcome = if planned.kept { PushOutcome::Unfinished } else { PushOutcome::UnfinishedWithNoRetry };
                            (planned.words, outcome)
                        }
                        _ => (text, PushOutcome::UnfinishedWithNoRetry),
                    };
                    if let Some(newest) = members.last()
                        && let Err(reason) = record_push_failure(
                            git_dir,
                            newest.item.branch_ref.as_str(),
                            newest.item.oid.as_str(),
                            outcome,
                            said.as_str(),
                        )
                    {
                        crate::service_error!("the failed push of {} could not be recorded for the person: {reason}", newest.item.oid);
                    }
                    settle_all(work_dir.as_path(), &held, &members, FAILED);
                }
                Verdict::Failed(text) => {
                    // A failure the service gives up on is no longer waiting for a new try.
                    if let Some(newest) = members.last() {
                        replanning.forget(git_dir, newest.item.branch_ref.as_str());
                    }
                    // The record goes before the items settle, so an item in `failed` always has its record.
                    if let Some(newest) = members.last()
                        && let Err(reason) =
                            record_push_failure(git_dir, newest.item.branch_ref.as_str(), newest.item.oid.as_str(), PushOutcome::Refused, text.as_str())
                    {
                        crate::service_error!("the failed push of {} could not be recorded for the person: {reason}", newest.item.oid);
                    }
                    settle_all(work_dir.as_path(), &held, &members, FAILED);
                }
                Verdict::Skipped(text) => {
                    // Nothing was pushed, so an earlier failure of the branch is still true and stays. A skip with words leaves
                    //  its own record before the items settle; one that could not be recorded settles as failed, so no item
                    //  is done without the person having been told.
                    let recorded: bool = match members.last() {
                        Some(newest) if !text.is_empty() => match record_push_skip(git_dir, newest.item.branch_ref.as_str(), newest.item.oid.as_str(), text.as_str()) {
                            Ok(()) => true,
                            Err(reason) => {
                                crate::service_error!("the skipped push of {} could not be recorded for the person: {reason}", newest.item.oid);
                                false
                            }
                        },
                        _ => true,
                    };
                    settle_all(work_dir.as_path(), &held, &members, if recorded { DONE } else { FAILED });
                }
            }
        }
        let mut upkept: Vec<PathBuf> = Vec::new();
        for item in still_held(&held, work_dir.as_path(), upkeep) {
            if !upkept.contains(&item.item.repository) {
                upkept.push(item.item.repository.clone());
                let upkeep_began: Instant = Instant::now();
                match handlers.maintenance(&item.item) {
                    Verdict::Failed(reason) | Verdict::Retry(reason) => crate::service_warn!("automatic maintenance did not run: {reason}"),
                    Verdict::Done | Verdict::Skipped(_) => crate::service_log!("automatic maintenance ran maintenance_ms={}", upkeep_began.elapsed().as_millis()),
                };
            }
            settle_all(work_dir.as_path(), &held, std::slice::from_ref(&item), DONE);
        }
        // The left-out files of a snapshot-made worktree are emptied after the person's backup and the upkeep, before the shadow
        //  reviews: freeing them is a bounded, local job.
        for item in still_held(&held, work_dir.as_path(), trashes) {
            match handlers.trash(&item.item) {
                Verdict::Done => settle_all(work_dir.as_path(), &held, std::slice::from_ref(&item), DONE),
                Verdict::Failed(reason) | Verdict::Retry(reason) => {
                    eprintln!("monochromatic-git-policy-cli-service: the trash {} could not be emptied: {reason}", item.item.parent);
                    settle_all(work_dir.as_path(), &held, std::slice::from_ref(&item), FAILED);
                }
                Verdict::Skipped(reason) => {
                    eprintln!("monochromatic-git-policy-cli-service: the trash {} was left as it is: {reason}", item.item.parent);
                    settle_all(work_dir.as_path(), &held, std::slice::from_ref(&item), DONE);
                }
            }
        }
        // Reviews go last: a push is the person's backup, a review is a shadow record, and each is bounded by its budget.
        for item in still_held(&held, work_dir.as_path(), reviews) {
            let review_began: Instant = Instant::now();
            let verdict: Verdict = handlers.review(&item.item);
            if matches!(verdict, Verdict::Done) {
                crate::service_log!("the review of {} is done review_ms={}", item.item.oid, review_began.elapsed().as_millis());
            }
            match verdict {
                Verdict::Done => settle_all(work_dir.as_path(), &held, std::slice::from_ref(&item), DONE),
                Verdict::Failed(reason) | Verdict::Retry(reason) => {
                    crate::service_warn!("the review of {} failed: {reason}", item.item.oid);
                    settle_all(work_dir.as_path(), &held, std::slice::from_ref(&item), FAILED);
                }
                Verdict::Skipped(reason) => {
                    crate::service_log!("the review of {} was skipped: {reason}", item.item.oid);
                    settle_all(work_dir.as_path(), &held, std::slice::from_ref(&item), DONE);
                }
            }
        }
    }
    // The renewal is stopped and waited for here, so a drain that returns leaves no task renaming a claim behind it; the wait is
    //  the one `ClaimRenewal`'s drop no longer makes, because a task that waits for a task holds the worker it waits for.
    renewing.stop_on_the_calling_thread();
}
