//! What:
//!  Writing it back:
//!  the merge of each of the person's two layers with the merge,
//!  path by path,
//!  once Git's own update has put the merge in the checkout.
//!  The clear before that update and the restore of a merge that did not land are
//!  `merge_isolation_carry_apply_clear_and_restore.rs`,
//!  and the readings this step judges are
//!  `merge_isolation_carry_apply_at_the_worktree_top.rs`.
//! Why:
//!   After the branch moved the person's layers are merged with the merge's
//!  (`merge_isolation_carry_layers.rs`),
//! so an edit away from the
//!  merge's lines comes back where it was and an edit on them keeps both sides.
//! A path that holds bytes neither the capture
//!  nor the change explains belongs to someone else and is never written:
//! it is named and left alone.
//!  The reads it judges are `merge_isolation_carry_reads.rs`
//! and the writes it plans are `merge_isolation_carry_writes.rs`,
//! so this file holds the judgement of each layer and the line a person reads.
//!  Each entry point keeps a synchronous face for a command's own step
//!  and an awaited twin for a caller that is already a task of the runtime.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await carry({ context, root, changes, capture });
//! ```

/// The entry of the process runtime that a synchronous caller takes.
use crate::lazy_runtime::run_on_runtime;
/// What the index holds once a step's records are written,
/// and the entries a command that writes the index would replace.
use crate::merge_isolation_carry_kept_path::{
    Staged,
    index_after,
    staged_of,
};
/// What one carried path adds to the report.
use crate::merge_isolation_carry_notice::Report;
/// Putting one written path into the report the lines are made from.
use crate::merge_isolation_carry_notice::add_to_the_report;
/// The person's work as the capture recorded it.
use crate::merge_isolation_carry_parts::Capture;
/// What stood at a path when it was judged,
/// and the entries an index check accepts.
use crate::merge_isolation_carry_place::{
    explained_entries,
    judged_of,
};
/// The decisions a path's layers come to,
/// and the planning of one path's writes.
use crate::merge_isolation_carry_plans::{
    plan_index_awaiting,
    plan_worktree_awaiting,
};
/// The Git read of one path,
/// which this file judges and never spells for Git itself.
use crate::merge_isolation_carry_reads::index_entry_awaiting;
/// The write plan of one path,
/// and the one step that lands it.
use crate::merge_isolation_carry_write_forms::{
    FileWrite,
    IndexWrite,
};
use crate::merge_isolation_carry_writes::{
    Batch,
    Step,
    StepResult,
    write_steps_awaiting,
};
/// The paths the merge changes,
/// the state a path holds now,
/// and the identity the write step checks before it replaces anything.
use crate::merge_isolation_changes::{
    Change,
    FileIdentity,
    WorktreeState,
    worktree_path,
    worktree_state_awaiting,
};
/// The runner the word-level pass is given,
/// and the Git context every read and write names.
use crate::merge_second_pass::ContextRunner;
/// The Git context a read and a write take.
use crate::transaction_git::GitContext;
/// A path of the worktree.
use std::path::Path;

/// What Git's own update left at each path,
/// which tells the merge's own file from a save that accepts it.
pub use crate::merge_isolation_carry_landed::{
    LandedFile,
    landed_file_at,
    the_file_git_landed,
};
/// The paths the merge changes,
/// read again by whoever holds a `merge_isolation_carry_reads` name.
pub use crate::merge_isolation_carry_reads::{
    changes_between,
    changes_between_awaiting,
};
/// Why a carry stopped.
pub use crate::merge_isolation_carry_write_failures::CarryFailure;
pub use crate::merge_isolation_carry_writes::LockWait;

/// What the capture and the merge's change say about one path,
/// and the context this step's reads and the layer merge start from.
use crate::merge_isolation_carry_apply_at_the_worktree_top::{
    blob_of,
    change_for,
    entry_explained,
    explained,
    index_pair,
    new_entry_of,
    new_oid_of,
    old_entry_of,
    rooted,
    runner_of,
};
/// The two writes that put a path back without a merge,
/// named here so a caller of the carry keeps one path.
pub use crate::merge_isolation_carry_apply_clear_and_restore::{
    clear,
    clear_awaiting,
    restore,
    restore_awaiting,
};

/// What:
///  Merge each captured layer with the merge and write it back,
/// and name every path the person must look at.
/// Why:
///   An edit away from the merge's lines comes back as it was,
/// a staged edit stays staged,
/// and an edit on the
///  merge's lines keeps both sides between markers,
/// so the command exits 0 with the merge made and the person's work
///  whole (`SPEC.md`,
/// "Merges and commits never fail").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function carry({ context, root, gitDir, changes, capture, out }): Promise<Report>;
/// ```
pub fn carry(
    context: &GitContext,
    root: &Path,
    git_dir: &Path,
    changes: &[Change],
    capture: &Capture,
    landed: &[LandedFile],
    transaction_id: &str,
    wait: LockWait,
    out: &mut dyn std::io::Write,
) -> Result<Report, CarryFailure> {
    return match run_on_runtime(carry_awaiting(
        context,
        root,
        git_dir,
        changes,
        capture,
        landed,
        transaction_id,
        wait,
        out,
    )) {
        Ok(found) => found,
        Err(refusal) => Err(CarryFailure::Failed(format!("{refusal}"))),
    };
}

/// What:
///  The same merge of every captured layer,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The copies a command that stopped left beside the merge's paths are decided by the caller before this step,
///  by the clear that hands Git its paths or by the recovery that reads them,
/// so this face names each copy
///  once and not once per layer.
///   The word-level merge,
/// the object writes and the index write are each a start,
/// and a synchronous start registers the step's pipes by entering the runtime,
///  which is refused from inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function carry({ context, root, gitDir, changes, capture, out }): Promise<Report>;
/// ```
pub async fn carry_awaiting<W: std::io::Write + ?Sized>(
    context: &GitContext,
    root: &Path,
    git_dir: &Path,
    changes: &[Change],
    capture: &Capture,
    landed: &[LandedFile],
    transaction_id: &str,
    wait: LockWait,
    out: &mut W,
) -> Result<Report, CarryFailure> {
    // The merge of one layer runs from the top of the worktree, where a relative global option of the invocation does not reach.
    let rooted_context: GitContext = rooted(context);
    let runner: ContextRunner<'_> = runner_of(&rooted_context, root);
    let mut report: Report = Report::default();
    let mut steps: Vec<Step> = Vec::with_capacity(capture.paths.len());
    for entry in &capture.paths {
        let Some(change) = change_for(changes, entry) else {
            continue;
        };
        let location: std::path::PathBuf = worktree_path(root, entry.path.as_slice());
        // The identity is read before the content, so a file written in between does not match the judgement and is never
        //  replaced, and the write step reads it again before it writes anything.
        let standing: LandedFile = landed_file_at(location.as_path(), entry.path.as_slice());
        let identity: Option<FileIdentity> = standing.identity;
        let state: WorktreeState =
            worktree_state_awaiting(&rooted(context), root, entry.path.as_slice())
                .await
                .map_err(CarryFailure::Failed)?;
        let captured_object: Option<&str> = entry.worktree.as_ref().map(blob_of);
        let merged_object: Option<&str> = new_oid_of(change);
        let (worktree, note, target) =
            plan_worktree_awaiting((&runner, context, root), change, entry).await?;
        let index: IndexWrite = plan_index_awaiting(&runner, change, entry).await?;
        // What stands at the path is the capture, the merge's own, or the bytes this carry writes:
        // a carry that was killed after writing one layer finds its
        //  own bytes there and writes the same ones again. Anything else was written by someone else since the capture, and
        //  a layer somebody else wrote keeps their bytes while the other layer is still carried. Where the identity of what
        //  Git's own update left is known, the file must still be that one: a save whose bytes accept the whole merge is the
        //  person's newer decision, not this carry's own write.
        let worktree_is_ours: bool =
            explained(&state, &[captured_object, merged_object, target.as_deref()])
                && the_file_git_landed(landed, &standing);
        // The reading is the only thing that proves the file standing at the path is the one Git's own update wrote, so a path
        //  it does not name is left as it is and the bytes this carry planned for it are kept beside it and named
        //  (`merge_isolation_carry_landed.rs`).
        let keep_the_planned_bytes: bool = worktree_is_ours
            && crate::merge_isolation_carry_landed::the_write_is_unproven(
                landed,
                entry.path.as_slice(),
                &state,
                merged_object,
            )
            && matches!(&worktree, FileWrite::Bytes { .. });
        let current: Option<(String, String)> =
            index_entry_awaiting(context, root, entry.path.as_slice())
                .await
                .map_err(CarryFailure::Failed)?;
        let captured_entry: Option<(&str, &str)> = entry.index.as_ref().map(index_pair);
        let merged_entry: Option<(&str, &str)> = new_entry_of(change);
        let old_entry: Option<(&str, &str)> = old_entry_of(change);
        // The entry this carry writes is its own, so a kill between the index records and the files finds it and writes the
        //  same layers again rather than reading its own work as someone else's.
        let index_is_ours: bool = entry_explained(
            &current,
            &explained_entries(&[captured_entry, merged_entry, old_entry], &index),
        );
        if !worktree_is_ours && !index_is_ours {
            report.changed.push(entry.path.clone());
            continue;
        }
        // What the index holds once this step's records are written, told apart by the bytes a command that writes the index
        //  would replace: the line about a path the merge deletes names `git rm` only with an entry, and `git add` only where
        //  the index holds no version of the person's.
        let staged: Staged = staged_of(
            index_after(&index, index_is_ours, current.as_ref()),
            target.as_deref(),
            merged_object,
        );
        add_to_the_report(
            &mut report,
            entry,
            change,
            note,
            staged,
            worktree_is_ours && !keep_the_planned_bytes,
        );
        let written: Step = Step {
            path: entry.path.clone(),
            index: if index_is_ours {
                index
            } else {
                IndexWrite::Leave
            },
            worktree: if worktree_is_ours {
                worktree
            } else {
                FileWrite::Leave
            },
            judged: judged_of(&state, identity),
            keep_the_planned_bytes,
        };
        steps.push(written);
    }
    let written: StepResult = write_steps_awaiting(
        context,
        root,
        git_dir,
        steps.as_slice(),
        transaction_id,
        wait,
        Batch::Carry,
        out,
    )
    .await?;
    // A path another program wrote while this carry ran keeps its bytes, and the line says so rather than claiming the file
    //  changed under a step that had in fact written it; the line the path's own step built goes, because the checkout holds
    //  the newer bytes and two lines about one path would disagree.
    report.a_path_written_again(written.changed);
    return Ok(report);
}

/// The writing half of a carry,
/// judged and written one path at a time.
#[cfg(test)]
#[path = "merge_isolation_carry_apply_tests.rs"]
mod tests;
