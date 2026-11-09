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

use crate::service_commit_input_notes_and_embedded_commit::managed_client_config;
use crate::service_server::Bind;
use crate::service_provision::ProvisionHost;
use crate::service_commit_input_notes_and_embedded_commit::route_for_this_worktree;
use crate::service_commit_fitting_request::embedded_that_fits;
use crate::service_commit_input_notes_and_embedded_commit::{Embedded, arguments_after_subcommand, embedded_commit, standard_input_for, switched_branch_name};
use crate::service_commit_requests_and_landing_report::{request_for_embedded, request_for_git_built};
/// The file a typed message is kept in when the commit cannot be made.
use crate::service_commit_saved_message::keep_the_typed_message;
use crate::commit_timing::CommitTiming;
use crate::commit_transaction::{CommitInvocation, TransactionResult, TransactionStop, failed};
use crate::diagnostic_log::warn;
use crate::policy_checks::ShippedChecks;
use crate::policy_engine::StageRequest;
use crate::random_id::random_uuid;
use crate::repository_facts::RepositoryFacts;
use crate::service_client::{ClientConfig, Submitted};
use crate::service_commit_waiting_notices::submit_saying_where_it_stands;
use crate::service_commit_args::{opens_editor, parse_service_commit, with_fixed_message};
use crate::service_handoff::ServiceHandoff;
use crate::service_commit_rebuild_ceiling::{AfterAPrepareAgain, after_a_prepare_again, may_rebuild};
/// Provisioning the service before the first submission.
use crate::service_provision::provision_when_needed;
use crate::service_branch_shows::{branch_moved, display_text};
/// The result of a commit that landed,
/// and the work after it.
use crate::service_landed_aftermath::{LandedAftermath, after_a_silent_or_failed_service, the_landed_commit};
use crate::service_messages::{ClientFailure, render};
/// Which end of the commit's scratch tree a submitted request leaves it to.
use crate::service_commit_tree_ownership::the_service_never_took;
use crate::service_native::{BuildInput, GitBuilt, build_with_git};
use crate::service_prepare::{HandedBackTree, NativeInputs, PrepareInputs, Prepared, Ready, prepare};
use crate::service_replay::{Captured, Retry, conflict_result, retry_of};
use crate::service_amend_replayed::{AmendedCommit, AmendsTree, amended_fields, the_amends_tree, the_amendment_lands_as_new};
use crate::service_protocol::CommitRequest;
use crate::service_pruned::{content_was_removed, prepared_content_missing, reach_service_phase};
use crate::transaction_phase::Phase;
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Instant;

/// What the service path returns:
/// the transaction result for the lifecycle,
/// and the handoff when the commit landed.
pub struct ServiceCommit {
    /// What the policies and the landing came to.
    pub result: TransactionResult,
    /// How to leave the after-commit work for the service;
    /// absent when nothing landed.
    pub handoff: Option<ServiceHandoff>,
}

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
    let (address, provisioning): (Bind, Option<ProvisionHost>) = route_for_this_worktree(invocation, timing)?;
    let after: &[OsString] = arguments_after_subcommand(invocation.arguments, invocation.subcommand_index);
    let id: String = match random_uuid() {
        Ok(made) => made,
        Err(error) => return Err(failed(format!("an identity for this commit could not be made: {error:?}"))),
    };
    // The identity is the key that joins this commit's timing record to the service's journal entry for it.
    timing.request = Some(id.clone());
    let standard_input: Option<Vec<u8>> = standard_input_for(invocation)?;
    let mut provisioned: bool = false;
    // Whether an editor may have run, which decides what a moved branch costs the person: a message to keep or a signature to redo.
    let editor_possible: bool = opens_editor(after);
    let config: ClientConfig = managed_client_config(address, provisioning.as_ref());
    // The name carries the request's identity and, per attempt, that attempt's number: the service removes the tree of the attempt
    //  whose landing is over, and the next attempt's tree must be another directory, or that removal would take it.
    // The service builds the commits it can from the command line; real Git builds the rest in a scratch repository. Which
    //  it is gets decided once the preparation has passed every cheap decision (head, configuration, index, policies), so a
    //  commit the service never takes starts no Git for a message or an identity.
    let mut embedded: Option<Embedded> = None;
    let mut decided: bool = false;
    // The message the person typed once the editor has run, kept in a file so a commit moved by a tip that changed is built again from it.
    let mut fixed_message: Option<PathBuf> = None;
    // The message of the command as the last preparation settled it, kept for the person when the commit cannot be made.
    let mut last_message: Option<String> = None;
    // What the previous preparation captured when the service sent the commit back because the branch moved: the next
    //  preparation merges it onto the branch as it is and never reads the worktree again.
    let mut retry: Option<Captured> = None;
    // The branch (or `HEAD`) the first preparation found, which every later preparation must find again.
    let mut first_reference: Option<String> = None;
    // The commits Git built that the branch moving made useless: Git printed a summary for each, so the wrapper names them
    //  once the commit that did land is known.
    let mut unlanded: Vec<String> = Vec::new();
    // The fields of the commit an amend prepared, kept from the attempt that prepared it: an amend whose branch moved lands
    //  them as a new commit on the branch as it is now (`service_amend_replayed.rs`), because `git commit --amend` would
    //  replace the commit that moved the branch.
    let mut amended: Option<AmendedCommit> = None;
    // The first build's start: the ceiling on rebuilding is counted from it.
    let first_build_began: Instant = Instant::now();
    for attempt in 0usize.. {
        // A rebuild replays what the batches that landed meanwhile captured; the first try replays nothing.
        let mut replay: Option<Retry<'_>> = None;
        if let Some(captured) = retry.as_ref() {
            replay = Some(Retry { captured, attempt: u64::try_from(attempt).unwrap_or(u64::MAX) });
        }
        let scratch_name: String = format!("monochromatic-git-policy-cli-service-{id}-{attempt}");
        let inputs: PrepareInputs<'_> = PrepareInputs {
            scratch_name: scratch_name.as_str(),
            allow_empty: invocation.region.allow_empty,
            standard_input: standard_input.as_deref(),
            first: attempt == 0,
            replay,
        };
        let preparing: Instant = Instant::now();
        let prepared: Result<Prepared, TransactionStop> = prepare(invocation, checks, request, &inputs);
        timing.prepare += preparing.elapsed();
        timing.attempts += 1;
        let mut ready: Box<Ready> = match prepared? {
            Prepared::Conflict(conflict) => {
                // The message is kept so the person can commit again once their files agree with the commit that landed first.
                let kept: Option<PathBuf> = keep_the_typed_message(conflict.git_dir.as_path(), id.as_str(), last_message.as_deref());
                let result: TransactionResult = conflict_result(&conflict, kept.as_deref());
                return Ok(ServiceCommit { result, handoff: None });
            }
            Prepared::Ended(result) => {
                // The policies could not read an object this commit had just written: something removed it meanwhile.
                //  The captured bytes are gone, and the worktree is not read again to replace them.
                if content_was_removed(result.events.as_slice()) {
                    return Err(TransactionStop::Message(render(&ClientFailure::ContentPruned)));
                }
                return Ok(ServiceCommit { result, handoff: None });
            }
            Prepared::Ready(ready) => ready,
        };
        // The branch the first preparation found. A worktree that switched since (the service refuses the landing, and a
        //  new preparation would read the new branch) is a failure like native Git's, with the typed message kept in its file.
        let building: Instant = Instant::now();
        if timing.reference.is_none() {
            timing.reference = Some(ready.reference.clone());
        }
        // The branch of the first preparation is the one every later preparation must be on.
        if first_reference.is_none() {
            first_reference = Some(ready.reference.clone());
        }
        let first_branch: String = match first_reference.as_ref() {
            Some(known) => known.clone(),
            None => ready.reference.clone(),
        };
        if first_branch != ready.reference {
            let failure: ClientFailure = ClientFailure::BranchSwitched {
                branch: switched_branch_name(first_branch.as_str()),
                now: switched_branch_name(ready.reference.as_str()),
                file: fixed_message.as_deref().map(display_text),
            };
            return Err(TransactionStop::Message(render(&failure)));
        }
        if !decided {
            decided = true;
            if !ready.needs_native
                && let Some(parsed) = parse_service_commit(after)
            {
                embedded = embedded_commit(invocation, parsed)?;
            }
        }
        let native_inputs: &NativeInputs = &ready.inputs;
        // An amend the service builds is made from the commit `HEAD` named when the message was read; if the branch left it
        //  since, the amend is not made. Its tree is the private index's, and native Git refuses an amend that leaves the
        //  commit empty against its first parent (a merge excepted) unless `--allow-empty` says it is meant, which Git then
        //  says itself.
        // An amend whose branch moved is not made as an amend again: its changes become a new commit on the branch as it is
        //  now, and the tree of that commit is the replay the preparation already settled in the private index.
        let lands_as_new: bool = amended.is_some() && retry.is_some();
        let mut tree: Option<String> = None;
        match the_amends_tree(&ready, invocation, native_inputs, embedded.as_ref(), lands_as_new)? {
            AmendsTree::Tree(written) => tree = Some(written),
            // Git says itself that an amend which leaves the replaced commit as it was is empty.
            AmendsTree::EmptyAgainstTheParent => embedded = None,
            AmendsTree::Nothing => {}
        }
        // A request over the line the service reads is refused as too long, and a commit must not fail for the names it
        //  changes: Git builds one that does not fit, as it did before a changed path carried its bytes.
        embedded = embedded_that_fits(embedded, &ready, native_inputs, id.as_str(), &tree);
        let native: bool = !lands_as_new && (ready.needs_native || embedded.is_none());
        // The embedded commit is used only when the native path is not taken.
        let embedded_in_use: Option<&Embedded> = if native || lands_as_new { None } else { embedded.as_ref() };
        let mut typed: Option<String> = None;
        let amendment: Option<&AmendedCommit> = if lands_as_new { amended.as_ref() } else { None };
        let (sent, first_parent): (CommitRequest, String) = match amendment {
            Some(fields) => {
                // The amended commit's own fields, on the tree the replay settled, with the branch as it is now as the
                //  single parent.
                let (sent, parent, subject) =
                    the_amendment_lands_as_new(fields, &ready, invocation, native_inputs, id.as_str(), arguments_after_subcommand(invocation.arguments, invocation.subcommand_index))?;
                typed = Some(subject);
                (sent, parent)
            }
            None => match embedded_in_use {
            None => {
                let layout_prefix: usize = crate::global_arguments::global_layout(native_inputs.arguments.as_slice()).prefix_len;
                let rebuilt_arguments: Option<Vec<OsString>> = match fixed_message.as_deref() {
                    Some(file) => with_fixed_message(native_inputs.arguments.as_slice(), layout_prefix, file),
                    None => None,
                };
                let built: GitBuilt = build_with_git(&BuildInput {
                    context: invocation.context,
                    cwd: invocation.cwd,
                    capture: &native_inputs.capture,
                    scratch: ready.scratch.0.as_path(),
                    commit_index: native_inputs.commit_index.as_path(),
                    arguments: rebuilt_arguments.as_deref().unwrap_or(native_inputs.arguments.as_slice()),
                    pathspecs: native_inputs.pathspecs.as_slice(),
                    mode: native_inputs.mode,
                    staged_all: native_inputs.staged_all,
                    selected_privately: native_inputs.selected_privately,
                })?;
                let parent: String = built.parents.first().cloned().unwrap_or_default();
                typed = Some(built.message.clone());
                let sent: CommitRequest = request_for_git_built(&ready, native_inputs, id.as_str(), built.commit, invocation.region.amend);
                (sent, parent)
            }
            Some(made) => {
                let sent: CommitRequest = request_for_embedded(&ready, native_inputs, id.as_str(), made, &tree);
                // The new commit's first parent: the tip it was made on, or for an amend the replaced commit's own parent.
                let parent: String = match made.source.as_ref() {
                    Some(replaced) => replaced.parents.first().cloned().unwrap_or_default(),
                    None => ready.base.clone(),
                };
                (sent, parent)
            }
            },
        };
        // What an amend carries into the commit a moved branch makes of it, kept once from the attempt that prepared it.
        if amended.is_none() && sent.amend {
            amended = Some(amended_fields(invocation, &sent, embedded.as_ref())?);
        }
        let mut remembered: Option<String> = typed.clone();
        if remembered.is_none()
            && let Some(made) = embedded_in_use
        {
            remembered = Some(made.message.clone());
        }
        last_message = remembered;
        // A test removes the prepared content here, between the preparation and the landing.
        timing.build += building.elapsed();
        reach_service_phase(Phase::ServicePrepared)?;
        // The first commit that reaches the service sets up its socket activation, and only then: a commit that was
        //  declined, blocked by a policy, or abandoned in the editor never needs it, and the manager starts the service.
        provisioned = provision_when_needed(provisioning.as_ref(), provisioned, &mut timing.provision)?;
        // The commit's scratch directory is renewed while the commit waits, however long: the sweep of a later command, and the
        //  removal of an abandoned tree, would otherwise take it from under a wait that outlasts its age.
        //  Both bounds of that wait, the renewed directory and the notices a terminal which stopped draining cannot hold, are
        //  in `service_commit_waiting_notices.rs`.
        let submitted: Result<Submitted, ClientFailure> = submit_saying_where_it_stands(&config, ready.scratch.0.as_path(), &sent, &mut timing.submit);
        // A request the service took is one whose tree the service removes once its landing is over, so this process leaves it
        //  alone; only a definite answer that the service never took the request leaves the tree to this process, which removes
        //  it as it ends.
        ready.scratch.held_by_the_service(!the_service_never_took(&submitted));
        match submitted {
            Ok(Submitted::Landed { commit, .. }) => {
                let after_landing: Instant = Instant::now();
                let outcome: ServiceCommit = the_landed_commit(LandedAftermath {
                    invocation,
                    ready: &ready,
                    events: ready.events.clone(),
                    native_inputs,
                    commit: Some(commit.as_str()),
                    first_parent,
                    fixed_message: fixed_message.as_ref(),
                    embedded_in_use,
                    native,
                    amended_as_new: if lands_as_new { amended.as_ref() } else { None },
                    unlanded: unlanded.as_slice(),
                    config: config.clone(),
                });
                timing.after_landing += after_landing.elapsed();
                return Ok(outcome);
            }
            Ok(Submitted::PrepareAgain { landed, .. }) => {
                // The service did not land this attempt, so it left the tree to this command, which has just read the change it
                //  captured from it: the tree is released beside the command now.
                ready.scratch.release_beside_the_command();
                match after_a_prepare_again(native, sent.amend, &ready, invocation, landed, attempt, first_build_began) {
                    AfterAPrepareAgain::Stopped(stop) => return Err(stop),
                    AfterAPrepareAgain::Retry(next) => retry = Some(next),
                }
            }
            Ok(Submitted::Moved) => {
                // Nothing landed on this attempt's tree, so the service left it to this command: whatever this arm comes to, the
                //  tree is released beside the command once the change it captured has been read from it.
                let _released: HandedBackTree = HandedBackTree(ready.scratch.0.clone());
                ready.scratch.held_by_the_service(true);
                let branch: String = ready.reference.trim_start_matches("refs/heads/").to_string();
                // An amend whose branch moved is not made as an amend again: its changes become a new commit on the branch as
                //  it is now, leaving the commit that moved it as it is (the owner's choice, 2026-11-09).
                //  No one waits, and no command is refused for another's commit.
                if sent.amend && amended.is_some() {
                    if !may_rebuild(branch.as_str(), attempt + 1, first_build_began) {
                        return Err(TransactionStop::Message(render(&ClientFailure::BranchKeepsChanging { branch, times: attempt + 1 })));
                    }
                    // The replay merges the amended tree onto the new tip by the same rule an ordinary commit's replay uses.
                    retry = Some(retry_of(invocation.context, invocation.cwd, &ready, Vec::new())?);
                    continue;
                }
                // Nobody typed or chose anything, and the commit does not depend on the tip it was made on: Git builds it
                //  again on the new tip (and signs it again), up to the bound.
                //  A commit that concludes a merge, a cherry-pick or a revert depends on the state the worktree holds, not on
                //  the tip: its parents come from `MERGE_HEAD`, `CHERRY_PICK_HEAD` or `REVERT_HEAD`, so the same rebuild makes
                //  it on the branch as it is now, and the preparation refuses when that state is no longer the same.
                let conclusion_may_move: bool = ready.cleanup.is_some();
                if native && !sent.amend && !editor_possible && !ready.interactive && (!ready.fixed_parents || conclusion_may_move) {
                    if !may_rebuild(branch.as_str(), attempt + 1, first_build_began) {
                        return Err(TransactionStop::Message(render(&ClientFailure::BranchKeepsChanging { branch, times: attempt + 1 })));
                    }
                        // Git builds it again from the tree this build captured, not from the worktree.
                    retry = Some(retry_of(invocation.context, invocation.cwd, &ready, Vec::new())?);
                    unlanded.extend(sent.commit.clone());
                    continue;
                }
                if native && editor_possible && let Some(text) = typed.as_deref() {
                    // The person typed this message: keep it where they can reuse it.
                    let file: PathBuf = native_inputs.capture.layout.git_dir.join(format!("monochromatic-git-policy-cli-saved-message-{}", id.get(..8).unwrap_or(id.as_str())));
                    match std::fs::write(&file, text) {
                        Ok(()) => {
                            // After the editor the message is fixed: while the bound allows, Git builds the commit again on the new
                            //  tip from `-F <file>`, with no editor and no person, unless the line borrows its author or its parents.
                            let prefix: usize = crate::global_arguments::global_layout(native_inputs.arguments.as_slice()).prefix_len;
                            let repeatable: bool = !sent.amend
                                && !ready.interactive
                                && (!ready.fixed_parents || conclusion_may_move)
                                && with_fixed_message(native_inputs.arguments.as_slice(), prefix, file.as_path()).is_some();
                            if repeatable && may_rebuild(branch.as_str(), attempt + 1, first_build_began) {
                                fixed_message = Some(file);
                                retry = Some(retry_of(invocation.context, invocation.cwd, &ready, Vec::new())?);
                                unlanded.extend(sent.commit.clone());
                                continue;
                            }
                            let failure: ClientFailure = ClientFailure::BranchMovedMessageSaved { branch, file: file.display().to_string(), amend: sent.amend };
                            return Err(TransactionStop::Message(render(&failure)));
                        }
                        Err(error) => warn("serviceCommit", format!("saving the typed message failed: {error}").as_str()),
                    }
                }
                return Err(branch_moved(&ready));
            }
            Err(ClientFailure::Unavailable)
                if prepared_content_missing(
                    native_inputs.capture.layout.common_dir.as_path(),
                    native_inputs.capture.layout.object_directory.as_path(),
                    native_inputs.capture.layout.object_format.as_str(),
                    &sent,
                ) =>
            {
                // The service could not land the request because an object it names is gone from the repository, and the
                //  worktree is not read again to make it.
                return Err(TransactionStop::Message(render(&ClientFailure::ContentPruned)));
            }
            Err(failure) => {
                return after_a_silent_or_failed_service(
                    failure,
                    LandedAftermath {
                        invocation,
                        ready: &ready,
                        events: ready.events.clone(),
                        native_inputs,
                        commit: sent.commit.as_deref(),
                        first_parent,
                        fixed_message: fixed_message.as_ref(),
                        embedded_in_use,
                        native,
                        amended_as_new: if lands_as_new { amended.as_ref() } else { None },
                        unlanded: unlanded.as_slice(),
                        config: config.clone(),
                    },
                );
            }
        }
    }
    return Err(TransactionStop::Message(render(&ClientFailure::BranchKeepsChanging { branch: String::from("the branch"), times: timing.attempts })));
}

/// Service commit controls stay out of the release executable.
#[cfg(test)]
#[path = "service_commit_tests.rs"]
mod tests;
