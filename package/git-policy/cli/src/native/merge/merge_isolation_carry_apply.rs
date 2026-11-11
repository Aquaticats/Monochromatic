//! What:
//!  How one merge carries a person's own work,
//!  path by path:
//!  putting the merge's paths back to the branch before
//!  Git updates them,
//! merging each of the person's two layers with the merge afterwards,
//! and putting their
//!  layers back when the merge does not land.
//! Why:
//!  Git's own two-way update refuses a path that holds anything but the branch's content,
//! so the checkout is
//!  cleared in one step whose every path is judged before anything is written;
//! and after the branch moved the
//!  person's layers are merged with the merge's (`merge_isolation_carry_layers.rs`),
//! so an edit away from the
//!  merge's lines comes back where it was and an edit on them keeps both sides.
//! A path that holds bytes neither the capture
//!  nor the change explains belongs to someone else and is never written:
//! it is named and left alone.
//!  The reads it judges are `merge_isolation_carry_reads.rs`
//! and the writes it plans are `merge_isolation_carry_writes.rs`,
//! so this file holds the judgements alone.
//!  Each entry point keeps a synchronous face for a command's own step
//!  and an awaited twin for a caller that is already a task of the runtime.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await clear({ context, root, changes, capture }); await carry({ context, root, changes, capture });
//! ```

/// The entry of the process runtime that a synchronous caller takes.
use crate::lazy_runtime::run_on_runtime;
/// The decisions a path's layers come to,
/// and the planning of one path's writes.
use crate::merge_isolation_carry_plans::{Note, plan_index_awaiting, plan_worktree_awaiting};
/// What one carried path adds to the report.
use crate::merge_isolation_carry_notice::Report;
/// The kept path's line,
/// and what stands at it.
use crate::merge_isolation_carry_kept_path::kept_path;
/// The person's work as the capture recorded it.
use crate::merge_isolation_carry_parts::{CapturedFile, CapturedIndex, CapturedPath, Capture};
/// The Git reads of one path,
/// which this file judges and never spells for Git itself.
use crate::merge_isolation_carry_reads::{file_form_awaiting, index_entry_awaiting, raw_bytes_awaiting};
/// The write plan of one path,
/// and the one step that lands it.
use crate::merge_isolation_carry_writes::{Batch, FileWrite, IndexWrite, Step, StepResult, write_steps_awaiting};
/// The line that says a path written after the capture was left as it is.
use crate::merge_isolation_carry_notice::note_left_alone;
/// What stood at a path when it was judged,
/// and the entries an index check accepts.
use crate::merge_isolation_carry_place::{explained_entries, judged_of};
/// The paths the merge changes,
/// the state a path holds now,
/// and the identity the write step checks before it replaces anything.
use crate::merge_isolation_changes::{Change, FileIdentity, WorktreeState, file_identity, worktree_path, worktree_state, worktree_state_awaiting};
/// The entries a killed step left under a private name,
/// which go back before a path is judged.
use crate::merge_isolation_take_asides::put_back_stranded_takes;
/// The runner the word-level pass is given,
/// and the Git context every read and write names.
use crate::merge_second_pass::ContextRunner;
/// The Git context a read and a write take.
use crate::transaction_git::GitContext;
/// A path of the worktree.
use std::path::Path;

/// The paths the merge changes,
/// read again by whoever holds a `merge_isolation_carry_reads` name.
pub use crate::merge_isolation_carry_reads::{changes_between, changes_between_awaiting};
/// Why a carry stopped.
pub use crate::merge_isolation_carry_writes::{CarryFailure, LockWait};
/// What Git's own update left at each path,
/// which tells the merge's own file from a save that accepts it.
pub use crate::merge_isolation_carry_landed::{LandedFile, landed_file_at, the_file_git_landed};

/// What:
///  Whether a worktree state is one the capture or the change explains.
/// Why:
///   Anything else was written after this command judged the path,
/// and replacing it would discard that
///  edit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const explained = (state, objects) => state.kind === 'missing' ? objects.includes(null) : objects.includes(state.oid);
/// ```
fn explained(state: &WorktreeState, objects: &[Option<&str>]) -> bool {
    return match state {
        WorktreeState::Missing => objects.contains(&None),
        WorktreeState::Content { object, .. } => objects.contains(&Some(object.as_str())),
        WorktreeState::Other => false,
    };
}

/// What:
///  Whether an index entry is one the capture or the change explains.
/// Why:
///   A staged entry nothing explains was staged after this command judged it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entryExplained = (entry, allowed) => (entry === null && allowed.includes(null)) || allowed.some((a) => a !== null && a.mode === entry.mode && a.oid === entry.oid);
/// ```
fn entry_explained(entry: &Option<(String, String)>, allowed: &[Option<(&str, &str)>]) -> bool {
    let Some((mode, oid)) = entry else { return allowed.contains(&None) };
    for candidate in allowed {
        if *candidate == Some((mode.as_str(), oid.as_str())) {
            return true;
        }
    }
    return false;
}

/// What:
///  The change the merge makes to one captured path.
/// Why:
///   A path the merge does not change is not the carry's to write,
///  and every step of the carry asks this same question.
fn change_for<'a>(changes: &'a [Change], entry: &CapturedPath) -> Option<&'a Change> {
    for change in changes {
        if change.path == entry.path {
            return Some(change);
        }
    }
    return None;
}

/// The object a captured worktree file holds,
/// as the filtered bytes Git records for it.
fn blob_of(file: &CapturedFile) -> &str {
    return file.blob.as_str();
}

/// The object the path held before the merge,
/// when the path existed then.
fn old_oid_of(change: &Change) -> Option<&str> {
    if !change.existed() {
        return None;
    }
    return Some(change.old_oid.as_str());
}

/// The object the merge makes at the path,
/// when the path exists after it.
fn new_oid_of(change: &Change) -> Option<&str> {
    if !change.exists_after() {
        return None;
    }
    return Some(change.new_oid.as_str());
}

/// The index entry the path held before the merge,
/// when the path existed then.
fn old_entry_of(change: &Change) -> Option<(&str, &str)> {
    if !change.existed() {
        return None;
    }
    return Some((change.old_mode.as_str(), change.old_oid.as_str()));
}

/// The index entry the merge makes at the path,
/// when the path exists after it.
fn new_entry_of(change: &Change) -> Option<(&str, &str)> {
    if !change.exists_after() {
        return None;
    }
    return Some((change.new_mode.as_str(), change.new_oid.as_str()));
}

/// The mode and object of one captured index entry.
fn index_pair(staged: &CapturedIndex) -> (&str, &str) {
    return (staged.mode.as_str(), staged.oid.as_str());
}

/// The runner that merges one layer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runner = { run: (args, input, index) => startGit(context, root, args, input, index) };
/// ```
fn rooted(context: &GitContext) -> GitContext {
    // The worktree's own shapes are read from the top of the worktree, where the invocation's global options do not reach.
    return GitContext { global_prefix: Vec::new(), ..context.clone() };
}

/// The runner that merges one layer,
/// from the top of the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runner = { run: (args, input, index) => startGit(rooted(context), root, args, input, index) };
/// ```
fn runner_of<'a>(context: &'a GitContext, root: &'a Path) -> ContextRunner<'a> {
    return ContextRunner { context, cwd: root, without_prefix: true, repository: Vec::new(), object_directory: None, environment: Vec::new() };
}

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
    return match run_on_runtime(clear_awaiting(context, root, git_dir, changes, capture, transaction_id, wait, out)) {
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
    put_back_stranded_takes(root, git_dir, changes, out);
    let mut steps: Vec<Step> = Vec::with_capacity(capture.paths.len());
    for entry in &capture.paths {
        let Some(change) = change_for(changes, entry) else { continue };
        let location: std::path::PathBuf = worktree_path(root, entry.path.as_slice());
        // The identity is read before the content, so a file written in between does not match the judgement and is never
        //  replaced, and the write step reads it again before it writes anything.
        let identity: Option<FileIdentity> = file_identity(location.as_path());
        let state: WorktreeState = worktree_state_awaiting(&rooted(context), root, entry.path.as_slice()).await.map_err(CarryFailure::Failed)?;
        let captured_object: Option<&str> = entry.worktree.as_ref().map(blob_of);
        let old_object: Option<&str> = old_oid_of(change);
        if !explained(&state, &[captured_object, old_object]) {
            return Err(CarryFailure::Foreign(entry.path.clone()));
        }
        let current: Option<(String, String)> = index_entry_awaiting(context, root, entry.path.as_slice()).await.map_err(CarryFailure::Failed)?;
        let captured_entry: Option<(&str, &str)> = entry.index.as_ref().map(index_pair);
        let old_entry: Option<(&str, &str)> = old_entry_of(change);
        if !entry_explained(&current, &[captured_entry, old_entry]) {
            return Err(CarryFailure::Foreign(entry.path.clone()));
        }
        let index: IndexWrite = match old_entry {
            Some((mode, oid)) => IndexWrite::Entry(String::from(mode), String::from(oid)),
            None => IndexWrite::Remove,
        };
        let worktree: FileWrite = match old_object {
            Some(oid) => FileWrite::Bytes(file_form_awaiting(context, root, entry.path.as_slice(), oid).await.map_err(CarryFailure::Failed)?, change.old_mode.clone()),
            None => FileWrite::Remove,
        };
        steps.push(Step { path: entry.path.clone(), index, worktree, judged: judged_of(&state, identity) });
    }
    let written: StepResult = write_steps_awaiting(context, root, git_dir, steps.as_slice(), transaction_id, wait, Batch::Clear, out).await?;
    // A path the write step found newer than its judgement stops the clear: the merge must not run over a checkout this step
    //  only half cleared, and the round is made again on the bytes the person has now.
    return match written.changed.first() {
        Some(path) => Err(CarryFailure::Changed(path.clone())),
        None => Ok(()),
    };
}

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
    return match run_on_runtime(carry_awaiting(context, root, git_dir, changes, capture, landed, transaction_id, wait, out)) {
        Ok(found) => found,
        Err(refusal) => Err(CarryFailure::Failed(format!("{refusal}"))),
    };
}

/// What:
///  The same merge of every captured layer,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
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
    put_back_stranded_takes(root, git_dir, changes, out);
    let mut report: Report = Report::default();
    let mut steps: Vec<Step> = Vec::with_capacity(capture.paths.len());
    for entry in &capture.paths {
        let Some(change) = change_for(changes, entry) else { continue };
        let location: std::path::PathBuf = worktree_path(root, entry.path.as_slice());
        // The identity is read before the content, so a file written in between does not match the judgement and is never
        //  replaced, and the write step reads it again before it writes anything.
        let standing: LandedFile = landed_file_at(location.as_path(), entry.path.as_slice());
        let identity: Option<FileIdentity> = standing.identity;
        let state: WorktreeState = worktree_state_awaiting(&rooted(context), root, entry.path.as_slice()).await.map_err(CarryFailure::Failed)?;
        let captured_object: Option<&str> = entry.worktree.as_ref().map(blob_of);
        let merged_object: Option<&str> = new_oid_of(change);
        let (worktree, note, target) = plan_worktree_awaiting((&runner, context, root), change, entry).await?;
        let index: IndexWrite = plan_index_awaiting(&runner, change, entry).await?;
        // What stands at the path is the capture, the merge's own, or the bytes this carry writes:
        // a carry that was killed after writing one layer finds its
        //  own bytes there and writes the same ones again. Anything else was written by someone else since the capture, and
        //  a layer somebody else wrote keeps their bytes while the other layer is still carried. Where the identity of what
        //  Git's own update left is known, the file must still be that one: a save whose bytes accept the whole merge is the
        //  person's newer decision, not this carry's own write.
        let worktree_is_ours: bool =
            explained(&state, &[captured_object, merged_object, target.as_deref()]) && the_file_git_landed(landed, &standing);
        let current: Option<(String, String)> = index_entry_awaiting(context, root, entry.path.as_slice()).await.map_err(CarryFailure::Failed)?;
        let captured_entry: Option<(&str, &str)> = entry.index.as_ref().map(index_pair);
        let merged_entry: Option<(&str, &str)> = new_entry_of(change);
        let old_entry: Option<(&str, &str)> = old_entry_of(change);
        // The entry this carry writes is its own, so a kill between the index records and the files finds it and writes the
        //  same layers again rather than reading its own work as someone else's.
        let index_is_ours: bool = entry_explained(&current, &explained_entries(&[captured_entry, merged_entry, old_entry], &index));
        if !worktree_is_ours && !index_is_ours {
            report.changed.push(entry.path.clone());
            continue;
        }
        // What the index holds once this step's records are written, which the line about a path the merge deletes names:
        //  `git rm` needs an entry and `rm` must not be given one.
        let in_the_index: bool = match (&index, index_is_ours) {
            (IndexWrite::Entry(_, _), true) => true,
            (IndexWrite::Remove, true) => false,
            _ => current.is_some(),
        };
        if worktree_is_ours {
            match note {
                Note::Merged => report.merged.push(entry.path.clone()),
                Note::Conflict => report.conflicts.push(entry.path.clone()),
                Note::Kept(why) => report.kept.push(kept_path(entry, change, why, in_the_index)),
                Note::None => {}
            }
        } else {
            // This layer is someone else's, and the other one is still carried.
            report.changed.push(entry.path.clone());
        }
        let written: Step = Step {
            path: entry.path.clone(),
            index: if index_is_ours { index } else { IndexWrite::Leave },
            worktree: if worktree_is_ours { worktree } else { FileWrite::Leave },
            judged: judged_of(&state, identity),
        };
        steps.push(written);
    }
    let written: StepResult = write_steps_awaiting(context, root, git_dir, steps.as_slice(), transaction_id, wait, Batch::Carry, out).await?;
    // A path another program wrote while this carry ran keeps its bytes, and the line says so rather than claiming the file
    //  changed under a step that had in fact written it.
    for path in written.changed {
        report.changed.push(path);
    }
    return Ok(report);
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
    return match run_on_runtime(restore_awaiting(context, root, git_dir, changes, capture, transaction_id, wait, out)) {
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
    put_back_stranded_takes(root, git_dir, changes, out);
    let mut steps: Vec<Step> = Vec::with_capacity(capture.paths.len());
    for entry in &capture.paths {
        let location: std::path::PathBuf = worktree_path(root, entry.path.as_slice());
        // The identity is read before the content, so a file written in between does not match the judgement and is never
        //  replaced, and the write step reads it again before it writes anything.
        let identity: Option<FileIdentity> = file_identity(location.as_path());
        let state: WorktreeState = worktree_state_awaiting(&rooted(context), root, entry.path.as_slice()).await.map_err(CarryFailure::Failed)?;
        let captured_object: Option<&str> = entry.worktree.as_ref().map(blob_of);
        // The branch's own content is what a clear leaves, so it is what a restore may replace.
        let change: Option<&Change> = change_for(changes, entry);
        let old_object: Option<&str> = change.and_then(old_oid_of);
        let merged_object: Option<&str> = change.and_then(new_oid_of);
        // A path written after the capture holds the person's own newer bytes: it is left as it is and named, because
        //  replacing it is what loses a save, and the merge it belongs to did not land.
        let worktree_is_ours: bool = explained(&state, &[captured_object, old_object, merged_object]);
        let current: Option<(String, String)> = index_entry_awaiting(context, root, entry.path.as_slice()).await.map_err(CarryFailure::Failed)?;
        let captured_entry: Option<(&str, &str)> = entry.index.as_ref().map(index_pair);
        let old_entry: Option<(&str, &str)> = change.and_then(old_entry_of);
        let merged_entry: Option<(&str, &str)> = change.and_then(new_entry_of);
        let index_is_ours: bool = entry_explained(&current, &[captured_entry, old_entry, merged_entry]);
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
        let worktree: FileWrite = match entry.worktree.as_ref() {
            Some(file) => FileWrite::Bytes(raw_bytes_awaiting(context, root, file.raw.as_str()).await.map_err(CarryFailure::Failed)?, file.mode.clone()),
            None => FileWrite::Remove,
        };
        let put_back: Step = Step {
            path: entry.path.clone(),
            index: if index_is_ours { index } else { IndexWrite::Leave },
            worktree: if worktree_is_ours { worktree } else { FileWrite::Leave },
            judged: judged_of(&state, identity),
        };
        steps.push(put_back);
    }
    let _written: StepResult = write_steps_awaiting(context, root, git_dir, steps.as_slice(), transaction_id, wait, Batch::Restore, out).await?;
    return Ok(());
}

/// Whether nothing but the branch's own content is at a captured path,
/// which is what a merge that never landed leaves behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function onlyTheBranchesContent(context, root, changes, capture): Promise<boolean>;
/// ```
pub fn only_the_branches_content(context: &GitContext, root: &Path, changes: &[Change], capture: &Capture) -> Result<bool, String> {
    for entry in &capture.paths {
        let Some(change) = change_for(changes, entry) else { continue };
        let state: WorktreeState = worktree_state(&rooted(context), root, entry.path.as_slice())?;
        let captured_object: Option<&str> = entry.worktree.as_ref().map(blob_of);
        let old_object: Option<&str> = old_oid_of(change);
        if !explained(&state, &[captured_object, old_object]) {
            return Ok(false);
        }
    }
    return Ok(true);
}

/// The writing half of a carry,
/// judged and written one path at a time.
#[cfg(test)]
#[path = "merge_isolation_carry_apply_tests.rs"]
mod tests;
