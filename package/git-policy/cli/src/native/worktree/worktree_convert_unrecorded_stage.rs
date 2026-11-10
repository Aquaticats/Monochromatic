//! What:
//!  The empty stage a conversion created and did not record,
//!  and whether the object at the converting name is it.
//! Why:
//!  A conversion creates `<worktree>.cli-git-converting` and writes its identity to the journal right after,
//!  so a kill between those two steps leaves a directory no record names.
//!  That leftover is the conversion's own,
//!  and leaving it there blocks the worktree with a refusal no command can lift;
//!  it is adopted when nothing has been written into it,
//!  and left alone when it holds anything at all.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const adopter = ownsUnrecordedStage(journal, worktree); // creating, no identity, empty directory at the stage name
//! const mine = stageIsThisConversions(journal, worktree); // recorded identity, or the empty unrecorded one
//! ```

use crate::worktree_convert_inspection::converting_path;
use crate::worktree_convert_journal::{Journal, Phase};
use crate::worktree_convert_phases::stage_identity_of;
use crate::subvolume_error::MigrateError;
use std::path::{Path, PathBuf};

/// What:
///  Whether the object at the converting name is the empty stage this conversion made but never recorded,
/// because it was killed between the creation and the line that names it.
/// Why:
///   A leftover of that kind is the conversion's own,
/// and leaving it there blocks the worktree with a refusal no command can
///  lift:
/// the journal says the phase that creates the stage,
/// holds no identity yet,
/// and the name holds
///  nothing,
/// which is the one state that window can leave.
/// Anything that holds even one entry may be somebody's copy and is never adopted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownsUnrecordedStage(journal, worktree): boolean; // creating, no identity, empty directory at the stage name
/// ```
pub fn owns_unrecorded_stage(journal: &Journal, worktree: &Path) -> Result<bool, MigrateError> {
    if journal.new_identity.is_some() || journal.phase != Phase::Creating {
        return Ok(false);
    }
    let converting: PathBuf = converting_path(worktree);
    return match std::fs::read_dir(converting.as_path()) {
        Ok(mut entries) => Ok(entries.next().is_none()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_unreadable) => Ok(false),
    };
}


/// What:
///  Whether the object at the converting name is one this conversion may go on with,
/// by the identity the journal recorded or as the empty stage it made and never recorded.
/// Why:
///   The preflight and the abandon ask one question,
/// and both must answer it the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stageIsThisConversions(journal, worktree): boolean { return journal.newIdentity === stageIdentityOf(worktree) || ownsUnrecordedStage(journal, worktree); }
/// ```
pub fn stage_is_this_conversions(journal: &Journal, worktree: &Path) -> Result<bool, MigrateError> {
    if journal.new_identity.is_some() && stage_identity_of(worktree)? == journal.new_identity {
        return Ok(true);
    }
    return owns_unrecorded_stage(journal, worktree);
}

/// Unrecorded-stage controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_convert_unrecorded_stage_tests.rs"]
mod tests;
