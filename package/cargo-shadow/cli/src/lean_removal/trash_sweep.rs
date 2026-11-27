//! What:
//! The finish of an earlier run's claims:
//! what one pass over the trash directory freed,
//! and the sweep that deletes every entry another run claimed there and did not finish deleting.
//! Why:
//! A run stopped between the move and the delete leaves judged entries in the trash,
//! and the next pass finishes them under the same identity check as any claimed entry.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type TrashSweep = { bytes: number; modesAdded: string[]; failures: string[] };
//! export function emptyTrash(trash: string): TrashSweep
//! ```
/// Imports the deletion a swept entry goes through,
/// and the rule the tool counts bytes by.
use super::deletion_walk::{
    allocated_bytes_of,
    delete_claimed,
};
/// Imports what a finished deletion did.
use super::item_and_deletion_record::Deleted;
/// Imports the refusals a swept entry can answer with.
use super::refusal_text::Refusal;
/// Imports the device and inode each swept entry is judged by.
use crate::lean_identity::{
    Identity,
    identity_of,
};
/// Imports the directory reading the sweep lists the trash with.
use std::fs;
/// Imports the path of the trash directory and the owned path type a widened directory is named by.
use std::path::{
    Path,
    PathBuf,
};
/// What:
/// `pub struct TrashSweep` is what finishing an earlier run's claims did.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrashSweep {
    /// The disk bytes the finished claims freed.
    pub bytes: u64,
    /// The directories whose owner access this sweep widened.
    pub modes_added: Vec<PathBuf>,
    /// One reason for each entry it could not finish removing.
    pub failures: Vec<String>,
}

/// What one finished deletion adds to a sweep.
impl TrashSweep {
    /// Adds what a finished deletion did.
    fn add(
        &mut self,
        deleted: Deleted,
    ) {
        self.bytes += deleted.bytes;
        self.modes_added.extend(deleted.modes_added);
    }
}

/// What:
/// `pub fn empty_trash(trash: &Path) -> TrashSweep` deletes what an earlier run claimed into this trash directory and did not
/// finish deleting (it was stopped between the move and the delete),
/// and reports what it freed and what it could not finish.
/// Why:
/// Everything in a trash directory was judged removable and claimed by a rename;
/// the trash is the tools' own,
/// so finishing the deletion discards nothing that was not already judged.
/// Each entry is deleted under the same identity check as any claimed entry,
/// and by the same walk,
/// so what a stopped run could not get through is finished here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function emptyTrash(trash: string): TrashSweep
/// ```
pub fn empty_trash(trash: &Path) -> TrashSweep {
    let mut sweep: TrashSweep = TrashSweep::default();
    let Ok(entries) = fs::read_dir(trash) else {
        return sweep;
    };
    for entry in entries.flatten() {
        let judged: Identity = match identity_of(&entry.path()) {
            Ok(found) => found,
            Err(error) => {
                sweep
                    .failures
                    .push(format!("{}: {error}", entry.path().display()));
                continue;
            }
        };
        match delete_claimed(&entry.path(), judged, allocated_bytes_of) {
            Ok(deleted) => sweep.add(deleted),
            Err(Refusal::Gone(_)) => {}
            Err(Refusal::PartlyRemoved {
                deleted,
                ..
            }) => {
                let reason: String = Refusal::PartlyRemoved {
                    path: entry.path(),
                    deleted: deleted.clone(),
                }
                .to_string();
                sweep.add(deleted);
                sweep.failures.push(reason);
            }
            Err(refusal) => sweep.failures.push(refusal.to_string()),
        }
    }
    return sweep;
}
