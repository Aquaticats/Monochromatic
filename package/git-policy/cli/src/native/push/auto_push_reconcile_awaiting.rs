//! What:
//!  The reconcile of a rejected push:
//! the one loop that fetches the remote tip,
//! merges it into the local tip,
//! publishes the merge and
//!  moves the branch,
//!  written once and awaited by whichever caller it has.
//! Why:
//!  Every step of the loop starts Git,
//! and a synchronous start is refused from inside the runtime,
//! so the loop is a task's work.
//!  A command that is not a task of the runtime reaches this one body through
//! `auto_push_reconcile.rs`,
//! which runs the merge push that command supplied on a thread of its own;
//! no decision of the reconcile is written twice,
//! so the two callers cannot publish or say different things.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const integrated = await integrateAndPushOnTheRuntime({ git, plan, rejected, pushMerge });
//! ```

use crate::auto_push_reconcile_plan::{Integrated, PushedSide, Reconcile, merge_message, wording_of};
use crate::auto_push_reconcile_basics_and_limits::{
    MAX_RECONCILE_ATTEMPTS, TAG, argv, fetch_remote_tip_awaiting, is_ancestor_awaiting, journal_identifier,
    unpublished, was_rejected_as_stale,
};
use crate::auto_push_reconcile_branch_advance::{Advanced, advice_to_move};
use crate::auto_push_reconcile_branch_advance_elsewhere::advance_pushed_branch_awaiting;
use crate::auto_push_reconcile_merge_commit::commit_merge_awaiting;
use crate::auto_push_single_flight::{PushAttempt, PushGit, SingleFlightError};
use crate::auto_push_destination::{Destination, read_destination_awaiting};
use crate::diagnostic_log::debug;
use crate::lfs_bounded_push::run_push_awaiting;
use crate::merge_second_pass_tree::announce;
use crate::push_retry_failure::CONFLICT_PHRASE;
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
///  The words that say where the branch the push sent is now.
/// Why:
///   The report's last line has to answer that question on its own,
/// so the sentence says what the move did and not what it was asked for:
///  the worktree here that followed,
/// the worktree elsewhere that followed,
/// a branch no worktree had checked out,
///  and every reason a move was left undone,
/// with the one step that finishes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function followedSentence(advanced: Advanced, branch: string): string;
/// ```
fn followed_sentence(advanced: &Advanced, branch: &str) -> String {
    return match advanced {
        Advanced::Here { kept: false } => format!("and {branch} and this worktree are now at it"),
        Advanced::Here { kept: true } => format!(
            "and {branch} and this worktree are now at it; your uncommitted changes were left as they are"
        ),
        Advanced::Elsewhere { worktree, kept: false } => {
            format!("and {branch} and the worktree at {} are now at it", worktree.display())
        }
        Advanced::Elsewhere { worktree, kept: true } => format!(
            "and {branch} and the worktree at {} are now at it; that worktree's uncommitted changes were left as they are",
            worktree.display()
        ),
        Advanced::Nowhere => format!("and {branch} is now at it"),
        Advanced::Not { reason, advice } => {
            debug(TAG, &format!("{branch} was not fast-forwarded: {reason}"));
            format!("and {branch} is not at it: {reason}; {advice}")
        }
        Advanced::WorktreeWentTo { worktree, shows } => {
            let there: String = shows_here(shows);
            format!(
                "and {branch} is now at it; the worktree at {} checked {there} out while the branch moved, so that worktree shows {there} and nothing of the merge",
                worktree.display()
            )
        }
        Advanced::WorktreeHolding { worktree, shows, paths, reason } => {
            let there: String = shows_here(shows);
            if paths.is_empty() {
                format!(
                    "and {branch} is now at it; the worktree at {} could not be put right: {reason}",
                    worktree.display()
                )
            } else {
                format!(
                    "and {branch} is now at it; the worktree at {}, which shows {there}, still differs from it on {} and was left as it is ({reason}), so a commit there would record those files as they are",
                    worktree.display(),
                    crate::auto_push_dirty_paths::listed(paths)
                )
            }
        }
    };
}

/// What:
///  The words for the branch a checkout shows,
/// or for a detached commit.
/// Why:
///   A worktree can show a commit rather than a branch,
/// and the report must not print nothing where a name belongs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const showsHere = (shows?: string) => shows === undefined ? 'a detached commit' : shows.replace(/^refs\/heads\//, '');
/// ```
fn shows_here(shows: &Option<String>) -> String {
    return match shows {
        Some(named) => named.strip_prefix("refs/heads/").unwrap_or(named).to_owned(),
        None => String::from("a detached commit"),
    };
}

/// What:
///  The reconcile loop,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   Every step below it is awaited,
/// so the loop holds no worker while Git runs;
/// it is the only body of these decisions,
/// so the entry a command outside the runtime cannot drift from it.
///  The merge it publishes is named after the branch the push sent,
/// and that branch follows it wherever it is checked out,
/// so a typed push of a branch that is not checked out is reported and resolved as that branch.
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
    let source: &str = wording.source.as_str();
    let merged_into: &str = wording.merged_into.as_str();
    let remote: &str = destination.remote.as_str();
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
                        "{remote}/{short} has commits {merged_into} lacks, and fetching them failed; nothing was pushed and no branch or worktree was moved.",
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
                    &format!(
                        "Git could not relate {merged_into} to {remote}/{short}; nothing was pushed and no branch or worktree was moved.",
                    ),
                )));
            }
        }
        let message: String = merge_message(plan, remote_tip.as_str());
        let committed = match commit_merge_awaiting(git, &local, &remote_tip, message.as_str()).await? {
            Ok(found) => found,
            Err(paths) => {
                return Ok(Integrated::Finished(unpublished(
                    failed_prefix,
                    &previous,
                    rejected.exit_code,
                    &format!(
                        "merging {remote}/{short} into {merged_into}{CONFLICT_PHRASE}{}. Nothing was pushed and no branch or worktree was moved; \
                         merge {remote}/{short} in a separate worktree, resolve the conflicts and push.",
                        paths.join(", "),
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
            let followed: String = match plan.pushed.branch() {
                None => String::from("and no local branch follows it"),
                Some(branch_ref) => {
                    let branch: &str = branch_ref.strip_prefix("refs/heads/").unwrap_or(branch_ref);
                    // What: the merge is published, so where the branch is now is a report and never a failure.
                    // Why: a move that could not even be asked -- a worktree whose folder went while the push ran, a
                    //      program the system refused -- used to leave the loop as an error, and the push then exited
                    //      nonzero with "may or may not have taken the merge" about a merge it had published
                    //      (a review of commit `98de70959d`, finding 1).
                    let advanced: Advanced = match advance_pushed_branch_awaiting(git, branch_ref, &local, &merge_oid).await {
                        Ok(answer) => answer,
                        Err(error) => Advanced::Not {
                            reason: format!("the move could not be attempted: {error}"),
                            advice: advice_to_move(&merge_oid, "in the worktree that has it checked out"),
                        },
                    };
                    followed_sentence(&advanced, branch)
                }
            };
            let mut merged_parts: String = String::new();
            for line in announce(line_prefix, (journal_identifier(plan.explicit), label), &committed.merged) {
                merged_parts.push_str(format!("{line}\n").as_str());
            }
            let cause: String = if plan.explicit {
                format!("{remote}/{short} had commits {source} did not have")
            } else {
                format!("{remote}/{short} had new commits")
            };
            // The undo line comes before the summary, because a caller that reads only the last line of the output must
            //  meet the statement of what was published and where the branch is, and never the advice on how to undo it.
            return Ok(Integrated::Finished(PushAttempt {
                exit_code: 0,
                output: format!(
                    "{}\n{merged_parts}{line_prefix}to undo this merge, `git revert -m 1 {merge_oid}` keeps both sides' commits and removes its effect.\n\
                     {line_prefix}{cause}, so the merge of {source} and {remote}/{short} was published to {remote}/{short}, {followed} (merge {merge_oid}).",
                    pushed.output
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
            "{remote}/{short} advanced again during each of {MAX_RECONCILE_ATTEMPTS} attempts, so nothing was pushed and no branch or worktree was moved. \
             Push again when the remote is quiet.",
        ),
    )));
}

/// What:
///  The awaited reconcile of a push a moved remote branch rejected,
///  merging the commit the push was allowed to send and never a commit that landed after it.
/// Why:
///   The push of the landing round is a task,
/// and every step below this one is a start of Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function reconcileDivergentPushOnTheRuntime({ git, branchRef, local, rejected }): Promise<PushAttempt>;
/// ```
pub async fn reconcile_divergent_push_awaiting(
    git: &PushGit,
    branch_ref: &str,
    local: &str,
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
    let plan = Reconcile {
        destination,
        local: local.to_owned(),
        pushed: PushedSide::Branch(branch_ref),
        label: "auto-push",
        explicit: false,
    };
    // The loop takes its push as a value so controls can stand in for it; this one carries the Git context.
    let mut push_merge: PushMergeThroughGitAwaiting<'_> = PushMergeThroughGitAwaiting { git };
    return Ok(match integrate_and_push_awaiting(git, &plan, rejected, &mut push_merge).await? {
        Integrated::NotDivergent(attempt) | Integrated::Finished(attempt) => attempt,
    });
}
