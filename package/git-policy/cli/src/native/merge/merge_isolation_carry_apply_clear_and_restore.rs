//! What:
//!  The two writes of a carry that put a path back without merging anything:
//!  the clear of the person's paths before Git's own update takes them,
//!  and the restore of their layers when the merge did not land.
//! Why:
//!   Git's own two-way update refuses a path that holds anything but the branch's content,
//! so the checkout is
//!  cleared in one step whose every path is judged before anything is written;
//!  and a merge killed after the clear leaves the branch's content where the person's work was,
//!  so the capture is what puts it back,
//!  with no merge applied at all.
//! A path that holds bytes neither the capture
//!  nor the change explains belongs to someone else and is never written:
//! it is named and left alone.
//!  The readings these two steps judge are
//!  `merge_isolation_carry_apply_at_the_worktree_top.rs`,
//!  the writes they plan are `merge_isolation_carry_writes.rs`,
//!  and the merge of the person's layers is `merge_isolation_carry_apply.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await clear({ context, root, changes, capture }); await restore({ context, root, gitDir, changes, capture });
//! ```

/// The entry of the process runtime that a synchronous caller takes.
use crate::lazy_runtime::run_on_runtime;
/// The line that says a path written after the capture was left as it is.
use crate::merge_isolation_carry_notice::note_left_alone;
/// The person's work as the capture recorded it.
use crate::merge_isolation_carry_parts::Capture;
/// What stood at a path when it was judged.
use crate::merge_isolation_carry_place::judged_of;
/// The write plan of one path.
use crate::merge_isolation_carry_write_forms::{
    FileWrite,
    IndexWrite,
};
/// The one step that lands a batch of writes,
/// and what it came to.
use crate::merge_isolation_carry_writes::{
    Batch,
    LockWait,
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
    file_identity,
    worktree_path,
    worktree_state_awaiting,
};
/// The entries a killed step left under a private name,
/// which go back before a path is judged.
use crate::merge_isolation_take_asides::put_back_stranded_takes_awaiting;
/// The Git context a read and a write take.
use crate::transaction_git::GitContext;
/// A path of the worktree.
use std::path::Path;

/// What the capture and the merge's change say about one path,
/// and the context the step's reads run from.
use crate::merge_isolation_carry_apply_at_the_worktree_top::{
    blob_of,
    change_for,
    entry_explained,
    explained,
    index_pair,
    new_entry_of,
    new_oid_of,
    old_entry_of,
    old_oid_of,
    rooted,
};
/// A Git read of one path,
/// which this file judges and never spells for Git itself.
use crate::merge_isolation_carry_reads::{
    file_form_awaiting,
    index_entry_awaiting,
    raw_bytes_awaiting,
};
/// Why a carry stopped,
/// and the words a write that stopped leaves about the copy it kept.
use crate::merge_isolation_carry_write_failures::CarryFailure;

/// What:
///  Put the branch's own content back at every captured path,
/// so Git's two-way update takes them.
/// Why:
///   Git refuses a merge whose paths hold anything but the branch's content,
/// and the person's bytes are already
///  durable elsewhere.
/// Every path is judged before anything is written,
/// so a path that changed after the capture
///  stops the clear with nothing written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function clear({ context, root, gitDir, changes, capture, out }): Promise<void>;
/// ```
pub fn clear(
    context: &GitContext,
    root: &Path,
    git_dir: &Path,
    changes: &[Change],
    capture: &Capture,
    transaction_id: &str,
    wait: LockWait,
    out: &mut dyn std::io::Write,
) -> Result<(), CarryFailure> {
    return match run_on_runtime(clear_awaiting(
        context,
        root,
        git_dir,
        changes,
        capture,
        transaction_id,
        wait,
        out,
    )) {
        Ok(found) => found,
        Err(refusal) => Err(CarryFailure::Failed(format!("{refusal}"))),
    };
}

/// What:
///  The same clear of the captured paths,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   Each captured path reads its own worktree state and index entry through Git,
/// and every start of a step is refused from inside the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function clear({ context, root, gitDir, changes, capture, out }): Promise<void>;
/// ```
pub async fn clear_awaiting<W: std::io::Write + ?Sized>(
    context: &GitContext,
    root: &Path,
    git_dir: &Path,
    changes: &[Change],
    capture: &Capture,
    transaction_id: &str,
    wait: LockWait,
    out: &mut W,
) -> Result<(), CarryFailure> {
    put_back_stranded_takes_awaiting(
        context,
        root,
        git_dir,
        changes,
        Some(capture),
        transaction_id,
        out,
    )
    .await;
    let mut steps: Vec<Step> = Vec::with_capacity(capture.paths.len());
    for entry in &capture.paths {
        let Some(change) = change_for(changes, entry) else {
            continue;
        };
        let location: std::path::PathBuf = worktree_path(root, entry.path.as_slice());
        // The identity is read before the content, so a file written in between does not match the judgement and is never
        //  replaced, and the write step reads it again before it writes anything.
        let identity: Option<FileIdentity> = file_identity(location.as_path());
        let state: WorktreeState =
            worktree_state_awaiting(&rooted(context), root, entry.path.as_slice())
                .await
                .map_err(CarryFailure::Failed)?;
        let captured_object: Option<&str> = entry.worktree.as_ref().map(blob_of);
        let old_object: Option<&str> = old_oid_of(change);
        if !explained(&state, &[captured_object, old_object]) {
            return Err(CarryFailure::Foreign(entry.path.clone()));
        }
        let current: Option<(String, String)> =
            index_entry_awaiting(context, root, entry.path.as_slice())
                .await
                .map_err(CarryFailure::Failed)?;
        let captured_entry: Option<(&str, &str)> = entry.index.as_ref().map(index_pair);
        let old_entry: Option<(&str, &str)> = old_entry_of(change);
        if !entry_explained(&current, &[captured_entry, old_entry]) {
            return Err(CarryFailure::Foreign(entry.path.clone()));
        }
        let index: IndexWrite = match old_entry {
            Some((mode, oid)) => IndexWrite::Entry(String::from(mode), String::from(oid)),
            None => IndexWrite::Remove,
        };
        // The clear puts the branch's own bytes back where Git's own update is about to write the merge's entry, so the file
        //  is created exactly as Git creates it and the system's umask and the directory's default ACL decide its mode.
        let worktree: FileWrite = match old_object {
            Some(oid) => FileWrite::Bytes {
                bytes: file_form_awaiting(context, root, entry.path.as_slice(), oid)
                    .await
                    .map_err(CarryFailure::Failed)?,
                mode: change.old_mode.clone(),
                the_persons_bits: None,
            },
            None => FileWrite::Remove,
        };
        steps.push(Step {
            path: entry.path.clone(),
            index,
            worktree,
            judged: judged_of(&state, identity),
            keep_the_planned_bytes: false,
        });
    }
    let written: StepResult = write_steps_awaiting(
        context,
        root,
        git_dir,
        steps.as_slice(),
        transaction_id,
        wait,
        Batch::Clear,
        out,
    )
    .await?;
    // A path the write step found newer than its judgement stops the clear: the merge must not run over a checkout this step
    //  only half cleared, and the round is made again on the bytes the person has now.
    return match written.changed.first() {
        Some(path) => Err(CarryFailure::Changed(path.clone())),
        None => Ok(()),
    };
}

/// What:
///  Put the person's layers back exactly as they were,
/// for a merge that did not land.
/// Why:
///   A merge killed after the clear leaves the branch's content where the person's work was;
/// the capture is what puts
///  it back,
/// and nothing of the merge is applied,
/// so each layer is written as captured with no merge at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function restore({ context, root, gitDir, capture, out }): Promise<void>;
/// ```
pub fn restore(
    context: &GitContext,
    root: &Path,
    git_dir: &Path,
    changes: &[Change],
    capture: &Capture,
    transaction_id: &str,
    wait: LockWait,
    out: &mut dyn std::io::Write,
) -> Result<(), CarryFailure> {
    return match run_on_runtime(restore_awaiting(
        context,
        root,
        git_dir,
        changes,
        capture,
        transaction_id,
        wait,
        out,
    )) {
        Ok(found) => found,
        Err(refusal) => Err(CarryFailure::Failed(format!("{refusal}"))),
    };
}

/// What:
///  The same putting back of the person's layers,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   Each captured path reads its state,
/// its index entry and the bytes to write through Git,
/// and every start of a step is refused from inside the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function restore({ context, root, gitDir, changes, capture, out }): Promise<void>;
/// ```
pub async fn restore_awaiting<W: std::io::Write + ?Sized>(
    context: &GitContext,
    root: &Path,
    git_dir: &Path,
    changes: &[Change],
    capture: &Capture,
    transaction_id: &str,
    wait: LockWait,
    out: &mut W,
) -> Result<(), CarryFailure> {
    put_back_stranded_takes_awaiting(
        context,
        root,
        git_dir,
        changes,
        Some(capture),
        transaction_id,
        out,
    )
    .await;
    let mut steps: Vec<Step> = Vec::with_capacity(capture.paths.len());
    for entry in &capture.paths {
        let location: std::path::PathBuf = worktree_path(root, entry.path.as_slice());
        // The identity is read before the content, so a file written in between does not match the judgement and is never
        //  replaced, and the write step reads it again before it writes anything.
        let identity: Option<FileIdentity> = file_identity(location.as_path());
        let state: WorktreeState =
            worktree_state_awaiting(&rooted(context), root, entry.path.as_slice())
                .await
                .map_err(CarryFailure::Failed)?;
        let captured_object: Option<&str> = entry.worktree.as_ref().map(blob_of);
        // The branch's own content is what a clear leaves, so it is what a restore may replace.
        let change: Option<&Change> = change_for(changes, entry);
        let old_object: Option<&str> = change.and_then(old_oid_of);
        let merged_object: Option<&str> = change.and_then(new_oid_of);
        // A path written after the capture holds the person's own newer bytes: it is left as it is and named, because
        //  replacing it is what loses a save, and the merge it belongs to did not land.
        let worktree_is_ours: bool =
            explained(&state, &[captured_object, old_object, merged_object]);
        let current: Option<(String, String)> =
            index_entry_awaiting(context, root, entry.path.as_slice())
                .await
                .map_err(CarryFailure::Failed)?;
        let captured_entry: Option<(&str, &str)> = entry.index.as_ref().map(index_pair);
        let old_entry: Option<(&str, &str)> = change.and_then(old_entry_of);
        let merged_entry: Option<(&str, &str)> = change.and_then(new_entry_of);
        let index_is_ours: bool =
            entry_explained(&current, &[captured_entry, old_entry, merged_entry]);
        if !worktree_is_ours {
            note_left_alone(out, entry.path.as_slice());
        }
        if !worktree_is_ours && !index_is_ours {
            continue;
        }
        let index: IndexWrite = match entry.index.as_ref() {
            Some(staged) => IndexWrite::Entry(staged.mode.clone(), staged.oid.clone()),
            None => IndexWrite::Remove,
        };
        // The merge did not land, so nothing of Git's stands where the person's own entry goes back, and the entry keeps
        //  every permission bit it had when the capture read it.
        let worktree: FileWrite = match entry.worktree.as_ref() {
            Some(file) => FileWrite::Bytes {
                bytes: raw_bytes_awaiting(context, root, file.raw.as_str())
                    .await
                    .map_err(CarryFailure::Failed)?,
                mode: file.mode.clone(),
                the_persons_bits: file.permissions,
            },
            None => FileWrite::Remove,
        };
        let put_back: Step = Step {
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
            keep_the_planned_bytes: false,
        };
        steps.push(put_back);
    }
    let _written: StepResult = write_steps_awaiting(
        context,
        root,
        git_dir,
        steps.as_slice(),
        transaction_id,
        wait,
        Batch::Restore,
        out,
    )
    .await?;
    return Ok(());
}
