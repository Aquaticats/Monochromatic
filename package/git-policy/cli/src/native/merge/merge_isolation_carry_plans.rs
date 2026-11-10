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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [write, note, target] = await planWorktree(places, change, entry);
//! ```

use crate::merge_isolation_carry_layers::{Decision, Layer, Outcome, Side, decide, write_layer_bytes};
use crate::merge_isolation_carry_parts::{CapturedFile, CapturedIndex, CapturedPath};
use crate::merge_isolation_carry_reads::{file_form, raw_bytes};
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

/// The mode a written path carries when neither side names one.
pub(crate) fn plain_file_mode() -> String {
    return String::from(PLAIN_FILE_MODE);
}

/// What:
///  One awaited step of the word-level pass,
/// run to its end from this synchronous path.
/// Why:
///   The pass awaits every Git start it makes
/// (`merge_second_pass_git_runner.rs`),
/// and the merge that
///  carries a person's work is a command's synchronous step,
/// so this one entry of the process runtime is taken for it.
/// Everything else the carry does is a plain start,
/// which must stay outside the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decision = runOnRuntime(decide(runner, path, layer));
/// ```
pub(crate) fn on_the_runtime<T>(future: impl std::future::Future<Output = Result<T, String>>) -> Result<T, CarryFailure> {
    return match crate::lazy_runtime::run_on_runtime(future) {
        Ok(found) => found.map_err(CarryFailure::Failed),
        Err(error) => Err(CarryFailure::Failed(format!(
            "the word-level pass over a path this merge carries could not run: {error}"
        ))),
    };
}

/// What:
///  The worktree side of one captured path.
/// Why:
///  The layer's decision and what the report says of it are one answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function planWorktree(places, change, entry, state): Promise<[FileWrite, Note]>;
/// ```
pub(crate) fn plan_worktree(
    places: (&dyn GitRunner, &GitContext, &Path),
    change: &Change,
    entry: &CapturedPath,
) -> Result<(FileWrite, Note, Option<String>), CarryFailure> {
    let (runner, context, root) = places;
    let layer: Layer = Layer { base: side(change, false), ours: entry.worktree.as_ref().map(side_of_file), theirs: side(change, true) };
    let decision: Decision = on_the_runtime(decide(runner, entry.path.as_slice(), &layer))?;
    let mode: String = decision.mode.clone().unwrap_or_else(plain_file_mode);
    return match decision.outcome {
        Outcome::Merges => Ok((FileWrite::Leave, Note::None, None)),
        Outcome::Ours => match &entry.worktree {
            Some(file) => Ok((
                FileWrite::Bytes(raw_bytes(context, root, file.raw.as_str()).map_err(CarryFailure::Failed)?, mode),
                Note::None,
                Some(file.blob.clone()),
            )),
            None => Ok((FileWrite::Remove, Note::None, None)),
        },
        Outcome::Merged { bytes, conflict, .. } => {
            let oid: String = on_the_runtime(write_layer_bytes(runner, bytes.as_slice()))?;
            let form: Vec<u8> = file_form(context, root, entry.path.as_slice(), oid.as_str()).map_err(CarryFailure::Failed)?;
            return Ok((FileWrite::Bytes(form, mode), if conflict { Note::Conflict } else { Note::Merged }, Some(oid)));
        }
        Outcome::Kept(why) => match &entry.worktree {
            Some(file) => Ok((
                FileWrite::Bytes(raw_bytes(context, root, file.raw.as_str()).map_err(CarryFailure::Failed)?, mode),
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function planIndex(runner, context, root, change, entry): Promise<IndexWrite>;
/// ```
pub(crate) fn plan_index(
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
    let decision: Decision = on_the_runtime(decide(runner, entry.path.as_slice(), &layer))?;
    let mode: String = decision.mode.clone().unwrap_or_else(plain_file_mode);
    return match decision.outcome {
        Outcome::Merges => Ok(IndexWrite::Leave),
        Outcome::Ours | Outcome::Kept(_) => match &entry.index {
            Some(staged) => Ok(IndexWrite::Entry(staged.mode.clone(), staged.oid.clone())),
            None => Ok(IndexWrite::Remove),
        },
        Outcome::Merged { bytes, .. } => {
            let oid: String = on_the_runtime(write_layer_bytes(runner, bytes.as_slice()))?;
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
