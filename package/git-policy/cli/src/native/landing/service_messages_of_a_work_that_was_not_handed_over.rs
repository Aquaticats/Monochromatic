//! What:
//!  The failure a hand-off reports when work it left for the commit service could not be handed over,
//!  in the words of what stopped it.
//! Why:
//!   The commit landed,
//! so the words of the wake's own failure are false when they reach the person:
//! `NoCommonVersion` says
//!  "this commit was not made ... Nothing in the repository changed",
//! and the caller renders whatever the hand-off returned
//!  (`commit_command.rs`).
//! The audit of service restarts and version skew (2026-11-09) printed exactly that after a commit that
//!  was on the branch:
//! the service was killed after the branch moved,
//! the unit named another release,
//! and the hand-off met it.
//! The words that are true are the ones for work that was not handed over,
//! and they name the reason and the remedy.
//! They live beside the rendering of every failure rather than inside the hand-off,
//! because the reasons are words and the hand-off's code is at the line bound the repository linter enforces
//!  (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // landing/serviceMessagesOfAWorkThatWasNotHandedOver.ts
//! export const notHandedOver = (failure: ClientFailure): ClientFailure => ...;
//! ```

use crate::service_client_what_became_of_a_wake::Woken;
use crate::service_messages::ClientFailure;

/// What:
///  The failure for work that could not be handed to the commit service because no service read the wake that names it.
/// Why:
///   The records are on disk and a later command of the repository runs them,
///  so the commit is saved and nothing of the work is lost;
///  what a wake the service did not read costs is the backup push's own moment,
///  and the words say what became of the wake so the person can decide whether to push now.
///  The wake's own failure text is never used here:
///  it speaks about a commit that was not made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handed = (woken) => ({ kind: 'work-not-handed-over', reason: woken.whyNothingTookIt() });
/// ```
pub(crate) fn not_handed_over_woken(woken: Woken) -> ClientFailure {
    return ClientFailure::WorkQueuedUnconfirmed(woken.why_nothing_took_it());
}

/// What:
///  The failure for work that could not be handed to the commit service after the commit landed.
/// Why:
///   Every reason here says what became of the work,
///   and the one failure whose own words cannot be the whole reason is a wait that ended without a greeting:
///   its reason is the two facts the wait remembered,
///   so it is asked of the file that words them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handed = (failure) => ({ kind: 'work-not-handed-over', reason: reasonOf(failure) });
/// ```
pub(crate) fn not_handed_over(failure: ClientFailure) -> ClientFailure {
    if let Some(worded) = crate::service_messages_of_a_start_that_was_asked_and_did_not_come::work_not_handed_over(&failure) {
        return worded;
    }
    let reason: &str = match &failure {
        ClientFailure::NoActivation(_) => "this account cannot start the commit service on demand",
        ClientFailure::ManagerDidNotAnswer { .. } => "the user service manager did not answer within the time one exchange is given",
        ClientFailure::ServiceSilent => "the commit service stopped answering",
        ClientFailure::NoCommonVersion => "the commit service that is running was started from a newer release of this git wrapper",
        ClientFailure::OtherRelease => "a commit service started from another release of this git wrapper is still running",
        ClientFailure::ProgramMissing(_) | ClientFailure::CouldNotStart(_) | ClientFailure::UnitInTheWay(_) | ClientFailure::Unavailable => "the commit service could not be reached",
        _ => "the commit service could not be told about it",
    };
    // What: A wake that could not be made leaves the records queued just as a wake nothing read does, so the same words say
    //       both: what is known, and the two ways the commit reaches the remote.
    // Why:  The reasons this is asked of are all of them wakes the service was not reached or not heard by, and the record was
    //       written before the wake (`service_handoff.rs`).
    return ClientFailure::WorkQueuedUnconfirmed(String::from(reason));
}
