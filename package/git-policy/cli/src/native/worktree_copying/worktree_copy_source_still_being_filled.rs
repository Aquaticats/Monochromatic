//! What:
//!  Whether one source worktree is itself the destination of an ignored-state copy that has not finished,
//!  and the line that tells the person a new worktree received no copy because of it.
//! Why:
//!  A worktree whose own copy still runs holds only part of its source's ignored files.
//!  Staging from it would give a second worktree the same half-filled set and say nothing about it,
//!  because that second copy succeeds.
//!  The answer comes from the durable items this folder already judges:
//!  a journal names its destination in its record,
//!  an add's claim names it in the record written beside the claim,
//!  and either one that names the source holds a copy back,
//!  whether its owner still runs or died before it finished:
//!  a dead owner's install did not complete either,
//!  so the source still holds only part of its ignored files.
//!  The owner decides only which sentence the person reads.
//!  A claim that records no destination (a copy's own) says nothing about the
//!  source,
//!  so it never holds a copy back.
//!  Claims are read before journals because the copy that owns a stage publishes
//!  its journal before it releases its claim:
//!  a reader that finds no claim is then looking at a stage whose journal already
//!  exists,
//!  and a reader that finds no journal saw no claim either.
//!  Nothing waits and nothing is locked:
//!  a source that is still being filled makes the command copy nothing and say so.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const busy = await sourceIsStillBeingFilled({ commonDir, sourceRoot });
//! if (busy) stderr.write(sourceStillBeingFilledNotice({ destination, source }));
//! ```

use crate::diagnostic_log::debug;
use crate::json_record::quote;
use crate::worktree_add_binding::{Recorded, decode_record};
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_journal::{journal_owner_is_dead, journal_root, read_pending_journals};
use crate::worktree_copy_owner::token_owner_is_dead;
use crate::worktree_copy_stage_claim::{DESTINATION_SUFFIX, claim_names};
use crate::worktree_copy_stage_claim_reaping::read_claim;
use std::path::{Path, PathBuf};

/// What:
///  The copy that has not finished filling one source,
/// by whether the process that owns it still runs.
/// Why:
///   The person reads a different sentence for a copy that runs now and for one
/// that was interrupted,
/// and both hold the new copy back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnfinishedCopy = 'running' | 'interrupted';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnfinishedCopy {
    /// The owner still runs:
    /// the copy is filling the source now.
    Running,
    /// The owner died before its install finished:
    /// the source holds part of its ignored files until recovery,
    /// or a new add,
    /// completes it.
    Interrupted,
}

/// What:
///  The copy that has not finished filling `source_root`,
/// when one is recorded.
/// Why:
///   A creating command stages the source's ignored state;
/// a source that is itself receiving one holds only part of it,
///  so the copy takes nothing from it and the person is told why.
///  A dead owner's item counts as well as a live one's:
/// its install did not finish either,
///  and a later command that finishes it does not repair the copies already
/// taken from the partial source.
///  An owner the platform will not report on counts as alive,
///  as it does everywhere else in this folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function sourceIsStillBeingFilled({ commonDir, sourceRoot }): Promise<UnfinishedCopy | undefined>;
/// ```
///
/// # Errors
///  When a journal or a claim cannot be read,
///  or an item of another process-id namespace is older than the
///  foreign-owner patience.
pub fn source_is_still_being_filled(
    common_dir: &Path,
    source_root: &Path,
) -> Result<Option<UnfinishedCopy>, WorktreeCopyError> {
    return source_is_still_being_filled_around(common_dir, source_root, &mut nothing_between);
}

/// What:
///  The ceiling that runs nothing between the two scans:
/// the answer a caller that observes no handoff asks for.
/// Why:
///   `between` is the control's seam,
/// and the command itself runs no code at that point.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nothingBetween(): void {}
/// ```
fn nothing_between() {}

/// What:
///  `source_is_still_being_filled`,
/// with `between` run once after the claims are read and before the journals are.
/// Why:
///   A control holds the copy's own handoff there,
/// publishing its journal and releasing its claim,
/// so the one instant the two scans could disagree is exercised without timing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function sourceIsStillBeingFilledAround({ commonDir, sourceRoot, between });
/// ```
pub fn source_is_still_being_filled_around(
    common_dir: &Path,
    source_root: &Path,
    between: &mut dyn FnMut(),
) -> Result<Option<UnfinishedCopy>, WorktreeCopyError> {
    // Claims come first: the copy that owns a stage publishes its journal before it releases its claim, so a reader that
    // reads the claims first and then the journals cannot see neither. `between` runs after the claim names are listed and
    // before any of them is read, so a control holds the copy's own handoff there: the journal is published and the claim
    // released after the listing, and the journal scan that follows still finds the copy.
    let root: PathBuf = journal_root(common_dir);
    let names: Vec<String> = claim_names(root.as_path())?;
    between();
    for name in names {
        let claim: PathBuf = root.join(name.as_str());
        // A copy's own claim records no destination, so it never makes a worktree its own source's business.
        let Some(recorded) = claim_destination(claim.as_path()) else {
            continue;
        };
        if !names_root(recorded.as_path(), source_root) {
            continue;
        }
        let Some((_container, owner)) = read_claim(claim.as_path())? else {
            continue;
        };
        // A claim of a live owner is its add still running; a dead owner's add never finished, so both hold the copy back.
        let unfinished: UnfinishedCopy = match token_owner_is_dead(owner.as_deref(), claim.as_path())? {
            true => UnfinishedCopy::Interrupted,
            false => UnfinishedCopy::Running,
        };
        debug(
            "worktreeCopySource",
            format!(
                "{} is still being filled by the claim {}",
                source_root.display(),
                claim.display()
            )
            .as_str(),
        );
        return Ok(Some(unfinished));
    }
    for pending in read_pending_journals(common_dir)? {
        // A journal of another destination is none of this command's business.
        if !names_root(pending.journal.destination_root.as_path(), source_root) {
            continue;
        }
        // A live owner is installing into this source now; a dead owner's install stopped before it finished.
        let unfinished: UnfinishedCopy = match journal_owner_is_dead(&pending)? {
            true => UnfinishedCopy::Interrupted,
            false => UnfinishedCopy::Running,
        };
        debug(
            "worktreeCopySource",
            format!(
                "{} is still being filled by the journal {}",
                source_root.display(),
                pending.path.display()
            )
            .as_str(),
        );
        return Ok(Some(unfinished));
    }
    return Ok(None);
}

/// What:
///  Whether a path an item records is the source worktree,
/// compared by the one canonical spelling both exist under.
/// Why:
///  Records hold the path as its command line named it,
/// which a symbolic link,
/// a trailing separator or a symlinked ancestor can spell differently from the
///  canonical source root;
/// a recorded path that no longer exists names no worktree to copy from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// canonicalize(recorded) === sourceRoot
/// ```
fn names_root(recorded: &Path, source_root: &Path) -> bool {
    // `let-else` keeps the deleted destination out of the comparison; nothing can be copied from a path that is gone.
    let Ok(canonical) = std::fs::canonicalize(recorded) else {
        return false;
    };
    return canonical.as_path() == source_root;
}

/// What:
///  The destination an add's claim records,
/// or nothing when the claim is a copy's own and records none.
/// Why:
///   The record is written before real Git runs and replaced with the binding after it,
/// so both forms name the worktree the claim is about.
/// A record that cannot be read names no destination;
/// recovery reports such a record when it settles the add.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = claim.replace(/\.stage$/, '.add'); return decodeRecord(await readFile(record))?.destination;
/// ```
pub(crate) fn claim_destination(claim: &Path) -> Option<PathBuf> {
    let record: PathBuf = claim.with_extension(&DESTINATION_SUFFIX[1..]);
    let text: String = std::fs::read_to_string(record).ok()?;
    return match decode_record(text.as_str())? {
        Recorded::Planned(destination) => Some(destination),
        Recorded::Registered(binding) => Some(binding.destination),
    };
}

/// What:
///  The one line for a worktree that received no ignored-state copy because its source is still being filled.
/// Why:
///   A worktree without the source's ignored files is told why,
/// and the command that has the copy made once the other add has finished:
///  removing this worktree and running the same add again is the only route the wrapper has,
/// and `git worktree remove` leaves
///  a worktree that holds changes alone,
/// so nothing the person wrote is discarded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: ${destination} was made, but it received no ignored-state copy, because ${source} ...`
/// ```
pub fn source_still_being_filled_notice(destination: &Path, source: &Path, unfinished: UnfinishedCopy) -> String {
    let shown_destination: String = quote(destination.to_string_lossy().as_ref());
    let shown_source: String = quote(source.to_string_lossy().as_ref());
    let why: String = match unfinished {
        UnfinishedCopy::Running => format!("because {shown_source} is itself being filled by another `git worktree add`, so its ignored files are only partly there. Once that add has finished"),
        UnfinishedCopy::Interrupted => format!("because the ignored-state copy of {shown_source} was interrupted before it finished, so its ignored files are only partly there. Once a command has completed that copy"),
    };
    return format!(
        "monochromatic-git-policy-cli: {shown_destination} was made, but it received no ignored-state copy, {why}, `git worktree remove {shown_destination}` and run this `git worktree add` line again to have the ignored state copied; `git worktree remove` leaves a worktree that holds changes alone, so save them first.\n"
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_source_still_being_filled_tests.rs"]
mod tests;
