//! What:
//!  Publishing a landed commit when the remote branch advanced after the commit landed:
//!       the merge of the remote tip and the local tip is computed in the object store,
//!       committed without a worktree or an index,
//! and pushed with a plain push.
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

/// The coordinator's Git,
/// its command builder,
/// queries and error type.
use crate::auto_push_single_flight::{
    PushAttempt, PushGit, SingleFlightError, git_output,
};
/// The auto-push's bounded `git push` with the Git LFS upload.
use crate::lfs_bounded_push::run_push;
/// The diagnostic log of the wrapper.
use crate::diagnostic_log::debug;
use crate::merge_second_pass_tree::announce;

/// Where the plain push goes,
/// as `auto_push_destination` decides it.
pub use crate::auto_push_destination::Destination;
/// The destination of the plain push of a branch,
/// from the settings alone.
use crate::auto_push_destination::read_destination;
/// The words that name a conflict in the report.
use crate::push_retry::CONFLICT_PHRASE;

use crate::auto_push_reconcile_basics_and_limits::{TAG, argv, fetch_remote_tip, is_ancestor, unpublished};
use crate::auto_push_reconcile_branch_advance::advance_local_branch;
use crate::auto_push_reconcile_merge_commit::commit_merge;
pub use crate::auto_push_reconcile_basics_and_limits::{MAX_RECONCILE_ATTEMPTS, RECONCILE_FAILED_PREFIX, RECONCILE_LINE_PREFIX, journal_identifier, was_rejected_as_stale};
/// What:
///  Merge the remote tip into the pushed branch's tip and publish the result,
/// a bounded number of times.
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function reconcileDivergentPush({ git, branchRef, rejected }): Promise<PushAttempt>;
/// ```
pub fn reconcile_divergent_push(
    git: &PushGit,
    branch_ref: &str,
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
    let local = git_output(
        git,
        &argv(&["rev-parse", "--verify", &format!("{branch_ref}^{{commit}}")]),
    )?;
    let plan = Reconcile {
        destination,
        local,
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

/// What:
///  What one reconcile run integrates,
/// where,
/// and how it words and limits itself.
///  `Option<&str>` is "the branch,
/// or nothing for a detached HEAD".
/// Why:
///   Auto-push and an explicit push share the loop;
/// they differ in the line prefix they print,
/// in whether the branch
///  follows the published merge,
/// and in how much of a failure they keep quiet about.
///   An explicit push already printed Git's own refusal,
/// so a remote that cannot be fetched,
/// a remote that did not
///  diverge,
/// and a failure that is not a stale update are left exactly as the push reported them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reconcile = { destination: Destination; local: string; branchRef?: string; label: string; explicit: boolean };
/// ```
pub struct Reconcile<'a> {
    /// The remote and the ref the rejected push updated.
    pub destination: Destination,
    /// The commit the rejected push sent.
    pub local: String,
    /// The branch that follows the published merge when it can,
    /// or nothing for a detached HEAD.
    pub branch_ref: Option<&'a str>,
    /// The word after `monochromatic-git-policy-cli:` in every line the run adds.
    pub label: &'a str,
    /// Whether the rejected push was typed by a person and has already shown its own output.
    pub explicit: bool,
}

/// How a reconcile run ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Integrated = { kind: 'not-divergent'; attempt } | { kind: 'finished'; attempt };
/// ```
pub enum Integrated {
    /// The remote did not diverge,
    /// or the run could not tell;
    /// the rejected attempt stands as it was.
    NotDivergent(PushAttempt),
    /// The run published the merge,
    /// or says why it did not.
    Finished(PushAttempt),
}

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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface MergePusher { pushMerge(destination: Destination, mergeOid: string): PushAttempt }
/// ```
pub trait MergePusher {
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
///  Merge the remote tip into the pushed commit and publish the result,
/// a bounded number of times.
/// Why:
///   Another worktree advancing the remote makes the plain push fail for good,
/// while both histories are valid and
///  must both be kept;
/// the merge is published as a new commit above the local tip,
/// so the landed commit stays
///  reachable and nothing is forced.
///   `push_merge` publishes one merge,
/// and is told the destination and the merge commit.
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
    let destination = plan.destination.clone();
    let local = plan.local.clone();
    let label = plan.label;
    let line_prefix = format!("monochromatic-git-policy-cli: {label}: ");
    let failed_prefix = format!("{line_prefix}not published: ");
    let short = destination
        .merge_ref
        .strip_prefix("refs/heads/")
        .unwrap_or(&destination.merge_ref)
        .to_owned();
    // The branch the merge is into, named the way Git's own message names it: the branch whose tip is the merge's
    // first parent, and the tracked ref's short name for a detached HEAD, which has no branch to name. A branch whose
    // upstream has another name (`branch.feature.merge = refs/heads/main`) is merged into by the same rule, so naming
    // the tracked ref would name a branch nobody merged into.
    let merged_into: &str = match plan.branch_ref {
        Some(reference) => reference.strip_prefix("refs/heads/").unwrap_or(reference),
        None => short.as_str(),
    };
    // The merge is published to the ref the rejected push updated, which the branch's name does not always spell. That
    // ref is not always the branch's upstream (an explicit `feature:main` push sends a branch that tracks
    // `origin/feature`), so the line says where the merge went and nothing about what the branch tracks.
    let published: String = if merged_into == short {
        String::new()
    } else {
        format!("\nPublished to {remote}/{short}.", remote = destination.remote)
    };
    let message = format!(
        "Merge {}/{short} into {merged_into}\n\nCreated by monochromatic-git-policy-cli {label} because {}/{short} advanced after the commit landed.{published}",
        destination.remote, destination.remote
    );
    let mut previous = rejected.output.clone();
    // The remote tip a merge was pushed against, with the refusal it got: a remote that still shows that tip did not
    // move, so the refusal was a rule and not a race.
    let mut refused_against: Option<(String, PushAttempt)> = None;
    for attempt in 1..=MAX_RECONCILE_ATTEMPTS {
        let remote_tip = match fetch_remote_tip(git, &destination)? {
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
                    &failed_prefix,
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
            is_ancestor(git, &remote_tip, &local)?,
            is_ancestor(git, &local, &remote_tip)?,
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
                    &failed_prefix,
                    &previous,
                    rejected.exit_code,
                    "Git could not relate the branch to the remote tip; nothing was pushed and the worktree was not touched.",
                )));
            }
        }
        let committed = match commit_merge(git, &local, &remote_tip, &message)? {
            Ok(found) => found,
            Err(paths) => {
                return Ok(Integrated::Finished(unpublished(
                    &failed_prefix,
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
        let merge_oid = committed.oid.clone();
        debug(
            TAG,
            &format!(
                "attempt {attempt}: pushing the merge {merge_oid} of {local} and {remote_tip}"
            ),
        );
        let pushed = push_merge.push_merge(&destination, &merge_oid)?;
        if pushed.exit_code == 0 {
            let branch = match plan.branch_ref {
                Some(reference) => reference.strip_prefix("refs/heads/").unwrap_or(reference),
                None => "HEAD",
            };
            let advanced = match plan.branch_ref {
                Some(branch_ref) => advance_local_branch(git, branch_ref, &local, &merge_oid)?,
                None => Err(String::from("HEAD is detached")),
            };
            let followed = match advanced {
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
            for line in announce(&line_prefix, (journal_identifier(plan.explicit), label), &committed.merged) {
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
        &failed_prefix,
        &previous,
        rejected.exit_code,
        &format!(
            "{}/{short} advanced again during each of {MAX_RECONCILE_ATTEMPTS} attempts, so nothing was pushed and the worktree was not touched. \
             Push again when the remote is quiet.",
            destination.remote
        ),
    )));
}

/// Reconcile behavior is controlled in the test module.
#[cfg(test)]
#[path = "auto_push_reconcile_tests.rs"]
mod tests;
