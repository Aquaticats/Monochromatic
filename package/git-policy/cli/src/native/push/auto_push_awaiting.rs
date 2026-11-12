//! What:
//!  The automatic push of a landed commit's branch as a task of the runtime:
//! the same plan,
//! the same refspec and the same
//!  words as the synchronous push,
//! awaited rather than waited on.
//! Why:
//!  The service's push is a task of its runtime,
//! and every Git start below it is refused from inside the runtime when it is made synchronously,
//! so the plan,
//!  the bounded push,
//! the held-reference retry and the reconcile each have an awaited face.
//! They are the synchronous face's questions,
//! arguments and wording,
//! so a push cannot differ between the two.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = await pushOnceOnTheRuntime({ gitPath, cwd }, branchRef, sentUpTo, blocked, undecided);
//! ```

use crate::auto_push::{
    AutoPushOutcome, AutoPushResult, PUSH_LOCK_PAUSE, PUSH_LOCK_RETRIES, Planned, REFERENCE_LOCK_REFUSAL, Refspec, Sent,
    TAG, held_back_result, moved_past_the_pin, push_arguments, pushed_tracking_ref, refspec_of, reported, skipped_with, source_of,
};
use crate::auto_push_destination::{NamedPush, named_push_awaiting};
use crate::auto_push_reconcile_awaiting::reconcile_divergent_push_awaiting;
use crate::auto_push_single_flight::{PushAttempt, PushGit, SingleFlightError, run_query_awaiting};
/// Whether a push Git reported as failed published the recorded branch all the same.
use crate::matching_push_delivered_the_recorded_branch::{MatchingPush, delivered_although_git_failed_awaiting};
use crate::diagnostic_log::debug;
use crate::lfs_bounded_push::run_push_awaiting;
use crate::push_held_back_by_a_blocked_commit::{WhereThePushStops, where_the_push_stops_awaiting};
use crate::work_item_holding::BlockedVerdict;
use std::ffi::OsString;
use std::io::Write;

/// What:
///  Whether a read-only Git query exits zero,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The landing round's push asks this as a task,
/// and the synchronous query enters the runtime and is refused from there.
/// The question and the reading of the exit status are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await gitQuerySucceedsOnTheRuntime({ gitPath, cwd, args });
/// ```
async fn query_succeeds_awaiting(git: &PushGit, arguments: &[&str]) -> Result<bool, SingleFlightError> {
    let argv: Vec<OsString> = arguments.iter().map(OsString::from).collect();
    return Ok(run_query_awaiting(git, &argv).await?.code == Some(0));
}

/// What:
///  The same plan of a push,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The plan asks Git twice and reads the destination's configuration,
/// and every one of those starts is refused from inside the runtime.
/// The questions and the arguments are the synchronous face's,
/// so the two plan the same push.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = await planNamedPushOnTheRuntime({ gitPath, cwd }, branchRef);
/// ```
pub(crate) async fn plan_named_push_awaiting(
    git: &PushGit,
    branch_ref: &str,
    err: &mut (dyn Write + Send),
) -> Result<Planned, SingleFlightError> {
    // Git reads `refs/heads/x@{upstream}` as a branch named `refs/heads/x` ("no such branch"), so the query takes the short name;
    //  a ref outside `refs/heads` has no upstream.
    let upstream: Option<String> = match branch_ref.strip_prefix("refs/heads/") {
        Some(name) => upstream_of_awaiting(git, name).await?,
        None => None,
    };
    if upstream.is_none() && !query_succeeds_awaiting(git, &["remote", "get-url", "origin"]).await? {
        debug(TAG, "no upstream and no origin remote; skipping auto-push");
        return Ok(Planned::Skip(AutoPushResult { outcome: AutoPushOutcome::Skipped, exit_code: 0, shown: String::new() }));
    }
    return Ok(match &upstream {
        Some(_upstream_ref) => match named_push_awaiting(git, branch_ref).await? {
            NamedPush::Push { remote, targets } => Planned::Push {
                branch_ref: branch_ref.to_owned(),
                upstream,
                sent: Sent { remote, targets },
                set_upstream: false,
            },
            // Git matches the branches itself: the plain push of the remote, with no refspec.
            NamedPush::Matching { remote } => Planned::Push {
                branch_ref: branch_ref.to_owned(),
                upstream,
                sent: Sent { remote, targets: Vec::new() },
                set_upstream: false,
            },
            NamedPush::Skip(note) => skipped_with(note, err),
        },
        // A branch with no upstream goes to `origin` under its own name and takes `origin` as its upstream.
        None => Planned::Push {
            branch_ref: branch_ref.to_owned(),
            upstream: None,
            sent: Sent { remote: String::from("origin"), targets: vec![branch_ref.to_owned()] },
            set_upstream: true,
        },
    });
}

/// What:
///  The remote-tracking reference a branch's pushes update,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The awaited plan asks the same question the synchronous one does,
/// and the query enters the runtime from inside it.
/// The answer decides where the
///  push goes and whether a blocked commit is already published.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const upstreamOf = async (git, short) => (await git(['rev-parse', '--abbrev-ref', '--symbolic-full-name', `${short}@{upstream}`])).ok || undefined;
/// ```
async fn upstream_of_awaiting(git: &PushGit, short: &str) -> Result<Option<String>, SingleFlightError> {
    let asked: String = format!("{short}@{{upstream}}");
    let arguments: Vec<OsString> = ["rev-parse", "--abbrev-ref", "--symbolic-full-name", asked.as_str()].iter().map(OsString::from).collect();
    let output = run_query_awaiting(git, &arguments).await?;
    if output.code != Some(0) {
        return Ok(None);
    }
    let named: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
    return Ok((!named.is_empty()).then_some(named));
}

/// What:
///  Whether the branch still names `commit`,
///  awaited by a caller that is itself a task of the runtime.
/// Why:
///   A matching push leaves the refspec to Git,
///  and the one push that would give a branch its upstream names the branch,
///  so both ask the branch what it names now,
///  and the synchronous face asks the same question.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const branchNamesThePin = async ({ gitPath, cwd }, branchRef, commit) => (await gitText(gitPath, ['rev-parse', '--verify', '--quiet', `${branchRef}^{commit}`], { cwd })) === commit;
/// ```
async fn branch_names_the_pin_awaiting(git: &PushGit, branch_ref: &str, commit: &str) -> Result<bool, SingleFlightError> {
    let asked: String = format!("{branch_ref}^{{commit}}");
    let arguments: Vec<OsString> = ["rev-parse", "--verify", "--quiet", asked.as_str()].iter().map(OsString::from).collect();
    let output = run_query_awaiting(git, &arguments).await?;
    if output.code != Some(0) {
        // The branch names no commit at all, so it does not name this one.
        return Ok(false);
    }
    return Ok(String::from_utf8_lossy(output.stdout.as_slice()).trim() == commit);
}

/// What:
///  Push a landed commit's branch once,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The push of the landing round is a task,
/// and every step below it starts Git,
/// which the synchronous face enters the runtime for and is refused from there.
/// The plan,
/// the held-reference retry,
/// the reconcile and the words the push shows are the synchronous face's,
/// so a push and its retry cannot differ between the two.
///  The pause between the retries is the runtime's own,
/// so no worker is held while the other worktree's lock is waited out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const result = await pushOnceOnTheRuntime({ gitPath, cwd }, branchRef, sentUpTo, blocked, undecided);
/// ```
pub async fn push_once_awaiting(
    git: &PushGit,
    branch_ref: &str,
    sent_up_to: &str,
    blocked: &[BlockedVerdict],
    undecided: &[String],
    err: &mut (dyn Write + Send),
) -> Result<AutoPushResult, SingleFlightError> {
    let planned: Planned = plan_named_push_awaiting(git, branch_ref, err).await?;
    let (pushed_ref, upstream, sent, set_upstream): (String, Option<String>, Sent, bool) = match planned {
        Planned::Skip(result) => return Ok(result),
        Planned::Push { branch_ref: planned_ref, upstream, sent, set_upstream } => (planned_ref, upstream, sent, set_upstream),
    };
    // A post-commit verdict on the branch's way out decides whether this push may send the branch's tip or only the base below it,
    //  and so does a commit whose own checks still run.
    let stop: WhereThePushStops = where_the_push_stops_awaiting(git, pushed_ref.as_str(), upstream.as_deref(), blocked, undecided).await;
    let Some(pinned) = source_of(&stop, &sent, sent_up_to) else {
        return Ok(held_back_result(pushed_ref.as_str(), &stop, err));
    };
    // Two pushes ask the branch what it names now: a matching one, which leaves the refspec to Git, and the one that would give
    //  the branch its upstream, because Git sets an upstream only from a refspec that names a branch (measured with Git 2.56.0:
    //  `git push --set-upstream origin <commit>:refs/heads/new` delivers the commit and leaves `branch.new.merge` unset).
    //  Both may name the branch only while it still names the commit this push may send.
    let names_the_pin: bool = match sent.targets.is_empty() || set_upstream {
        true => branch_names_the_pin_awaiting(git, pushed_ref.as_str(), pinned.as_str()).await?,
        false => false,
    };
    // A matching push names no refspec at all, so Git resolves the branch itself and sends whatever it names by then: a commit that
    //  landed after the reading has no verdict in it, and its own gate may still keep it local. Nothing is sent while the branch has
    //  moved past the commit this push may send, and the retry plan tries the branch again.
    if sent.targets.is_empty() && !names_the_pin {
        return Ok(moved_past_the_pin(pushed_ref.as_str(), err));
    }
    let refspec: Refspec = refspec_of(pinned.as_str(), pushed_ref.as_str(), names_the_pin);
    let pushing_whole: bool = stop == WhereThePushStops::Nothing;
    let arguments: Vec<String> = push_arguments(refspec.source.as_str(), &sent, refspec.set_upstream);
    let argument_views: Vec<&str> = arguments.iter().map(String::as_str).collect();
    debug(TAG, &format!("pushing {sent_up_to} with {argument_views:?} on {pushed_ref}"));
    let mut attempt: PushAttempt = run_push_command_awaiting(git, argument_views.as_slice()).await?;
    // The instances of two worktrees share the repository's references. While another holds the lock of the remote-tracking
    //  reference this push updates, Git still delivers the push, prints the refusal to update the reference, and exits 0, which
    //  leaves the reference stale; the same push again finds nothing to deliver and brings the reference up to date.
    let mut pause: u32 = 0;
    while attempt.output.contains(REFERENCE_LOCK_REFUSAL) && pause < PUSH_LOCK_RETRIES {
        pause += 1;
        tokio::time::sleep(PUSH_LOCK_PAUSE * pause).await;
        attempt = run_push_command_awaiting(git, argument_views.as_slice()).await?;
    }
    // The push delivered, and Git may have been refused the update of the branch's remote-tracking reference while another Git
    //  held its lock; the repeats above write it when that lock went in the meantime, and one that outlives them leaves the
    //  reference behind, so it is written here and the state is said when it could not be
    //  (`push_tracking_ref_repair.rs`). Nothing a person asked for waits on it: the push is over.
    let mut note: String = String::new();
    if attempt.exit_code == 0
        && let Some(of) = pushed_tracking_ref(upstream.as_deref(), &sent, pushed_ref.as_str())
    {
        // The reference is written by its full name: the name a branch's upstream is answered with, `origin/main`, is no ref.
        let full: String = match crate::push_tracking_ref_repair::resolve_awaiting(git, of).await {
            Some(named) => named,
            None => of.to_owned(),
        };
        if !crate::push_tracking_ref_repair::bring_up_to_date_awaiting(git, full.as_str(), pinned.as_str()).await {
            note = crate::push_tracking_ref_repair::still_behind(full.as_str(), pinned.as_str());
        }
    }
    // A push held back by a verdict is never reconciled: the merge the reconcile publishes is made above the local tip, so it would
    //  carry the very commit the gate refused. It stays a plain push of the base, and the branch is tried again by itself.
    if upstream.is_some() && pushing_whole && attempt.exit_code != 0 {
        attempt = reconcile_divergent_push_awaiting(git, &pushed_ref, sent_up_to, attempt).await?;
    }
    // Under `push.default=matching` this one push carried every branch the remote has, so Git's status is the worst of them: a
    //  sibling the remote refused makes it non-zero while this branch arrived. The remote says which, and a delivery it proves is
    //  not reported as a failure nor retried.
    let delivered: bool = delivered_although_git_failed_awaiting(
        git,
        &MatchingPush {
            git_matched: sent.targets.is_empty(),
            remote: sent.remote.as_str(),
            branch_ref: pushed_ref.as_str(),
            exit_code: attempt.exit_code,
            landed_oid: sent_up_to,
        },
    )
    .await?;
    return reported(attempt, delivered, &stop, pushed_ref.as_str(), note.as_str(), err);
}


/// What:
///  Run one real `git push` and capture its interleaved output,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The awaited push is the synchronous one with its wait on the runtime,
/// so the arguments and the captured output are the same.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attempt = await runPushAwaiting(git, pushArgs);
/// ```
async fn run_push_command_awaiting(git: &PushGit, arguments: &[&str]) -> Result<PushAttempt, SingleFlightError> {
    let argv: Vec<OsString> = arguments.iter().map(OsString::from).collect();
    return run_push_awaiting(git, &argv).await;
}
