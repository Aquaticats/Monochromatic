//! What:
//!  One entry of the frozen index made in the destination:
//!  a clone of a regular file with the source's mode and times,
//!  or a link with the same target,
//!  compared with the entry's own stat data before and after,
//!  and the run of such entries one thread or one slice owns.
//! Why:
//!  A serial run and a slice of a parallel run must check the same things in the same order,
//!  so an entry's body lives here once,
//!  and the run that walks the index asks this module for each entry.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { created, stats, failure } = cloneTheEntries({ entries, positions, source, root, destination, seams, failed });
//! ```

use crate::add_timing::{begin_work, note_work};
use crate::fs_beneath::{create_file_at, entry_metadata_at, open_file_at, read_link_at, set_times_at, symlink_at};
use crate::fs_clone_exchange::clone_file;
use crate::subvolume_index::StatFields;
use crate::subvolume_index_files::fields_of;
use crate::worktree_clean::{StatEntry, metadata_differs_from_entry};
use crate::worktree_clone_paths::{Created, Directories, ensure_parents, entry_names, identity_of_metadata, source_directory, times_of};
use crate::worktree_path_failure_message::subject_failure;
use crate::worktree_snapshot_source::SnapshotSource;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// What:
///  The moments of a clone at which the controls move the source or the destination tree.
/// Why:
///   The races the held handles and the descriptor comparisons close happen between two system calls,
/// so a control that
///  wants one must run its change there;
/// the release code passes calls that do nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Seams = { afterLookup: () => void; afterClone: () => void; beforeIndex: () => void };
/// ```
pub(crate) struct Seams<'a> {
    /// Called after the entry was looked at and before it is opened.
    pub(crate) after_lookup: &'a mut dyn FnMut(),
    /// Called after the data was cloned and before the source is looked at again.
    pub(crate) after_clone: &'a mut dyn FnMut(),
    /// Called after every entry was made and before the index is installed.
    pub(crate) before_index: &'a mut dyn FnMut(),
}

/// What one clone call works on:
/// the entry,
/// its names and the places the results go.
pub(crate) struct Work<'a> {
    /// The entry of the frozen index.
    pub(crate) entry: &'a StatEntry,
    /// The components of its path.
    pub(crate) names: &'a [OsString],
    /// The whole seconds of the index file's time.
    pub(crate) index_seconds: u32,
}

/// What:
///  The refusal for a source that changed while it was cloned.
/// Why:
///   A clone of a moving source would not equal the source's index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the source changed while it was cloned (${reason})`
/// ```
fn source_changed(reason: String) -> String {
    return format!("the source changed while it was cloned ({reason})");
}

/// What:
///  Create one destination entry from its source below the held directories:
/// a clone of a regular file with the source's
///  mode and times,
/// or a link with the same target.
/// The source's descriptor metadata must equal the frozen index entry
///  before the clone and be unchanged after it;
/// the stat fields recorded for the index are those of the object made here.
/// Why:
///   The bytes cloned are those of the inode that was compared,
/// so a swap of a parent directory cannot certify other
///  bytes as the index's,
/// and a destination file replaced afterwards no longer has the inode the index entry records.
/// An
///  entry of any other type is not a tracked file Git checks out this way,
/// so the rung ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await cloneEntry({ sourceDir, destinationDir, work, created, stats });
/// ```
pub(crate) fn clone_entry(source_dir: &File, destination_dir: &File, work: &Work<'_>, results: (&mut Vec<Created>, &mut HashMap<Vec<u8>, StatFields>), seams: &mut Seams<'_>) -> Result<(), String> {
    let (created, stats): (&mut Vec<Created>, &mut HashMap<Vec<u8>, StatFields>) = results;
    let Work { entry, names: all_names, index_seconds }: &Work<'_> = work;
    let name: &OsStr = all_names[all_names.len() - 1].as_os_str();
    let shown: String = String::from_utf8_lossy(entry.path.as_slice()).into_owned();
    let metadata: std::fs::Metadata = subject_failure(entry_metadata_at(source_dir, name), &shown, "is unreadable")?;
    if metadata.file_type().is_symlink() {
        if let Some(reason) = metadata_differs_from_entry(entry, &metadata, *index_seconds) {
            return Err(source_changed(reason));
        }
        let target: PathBuf = subject_failure(read_link_at(source_dir, name), &shown, "is unreadable")?;
        subject_failure(symlink_at(target.as_path(), destination_dir, name), &shown, "cannot be linked")?;
        created.push(Created { names: all_names.to_vec(), directory: false });
        if let Err(error) = set_times_at(destination_dir, name, times_of(&metadata)) {
            return Err(format!("the times of {shown} cannot be set ({error})"));
        }
        let after: std::fs::Metadata = subject_failure(entry_metadata_at(source_dir, name), &shown, "is unreadable")?;
        if identity_of_metadata(&after) != identity_of_metadata(&metadata) {
            return Err(source_changed(format!("{shown} was replaced")));
        }
        // A link cannot be held by a descriptor that reads its metadata, so the name is not asked: the entry keeps the
        // source's stat fields, which no longer match the new link, and Git compares the link's target on first use.
        return Ok(());
    }
    if !metadata.is_file() {
        return Err(format!("{shown} is neither a file nor a link"));
    }
    (seams.after_lookup)();
    let source: File = subject_failure(open_file_at(source_dir, name), &shown, "is unreadable")?;
    let held: std::fs::Metadata = subject_failure(source.metadata(), &shown, "is unreadable")?;
    if let Some(reason) = metadata_differs_from_entry(entry, &held, *index_seconds) {
        return Err(source_changed(reason));
    }
    let destination: File = subject_failure(create_file_at(destination_dir, name, 0o600), &shown, "cannot be created")?;
    created.push(Created { names: all_names.to_vec(), directory: false });
    if held.len() > 0 {
        subject_failure(clone_file(&source, &destination, Path::new(name)), &"the clone", "was refused")?;
    }
    (seams.after_clone)();
    let after: std::fs::Metadata = subject_failure(source.metadata(), &shown, "is unreadable")?;
    if identity_of_metadata(&after) != identity_of_metadata(&held) {
        return Err(source_changed(format!("{shown} was written during the clone")));
    }
    if let Err(error) = destination.set_permissions(std::fs::Permissions::from_mode(held.mode() & 0o7777)) {
        return Err(format!("the mode of {shown} cannot be set ({error})"));
    }
    let times: [libc::timespec; 2] = times_of(&held);
    // SAFETY: `destination` is an open descriptor and `times` holds the two timestamps the call reads.
    let result: libc::c_int = unsafe { libc::futimens(destination.as_raw_fd(), times.as_ptr()) };
    if result != 0 {
        return Err(format!("the times of {shown} cannot be set ({})", std::io::Error::last_os_error()));
    }
    // The fields of the object made here, taken from its descriptor after its last change.
    let made: std::fs::Metadata = subject_failure(destination.metadata(), &shown, "cannot be examined")?;
    stats.insert(entry.path.clone(), fields_of(&made));
    return Ok(());
}

/// What one entry's clone needs from the run around it.
pub(crate) struct OneEntry<'a, 'run> {
    /// The entry of the frozen index.
    entry: &'a StatEntry,
    /// What the checks proved about the source.
    source: &'a SnapshotSource,
    /// The held root of the source's files.
    source_root: &'a File,
    /// The destination directories held so far.
    directories: &'a mut Directories,
    /// The source directory held for the entries before this one.
    held_source: &'a mut Option<(Vec<OsString>, File)>,
    /// The moments a control moves the trees.
    seams: &'a mut Seams<'run>,
}

/// What one entry's clone records.
pub(crate) struct EntryResults<'a> {
    /// The entries created,
    /// in creation order.
    created: &'a mut Vec<Created>,
    /// The stat fields of the files made,
    /// by path.
    stats: &'a mut HashMap<Vec<u8>, StatFields>,
    /// What the entries of this run spent in each part of an entry,
    /// summed here so the ledger is written once per run.
    spent: &'a mut SpentByTheEntries,
}

/// What the entries of one run spent,
/// by the part of an entry that spent it.
#[derive(Default)]
pub(crate) struct SpentByTheEntries {
    /// Opening or reusing the source directory of an entry.
    source_dirs: Duration,
    /// Making and finding the destination directories of an entry.
    parents: Duration,
    /// Everything else of an entry:
    /// its look,
    /// its clone,
    /// its mode,
    /// its times and its index fields.
    files: Duration,
}

impl SpentByTheEntries {
    /// Note the run's sums in the add timing record,
    /// one entry per name.
    fn note(&self) {
        note_work("cloneSourceDirs", self.source_dirs);
        note_work("cloneDestinationDirs", self.parents);
        note_work("cloneFiles", self.files);
    }
}

/// What:
///  Clone one entry of the frozen index into the destination,
/// recording what was created and the stat fields of the file made.
/// Why:
///   One body clones an entry whichever engine owns it,
/// so the serial rung and a slice of a parallel run check the
///  same things in the same order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cloneEntryAt({ entry, source, root, directories, heldSource, seams }, { created, stats }); // throws
/// ```
pub(crate) fn clone_one_entry(work: OneEntry<'_, '_>, results: EntryResults<'_>) -> Result<(), String> {
    let OneEntry { entry, source, source_root, directories, held_source, seams }: OneEntry<'_, '_> = work;
    let names: Vec<OsString> = entry_names(entry.path.as_slice())?;
    let source_entered: Instant = begin_work();
    let source_dir: &File = source_directory(held_source, source_root, &names[..names.len() - 1], entry.path.as_slice())?;
    results.spent.source_dirs += source_entered.elapsed();
    let parents_entered: Instant = begin_work();
    let destination_dir: &File = ensure_parents(directories, names.as_slice(), results.created)?;
    results.spent.parents += parents_entered.elapsed();
    let job: Work<'_> = Work { entry, names: names.as_slice(), index_seconds: source.clean.index_seconds };
    let files_entered: Instant = begin_work();
    let outcome: Result<(), String> = clone_entry(source_dir, destination_dir, &job, (results.created, results.stats), seams);
    results.spent.files += files_entered.elapsed();
    return outcome;
}

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
