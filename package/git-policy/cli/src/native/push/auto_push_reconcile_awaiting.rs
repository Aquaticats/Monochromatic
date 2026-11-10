//! What:
//!  The reconcile of a rejected push,
//! awaited by a caller that is itself a task of the runtime:
//! the loop that fetches the remote tip,
//! merges it into the local tip,
//! publishes the merge and
//!  moves the branch.
//! Why:
//!  Every step of the loop starts Git,
//! and a synchronous start is refused from inside the runtime,
//! so the landing round's push cannot use the synchronous loop.
//!  The loop's own decisions,
//! its wording,
//! its attempt limit and the bookkeeping of a refusal are the synchronous loop's,
//! so the two cannot publish or say different things.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const integrated = await integrateAndPushOnTheRuntime({ git, plan, rejected, pushMerge });
//! ```

use crate::auto_push_reconcile::{Integrated, Reconcile, wording_of};
use crate::auto_push_reconcile_basics_and_limits::{
    MAX_RECONCILE_ATTEMPTS, TAG, argv, fetch_remote_tip_awaiting, is_ancestor_awaiting, journal_identifier,
    unpublished, was_rejected_as_stale,
};
use crate::auto_push_reconcile_branch_advance::advance_local_branch_awaiting;
use crate::auto_push_reconcile_merge_commit::commit_merge_awaiting;
use crate::auto_push_single_flight::{PushAttempt, PushGit, SingleFlightError, git_output_awaiting};
use crate::auto_push_destination::{Destination, read_destination_awaiting};
use crate::diagnostic_log::debug;
use crate::lfs_bounded_push::run_push_awaiting;
use crate::merge_second_pass_tree::announce;
use crate::push_retry::CONFLICT_PHRASE;
use std::ffi::OsString;
use std::future::Future;
use std::pin::Pin;

/// What:
///  What publishes one merge commit to a destination,
/// awaited by a caller that is itself a task of the runtime.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   The loop takes its push as a value so controls can stand in for it;
/// a push that starts Git from a task must hand back a future,
/// because a synchronous start is refused from inside the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface MergePusherAwaiting { pushMerge(destination: Destination, mergeOid: string): Promise<PushAttempt> }
/// ```
pub trait MergePusherAwaiting: Send {
    /// Push the merge commit `merge_oid` to `destination`.
    fn push_merge_awaiting<'a>(
        &'a mut self,
        destination: &'a Destination,
        merge_oid: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<PushAttempt, SingleFlightError>> + Send + 'a>>;
}

/// What:
///  The automatic push of a merge,
/// awaited by a caller that is itself a task of the runtime:
/// plain `git push` of the merge commit to the destination,
/// through the push coordinator.
/// Why:
///   The reconcile has only the Git context to carry,
/// and the push it makes is the awaited one,
/// with the same arguments and the same Git LFS step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PushMergeThroughGitAwaiting implements MergePusherAwaiting { async pushMerge(target, mergeOid) { return await runPushAwaiting(git, ['push', target.remote, `${mergeOid}:${target.mergeRef}`]); } }
/// ```
pub struct PushMergeThroughGitAwaiting<'a> {
    /// The coordinator's Git.
    pub git: &'a PushGit,
}

impl MergePusherAwaiting for PushMergeThroughGitAwaiting<'_> {
    fn push_merge_awaiting<'a>(
        &'a mut self,
        target: &'a Destination,
        merge_oid: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<PushAttempt, SingleFlightError>> + Send + 'a>> {
        let arguments: Vec<OsString> = argv(&[
            "push",
            &target.remote,
            &format!("{merge_oid}:{}", target.merge_ref),
        ]);
        return Box::pin(async move { return run_push_awaiting(self.git, &arguments).await; });
    }
}

/// What:
///  The same reconcile loop,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   Every step below it is awaited,
/// so the loop holds no worker while Git runs;
/// the decisions,
/// the wording and the limits are the synchronous loop's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function integrateAndPushOnTheRuntime({ git, plan, rejected, pushMerge }): Promise<Integrated>;
/// ```
pub async fn integrate_and_push_awaiting(
    git: &PushGit,
    plan: &Reconcile<'_>,
    rejected: PushAttempt,
    push_merge: &mut dyn MergePusherAwaiting,
) -> Result<Integrated, SingleFlightError> {
    let destination: Destination = plan.destination.clone();
    let local: String = plan.local.clone();
    let label: &str = plan.label;
    let wording = wording_of(plan);
    let line_prefix: &str = wording.line_prefix.as_str();
    let failed_prefix: &str = wording.failed_prefix.as_str();
    let short: &str = wording.short.as_str();
    let merged_into: &str = wording.merged_into.as_str();
    let message: &str = wording.message.as_str();
    let mut previous: String = rejected.output.clone();
    // The remote tip a merge was pushed against, with the refusal it got: a remote that still shows that tip did not
    // move, so the refusal was a rule and not a race.
    let mut refused_against: Option<(String, PushAttempt)> = None;
    for attempt in 1..=MAX_RECONCILE_ATTEMPTS {
        let remote_tip: String = match (fetch_remote_tip_awaiting(git, &destination)).await? {
            Ok(tip) => tip,
            Err(failed) => {
                if plan.explicit && refused_against.is_none() {
                    debug(
                        TAG,
                        "the remote could not be fetched after an explicit push failed; its own report stands",
                    );
                    return Ok(Integrated::NotDivergent(rejected));
                }
                return Ok(Integrated::Finished(unpublished(
                    failed_prefix,
                    &format!("{previous}\n{}", failed.output),
                    failed.exit_code,
                    &format!(
                        "{}/{short} has commits this branch lacks, and fetching them failed; nothing was pushed and the worktree was not touched.",
                        destination.remote
                    ),
                )));
            }
        };
        if let Some((tip, refusal)) = refused_against.take()
            && tip == remote_tip
        {
            debug(
                TAG,
                "the remote did not move after it refused the merge; the refusal is final",
            );
            return Ok(Integrated::Finished(refusal));
        }
        match (
            (is_ancestor_awaiting(git, &remote_tip, &local)).await?,
            (is_ancestor_awaiting(git, &local, &remote_tip)).await?,
        ) {
            (Some(true), _) => {
                debug(
                    TAG,
                    "the remote tip is behind the branch; the rejection was not a divergence",
                );
                return Ok(Integrated::NotDivergent(rejected));
            }
            (_, Some(true)) if plan.explicit => {
                debug(
                    TAG,
                    "the remote already contains the pushed commit; the explicit push failed for another reason",
                );
                return Ok(Integrated::NotDivergent(rejected));
            }
            (_, Some(true)) => {
                return Ok(Integrated::Finished(PushAttempt {
                    exit_code: 0,
                    output: format!(
                        "{line_prefix}{}/{short} already contains {local}; nothing to push.",
                        destination.remote
                    ),
                }));
            }
            (Some(false), Some(false)) => {}
            _ => {
                if plan.explicit {
                    return Ok(Integrated::NotDivergent(rejected));
                }
                return Ok(Integrated::Finished(unpublished(
                    failed_prefix,
                    &previous,
                    rejected.exit_code,
                    "Git could not relate the branch to the remote tip; nothing was pushed and the worktree was not touched.",
                )));
            }
        }
        let committed = match commit_merge_awaiting(git, &local, &remote_tip, message).await? {
            Ok(found) => found,
            Err(paths) => {
                return Ok(Integrated::Finished(unpublished(
                    failed_prefix,
                    &previous,
                    rejected.exit_code,
                    &format!(
                        "merging {}/{short} into {merged_into}{CONFLICT_PHRASE}{}. Nothing was pushed and the worktree was not touched; \
                         merge {}/{short} in a separate worktree, resolve the conflicts and push.",
                        destination.remote,
                        paths.join(", "),
                        destination.remote
                    ),
                )));
            }
        };
        let merge_oid: String = committed.oid.clone();
        debug(
            TAG,
            &format!(
                "attempt {attempt}: pushing the merge {merge_oid} of {local} and {remote_tip}"
            ),
        );
        let pushed: PushAttempt = (push_merge.push_merge_awaiting(&destination, &merge_oid)).await?;
        if pushed.exit_code == 0 {
            let branch: &str = match plan.branch_ref {
                Some(reference) => reference.strip_prefix("refs/heads/").unwrap_or(reference),
                None => "HEAD",
            };
            let advanced = match plan.branch_ref {
                Some(branch_ref) => (advance_local_branch_awaiting(git, branch_ref, &local, &merge_oid)).await?,
                None => Err(String::from("HEAD is detached")),
            };
            let followed: String = match advanced {
                Ok(false) => format!("{branch} and the worktree were fast-forwarded to it."),
                Ok(true) => format!(
                    "{branch} and the worktree were fast-forwarded to it; your uncommitted changes were left as they are."
                ),
                Err(reason) => {
                    debug(TAG, &format!("{branch} was not fast-forwarded: {reason}"));
                    format!(
                        "{branch} itself and the worktree are unchanged ({reason}); update {branch} with `git merge --ff-only {}/{short}` \
                         when the worktree allows.",
                        destination.remote
                    )
                }
            };
            let mut merged_parts: String = String::new();
            for line in announce(line_prefix, (journal_identifier(plan.explicit), label), &committed.merged) {
                merged_parts.push_str(format!("{line}\n").as_str());
            }
            return Ok(Integrated::Finished(PushAttempt {
                exit_code: 0,
                output: format!(
                    "{}\n{merged_parts}{line_prefix}{}/{short} had new commits, so the merge {merge_oid} of {branch} and {}/{short} \
                     was published. {followed}",
                    pushed.output, destination.remote, destination.remote
                )
                .trim_start_matches('\n')
                .to_owned(),
            }));
        }
        if !plan.explicit && !was_rejected_as_stale(&pushed.output) {
            return Ok(Integrated::Finished(pushed));
        }
        previous = pushed.output.clone();
        refused_against = Some((remote_tip, pushed));
    }
    return Ok(Integrated::Finished(unpublished(
        failed_prefix,
        &previous,
        rejected.exit_code,
        &format!(
            "{}/{short} advanced again during each of {MAX_RECONCILE_ATTEMPTS} attempts, so nothing was pushed and the worktree was not touched. \
             Push again when the remote is quiet.",
            destination.remote
        ),
    )));
}

/// What:
///  The awaited reconcile of a push a moved remote branch rejected.
/// Why:
///   The push of the landing round is a task,
/// and every step below this one is a start of Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function reconcileDivergentPushOnTheRuntime({ git, branchRef, rejected }): Promise<PushAttempt>;
/// ```
pub async fn reconcile_divergent_push_awaiting(
    git: &PushGit,
    branch_ref: &str,
    rejected: PushAttempt,
) -> Result<PushAttempt, SingleFlightError> {
    if !was_rejected_as_stale(&rejected.output) {
        return Ok(rejected);
    }
    let Some(destination) = (read_destination_awaiting(git, branch_ref)).await? else {
        debug(
            TAG,
            "the push destination cannot be resolved to a remote branch; no reconcile",
        );
        return Ok(rejected);
    };
    let local: String = git_output_awaiting(
        git,
        &argv(&["rev-parse", "--verify", &format!("{branch_ref}^{{commit}}")]),
    )
    .await?;
    let plan = Reconcile {
        destination,
        local,
        branch_ref: Some(branch_ref),
        label: "auto-push",
        explicit: false,
    };
    // The loop takes its push as a value so controls can stand in for it; this one carries the Git context.
    let mut push_merge: PushMergeThroughGitAwaiting<'_> = PushMergeThroughGitAwaiting { git };
    return Ok(match integrate_and_push_awaiting(git, &plan, rejected, &mut push_merge).await? {
        Integrated::NotDivergent(attempt) | Integrated::Finished(attempt) => attempt,
    });
}
