//! What:
//!  The service's work that can take minutes:
//! a lane of its own,
//! one repository at a time,
//! and inside one work directory one item per worktree at a time and different worktrees' items side by side,
//!  beside the round that serves the person's pushes.
//! Why:
//!  A round holds the after-commit work until it ends,
//! and a landing's push is claimed by the round that its wake starts,
//! so every step a round runs stands between a landing and its push.
//! On `2026-11-10` that was measured:
//! the round of the main checkout emptied trashes of removed worktrees for 18 minutes
//!  (`18:16:11` to `18:34:47`) while seven landings of `refs/heads/main` waited,
//! and the day holds four more such intervals,
//! the longest 4 946 s
//! (`doc/planning/git-policy-cli-push-jams-2026-11-10.md`).
//! The owner's rules this serves are "nothing should ever wait forever or indefinitely" and
//! "Do not disturb,
//! delay or slow other agents":
//! a step that can take minutes is waited for by nobody,
//! so it runs here and the round is left with the pushes
//!  (`PUSH_KINDS` and `SLOW_KINDS` in `service_work_handlers_replanning_and_acceptance.rs`).
//! One lane per service,
//! so one repository's records are drained by one task and a worktree's items are never taken twice;
//! inside that drain the items of different worktrees run beside each other
//! (`service_work_items_side_by_side.rs`),
//! and one worktree's trashes keep their turn
//! (`worktree_trash_dispose.rs`).
//! The claims are the drain's own:
//! the lane claims `SLOW_KINDS` and the round claims `PUSH_KINDS`,
//! so the two never take each other's records and no lock is needed.
//! What a person waits for is kept free of the lane's queue as well:
//! every storage step of these kinds asks through the gate that leaves one blocking place to the round
//! (`run_a_slow_job_within_the_budget` in `blocking_jobs_within_the_budget.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const lane = SlowLane.start(handlers, busy); lane.ask(wake); // for await (const wake of inbox) await drainSlow(wake.gitDir)
//! ```

use crate::housekeeping_of_one_round::HOUSEKEEPING_OF_ONE_ROUND;
use crate::service_claim_renewal_running::Renewal;
use crate::service_review_key::ReviewKey;
use crate::service_work::{CLAIM_LEASE, GitHandlers};
use crate::service_work_drain_pending_claims::drain_pending_awaiting;
use crate::service_work_handlers_replanning_and_acceptance::{Replanning, SLOW_KINDS, Wake, WorkHandlers};
use crate::service_work_storage_awaited::{LOOK_AGAIN_AFTER_A_STEP_THAT_GAVE_NO_ANSWER, common_directory_to_drain_awaiting, refused};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

/// What:
///  The way the after-commit loop asks the lane to work on a repository.
/// Why:
///   The lane is a task of the process's runtime,
/// so what reaches it is a message and never a call:
/// the loop sends the wake it was given,
/// with the review credential it carried,
///  and goes on to run the round that serves the pushes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SlowLane { ask(wake: Wake): void }
/// ```
pub struct SlowLane {
    /// Where the lane takes its work;
    /// dropped with the lane,
    /// which ends the task.
    inbox: UnboundedSender<Wake>,
}

impl SlowLane {
    /// What:
    ///  Start the lane on the runtime this call is made from,
    /// with a copy of `handlers` that holds no credential of its own.
    /// Why:
    ///   The lane outlives every round,
    /// so it may not borrow the handlers of one:
    /// a copy is made here and a copy per request carries the credential of that request's wake,
    /// which the lane drops when the request is over.
    /// The lane counts itself busy for as long as it works,
    /// so the service does not exit idle while a claim of it is held.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const lane = SlowLane.start(handlers, busy); // tokio.spawn(run(lane, handlers.copy(), busy))
    /// ```
    pub fn start(handlers: &GitHandlers, busy: Arc<AtomicUsize>) -> SlowLane {
        return SlowLane::start_with_slice(handlers, busy, HOUSEKEEPING_OF_ONE_ROUND);
    }

    /// What:
    ///  Start the lane with the housekeeping one look at a repository may start given,
    /// which the shipped service takes from `HOUSEKEEPING_OF_ONE_ROUND`.
    /// Why:
    ///   A control states a slice it can spend in a moment,
    /// so the bound that separates one look from the next is a state the control reaches instead of a duration it waits out.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const lane = SlowLane.startWithSlice(handlers, busy, slice);
    /// ```
    fn start_with_slice(handlers: &GitHandlers, busy: Arc<AtomicUsize>, housekeeping: Duration) -> SlowLane {
        let (inbox, taken): (UnboundedSender<Wake>, UnboundedReceiver<Wake>) = unbounded_channel::<Wake>();
        let owned: GitHandlers = handlers.for_round(std::collections::HashMap::<PathBuf, ReviewKey>::new());
        let _running: tokio::task::JoinHandle<()> = tokio::spawn(run_awaiting(taken, owned, busy, housekeeping));
        return SlowLane { inbox };
    }

    /// Ask the lane to look at the repository `wake` names.
    /// A lane that ended is a service that is stopping:
    /// the records keep their places and the service started next drains them.
    pub fn ask(&self, wake: Wake) {
        let _asked: Result<(), tokio::sync::mpsc::error::SendError<Wake>> = self.inbox.send(wake);
    }
}

/// What:
///  The lane itself:
/// take the wake it was given,
/// then every wake already queued behind it,
/// look at each repository once,
/// and wait for the next.
/// Why:
///   One look per repository and not one per wake keeps a burst of landings to one pass over the directory,
/// as the round does.
/// A wake that arrives while a step runs waits in the channel and is served when that step ends,
/// which is the bound the lane exists for:
/// a push never waits here,
/// because a push is never claimed here.
///  A look that could not start every housekeeping item it found leaves the rest `pending` and asks to be looked at again
///  (`housekeeping_of_one_round.rs`),
/// so one repository's backlog is taken in slices:
/// the repositories queued with it are looked at first,
/// and it is looked at again after the pause the drain asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const wake of inbox) { busy.add(1); let left = [wake, ...drain(inbox)]; while (left.length) { left = await lookOnce(housekeeping, left) } busy.sub(1) }
/// ```
async fn run_awaiting(mut inbox: UnboundedReceiver<Wake>, handlers: GitHandlers, busy: Arc<AtomicUsize>, housekeeping: Duration) {
    loop {
        let Some(first) = inbox.recv().await else { return };
        busy.fetch_add(1, Ordering::SeqCst);
        let mut wakes: Vec<Wake> = vec![first];
        while let Ok(queued) = inbox.try_recv() {
            wakes.push(queued);
        }
        while !wakes.is_empty() {
            let mut left: Vec<Wake> = Vec::new();
            let mut soonest: Option<Duration> = None;
            let mut looked_at: Vec<PathBuf> = Vec::new();
            for (git_dir, review_key) in std::mem::take(&mut wakes) {
                if looked_at.contains(&git_dir) {
                    continue;
                }
                looked_at.push(git_dir.clone());
                let mut keys: std::collections::HashMap<PathBuf, ReviewKey> = std::collections::HashMap::new();
                if let Some(key) = review_key.clone() {
                    keys.insert(git_dir.clone(), key);
                }
                if let Some(wait) = drain_the_slow_work_awaiting(git_dir.as_path(), &handlers.for_round(keys), housekeeping).await {
                    soonest = Some(match soonest {
                        Some(already) => already.min(wait),
                        None => wait,
                    });
                    left.push((git_dir, review_key));
                }
            }
            // Nothing was left for a later look, so the lane waits for the next wake.
            let Some(wait) = soonest else { break };
            tokio::time::sleep(wait).await;
            // A wake that arrived during the look is served with the repository that asked to be looked at again.
            wakes = left;
            while let Ok(queued) = inbox.try_recv() {
                wakes.push(queued);
            }
        }
        busy.fetch_sub(1, Ordering::SeqCst);
    }
}

/// What:
///  Run every slow kind that waits for this repository,
/// for the directory itself and for the common directory of a bare repository,
/// as the round runs the pushes,
/// and answer how soon it asks to be looked at again.
/// Why:
///   The records of a bare repository's worktrees sit in its common directory's work directory,
/// and the round reaches that directory too (`drain_common_records_planned_awaiting`),
/// so the lane must reach it or those records are never run.
/// The look for that directory reads its `commondir` file and its `config`,
/// which is a job of the process's blocking pool with a ceiling,
/// so this task never waits on a store that stopped answering (finding 1 of the review of `c02f49638e`).
/// A look that gave no answer is not "there is none": the lane asks to be woken for it again.
/// A directory whose look left housekeeping for a later one names the pause it wants
/// (`housekeeping_of_one_round.rs`),
/// and the earlier of the two looks is what the lane waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await drainSlow(gitDir); if (bareCommon(gitDir)) await drainSlow(common)
/// ```
pub(crate) async fn drain_the_slow_work_awaiting<H: WorkHandlers + Sync + ?Sized>(git_dir: &Path, handlers: &H, housekeeping: Duration) -> Option<Duration> {
    let own: Option<Duration> = drain_the_slow_kinds_awaiting(git_dir, handlers, housekeeping).await;
    let common: PathBuf = match common_directory_to_drain_awaiting(git_dir).await {
        Ok(Some(common)) => common,
        // Nothing to run there: this repository has no bare common directory whose records the lane would run.
        Ok(None) => return own,
        Err(said) => {
            refused(said.as_str());
            return Some(own.unwrap_or(LOOK_AGAIN_AFTER_A_STEP_THAT_GAVE_NO_ANSWER));
        }
    };
    let shared: Option<Duration> = drain_the_slow_kinds_awaiting(common.as_path(), handlers, housekeeping).await;
    return match (own, shared) {
        (Some(one), Some(other)) => Some(one.min(other)),
        (one, other) => one.or(other),
    };
}

/// What:
///  One drain of the slow kinds of one directory,
/// and what it left for a later one.
/// Why:
///   The drain of the pushes plans the pushes that are tried again,
/// and the plan belongs to the loop that owns it;
/// a drain here asks nothing of it,
/// so it is given a plan that keeps nothing and its answer is dropped unread.
/// Every slow kind settles itself and none of them reads the plan
///  (`service_work_drain_pending_claims.rs`).
///  A look starts housekeeping for no longer than the slice it was given,
/// and says when to look again when it left some of it for a later one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await drainPendingAwaiting(gitDir, handlers, CLAIM_LEASE, Renewal.Timed, { first: undefined, changes: [], plans: undefined }, SLOW_KINDS, housekeeping)
/// ```
async fn drain_the_slow_kinds_awaiting<H: WorkHandlers + Sync + ?Sized>(git_dir: &Path, handlers: &H, housekeeping: Duration) -> Option<Duration> {
    let mut nothing_to_plan: Replanning = Replanning { first: None, changes: Vec::new(), plans: None };
    return drain_pending_awaiting(git_dir, handlers, CLAIM_LEASE, Renewal::Timed, &mut nothing_to_plan, &SLOW_KINDS, housekeeping).await;
}

/// Controls for the lane stay out of the release executable.
#[cfg(test)]
#[path = "service_work_slow_lane_tests.rs"]
mod tests;
