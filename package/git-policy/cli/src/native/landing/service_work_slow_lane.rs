//! What:
//!  The service's work that can take minutes:
//! a lane of its own,
//! one repository at a time and one step at a time,
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
//! so one repository's trash is still emptied one at a time
//! (`worktree_trash_dispose.rs`),
//! and the claims are the drain's own:
//! the lane claims `SLOW_KINDS` and the round claims `PUSH_KINDS`,
//! so the two never take each other's records and no lock is needed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const lane = SlowLane.start(handlers, busy); lane.ask(wake); // for await (const wake of inbox) await drainSlow(wake.gitDir)
//! ```

use crate::service_claim_renewal_running::Renewal;
use crate::service_review_key::ReviewKey;
use crate::service_work::{CLAIM_LEASE, GitHandlers};
use crate::service_work_drain_pending_claims::drain_pending_awaiting;
use crate::service_work_handlers_replanning_and_acceptance::{
    Replanning, SLOW_KINDS, Wake, WorkHandlers, common_directory_of, is_bare_common_directory, work_directory,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
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
        let (inbox, taken): (UnboundedSender<Wake>, UnboundedReceiver<Wake>) = unbounded_channel::<Wake>();
        let owned: GitHandlers = handlers.for_round(std::collections::HashMap::<PathBuf, ReviewKey>::new());
        let _running: tokio::task::JoinHandle<()> = tokio::spawn(run_awaiting(taken, owned, busy));
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const wake of inbox) { busy.add(1); for (const one of [wake, ...drain(inbox)]) await lookOnce(one); busy.sub(1) }
/// ```
async fn run_awaiting(mut inbox: UnboundedReceiver<Wake>, handlers: GitHandlers, busy: Arc<AtomicUsize>) {
    loop {
        let Some(first) = inbox.recv().await else { return };
        busy.fetch_add(1, Ordering::SeqCst);
        let mut wakes: Vec<Wake> = vec![first];
        while let Ok(queued) = inbox.try_recv() {
            wakes.push(queued);
        }
        let mut looked_at: Vec<PathBuf> = Vec::new();
        for (git_dir, review_key) in wakes {
            if looked_at.contains(&git_dir) {
                continue;
            }
            looked_at.push(git_dir.clone());
            let mut keys: std::collections::HashMap<PathBuf, ReviewKey> = std::collections::HashMap::new();
            if let Some(key) = review_key {
                keys.insert(git_dir.clone(), key);
            }
            drain_the_slow_work_awaiting(git_dir.as_path(), &handlers.for_round(keys)).await;
        }
        busy.fetch_sub(1, Ordering::SeqCst);
    }
}

/// What:
///  Run every slow kind that waits for this repository,
/// for the directory itself and for the common directory of a bare repository,
/// as the round runs the pushes.
/// Why:
///   The records of a bare repository's worktrees sit in its common directory's work directory,
/// and the round reaches that directory too (`drain_common_records_planned_awaiting`),
/// so the lane must reach it or those records are never run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await drainSlow(gitDir); if (bareCommon(gitDir)) await drainSlow(common)
/// ```
pub(crate) async fn drain_the_slow_work_awaiting<H: WorkHandlers + Sync + ?Sized>(git_dir: &Path, handlers: &H) {
    drain_the_slow_kinds_awaiting(git_dir, handlers).await;
    let Some(common) = common_directory_of(git_dir) else { return };
    if common == git_dir || !is_bare_common_directory(common.as_path()) || !work_directory(common.as_path()).is_dir() {
        return;
    }
    drain_the_slow_kinds_awaiting(common.as_path(), handlers).await;
}

/// What:
///  One drain of the slow kinds of one directory.
/// Why:
///   The drain of the pushes plans the pushes that are tried again,
/// and the plan belongs to the loop that owns it;
/// a drain here asks nothing of it,
/// so it is given a plan that keeps nothing and its answer is dropped unread.
/// Every slow kind settles itself and none of them reads the plan
///  (`service_work_drain_pending_claims.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await drainPendingAwaiting(gitDir, handlers, CLAIM_LEASE, Renewal.Timed, { first: undefined, changes: [], plans: undefined }, SLOW_KINDS)
/// ```
async fn drain_the_slow_kinds_awaiting<H: WorkHandlers + Sync + ?Sized>(git_dir: &Path, handlers: &H) {
    let mut nothing_to_plan: Replanning = Replanning { first: None, changes: Vec::new(), plans: None };
    drain_pending_awaiting(git_dir, handlers, CLAIM_LEASE, Renewal::Timed, &mut nothing_to_plan, &SLOW_KINDS).await;
}

/// Controls for the lane stay out of the release executable.
#[cfg(test)]
#[path = "service_work_slow_lane_tests.rs"]
mod tests;
