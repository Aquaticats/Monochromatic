//! What:
//!  The word each failure of a commit goes by in the timing record.
//! Why:
//!  A client that watched the commit stop knows what stopped it,
//!  and the record used to say `unidentified` for every one of these:
//!  a service of another release that would not exit,
//!  a service manager that answered nothing,
//!  a service that answered the request with a failure.
//!  Seven of the 136 commits of release 0.6.1-alpha.12's first three hours stopped this way,
//!  and each one's person had been told the cause in words.
//!  The words live here rather than beside the renderings,
//!  because `service_messages.rs` is at the line bound the repository linter enforces
//!  (`rust/max-lines` in `monochromatic-lint.config.jsonc`).
//!  Every word is a literal of this crate,
//!  so the record still carries no path and no message,
//!  and the words each failure renders stay the person's to read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // landing/serviceMessagesOfAFailure.ts
//! export const failureCode = (failure: ClientFailure): string => ...;
//! ```

use crate::service_messages::ClientFailure;

/// What:
///  The failure as the timing record names it,
///  in one word that says what stopped the commit.
/// Why:
///  A reader of a record has no terminal to read,
///  so the one thing the client knew must reach the line:
///  which of the ways a commit can fail did this one fail by.
///  The word is one of this crate's own literals,
///  never a path and never a message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// failureCode(failure) // 'refused' | 'other-release' | 'did-not-start' | ...
/// ```
pub fn failure_code(failure: &ClientFailure) -> &'static str {
    return match failure {
        ClientFailure::DidNotStart => "did-not-start",
        ClientFailure::NothingAnswered { .. } => "nothing-answered",
        ClientFailure::NoActivation(_) => "no-activation",
        ClientFailure::OtherAccount => "other-account",
        ClientFailure::MayNotHaveLanded => "may-not-have-landed",
        ClientFailure::NoCommonVersion => "no-common-version",
        ClientFailure::OtherRelease => "other-release",
        ClientFailure::RepositoryFull(_) => "repository-full",
        ClientFailure::RepositoryReadOnly(_) => "repository-read-only",
        ClientFailure::PermissionDenied(_) => "permission-denied",
        ClientFailure::RepositoryFrozen(_) => "repository-frozen",
        ClientFailure::BranchKeepsChanging { .. } => "branch-keeps-changing",
        ClientFailure::BranchMoved { .. } => "branch-moved",
        ClientFailure::ConclusionStateChanged { .. } => "conclusion-state-changed",
        ClientFailure::BranchMovedMessageSaved { .. } => "branch-moved-message-saved",
        ClientFailure::BranchSwitched { .. } => "branch-switched",
        ClientFailure::ServiceSilent => "service-silent",
        ClientFailure::ServiceSilentAfterPreparing { .. } => "service-silent-after-preparing",
        ClientFailure::StillWithService { .. } => "still-with-service",
        ClientFailure::WithdrawalUnconfirmed { .. } => "withdrawal-unconfirmed",
        ClientFailure::Interrupted { .. } => "interrupted",
        ClientFailure::InterruptedNotKnown { .. } => "interrupted-not-known",
        ClientFailure::StepTooSlow => "step-too-slow",
        ClientFailure::Refused => "refused",
        ClientFailure::IndexInUse(_) => "index-in-use",
        ClientFailure::ReferenceLocked { .. } => "reference-locked",
        ClientFailure::Unavailable => "unavailable",
        ClientFailure::ProgramMissing(_) => "program-missing",
        ClientFailure::CouldNotStart(_) => "could-not-start",
        ClientFailure::SocketPathTooLong { .. } => "socket-path-too-long",
        ClientFailure::ServiceElsewhere { .. } => "service-elsewhere",
        ClientFailure::UnitInTheWay(_) => "unit-in-the-way",
        ClientFailure::WorkNotHandedOver(_) => "work-not-handed-over",
        ClientFailure::ContentPruned => "content-pruned",
    };
}
