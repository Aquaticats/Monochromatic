//! What:
//!  Which end of a commit's scratch tree a submitted request leaves it to.
//! Why:
//!  The tree holds the private index and the conclusion copies a landing reads,
//!  so it is removed by the service that took the request,
//!  once that landing is over.
//!  A client that removed it because a submission failed would take those from under a landing
//!  that is still running,
//!  and an unanswered request is not proof that nothing took it.
//!  The other way round,
//!  a request the service answered as not landed is one whose private index this command still reads:
//!  the tree of the attempt that captured the change is written as a tree for the rebuild
//!  (`service_replay.rs`,
//!  `captured_tree`),
//!  so a service that removed it would take the index from under that read,
//!  and the commit would fail with "No such file or directory" or,
//!  worse,
//!  rebuild from the empty tree that Git writes for an index that is gone.
//!  Kept in a file of its own because the commit path is at its line bound.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const goesBack = (result) => result.kind === 'still-with-service' || ... || result.kind === 'prepare-again' || result.kind === 'moved';
//! ```

/// What a submission came to.
use crate::service_client::Submitted;
/// Why a submission failed.
use crate::service_messages::ClientFailure;

/// What:
///  Whether the commit's scratch tree goes back to this process,
///  which then removes it as it goes out of scope.
/// Why:
///   A request the service took and landed is one whose tree the service removes once its landing is over,
///  so this process leaves it alone.
///   A request the service answered as not landed is one whose private index this process still reads to capture the
///  change for the rebuild,
///  so the service leaves that tree alone and this process removes it after the read.
///  An unanswered result is not proof that nothing took the request:
///  a withdrawal the service did not confirm,
///  a silence and a broken connection can all leave a branch write running,
///  and removing the tree here would take the private index and the conclusion copies
///  from under that landing.
///  Such a tree waits for the service,
///  and when no service ever collected it,
///  for the age sweep of the repository scratch directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const goesBack = (result) => result.kind === 'prepare-again' || result.kind === 'moved' || result.kind === 'still-with-service' || ...;
/// ```
pub(crate) fn the_tree_goes_back_to_this_process(result: &Result<Submitted, ClientFailure>) -> bool {
    return match result {
        // The service answered that the commit is not on the branch, and the next attempt captures the change from this
        //  attempt's private index: the service leaves that tree to this process.
        Ok(Submitted::PrepareAgain { .. } | Submitted::Moved) => true,
        Ok(Submitted::Landed { .. }) => false,
        Err(failure) => matches!(
            failure,
            // The service answered that it withdrew the request, so it will never land and its tree is no landing's to read.
            ClientFailure::StillWithService { .. }
                // The service refused the request before it queued it.
                | ClientFailure::Refused
                // No service was reached at all, so nothing was written and nothing can be running.
                | ClientFailure::DidNotStart
                | ClientFailure::NoActivation(_)
                | ClientFailure::OtherAccount
                | ClientFailure::NoCommonVersion
                | ClientFailure::OtherRelease
                | ClientFailure::ProgramMissing(_)
                | ClientFailure::CouldNotStart(_)
                | ClientFailure::UnitInTheWay(_)
        ),
    };
}

/// The decision is controlled in the test module.
#[cfg(test)]
#[path = "service_commit_tree_ownership_tests.rs"]
mod tests;
