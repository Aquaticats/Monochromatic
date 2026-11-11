//! What:
//!  Publishing a landed commit when the remote branch advanced after the commit landed,
//! from a command that is not a task of the runtime:
//!  the plan is read here,
//! the loop that does the work is the awaited one
//! (`auto_push_reconcile_awaiting.rs`),
//!  and the merge push this side supplied runs on a thread of its own while the loop awaits its
//!  answer.
//! Why:
//!  A rejected `push` of a branch that the remote moved cannot succeed again,
//! and an ordinary merge in the active
//!  worktree can be blocked,
//! or made unsafe,
//! by another consumer's uncommitted files.
//!  `git merge-tree --write-tree` and `git commit-tree` read and write only objects,
//! so the active worktree,
//!  its index and its untracked files are never touched while merging,
//! and both histories stay reachable from the
//!  published tip.
//!  Only after the push succeeds is the branch fast-forwarded,
//! and only where nothing can be lost:
//! a dirty worktree
//!  follows too when the move changes none of the files its uncommitted work touches.
//!  An amended or rebased branch is merged like any other:
//! nothing already published is lost,
//! and the replaced
//!  commits stay in the remote history beside the new ones.
//!  The reconcile of an explicit `git push` is here rather than in the awaited file because its
//!  merge push is `gated_merge_push`,
//! which plans and runs a real Git push synchronously
//! (`push_stacking.rs`);
//! that push is a start of Git,
//! and a start of Git from inside the runtime is refused.
//!  So this file holds no decision of the reconcile:
//! it hands the one loop a merge push it can await,
//! and that merge push calls back into this side on a plain thread,
//! never a job of the blocking pool,
//! because a job of the pool is inside the runtime and would refuse the same start
//! (`doc/planning/git-policy-cli-full-tokio.md`).

/// The coordinator's Git,
/// its command builder,
/// queries and error type.
use crate::auto_push_single_flight::{
    PushAttempt, PushGit, SingleFlightError,
};
/// The auto-push's bounded `git push` with the Git LFS upload.
use crate::lfs_bounded_push::run_push;
/// The diagnostic log of the wrapper.
use crate::diagnostic_log::debug;
/// The reconcile loop that is the only body of these decisions,
/// and the seam a task publishes its merge through.
use crate::auto_push_reconcile_awaiting::{MergePusherAwaiting, integrate_and_push_awaiting};
/// What one reconcile run integrates,
/// how it words itself and how it ended,
/// shared with the loop.
pub use crate::auto_push_reconcile_plan::{Integrated, Reconcile};

/// Where the plain push goes,
/// as `auto_push_destination` decides it.
pub use crate::auto_push_destination::Destination;
/// The destination of the plain push of a branch,
/// from the settings alone.
use crate::auto_push_destination::read_destination;

/// The failed-attempt value a control of the loop still builds.
#[cfg(test)]
use crate::auto_push_reconcile_basics_and_limits::unpublished;
/// The synchronous merge commit the control that compares the two faces still asks.
#[cfg(test)]
use crate::auto_push_reconcile_merge_commit::commit_merge;
/// The synchronous branch move the control that compares the two faces still asks.
#[cfg(test)]
use crate::auto_push_reconcile_branch_advance::advance_local_branch;
use crate::auto_push_reconcile_basics_and_limits::{TAG, argv};
pub use crate::auto_push_reconcile_basics_and_limits::{
    MAX_RECONCILE_ATTEMPTS, RECONCILE_FAILED_PREFIX, RECONCILE_LINE_PREFIX, journal_identifier, was_rejected_as_stale,
};
/// A future the loop awaits,
/// and the box it travels in.
use std::future::Future;
use std::pin::Pin;
/// The scope the bridge runs in,
/// and the handle of the thread it starts.
use std::thread::{Scope, ScopedJoinHandle};
/// The one place that makes the callable `std::thread::scope` accepts,
/// and the body it runs.
use crate::side_by_side_threads::{ScopeBody, in_thread_scope};

/// What:
///  What publishes one merge commit to a destination.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   The loop takes its push as a value so controls can stand in for it;
/// the state a push needs (the Git context,
/// the tokens,
/// the
///  planner) is a small struct that implements this.
///  A pusher travels to the thread that calls it,
/// so it must be sendable there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface MergePusher { pushMerge(destination: Destination, mergeOid: string): PushAttempt }
/// ```
pub trait MergePusher: Send {
    /// Push the merge commit `merge_oid` to `destination`.
    fn push_merge(&mut self, destination: &Destination, merge_oid: &str) -> Result<PushAttempt, SingleFlightError>;
}

/// What:
///  The automatic push of a merge:
/// plain `git push` of the merge commit to the destination,
/// through the push coordinator.
/// Why:
///   The auto-push reconcile has only the Git context to carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PushMergeThroughGit implements MergePusher { pushMerge(target, mergeOid) { return runPush(git, ['push', target.remote, `${mergeOid}:${target.mergeRef}`]); } }
/// ```
struct PushMergeThroughGit<'a> {
    /// The coordinator's Git.
    git: &'a PushGit,
}

impl MergePusher for PushMergeThroughGit<'_> {
    fn push_merge(&mut self, target: &Destination, merge_oid: &str) -> Result<PushAttempt, SingleFlightError> {
        let arguments = argv(&[
            "push",
            &target.remote,
            &format!("{merge_oid}:{}", target.merge_ref),
        ]);
        return run_push(self.git, &arguments);
    }
}

/// What:
///  One merge the loop asks the thread of this side to push,
/// and where that thread puts the answer.
/// Why:
///   The push a command outside the runtime supplied runs on a thread of its own,
/// and the loop awaits its answer as a task;
/// this message is the whole of what crosses between them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MergePushAsked = { destination: Destination; mergeOid: string; answer: (pushed: PushAttempt) => void };
/// ```
struct MergePushAsked {
    /// Where the merge is published.
    destination: Destination,
    /// The merge commit to publish.
    merge_oid: String,
    /// Where the push's answer goes,
    /// or nothing when the loop stopped waiting.
    answer: tokio::sync::oneshot::Sender<Result<PushAttempt, SingleFlightError>>,
}

/// What:
///  The merge push of a command outside the runtime,
/// awaited by the loop as a task.
/// Why:
///   That push starts Git synchronously and cannot be called from a task,
/// so it runs on the plain thread `integrate_and_push` starts,
/// and the loop awaits the answer here.
///  An answer that never comes is a failure and not a wait:
/// a thread that ended or a loop that stopped waiting closes the channel,
/// and the loop then reports what it knows instead of holding the push for good.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class MergePushedByAPlainThread implements MergePusherAwaiting { async pushMerge(target, mergeOid) { return await askedOverTheChannel(target, mergeOid); } }
/// ```
struct MergePushedByAPlainThread {
    /// The thread of this side,
    /// and the channel its asks travel on.
    asks: std::sync::mpsc::Sender<MergePushAsked>,
}

impl MergePusherAwaiting for MergePushedByAPlainThread {
    fn push_merge_awaiting<'a>(
        &'a mut self,
        destination: &'a Destination,
        merge_oid: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<PushAttempt, SingleFlightError>> + Send + 'a>> {
        return Box::pin(async move {
            let (answer, answered) = tokio::sync::oneshot::channel();
            let asked = MergePushAsked {
                destination: destination.clone(),
                merge_oid: merge_oid.to_owned(),
                answer,
            };
            if self.asks.send(asked).is_err() {
                return Err(SingleFlightError::Start(std::io::Error::other(
                    "the thread that pushes the merge of this reconcile is gone, so the merge was not pushed and nothing was \
                     published; the push's own report stands",
                )));
            }
            return match answered.await {
                Ok(pushed) => pushed,
                Err(_ended) => Err(SingleFlightError::Start(std::io::Error::other(
                    "the thread that pushes the merge of this reconcile ended without an answer, so what became of the merge is \
                     not known; nothing was force-pushed and the branch was not moved",
                ))),
            };
        });
    }
}

/// What:
///  Push what the loop asks for,
/// one merge after another,
/// until the loop is done.
/// Why:
///   The thread exists for the length of one reconcile run and no longer:
/// the channel closes when the loop drops its end,
/// which is what ends this loop and lets the scope of `integrate_and_push` finish.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const asked of asks) asked.answer(pushMerge(asked));
/// ```
fn push_what_the_loop_asks(asked: std::sync::mpsc::Receiver<MergePushAsked>, push_merge: &mut dyn MergePusher) {
    for ask in asked {
        let pushed: Result<PushAttempt, SingleFlightError> = push_merge.push_merge(&ask.destination, ask.merge_oid.as_str());
        // A loop that stopped waiting has nowhere to put the answer, and the push itself is done either way.
        let _told = ask.answer.send(pushed);
    }
}

/// Whether a control refused the threads of this thread,
/// as a system with no thread to give does.
#[cfg(test)]
fn a_control_refuses_the_merge_push_thread() -> bool {
    return crate::scoped_start::STARTS_FAIL_ON_THIS_THREAD.with(std::cell::Cell::get);
}

/// Whether a control refused the merge-push thread;
/// nothing outside the controls refuses one.
#[cfg(not(test))]
fn a_control_refuses_the_merge_push_thread() -> bool {
    return false;
}

/// What:
///  The work of `integrate_and_push` inside one thread scope:
/// it starts the thread that carries the caller's merge push,
/// runs the one reconcile loop on the runtime,
/// and answers how the run ended.
/// Why:
///   The scope is what lets a plain thread borrow the caller's pusher,
/// the plan and the rejected attempt;
/// `side_by_side_threads` is the one place that makes the callable
/// `std::thread::scope` accepts,
/// so no closure of this file carries them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class BridgedReconcile implements ScopeBody<Result<Integrated, SingleFlightError>> { run(scope) { ... } }
/// ```
struct BridgedReconcile<'run> {
    /// The coordinator's Git.
    git: &'run PushGit,
    /// What the run integrates.
    plan: &'run Reconcile<'run>,
    /// The rejected attempt the run starts from.
    rejected: PushAttempt,
    /// The merge push of the command that asked for the run.
    push_merge: &'run mut dyn MergePusher,
}

impl<'env> ScopeBody<'env> for BridgedReconcile<'env> {
    type Output = Result<Integrated, SingleFlightError>;

    /// Start the merge push's thread,
    /// then run the loop on the runtime and wait for it.
    fn run<'scope>(self, scope: &'scope Scope<'scope, 'env>) -> Self::Output {
        let BridgedReconcile { git, plan, rejected, push_merge } = self;
        let (asks, asked) = std::sync::mpsc::channel::<MergePushAsked>();
        #[cfg_attr(
            dylint_lib = "monochromatic_lint_closure",
            expect(
                avoidable_closure,
                reason = "`Builder::spawn_scoped` accepts only a callable, and it must carry the channel and the pusher of the caller; see package/git-policy/cli/doc/closure-lint-suppressions.md"
            )
        )]
        let started: std::io::Result<ScopedJoinHandle<'scope, ()>> = if a_control_refuses_the_merge_push_thread() {
            Err(std::io::Error::from_raw_os_error(libc::EAGAIN))
        } else {
            std::thread::Builder::new()
                .name(String::from("git-policy-merge-push"))
                .spawn_scoped(scope, move || push_what_the_loop_asks(asked, push_merge))
        };
        if let Err(refused) = started {
            debug(
                TAG,
                &format!(
                    "{}; nothing was pushed or changed, and the push's own report stands",
                    crate::scoped_start::refused_thread_sentence("push the merge of a reconcile", &refused)
                ),
            );
            return Ok(Integrated::NotDivergent(rejected));
        }
        let mut awaited: MergePushedByAPlainThread = MergePushedByAPlainThread { asks };
        let answered = crate::lazy_runtime::run_on_runtime(integrate_and_push_awaiting(git, plan, rejected, &mut awaited));
        // The thread ends as soon as this side's end of the channel goes, and the scope waits for it here.
        drop(awaited);
        return match answered {
            Ok(integrated) => integrated,
            Err(refusal) => Err(SingleFlightError::Start(std::io::Error::other(refusal.to_string()))),
        };
    }
}

/// What:
///  Run the one reconcile loop on this process's runtime,
/// with the merge push this side supplied carried out on a thread of its own.
/// Why:
///   The loop is a task's work,
/// and a command outside the runtime reaches it by blocking this thread on the runtime once
/// (`run_on_runtime`),
/// as every other synchronous face of the wrapper does.
///  The merge push cannot travel with the loop:
/// it starts Git synchronously,
/// and that is refused from inside the runtime.
/// It therefore runs on a plain thread beside the loop,
/// which is the whole bridge,
/// and that thread ends with the run.
///  A thread the system refuses is not run here:
/// the push's own report then stands as it was,
/// and the branch,
/// the worktree and the remote are exactly as the failed push left them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function integrateAndPush({ git, plan, rejected, pushMerge }): Promise<Integrated>;
/// ```
pub fn integrate_and_push(
    git: &PushGit,
    plan: &Reconcile<'_>,
    rejected: PushAttempt,
    push_merge: &mut dyn MergePusher,
) -> Result<Integrated, SingleFlightError> {
    return in_thread_scope(BridgedReconcile { git, plan, rejected, push_merge });
}

/// What:
///  Merge the remote tip into the pushed branch's tip and publish the result,
/// from a command that is not a task of the runtime.
/// Why:
///   Another worktree advancing the remote makes the plain push fail for good,
/// while both histories are valid and
///  must both be kept;
/// the merge is published as a new commit above the local tip,
/// so the landed commit stays
///  reachable and nothing is forced.
///   Once the merge is published,
/// the branch and its worktree are fast-forwarded to it when that cannot lose or
///  overwrite anything;
/// otherwise they are left as they are and the report says why.
///   A push destination Git cannot name,
/// a conflict,
/// an exhausted retry budget and every failure of a command are
///  reported,
/// and only the first leaves the original rejection unchanged.
///   `local` is the commit the push was allowed to send,
/// fixed before the worktree's verdicts were read:
/// merging it, and never
///  the branch's reference, is what keeps a commit that landed after that reading out of the merge this publishes, because
///  that commit's own gate may still refuse it.
/// A branch that moved since is left where it is and the report says why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function reconcileDivergentPush({ git, branchRef, local, rejected }): Promise<PushAttempt>;
/// ```
pub fn reconcile_divergent_push(
    git: &PushGit,
    branch_ref: &str,
    local: &str,
    rejected: PushAttempt,
) -> Result<PushAttempt, SingleFlightError> {
    if !was_rejected_as_stale(&rejected.output) {
        return Ok(rejected);
    }
    let Some(destination) = read_destination(git, branch_ref)? else {
        debug(
            TAG,
            "the push destination cannot be resolved to a remote branch; no reconcile",
        );
        return Ok(rejected);
    };
    let plan = Reconcile {
        destination,
        local: local.to_owned(),
        branch_ref: Some(branch_ref),
        label: "auto-push",
        explicit: false,
    };
    // The loop takes its push as a value so controls can stand in for it; this one carries the Git context.
    let mut push_merge: PushMergeThroughGit<'_> = PushMergeThroughGit { git };
    return Ok(match integrate_and_push(git, &plan, rejected, &mut push_merge)? {
        Integrated::NotDivergent(attempt) | Integrated::Finished(attempt) => attempt,
    });
}

/// Reconcile behavior is controlled in the test module.
#[cfg(test)]
#[path = "auto_push_reconcile_tests.rs"]
mod tests;
