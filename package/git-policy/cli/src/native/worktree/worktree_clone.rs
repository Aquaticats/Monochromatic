//! What:
//!  Rung 2 of the cheap-worktree ladder:
//! each tracked file of the clean source is cloned (`FICLONE`) into the new worktree
//!  with the source's mode and times,
//! and the source's index is installed with fresh stat fields.
//! Why:
//!  A clone shares every data block with the source on btrfs,
//! on XFS with reflink and inside an overlay,
//! which the
//!  snapshot rung cannot reach (overlayfs and XFS have reflink but no subvolumes).
//! It takes no less time than Git's checkout
//!  on btrfs (`doc/planning/cli-git-btrfs-worktree.md`,
//! option B),
//! so it saves disk,
//! not time,
//! and it is the accelerated
//!  rung where a snapshot is impossible.
//! The wrapper's own index writer is `rewrite_index_stat_with`:
//! with each file's
//!  size and modification time kept,
//! only the inode,
//! change time and device of every entry change.
//! The first failure ends
//!  the rung with its reason,
//! the entries this run created stay (a name cannot be checked and unlinked in one step),
//! and
//!  Git's own checkout completes the worktree and overwrites the tracked ones.
//!  Every source file is opened below a held handle of the source root without following a link,
//! and its descriptor's
//!  metadata,
//! not a path,
//! is compared with the frozen index entry before and after the clone;
//! every destination entry is
//!  created below a held handle of the destination.
//! A directory swapped for a link while the run goes on is an error,
//!  never a detour.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await cloneTrackedFiles(source, destination, admin); // throws a reason
//! ```

use crate::add_timing::{begin_step, begin_work, end_step, note, note_work};
use crate::diagnostic_log::debug;
use crate::fs_beneath::{create_file_at, entry_metadata_at, make_directory_at, open_directory_at, open_file_at, open_root, plain_components, read_link_at, set_times_at, symlink_at};
use crate::fs_beneath_walk_chain::open_directory_beneath;
use crate::fs_clone_exchange::clone_file;
use crate::parallel_budget::{HostResources, PARALLEL_BREAK_EVEN_ENTRIES, sliced_workers};
use crate::side_by_side_on_the_runtime::run_all_side_by_side_on_the_runtime;
use crate::subvolume_index::{StatFields, StatLookup};
use crate::subvolume_index_files::{fields_of, install_index_in};
use crate::worktree_copy_stage_leaf_copying::PARALLEL_COPY_MAX_WORKERS;
use crate::worktree_path_failure_message::subject_failure;
use crate::worktree_clean::{StatEntry, metadata_differs_from_entry, stored_stats};
use crate::worktree_snapshot_source::SnapshotSource;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Tag used for diagnostics from this module.
const TAG: &str = "cloneTrackedFiles";

/// What:
///  The refusal for an index path that is not a plain path inside the worktree.
/// Why:
///   A path with a link,
/// a `..` or a `.git` component could reach outside the tracked files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the index path ${path} is not a plain path`
/// ```
fn index_path_refusal(path: &[u8]) -> String {
    return format!("the index path {} is not a plain path", String::from_utf8_lossy(path));
}

/// What:
///  Whether any component of a path is `.git`.
/// Why:
///   The clones never write into a Git directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// names.includes('.git')
/// ```
fn names_dot_git(names: &[OsString]) -> bool {
    for name in names {
        if name == ".git" {
            return true;
        }
    }
    return false;
}

/// What:
///  The components of a path joined with slashes,
/// as text.
/// Why:
///   The report names a few of the entries the clones created.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// names.join('/')
/// ```
fn joined_names(names: &[OsString]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for name in names {
        parts.push(name.to_string_lossy().into_owned());
    }
    return parts.join("/");
}

/// What:
///  The names of a tracked entry's path below a root,
/// refused unless every component is a plain name and none is `.git`.
/// Why:
///   The index is the source's own,
/// but a path that climbs out of the destination or enters `.git` is never created.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryNames(path: Uint8Array): string[]; // throws
/// ```
fn entry_names(path: &[u8]) -> Result<Vec<OsString>, String> {
    let names: Vec<OsString> = match plain_components(Path::new(OsStr::from_bytes(path))) {
        Ok(found) => found,
        Err(_not_plain) => return Err(index_path_refusal(path)),
    };
    return if names.is_empty() || names_dot_git(names.as_slice()) { Err(index_path_refusal(path)) } else { Ok(names) };
}

/// What:
///  The access and modification times of metadata,
/// to the nanosecond.
/// Why:
///   The index entry keeps the source's modification time,
/// and Git trusts a file whose size and time equal it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const times = [atime, mtime];
/// ```
fn times_of(from: &std::fs::Metadata) -> [libc::timespec; 2] {
    return [
        libc::timespec { tv_sec: from.atime() as libc::time_t, tv_nsec: from.atime_nsec() as libc::c_long },
        libc::timespec { tv_sec: from.mtime() as libc::time_t, tv_nsec: from.mtime_nsec() as libc::c_long },
    ];
}

/// What:
///  The identity a clone must find again after it:
/// device,
/// inode,
/// size,
/// and both times.
/// Why:
///   A source that is written while it is cloned changes its size or times;
/// a swapped inode changes the inode.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Identity = [dev, ino, size, mtime, ctime];
/// ```
fn identity_of_metadata(from: &std::fs::Metadata) -> [i64; 8] {
    return [from.dev() as i64, from.ino() as i64, from.size() as i64, from.mtime(), from.mtime_nsec(), from.ctime(), from.ctime_nsec(), i64::from(from.mode())];
}

/// What:
///  An entry this run created,
/// named by its components below the destination.
/// Why:
///   Nothing is removed on a failure:
/// a name cannot be checked and unlinked in one step,
/// and the entry may by then be another
///  process's,
/// so the run keeps what it made and reports it.
/// The list is what the report names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Created = { names: string[]; directory: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Created {
    /// The components below the destination.
    pub names: Vec<OsString>,
    /// Whether the entry is a directory.
    pub directory: bool,
}

/// The directories this entry's path was walked through,
/// by their components below the destination (the empty list is the destination),
/// each
///  held open so a later file is placed through the directory it was entered by,
/// whatever its name became.
/// Only the levels of the entry being cloned are held;
/// the index is sorted by path,
/// so no directory the walk left is entered again.
type Directories = HashMap<Vec<OsString>, File>;

/// What:
///  What a successful clone run holds:
/// the entries it created.
/// Why:
///   A later step that fails (the tree difference) reports them;
/// Git's own checkout then overwrites the tracked ones,
/// and the
///  rest stay as untracked files for the person to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Cloned = { created: Created[] };
/// ```
#[derive(Debug)]
pub struct Cloned {
    /// The entries created,
    /// in creation order.
    created: Vec<Created>,
}

impl Cloned {
    /// The entries the run created,
    /// in creation order.
    pub fn created(&self) -> &[Created] {
        return self.created.as_slice();
    }
}

/// What:
///  The report of entries a clone run created and leaves in place.
/// Why:
///   The tracked ones are overwritten by the checkout that completes the worktree;
/// any other stays as an untracked file,
/// and
///  the person is told where to look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = leftover_report(created, destination);
/// ```
pub fn leftover_report(created: &[Created], destination: &Path) -> String {
    // The first three entries that are not directories, as `filter` and `take` chose them.
    let mut shown: Vec<String> = Vec::new();
    for entry in created {
        if entry.directory {
            continue;
        }
        if shown.len() == 3 {
            break;
        }
        shown.push(joined_names(entry.names.as_slice()));
    }
    return format!("{} entries the clones created stay in {} (for example {}); the checkout that completes the worktree overwrites the tracked ones, and `git -C {} status` lists any other as untracked", created.len(), destination.display(), shown.join(", "), destination.display());
}

/// What:
///  Create the directories of `names[..len - 1]` below the destination and return a handle on the last one,
/// recording the ones made
///  and holding the directories on that entry's path.
/// Why:
///   Each component is opened without following a link or crossing a mount,
/// so the file goes into a directory this run
///  made or found,
/// on the destination's device.
/// The handles of the directories on the path are kept and reused,
/// so an entry pays a lookup for each level
///  above it and no duplicated descriptor,
/// and the handles the walk left are dropped,
/// so a run holds one
///  descriptor per level and never one per directory of the tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parent = ensureParents(directories, names, created);
/// ```
fn ensure_parents<'a>(directories: &'a mut Directories, names: &[OsString], created: &mut Vec<Created>) -> Result<&'a File, String> {
    let parent: &[OsString] = &names[..names.len() - 1];
    if !directories.contains_key(parent) {
        let mut key: Vec<OsString> = Vec::new();
        for index in 0..parent.len() {
            key.push(parent[index].clone());
            if directories.contains_key(key.as_slice()) {
                continue;
            }
            let held: &File = match directories.get(key[..index].as_ref()) {
                Some(found) => found,
                None => return Err(String::from("the destination is not held")),
            };
            let name: &OsString = &parent[index];
            let made: bool = subject_failure(make_directory_at(held, name.as_os_str(), 0o777), &name.to_string_lossy(), "cannot be created")?;
            let next: File = subject_failure(open_directory_at(held, name.as_os_str()), &name.to_string_lossy(), "cannot be entered")?;
            if crate::fs_beneath::device_of(&next).ok() != crate::fs_beneath::device_of(held).ok() {
                return Err(format!("{} is on another device", name.to_string_lossy()));
            }
            if made {
                created.push(Created { names: key.clone(), directory: true });
            }
            directories.insert(key.clone(), next);
        }
        // The index is sorted by path, so a directory this entry left is never needed again;
        // dropping it keeps one descriptor per level of one entry, whichever engine holds the map.
        directories.retain(|held: &Vec<OsString>, _file: &mut File| { return parent.starts_with(held.as_slice()); });
    }
    return match directories.get(parent) {
        Some(found) => Ok(found),
        None => Err(String::from("the destination is not held")),
    };
}

/// What:
///  The source directory holding one entry,
/// opened below the held root or reused from the entry before it.
/// Why:
///   Every source file is opened below a handle of the source root without following a link;
/// consecutive entries of the
///  sorted index share their directory,
/// so the walk is made once per directory and not once per file,
/// and a directory
///  renamed away during the run is read through the handle taken for it,
/// which the rung's own controls state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sourceDirectory = (held, root, names) => held?.[0].join('#') === names.join('#') ? held[1] : (held = [names, openDirectoryBeneath(root, names)])[1];
/// ```
fn source_directory<'a>(held: &'a mut Option<(Vec<OsString>, File)>, root: &File, names: &[OsString], shown: &[u8]) -> Result<&'a File, String> {
    let shared: bool = matches!(held.as_ref(), Some((key, _file)) if key.as_slice() == names);
    if !shared {
        let opened: File = match open_directory_beneath(root, names) {
            Ok(found) => found,
            Err(error) => return Err(format!("the source directory of {} cannot be opened ({error})", String::from_utf8_lossy(shown))),
        };
        *held = Some((names.to_vec(), opened));
    }
    return match held.as_ref() {
        Some((_key, found)) => Ok(found),
        None => Err(String::from("the source directory was not opened")),
    };
}

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
struct Seams<'a> {
    /// Called after the entry was looked at and before it is opened.
    after_lookup: &'a mut dyn FnMut(),
    /// Called after the data was cloned and before the source is looked at again.
    after_clone: &'a mut dyn FnMut(),
    /// Called after every entry was made and before the index is installed.
    before_index: &'a mut dyn FnMut(),
}

/// What one clone call works on:
/// the entry,
/// its names and the places the results go.
struct Work<'a> {
    /// The entry of the frozen index.
    entry: &'a StatEntry,
    /// The components of its path.
    names: &'a [OsString],
    /// The whole seconds of the index file's time.
    index_seconds: u32,
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

fn clone_entry(source_dir: &File, destination_dir: &File, work: &Work<'_>, results: (&mut Vec<Created>, &mut HashMap<Vec<u8>, StatFields>), seams: &mut Seams<'_>) -> Result<(), String> {
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
struct OneEntry<'a, 'run> {
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
struct EntryResults<'a> {
    /// The entries created, in creation order.
    created: &'a mut Vec<Created>,
    /// The stat fields of the files made, by path.
    stats: &'a mut HashMap<Vec<u8>, StatFields>,
    /// What the entries of this run spent in each part of an entry, summed here so the ledger is written once per run.
    spent: &'a mut SpentByTheEntries,
}

/// What the entries of one run spent, by the part of an entry that spent it.
#[derive(Default)]
struct SpentByTheEntries {
    /// Opening or reusing the source directory of an entry.
    source_dirs: Duration,
    /// Making and finding the destination directories of an entry.
    parents: Duration,
    /// Everything else of an entry: its look, its clone, its mode, its times and its index fields.
    files: Duration,
}

impl SpentByTheEntries {
    /// Note the run's sums in the add timing record, one entry per name.
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
fn clone_one_entry(work: OneEntry<'_, '_>, results: EntryResults<'_>) -> Result<(), String> {
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

/// What one run of entries owns, whichever engine runs it.
struct EntriesToClone<'a, 'run> {
    /// The frozen index entries, whole.
    entries: &'a [StatEntry],
    /// The entries this run owns, as indexes into `entries`,
    /// in the order the index lists them.
    positions: &'a [usize],
    /// What the checks proved about the source.
    source: &'a SnapshotSource,
    /// The held root of the source's files.
    source_root: &'a File,
    /// The held root the destinations are opened below.
    destination: &'a File,
    /// The moments a control moves the trees;
    /// one turn in a test supplies them and a slice supplies calls that do nothing.
    seams: &'a mut Seams<'run>,
    /// Raised by the first failure, so the other slices stop at their next entry.
    failed: &'a AtomicBool,
}

/// What one run of entries made, what it recorded for the index, and why it stopped.
struct ClonedEntries {
    /// The entries created, in the order the index lists them.
    created: Vec<Created>,
    /// The stat fields of the files made, by path.
    stats: HashMap<Vec<u8>, StatFields>,
    /// The reason the run stopped, when it did.
    failure: Option<String>,
}

/// What:
///  Clone the entries at `positions`, in the order they are listed,
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
fn clone_the_entries(work: EntriesToClone<'_, '_>) -> ClonedEntries {
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
///  The stat fields the clones recorded,
/// looked up by path for the index the new worktree takes.
/// Why:
///   Each clone recorded the fields of the file it made,
/// and the index install asks for them by the entry's path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class StatsMadeByTheClones implements StatLookup { statNow(path) { return this.stats.get(path); } }
/// ```
struct StatsMadeByTheClones<'a> {
    /// The fields by path.
    stats: &'a HashMap<Vec<u8>, StatFields>,
}

impl StatLookup for StatsMadeByTheClones<'_> {
    fn stat_now(&mut self, path: &[u8]) -> Option<StatFields> {
        return self.stats.get(path).copied();
    }
}

/// `clone_all` in one turn, with the moments at which a control may move the tree.
fn clone_with(source: &SnapshotSource, directories: &mut Directories, admin: &File, created: &mut Vec<Created>, seams: &mut Seams<'_>) -> Result<(), String> {
    let Some(entries) = stored_stats(source.clean.index_bytes.as_slice(), source.clean.hash_length) else {
        return Err(String::from("the source's index changed shape"));
    };
    let root: File = subject_failure(open_root(source.root.as_path()), &"the source's root", "cannot be opened")?;
    let Some(destination) = directories.get(&Vec::new()) else {
        return Err(String::from("the destination is not held"));
    };
    let positions: Vec<usize> = (0..entries.len()).collect();
    let unshared: AtomicBool = AtomicBool::new(false);
    let entries_entered: bool = begin_step("cloneEntries");
    let cloned: ClonedEntries = clone_the_entries(EntriesToClone { entries: entries.as_slice(), positions: positions.as_slice(), source, source_root: &root, destination, seams, failed: &unshared });
    end_step(entries_entered);
    let ClonedEntries { created: made, stats, failure }: ClonedEntries = cloned;
    created.extend(made);
    if let Some(reason) = failure {
        return Err(reason);
    }
    (seams.before_index)();
    let index_entered: bool = begin_step("cloneIndex");
    let installed: Result<String, String> = install_index_in(admin, source.clean.index_bytes.as_slice(), &mut StatsMadeByTheClones { stats: &stats }, source.clean.hash_length);
    end_step(index_entered);
    let report: String = installed?;
    debug_report(report.as_str());
    return Ok(());
}

/// What one slice of the clones works on.
#[derive(Clone)]
struct SliceOfClones {
    /// The frozen entries, whole.
    entries: Arc<[StatEntry]>,
    /// The entries this slice owns, as indexes into `entries`.
    positions: Vec<usize>,
    /// What the checks proved about the source.
    source: Arc<SnapshotSource>,
    /// The held root of the source's files.
    source_root: Arc<File>,
    /// The held root the destinations are opened below.
    destination: Arc<File>,
    /// Raised by the first failure, so the other slices stop at their next entry.
    failed: Arc<AtomicBool>,
}

/// What:
///  Clone one slice as a job of the blocking pool.
/// Why:
///   A slice owns its entries and its directory handles,
/// so its answer is its own and the flag is all it shares
///  with the others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cloneASlice(state) { const cloned = cloneTheEntries(state); if (cloned.failure) state.failed.store(true); return cloned; }
/// ```
fn clone_a_slice(state: SliceOfClones) -> ClonedEntries {
    let mut seams: Seams<'_> = Seams { after_lookup: &mut do_nothing, after_clone: &mut do_nothing, before_index: &mut do_nothing };
    let cloned: ClonedEntries = clone_the_entries(EntriesToClone { entries: state.entries.as_ref(), positions: state.positions.as_slice(), source: state.source.as_ref(), source_root: state.source_root.as_ref(), destination: state.destination.as_ref(), seams: &mut seams, failed: state.failed.as_ref() });
    if cloned.failure.is_some() {
        state.failed.store(true, Ordering::Relaxed);
    }
    return cloned;
}

/// What:
///  `clone_a_slice` for every slice on the process's runtime, in slice order.
/// Why:
///   A job that panicked or that the pool refused answered nothing about its entries,
/// and the caller must hear that
///  as a failure like any other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answers = await Promise.allSettled(states.map(cloneASlice));
/// ```
async fn clone_the_slices(states: Vec<SliceOfClones>) -> Vec<Result<ClonedEntries, String>> {
    let mut answers: Vec<Result<ClonedEntries, String>> = Vec::new();
    for answer in run_all_side_by_side_on_the_runtime(states, clone_a_slice).await {
        answers.push(match answer {
            Ok(cloned) => Ok(cloned),
            Err(crate::blocking_job::BlockingJobError::Panicked) => Err(String::from("a clone worker panicked")),
            Err(refused) => Err(format!("a clone worker did not run ({refused})")),
        });
    }
    return answers;
}

/// What:
///  How many threads the clones of this many entries use:
/// one below the break-even the copy stage and `clone_tree` were measured at,
/// else the host's
///  budget under the ceiling the filesystem was measured to need.
/// Why:
///   Making files by cloning is the copy stage's own load on the filesystem's metadata locks,
/// so the ceiling
///  measured for that stage is the ceiling here.
/// Both limits are borrowed:
/// they were measured on btrfs reflinked trees of this repository,
/// for copies of whole
///  files, the install and `clone_tree`,
/// and never for this rung,
/// not on XFS, overlayfs or ZFS,
/// and not between the break-even this rung uses and the entry counts those
///  measurements took,
/// which is where a small repository sits;
/// the borrow is stated here,
/// and
///  measuring this rung at a few hundred entries is owed (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
/// step 3's open items).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cloneWorkerCount = (entries) => entries < 16 ? 1 : slicedWorkers(entries, 8, host);
/// ```
fn clone_worker_count(entries: usize) -> usize {
    if entries < PARALLEL_BREAK_EVEN_ENTRIES {
        return 1;
    }
    return sliced_workers(entries, PARALLEL_COPY_MAX_WORKERS, &HostResources::read());
}

/// What:
///  Clone the frozen entries in contiguous slices,
/// one job of the blocking pool each,
/// and install the index with the stat
///  fields of every slice.
/// Why:
///   Each entry is a few metadata calls and one `FICLONE`,
/// which the filesystem serializes at its own metadata anyway;
///  contiguous slices keep neighbouring entries in one slice,
/// so neighbouring files share a directory the way they do on one thread,
///  and the answers are read in slice order,
/// so the entries stay in the index's order.
/// The slices clone in one turn when no runtime can be entered here,
/// which is the same work on one thread.
/// The step around the fan-out is entered on this thread,
/// so the record holds the wait for the slices,
/// and a note names the entries, the slices and the workers the wait was split across,
/// because neither is visible in a record that only sums the work per slice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cloneInSlices({ source, entries, destination, admin, created, workers }); // throws a reason
/// ```
fn clone_in_slices(source: &SnapshotSource, entries: Arc<[StatEntry]>, destination: &File, admin: &File, created: &mut Vec<Created>, workers: usize) -> Result<(), String> {
    let source_root: Arc<File> = Arc::new(subject_failure(open_root(source.root.as_path()), &"the source's root", "cannot be opened")?);
    let destination_held: Arc<File> = Arc::new(subject_failure(destination.try_clone(), &"the destination handle", "cannot be kept")?);
    let shared_source: Arc<SnapshotSource> = Arc::new(source.clone());
    let failed: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let slice_length: usize = entries.len().div_ceil(workers).max(1);
    let mut states: Vec<SliceOfClones> = Vec::new();
    let mut at: usize = 0;
    for chunk in entries.chunks(slice_length) {
        states.push(SliceOfClones { entries: Arc::clone(&entries), positions: (at..at + chunk.len()).collect(), source: Arc::clone(&shared_source), source_root: Arc::clone(&source_root), destination: Arc::clone(&destination_held), failed: Arc::clone(&failed) });
        at += chunk.len();
    }
    let slices: usize = states.len();
    note("cloneSlices", format!("{} entries in {} slices, up to {} workers", entries.len(), slices, workers).as_str());
    let entries_entered: bool = begin_step("cloneEntries");
    let ran = crate::lazy_runtime::run_on_runtime(clone_the_slices(states));
    end_step(entries_entered);
    let answers: Vec<Result<ClonedEntries, String>> = match ran {
        Ok(answers) => answers,
        Err(refusal) => {
            debug(TAG, format!("the clones run in one turn ({refusal})").as_str());
            let mut directories: Directories = HashMap::new();
            directories.insert(Vec::new(), subject_failure(destination.try_clone(), &"the destination handle", "cannot be kept")?);
            let mut seams: Seams<'_> = Seams { after_lookup: &mut do_nothing, after_clone: &mut do_nothing, before_index: &mut do_nothing };
            return clone_with(source, &mut directories, admin, created, &mut seams);
        }
    };
    let mut stats: HashMap<Vec<u8>, StatFields> = HashMap::new();
    let mut failure: Option<String> = None;
    for answer in answers {
        match answer {
            Ok(cloned) => {
                created.extend(cloned.created);
                stats.extend(cloned.stats);
                if failure.is_none() {
                    failure = cloned.failure;
                }
            }
            Err(reason) => {
                if failure.is_none() {
                    failure = Some(reason);
                }
            }
        }
    }
    if let Some(reason) = failure {
        return Err(reason);
    }
    let index_entered: bool = begin_step("cloneIndex");
    let installed: Result<String, String> = install_index_in(admin, source.clean.index_bytes.as_slice(), &mut StatsMadeByTheClones { stats: &stats }, source.clean.hash_length);
    end_step(index_entered);
    let report: String = installed?;
    debug_report(report.as_str());
    return Ok(());
}

/// The run itself:
/// every tracked file in the slices the host's budget allows,
/// then the index;
/// the caller removes `created` when this fails.
fn clone_all(source: &SnapshotSource, destination: &File, admin: &File, created: &mut Vec<Created>) -> Result<(), String> {
    let Some(entries) = stored_stats(source.clean.index_bytes.as_slice(), source.clean.hash_length) else {
        return Err(String::from("the source's index changed shape"));
    };
    let workers: usize = clone_worker_count(entries.len());
    if workers > 1 {
        return clone_in_slices(source, Arc::from(entries), destination, admin, created, workers);
    }
    let mut directories: Directories = HashMap::new();
    directories.insert(Vec::new(), subject_failure(destination.try_clone(), &"the destination handle", "cannot be kept")?);
    let mut seams: Seams<'_> = Seams { after_lookup: &mut do_nothing, after_clone: &mut do_nothing, before_index: &mut do_nothing };
    return clone_with(source, &mut directories, admin, created, &mut seams);
}

/// Log what the index install did.
/// Do nothing;
/// the seams of the clones run this unless a control puts work there.
fn do_nothing() {}

fn debug_report(report: &str) {
    debug(TAG, report);
}

/// What:
///  Clone every tracked file of the frozen index into the held `destination`,
/// and install that index in the held `admin`
///  with the stat fields of the regular files made.
/// On an error nothing is removed:
/// the entries this run created stay and
///  are counted in the reason.
/// Why:
///   The index bytes are the ones proven clean,
/// so what is installed is what was proven;
/// the destination may hold files
///  of other writers,
/// and they are not this run's to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cloned = await cloneTrackedFiles(source, destination, admin); // throws a reason
/// ```
pub fn clone_tracked_files(source: &SnapshotSource, destination: &File, admin: &File) -> Result<Cloned, String> {
    let mut created: Vec<Created> = Vec::new();
    if let Err(reason) = clone_all(source, destination, admin, &mut created) {
        return Err(if created.is_empty() { reason } else { format!("{reason}; {} entries it created stay", created.len()) });
    }
    return Ok(Cloned { created });
}

/// Clone rung controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_clone_tests.rs"]
mod tests;
