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
use crate::service_pruned::the_objects_it_names_are_gone;
use crate::service_amend_replayed::AmendedCommit;
use crate::service_land_after_the_service_stopped::{WithoutTheService, land_after_the_service_stopped};
use crate::service_landed_aftermath::{LandedAftermath, after_a_silent_or_failed_service, the_landed_commit};
use crate::service_messages::{ClientFailure, render};
use crate::service_prepare::{NativeInputs, Ready};
use crate::service_protocol::CommitRequest;
use crate::service_client::ClientConfig;
use crate::policy_events::PolicyEvent;
use std::io::Write;
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
        && the_objects_it_names_are_gone(attempt.sent, &attempt.native_inputs.capture.layout)
    {
        return Err(content_was_pruned());
    }
    let journal: Option<PathBuf> = journal_of_request(attempt.sent);
    let config: ClientConfig = attempt.config.clone();
    // What:
    //  A service that gave no answer at all is a commit this process makes itself.
    // Why:
    //  The rule is that a commit never fails where native Git would succeed, and native Git, on this repository with no service
    //  at all, makes the commit.
    //  The commit was prepared here already, so what the service would have done is done here instead
    //  (`service_land_after_the_service_stopped.rs`),
    //  and only a commit that cannot be made here either ends as a failure.
    if gave_no_answer(&attempt.failure) {
        return made_here(attempt);
    }
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

/// What:
///  Whether the service gave no answer at all,
///  which is the commit this process makes for itself.
/// Why:
///   A service that went silent, broke the connection after the request was written, answered nothing at the address, or could
///  not be confirmed to have given the request up has said nothing this command can act on.
///  Every other failure is the service's considered answer about the commit (a refusal, a branch that moved, an
///  interrupted command), and a commit made here would override a decision the service made with the person's own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gaveNoAnswer = (failure) => ['service-silent', 'may-not-have-landed', 'nothing-answered', 'did-not-start', 'withdrawal-unconfirmed'].includes(failure.kind)
/// ```
fn gave_no_answer(failure: &ClientFailure) -> bool {
    return matches!(
        failure,
        ClientFailure::ServiceSilent
            | ClientFailure::MayNotHaveLanded
            | ClientFailure::NothingAnswered { .. }
            | ClientFailure::DidNotStart
            | ClientFailure::WithdrawalUnconfirmed { .. }
    );
}

/// What:
///  The command's end for a commit the service gave no answer to:
///  the commit made here,
///  kept here,
///  or the reason neither could be done.
/// Why:
///   The one rule is that a commit never fails where native Git would succeed,
///  and a commit that lands here is reported exactly as one the service landed,
///  with the after-commit work handed over by the same call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = await landAfterTheServiceStopped(context, sent); ...
/// ```
fn made_here(attempt: FailedAttempt<'_>) -> Result<ServiceCommit, TransactionStop> {
    let journal: Option<PathBuf> = journal_of_request(attempt.sent);
    let outcome: WithoutTheService = land_after_the_service_stopped(attempt.invocation.context, attempt.sent, journal.as_deref());
    match outcome {
        WithoutTheService::Landed { commit, the_index_waits } => {
            if the_index_waits {
                let _written = writeln!(
                    std::io::stderr(),
                    "{}: {commit} is on the branch; the index update of this commit is recorded and the next command of this worktree installs it.",
                    crate::wrapper_names_old_and_new::PROGRAM.current
                );
            }
            let landed: LandedAftermath<'_> = LandedAftermath {
                invocation: attempt.invocation,
                ready: attempt.ready,
                events: attempt.events.to_vec(),
                native_inputs: attempt.native_inputs,
                commit: Some(commit.as_str()),
                first_parent: attempt.first_parent.to_string(),
                fixed_message: attempt.fixed_message,
                embedded_in_use: attempt.embedded_in_use,
                amended_as_new: attempt.amended_as_new,
                unlanded: attempt.unlanded,
                request_id: attempt.request_id,
                journal: journal.as_deref(),
                config: attempt.config.clone(),
            };
            return Ok(the_landed_commit(landed));
        }
        WithoutTheService::Held { commit: _commit, words } => {
            let _written = writeln!(std::io::stderr(), "{}: {words}", crate::wrapper_names_old_and_new::PROGRAM.current);
            return Ok(ServiceCommit { result: crate::commit_transaction::TransactionResult { events: Vec::new(), exit_code: 0, landed_oid: None, uses_lfs: false }, handoff: None });
        }
        // A commit that could not be made here either keeps what the person staged and wrote, and the words say where the
        //  commit it made is kept; the words carry that sentence already.
        WithoutTheService::NotMade { commit, words } => {
            let _said = commit;
            return Err(TransactionStop::Message(words));
        }
    }
}
