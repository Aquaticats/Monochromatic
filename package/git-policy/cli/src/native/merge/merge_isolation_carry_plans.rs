//! What:
//!  What one captured path's writes are,
//! once the merge's change is known:
//!  the worktree side,
//! through the word-level pass when the two sides differ,
//!  and the index side,
//!  which is a layer of its own.
//! Why:
//!   The carry decides each path from the person's two layers and the merge's side,
//!  and the report says of it what the decision came to;
//!  the step that lands these writes is `merge_isolation_carry_writes.rs`
//!  and the judgements that call them are `merge_isolation_carry_apply.rs`,
//!  which is at its code-line limit.
//!  Both plans are awaited,
//!  because the word-level pass awaits every Git start it makes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [write, note, target] = await planWorktreeAwaiting(places, change, entry);
//! ```

use crate::merge_isolation_carry_layers::{Decision, Layer, Outcome, Side, MODIFY_DELETE, decide, write_layer_bytes};
use crate::merge_isolation_carry_parts::{CapturedFile, CapturedIndex, CapturedPath};
use crate::merge_isolation_carry_permissions::{carries_a_mode_a_person_set, the_mode_with_the_persons_bits};
use crate::merge_isolation_carry_reads::{file_form_awaiting, raw_bytes_awaiting};
use crate::merge_isolation_carry_writes::{CarryFailure, FileWrite, IndexWrite};
use crate::merge_isolation_changes::Change;
use crate::merge_second_pass::GitRunner;
use crate::transaction_git::GitContext;
use std::path::Path;

/// The mode a plain file is written with when neither side names one.
const PLAIN_FILE_MODE: &str = "100644";

/// What one carried path adds to the report.
pub(crate) enum Note {
    /// Nothing worth a line.
    None,
    /// The word pass merged it.
    Merged,
    /// Both sides stand in the file between markers.
    Conflict,
    /// The person's bytes stand whole,
    /// for this reason.
    Kept(&'static str),
}

/// The side of the change at one path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const side = (change, after) => after ? (change.existsAfter ? { mode: change.newMode, oid: change.newOid } : null) : (change.existed ? { mode: change.oldMode, oid: change.oldOid } : null);
/// ```
pub(crate) fn side(change: &Change, after: bool) -> Option<Side> {
    let (held, mode, oid): (bool, &String, &String) = if after {
        (change.exists_after(), &change.new_mode, &change.new_oid)
    } else {
        (change.existed(), &change.old_mode, &change.old_oid)
    };
    if !held {
        return None;
    }
    return Some(Side { mode: mode.clone(), oid: oid.clone() });
}

/// What:
///  The permission bits the capture read for one worktree entry.
/// Why:
///   A record an older release wrote holds none,
/// and a link has none of its own,
///  so a write takes Git's mode alone where this answers nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const thePermissionsOf = (file) => file.permissions;
/// ```
fn the_permissions_of(file: &CapturedFile) -> Option<u32> {
    return file.permissions;
}

/// The mode a written path carries when neither side names one.
pub(crate) fn plain_file_mode() -> String {
    return String::from(PLAIN_FILE_MODE);
}

/// What:
///  The worktree side of one captured path.
/// Why:
///  The layer's decision and what the report says of it are one answer,
///  and the decision is awaited
///  because the pass behind it awaits every Git start it makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function planWorktreeAwaiting(places, change, entry, state): Promise<[FileWrite, Note]>;
/// ```
pub(crate) async fn plan_worktree_awaiting(
    places: (&dyn GitRunner, &GitContext, &Path),
    change: &Change,
    entry: &CapturedPath,
) -> Result<(FileWrite, Note, Option<String>), CarryFailure> {
    let (runner, context, root) = places;
    let layer: Layer = Layer { base: side(change, false), ours: entry.worktree.as_ref().map(side_of_file), theirs: side(change, true) };
    // A mode the person set at a path the merge deletes is the whole of what the deletion would discard, and the entry rules
    //  read it as no change at all when the bytes and Git's executable bit are the branch's own: nothing of the merge's stands
    //  there to take the mode, so their file stays with it and is named (a Kimi K3.1 Pro audit of `0.6.1-alpha.12`, its second
    //  finding).
    if !change.exists_after()
        && let Some(file) = &entry.worktree
        && carries_a_mode_a_person_set(file.mode.as_str(), file.permissions)
    {
        let bytes: Vec<u8> = raw_bytes_awaiting(context, root, file.raw.as_str()).await.map_err(CarryFailure::Failed)?;
        return Ok((FileWrite::Bytes(bytes, the_mode_with_the_persons_bits(file.mode.as_str(), file.permissions)), Note::Kept(MODIFY_DELETE), Some(file.blob.clone())));
    }
    let decision: Decision = decide(runner, entry.path.as_slice(), &layer).await.map_err(CarryFailure::Failed)?;
    // Every bit the person set goes with the bytes this plan writes, because Git's mode records the executable bit alone and the
    //  entry a write replaces is not there when a path the merge deleted is made again.
    let mode: String = the_mode_with_the_persons_bits(decision.mode.as_deref().unwrap_or(PLAIN_FILE_MODE), entry.worktree.as_ref().and_then(the_permissions_of));
    return match decision.outcome {
        // The merge's own bytes with a mode of the person's: the layer holds nothing of theirs to merge, but the mode they
        //  set is theirs, and Git's update writes the merge's own.
        Outcome::Merges => match side(change, true) {
            Some(merge) if merge.mode != mode => {
                let form: Vec<u8> = file_form_awaiting(context, root, entry.path.as_slice(), merge.oid.as_str()).await.map_err(CarryFailure::Failed)?;
                Ok((FileWrite::Bytes(form, mode), Note::None, Some(merge.oid.clone())))
            }
            _ => Ok((FileWrite::Leave, Note::None, None)),
        },
        Outcome::Ours => match &entry.worktree {
            Some(file) => Ok((
                FileWrite::Bytes(raw_bytes_awaiting(context, root, file.raw.as_str()).await.map_err(CarryFailure::Failed)?, mode),
                Note::None,
                Some(file.blob.clone()),
            )),
            None => Ok((FileWrite::Remove, Note::None, None)),
        },
        Outcome::Merged { bytes, conflict, .. } => {
            let oid: String = write_layer_bytes(runner, bytes.as_slice()).await.map_err(CarryFailure::Failed)?;
            let form: Vec<u8> = file_form_awaiting(context, root, entry.path.as_slice(), oid.as_str()).await.map_err(CarryFailure::Failed)?;
            return Ok((FileWrite::Bytes(form, mode), if conflict { Note::Conflict } else { Note::Merged }, Some(oid)));
        }
        Outcome::Kept(why) => match &entry.worktree {
            Some(file) => Ok((
                FileWrite::Bytes(raw_bytes_awaiting(context, root, file.raw.as_str()).await.map_err(CarryFailure::Failed)?, mode),
                Note::Kept(why),
                Some(file.blob.clone()),
            )),
            None => Ok((FileWrite::Remove, Note::Kept(why), None)),
        },
        Outcome::Gone(why) => Ok((FileWrite::Remove, Note::Kept(why), None)),
    };
}

/// What:
///  The index side of one captured path.
/// Why:
///   A staged edit is a layer of its own:
/// it stays staged over the merged file when it merges,
/// and holds both
///  sides when it does not.
///  The decision is awaited,
///  because the pass behind it awaits every Git start it makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function planIndexAwaiting(runner, context, root, change, entry): Promise<IndexWrite>;
/// ```
pub(crate) async fn plan_index_awaiting(
    runner: &dyn GitRunner,
    change: &Change,
    entry: &CapturedPath,
) -> Result<IndexWrite, CarryFailure> {
    let base: Option<Side> = side(change, false);
    let ours: Option<Side> = entry.index.as_ref().map(side_of_index);
    if let (Some(found), Some(person)) = (&base, &ours)
        && found.oid == person.oid
        && found.mode == person.mode
    {
        return Ok(IndexWrite::Leave);
    }
    // Nothing of the person's is in this layer, so the merge's entry is the layer's own.
    if base.is_none() && ours.is_none() {
        return Ok(IndexWrite::Leave);
    }
    let layer: Layer = Layer { base, ours: ours.clone(), theirs: side(change, true) };
    let decision: Decision = decide(runner, entry.path.as_slice(), &layer).await.map_err(CarryFailure::Failed)?;
    let mode: String = decision.mode.clone().unwrap_or_else(plain_file_mode);
    return match decision.outcome {
        // A staged mode of the person's stays staged over the merge's own object, as their staged bytes do.
        Outcome::Merges => match side(change, true) {
            Some(merge) if merge.mode != mode => Ok(IndexWrite::Entry(mode, merge.oid.clone())),
            _ => Ok(IndexWrite::Leave),
        },
        Outcome::Ours | Outcome::Kept(_) => match &entry.index {
            Some(staged) => Ok(IndexWrite::Entry(staged.mode.clone(), staged.oid.clone())),
            None => Ok(IndexWrite::Remove),
        },
        Outcome::Merged { bytes, .. } => {
            let oid: String = write_layer_bytes(runner, bytes.as_slice()).await.map_err(CarryFailure::Failed)?;
            Ok(IndexWrite::Entry(mode, oid))
        }
        Outcome::Gone(_) => Ok(IndexWrite::Remove),
    };
}

/// The side a captured worktree file is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sideOfFile = (file: CapturedFile): Side => ({ mode: file.mode, oid: file.blob });
/// ```
pub(crate) fn side_of_file(file: &CapturedFile) -> Side {
    return Side { mode: file.mode.clone(), oid: file.blob.clone() };
}

/// The side a captured index entry is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sideOfIndex = (staged: CapturedIndex): Side => ({ mode: staged.mode, oid: staged.oid });
/// ```
pub(crate) fn side_of_index(staged: &CapturedIndex) -> Side {
    return Side { mode: staged.mode.clone(), oid: staged.oid.clone() };
}
