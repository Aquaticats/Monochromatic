//! What:
//!  Drains the pending items of one work directory as a task of the runtime:
//! claims them under a lease,
//!  settles the unacceptable ones,
//! groups the pushes by branch,
//! and runs each kind with its awaited handler,
//!  every storage step in a job of the blocking pool.
//! Inside one kind the items of different worktrees run beside each other and one worktree's items keep their turn
//! (`service_work_items_side_by_side.rs`),
//! and the pushes of different branches of the repository run beside each other.
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
//!  A round also starts housekeeping for no longer than the caller's bound,
//! and leaves the rest of it to a later round
//!  (`housekeeping_of_one_round.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await drainPending(gitDir, handlers, now, lease, renewal, retrying)
//! ```

use super::service_work_handlers_replanning_and_acceptance::*;
use super::service_work_settling_a_verdict::settle_after_the_push;
use crate::housekeeping_of_one_round::{HousekeepingOfOneRound, LOOK_AGAIN_AFTER_DEFERRING};
use crate::service_work_storage_awaited::{acceptable_awaiting, claim_all_awaiting, settle_all_awaiting, still_held_awaiting};
use crate::journald_native::Priority;
use crate::service_claim_renewal::{HeldClaims, held_claims, renewal_interval};
use crate::service_work_drain_deletes_subvolumes::delete_subvolumes_awaiting;
use crate::service_work_items_side_by_side::{pushes_side_by_side, run_and_settle_side_by_side, slow_items_side_by_side, waves_of};
use crate::service_claim_renewal_running::{ClaimRenewal, Renewal};
use crate::service_timing::log_entry;
use crate::parallel_budget::HostResources;
use crate::side_by_side_on_the_current_task::run_all_side_by_side_on_the_current_task;
use crate::work_item::{Claimed, DONE, FAILED, PENDING, WorkItem, WorkKind};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::time::{Duration, Instant, SystemTime};

/// The key two pushes may stand in each other's way over:
/// the repository and the branch the group's newest record names.
/// A group is never empty,
/// so a group with no newest record is a state this module cannot make.
fn branch_of(members: &[Claimed]) -> (PathBuf, String) {
    return match members.last() {
        Some(newest) => (newest.item.repository.clone(), newest.item.branch_ref.clone()),
        None => (PathBuf::new(), String::new()),
    };
}

/// What:
///  Ask the relink of one item and say how long it took,
///  as the wave of relinks asks it.
/// Why:
///  A named function rather than a closure,
/// because what a wave's item is asked for is a body that outlives the poll
/// that started it and captures nothing of the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function askRelink(handlers, item) { const t0 = now(); const v = await handlers.relink(item); if (v.done) log(t0); return v }
/// ```
fn ask_relink<'lent, H: WorkHandlers + Sync + ?Sized>(handlers: &'lent H, item: &'lent WorkItem) -> Pin<Box<dyn Future<Output = Verdict> + Send + 'lent>> {
    return Box::pin(async move {
        let relinking: Instant = Instant::now();
        let verdict: Verdict = handlers.relink_awaiting(item).await;
        if matches!(verdict, Verdict::Done) {
            crate::service_log!("the package state of {} was relinked relink_ms={}", item.repository.display(), relinking.elapsed().as_millis());
        }
        return verdict;
    });
}

/// The prewarm of one item,
/// as its wave asks it.
fn ask_prewarm<'lent, H: WorkHandlers + Sync + ?Sized>(handlers: &'lent H, item: &'lent WorkItem) -> Pin<Box<dyn Future<Output = Verdict> + Send + 'lent>> {
    return Box::pin(async move { return handlers.prewarm_awaiting(item).await; });
}

/// The emptying of one trash,
/// as its wave asks it.
fn ask_trash<'lent, H: WorkHandlers + Sync + ?Sized>(handlers: &'lent H, item: &'lent WorkItem) -> Pin<Box<dyn Future<Output = Verdict> + Send + 'lent>> {
    return Box::pin(async move { return handlers.trash_awaiting(item).await; });
}

/// The review of one item and the line that says how long it took,
/// as its wave asks it.
fn ask_review<'lent, H: WorkHandlers + Sync + ?Sized>(handlers: &'lent H, item: &'lent WorkItem) -> Pin<Box<dyn Future<Output = Verdict> + Send + 'lent>> {
    return Box::pin(async move {
        let review_began: Instant = Instant::now();
        let verdict: Verdict = handlers.review_awaiting(item).await;
        if matches!(verdict, Verdict::Done) {
            crate::service_log!("the review of {} is done review_ms={}", item.oid, review_began.elapsed().as_millis());
        }
        return verdict;
    });
}

/// What:
///  The identity of a claimed item,
/// for the one line that names the housekeeping a round left for a later one.
/// Why:
///  A named function keeps the line that builds the list to the words that describe it,
/// and the drain reads it once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const idOf = (item: Claimed) => item.id;
/// ```
fn id_of_claimed(item: &Claimed) -> String {
    return item.id.clone();
}

/// What:
///  Whether this round may start the housekeeping item it is holding:
/// its slice still has room,
/// or no housekeeping of this round has started yet.
/// Why:
///   The bound is about how much housekeeping a round starts,
/// so the first item of a round always starts.
/// A round that could spend its whole slice on the listing,
/// the settling of the claims it refused and the steps of the kinds before this one would otherwise leave every item for a
///  later round that would spend its slice the same way,
/// so a repository's trash would be left to a round that never comes and the loop of later rounds would spin.
fn may_start_housekeeping(slice: &HousekeepingOfOneRound, started: &mut bool) -> bool {
    if *started && slice.spent() {
        return false;
    }
    *started = true;
    return true;
}

/// What:
///  Claims and runs every pending item of one work directory of the kinds `kinds` names,
/// until none is left;
/// the claims held are renewed every third of `lease`,
/// so that none of them looks abandoned to another instance while it waits or runs.
/// Why:
///   See the module description.
/// The kinds are the caller's:
/// the round that holds the service claims `PUSH_KINDS` alone,
/// so a step that can take minutes never holds a person's push,
/// the slow lane claims `SLOW_KINDS`
///  (`landing/service_work_slow_lane.rs`),
/// and a drain outside the service claims every kind (`SERVICE_KINDS`),
///  because nothing else would run them there.
/// A directory that cannot be listed ends the round where it is,
/// as before,
/// and the records it holds keep their lease.
///  A round starts housekeeping only while `housekeeping` has room,
/// and answers how soon to look at this directory
///  again when it left some for a later round.
/// An item that has started is never interrupted:
/// the answer says what a round may start,
///  not how long one step of it may last
/// (`housekeeping_of_one_round.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await drainPendingAwaiting(gitDir, handlers, lease, renewal, replanning, kinds, housekeeping) // Duration | undefined
/// ```
pub(crate) async fn drain_pending_awaiting<H: WorkHandlers + Sync + ?Sized>(
    git_dir: &Path,
    handlers: &H,
    lease: Duration,
    renewal: Renewal<'_>,
    replanning: &mut Replanning,
    kinds: &'static [WorkKind],
    housekeeping: Duration,
) -> Option<Duration> {
    let work_dir: PathBuf = work_directory(git_dir);
    let (the_claims, renewer) = held_claims(work_dir.as_path());
    let mut held: HeldClaims = the_claims;
    // The housekeeping this round may still start, and what it left for a later round.
    let slice: HousekeepingOfOneRound = HousekeepingOfOneRound::starting_now(housekeeping);
    let mut housekeeping_has_started: bool = false;
    let mut left_for_a_later_round: Vec<Claimed> = Vec::new();
    let mut look_again: Option<Duration> = None;
    let renewing: ClaimRenewal = match renewal {
        Renewal::Timed => ClaimRenewal::start(renewer, renewal_interval(lease)),
        Renewal::Driven(driver) => ClaimRenewal::driven(renewer, driver),
    };
    // A round that cannot hand its claims back is over: the set went with a job the system refused a thread for, so nothing here
    //  can settle what it claimed, and every record keeps its lease for the next round.
    'round: loop {
        // One listing of `pending` for the whole round: each claim of it carries the reading of the moment it is taken, not of
        //  the start of the round, which a long round makes most of a lease old.
        let claimed: Vec<Claimed> = match claim_all_awaiting(work_dir.as_path(), kinds).await {
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
        // The host's CPUs and memory, read once for every batch of this round, so a round's widths are one reading.
        let host: HostResources = HostResources::read();
        let mut pushes: Vec<Claimed> = Vec::new();
        let mut trashes: Vec<Claimed> = Vec::new();
        let mut prewarms: Vec<Claimed> = Vec::new();
        let mut relinks: Vec<Claimed> = Vec::new();
        let mut deletions: Vec<Claimed> = Vec::new();
        let mut reviews: Vec<Claimed> = Vec::new();
        let mut ahead: Vec<Claimed> = Vec::new();
        let mut upkeep: Vec<Claimed> = Vec::new();
        for item in acceptable {
            match item.item.kind {
                WorkKind::Push => pushes.push(item),
                WorkKind::Trash => trashes.push(item),
                WorkKind::Prewarm => prewarms.push(item),
                WorkKind::Relink => relinks.push(item),
                WorkKind::SubvolumeDelete => deletions.push(item),
                WorkKind::Review => reviews.push(item),
                WorkKind::SnapshotAhead => ahead.push(item),
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
        // The claims of every branch are checked before any push starts, so one listing of the claim directory serves
        //  them all, and a branch another instance took over is left to it.
        let mut branches: Vec<Vec<Claimed>> = Vec::new();
        for (_, claimed_members) in groups {
            let members: Vec<Claimed> = match still_held_awaiting(held, work_dir.as_path(), claimed_members).await {
                Some((back, mine)) => {
                    held = back;
                    mine
                }
                None => break 'round,
            };
            if !members.is_empty() {
                branches.push(members);
            }
        }
        // The pushes of different branches run side by side: each names its own branch, its own objects are already in the
        //  repository's store, and only the branch it updates and the reference it tracks are its own. The same branch's
        //  records are one group, so one branch is never pushed twice at once.
        let branches_wide: usize = pushes_side_by_side(branches.len(), &host);
        let branch_keys: Vec<(PathBuf, String)> = branches.iter().map(Vec::as_slice).map(branch_of).collect();
        for wave in waves_of(branch_keys.as_slice(), branches_wide) {
            let mut running: Vec<Pin<Box<dyn Future<Output = (Verdict, SystemTime, u128)> + Send + '_>>> = Vec::with_capacity(wave.len());
            for index in &wave {
                let mut items: Vec<WorkItem> = Vec::new();
                for member in &branches[*index] {
                    items.push(member.item.clone());
                }
                running.push(Box::pin(async move {
                    // The push carries what existed when it began; a notice written after this moment is about something newer.
                    let pushing_since: SystemTime = SystemTime::now();
                    let pushing: Instant = Instant::now();
                    let verdict: Verdict = handlers.push_awaiting(items.as_slice()).await;
                    return (verdict, pushing_since, pushing.elapsed().as_millis());
                }));
            }
            let pushed: Vec<(Verdict, SystemTime, u128)> = run_all_side_by_side_on_the_current_task(running).await;
            for (index, (verdict, pushing_since, push_ms)) in wave.iter().zip(pushed) {
                let members: &Vec<Claimed> = &branches[*index];
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
                            "pushed branch={:?} oid={} items={} outcome={outcome} push_ms={push_ms}{}{said}",
                            newest.item.branch_ref,
                            newest.item.oid,
                            members.len(),
                            if said.is_empty() { "" } else { ": " }
                        )
                        .as_str(),
                        &fields,
                    );
                }
                match settle_after_the_push(held, work_dir.as_path(), git_dir, members, verdict, pushing_since, replanning).await {
                    Some(back) => held = back,
                    None => break 'round,
                }
            }
        }
        // A snapshot ahead is the one kind of work a person's own command waits for, so it runs before the upkeep
        //  and before every kind that frees space.
        match crate::service_work_drains_a_snapshot_ahead::drain_a_snapshot_ahead(held, work_dir.as_path(), ahead, handlers).await {
            Some(back) => held = back,
            None => break 'round,
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
            // The person's push is the backup and the upkeep is what keeps the repository cheap to read, so a round that has
            //  spent its housekeeping ends here and the rest waits for a later round, named below.
            if !may_start_housekeeping(&slice, &mut housekeeping_has_started) {
                left_for_a_later_round.push(item);
                continue;
            }
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
        let relinks_wide: usize = slow_items_side_by_side(relinked.len(), &host);
        match run_and_settle_side_by_side(held, work_dir.as_path(), handlers, relinked, relinks_wide, "relink", ask_relink)
        .await
        {
            Some(back) => held = back,
            None => break 'round,
        }
        // A new worktree's Cargo build directory is filled before the left-out files are emptied: the filling is the work
        //  the add stopped doing, and the sooner it lands the more likely the first build in the worktree finds it.
        let mut prewarmed: Vec<Claimed> = match still_held_awaiting(held, work_dir.as_path(), prewarms).await {
            Some((back, mine)) => {
                held = back;
                mine
            }
            None => break 'round,
        };
        // The whole prewarm step of this round is one batch of waves, so a round with no housekeeping left defers the step
        //  itself rather than start it and leave its records claimed, as the deletion step below does. A step with nothing in
        //  it neither starts nor spends the round's first start: it is the items that are housekeeping and not the kinds.
        if !prewarmed.is_empty() {
            if !may_start_housekeeping(&slice, &mut housekeeping_has_started) {
                left_for_a_later_round.append(&mut prewarmed);
            } else {
                let prewarms_wide: usize = slow_items_side_by_side(prewarmed.len(), &host);
                match run_and_settle_side_by_side(held, work_dir.as_path(), handlers, prewarmed, prewarms_wide, "prewarm", ask_prewarm).await {
                    Some(back) => held = back,
                    None => break 'round,
                }
            }
        }
        // The review of a commit runs before the steps that free space: it is bounded by its own ceiling (`REVIEW_CEILING`), it
        //  decides nothing about any push, and on 2026-11-11 two reviews stayed claimed for a quarter of an hour while the round
        //  emptied the trashes of removed worktrees ahead of them, one of which can run for many minutes
        //  (`doc/planning/git-policy-cli-push-jams-2026-11-10.md`).
        let reviewed: Vec<Claimed> = match still_held_awaiting(held, work_dir.as_path(), reviews).await {
            Some((back, mine)) => {
                held = back;
                mine
            }
            None => break 'round,
        };
        let reviews_wide: usize = slow_items_side_by_side(reviewed.len(), &host);
        match run_and_settle_side_by_side(held, work_dir.as_path(), handlers, reviewed, reviews_wide, "review", ask_review)
        .await
        {
            Some(back) => held = back,
            None => break 'round,
        }
        // A removed worktree's subvolume is deleted before the left-out files are emptied: it is one kernel call on a directory
        //  nothing else names, and it frees everything the worktree held.
        // The whole deletion step of this round is one call, so a round with no housekeeping left defers the step itself rather
        //  than start it and leave its records claimed. A step with nothing in it neither starts nor spends the round's first
        //  start: it is the items that are housekeeping and not the kinds.
        if !deletions.is_empty() {
            if !may_start_housekeeping(&slice, &mut housekeeping_has_started) {
                left_for_a_later_round.append(&mut deletions);
            } else {
                match delete_subvolumes_awaiting(held, work_dir.as_path(), deletions, handlers).await {
                    Some(back) => held = back,
                    None => break 'round,
                }
            }
        }
        // The left-out files of a snapshot-made worktree are emptied last: freeing them is a local job nothing waits for, and a
        //  trash of a removed worktree can take minutes (`worktree_trash_dispose.rs`).
        let trashed: Vec<Claimed> = match still_held_awaiting(held, work_dir.as_path(), trashes).await {
            Some((back, mine)) => {
                held = back;
                mine
            }
            None => break 'round,
        };
        // The whole trash step of this round is one batch of waves, so a round with no housekeeping left defers the step
        //  itself rather than start it and leave its records claimed, as the deletion step above does. A step with nothing in
        //  it neither starts nor spends the round's first start: it is the items that are housekeeping and not the kinds.
        if !trashed.is_empty() {
            if !may_start_housekeeping(&slice, &mut housekeeping_has_started) {
                left_for_a_later_round.extend(trashed);
            } else {
                let trashes_wide: usize = slow_items_side_by_side(trashed.len(), &host);
                match run_and_settle_side_by_side(held, work_dir.as_path(), handlers, trashed, trashes_wide, "trash", ask_trash).await {
                    Some(back) => held = back,
                    None => break 'round,
                }
            }
        }
        // The housekeeping this round did not start goes back to `pending` for a later round, named here once, so a person
        //  reading the journal sees where their trash went and the service says when it looks again.
        if !left_for_a_later_round.is_empty() {
            let names: Vec<String> = left_for_a_later_round.iter().map(id_of_claimed).collect();
            crate::service_log!(
                "the round leaves {} housekeeping items of {} for a later round, because it spent {} ms of its {} ms on housekeeping: {}",
                names.len(),
                git_dir.display(),
                slice.spent_ms(),
                slice.bound().as_millis(),
                names.join(", ")
            );
            // A set the system refused a thread for cannot be handed back, and the records it held keep their lease.
            let _settled: Option<HeldClaims> = settle_all_awaiting(held, work_dir.as_path(), std::mem::take(&mut left_for_a_later_round), PENDING).await;
            look_again = Some(LOOK_AGAIN_AFTER_DEFERRING);
            break 'round;
        }
    }
    // The renewal is stopped and waited for here, so a drain that returns leaves no task renaming a claim behind it.
    renewing.stop_awaiting().await;
    return look_again;
}
