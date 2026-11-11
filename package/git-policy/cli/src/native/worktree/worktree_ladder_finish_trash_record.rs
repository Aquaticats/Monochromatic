//! What:
//!  Write the trash's record now that the worktree holding it is published,
//! and answer what the hand-over still owes,
//!  which is the same trash when the record stands.
//! Why:
//!   The record is what outlives the command,
//! so it is written before the rung that published the worktree returns,
//! and a command killed after
//!  the swap leaves a trash the service empties.
//! A record that does not stand is a trash nothing will ever empty,
//! and the add
//!  pays the emptying itself in that case,
//! which no ordinary add reaches;
//! a trash that then stays is reported with its path
//!  and the reason.
//! The queue knows a trash's record by the trash's name,
//! so an id the queue already holds is this trash's record,
//! and only a
//!  record the service settled as failed leaves nothing to empty the trash.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const recordBeforeTheRungReturns = (pending) => { try { recordTheTrash(pending); return pending; } catch (reason) { warn(reason); deleteTrash(pending); return undefined; } };
//! ```

use crate::diagnostic_log::warn;
use crate::worktree_trash::{PendingTrash, Recorded, TRASH_PARENT, delete_trash, record_the_trash};
use std::path::PathBuf;

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeLadderFinishTrashRecord";

/// What:
///  Write the trash's record now that the worktree holding it is published,
/// and answer what the hand-over still owes,
///  which is the same trash when the record stands.
/// Why:
///   See the module description:
/// the record is what outlives the command,
/// and a record that does not stand is a trash the
///  add empties itself,
/// before it returns,
/// and says so when even that leaves it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recordBeforeTheRungReturns = (pending) => { recordTheTrash(pending); return pending; };
/// ```
pub fn record_before_the_rung_returns(pending: PendingTrash) -> Option<PendingTrash> {
    let shown: PathBuf = pending.destination.join(TRASH_PARENT).join(&pending.manifest.name);
    // A record that is already there is this trash's record, because the trash's name is its id; only a record the service
    //  gave up on leaves nothing to empty the trash, and then this add empties it itself as for a record it could not write.
    let refused: Option<String> = match record_the_trash(&pending) {
        Ok(Recorded::Written | Recorded::There) => None,
        Ok(Recorded::GivenUp) => Some(String::from("the commit service gave up on that record and never claims it again")),
        Err(reason) => Some(format!("the work item could not be written: {reason}")),
    };
    if let Some(reason) = refused {
        warn(TAG, format!("monochromatic-git-policy-cli: no record the service still works on holds the trash {} ({reason}), so this add empties it before it returns", shown.display()).as_str());
        if let Err(left) = delete_trash(pending.destination.as_path(), pending.common_dir.as_path(), &pending.manifest) {
            warn(TAG, format!("monochromatic-git-policy-cli: the trash {} stays: {}", shown.display(), left.reason).as_str());
        }
        return None;
    }
    return Some(pending);
}
