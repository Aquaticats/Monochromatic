//! What:
//!  Whether the copy an interrupted add's record still owes has a future in a worktree other than the one the command
//! runs in.
//! Why:
//!  The reaping of stage claims asks this before it settles an add's records,
//! and the answer decides whether this command settles an obligation that belongs to another worktree or leaves it
//!  whole and names it.
//! It lives beside the reaping rather than in it because that file was over the line limit
//! (`worktree_copy_stage_claim_reaping.rs`,
//!  which calls it).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const owedElsewhere = owedCopyToAnotherWorktree({ commonDir, claim, record, destinations });
//! ```

use crate::worktree_copy_interrupted_add_settlement::{add_is_finished, registration_directory};
use crate::worktree_copy_recovery::{RecoveryDestinations, finishes_destination};
use std::path::{Path, PathBuf};

/// What:
///  Whether an interrupted add's record still owes a copy into a worktree other than the one the command runs
/// in,
/// with a checkout whole enough for the installation.
/// Why:
///  Such a copy is that worktree's own work and has a future there,
/// so this command must neither settle the obligation nor drop
///  the claim that names it;
/// the pending check writes one line naming the worktree
///  instead,
/// and the first command run there makes the copy or names why it cannot.
/// A destination that is not a worktree of this repository,
/// or one whose
///  checkout is not whole,
/// has no such future and settles as it always did,
/// which is what names a half-made
///  worktree to the person.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owedElsewhere = (record, destinations) => { const text = read(record); const destination = destinationOfRecord(text); return destination !== undefined && !finishesDestinations(destinations, destination) && owedCopyOf(text) !== undefined && addIsFinished(destination); };
/// ```
pub(crate) fn copy_owed_to_another_worktree(
    common_dir: &Path,
    claim: &Path,
    record: &Path,
    destinations: &RecoveryDestinations<'_>,
) -> bool {
    let Ok(text) = std::fs::read_to_string(record) else {
        return false;
    };
    let destination: PathBuf = match crate::worktree_add_binding::decode_record(text.as_str()) {
        Some(crate::worktree_add_binding::Recorded::Planned(named)) => named,
        Some(crate::worktree_add_binding::Recorded::Registered(binding)) => binding.destination,
        None => return false,
    };
    if finishes_destination(destinations, destination.as_path()) {
        return false;
    }
    // A record written before the add named its source and selection owes no copy, so nothing may be filled from it.
    if crate::worktree_add_binding::owed_copy_of(text.as_str()).is_none() {
        return false;
    }
    // A destination holding the object the add's own snapshot became received the ignored state with it,
    // so no copy is owed anywhere: the records settle here, and no command says the worktree is missing files it holds.
    if crate::worktree_copy_stage_made_object::the_made_object_stands_at(claim, destination.as_path()) {
        return false;
    }
    // The copy has a future only where the registration and the checkout are whole under this repository.
    if crate::worktree_copy_recovery::destination_registration(common_dir, destination.as_path())
        != crate::worktree_copy_recovery::Registration::Registered
    {
        return false;
    }
    let Some(admin) = registration_directory(destination.as_path()) else {
        return false;
    };
    return add_is_finished(destination.as_path(), admin.as_path());
}
