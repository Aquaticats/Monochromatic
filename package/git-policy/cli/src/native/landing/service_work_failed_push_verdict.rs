//! What:
//!  The verdict of one push Git refused,
//!  from what the push did address by address and from Git's own words.
//! Why:
//!  Three readings decide it, and the addresses come first
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
//!    (the review of issue 6293, finding 2, which found the notice saying the commit was only on this computer
//!    while a remote held it);
//!    when the address it failed at may come back,
//!    the plan is kept, so that address still gets the commit when it returns;
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
//! verdictOfAFailedPush(result, branchRef, repository) // { verdict, report? }
//! ```

/// The push that failed,
/// whose addresses and words decide the verdict.
use crate::auto_push::AutoPushResult;
/// The addresses that can never be a remote,
/// and the lines the person reads for them.
use crate::push_address_that_cannot_be_a_remote::{CannotBeARemote, cannot_be_a_remote, words, words_after_delivery};
/// What one failed push did, address by address.
use crate::push_attempts_of_a_failed_push::{FailedPushByAddress, failed_push_by_address};
/// What a failed push's own words mean.
use crate::push_retry_failure::{final_failure, plain_words, unrecognised_rejection};
/// The verdicts a push's settlement reads.
use crate::service_work::Verdict;
use std::path::Path;

/// What:
///  What one refused push comes to,
///  and the words the next command shows once when the push delivered.
/// Why:
///   A failure the service gives up on forgets the branch's plan and writes one record,
/// and a retry keeps the plan and promises it in the record;
/// the caller settles the item with the answer this function gives.
/// A push that reached an address has nothing to retry and nothing to fail,
/// so its words are a report,
/// which the notice directory keeps apart from failures
/// (`push_failure_notice_reports.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SettledFailedPush = { verdict: Verdict; report?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SettledFailedPush {
    /// What the drain settles the item as.
    pub(crate) verdict: Verdict,
    /// The words the next command shows once,
    /// and nothing for a push that reached no address.
    pub(crate) report: Option<String>,
}

/// What:
///  What one refused push comes to.
/// Why:
///   The address Git failed at decides whether a plan ends,
/// and what the push reached decides whether a failure is reported at all:
/// a branch a remote holds is not a branch to retry or to fail.
/// The configuration's other addresses never decide anything on their own,
/// which is what the review of this branch found wrong twice
/// (findings 2 and 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const read = failedPushByAddress({ output: result.shown, destinationRef: branchRef, configured: result.pushAddresses });
/// const failing = read.failing && cannotBeARemote(read.failing);
/// if (read.arrived) return failing ? { verdict: Done, report: wordsAfterDelivery(...) } : { verdict: Retry(reachedThenFailed(read, result.shown)) };
/// if (failing) return { verdict: Failed(words(...) + result.shown) };
/// return { verdict: fromWords(result.shown), report: undefined };
/// ```
pub(crate) fn verdict_of_a_failed_push(result: AutoPushResult, branch_ref: &str, repository: &Path) -> SettledFailedPush {
    let read: FailedPushByAddress = failed_push_by_address(result.shown.as_str(), branch_ref, result.push_addresses.as_slice());
    let failing: Option<(&str, CannotBeARemote)> = match read.failing.as_deref() {
        Some(address) => match cannot_be_a_remote(address) {
            Some(reason) => Some((address, reason)),
            None => None,
        },
        None => None,
    };
    if read.arrived {
        return match failing {
            Some((address, reason)) => SettledFailedPush {
                verdict: Verdict::Done,
                report: Some(format!("{}\n{}", words_after_delivery(address, reason, read.arrived_at().as_slice()), result.shown)),
            },
            None => SettledFailedPush { verdict: Verdict::Retry(reached_then_failed(&read, result.shown.as_str())), report: None },
        };
    }
    if let Some((address, reason)) = failing {
        return SettledFailedPush { verdict: Verdict::Failed(format!("{}\n{}", words(address, reason), result.shown)), report: None };
    }
    return SettledFailedPush { verdict: verdict_from_the_words(result.shown.as_str(), repository), report: None };
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
///  The words of a retried push that reached an address before it failed at another.
/// Why:
///   The retry's record says the branch is backed up,
/// so the address that failed is the one to look at,
/// and Git's own words follow.
/// Without this line the record said the remote "may or may not have taken that commit" about a commit a remote
/// already held.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the push reached ${arrivedAt.join(', ')}; the push to ${failing} failed\n${shown}`
/// ```
fn reached_then_failed(read: &FailedPushByAddress, shown: &str) -> String {
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
         {failed_at} is the one that failed.\n{shown}"
    );
}
