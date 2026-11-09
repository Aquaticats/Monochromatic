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
//!  and the owner of each decides whether it still runs.
//!  A claim whose owner is dead,
//!  and a claim that records no destination (a copy's own),
//!  say nothing about the source,
//!  so neither holds a copy back.
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
///  Whether `source_root` is the destination of a copy whose owner still runs.
/// Why:
///   A creating command stages the source's ignored state;
/// a source that is itself receiving one holds only part of it,
///  so the copy takes nothing from it and the person is told why.
///  The two kinds of item are read in the same order recovery reads them,
///  and an owner the platform will not report on counts as alive,
///  as it does everywhere else in this folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function sourceIsStillBeingFilled({ commonDir, sourceRoot }): Promise<boolean>;
/// ```
///
/// # Errors
///  When a journal or a claim cannot be read,
///  or an item of another process-id namespace is older than the
///  foreign-owner patience.
pub fn source_is_still_being_filled(
    common_dir: &Path,
    source_root: &Path,
) -> Result<bool, WorktreeCopyError> {
    for pending in read_pending_journals(common_dir)? {
        // A journal of another destination is none of this command's business.
        if !names_root(Path::new(pending.journal.destination_root.as_str()), source_root) {
            continue;
        }
        // A copy whose owner still runs is filling this source; a dead owner's journal is recovery's to end.
        if !journal_owner_is_dead(&pending)? {
            debug(
                "worktreeCopySource",
                format!(
                    "{} is still being filled by the journal {}",
                    source_root.display(),
                    pending.path.display()
                )
                .as_str(),
            );
            return Ok(true);
        }
    }
    let root: PathBuf = journal_root(common_dir);
    for name in claim_names(root.as_path())? {
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
        // A claim of a live owner is its add or copy still running; one no owner is written into counts as a dead owner's.
        if !token_owner_is_dead(owner.as_deref(), claim.as_path())? {
            debug(
                "worktreeCopySource",
                format!(
                    "{} is still being filled by the claim {}",
                    source_root.display(),
                    claim.display()
                )
                .as_str(),
            );
            return Ok(true);
        }
    }
    return Ok(false);
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
fn claim_destination(claim: &Path) -> Option<PathBuf> {
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
pub fn source_still_being_filled_notice(destination: &Path, source: &Path) -> String {
    let shown_destination: String = quote(destination.to_string_lossy().as_ref());
    return format!(
        "monochromatic-git-policy-cli: {shown_destination} was made, but it received no ignored-state copy, because {} is itself being filled by another `git worktree add`, so its ignored files are only partly there. Once that add has finished, `git worktree remove {shown_destination}` and run this `git worktree add` line again to have the ignored state copied; `git worktree remove` leaves a worktree that holds changes alone, so save them first.\n",
        quote(source.to_string_lossy().as_ref())
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_source_still_being_filled_tests.rs"]
mod tests;
