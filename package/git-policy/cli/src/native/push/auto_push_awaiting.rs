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
//! const result = await pushOnceOnTheRuntime({ gitPath, cwd }, branchRef, landedOid, blocked);
//! ```

use crate::auto_push::{
    AutoPushOutcome, AutoPushResult, PUSH_LOCK_PAUSE, PUSH_LOCK_RETRIES, Planned, REFERENCE_LOCK_REFUSAL, Sent,
    TAG, held_back_result, push_arguments, pushed_tracking_ref, reported, skipped_with, source_of,
};
use crate::auto_push_destination::{NamedPush, named_push_awaiting};
use crate::auto_push_reconcile_awaiting::reconcile_divergent_push_awaiting;
use crate::auto_push_single_flight::{PushAttempt, PushGit, SingleFlightError, run_query_awaiting};
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
/// const result = await pushOnceOnTheRuntime({ gitPath, cwd }, branchRef, landedOid);
/// ```
pub async fn push_once_awaiting(
    git: &PushGit,
    branch_ref: &str,
    landed_oid: &str,
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
    let Some(source) = source_of(&stop, &sent, pushed_ref.as_str()) else {
        return Ok(held_back_result(pushed_ref.as_str(), &stop, err));
    };
    let pushing_whole: bool = stop == WhereThePushStops::Nothing;
    let arguments: Vec<String> = push_arguments(source.as_str(), &sent, set_upstream);
    let argument_views: Vec<&str> = arguments.iter().map(String::as_str).collect();
    debug(TAG, &format!("pushing {landed_oid} with {argument_views:?} on {pushed_ref}"));
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
        let full: String = crate::push_tracking_ref_repair::resolve_awaiting(git, of).await.unwrap_or_else(|| { return of.to_owned(); });
        if !crate::push_tracking_ref_repair::bring_up_to_date_awaiting(git, full.as_str(), source.as_str()).await {
            note = crate::push_tracking_ref_repair::still_behind(full.as_str(), source.as_str());
        }
    }
    // A push held back by a verdict is never reconciled: the merge the reconcile publishes is made above the local tip, so it would
    //  carry the very commit the gate refused. It stays a plain push of the base, and the branch is tried again by itself.
    if upstream.is_some() && pushing_whole && attempt.exit_code != 0 {
        attempt = reconcile_divergent_push_awaiting(git, &pushed_ref, attempt).await?;
    }
    return reported(attempt, &stop, pushed_ref.as_str(), note.as_str(), err);
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
