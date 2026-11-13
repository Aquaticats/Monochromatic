//! What:
//!  The verdict of one push Git refused,
//!  from where the push was sent and from Git's own words.
//! Why:
//!  Two readings decide it,
//! and the address comes first:
//!  - a push whose address can never be a remote ends its plan at once,
//!    with one line that says so
//!    (issue 6293,
//!    the owner's decision of 2026-11-13,
//!    "End the plan at once,
//!    say so once").
//!    Git's words cannot tell such an address from one that may come back,
//!    so the address decides
//!    (`push_address_that_cannot_be_a_remote.rs`),
//!    and no retry can make it work;
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
//! verdictOfAFailedPush(result, repository) // Failed(words) | Retry(output)
//! ```

/// The push that failed,
/// whose addresses and words decide the verdict.
use crate::auto_push::AutoPushResult;
/// The addresses that can never be a remote,
/// and the one line the person reads for them.
use crate::push_address_that_cannot_be_a_remote::{first_that_cannot_be_remote, words};
/// What a failed push's own words mean.
use crate::push_retry_failure::{final_failure, plain_words, unrecognised_rejection};
/// The verdicts a push's settlement reads.
use crate::service_work::Verdict;
use std::path::Path;

/// What:
///  What one refused push comes to:
///  a failure the service gives up on,
///  with the words for the person,
///  or a retry of the branch.
/// Why:
///   A failure the service gives up on forgets the branch's plan and writes one record,
/// and a retry keeps the plan and promises it in the record;
/// the caller settles the item with the answer this function gives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cause = notARemote(result.pushAddresses); if (cause) Failed(words(cause) + output) else fromWords(output)
/// ```
pub(crate) fn verdict_of_a_failed_push(result: AutoPushResult, repository: &Path) -> Verdict {
    if let Some((address, why)) = first_that_cannot_be_remote(result.push_addresses.as_slice()) {
        return Verdict::Failed(format!("{}\n{}", words(address, why), result.shown));
    }
    return match final_failure(result.shown.as_str()) {
        Some(cause) => Verdict::Failed(match plain_words(cause) {
            Some(words) => format!("{words}\n{}", result.shown),
            None => result.shown,
        }),
        None => {
            // A rejection by the server whose reason is not known is tried again, and named so that it can be learned.
            if let Some(line) = unrecognised_rejection(result.shown.as_str()) {
                crate::service_warn!(
                    "the server rejected the push {} for a reason this release does not know, so it is tried again: {line}",
                    crate::service_timing::in_the_repository_at(repository)
                );
            }
            Verdict::Retry(result.shown)
        }
    };
}
