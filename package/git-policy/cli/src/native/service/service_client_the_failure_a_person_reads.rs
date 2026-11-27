//! What:
//!  The plain failure a person reads for each code the service answers with,
//!  and the read-only one that says whether a repository move froze this repository.
//! Why:
//!  The service answers with a code and nothing a person can read;
//!  this is the one place a code becomes a sentence of `service_messages.rs`,
//!  so the waiting path never spells the words and no two paths can disagree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const failureFor = (code: FailureCode, request: CommitRequest): ClientFailure => { ... };
//! ```

use crate::service_messages::ClientFailure;
use crate::service_protocol::{
    CommitRequest,
    FailureCode,
};
use std::path::Path;
/// What:
///  The failure for a repository whose filesystem is read-only:
/// the frozen-move one when the state of a repository move says it froze this repository,
/// the plain read-only one otherwise.
/// Why:
///  The sentence about a move is told only when the move's own record says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readOnlyFailure(repository, stateFolder)
/// ```
pub fn read_only_failure(
    repository: &str,
    state_folder: Option<&Path>,
) -> ClientFailure {
    if let Some(folder) = state_folder
        && crate::repository_move_freeze::freeze_covers(folder, Path::new(repository))
    {
        return ClientFailure::RepositoryFrozen(repository.to_string());
    }
    return ClientFailure::RepositoryReadOnly(repository.to_string());
}

/// The user-facing failure for a code the service gave.
pub fn failure_for(
    code: FailureCode,
    request: &CommitRequest,
) -> ClientFailure {
    let repository: String = request.repository.display().to_string();
    return match code {
        FailureCode::RepositoryFull => ClientFailure::RepositoryFull(repository),
        FailureCode::RepositoryReadOnly => read_only_failure(
            repository.as_str(),
            crate::repository_move_freeze::move_state_folder().as_deref(),
        ),
        FailureCode::PermissionDenied => ClientFailure::PermissionDenied(repository),
        FailureCode::IndexInUse => ClientFailure::IndexInUse(String::from("Another program")),
        FailureCode::ReferenceLocked => ClientFailure::ReferenceLocked {
            branch: request.reference.short().spoken(),
            words: String::new(),
        },
        FailureCode::Rejected => ClientFailure::Refused,
        FailureCode::BranchKeepsChanging => ClientFailure::BranchKeepsChanging {
            branch: request.reference.short().spoken(),
            times: crate::service_land::LANDING_ATTEMPTS,
        },
        FailureCode::BranchMoved => ClientFailure::BranchMoved {
            branch: request.reference.short().spoken(),
        },
        FailureCode::StepTooSlow => ClientFailure::StepTooSlow,
        FailureCode::Unavailable => ClientFailure::Unavailable,
    };
}
