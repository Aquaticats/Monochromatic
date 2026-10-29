//! What:
//!  The conclusion of a settled commit transaction:
//!  the normalization-only landing when the corrections removed every change, otherwise the hook dispatcher,
//!  native preparation in the shadow, the prepared record, the landing loop and the post-landing completion.
//! Why:
//!  The incumbent's `commit-transaction-conclusion.ts` runs these steps in this order: the tree is written from the
//!  stable private index, the dispatcher shim is shared by preparation and the post-landing `post-commit`, the
//!  prepared record is durable before the preparation marker, and the transaction directory is removed only after
//!  the worktree copies are installed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! return await concludeCommitTransaction(settled);
//! ```

use super::capture_invocation::InvocationCapture;
use super::commit_args::{resolve_private_commit_args, split_commit_invocation, without_all_flag};
use super::commit_landing_loop::{LoopInput, LoopOutcome, land_with_replay};
use super::commit_facts::PrivateIndexFacts;
use super::commit_index::write_private_tree;
use super::commit_no_change::{NoChangeInput, complete_no_change_transaction};
use super::commit_preparation::{PREPARED_FILENAME, PreparationHooks, PreparationRequest, read_prepared_commit, run_native_commit};
use super::commit_replay_options::{ReplayOptions, replay_options};
use super::commit_transaction::{CommitInvocation, Selection, TransactionResult, TransactionStop, failed, write_journal_record};
use super::commit_workspace::CommitWorkspace;
use super::commit_worktree_records::{HookChanges, describe_hook_changes, selected_worktree_records};
use super::diagnostic_log::debug;
use super::global_arguments::global_layout;
use super::hook_dispatch_entries::write_hook_dispatch;
use super::hook_dispatch_plan::{HookDispatchPlan, HookPlanInputs, SILENCED_HOOK_EVENTS, compute_hook_dispatch_plan, new_preparation_lease};
use super::policy_checks::ShippedChecks;
use super::policy_engine::StageRequest;
use super::policy_events::PolicyEvent;
use super::policy_pass::PassResult;
use super::post_landing::{LandedTransaction, PostLandingRequest, complete_landed_transaction};
use super::repository_facts::RepositoryFacts;
use super::transaction_git::GitContext;
use super::transaction_journal::{AddedPath, PreparedRecord};
use super::transaction_journal_encode::encode_prepared;
use super::transaction_phase::{Phase, reach_transaction_phase};
use std::ffi::OsString;
use std::path::Path;

/// Module tag for diagnostics.
const TAG: &str = "concludeCommitTransaction";

/// What:
///  Everything the conclusion reads from the settled transaction.
/// Why:
///   The steps share the workspace, the capture, the policies' settled pass and the selection facts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SettledCommitTransaction = { workspace; capture; mode; pass; changedPasses; changedPaths; ... };
/// ```
pub struct ConclusionInput<'a, F: RepositoryFacts> {
    /// The invocation's fixed facts.
    pub invocation: &'a CommitInvocation<'a>,
    /// The published workspace.
    pub workspace: &'a mut CommitWorkspace,
    /// The invocation capture.
    pub capture: &'a InvocationCapture,
    /// The selection facts.
    pub selection: &'a Selection,
    /// The request the policies run under.
    pub request: &'a StageRequest,
    /// The shipped checks over the private index.
    pub checks: &'a mut ShippedChecks<PrivateIndexFacts<F>>,
    /// The settled pass.
    pub pass: PassResult,
    /// How many passes changed content.
    pub changed_passes: u64,
    /// The paths the corrections wrote.
    pub changed_paths: Vec<Vec<u8>>,
    /// The paths the private index changes against the base.
    pub committed_paths: Vec<Vec<u8>>,
    /// The positional pathspec tokens.
    pub pathspecs: &'a [String],
}

/// What:
///  The settled pass's events with the fix summary the corrections earned.
/// Why:
///   The summary appears only after some pass changed candidate bytes, naming each changed path once in byte order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withChangedFixSummary({ result: pass, trigger: 'pre-forward', passes, changedPaths });
/// ```
pub fn events_with_changed_summary(events: Vec<PolicyEvent>, passes: u64, changed_paths: &[Vec<u8>]) -> Vec<PolicyEvent> {
    if passes == 0 {
        return events;
    }
    return events_with_summary(events, passes, changed_paths);
}

/// What:
///  The events with a fix summary appended whatever the pass count.
/// Why:
///   A normalization that removed every change always reports its summary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withFixSummary({ result, trigger: 'pre-forward', passes, changedPaths });
/// ```
pub fn events_with_summary(mut events: Vec<PolicyEvent>, passes: u64, changed_paths: &[Vec<u8>]) -> Vec<PolicyEvent> {
    let mut sorted: Vec<Vec<u8>> = changed_paths.to_vec();
    sorted.sort();
    events.push(PolicyEvent::FixSummary {
        trigger: super::policy_trigger::Trigger::PreForward,
        passes,
        changed_paths: sorted
            .iter()
            .map(|path| { return super::event_path::EventPath::from_git_bytes(path.as_slice()); })
            .collect(),
    });
    return events;
}

/// What:
///  The kept global options and the commit region of the private arguments.
/// Why:
///   Native `git commit` runs in the shadow, so the options that would redirect it away are dropped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// splitCommitInvocation({ args: privateArgs, subcommandIndex });
/// ```
fn private_invocation(pass: &PassResult, pathspecs: &[String], selection: &Selection) -> (Vec<String>, Vec<String>) {
    let text: Vec<String> = pass.arguments.iter().map(|token| { return token.to_string_lossy().into_owned(); }).collect();
    let private_args: Vec<String> =
        resolve_private_commit_args(text.as_slice(), pathspecs, selection.mode, selection.read_only);
    let owned: Vec<OsString> = private_args.iter().map(OsString::from).collect();
    let index: usize = global_layout(owned.as_slice()).prefix_len;
    return split_commit_invocation(private_args.as_slice(), index);
}

/// What:
///  Complete a settled policy pass through a landed commit or a normalization-only reconciliation.
/// Why:
///   This is the incumbent's `concludeCommitTransaction`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return await concludeCommitTransaction(settled);
/// ```
pub fn conclude_commit_transaction<F: RepositoryFacts>(
    input: ConclusionInput<'_, F>,
) -> Result<TransactionResult, TransactionStop> {
    let ConclusionInput {
        invocation,
        workspace,
        capture,
        selection,
        request,
        checks,
        pass,
        changed_passes,
        changed_paths,
        committed_paths,
        pathspecs,
    } = input;
    let context: &GitContext = invocation.context;
    let cwd: &Path = invocation.cwd;
    let repository_root: &Path = capture.layout.repository_root.as_path();
    let timeout_ms: u64 = invocation.config.concurrency.index_lock.unproven_owner_timeout_ms;
    let intended_tree: String = write_private_tree(
        context,
        cwd,
        workspace.landing.commit_index.as_path(),
        workspace.object_directory.as_path(),
    )?;
    let changes = checks.content.install_changes();
    let selected: Vec<AddedPath> = selected_worktree_records(
        context,
        cwd,
        repository_root,
        workspace.landing.commit_index.as_path(),
        changes.as_slice(),
    )?;
    let eligible: bool = !changes.is_empty()
        && !invocation.region.allow_empty
        && !invocation.region.amend
        && !selection.concludes_sequencer;
    if let Some(result) = complete_no_change_transaction(NoChangeInput {
        invocation,
        workspace,
        capture,
        mode: selection.mode,
        eligible,
        intended_tree: intended_tree.as_str(),
        committed_paths: committed_paths.as_slice(),
        selected_worktree_paths: selected.as_slice(),
        events: &pass.events,
        changed_passes,
        changed_paths: changed_paths.as_slice(),
        index_lock_timeout_ms: timeout_ms,
    })? {
        return Ok(result);
    }
    let (global_args, commit_args): (Vec<String>, Vec<String>) = private_invocation(&pass, pathspecs, selection);
    let global_os: Vec<OsString> = global_args.iter().map(OsString::from).collect();
    let lease: String = new_preparation_lease(workspace.landing.directory.as_path())
        .map_err(|error| { return failed(format!("{error:?}")); })?;
    let plan: HookDispatchPlan = compute_hook_dispatch_plan(&HookPlanInputs {
        context,
        cwd,
        global_arguments: global_os.as_slice(),
        common_directory: capture.layout.common_dir.as_path(),
        worktree_root: repository_root,
        lease: lease.as_str(),
        concurrent_commits: invocation.config.concurrency.hooks.concurrent_commits,
        inherited_lease_valid: invocation.inherited_lease_valid,
        environment: invocation.environment,
        snapshot: capture.layout.config_snapshot.as_ref(),
    })
    .map_err(|error| { return failed(format!("{error:?}")); })?;
    write_hook_dispatch(workspace.hooks_directory.as_path(), &plan, invocation.executable)
        .map_err(|error| { return failed(error.to_string()); })?;
    let hooks: PreparationHooks = PreparationHooks {
        directory: workspace.hooks_directory.clone(),
        silenced_events: SILENCED_HOOK_EVENTS.iter().map(|event| { return String::from(*event); }).collect(),
    };
    let native_args: Vec<String> = if selection.staged_all {
        without_all_flag(commit_args.as_slice())
    } else {
        commit_args.clone()
    };
    let preparation: PreparationRequest<'_> = PreparationRequest {
        capture,
        shadow: workspace.landing.shadow.as_path(),
        hooks: &hooks,
        commit_index: workspace.landing.commit_index.as_path(),
        global_args: global_args.as_slice(),
        commit_args: native_args.as_slice(),
    };
    let status: i32 = run_native_commit(context, cwd, &preparation).map_err(|error| { return TransactionStop::Message(error.to_string()); })?;
    if status != 0 {
        return Err(TransactionStop::NativeExit(status));
    }
    let prepared = read_prepared_commit(context, workspace.landing.shadow.as_path())?;
    let hook_changes: HookChanges = describe_hook_changes(
        context,
        cwd,
        workspace.object_directory.as_path(),
        intended_tree.as_str(),
        prepared.tree_oid.as_str(),
    )?;
    debug(
        TAG,
        format!("prepared {}; hooks changed {} paths", prepared.oid, hook_changes.paths.len()).as_str(),
    );
    let mut prepared_records: Vec<AddedPath> = selected.clone();
    prepared_records.extend(hook_changes.worktree_records.iter().cloned());
    write_journal_record(
        workspace.landing.directory.as_path(),
        PREPARED_FILENAME,
        encode_prepared(&PreparedRecord {
            shadow_path: workspace.landing.shadow.display().to_string(),
            prepared_oid: prepared.oid.clone(),
            signed: prepared.signed,
            intended_tree_oid: intended_tree.clone(),
            committed_paths: committed_paths.clone(),
            added_paths: Vec::new(),
            selected_worktree_paths: prepared_records.clone(),
        })
        .as_str(),
    )?;
    reach_transaction_phase(Phase::PreparationDone, None)?;
    let settled_events: Vec<PolicyEvent> = events_with_changed_summary(pass.events.clone(), changed_passes, changed_paths.as_slice());
    let options: ReplayOptions = replay_options(commit_args.as_slice());
    let outcome: LoopOutcome = land_with_replay(LoopInput {
        invocation,
        workspace,
        capture,
        mode: selection.mode,
        request,
        checks,
        prepared: &prepared,
        settled_events: settled_events.clone(),
        options: &options,
        global_args: global_args.as_slice(),
        read_only_selection: selection.read_only,
        committed_paths: committed_paths.clone(),
        added_paths: Vec::new(),
        worktree_records: prepared_records.clone(),
        hook_changes,
        index_lock_timeout_ms: timeout_ms,
        reserve_after_lost_races: invocation.config.concurrency.landing.reserve_after_lost_races,
    })?;
    let (oid, loop_events, final_worktree, final_added): (String, Vec<PolicyEvent>, Vec<AddedPath>, Vec<AddedPath>) =
        match outcome {
            LoopOutcome::Failed { events, exit_code } => {
                return Ok(TransactionResult {
                    events,
                    exit_code,
                    landed_oid: None,
                });
            }
            LoopOutcome::Landed {
                oid,
                events,
                worktree_records,
                added_paths,
            } => (oid, events, worktree_records, added_paths),
        };
    // Before cleanup, so an interruption leaves the journal for startup recovery to finish the worktree copies.
    let landed: LandedTransaction = LandedTransaction {
        landed_oid: oid.clone(),
        branch: capture.layout.target_ref.clone(),
        cwd: cwd.to_path_buf(),
        repository_root: repository_root.to_path_buf(),
        directory: workspace.landing.directory.clone(),
        shadow: workspace.landing.shadow.clone(),
        git_dir: capture.layout.git_dir.clone(),
        registry_root: capture.layout.registry_root.clone(),
        real_index: capture.layout.real_index_path.clone(),
        added_paths: final_added,
        worktree_records: final_worktree,
        config_snapshot: capture.layout.config_snapshot.clone(),
    };
    let completed = complete_landed_transaction(PostLandingRequest {
        context,
        plan: &plan,
        landed: &landed,
    });
    if let Err(error) = completed {
        return Err(TransactionStop::Message(error.to_string()));
    }
    // The completion removed the shadow and the directory, so the workspace has nothing left to dispose of; on a
    // failure the landing's mark stays, and recovery finishes what the completion did not.
    workspace.finish_transaction();
    workspace.mark_disposed();
    let mut events: Vec<PolicyEvent> = settled_events;
    events.extend(loop_events);
    return Ok(TransactionResult {
        events,
        exit_code: 0,
        landed_oid: Some(oid),
    });
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_conclusion_tests.rs"]
mod tests;
