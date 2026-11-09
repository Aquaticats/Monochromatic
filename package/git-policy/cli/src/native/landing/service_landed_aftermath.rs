//! What:
//!  What a landed commit leaves the command to do:
//!  the result it returns,
//!  and the work after the landing,
//!  reported as far as it goes.
//! Why:
//!   Two paths end in a landed commit:
//!  the answer the service sent,
//! and the silence it left after the branch read proved the commit is in the branch.
//! Both report the landing,
//!  hand the after-commit work over and exit 0,
//! so one function builds them and neither can fail a commit that landed
//! (findings 6 of the third review and 3 of the fourth).
//!  A refusal from that work is a warning:
//! the commit is made,
//!  and the owner's rule is that commits never fail.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = theLandedCommit({ invocation, ready, events, commit, firstParent, ... });
//! ```

/// The command being run.
use crate::commit_transaction::{CommitInvocation, TransactionResult, TransactionStop};
/// The prepared commit and what it came to.
use crate::service_prepare::{NativeInputs, Ready};
/// The commit the service built from the request,
/// when it did.
use crate::service_commit_input_notes_and_embedded_commit::Embedded;
/// The commit an amend makes when the branch moved under it.
use crate::service_amend_replayed::AmendedCommit;
/// The landing report.
use crate::service_commit_requests_and_landing_report::report_landing;
/// The result and the after-commit work.
use crate::service_commit::ServiceCommit;
/// The words of a failure.
use crate::service_messages::{ClientFailure, render};
/// What the branch shows once the service went silent.
use crate::service_branch_shows::the_silent_service_after_a_prepare;
/// The look for the commit a request that was already sent made.
use crate::service_commit_looking_for_what_landed::{LookForWhatLanded, looking_for_what_landed};
/// The after-commit work handed to the service.
use crate::service_handoff::ServiceHandoff;
/// How to reach the service.
use crate::service_client::ClientConfig;
/// The events the result carries.
/// A diagnostics line.
use crate::diagnostic_log::warn;
/// A path on the filesystem.
use std::path::{Path, PathBuf};

/// What:
///  Everything a commit that landed still needs,
/// gathered so the answer that landed and the silence that ended in one
///  take the same path.
pub(crate) struct LandedAftermath<'landed> {
    /// The command being run.
    pub(crate) invocation: &'landed CommitInvocation<'landed>,
    /// What the preparation came to.
    pub(crate) ready: &'landed Ready,
    /// The events the preparation captured,
    /// which the result carries.
    pub(crate) events: Vec<crate::policy_events::PolicyEvent>,
    /// The native inputs of the command.
    pub(crate) native_inputs: &'landed NativeInputs,
    /// The commit the command prepared,
    /// and nothing for a commit the service builds itself from the request.
    pub(crate) commit: Option<&'landed str>,
    /// The branch value it was made on.
    pub(crate) first_parent: String,
    /// A typed message that was saved and used.
    pub(crate) fixed_message: Option<&'landed PathBuf>,
    /// The commit the service built from the request,
    /// when it did.
    pub(crate) embedded_in_use: Option<&'landed Embedded>,
    /// Whether Git built the commit.
    pub(crate) native: bool,
    /// The amend this commit was,
    /// when the branch moved and its changes became a new commit on top of it.
    pub(crate) amended_as_new: Option<&'landed AmendedCommit>,
    /// Commits earlier attempts prepared and left behind.
    pub(crate) unlanded: &'landed [String],
    /// The identity the request was sent with,
    /// which names its batch in the service's journal.
    pub(crate) request_id: &'landed str,
    /// The journal folder of the worktree's service,
    /// nothing when the environment names no state folder or the Git directory no instance.
    pub(crate) journal: Option<&'landed Path>,
    /// How to reach the service afterwards.
    pub(crate) config: ClientConfig,
}

/// What:
///  The command's result for a commit that landed,
/// with the work after it reported as far as it goes.
/// Why:
///   A commit the branch shows as made is a landed commit even when its answer never arrived,
/// and the work after a landing is reported by one function for either path,
/// so neither can fail a commit that landed
/// (findings 6 of the third review and 3 of the fourth).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = theLandedCommit(input);
/// ```
pub(crate) fn the_landed_commit(input: LandedAftermath<'_>) -> ServiceCommit {
    // A commit the service builds itself has no commit of its own to name, so the branch value it was made on stands in.
    let commit: String = input.commit.unwrap_or(input.ready.base.as_str()).to_string();
    report_after_a_landing(
        report_landing(
            input.invocation,
            input.ready,
            input.native_inputs,
            commit.as_str(),
            input.first_parent.as_str(),
            input.fixed_message,
            input.embedded_in_use,
            input.native,
            input.amended_as_new,
            input.unlanded,
        ),
    );
    let handoff: ServiceHandoff = ServiceHandoff {
        config: input.config,
        repository: input.ready.repository.clone(),
        git_dir: input.native_inputs.capture.layout.git_dir.clone(),
        reference: input.ready.reference.clone(),
        parent: input.first_parent,
    };
    let result: TransactionResult = TransactionResult { events: input.events, exit_code: 0, landed_oid: Some(commit), uses_lfs: input.ready.uses_lfs };
    return ServiceCommit { result, handoff: Some(handoff) };
}

/// What:
///  What the command does when the service answered with a failure:
///  the landed commit when a bounded look finds the commit the request made or the branch shows it was made,
///  and the failure's words otherwise.
/// Why:
///   An answer that was lost is not proof that the service never took the request:
/// the audit of service restarts saw
///  "this commit was not made ... Nothing in the repository changed"
/// while the commit was on the branch,
/// and the honest "not known" while the service landed it seconds later.
/// The journal names the commit the request's batch made or is making,
/// so one bounded look turns both into the true answer,
/// and a landed commit is never reported as a failure
/// (findings 6 of the third review and 3 of the fourth).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// afterASilentOrFailedService(failure, placed);
/// ```
pub(crate) fn after_a_silent_or_failed_service(failure: ClientFailure, placed: LandedAftermath<'_>) -> Result<ServiceCommit, TransactionStop> {
    // The failures a lost answer leaves: the service went silent, or it broke the connection after the request was written.
    let found: Option<String> = match failure {
        ClientFailure::ServiceSilent | ClientFailure::MayNotHaveLanded => looking_for_what_landed(LookForWhatLanded {
            context: placed.invocation.context,
            cwd: placed.invocation.cwd,
            journal: placed.journal,
            id: placed.request_id,
            wait: crate::wait_ceilings::LOOKING_FOR_A_LANDED_COMMIT,
        }),
        _ => None,
    };
    if let Some(commit) = found {
        // The commit is made: the command reports the landing and succeeds, whatever the answer said.
        let mut landed: LandedAftermath<'_> = placed;
        landed.commit = Some(commit.as_str());
        return Ok(the_landed_commit(landed));
    }
    let worded: Option<ClientFailure> = match failure {
        // The one silence that could not state a fact reads the branch once and states what it shows.
        ClientFailure::ServiceSilent => the_silent_service_after_a_prepare(placed.invocation.context, placed.invocation.cwd, placed.ready.reference.as_str(), placed.commit),
        other => Some(other),
    };
    let Some(reported) = worded else {
        // The branch holds the commit this command prepared, so the commit was made, and the command succeeds.
        return Ok(the_landed_commit(placed));
    };
    return Err(TransactionStop::Message(render(&reported)));
}

/// What:
///  Take the work a landed commit leaves behind as far as it goes,
/// and report a refusal as a warning.
/// Why:
///   The commit landed,
/// and the owner's rule is that commits never fail:
/// so one function reports the work after a landing for both the acknowledged landing and the silence that ended in one,
/// and neither can turn a refusal into the command's failure (finding 3 of the fourth review).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reportAfterALanding(reportLanding(...));
/// ```
fn report_after_a_landing(reported: Result<(), TransactionStop>) {
    let Err(stop) = reported else {
        return;
    };
    return match stop {
        TransactionStop::NativeExit(code) => warn("serviceCommit", format!("the commit landed and the work after it exited {code}").as_str()),
        TransactionStop::Message(words) => warn("serviceCommit", format!("the commit landed and the work after it failed: {words}").as_str()),
        TransactionStop::Failed { code, message } => warn("serviceCommit", format!("the commit landed and the work after it failed ({code:?}): {message}").as_str()),
    };
}
