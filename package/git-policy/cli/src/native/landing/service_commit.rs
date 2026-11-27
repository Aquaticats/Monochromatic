//! What:
//!  The commit command's path through the resident service:
//! prepare the commit in this process (private index,
//!  policies),
//! send the prepared change to the service,
//! and report what it landed.
//! Why:
//!  The service is the only writer,
//! so a commit takes no lock and polls nothing:
//! preparation and policies run here,
//!  outside the service's serialized step,
//! and the service moves the branch once for a whole batch of commits
//!  (`doc/planning/cli-git-resident-service.md`,
//! "Where preparation and policies run").
//!  Every command line lands through it.
//! The lines `service_commit_args.rs` accepts are built here from the command line
//!  (message and identity options,
//! sign-off,
//! trailers,
//! a reused or fixup message,
//! `-a`,
//! an index commit,
//! `--amend`);
//! every
//!  other line,
//! and every commit that is not on an ordinary branch with an ordinary state,
//! is built by real Git in a scratch
//!  repository and landed as Git made it.
//! Which route a commit takes is decided by `service_route.rs`;
//! what is prepared is
//!  `service_prepare.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = await tryServiceCommit(invocation, checks, request); // undefined only when no service can be reached
//! ```
//!
//! The values every attempt carries stand in `service_commit_attempt_values.rs`,
//! and what the command does with the service's answer is `service_commit_the_answers_of_the_service.rs`;
//! this file holds the order the steps of one attempt run in.

/// The build phase's stretches by name
/// for that record.
use crate::commit_build_part_durations::BuildPartClock;
use crate::commit_timing::CommitTiming;
use crate::commit_transaction::{
    CommitInvocation,
    TransactionResult,
    TransactionStop,
};
use crate::policy_checks::ShippedChecks;
use crate::policy_engine::StageRequest;
use crate::repository_facts::RepositoryFacts;
use crate::service_amend_replayed::{
    AmendedCommit,
    AmendsTree,
    amended_fields,
    the_amends_tree,
};
use crate::service_branch_shows::refused_when_the_branch_switched;
use crate::service_client::{
    ClientConfig,
    Submitted,
};
use crate::service_commit_args::parse_service_commit;
use crate::service_commit_attempt_values::{
    TheAttempt,
    the_attempt,
};
/// The look for the commit a request that was already sent made.
use crate::service_commit_fitting_request::embedded_that_fits;
use crate::service_commit_input_notes_and_embedded_commit::{
    Embedded,
    embedded_commit,
};
/// The request this attempt sends,
/// and the message Git typed into the commit it built.
use crate::service_commit_request_of::{
    RequestOf,
    the_request_of,
};
/// The file a typed message is kept in when the commit cannot be made.
use crate::service_commit_saved_message::keep_the_typed_message;
use crate::service_commit_the_answers_of_the_service::{
    TheAnswerOfTheService,
    WhatTheServiceSaid,
    the_answer_of_the_service,
};
/// Which end of the commit's scratch tree a submitted request leaves it to.
use crate::service_commit_tree_ownership::the_service_never_took;
use crate::service_commit_waiting_notices::submit_saying_where_it_stands;
/// What a commit does when the service answers that the branch moved under it.
pub use crate::service_landed_aftermath::ServiceCommit;
use crate::service_messages::ClientFailure;
use crate::service_prepare::{
    NativeInputs,
    PrepareInputs,
    Prepared,
    Ready,
    prepare,
};
use crate::service_protocol::CommitRequest;
/// Provisioning the service before the first submission.
use crate::service_provision::provision_when_needed;
use crate::service_pruned::{
    content_was_removed,
    reach_service_phase,
};
use crate::service_replay::{
    Retry,
    conflict_result,
};
use crate::transaction_phase::Phase;
use std::path::PathBuf;
use std::time::Instant;

/// What:
///  Lands the commit through the service,
/// for every command line.
/// Why:
///   The service is the only writer of a commit.
/// A machine where none can be reached or started ends the command with the
///  plain message of its platform,
/// before anything was changed;
/// every other ending is the commit's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await serviceCommit(invocation, checks, request, timing)
/// ```
pub fn service_commit<F: RepositoryFacts>(
    invocation: &CommitInvocation<'_>,
    checks: &mut ShippedChecks<F>,
    request: &StageRequest,
    timing: &mut CommitTiming,
) -> Result<ServiceCommit, TransactionStop> {
    // The values every attempt of this command reads and writes: the route to the service, the identity of the request,
    //  what the person's line asked for, and the records a rebuild is made from (`service_commit_attempt_values.rs`).
    let mut the: TheAttempt = the_attempt(invocation, timing)?;
    for attempt in 0usize.. {
        // A rebuild replays what the batches that landed meanwhile captured; the first try replays nothing.
        let mut replay: Option<Retry<'_>> = None;
        if let Some(captured) = the.retry.as_ref() {
            replay = Some(Retry {
                captured,
                attempt: u64::try_from(attempt).unwrap_or(u64::MAX),
            });
        }
        let scratch_name: String =
            format!("monochromatic-git-policy-cli-service-{}-{attempt}", the.id);
        let inputs: PrepareInputs<'_> = PrepareInputs {
            scratch_name: scratch_name.as_str(),
            // A `--fixup=reword:` commit carries the branch's own tree, so the preparation of the attempt that lands it on the
            //  branch as it is now holds no change at all, and an empty commit is exactly what the person asked for.
            allow_empty: invocation.region.allow_empty || the.rewords,
            standard_input: the.standard_input.as_deref(),
            first: attempt == 0,
            replay,
        };
        let preparing: Instant = Instant::now();
        if attempt == 0 {
            // Everything before the first preparation is the one stretch of the command that belongs to no other phase.
            timing.note_route();
        }
        let prepared: Result<Prepared, TransactionStop> =
            prepare(invocation, checks, request, &inputs);
        timing.prepare += preparing.elapsed();
        timing.attempts += 1;
        let mut ready: Box<Ready> = match prepared? {
            Prepared::Conflict(conflict) => {
                // The message is kept so the person can commit again once their files agree with the commit that landed first.
                let kept: Option<PathBuf> = keep_the_typed_message(
                    conflict.git_dir.as_path(),
                    the.id.as_str(),
                    the.last_message.as_deref(),
                );
                let result: TransactionResult = conflict_result(&conflict, kept.as_deref());
                return Ok(ServiceCommit {
                    result,
                    handoff: None,
                });
            }
            Prepared::Ended(result) => {
                // The policies could not read an object this commit had just written: something removed it meanwhile.
                //  The captured bytes are gone, and the worktree is not read again to replace them.
                if content_was_removed(result.events.as_slice()) {
                    return Err(crate::commit_transaction::client_stop(
                        ClientFailure::ContentPruned,
                    ));
                }
                return Ok(ServiceCommit {
                    result,
                    handoff: None,
                });
            }
            Prepared::Ready(ready) => ready,
        };
        // Every stretch of this phase names itself, so a slow build says which of them waited
        //  (`doc/planning/cli-git-commit-latency-under-many-agents.md`).
        timing.begin_build();
        let mut clock: BuildPartClock = BuildPartClock::begin("branch");
        // The branch the first preparation found. A worktree that switched since (the service refuses the landing, and a
        //  new preparation would read the new branch) is a failure like native Git's, with the typed message kept in its file.
        if timing.reference.is_none() {
            timing.reference = Some(ready.reference.spoken());
        }
        // The branch of the first preparation is the one every later preparation must be on.
        if the.first_reference.is_none() {
            the.first_reference = Some(ready.reference.clone());
        }
        if let Some(stop) = refused_when_the_branch_switched(
            the.first_reference.as_ref(),
            &ready.reference,
            the.fixed_message.as_deref(),
        ) {
            return Err(stop);
        }
        if !the.decided {
            the.decided = true;
            clock.next("message");
            if !ready.needs_native
                && let Some(parsed) = parse_service_commit(the.after.as_slice())
            {
                the.embedded = embedded_commit(invocation, parsed)?;
            }
        }
        let native_inputs: &NativeInputs = &ready.inputs;
        // An amend the service builds is made from the commit `HEAD` named when the message was read; if the branch left it
        //  since, the amend is not made. Its tree is the private index's, and native Git refuses an amend that leaves the
        //  commit empty against its first parent (a merge excepted) unless `--allow-empty` says it is meant, which Git then
        //  says itself.
        // An amend whose branch moved is not made as an amend again: its changes become a new commit on the branch as it is
        //  now, and the tree of that commit is the replay the preparation already settled in the private index.
        let lands_as_new: bool = the.amended.is_some() && the.retry.is_some();
        let mut tree: Option<String> = None;
        clock.next("amend tree");
        match the_amends_tree(
            &ready,
            invocation,
            native_inputs,
            the.embedded.as_ref(),
            lands_as_new,
        )? {
            AmendsTree::Tree(written) => tree = Some(written),
            // Git says itself that an amend which leaves the replaced commit as it was is empty.
            AmendsTree::EmptyAgainstTheParent => the.embedded = None,
            AmendsTree::Nothing => {}
        }
        // A request over the line the service reads is refused as too long, and a commit must not fail for the names it
        //  changes: Git builds one that does not fit, as it did before a changed path carried its bytes.
        clock.next("fitting request");
        the.embedded =
            embedded_that_fits(the.embedded, &ready, native_inputs, the.id.as_str(), &tree);
        let native: bool = !lands_as_new && (ready.needs_native || the.embedded.is_none());
        // The embedded commit is used only when the native path is not taken.
        let embedded_in_use: Option<&Embedded> = if native || lands_as_new {
            None
        } else {
            the.embedded.as_ref()
        };
        let amendment: Option<&AmendedCommit> = if lands_as_new {
            the.amended.as_ref()
        } else {
            None
        };
        // The request reads the preparation's values and, when Git builds the commit, creates the shadow repository and runs
        //  Git in it; those stretches name themselves under this clock.
        clock.next("request");
        // The finished shadow this attempt was handed, when the service drew one: the arm that answers the service decides
        //  whether the service removes it, and every other way out of this attempt releases it beside the command.
        let mut handed: Option<crate::shadow_drawn::HandedShadow> = None;
        let (sent, first_parent, typed): (CommitRequest, String, Option<String>) =
            the_request_of(RequestOf {
                amendment,
                embedded_in_use,
                tree: &tree,
                ready: &ready,
                invocation,
                native_inputs,
                // A commit Git built before, whose editor has run and whose branch moved, is built again from the line that rebuild
                //  was made with; every other attempt builds from the person's own line.
                reissued: the.reissued_line.as_ref(),
                id: the.id.as_str(),
                clock: &mut clock,
                service: &the.address,
                handed: &mut handed,
            })?;
        // What an amend carries into the commit a moved branch makes of it, kept once from the attempt that prepared it.
        if the.amended.is_none() && sent.amend {
            clock.next("amend fields");
            the.amended = Some(amended_fields(invocation, &sent, the.embedded.as_ref())?);
        }
        let mut remembered: Option<String> = typed.clone();
        if remembered.is_none()
            && let Some(made) = embedded_in_use
        {
            remembered = Some(made.message.clone());
        }
        the.last_message = remembered;
        // A test removes the prepared content here, between the preparation and the landing.
        drop(clock);
        timing.end_build();
        reach_service_phase(Phase::ServicePrepared)?;
        // The first commit that reaches the service sets up its socket activation, and only then: a commit that was
        //  declined, blocked by a policy, or abandoned in the editor never needs it, and the manager starts the service.
        the.provisioned = provision_when_needed(
            the.provisioning.as_ref(),
            the.provisioned,
            &mut timing.provision,
        )?;
        // The commit's scratch directory is renewed while the commit waits, however long: the sweep of a later command, and the
        //  removal of an abandoned tree, would otherwise take it from under a wait that outlasts its age.
        //  Both bounds of that wait, the renewed directory and the notices a terminal which stopped draining cannot hold, are
        //  in `service_commit_waiting_notices.rs`.
        // This attempt gets its own copy of the client configuration, so the line that submits reads exactly as the control of the
        //  waiting notices pins it (`foundation/notice_stream_that_does_not_hold_its_writer.rs`).
        let config: ClientConfig = the.config.clone();
        let submitted: Result<Submitted, ClientFailure> = submit_saying_where_it_stands(
            &config,
            ready.scratch.0.as_path(),
            &sent,
            &mut timing.submit,
        );
        // A request the service took is one whose tree the service removes once its landing is over, so this process leaves it
        //  alone; only a definite answer that the service never took the request leaves the tree to this process, which removes
        //  it as it ends. A request the service answered without landing it is released by the arm that answers it, beside this
        //  command, once that arm has read the change it captured from the tree's private index.
        ready
            .scratch
            .held_by_the_service(!the_service_never_took(&submitted));
        // The drawn tree's removal follows the same rule, with one difference: a request the service answered as not landed
        //  hands its trees back to this command, so the commit removes them itself once that answer's arm has read what it needs.
        //  A request the service took, answered or not, leaves both trees to the service.
        let handed_back_to_this_command: bool = matches!(
            submitted,
            Ok(Submitted::PrepareAgain { .. } | Submitted::Moved)
        );
        if let Some(held) = handed.as_mut()
            && !handed_back_to_this_command
            && !the_service_never_took(&submitted)
        {
            held.the_service_removes_it();
        }
        // The service answered that this attempt did not land the commit: the rebuild below captures the change from this
        //  attempt's private index, so this is the instant a control holds to prove that index is still there to be read.
        if matches!(
            submitted,
            Ok(Submitted::PrepareAgain { .. } | Submitted::Moved)
        ) {
            reach_service_phase(Phase::ServiceSentBack)?;
        }
        // The service's answer is read where the words of each ending are (`service_commit_the_answers_of_the_service.rs`).
        match the_answer_of_the_service(TheAnswerOfTheService {
            values: &mut the,
            timing,
            invocation,
            submitted,
            ready: ready.as_mut(),
            sent: &sent,
            first_parent,
            typed,
            native,
            lands_as_new,
            attempt,
        })? {
            WhatTheServiceSaid::Landed(commit) => return Ok(*commit),
            WhatTheServiceSaid::Rebuild => {}
        }
    }
    return Err(crate::commit_transaction::client_stop(
        ClientFailure::BranchKeepsChanging {
            branch: String::from("the branch"),
            times: timing.attempts,
        },
    ));
}

/// Service commit controls stay out of the release executable.
#[cfg(test)]
#[path = "service_commit_tests.rs"]
mod tests;
