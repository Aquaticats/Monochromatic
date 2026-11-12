//! What:
//!  The two sweeps of the journal root that remove files rather than stages:
//! the temporaries dead writers left,
//! and the records of stage objects that no claim names.
//! Why:
//!  Both read a listing of the private journal root and remove only names a live writer never holds,
//! and both answer whether they removed anything,
//! because the sweep that runs them flushes the directory only when one of them changed it.
//! They live beside the reaping because that file was over the line limit before this change
//! (`worktree_copy_stage_claim_reaping.rs`,
//! which re-exports both).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const temporaries = await removeDeadTemporaries(root, verdicts);
//! const records = await removeRecordsNothingClaims(root);
//! ```

use crate::worktree_copy_error::{WorktreeCopyError, raw_failure, raw_failure_of};
use crate::worktree_copy_journal_removal::remove_file_and_say;
use crate::worktree_copy_stage_claim_reaping::StageVerdicts;
use crate::worktree_copy_stage_made_object::MADE_SUFFIX;
use std::path::{Path, PathBuf};

/// What:
///  Removes the temporaries that dead writers left in the journal root.
/// Why:
///   A write cut off before its rename leaves a file nothing else names.
/// The writer is in the file's name,
/// so the sweep
///  removes only what a dead process left and can never delete a temporary a live writer is about to rename.
///
/// The answer says whether anything was removed,
/// so the caller flushes the directory only when one was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of await readdir(root)) if (name.endsWith('.tmp') && ownerIsDead(writerOf(name))) await rm(join(root, name), { force: true });
/// ```
pub fn remove_dead_temporaries(root: &Path, verdicts: &StageVerdicts) -> Result<bool, WorktreeCopyError> {
    let mut removed: bool = false;
    for name in verdicts.dead_temporary_names() {
        removed |= remove_file_and_say(root.join(name).as_path())?;
    }
    return Ok(removed);
}

/// What:
///  Removes every record of a stage's object that no claim beside it names.
/// Why:
///   A claim is released with its record,
/// but a kill between the two removals leaves the record,
/// and so does a claim another command
///  reaped;
/// a record nothing claims proves nothing about any folder and no later command reads it,
/// so it would stay for ever.
/// The sweep names only records,
/// leaves every other file of the journal root alone,
/// and a file a live writer is renaming is never
///  touched,
/// because a write publishes its record with one rename and no temporary of that name is listed.
///
/// The answer says whether anything was removed,
/// so the caller flushes the directory only when one was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of list(root).filter((name) => name.endsWith(MADE_SUFFIX))) if (!exists(claimOf(name))) await rm(join(root, name));
/// ```
pub(crate) fn remove_records_nothing_claims(root: &Path) -> Result<bool, WorktreeCopyError> {
    // `match` treats an absent root as holding no records, and every other failure as fatal.
    let listing = match std::fs::read_dir(root) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(raw_failure(&error)),
    };
    let mut unclaimed: Vec<PathBuf> = Vec::new();
    for listed in listing {
        let path: PathBuf = listed.map_err(raw_failure_of)?.path();
        let name: String = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let Some(identifier) = name.strip_suffix(MADE_SUFFIX) else { continue };
        if !root.join(format!("{identifier}.stage")).exists() {
            unclaimed.push(path);
        }
    }
    let mut removed: bool = false;
    for path in unclaimed {
        removed |= remove_file_and_say(path.as_path())?;
    }
    return Ok(removed);
}
