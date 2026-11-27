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

use crate::commit_conclusion::events_with_changed_summary;
use crate::commit_index::{
    IndexSelection,
    list_changed_index_paths,
    list_unmerged_index_paths,
    prepare_interactive_selection,
    write_private_tree,
};
use crate::commit_prepare_part_durations::PreparePartClock;
use crate::commit_transaction::{
    CommitInvocation,
    TransactionResult,
    TransactionStop,
};
use crate::commit_worktree_records::selected_worktree_records;
use crate::policy_checks::ShippedChecks;
use crate::policy_engine::{
    StageRequest,
    pass_exit_code,
};
use crate::policy_events::PolicyEvent;
use crate::repository_facts::RepositoryFacts;
use crate::service_prepare_changes_config_and_pathspec_steps::{
    changes_of,
    normalized,
};
pub use crate::service_prepare_changes_config_and_pathspec_steps::{
    config_needs_native,
    config_signs,
};
use crate::service_prepare_own_git_steps::{
    base_tree_of,
    stage_the_unmerged_files,
};
use crate::service_prepare_ready_shapes::ReadyShape;
pub use crate::service_prepare_ready_shapes::{
    HandedBackTree,
    NativeInputs,
    Prepared,
    Ready,
    Scratch,
};
use crate::service_prepare_selection_refusals::staged_deletion_ending;
use crate::service_prepare_the_commit_index::{
    IndexBuilt,
    the_commit_index,
};
use crate::service_prepare_the_policies::the_policies;
use crate::service_prepare_the_ready_shape::the_ready_shape;
use crate::service_prepare_what_it_reads::{
    Preparation,
    read_the_setup,
};
use crate::service_protocol::Change;
use crate::service_pruned::reach_service_phase;
use crate::service_replay::{
    Retry,
    already_committed_result,
    capture_stamp,
};
use crate::transaction_git::GitContext;
use crate::transaction_journal::{
    AddedPath,
    Conclusion,
};
use crate::transaction_phase::Phase;
use std::path::Path;

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
    let setup: Preparation = read_the_setup(
        invocation,
        inputs.first,
        inputs.scratch_name,
        inputs.standard_input,
        &mut part,
    )?;
    let retry: Option<&Retry<'_>> = inputs.replay.as_ref();
    let index_selection: IndexSelection<'_> = IndexSelection {
        mode: setup.selection.mode,
        pathspecs: setup.pathspecs.as_slice(),
        pathspec_file: setup.pathspec_file.as_deref(),
        pathspec_file_nul: region.pathspec_file_nul,
        stage_into_index: region.include,
        stage_tracked_changes: setup.selection.staged_all,
        base_revision: setup.base.as_str(),
    };
    part.next("stagedDeletions");
    if setup.named && retry.is_none() {
        // Before any private staging, so Git's "nothing to commit" or its force-add hint never stands in for the reason.
        //  The selection is asked as one thing, so a path a `--pathspec-from-file` names reaches this guard exactly as a
        //  path on the command line does (finding `commit-only/staged-deletion-worktree-present`).
        let present: Option<TransactionResult> = staged_deletion_ending(
            context,
            cwd,
            setup.capture.layout.repository_root.as_path(),
            setup.capture.layout.real_index_path.as_path(),
            setup.base.as_str(),
            &index_selection,
        )?;
        if let Some(ending) = present {
            return Ok(Prepared::Ended(ending));
        }
    }
    part.next("index");
    let word_merge_events: Option<PolicyEvent> =
        match the_commit_index(invocation, &setup, &index_selection, retry, inputs.first)? {
            IndexBuilt::Ended(prepared) => return Ok(prepared),
            IndexBuilt::Ready {
                word_merge_events,
            } => word_merge_events,
        };
    part.next("interactive");
    // The bytes are captured now; a retry keeps the stamp of the capture it replays.
    let captured_at: u64 = match retry {
        Some(again) => again.captured.captured,
        None => capture_stamp(),
    };
    // A test removes the staged content here, between the writing and the reading by the policies.
    reach_service_phase(Phase::ServiceStaged)?;
    if (region.interactive || region.patch) && retry.is_none() {
        prepare_interactive_selection(
            context,
            cwd,
            setup.indexes.commit_index.as_path(),
            setup.indexes.object_directory.as_path(),
            region.patch,
            &index_selection,
        )?;
    }
    part.next("conclusionState");
    // What: `the_ready_shape` reads the conclusion state and builds the record every ending of this preparation shares.
    // Why: a retry is compared against the state read here, and each ending needs the same pathspecs, reference and scratch.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [shape, statePresent] = theReadyShape(invocation, setup, capturedAt);
    // ```
    let (shape, state_present): (ReadyShape, bool) =
        the_ready_shape(invocation, &setup, captured_at)?;
    // Files still unmerged when the commit names no paths: Git refuses here (`builtin/commit.c:348`, `refresh_cache_or_die`, which
    //  exits with "Committing is not possible because you have unmerged files"). The owner's rule is "Warn and finish the merge"
    //  (2026-11-03), so the working copy of each unmerged file is staged in the commit's private index, exactly as `git add` stages
    //  it, and the commit that follows is the one Git would have built for a resolved merge. The files are named in the warning.
    part.next("unmergedFiles");
    let unmerged_files: Vec<Vec<u8>> =
        list_unmerged_index_paths(context, cwd, setup.indexes.commit_index.as_path())?;
    stage_the_unmerged_files(context, cwd, &setup.indexes, unmerged_files.as_slice())?;
    part.next("changedPaths");
    let candidate_paths: Vec<Vec<u8>> = list_changed_index_paths(
        context,
        cwd,
        setup.indexes.commit_index.as_path(),
        setup.base.as_str(),
        setup.indexes.object_directory.as_path(),
    )?;
    // Nothing to commit is Git's to say, and an amend or an empty commit that was asked for is not nothing.
    if candidate_paths.is_empty() && !inputs.allow_empty && !setup.amends {
        if retry.is_some() {
            // Commits that landed meanwhile hold everything this one changes.
            return Ok(Prepared::Ended(already_committed_result(
                setup.reference.short().spoken().as_str(),
            )));
        }
        return Ok(Prepared::Ready(shape.by_git(
            setup.capture,
            Vec::new(),
            None,
            Vec::new(),
            invocation.arguments.to_vec(),
            setup.scratch,
        )));
    }
    part.next("policies");
    // What: `the_policies` runs the checks over the commit's own index and hands back the checks the records are read through.
    // Why: a selection the person made leaves them read-only, and a policy that refused the commit decides the exit status here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [private, pass, changedPasses, changedPaths] = thePolicies(invocation, checks, request, setup, candidatePaths);
    // ```
    let (private, pass, changed_passes, changed_paths) = the_policies(
        invocation,
        checks,
        request,
        &setup,
        candidate_paths.as_slice(),
    )?;
    let code: i32 = pass_exit_code(pass.events.as_slice(), pass.end);
    if code != 0 {
        return Ok(Prepared::Ended(TransactionResult {
            events: pass.events,
            exit_code: code,
            landed_oid: None,
            uses_lfs: false,
        }));
    }
    part.next("worktreeRecords");
    let repository_root: &Path = setup.capture.layout.repository_root.as_path();
    let install = private.content.install_changes();
    let records: Vec<AddedPath> = selected_worktree_records(
        context,
        cwd,
        repository_root,
        setup.indexes.commit_index.as_path(),
        install.as_slice(),
    )?;
    // Corrections that removed every change leave nothing to commit: the copies are reconciled and the person is told.
    let eligible: bool = !install.is_empty()
        && !region.allow_empty
        && !setup.amends
        && !setup.selection.concludes_sequencer;
    part.next("treeCheck");
    if eligible && !setup.unborn {
        let intended: String = write_private_tree(
            context,
            cwd,
            setup.indexes.commit_index.as_path(),
            setup.indexes.object_directory.as_path(),
        )?;
        let base_tree: String = base_tree_of(context, cwd, setup.base.as_str())?;
        if base_tree == intended {
            return normalized(
                invocation,
                &setup.capture,
                setup.base.as_str(),
                &candidate_paths,
                records.as_slice(),
                &(pass.events, changed_passes, changed_paths),
            );
        }
    }
    part.next("changes");
    let mut events: Vec<PolicyEvent> = events_with_changed_summary(
        pass.events.clone(),
        changed_passes,
        changed_paths.as_slice(),
    );
    events.extend(word_merge_events);
    let changes: Option<Vec<Change>> =
        if setup.config_native || setup.unborn || setup.detached || state_present {
            None
        } else {
            changes_of(context, cwd, &setup.indexes, setup.base.as_str())?
        };
    let forced: bool =
        setup.config_native || setup.unborn || setup.detached || state_present || changes.is_none();
    // A commit of named paths leaves the merge or cherry-pick whose state it does not conclude in progress.
    let conclusion_kind: Conclusion = setup.capture.layout.conclusion;
    let leaves_open: Option<Conclusion> =
        if setup.named && matches!(conclusion_kind, Conclusion::Merge | Conclusion::CherryPick) {
            Some(conclusion_kind)
        } else {
            None
        };
    let mut ready: Box<Ready> = shape.by_git(
        setup.capture,
        events,
        changes,
        records,
        pass.arguments.clone(),
        setup.scratch,
    );
    ready.needs_native = forced;
    // A path in conflict that the commit takes is committed with the bytes the worktree holds, markers included.
    for path in setup.unmerged_before {
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
