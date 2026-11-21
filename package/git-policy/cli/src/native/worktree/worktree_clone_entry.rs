//! What:
//!  One tracked entry of the clone rung:
//! its path's components,
//! the source directory it is read from,
//! the destination folders it is written through,
//!  and the clone of the file or link itself,
//! with the checks that prove the source did not move under the copy.
//! Why:
//!  The rung's whole safety argument is per entry,
//! and `worktree_clone.rs` was over the line limit with it in the same file:
//!  every source file is opened below a held handle of the source root without following a link,
//!  its descriptor's metadata is compared with the frozen index entry before and after the clone,
//!  and every destination entry is created below a held handle of the destination.
//! A directory swapped for a link while the run goes on is an error,
//! never a detour.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! cloneOneEntry({ entry, source, sourceRoot, directories, heldSource, seams }, { created, stats }); // throws a reason
//! ```

use crate::add_timing::{begin_work, note_work};
use crate::fs_beneath::{create_file_at, entry_metadata_at, make_directory_at, open_directory_at, open_file_at, plain_components, read_link_at, set_times_at, symlink_at};
use crate::fs_beneath_walk_chain::open_directory_beneath;
use crate::fs_clone_exchange::clone_file;
use crate::subvolume_index::StatFields;
use crate::subvolume_index_files::fields_of;
use crate::worktree_clean::{StatEntry, metadata_differs_from_entry};
use crate::worktree_clone::{Created, Directories};
use crate::worktree_path_failure_message::subject_failure;
use crate::worktree_snapshot_source::SnapshotSource;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

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
pub(crate) fn entry_names(path: &[u8]) -> Result<Vec<OsString>, String> {
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
    pub(crate) entry: &'a StatEntry,
    /// What the checks proved about the source.
    pub(crate) source: &'a SnapshotSource,
    /// The held root of the source's files.
    pub(crate) source_root: &'a File,
    /// The destination directories held so far.
    pub(crate) directories: &'a mut Directories,
    /// The source directory held for the entries before this one.
    pub(crate) held_source: &'a mut Option<(Vec<OsString>, File)>,
    /// The moments a control moves the trees.
    pub(crate) seams: &'a mut Seams<'run>,
}

/// What one entry's clone records.
pub(crate) struct EntryResults<'a> {
    /// The entries created,
    /// in creation order.
    pub(crate) created: &'a mut Vec<Created>,
    /// The stat fields of the files made,
    /// by path.
    pub(crate) stats: &'a mut HashMap<Vec<u8>, StatFields>,
    /// What the entries of this run spent in each part of an entry,
    /// summed here so the ledger is written once per run.
    pub(crate) spent: &'a mut SpentByTheEntries,
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
    pub(crate) fn note(&self) {
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
