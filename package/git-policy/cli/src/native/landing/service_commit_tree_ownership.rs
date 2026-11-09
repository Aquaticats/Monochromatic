//! What:
//!  Which end of a commit's scratch tree a submitted request leaves it to.
//! Why:
//!  The tree holds the private index and the conclusion copies a landing reads,
//!  so it is removed by the service that took the request,
//!  once that landing is over.
//!  A client that removed it because a submission failed would take those from under a landing
//!  that is still running,
//!  and an unanswered request is not proof that nothing took it.
//!  Kept in a file of its own because the commit path is at its line bound.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const neverTook = (result) => result.kind === 'still-with-service' || ...;
//! ```

/// What a submission came to.
use crate::service_client::Submitted;
/// Why a submission failed.
use crate::service_messages::ClientFailure;

/// What:
///  Whether a submission result is a definite answer that the service never took the request,
///  so the commit's scratch tree goes back to this process and is removed as it ends.
/// Why:
///   A request the service took is one whose tree the service removes once its landing is over,
///  so this process leaves it alone.
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
/// const neverTook = (result) => result.kind === 'still-with-service' || result.kind === 'refused' || result.kind === 'did-not-start' || ...;
/// ```
pub(crate) fn the_service_never_took(result: &Result<Submitted, ClientFailure>) -> bool {
    if the_service_asks_another_preparation(result) {
        // The service answered that the commit is prepared again, so it keeps the tree for this process: the retry reads the
        //  private index it names, and this process removes it once that retry is made.
        return true;
    }
    let Err(failure) = result else { return false };
    return matches!(
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
    );
}

/// What:
///  Whether the service's answer says the commit is prepared again,
///  so this process still reads the tree it named.
/// Why:
///   A branch that moved under the build,
/// and a change the landing wants merged onto the new tip,
///  both make the next preparation read the private index of this one;
/// the service keeps that tree for the answer and this process removes it,
///  which is the same rule as for a request no service ever took.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const asksAnother = (result) => result.kind === 'moved' || result.kind === 'prepare-again';
/// ```
fn the_service_asks_another_preparation(result: &Result<Submitted, ClientFailure>) -> bool {
    return match result {
        Ok(Submitted::Moved) | Ok(Submitted::PrepareAgain { .. }) => true,
        Ok(Submitted::Landed { .. }) | Err(_) => false,
    };
}

/// The decision is controlled in the test module.
#[cfg(test)]
#[path = "service_commit_tree_ownership_tests.rs"]
mod tests;
