//! What:
//!  Drains the pending items of one work directory as a task of the runtime:
//! claims them under a lease,
//!  settles the unacceptable ones,
//! groups the pushes by branch,
//! and runs each kind with its awaited handler,
//!  every storage step in a job of the blocking pool.
//!  A look first takes back the claims whose holder stopped renewing them,
//! and it answers when to look at this directory again:
//!  the earliest of the housekeeping it left for a later look and the end of the earliest claim still held here,
//! so a lane that keeps a claim
//!  (a retryable answer,
//! or a settling step that could not run) is left with a wake for it.
//! Inside one kind the items of different worktrees run beside each other and one worktree's items keep their turn
//! (`service_work_items_side_by_side.rs`),
//! and the pushes of different branches of the repository run beside each other.
//!  Each item is settled, and its notice written, as that item's work ends,
//! and an item whose handler stopped unexpectedly fails alone (`a_panic_that_ends_one_item.rs`),
//!  so a finished push does not wait for a sibling that a silent server holds.
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

use super::service_work_drain_claims_by_kind::{
    ClaimsByKind,
    claims_by_kind,
};
use super::service_work_drain_housekeeping_and_upkeep::{
    leave_housekeeping_for_a_later_round,
    may_start_housekeeping,
    upkeep_the_held_items_awaiting,
};
use super::service_work_drain_look_again::look_again_awaiting;
use super::service_work_drain_pushes_of_each_branch::push_the_branches_awaiting;
use super::service_work_drain_waves_of_one_kind::{
    ask_prewarm,
    ask_relink,
    ask_review,
    ask_trash,
};
use super::service_work_handlers_replanning_and_acceptance::*;
use crate::claim_time::ClaimTime;
use crate::housekeeping_of_one_round::{
    HousekeepingOfOneRound,
    LOOK_AGAIN_AFTER_DEFERRING,
};
use crate::parallel_budget::HostResources;
use crate::service_claim_renewal::{
    HeldClaims,
    held_claims,
    renewal_interval,
};
use crate::service_claim_renewal_running::{
    ClaimRenewal,
    Renewal,
};
use crate::service_work_drain_deletes_subvolumes::delete_subvolumes_awaiting;
use crate::service_work_items_side_by_side::{
    run_and_settle_side_by_side,
    run_and_settle_what_the_round_may_start,
    slow_items_side_by_side,
};
use crate::service_work_storage_awaited::{
    acceptable_awaiting,
    claim_all_awaiting,
    requeue_expired_awaiting,
    settle_all_awaiting,
    still_held_awaiting,
};
use crate::work_item::{
    Claimed,
    FAILED,
    WorkKind,
};
use std::path::{
    Path,
    PathBuf,
};
use std::time::Duration;

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
/// asked as each housekeeping item starts rather than once for a batch of them,
///  sizes every batch it runs by the CPUs and memory of `host`,
/// and answers how soon to look at this directory again:
/// the earliest of what it left for a later look and the end of the earliest claim still held in this directory,
///  so a claim it keeps is looked at again when its lease runs out even though no other wake names this directory.
/// An item that has started is never interrupted:
/// the answer says what a round may start,
///  not how long one step of it may last
/// (`housekeeping_of_one_round.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await drainPendingAwaiting(gitDir, handlers, now, lease, renewal, replanning, kinds, housekeeping, host) // Duration | undefined
/// ```
pub(crate) async fn drain_pending_awaiting<H: WorkHandlers + Sync + ?Sized>(
    git_dir: &Path,
    handlers: &H,
    now: &ClaimTime,
    lease: Duration,
    renewal: Renewal<'_>,
    replanning: &mut Replanning,
    kinds: &'static [WorkKind],
    housekeeping: Duration,
    host: &HostResources,
) -> Option<Duration> {
    let work_dir: PathBuf = work_directory(git_dir);
    // What this look's caller must look at this directory again for, gathered as the look goes.
    let mut look_again: Option<Duration> = None;
    // A claim whose holder stopped renewing it is taken back before this look claims anything, so a look taken at the end of a
    //  claim finds the record `pending` and runs it again: the lease is what a retry waits for, and a look that did not take
    //  the claim back would find nothing to do and answer the same wait for ever.
    if !requeue_expired_awaiting(work_dir.as_path(), git_dir, now, lease).await {
        // The step that takes expired claims back got no place, so nothing here was claimed either, and no record of this
        //  directory names a moment to come back for: the round names the pause itself, as it does for a claim step that
        //  got none, or the taking back waits for a wake nothing sends.
        look_again = Some(LOOK_AGAIN_AFTER_DEFERRING);
    }
    let (the_claims, renewer) = held_claims(work_dir.as_path());
    let mut held: HeldClaims = the_claims;
    // The housekeeping this round may still start, and what it left for a later round.
    let slice: HousekeepingOfOneRound = HousekeepingOfOneRound::starting_now(housekeeping);
    let mut housekeeping_has_started: bool = false;
    let mut left_for_a_later_round: Vec<Claimed> = Vec::new();
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
                // Nothing was claimed, so no claim's lease runs out and no record of this directory names a moment to
                //  come back for: the round names the pause itself, or the pushes waiting here wait for a wake nothing
                //  sends, which is a wait with no bound.
                look_again = Some(LOOK_AGAIN_AFTER_DEFERRING);
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
        let (acceptable, refused): (Vec<Claimed>, Vec<Claimed>) =
            acceptable_awaiting(git_dir, claimed).await;
        match settle_all_awaiting(held, work_dir.as_path(), refused, FAILED).await {
            Some(back) => held = back,
            None => break 'round,
        }
        let ClaimsByKind {
            pushes,
            trashes,
            prewarms,
            relinks,
            mut deletions,
            reviews,
            ahead,
            upkeep,
        } = claims_by_kind(acceptable);
        // The pushes of this round's branches run beside each other, and each branch is settled, and its notice written,
        //  as its own push ends (`service_work_drain_pushes_of_each_branch.rs`).
        let pushed: Option<HeldClaims> = push_the_branches_awaiting(
            held,
            work_dir.as_path(),
            git_dir,
            handlers,
            pushes,
            host,
            replanning,
        )
        .await;
        match pushed {
            Some(back) => held = back,
            None => break 'round,
        }
        // A snapshot ahead is the one kind of work a person's own command waits for, so it runs before the upkeep
        //  and before every kind that frees space.
        match crate::service_work_drains_a_snapshot_ahead::drain_a_snapshot_ahead(
            held,
            work_dir.as_path(),
            ahead,
            handlers,
        )
        .await
        {
            Some(back) => held = back,
            None => break 'round,
        }
        // The housekeeping this round runs, and the items it leaves for a later round, are one step of their own
        //  (`service_work_drain_housekeeping_and_upkeep.rs`).
        match upkeep_the_held_items_awaiting(
            held,
            work_dir.as_path(),
            handlers,
            upkeep,
            &slice,
            &mut housekeeping_has_started,
            &mut left_for_a_later_round,
        )
        .await
        {
            Some(back) => held = back,
            None => break 'round,
        }
        // A new worktree's copied package state is relinked before anything else it asked for: it is the work the add
        //  stopped doing, and the person's next command in the worktree depends on it.
        let relinked: Vec<Claimed> =
            match still_held_awaiting(held, work_dir.as_path(), relinks).await {
                Some((back, mine)) => {
                    held = back;
                    mine
                }
                None => break 'round,
            };
        let relinks_wide: usize = slow_items_side_by_side(relinked.len(), host);
        match run_and_settle_side_by_side(
            held,
            work_dir.as_path(),
            handlers,
            relinked,
            relinks_wide,
            "relink",
            ask_relink,
        )
        .await
        {
            Some(back) => held = back,
            None => break 'round,
        }
        // A new worktree's Cargo build directory is filled before the left-out files are emptied: the filling is the work
        //  the add stopped doing, and the sooner it lands the more likely the first build in the worktree finds it.
        // The step runs in the waves the round may start, asking its bound as each item of a wave starts, and the items it
        //  may not start are the housekeeping a later round is named.
        let prewarmed: Vec<Claimed> =
            match still_held_awaiting(held, work_dir.as_path(), prewarms).await {
                Some((back, mine)) => {
                    held = back;
                    mine
                }
                None => break 'round,
            };
        let prewarms_wide: usize = slow_items_side_by_side(prewarmed.len(), host);
        let mut may_start_prewarm_now = || {
            return may_start_housekeeping(&slice, &mut housekeeping_has_started);
        };
        match run_and_settle_what_the_round_may_start(
            held,
            work_dir.as_path(),
            handlers,
            prewarmed,
            prewarms_wide,
            "prewarm",
            ask_prewarm,
            &mut may_start_prewarm_now,
            &mut left_for_a_later_round,
        )
        .await
        {
            Some(back) => held = back,
            None => break 'round,
        }
        // The review of a commit runs before the steps that free space: it is bounded by its own ceiling (`REVIEW_CEILING`), it
        //  decides nothing about any push, and on 2026-11-11 two reviews stayed claimed for a quarter of an hour while the round
        //  emptied the trashes of removed worktrees ahead of them, one of which can run for many minutes
        //  (`doc/planning/git-policy-cli-push-jams-2026-11-10.md`).
        let reviewed: Vec<Claimed> =
            match still_held_awaiting(held, work_dir.as_path(), reviews).await {
                Some((back, mine)) => {
                    held = back;
                    mine
                }
                None => break 'round,
            };
        let reviews_wide: usize = slow_items_side_by_side(reviewed.len(), host);
        match run_and_settle_side_by_side(
            held,
            work_dir.as_path(),
            handlers,
            reviewed,
            reviews_wide,
            "review",
            ask_review,
        )
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
                match delete_subvolumes_awaiting(held, work_dir.as_path(), deletions, handlers)
                    .await
                {
                    Some(back) => held = back,
                    None => break 'round,
                }
            }
        }
        // The left-out files of a snapshot-made worktree are emptied last, in the waves the round may start, asking its
        //  bound as each item of a wave starts: freeing them is a local job nothing waits for, and a trash of a removed
        //  worktree can take minutes (`worktree_trash_dispose.rs`). The items the round may not start are the housekeeping
        //  a later round is named.
        let trashed: Vec<Claimed> =
            match still_held_awaiting(held, work_dir.as_path(), trashes).await {
                Some((back, mine)) => {
                    held = back;
                    mine
                }
                None => break 'round,
            };
        let trashes_wide: usize = slow_items_side_by_side(trashed.len(), host);
        let mut may_start_trash_now = || {
            return may_start_housekeeping(&slice, &mut housekeeping_has_started);
        };
        match run_and_settle_what_the_round_may_start(
            held,
            work_dir.as_path(),
            handlers,
            trashed,
            trashes_wide,
            "trash",
            ask_trash,
            &mut may_start_trash_now,
            &mut left_for_a_later_round,
        )
        .await
        {
            Some(back) => held = back,
            None => break 'round,
        }
        // The housekeeping this round did not start goes back to `pending` for a later round, named here once, so a person
        //  reading the journal sees where their trash went and the service says when it looks again.
        if !left_for_a_later_round.is_empty() {
            leave_housekeeping_for_a_later_round(
                held,
                git_dir,
                work_dir.as_path(),
                &mut left_for_a_later_round,
                &slice,
            )
            .await;
            look_again = Some(LOOK_AGAIN_AFTER_DEFERRING);
            break 'round;
        }
    }
    // The renewal is stopped and waited for here, so a drain that returns leaves no task renaming a claim behind it.
    renewing.stop_awaiting().await;
    // When this look's caller must look at this directory again (`service_work_drain_look_again.rs`).
    return look_again_awaiting(work_dir.as_path(), lease, look_again).await;
}
