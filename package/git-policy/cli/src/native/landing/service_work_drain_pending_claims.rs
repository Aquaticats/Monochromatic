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
use super::service_work_settling_a_verdict::settle_after_the_push;
use crate::service_work_storage_awaited::{
    acceptable_awaiting, claim_all_awaiting, settle_all_awaiting, still_held_awaiting,
};
use crate::journald_native::Priority;
use crate::service_claim_renewal::{HeldClaims, held_claims, renewal_interval};
use crate::service_work_drain_deletes_subvolumes::delete_subvolumes_awaiting;
use crate::service_work_settling_with_words::settle_with_the_words;
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
        let mut prewarms: Vec<Claimed> = Vec::new();
        let mut relinks: Vec<Claimed> = Vec::new();
        let mut deletions: Vec<Claimed> = Vec::new();
        let mut reviews: Vec<Claimed> = Vec::new();
        let mut upkeep: Vec<Claimed> = Vec::new();
        for item in acceptable {
            match item.item.kind {
                WorkKind::Push => pushes.push(item),
                WorkKind::Trash => trashes.push(item),
                WorkKind::Prewarm => prewarms.push(item),
                WorkKind::Relink => relinks.push(item),
                WorkKind::SubvolumeDelete => deletions.push(item),
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
        // A new worktree's copied package state is relinked before anything else it asked for: it is the work the add
        //  stopped doing, and the person's next command in the worktree depends on it.
        let relinked: Vec<Claimed> = match still_held_awaiting(held, work_dir.as_path(), relinks).await {
            Some((back, mine)) => {
                held = back;
                mine
            }
            None => break 'round,
        };
        for item in relinked {
            let relinking: Instant = Instant::now();
            let verdict: Verdict = handlers.relink_awaiting(&item.item).await;
            if matches!(verdict, Verdict::Done) {
                crate::service_log!("the package state of {} was relinked relink_ms={}", item.item.repository.display(), relinking.elapsed().as_millis());
            }
            match settle_with_the_words(held, work_dir.as_path(), item, verdict, "relink").await {
                Some(back) => held = back,
                None => break 'round,
            }
        }
        // A new worktree's Cargo build directory is filled before the left-out files are emptied: the filling is the work
        //  the add stopped doing, and the sooner it lands the more likely the first build in the worktree finds it.
        let prewarmed: Vec<Claimed> = match still_held_awaiting(held, work_dir.as_path(), prewarms).await {
            Some((back, mine)) => {
                held = back;
                mine
            }
            None => break 'round,
        };
        for item in prewarmed {
            let verdict: Verdict = handlers.prewarm_awaiting(&item.item).await;
            match settle_with_the_words(held, work_dir.as_path(), item, verdict, "prewarm").await {
                Some(back) => held = back,
                None => break 'round,
            }
        }
        // A removed worktree's subvolume is deleted before the left-out files are emptied: it is one kernel call on a directory
        //  nothing else names, and it frees everything the worktree held.
        match delete_subvolumes_awaiting(held, work_dir.as_path(), deletions, handlers).await {
            Some(back) => held = back,
            None => break 'round,
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
