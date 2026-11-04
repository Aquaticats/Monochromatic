//! What:
//!  Rung 2 of the cheap-worktree ladder: each tracked file of the clean source is cloned (`FICLONE`) into the new worktree
//!  with the source's mode and times, and the source's index is installed with fresh stat fields.
//! Why:
//!  A clone shares every data block with the source on btrfs, on XFS with reflink and inside an overlay, which the
//!  snapshot rung cannot reach (overlayfs and XFS have reflink but no subvolumes). It takes no less time than Git's checkout
//!  on btrfs (`doc/planning/cli-git-btrfs-worktree.md`, option B), so it saves disk, not time, and it is the accelerated
//!  rung where a snapshot is impossible. The wrapper's own index writer is `rewrite_index_stat_with`: with each file's
//!  size and modification time kept, only the inode, change time and device of every entry change. The first failure ends
//!  the rung with its reason, the entries this run created stay (a name cannot be checked and unlinked in one step), and
//!  Git's own checkout completes the worktree and overwrites the tracked ones.
//!  Every source file is opened below a held handle of the source root without following a link, and its descriptor's
//!  metadata, not a path, is compared with the frozen index entry before and after the clone; every destination entry is
//!  created below a held handle of the destination. A directory swapped for a link while the run goes on is an error,
//!  never a detour.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await cloneTrackedFiles(source, destination, admin); // throws a reason
//! ```

use crate::fs_beneath::{create_file_at, entry_metadata_at, make_directory_at, open_directory_at, open_file_at, open_root, plain_components, read_link_at, set_times_at, symlink_at};
use crate::fs_clone_exchange::clone_file;
use crate::subvolume_index::StatFields;
use crate::subvolume_index_files::{fields_of, install_index_in};
use crate::worktree_clean::{StatEntry, metadata_differs_from_entry, stored_stats};
use crate::worktree_snapshot_source::SnapshotSource;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// What:
///  The names of a tracked entry's path below a root, refused unless every component is a plain name and none is `.git`.
/// Why:
///   The index is the source's own, but a path that climbs out of the destination or enters `.git` is never created.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryNames(path: Uint8Array): string[]; // throws
/// ```
fn entry_names(path: &[u8]) -> Result<Vec<OsString>, String> {
    let refused = || { return format!("the index path {} is not a plain path", String::from_utf8_lossy(path)); };
    let names: Vec<OsString> = plain_components(Path::new(OsStr::from_bytes(path))).map_err(|_error| { return refused(); })?;
    return if names.is_empty() || names.iter().any(|name: &OsString| { return name == ".git"; }) { Err(refused()) } else { Ok(names) };
}

/// What:
///  The access and modification times of metadata, to the nanosecond.
/// Why:
///   The index entry keeps the source's modification time, and Git trusts a file whose size and time equal it.
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
///  The identity a clone must find again after it: device, inode, size, and both times.
/// Why:
///   A source that is written while it is cloned changes its size or times; a swapped inode changes the inode.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Identity = [dev, ino, size, mtime, ctime];
/// ```
fn identity_of_metadata(from: &std::fs::Metadata) -> [i64; 8] {
    return [from.dev() as i64, from.ino() as i64, from.size() as i64, from.mtime(), from.mtime_nsec(), from.ctime(), from.ctime_nsec(), i64::from(from.mode())];
}

/// What:
///  An entry this run created, named by its components below the destination.
/// Why:
///   Nothing is removed on a failure: a name cannot be checked and unlinked in one step, and the entry may by then be another
///  process's, so the run keeps what it made and reports it. The list is what the report names.
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

/// The directories this run walked, by their components below the destination (the empty list is the destination), each
///  held open so that a later removal acts in the directory where the entry was made, whatever its name became.
type Directories = HashMap<Vec<OsString>, File>;

/// What:
///  What a successful clone run holds: the entries it created.
/// Why:
///   A later step that fails (the tree difference) reports them; Git's own checkout then overwrites the tracked ones, and the
///  rest stay as untracked files for the person to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Cloned = { created: Created[] };
/// ```
#[derive(Debug)]
pub struct Cloned {
    /// The entries created, in creation order.
    created: Vec<Created>,
}

impl Cloned {
    /// The entries the run created, in creation order.
    pub fn created(&self) -> &[Created] {
        return self.created.as_slice();
    }
}

/// What:
///  The report of entries a clone run created and leaves in place.
/// Why:
///   The tracked ones are overwritten by the checkout that completes the worktree; any other stays as an untracked file, and
///  the person is told where to look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = leftover_report(created, destination);
/// ```
pub fn leftover_report(created: &[Created], destination: &Path) -> String {
    let shown: Vec<String> = created.iter().filter(|entry: &&Created| { return !entry.directory; }).take(3).map(|entry: &Created| { return entry.names.iter().map(|name: &OsString| { return name.to_string_lossy().into_owned(); }).collect::<Vec<String>>().join("/"); }).collect();
    return format!("{} entries the clones created stay in {} (for example {}); the checkout that completes the worktree overwrites the tracked ones, and `git -C {} status` lists any other as untracked", created.len(), destination.display(), shown.join(", "), destination.display());
}

/// What:
///  Create the directories of `names[..len - 1]` below the destination and return the last one, recording the ones made
///  and holding every directory walked.
/// Why:
///   Each component is opened without following a link or crossing a mount, so the file goes into a directory this run
///  made or found, on the destination's device.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parent = ensureParents(directories, names, created);
/// ```
fn ensure_parents(directories: &mut Directories, names: &[OsString], created: &mut Vec<Created>) -> Result<File, String> {
    let mut current: File = directories.get(&Vec::new()).ok_or_else(|| { return String::from("the destination is not held"); })?.try_clone().map_err(|error| { return format!("the destination handle cannot be kept ({error})"); })?;
    for index in 0..names.len() - 1 {
        let key: Vec<OsString> = names[..=index].to_vec();
        if let Some(known) = directories.get(&key) {
            current = known.try_clone().map_err(|error| { return format!("a directory handle cannot be kept ({error})"); })?;
            continue;
        }
        let name: &OsString = &names[index];
        let made: bool = make_directory_at(&current, name.as_os_str(), 0o777).map_err(|error| { return format!("{} cannot be created ({error})", name.to_string_lossy()); })?;
        let next: File = open_directory_at(&current, name.as_os_str()).map_err(|error| { return format!("{} cannot be entered ({error})", name.to_string_lossy()); })?;
        if crate::fs_beneath::device_of(&next).ok() != crate::fs_beneath::device_of(&current).ok() {
            return Err(format!("{} is on another device", name.to_string_lossy()));
        }
        if made {
            created.push(Created { names: key.clone(), directory: true });
        }
        directories.insert(key, next.try_clone().map_err(|error| { return format!("a directory handle cannot be kept ({error})"); })?);
        current = next;
    }
    return Ok(current);
}

/// What:
///  The moments of a clone at which the controls move the source or the destination tree.
/// Why:
///   The races the held handles and the descriptor comparisons close happen between two system calls, so a control that
///  wants one must run its change there; the release code passes calls that do nothing.
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

/// What one clone call works on: the entry, its names and the places the results go.
struct Work<'a> {
    /// The entry of the frozen index.
    entry: &'a StatEntry,
    /// The components of its path.
    names: &'a [OsString],
    /// The whole seconds of the index file's time.
    index_seconds: u32,
}

/// What:
///  Create one destination entry from its source below the held directories: a clone of a regular file with the source's
///  mode and times, or a link with the same target. The source's descriptor metadata must equal the frozen index entry
///  before the clone and be unchanged after it; the stat fields recorded for the index are those of the object made here.
/// Why:
///   The bytes cloned are those of the inode that was compared, so a swap of a parent directory cannot certify other
///  bytes as the index's, and a destination file replaced afterwards no longer has the inode the index entry records. An
///  entry of any other type is not a tracked file Git checks out this way, so the rung ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await cloneEntry({ sourceDir, destinationDir, work, created, stats });
/// ```
fn clone_entry(source_dir: &File, destination_dir: &File, work: &Work<'_>, results: (&mut Vec<Created>, &mut HashMap<Vec<u8>, StatFields>), seams: &mut Seams<'_>) -> Result<(), String> {
    let (created, stats): (&mut Vec<Created>, &mut HashMap<Vec<u8>, StatFields>) = results;
    let Work { entry, names: all_names, index_seconds }: &Work<'_> = work;
    let name: &OsStr = all_names[all_names.len() - 1].as_os_str();
    let shown: String = String::from_utf8_lossy(entry.path.as_slice()).into_owned();
    let changed = |reason: String| { return format!("the source changed while it was cloned ({reason})"); };
    let metadata: std::fs::Metadata = entry_metadata_at(source_dir, name).map_err(|error| { return format!("{shown} is unreadable ({error})"); })?;
    if metadata.file_type().is_symlink() {
        if let Some(reason) = metadata_differs_from_entry(entry, &metadata, *index_seconds) {
            return Err(changed(reason));
        }
        let target: PathBuf = read_link_at(source_dir, name).map_err(|error| { return format!("{shown} is unreadable ({error})"); })?;
        symlink_at(target.as_path(), destination_dir, name).map_err(|error| { return format!("{shown} cannot be linked ({error})"); })?;
        created.push(Created { names: all_names.to_vec(), directory: false });
        set_times_at(destination_dir, name, times_of(&metadata)).map_err(|error| { return format!("the times of {shown} cannot be set ({error})"); })?;
        let after: std::fs::Metadata = entry_metadata_at(source_dir, name).map_err(|error| { return format!("{shown} is unreadable ({error})"); })?;
        if identity_of_metadata(&after) != identity_of_metadata(&metadata) {
            return Err(changed(format!("{shown} was replaced")));
        }
        // A link cannot be held by a descriptor that reads its metadata, so the name is not asked: the entry keeps the
        // source's stat fields, which no longer match the new link, and Git compares the link's target on first use.
        return Ok(());
    }
    if !metadata.is_file() {
        return Err(format!("{shown} is neither a file nor a link"));
    }
    (seams.after_lookup)();
    let source: File = open_file_at(source_dir, name).map_err(|error| { return format!("{shown} is unreadable ({error})"); })?;
    let held: std::fs::Metadata = source.metadata().map_err(|error| { return format!("{shown} is unreadable ({error})"); })?;
    if let Some(reason) = metadata_differs_from_entry(entry, &held, *index_seconds) {
        return Err(changed(reason));
    }
    let destination: File = create_file_at(destination_dir, name, 0o600).map_err(|error| { return format!("{shown} cannot be created ({error})"); })?;
    created.push(Created { names: all_names.to_vec(), directory: false });
    if held.len() > 0 {
        clone_file(&source, &destination, Path::new(name)).map_err(|error| { return format!("the clone was refused ({error})"); })?;
    }
    (seams.after_clone)();
    let after: std::fs::Metadata = source.metadata().map_err(|error| { return format!("{shown} is unreadable ({error})"); })?;
    if identity_of_metadata(&after) != identity_of_metadata(&held) {
        return Err(changed(format!("{shown} was written during the clone")));
    }
    destination.set_permissions(std::fs::Permissions::from_mode(held.mode() & 0o7777)).map_err(|error| { return format!("the mode of {shown} cannot be set ({error})"); })?;
    let times: [libc::timespec; 2] = times_of(&held);
    // SAFETY: `destination` is an open descriptor and `times` holds the two timestamps the call reads.
    let result: libc::c_int = unsafe { libc::futimens(destination.as_raw_fd(), times.as_ptr()) };
    if result != 0 {
        return Err(format!("the times of {shown} cannot be set ({})", std::io::Error::last_os_error()));
    }
    // The fields of the object made here, taken from its descriptor after its last change.
    let made: std::fs::Metadata = destination.metadata().map_err(|error| { return format!("{shown} cannot be examined ({error})"); })?;
    stats.insert(entry.path.clone(), fields_of(&made));
    return Ok(());
}

/// The run itself: every tracked file, then the index; the caller removes `created` when this fails.
fn clone_all(source: &SnapshotSource, directories: &mut Directories, admin: &File, created: &mut Vec<Created>) -> Result<(), String> {
    let mut seams: Seams<'_> = Seams { after_lookup: &mut || {}, after_clone: &mut || {}, before_index: &mut || {} };
    return clone_with(source, directories, admin, created, &mut seams);
}

/// `clone_all` with the moments at which a control may move the tree.
fn clone_with(source: &SnapshotSource, directories: &mut Directories, admin: &File, created: &mut Vec<Created>, seams: &mut Seams<'_>) -> Result<(), String> {
    let entries: Vec<StatEntry> = stored_stats(source.clean.index_bytes.as_slice(), source.clean.hash_length).ok_or_else(|| { return String::from("the source's index changed shape"); })?;
    let root: File = open_root(source.root.as_path()).map_err(|error| { return format!("the source's root cannot be opened ({error})"); })?;
    let mut stats: HashMap<Vec<u8>, StatFields> = HashMap::new();
    for entry in &entries {
        let names: Vec<OsString> = entry_names(entry.path.as_slice())?;
        let source_dir: File = crate::fs_beneath::open_directory_beneath(&root, &names[..names.len() - 1]).map_err(|error| { return format!("the source directory of {} cannot be opened ({error})", String::from_utf8_lossy(entry.path.as_slice())); })?;
        let destination_dir: File = ensure_parents(directories, names.as_slice(), created)?;
        let work: Work<'_> = Work { entry, names: names.as_slice(), index_seconds: source.clean.index_seconds };
        clone_entry(&source_dir, &destination_dir, &work, (&mut *created, &mut stats), seams)?;
    }
    (seams.before_index)();
    let report: String = install_index_in(admin, source.clean.index_bytes.as_slice(), &mut |path: &[u8]| { return stats.get(path).copied(); }, source.clean.hash_length)?;
    debug_report(report.as_str());
    return Ok(());
}

/// Log what the index install did.
fn debug_report(report: &str) {
    crate::diagnostic_log::debug("cloneTrackedFiles", report);
}

/// What:
///  Clone every tracked file of the frozen index into the held `destination`, and install that index in the held `admin`
///  with the stat fields of the regular files made. On an error nothing is removed: the entries this run created stay and
///  are counted in the reason.
/// Why:
///   The index bytes are the ones proven clean, so what is installed is what was proven; the destination may hold files
///  of other writers, and they are not this run's to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cloned = await cloneTrackedFiles(source, destination, admin); // throws a reason
/// ```
pub fn clone_tracked_files(source: &SnapshotSource, destination: &File, admin: &File) -> Result<Cloned, String> {
    let mut directories: Directories = HashMap::new();
    directories.insert(Vec::new(), destination.try_clone().map_err(|error| { return format!("the destination handle cannot be kept ({error})"); })?);
    let mut created: Vec<Created> = Vec::new();
    if let Err(reason) = clone_all(source, &mut directories, admin, &mut created) {
        return Err(if created.is_empty() { reason } else { format!("{reason}; {} entries it created stay", created.len()) });
    }
    return Ok(Cloned { created });
}

/// Clone rung controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_clone_tests.rs"]
mod tests;
