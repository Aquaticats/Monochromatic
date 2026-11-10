//! What:
//!  What decides,
//! path by path,
//! how a merge carries a person's own work:
//! putting the merge's paths back to the branch before
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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await clear({ context, root, changes, capture }); await carry({ context, root, changes, capture });
//! ```

/// The decisions a path's layers come to,
/// and the planning of one path's writes.
use crate::merge_isolation_carry_plans::{Note, plan_index, plan_worktree};
/// What one carried path adds to the report.
use crate::merge_isolation_carry_notice::Report;
/// The person's work as the capture recorded it.
use crate::merge_isolation_carry_parts::{CapturedFile, CapturedIndex, CapturedPath, Capture};
/// The Git reads of one path,
/// which this file judges and never spells for Git itself.
use crate::merge_isolation_carry_reads::{file_form, index_entry, raw_bytes};
/// The write plan of one path,
/// and the one step that lands it.
use crate::merge_isolation_carry_writes::{FileWrite, IndexWrite, Step, write_steps};
/// The paths the merge changes,
/// and the state a path holds now.
use crate::merge_isolation_changes::{Change, WorktreeState, worktree_state};
/// The runner the word-level pass is given,
/// and the Git context every read and write names.
use crate::merge_second_pass::ContextRunner;
/// The Git context a read and a write take.
use crate::transaction_git::GitContext;
/// A path of the worktree.
use std::path::Path;

/// The paths the merge changes,
/// read again by whoever holds a `merge_isolation_carry_reads` name.
pub use crate::merge_isolation_carry_reads::changes_between;
/// Why a carry stopped.
pub use crate::merge_isolation_carry_writes::{CarryFailure, LockWait};















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
    return ContextRunner { context, cwd: root, without_prefix: true, object_directory: None, environment: Vec::new() };
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
    wait: LockWait,
    out: &mut dyn std::io::Write,
) -> Result<(), CarryFailure> {
    let mut steps: Vec<Step> = Vec::with_capacity(capture.paths.len());
    for entry in &capture.paths {
        let Some(change) = change_for(changes, entry) else { continue };
        let state: WorktreeState = worktree_state(&rooted(context), root, entry.path.as_slice()).map_err(CarryFailure::Failed)?;
        let captured_object: Option<&str> = entry.worktree.as_ref().map(blob_of);
        let old_object: Option<&str> = old_oid_of(change);
        if !explained(&state, &[captured_object, old_object]) {
            return Err(CarryFailure::Foreign(entry.path.clone()));
        }
        let current: Option<(String, String)> = index_entry(context, root, entry.path.as_slice()).map_err(CarryFailure::Failed)?;
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
            Some(oid) => FileWrite::Bytes(file_form(context, root, entry.path.as_slice(), oid).map_err(CarryFailure::Failed)?, change.old_mode.clone()),
            None => FileWrite::Remove,
        };
        steps.push(Step { path: entry.path.clone(), index, worktree });
    }
    return write_steps(context, root, git_dir, steps.as_slice(), wait, out);
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
    wait: LockWait,
    out: &mut dyn std::io::Write,
) -> Result<Report, CarryFailure> {
    // The merge of one layer runs from the top of the worktree, where a relative global option of the invocation does not reach.
    let rooted_context: GitContext = rooted(context);
    let runner: ContextRunner<'_> = runner_of(&rooted_context, root);
    let mut report: Report = Report::default();
    let mut steps: Vec<Step> = Vec::with_capacity(capture.paths.len());
    for entry in &capture.paths {
        let Some(change) = change_for(changes, entry) else { continue };
        let state: WorktreeState = worktree_state(&rooted(context), root, entry.path.as_slice()).map_err(CarryFailure::Failed)?;
        let captured_object: Option<&str> = entry.worktree.as_ref().map(blob_of);
        let merged_object: Option<&str> = new_oid_of(change);
        let (worktree, note, target) = plan_worktree((&runner, context, root), change, entry)?;
        // What stands there is the capture, the merge's own, or the bytes this carry writes:
        // a carry that was killed after writing a path finds its
        //  own bytes and writes the same ones again. Anything else was written by someone else since the capture.
        if !explained(&state, &[captured_object, merged_object, target.as_deref()]) {
            report.changed.push(entry.path.clone());
            continue;
        }
        let current: Option<(String, String)> = index_entry(context, root, entry.path.as_slice()).map_err(CarryFailure::Failed)?;
        let captured_entry: Option<(&str, &str)> = entry.index.as_ref().map(index_pair);
        let merged_entry: Option<(&str, &str)> = new_entry_of(change);
        let old_entry: Option<(&str, &str)> = old_entry_of(change);
        if !entry_explained(&current, &[captured_entry, merged_entry, old_entry]) {
            report.changed.push(entry.path.clone());
            continue;
        }
        let index: IndexWrite = plan_index(&runner, change, entry)?;
        match note {
            Note::Merged => report.merged.push(entry.path.clone()),
            Note::Conflict => report.conflicts.push(entry.path.clone()),
            Note::Kept(why) => report.kept.push((entry.path.clone(), why)),
            Note::None => {}
        }
        steps.push(Step { path: entry.path.clone(), index, worktree });
    }
    write_steps(context, root, git_dir, steps.as_slice(), wait, out)?;
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
    wait: LockWait,
    out: &mut dyn std::io::Write,
) -> Result<(), CarryFailure> {
    let mut steps: Vec<Step> = Vec::with_capacity(capture.paths.len());
    for entry in &capture.paths {
        let state: WorktreeState = worktree_state(&rooted(context), root, entry.path.as_slice()).map_err(CarryFailure::Failed)?;
        let captured_object: Option<&str> = entry.worktree.as_ref().map(blob_of);
        // The branch's own content is what a clear leaves, so it is what a restore may replace.
        let change: Option<&Change> = change_for(changes, entry);
        let old_object: Option<&str> = change.and_then(old_oid_of);
        let merged_object: Option<&str> = change.and_then(new_oid_of);
        if !explained(&state, &[captured_object, old_object, merged_object]) {
            return Err(CarryFailure::Foreign(entry.path.clone()));
        }
        let current: Option<(String, String)> = index_entry(context, root, entry.path.as_slice()).map_err(CarryFailure::Failed)?;
        let captured_entry: Option<(&str, &str)> = entry.index.as_ref().map(index_pair);
        let old_entry: Option<(&str, &str)> = change.and_then(old_entry_of);
        let merged_entry: Option<(&str, &str)> = change.and_then(new_entry_of);
        if !entry_explained(&current, &[captured_entry, old_entry, merged_entry]) {
            return Err(CarryFailure::Foreign(entry.path.clone()));
        }
        let index: IndexWrite = match entry.index.as_ref() {
            Some(staged) => IndexWrite::Entry(staged.mode.clone(), staged.oid.clone()),
            None => IndexWrite::Remove,
        };
        let worktree: FileWrite = match entry.worktree.as_ref() {
            Some(file) => FileWrite::Bytes(raw_bytes(context, root, file.raw.as_str()).map_err(CarryFailure::Failed)?, file.mode.clone()),
            None => FileWrite::Remove,
        };
        steps.push(Step { path: entry.path.clone(), index, worktree });
    }
    return write_steps(context, root, git_dir, steps.as_slice(), wait, out);
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
