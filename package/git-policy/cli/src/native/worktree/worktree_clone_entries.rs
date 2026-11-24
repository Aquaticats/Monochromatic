//! What:
//!  One run of the clone rung's entries:
//! the entries a run owns,
//! what it created,
//! the stat fields it recorded,
//!  and why it stopped.
//! Why:
//!  A whole rung and one slice of a parallel run clone entries through the same body,
//! so the loop and its answer live
//!  between the per-entry work (`worktree_clone_entry.rs`),
//! the paths it walks (`worktree_clone_paths.rs`) and the fan-out that cuts the slices
//! (`worktree_clone_slices.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { created, stats, failure } = cloneTheEntries({ entries, positions, source, root, destination, seams, failed });
//! ```

use crate::add_timing::{begin_step, end_step};
use crate::fs_beneath::open_root;
use crate::subvolume_index::StatFields;
use crate::worktree_clean::StatEntry;
#[cfg(test)]
use crate::worktree_clean::stored_stats;
use crate::worktree_clone_entry::{EntryResults, OneEntry, Seams, SpentByTheEntries, clone_one_entry};
use crate::worktree_clone_paths::{Created, Directories};
use crate::worktree_path_failure_message::subject_failure;
use crate::worktree_snapshot_source::SnapshotSource;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::sync::atomic::{AtomicBool, Ordering};

/// What one run of entries owns,
/// whichever engine runs it.
pub(crate) struct EntriesToClone<'a, 'run> {
    /// The frozen index entries,
    /// whole.
    pub(crate) entries: &'a [StatEntry],
    /// The entries this run owns,
    /// as indexes into `entries`,
    /// in the order the index lists them.
    pub(crate) positions: &'a [usize],
    /// What the checks proved about the source.
    pub(crate) source: &'a SnapshotSource,
    /// The held root of the source's files.
    pub(crate) source_root: &'a File,
    /// The held root the destinations are opened below.
    pub(crate) destination: &'a File,
    /// The moments a control moves the trees;
    /// one turn in a test supplies them and a slice supplies calls that do nothing.
    pub(crate) seams: &'a mut Seams<'run>,
    /// Raised by the first failure,
    /// so the other slices stop at their next entry.
    pub(crate) failed: &'a AtomicBool,
}

/// What one run of entries made,
/// what it recorded for the index,
/// and why it stopped.
pub(crate) struct ClonedEntries {
    /// The entries created,
    /// in the order the index lists them.
    pub(crate) created: Vec<Created>,
    /// The stat fields of the files made,
    /// by path.
    pub(crate) stats: HashMap<Vec<u8>, StatFields>,
    /// The reason the run stopped,
    /// when it did.
    pub(crate) failure: Option<String>,
}

/// What:
///  Clone the entries at `positions`,
/// in the order they are listed,
/// recording what was created and the stat fields of the files made.
/// Why:
///   A whole rung and one slice of a parallel run do the same work per entry,
/// and a failure is carried out with what
///  the run had made by then,
/// because nothing it created is removed.
/// The caller enters the `cloneEntries` step around the whole run:
/// a slice of a parallel run works on a blocking thread,
/// which holds no recording of the add,
/// so a step entered here would be lost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { created, stats, failure } = cloneTheEntries({ entries, positions, source, root, destination, seams, failed });
/// ```
pub(crate) fn clone_the_entries(work: EntriesToClone<'_, '_>) -> ClonedEntries {
    let EntriesToClone { entries, positions, source, source_root, destination, seams, failed }: EntriesToClone<'_, '_> = work;
    let mut created: Vec<Created> = Vec::new();
    let mut stats: HashMap<Vec<u8>, StatFields> = HashMap::new();
    let mut held_source: Option<(Vec<OsString>, File)> = None;
    let mut spent: SpentByTheEntries = SpentByTheEntries::default();
    let mut directories: Directories = HashMap::new();
    let held_destination: File = match destination.try_clone() {
        Ok(held) => held,
        Err(error) => return ClonedEntries { created, stats, failure: Some(format!("the destination handle cannot be kept ({error})")) },
    };
    directories.insert(Vec::new(), held_destination);
    for at in positions {
        if failed.load(Ordering::Relaxed) {
            break;
        }
        let Some(entry) = entries.get(*at) else { continue };
        let run: OneEntry<'_, '_> = OneEntry { entry, source, source_root, directories: &mut directories, held_source: &mut held_source, seams: &mut *seams };
        let outcome: Result<(), String> = clone_one_entry(run, EntryResults { created: &mut created, stats: &mut stats, spent: &mut spent });
        if let Err(reason) = outcome {
            spent.note();
            return ClonedEntries { created, stats, failure: Some(reason) };
        }
    }
    spent.note();
    return ClonedEntries { created, stats, failure: None };
}

/// What:
///  Clone the given entries of the source on this thread,
/// with the moments at which a control may move the tree.
/// Why:
///   This is the whole run in one turn:
/// the fallback a runtime that cannot start the slices takes,
/// and the shape a repository below the clone's break-even uses;
/// a source with changes
///  hands the entries its proof settled instead of the whole index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { stats } = cloneWithEntries({ source, entries, directories, created, seams }); // throws a reason
/// ```
pub(crate) fn clone_with_entries(source: &SnapshotSource, entries: &[StatEntry], directories: &mut Directories, created: &mut Vec<Created>, seams: &mut Seams<'_>) -> Result<HashMap<Vec<u8>, StatFields>, String> {
    let root: File = subject_failure(open_root(source.root.as_path()), &"the source's root", "cannot be opened")?;
    let Some(destination) = directories.get(&Vec::new()) else {
        return Err(String::from("the destination is not held"));
    };
    let positions: Vec<usize> = (0..entries.len()).collect();
    let unshared: AtomicBool = AtomicBool::new(false);
    let entries_entered: bool = begin_step("cloneEntries");
    let cloned: ClonedEntries = clone_the_entries(EntriesToClone { entries, positions: positions.as_slice(), source, source_root: &root, destination, seams, failed: &unshared });
    end_step(entries_entered);
    let ClonedEntries { created: made, stats, failure }: ClonedEntries = cloned;
    created.extend(made);
    if let Some(reason) = failure {
        return Err(reason);
    }
    (seams.before_index)();
    return Ok(stats);
}

/// What:
///  Clone every entry of the source on this thread,
/// with the moments at which a control may move the tree.
/// Why:
///   This is a control's shape:
/// the source's own index names the entries,
/// and the rung installs that index after the copies.
/// The rung itself never reads the index again:
/// the one-turn arm clones the entry set `clone_in_slices` was handed
/// (`worktree_clone_slices`),
///  so a source with changes clones the paths its proof settled and no staged addition.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { stats } = cloneWith({ source, directories, created, seams }); // throws a reason
/// ```
#[cfg(test)]
pub(crate) fn clone_with(source: &SnapshotSource, directories: &mut Directories, created: &mut Vec<Created>, seams: &mut Seams<'_>) -> Result<HashMap<Vec<u8>, StatFields>, String> {
    let Some(entries) = stored_stats(source.clean.index_bytes.as_slice(), source.clean.hash_length) else {
        return Err(String::from("the source's index changed shape"));
    };
    return clone_with_entries(source, entries.as_slice(), directories, created, seams);
}
