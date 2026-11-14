//! What:
//!  The verdict of one push Git refused,
//!  from what the push did address by address and from Git's own words.
//! Why:
//!  Three readings decide it,
//! and the addresses come first
//! (`push_attempts_of_a_failed_push.rs`):
//!  - a push that reached no address and failed at one that can never be a remote ends its plan at once,
//!    with one line that says so
//!    (issue 6293,
//!    the owner's decision of 2026-11-13,
//!    "End the plan at once,
//!    say so once").
//!    Git's words cannot tell such an address from one that may come back,
//!    so the address decides
//!    (`push_address_that_cannot_be_a_remote.rs`),
//!    and no retry can make it work;
//!  - a push that reached an address is a branch that is backed up,
//!    whatever another address did:
//!    when the address it failed at can never be a remote,
//!    that address is given up for itself,
//!    the plan ends because there is nothing left to push,
//!    and the person reads one line saying the commit is on the address that holds it
//!    (the first review of issue 6293,
//!    finding 2,
//!    which found the notice saying the commit was only on this computer
//!    while a remote held it);
//!    when the address it failed at may come back,
//!    the plan is kept,
//!    so that address still gets the commit when it returns;
//!    and when what Git said there is a failure no retry can mend
//!    (a server hook's decline,
//!    a login the service does not have,
//!    a conflict),
//!    the plan ends with one report that says where the commit is and which address refused it,
//!    because that refusal repeats on every try
//!    (the second review of issue 6293,
//!    finding 2,
//!    which found the plan retrying a declined address for ever);
//!  - otherwise what a failed push's own words mean decides it (`push_retry_failure.rs`):
//!    a conflict,
//!    a login the service does not have and a server hook's decline need the person,
//!    and every other failure can pass (a stall,
//!    the network,
//!    an LFS server error,
//!    a held lock),
//!    so it keeps its retries.
//!  It lives beside the drain's verdicts rather than inside it because the drain is long and this is one decision.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! verdictOfAFailedPush(result, branchRef, repository) // { reached: words } | { unreached: verdict }
//! ```

/// The push that failed,
/// whose addresses and words decide the verdict.
use crate::auto_push::AutoPushResult;
/// The addresses that can never be a remote,
/// and the lines the person reads for them.
use crate::push_address_that_cannot_be_a_remote::{CannotBeARemote, cannot_be_a_remote, words, words_after_delivery};
/// What one failed push did,
/// address by address,
/// which the caller reads once and hands in.
use crate::push_attempts_of_a_failed_push::FailedPushByAddress;
/// What a failed push's own words mean,
/// and which of those failures no retry can mend.
use crate::push_retry_failure::{FinalFailure, final_failure, plain_words, unrecognised_rejection};
/// The verdicts a push's settlement reads.
use crate::service_work::Verdict;
use std::path::Path;

/// What:
///  What one refused push comes to:
///  a branch that reached an address,
/// with the words the next command shows once,
///  or a branch that reached none,
/// with the verdict for the item.
/// Why:
///   A failure the service gives up on forgets the branch's plan and writes one record,
/// and a retry keeps the plan and promises it in the record;
/// the caller settles the item with the answer this function gives.
/// A push that reached an address has nothing to retry and nothing to fail:
/// the branch is backed up,
/// so its item is done and its words are a report,
/// which the notice directory keeps apart from failures
/// (`push_failure_notice_reports.rs`).
/// The two answers are separate variants rather than a verdict beside an optional report,
/// so that no caller can settle a branch that reached an address as failed,
/// or a branch that reached nothing as landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SettledFailedPush = { reached: string } | { unreached: Verdict };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SettledFailedPush {
    /// The branch reached at least one address,
    /// so its item is done;
    /// these are the words the next command shows once,
    /// which say where the commit is and what became of the address that failed,
    /// and they are never empty.
    Reached(String),
    /// The branch reached no address,
    /// and this is what the item settles as.
    Unreached(Verdict),
}

/// What:
///  What one refused push comes to.
/// Why:
///   The address Git failed at decides whether a plan ends,
/// and what the push reached decides whether a failure is reported at all:
/// a branch a remote holds is not a branch to retry or to fail.
/// The configuration's other addresses never decide anything on their own,
/// which is what the first review of this branch found wrong twice
/// (findings 2 and 3).
/// The branch's own fate is handed in rather than asked here:
/// a commit a remote holds stays there after the branch is deleted,
/// so a push that reached an address is reported as one even when the branch is gone,
/// and the branch being gone only ends every further attempt
/// (the third review of issue 6293,
/// finding 3,
/// which found the branch-gone answer taking a recorded delivery away).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failing = read.failing && cannotBeARemote(read.failing);
/// if (read.arrived) {
///   const words = failing ? wordsAfterDelivery(...) : finalFailure(shown) ? deliveredThenGivenUp(...) : reachedThenFailed(...);
///   if (!failing && !finalFailure(shown) && !branchIsGone) return { unreached: { retry: words + '\n' + shown } };
///   return { reached: words + (branchIsGone ? '\n' + branchGoneLine : '') + '\n' + shown };
/// }
/// if (failing) return { unreached: { failed: words(...) } };
/// return { unreached: fromWords(shown) };
/// ```
pub(crate) fn verdict_of_a_failed_push(
    result: AutoPushResult,
    read: FailedPushByAddress,
    branch_ref: &str,
    repository: &Path,
    branch_is_gone: bool,
) -> SettledFailedPush {
    let failing: Option<(&str, CannotBeARemote)> = match read.failing.as_deref() {
        Some(address) => match cannot_be_a_remote(address) {
            Some(reason) => Some((address, reason)),
            None => None,
        },
        None => None,
    };
    if read.arrived {
        // What: The words of a push that reached an address, chosen by what Git said at the address that failed.
        // Why:  Each says where the commit is;
        // only the case that keeps a plan goes back to the retry, and only while nothing else ends the branch.
        // What: The words of this arm, with Git's own output appended once below them.
        // Why:  Each arm's words say where the commit is and what happened at the address that failed; the output is evidence and is
        //       printed once, whichever arm is chosen.
        let said: String = if let Some((address, reason)) = failing {
            words_after_delivery(address, reason, read.arrived_at().as_slice())
        } else if let Some(cause) = final_failure(result.shown.as_str()) {
            // A failure no retry can mend at the address that failed ends the plan here too, with one report.
            delivered_then_given_up(&read, cause)
        } else if !branch_is_gone {
            return SettledFailedPush::Unreached(Verdict::Retry(format!("{}\n{}", reached_then_failed(&read), result.shown)));
        } else {
            // The address would have been tried again, and the branch it would send is gone: nothing is tried again.
            reached_then_failed(&read)
        };
        // What: One sentence more when the branch is gone, so the person knows nothing more will be tried for it.
        // Why:  The record is the last thing said about this push, and a branch deleted while it ran is what keeps its plan from
        //       going on; the record must not leave the person waiting for another try.
        let gone: String = if branch_is_gone { format!("\n{}", branch_gone_line(branch_ref)) } else { String::new() };
        return SettledFailedPush::Reached(format!("{said}{gone}\n{}", result.shown));
    }
    if let Some((address, reason)) = failing {
        return SettledFailedPush::Unreached(Verdict::Failed(format!("{}\n{}", words(address, reason), result.shown)));
    }
    return SettledFailedPush::Unreached(verdict_from_the_words(result.shown.as_str(), repository));
}

/// What:
///  The one sentence a record carries when the branch it pushed was deleted while the push ran.
/// Why:
///   The commit is a fact of the remote and the branch is a name in this repository,
/// so the record says where the commit is and that the name is gone,
/// and says plainly that nothing is tried again for it:
/// a reader who is not told that waits for a retry that will not come.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// branchGoneLine('refs/heads/main') // 'the branch main no longer exists in this repository, ...'
/// ```
fn branch_gone_line(branch_ref: &str) -> String {
    let branch: &str = branch_ref.strip_prefix("refs/heads/").unwrap_or(branch_ref);
    return format!(
        "the branch {branch} no longer exists in this repository (it was deleted while this push ran), so nothing is tried again for \
         it. The commit is where this record says it is; a commit you want on a branch again can be pushed by hand after naming it \
         with `git branch`."
    );
}

/// What:
///  What the words of a push that reached nothing mean,
///  which is the classification this release has always used.
/// Why:
///   Nothing about the addresses says more than Git's own words there,
/// so a conflict,
/// a login,
/// a server hook's decline and an unknown rejection keep their own verdicts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// finalFailure(output) ? Failed(plainWords(cause) + output) : Retry(output)
/// ```
fn verdict_from_the_words(shown: &str, repository: &Path) -> Verdict {
    return match final_failure(shown) {
        Some(cause) => Verdict::Failed(match plain_words(cause) {
            Some(words) => format!("{words}\n{shown}"),
            None => shown.to_owned(),
        }),
        None => {
            // A rejection by the server whose reason is not known is tried again, and named so that it can be learned.
            if let Some(line) = unrecognised_rejection(shown) {
                crate::service_warn!(
                    "the server rejected the push {} for a reason this release does not know, so it is tried again: {line}",
                    crate::service_timing::in_the_repository_at(repository)
                );
            }
            Verdict::Retry(shown.to_owned())
        }
    };
}

/// What:
///  The words of a push that reached an address and was refused for good at another.
/// Why:
///   The person is told what is true:
/// the commit is backed up on the addresses that took it,
/// the address named is the one that refused it,
/// nothing tries that address again by itself,
/// and what to do about it is the words this release already uses for that cause
/// (`plain_words`),
/// with Git's own words under both,
/// appended once by the caller that builds the whole record.
/// A conflict arrives here only if another address took the commit first,
/// and then its words say that only a person can settle the two histories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// deliveredThenGivenUp(read, cause) // 'monochromatic-git-policy-cli: the push of this branch reached ...'
/// ```
fn delivered_then_given_up(read: &FailedPushByAddress, cause: FinalFailure) -> String {
    let holders: String = read.arrived_at().join(", ");
    let failed_at: &str = read.failing.as_deref().unwrap_or("another address");
    // What: The cause's own advice, which already says the service stops pushing this branch until the next commit.
    // Why:  A conflict is the one cause whose advice is not written there, because a reconcile that conflicts never reaches an
    //       address; it is named here so that no cause arrives without one.
    let why: &str = plain_words(cause).unwrap_or(
        "this failure is one no retry can mend: the remote's commits and this branch cannot be merged without a person deciding how",
    );
    return format!(
        "monochromatic-git-policy-cli: the push of this branch reached {holders}, so the commit is backed up there, and the push to \
         {failed_at} is the one that failed. No retry can mend it, so the commit service is not trying that branch through it again.\n{why}"
    );
}

/// What:
///  The words of a retried push that reached an address before it failed at another.
/// Why:
///   The retry's record says the branch is backed up,
/// so the address that failed is the one to look at,
/// and Git's own words follow once,
/// appended by the caller that builds the whole record.
/// Without this line the record said the remote "may or may not have taken that commit" about a commit a remote
/// already held.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the push reached ${arrivedAt.join(', ')}; the push to ${failing} failed\n${shown}`
/// ```
fn reached_then_failed(read: &FailedPushByAddress) -> String {
    let mut holders: String = String::new();
    for holder in read.arrived_at() {
        if !holders.is_empty() {
            holders.push_str(", ");
        }
        holders.push_str(holder);
    }
    let failed_at: &str = read.failing.as_deref().unwrap_or("another address");
    return format!(
        "monochromatic-git-policy-cli: the push of this branch reached {holders}, so the commit is backed up there, and the push to \
         {failed_at} is the one that failed."
    );
}

/// The answers and the words above are controlled beside them.
#[cfg(test)]
#[path = "service_work_failed_push_verdict_tests.rs"]
mod tests;
