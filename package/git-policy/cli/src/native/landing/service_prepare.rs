//! What:
//!  The preparation of a commit that goes through the resident service:
//! the private index,
//! the selection UI,
//! the policies,
//!  the settled changes,
//! and the facts the landing needs,
//! for every command line.
//! Why:
//!  The service lands what is prepared here,
//! so the preparation covers every shape of commit:
//! a branch,
//! a detached head or a
//!  branch that does not exist yet,
//! an ordinary commit or the conclusion of a merge,
//! a cherry-pick or a revert,
//! a selection
//!  by path,
//! by `--include`,
//! by `--patch`/`--interactive` or by a pathspec file,
//! and a policy that rewrote a file.
//! Whatever
//!  Git alone can answer (an unmerged index,
//! nothing to commit) is marked for Git to answer in the scratch repository,
//! so its
//!  words and its exit status are the person's.
//! Nothing here locks anything,
//! and nothing is registered
//!  (`doc/planning/cli-git-resident-service.md`,
//! "Landing every command line").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const prepared = await prepare(invocation, checks, request, scratchName, attempt);
//! ```

use crate::service_prepare_ready_shapes::ReplayConflict;
pub use crate::service_prepare_ready_shapes::{NativeInputs, Prepared, Ready, Scratch};
use crate::service_prepare_ready_shapes::ReadyShape;
pub use crate::service_prepare_changes_config_and_pathspec_steps::{config_needs_native, config_signs};
use crate::service_prepare_changes_config_and_pathspec_steps::{changes_of, normalized, pathspec_file_for, zero_object};
use crate::capture_invocation::{InvocationCapture, base_revision, capture_invocation_base, capture_invocation_layout};
use crate::commit_prepare_part_durations::PreparePartClock;
use crate::private_index_request::{CaptureIndexes, indexed_request};
use crate::commit_conclusion::events_with_changed_summary;
use crate::commit_convergence::{CommitConvergence, CommitConvergenceInput, converge_commit_policies};
use crate::commit_facts::PrivateIndexFacts;
use crate::commit_index::{
    IndexSelection, initialize_commit_index, initialize_retry_index, list_changed_index_paths, list_unmerged_index_paths, prepare_interactive_selection,
    write_private_tree,
};
use crate::commit_staged_deletions::{STAGED_DELETION_CODE, STAGED_DELETION_CORE_ID, staged_deletion_message, staged_deletions_present_on_disk};
use crate::commit_transaction::{
    CommitInvocation, Selection, TransactionResult, TransactionStop, base_object, decide_selection, failed, private_checks, settled_passes, staged_at_base,
};
use crate::commit_worktree_records::selected_worktree_records;
use crate::conclusion_cleanup::{CONCLUSION_STATE_FILES, SEQUENCER_DIRECTORY};
use crate::policy_checks::ShippedChecks;
use crate::policy_engine::{StageRequest, pass_exit_code};
use crate::policy_events::PolicyEvent;
use crate::policy_pass::{PassResult, run_policy_pass};
use crate::private_index_stage::PrivateStage;
use crate::repository_facts::RepositoryFacts;
use crate::service_pruned::reach_service_phase;
use crate::transaction_phase::Phase;
use crate::service_protocol::{Change, Cleanup};
use crate::merge_second_pass::Merged as MergedFile;
use crate::service_replay::{MergeSite, Merged, Retry, already_committed_result, capture_stamp, first_landed, merge_captured, word_merge_notice};
use crate::external_hooks::warn_external_hooks;
use crate::preparation_notice::deliver_notice;
use crate::standard_error_descriptor::StandardErrorDescriptor;
use crate::traditional_hook_warning::warn_traditional_hooks;
use crate::transaction_git::{GitContext, GitRequest, run_git_checked};
use crate::transaction_journal::{AddedPath, Base, Conclusion, RefFormat, SymbolicHead, TransactionMode};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  The staging start that puts the working copy of every unmerged file into the commit's own index,
/// each named with `:(top,literal)` magic.
/// Why:
///   Every name is a pathspec the wrapper wrote,
/// and this start skips the caller's prefix,
/// so every mode the caller set is removed from the child's environment:
/// under literal mode the names would match nothing,
/// and two glob modes at once make Git refuse the start and fail the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unmergedStagingRequest = (cwd, index, objects, paths) => ownPathspecs(indexedRequest(cwd, ['add', '--all', '--', ...paths.map((path) => `:(top,literal)${path}`)], index, objects));
/// ```
fn unmerged_staging_request(cwd: &Path, index: &Path, object_directory: &Path, paths: &[Vec<u8>]) -> GitRequest {
    let mut staging: Vec<OsString> = vec![
        OsString::from("add"),
        OsString::from("--all"),
        OsString::from("--"),
    ];
    for path in paths {
        let mut spec: Vec<u8> = b":(top,literal)".to_vec();
        spec.extend_from_slice(path.as_slice());
        staging.push(<OsString as std::os::unix::ffi::OsStringExt>::from_vec(spec));
    }
    return indexed_request(cwd, staging.as_slice(), Some(index), Some(object_directory)).own_pathspecs();
}

/// What the preparation reads that does not change between two preparations of one command.
pub struct PrepareInputs<'a> {
    /// The name of the commit's scratch directory under the Git directory.
    pub scratch_name: &'a str,
    /// Whether an empty commit is asked for.
    pub allow_empty: bool,
    /// The bytes of standard input when a pathspec file is read from it.
    pub standard_input: Option<&'a [u8]>,
    /// Whether this is the first preparation of the command,
    /// which says what a configuration warns about.
    pub first: bool,
    /// The change the first preparation captured,
    /// when the service sent the commit back because the branch moved:
    /// it is
    ///  merged onto the branch as it is now,
    /// and the worktree is not read again.
    pub replay: Option<Retry<'a>>,
}

/// Prepares the commit once against the branch as it is now.
pub fn prepare<F: RepositoryFacts>(
    invocation: &CommitInvocation<'_>,
    checks: &mut ShippedChecks<F>,
    request: &StageRequest,
    inputs: &PrepareInputs<'_>,
) -> Result<Prepared, TransactionStop> {
    let context: &GitContext = invocation.context;
    let cwd: &Path = invocation.cwd;
    let region = invocation.region;
    // The stretches of this preparation, one name at each seam; the Git children inside them are taken out of their figures,
    //  because the step ledger records those apart (`commit_prepare_part_durations.rs`).
    let mut part: PreparePartClock = PreparePartClock::begin("layout");
    let layout = capture_invocation_layout(context, cwd, region.amend)?;
    if inputs.first {
        // The descriptor itself, not the standard library's shared handle: a worker left at the ceiling holds no lock this
        //  command's own writes would wait for (`foundation/standard_error_descriptor.rs`).
        warn_traditional_hooks(layout.config_snapshot.as_ref(), StandardErrorDescriptor);
        warn_external_hooks(layout.config_snapshot.as_ref(), StandardErrorDescriptor);
    }
    part.next("unmergedBefore");
    // Named paths beside another session's unfinished merge or stopped cherry-pick land as an ordinary commit, one in conflict
    //  included: the owner's rule is "Committing a conflicted path is fine. Warn and commit anyway, even if conflict markers will
    //  be committed." The paths in conflict are listed here so the warning can name those the commit takes. `-a` stages the working
    //  copy of every tracked file, an unmerged one included, so it commits their markers too and is listed the same way.
    let names_paths: bool = !region.pathspecs.is_empty() || region.pathspec_from_file.is_some();
    let unmerged_before: Vec<Vec<u8>> = if region.all || (matches!(layout.conclusion, Conclusion::Merge | Conclusion::CherryPick) && names_paths) {
        list_unmerged_index_paths(context, cwd, layout.real_index_path.as_path())?
    } else {
        Vec::new()
    };
    part.next("capture");
    let selection: Selection = decide_selection(region, layout.conclusion);
    let reference: String = match &layout.symbolic_head {
        SymbolicHead::Branch(name) => name.clone(),
        SymbolicHead::Detached => String::from("HEAD"),
    };
    let detached: bool = layout.symbolic_head == SymbolicHead::Detached;
    let named: bool = selection.mode == TransactionMode::ExplicitPath;
    let capture: InvocationCapture = capture_invocation_base(context, cwd, layout)?;
    part.next("stagedDeletions");
    let unborn: bool = matches!(capture.base, Base::Unborn);
    // Read from the snapshot the layout already holds, so nothing starts Git for the message or the identities before the
    //  cheap decisions above are made, and before a service is provisioned.
    let config_native: bool = config_needs_native(capture.layout.config_snapshot.as_ref());
    let base: String = base_revision(&capture);
    let announced_base: String = match &capture.base {
        Base::Commit(oid) => oid.clone(),
        Base::Unborn => zero_object(capture.layout.empty_tree_oid.as_str()),
    };
    let region_arguments: &[OsString] = &invocation.arguments[invocation.subcommand_index + 1..];
    let mut pathspecs: Vec<OsString> = Vec::new();
    for at in &region.pathspecs {
        pathspecs.push(region_arguments[*at].clone());
    }
    let retry: Option<&Retry<'_>> = inputs.replay.as_ref();
    if named && retry.is_none() {
        // Before any private staging, so Git's "nothing to commit" or its force-add hint never stands in for the reason.
        let present: Vec<Vec<u8>> = staged_deletions_present_on_disk(
            context,
            cwd,
            capture.layout.repository_root.as_path(),
            capture.layout.real_index_path.as_path(),
            base.as_str(),
            pathspecs.as_slice(),
        )?;
        if !present.is_empty() {
            return Ok(Prepared::Ended(TransactionResult {
                events: vec![PolicyEvent::CoreFinding {
                    core_id: STAGED_DELETION_CORE_ID,
                    code: STAGED_DELETION_CODE,
                    message: staged_deletion_message(present.as_slice()),
                }],
                exit_code: 1,
                landed_oid: None,
                uses_lfs: false,
            }));
        }
    }
    part.next("scratch");
    let amends: bool = region.amend;
    // The tree goes where the sweep of the repository scratch directory covers it, so a command killed at any instruction leaves
    //  nothing that only this process could have removed (`Scratch`).
    let scratch: Scratch = match Scratch::make(capture.layout.common_dir.as_path(), capture.layout.git_dir.as_path(), inputs.scratch_name) {
        Ok(made) => made,
        Err(error) => return Err(failed(format!("preparing the commit failed: {error}"))),
    };
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: scratch.0.join("captured.index"),
        commit_index: scratch.0.join("commit.index"),
        object_directory: capture.layout.object_directory.clone(),
    };
    let mut pathspec_source: Option<String> = None;
    if let Some(value) = region.pathspec_from_file {
        pathspec_source = Some(String::from_utf8_lossy(crate::command_options_query::value_bytes(region_arguments, value)).into_owned());
    }
    let pathspec_file: Option<PathBuf> = match pathspec_file_for(scratch.0.as_path(), cwd, pathspec_source.as_deref(), inputs.standard_input) {
        Ok(made) => made,
        Err(error) => return Err(failed(format!("reading the pathspec file failed: {error}"))),
    };
    let index_selection: IndexSelection<'_> = IndexSelection {
        mode: selection.mode,
        pathspecs: pathspecs.as_slice(),
        pathspec_file: pathspec_file.as_deref(),
        pathspec_file_nul: region.pathspec_file_nul,
        stage_into_index: region.include,
        stage_tracked_changes: selection.staged_all,
        base_revision: base.as_str(),
    };
    part.next("index");
    // The parts the word-level second pass merged when a retry combined the change with the branch, told with the commit.
    let mut word_merge_events: Option<PolicyEvent> = None;
    // The file the commit copies its index from, as it was before the copy: a live index that is another file later has entries
    //  the copy does not, and those are what the commit must not record.
    let index_witness: crate::service_land_start::IndexWitness = crate::service_land_start::witness_of(capture.layout.real_index_path.as_path()).unwrap_or(crate::service_land_start::IndexWitness::Absent);
    if let Some(again) = retry {
        // The change this command captured when it started, on the branch as it is now; the worktree is not read.
        let site: MergeSite<'_> = MergeSite {
            context,
            cwd,
            common_dir: capture.layout.common_dir.as_path(),
            object_directory: indexes.object_directory.as_path(),
            scratch: scratch.0.as_path(),
        };
        let (tree, combined): (String, Vec<MergedFile>) = match merge_captured(&site, again.captured, base.as_str(), again.attempt)? {
            Merged::Conflict(paths) => {
                let landed = first_landed(context, cwd, again.captured.base.as_str(), base.as_str(), paths.as_slice())?;
                return Ok(Prepared::Conflict(Box::new(ReplayConflict { paths, landed, git_dir: capture.layout.git_dir.clone() })));
            }
            Merged::Tree(tree) => (tree, Vec::new()),
            Merged::Combined { tree, files } => (tree, files),
        };
        // Built from a copy of the real index, so the entries the tree shares with it keep their stat data and the scratch commit
        //  does not hash the whole work tree again (`commit_index.rs`, `initialize_retry_index`).
        initialize_retry_index(context, cwd, capture.layout.real_index_path.as_path(), &indexes, tree.as_str())?;
        // Every part the word pass merged is told to the person with the commit and sent to the journal.
        word_merge_events = word_merge_notice(combined.as_slice());
    } else {
        if let Err(error) = initialize_commit_index(context, cwd, capture.layout.real_index_path.as_path(), &indexes, &index_selection) {
            return Err(failed(error.0));
        }
    }
    // A commit that landed but did not finish its index update leaves this worktree's index describing the branch before that
    //  landing, so a plain commit of this index would record the landed commit's own paths as their old content, and a path it added
    //  as a staged deletion. Which entries are the landing's own cannot be decided from what they hold, so nothing here writes the
    //  live index: this commit's own index is set to the head for the paths whose entry disagrees with it, and the person is told
    //  (`owed_index_repair.rs`, `doc/planning/cli-git-commit-and-landing-route-findings-on-main.md`). An explicit-path commit names
    //  its own selection, so only the notice is given for it.
    let (excluded, unreadable_note): (Vec<String>, Option<String>) = crate::owed_index_update::exclude_or_stop(context, &capture, &indexes, selection.mode, &index_witness)?;
    if inputs.first && let Some(note) = unreadable_note.as_ref() {
        // A notice is worth less than a commit that returns: what cannot be said within the ceiling is left unsaid.
        let _delivered = deliver_notice(format!("{note}\n"));
    }
    if inputs.first
        && let Some(notice) = crate::owed_index_update::excluded_notice(excluded.as_slice())
    {
        let _delivered = deliver_notice(notice);
    }
    part.next("interactive");
    // The bytes are captured now; a retry keeps the stamp of the capture it replays.
    let captured_at: u64 = match retry {
        Some(again) => again.captured.captured,
        None => capture_stamp(),
    };
    // A test removes the staged content here, between the writing and the reading by the policies.
    reach_service_phase(Phase::ServiceStaged)?;
    if (region.interactive || region.patch) && retry.is_none() {
        prepare_interactive_selection(context, cwd, indexes.commit_index.as_path(), indexes.object_directory.as_path(), region.patch, &index_selection)?;
    }
    part.next("conclusionState");
    let interactive: bool = region.interactive || region.patch;
    let reftable: bool = capture.layout.ref_format == RefFormat::Reftable;
    // Whatever state a merge or a pick left (also a squash message with no merge in progress) is removed as native Git removes it.
    let mut state_present: bool = selection.concludes_sequencer;
    for entry in CONCLUSION_STATE_FILES.iter().chain([SEQUENCER_DIRECTORY].iter()) {
        if state_present {
            break;
        }
        if capture.layout.git_dir.join(entry).exists() {
            state_present = true;
        }
    }
    let cleanup: Option<Cleanup> = if state_present { Some(Cleanup { git_dir: capture.layout.git_dir.clone(), scratch: scratch.0.clone(), reftable }) } else { None };
    let selected_privately: bool = selection.read_only;
    let fixed_parents: bool = selection.concludes_sequencer || unborn;
    let mut pathspec_texts: Vec<String> = Vec::new();
    for token in &pathspecs {
        pathspec_texts.push(token.to_string_lossy().into_owned());
    }
    // What every `Ready` of this preparation shares; the closures that carried it are the methods of this record.
    let shape: ReadyShape = ReadyShape {
        pathspecs: pathspec_texts,
        mode: selection.mode,
        staged_all: selection.staged_all,
        selected_privately,
        commit_index: indexes.commit_index.clone(),
        reference: reference.clone(),
        announced_base: announced_base.clone(),
        cleanup: cleanup.clone(),
        interactive,
        fixed_parents,
        captured_at,
    };
    // Files still unmerged when the commit names no paths: Git refuses here (`builtin/commit.c:348`, `refresh_cache_or_die`, which
    //  exits with "Committing is not possible because you have unmerged files"). The owner's rule is "Warn and finish the merge"
    //  (2026-11-03), so the working copy of each unmerged file is staged in the commit's private index, exactly as `git add` stages
    //  it, and the commit that follows is the one Git would have built for a resolved merge. The files are named in the warning.
    part.next("unmergedFiles");
    let unmerged_files: Vec<Vec<u8>> = list_unmerged_index_paths(context, cwd, indexes.commit_index.as_path())?;
    if !unmerged_files.is_empty() {
        run_git_checked(context, &unmerged_staging_request(cwd, indexes.commit_index.as_path(), indexes.object_directory.as_path(), unmerged_files.as_slice()))?;
    }
    part.next("changedPaths");
    let candidate_paths: Vec<Vec<u8>> =
        list_changed_index_paths(context, cwd, indexes.commit_index.as_path(), base.as_str(), indexes.object_directory.as_path())?;
    // Nothing to commit is Git's to say, and an amend or an empty commit that was asked for is not nothing.
    if candidate_paths.is_empty() && !inputs.allow_empty && !amends {
        if retry.is_some() {
            // Commits that landed meanwhile hold everything this one changes.
            return Ok(Prepared::Ended(already_committed_result(reference.trim_start_matches("refs/heads/"))));
        }
        return Ok(Prepared::Ready(shape.by_git(capture, Vec::new(), None, Vec::new(), invocation.arguments.to_vec(), scratch)));
    }
    part.next("policies");
    let mut private: ShippedChecks<PrivateIndexFacts<&mut F>> = private_checks(
        checks,
        indexes.commit_index.as_path(),
        indexes.object_directory.as_path(),
        staged_at_base(base_object(&capture)?, candidate_paths.as_slice()),
    );
    // A selection the person made leaves the policies read-only: they report, and the commit is exactly what was chosen.
    let (pass, changed_passes, changed_paths): (PassResult, u64, Vec<Vec<u8>>) = if selection.read_only {
        private.can_apply_patches = false;
        (run_policy_pass(request, &mut private), 0, Vec::new())
    } else {
        let outcome: CommitConvergence = converge_commit_policies(CommitConvergenceInput {
            request,
            checks: &mut private,
            stage: PrivateStage { context, cwd, index: indexes.commit_index.as_path(), object_directory: Some(indexes.object_directory.as_path()) },
        });
        let passes: u64 = settled_passes(&outcome.convergence);
        (outcome.pass, passes, outcome.changed_paths)
    };
    let code: i32 = pass_exit_code(pass.events.as_slice(), pass.end);
    if code != 0 {
        return Ok(Prepared::Ended(TransactionResult { events: pass.events, exit_code: code, landed_oid: None, uses_lfs: false }));
    }
    part.next("worktreeRecords");
    let repository_root: &Path = capture.layout.repository_root.as_path();
    let install = private.content.install_changes();
    let records: Vec<AddedPath> = selected_worktree_records(context, cwd, repository_root, indexes.commit_index.as_path(), install.as_slice())?;
    // Corrections that removed every change leave nothing to commit: the copies are reconciled and the person is told.
    let eligible: bool = !install.is_empty() && !region.allow_empty && !amends && !selection.concludes_sequencer;
    part.next("treeCheck");
    if eligible && !unborn {
        let intended: String = write_private_tree(context, cwd, indexes.commit_index.as_path(), indexes.object_directory.as_path())?;
        let mut tree_request = indexed_request(cwd, &["rev-parse", "--verify", format!("{base}^{{tree}}").as_str()], None, None);
        tree_request.without_prefix = true;
        let base_tree: String = String::from_utf8_lossy(run_git_checked(context, &tree_request)?.stdout.as_slice()).trim().to_string();
        if base_tree == intended {
            return normalized(invocation, &capture, base.as_str(), &candidate_paths, records.as_slice(), &(pass.events, changed_passes, changed_paths));
        }
    }
    part.next("changes");
    let mut events: Vec<PolicyEvent> = events_with_changed_summary(pass.events.clone(), changed_passes, changed_paths.as_slice());
    events.extend(word_merge_events);
    let changes: Option<Vec<Change>> = if config_native || unborn || detached || state_present { None } else { changes_of(context, cwd, &indexes, base.as_str())? };
    let forced: bool = config_native || unborn || detached || state_present || changes.is_none();
    // A commit of named paths leaves the merge or cherry-pick whose state it does not conclude in progress.
    let conclusion_kind: Conclusion = capture.layout.conclusion;
    let leaves_open: Option<Conclusion> = if named && matches!(conclusion_kind, Conclusion::Merge | Conclusion::CherryPick) { Some(conclusion_kind) } else { None };
    let mut ready: Box<Ready> = shape.by_git(capture, events, changes, records, pass.arguments.clone(), scratch);
    ready.needs_native = forced;
    // A path in conflict that the commit takes is committed with the bytes the worktree holds, markers included.
    for path in unmerged_before {
        if candidate_paths.contains(&path) {
            ready.committed_conflicted.push(path);
        }
    }
    ready.committed_conflicted.extend(unmerged_files);
    ready.leaves_open = leaves_open;
    return Ok(Prepared::Ready(ready));
}

/// Controls for the staging start of unmerged files.
#[cfg(test)]
#[path = "service_prepare_tests.rs"]
mod tests;
