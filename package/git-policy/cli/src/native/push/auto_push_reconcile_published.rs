//! What:
//!  The one question both arms of the reconcile ask about a merge they pushed, and the one sentence a
//!  published merge is reported with.
//! Why:
//!  A push command can report a failure after the remote took what it sent (a kill, a connection
//!  dropped after acceptance), and the remote may then have moved past that merge (a post-receive hook,
//!  another push), so what landed is decided by the remote's own history and never by what the push
//!  command said.
//!  The automatic arm and the explicit arm of the loop answer with this one question and this one
//!  sentence, so neither can publish or say something the other would not
//!  (a review of commit `c15fc5a7b1`, its findings 1 and 2, and commit `cae78e1d7e`, its finding 1).
//!  Kept in a file of its own because the loop's file is at the repository's line budget.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const holds = await theRemoteHoldsTheMerge({ git, merge, remoteTip });
//! const attempt = thePublishedMergeReport({ plan, remoteTip, merge, followed });
//! ```

use crate::auto_push_reconcile_basics_and_limits::is_ancestor_awaiting;
use crate::auto_push_reconcile_plan::{Reconcile, wording_of};
use crate::auto_push_single_flight::{PushAttempt, PushGit, SingleFlightError};

/// What:
///  Whether the remote holds the merge this run published,
/// asked of the remote's own history and never of what a push command said.
/// Why:
///   The merge is looked for in the remote tip's history and not at the tip alone,
/// because the remote may have moved past it before the question is asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holds = (await isAncestor(git, merge, remoteTip)) ? merge : undefined;
/// ```
pub(crate) async fn the_remote_holds_the_merge_awaiting(
    git: &PushGit,
    merge: &str,
    remote_tip: &str,
) -> Result<Option<String>, SingleFlightError> {
    return match (is_ancestor_awaiting(git, merge, remote_tip)).await? {
        Some(true) => Ok(Some(merge.to_owned())),
        _ => Ok(None),
    };
}

/// What:
///  The report of a merge this run published, once the remote is asked and agrees:
/// the undo step, then the line that says what was published and where the pushed branch is.
/// Why:
///   A merge that was published is never reported as a failed push, whichever arm met it,
/// and a caller that reads only the last line of the output must meet the statement of what was
/// published and where the branch is, never the advice on how to undo it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const published = ({ plan, remoteTip, merge, followed }) => ({ exitCode: 0, output: ... });
/// ```
pub(crate) fn the_published_merge_report(plan: &Reconcile<'_>, remote_tip: &str, merge: &str, followed: &str) -> PushAttempt {
    let wording = wording_of(plan);
    let line_prefix: &str = wording.line_prefix.as_str();
    let remote: &str = plan.destination.remote.as_str();
    let short: &str = wording.short.as_str();
    return PushAttempt {
        exit_code: 0,
        output: format!(
            "{line_prefix}to undo this merge, `git revert -m 1 {merge}` keeps both sides' commits and removes its effect.\n\
             {line_prefix}{remote}/{short} is at {remote_tip}, which contains the merge {merge} this push made, so the merge was published \
             even though its own command reported a failure, {followed} (merge {merge}).",
        ),
    };
}
