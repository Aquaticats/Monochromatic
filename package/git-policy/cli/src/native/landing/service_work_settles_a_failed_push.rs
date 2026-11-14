//! What:
//!  What became of one failed push,
//!  as the verdict the service settles its work item with.
//! Why:
//!  Three facts of one push decide it:
//! what it reached,
//! whether its branch is still there,
//! and what Git said at the address that failed.
//! They are read once here,
//! and the answer is written in `service_work_failed_push_verdict.rs`;
//! the file lives beside the landing round because `service_work.rs` is at its line bound,
//! and the round is where the three facts are at hand.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = await answerFailedPush(failed, sent, newest, git);
//! ```

use crate::service_work::Verdict;
use crate::push_attempts_of_a_failed_push::{FailedPushByAddress, failed_push_by_address};
use crate::service_work_branch_is_gone::branch_is_gone_words_awaiting;
use crate::service_work_failed_push_verdict::{SettledFailedPush, verdict_of_a_failed_push};
use crate::auto_push::AutoPushResult;
use crate::auto_push_single_flight::PushGit;
use crate::service_work::branch_exists_awaiting;
use crate::work_item::WorkItem;

/// What:
///  The verdict of a push that failed at an address,
///  with the record the person reads when it reached another one.
/// Why:
///  A commit a remote holds stays there,
/// so a delivery is answered as a delivery and never as a push that was not made,
/// and a branch the person deleted ends every further attempt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// answer(failed, sent, newest, git) // delivered | skipped | failed | retry
/// ```
pub(crate) async fn answer(
    failed: AutoPushResult,
    sent: &str,
    newest: &WorkItem,
    git: &PushGit,
) -> Verdict {
    // What: The reference this reading looks for is the one the push wrote to.
    // Why:  Git's status line names the reference it updated (`... <oid> -> main` for a local `topic`), so looking for the local
    //       name found no line and called a delivered push one that reached nowhere, with a final notice that said the commit was
    //       saved only here (the fifth review of issue 6293, finding 1).
    let destination: &str = failed.destination_or(newest.branch_ref.as_str());
    let read: FailedPushByAddress = failed_push_by_address(failed.shown.as_str(), destination, failed.push_addresses.as_slice());
    let branch_is_gone: bool = branch_exists_awaiting(git, newest.branch_ref.as_str()).await == Some(false);
    let settled: SettledFailedPush = verdict_of_a_failed_push(failed, read, newest.branch_ref.as_str(), newest.repository.as_path(), branch_is_gone);
    return match settled {
        // What: A push that reached an address is a push that landed, whatever another address did.
        // Why:  The branch is backed up, so nothing is retried and nothing failed; what is left to say is where the commit is and which
        //       address was given up, and that goes where every accepted push's words go: a record the next command shows once,
        //       named for the commit this push sent.
        SettledFailedPush::Reached(said) => Verdict::Delivered { commit: String::from(sent), said },
        // What: A push that reached no address on a branch the person deleted is answered as it always was.
        // Why:  Nothing was sent, so nothing can pass by waiting, and the repository is asked where the commit still lives, never the
        //       words of the output: any program a push runs can print a line about a branch of its own.
        SettledFailedPush::Unreached(_) if branch_is_gone => Verdict::Skipped(branch_is_gone_words_awaiting(git, newest).await),
        SettledFailedPush::Unreached(verdict) => verdict,
    };
}
