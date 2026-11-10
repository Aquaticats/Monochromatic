//! What:
//!  What a commit does when the service answered without landing it:
//!  the words for content a prune removed,
//!  and the aftermath of a service that went silent,
//!  failed,
//!  or could not be reached after the request was sent.
//! Why:
//!   Every one of these endings is the commit's,
//!  and each needs the same facts about the attempt:
//!  what was prepared,
//!  what the person typed,
//!  and which commit the request named.
//!  They sit beside the commit path and not in it,
//!  because that path is at the file limit the repository linter enforces
//!  (`monochromatic-lint.config.jsonc`,
//!  `rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! return answerAfterFailure({ failure, sent, ready, invocation, ... }) // the commit the journal names, or the failure's own words
//! ```

use crate::commit_transaction::{CommitInvocation, TransactionStop};
use crate::service_commit::ServiceCommit;
use crate::service_commit_looking_for_what_landed::journal_of_request;
use crate::service_commit_input_notes_and_embedded_commit::Embedded;
use crate::service_pruned::prepared_content_missing;
use crate::service_amend_replayed::AmendedCommit;
use crate::service_landed_aftermath::{LandedAftermath, after_a_silent_or_failed_service};
use crate::service_messages::{ClientFailure, render};
use crate::service_prepare::{NativeInputs, Ready};
use crate::service_protocol::CommitRequest;
use crate::service_client::ClientConfig;
use crate::policy_events::PolicyEvent;
use std::path::{Path, PathBuf};

/// What a commit knows about the attempt the service did not land.
pub(crate) struct FailedAttempt<'a> {
    /// What the service answered.
    pub failure: ClientFailure,
    /// What the attempt sent.
    pub sent: &'a CommitRequest,
    /// The events the preparation captured.
    pub events: &'a [PolicyEvent],
    /// The preparation the attempt read.
    pub ready: &'a Ready,
    /// The invocation whose commit this is.
    pub invocation: &'a CommitInvocation<'a>,
    /// The native inputs of the commit line.
    pub native_inputs: &'a NativeInputs,
    /// The branch value the commit was made on.
    pub first_parent: &'a str,
    /// The file holding a typed message,
    /// when one was kept.
    pub fixed_message: Option<&'a PathBuf>,
    /// The commit the service built from the request,
    /// when it did.
    pub embedded_in_use: Option<&'a Embedded>,
    /// The amend whose change became a new commit,
    /// when the branch moved.
    pub amended_as_new: Option<&'a AmendedCommit>,
    /// The commits earlier attempts prepared and left behind.
    pub unlanded: &'a [String],
    /// The identity the request was sent with.
    pub request_id: &'a str,
    /// How to reach the service afterwards.
    pub config: &'a ClientConfig,
}

/// What:
///  The ending of a request whose objects a prune removed from the repository.
/// Why:
///  The service answers `unavailable` when a commit it was told to land names an object that is gone,
///  and the worktree is not read again to make that commit:
///  the person is told that the content it captured is no longer there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new TransactionStop(render(ClientFailure.ContentPruned))
/// ```
pub(crate) fn content_was_pruned() -> TransactionStop {
    return TransactionStop::Message(render(&ClientFailure::ContentPruned));
}

/// What:
///  The ending of a request the service answered without landing,
///  after the journal was asked whether the commit landed anyway.
/// Why:
///  A commit that landed before its answer was lost is reported as the commit it made,
///  and every other ending is the failure's own words,
///  so nothing is reported as not made when the branch shows it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// afterASilentOrFailedService(failure, { ...landed, journal: journalOfRequest(sent), commit: sent.commit })
/// ```
///
/// # Errors
///
/// `TransactionStop` with the failure's own words when nothing landed.
pub(crate) fn answer_after_a_failure(attempt: FailedAttempt<'_>) -> Result<ServiceCommit, TransactionStop> {
    // Content a prune removed is its own ending: the service answered that the commit names an object that is gone, and the
    //  worktree is not read again to make that commit.
    if matches!(attempt.failure, ClientFailure::Unavailable)
        && prepared_content_missing(
            attempt.native_inputs.capture.layout.common_dir.as_path(),
            attempt.native_inputs.capture.layout.object_directory.as_path(),
            attempt.native_inputs.capture.layout.object_format.as_str(),
            attempt.sent,
        )
    {
        return Err(content_was_pruned());
    }
    let journal: Option<PathBuf> = journal_of_request(attempt.sent);
    let config: ClientConfig = attempt.config.clone();
    return after_a_silent_or_failed_service(
        attempt.failure,
        LandedAftermath {
            invocation: attempt.invocation,
            ready: attempt.ready,
            events: attempt.events.to_vec(),
            native_inputs: attempt.native_inputs,
            commit: attempt.sent.commit.as_deref(),
            first_parent: attempt.first_parent.to_string(),
            fixed_message: attempt.fixed_message,
            embedded_in_use: attempt.embedded_in_use,
            amended_as_new: attempt.amended_as_new,
            unlanded: attempt.unlanded,
            request_id: attempt.request_id,
            journal: journal.as_deref() as Option<&Path>,
            config,
        },
    );
}
