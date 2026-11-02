//! What:
//!  Rung 2 of the cheap-worktree ladder: each tracked file of the clean source is cloned (`FICLONE`) into the new worktree
//!  with the source's mode and times, and the source's index is installed with fresh stat fields.
//! Why:
//!  A clone shares every data block with the source on btrfs, on XFS with reflink and inside an overlay, which the
//!  snapshot rung cannot reach (overlayfs and XFS have reflink but no subvolumes). It takes no less time than Git's checkout
//!  on btrfs (`doc/planning/cli-git-btrfs-worktree.md`, option B), so it saves disk, not time, and it is the accelerated
//!  rung where a snapshot is impossible. The wrapper's own index writer is `rewrite_index_stat_with`: with each file's
//!  size and modification time kept, only the inode, change time and device of every entry change. The first failure ends
//!  the rung with its reason after the entries this run created are removed (and nothing else), and Git's own checkout
//!  completes the worktree.
//!  Every source file is opened below a held handle of the source root without following a link, and its descriptor's
//!  metadata, not a path, is compared with the frozen index entry before and after the clone; every destination entry is
//!  created below a held handle of the destination. A directory swapped for a link while the run goes on is an error,
//!  never a detour.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await cloneTrackedFiles(source, destination, admin); // throws a reason
//! ```

use super::fs_beneath::{create_file_at, entry_metadata_at, make_directory_at, open_directory, open_directory_at, open_directory_beneath, open_file_at, plain_components, read_link_at, remove_at, set_times_at, symlink_at};
use super::fs_clone_exchange::clone_file;
use super::subvolume_index_files::{IndexLock, RefreshOptions, install_index_file};
use super::worktree_clean::{StatEntry, metadata_differs_from_entry, stored_stats};
use super::worktree_snapshot_source::SnapshotSource;
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
fn identity_of(from: &std::fs::Metadata) -> [i64; 8] {
    return [from.dev() as i64, from.ino() as i64, from.size() as i64, from.mtime(), from.mtime_nsec(), from.ctime(), from.ctime_nsec(), i64::from(from.mode())];
}

/// What:
///  An entry this run created, named by its components below the destination.
/// Why:
///   The run removes exactly these, and only through the destination handle, when it fails.
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

/// What:
///  What a successful clone run holds: the destination handle and the entries the run created.
/// Why:
///   A later step that fails (the tree difference) removes the clones through the same handle, and nothing the run did not
///  create.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Cloned = { destination: Dir; created: Created[]; remove(): void };
/// ```
#[derive(Debug)]
pub struct Cloned {
    /// The destination directory as it was opened.
    destination: File,
    /// The entries created, in creation order.
    created: Vec<Created>,
}

impl Cloned {
    /// What:
    ///  Remove the entries this run created, last first, below the held destination handle.
    /// Why:
    ///   A name that is gone is fine; a directory that is not empty holds something the run did not create and stays.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// cloned.remove();
    /// ```
    pub fn remove(&self) {
        remove_created(&self.destination, self.created.as_slice());
    }

    /// The entries the run created, in creation order.
    pub fn created(&self) -> &[Created] {
        return self.created.as_slice();
    }
}

/// Remove `created` below `destination`, last first, ignoring what is gone and what a directory still holds.
fn remove_created(destination: &File, created: &[Created]) {
    for entry in created.iter().rev() {
        let Some((name, parents)) = entry.names.split_last() else { continue };
        let Ok(parent) = open_directory_beneath(destination, parents) else { continue };
        let _removed = remove_at(&parent, name.as_os_str(), entry.directory);
    }
}

/// The path through which a held directory handle names that directory, for tools that take a path.
fn handle_path(directory: &File) -> PathBuf {
    return Path::new("/proc/self/fd").join(directory.as_raw_fd().to_string());
}

/// What:
///  Create the directories of `names[..len - 1]` below `destination` and return the last one, recording the ones made.
/// Why:
///   Each component is opened without following a link, so the file goes into a directory this run made or found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parent = ensureParents(destination, names, created);
/// ```
fn ensure_parents(destination: &File, names: &[OsString], created: &mut Vec<Created>) -> Result<File, String> {
    let mut current: File = destination.try_clone().map_err(|error| { return format!("the destination handle cannot be kept ({error})"); })?;
    for (index, name) in names[..names.len() - 1].iter().enumerate() {
        let made: bool = make_directory_at(&current, name.as_os_str(), 0o777).map_err(|error| { return format!("{} cannot be created ({error})", name.to_string_lossy()); })?;
        if made {
            created.push(Created { names: names[..=index].to_vec(), directory: true });
        }
        current = open_directory_at(&current, name.as_os_str()).map_err(|error| { return format!("{} cannot be entered ({error})", name.to_string_lossy()); })?;
    }
    return Ok(current);
}

/// What:
///  The two moments of a clone at which the controls move the source tree.
/// Why:
///   The races the held handles and the descriptor comparisons close happen between two system calls, so a control that
///  wants one must run its change there; the release code passes calls that do nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Seams = { afterLookup: () => void; afterClone: () => void };
/// ```
struct Seams<'a> {
    /// Called after the entry was looked at and before it is opened.
    after_lookup: &'a mut dyn FnMut(),
    /// Called after the data was cloned and before the source is looked at again.
    after_clone: &'a mut dyn FnMut(),
}

/// What:
///  Create one destination entry from its source below the held directories: a clone of a regular file with the source's
///  mode and times, or a link with the same target. The source's descriptor metadata must equal the frozen index entry
///  before the clone and be unchanged after it.
/// Why:
///   The bytes cloned are those of the inode that was compared, so a swap of a parent directory cannot certify other
///  bytes as the index's. An entry of any other type is not a tracked file Git checks out this way, so the rung ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await cloneEntry({ sourceDir, destinationDir, name, entry, indexSeconds });
/// ```
fn clone_entry(source_dir: &File, destination_dir: &File, names: (&OsStr, &[OsString]), entry: &StatEntry, index_seconds: u32, created: &mut Vec<Created>, seams: &mut Seams<'_>) -> Result<(), String> {
    let (name, all_names): (&OsStr, &[OsString]) = names;
    let shown: String = String::from_utf8_lossy(entry.path.as_slice()).into_owned();
    let changed = |reason: String| { return format!("the source changed while it was cloned ({reason})"); };
    let metadata: std::fs::Metadata = entry_metadata_at(source_dir, name).map_err(|error| { return format!("{shown} is unreadable ({error})"); })?;
    if metadata.file_type().is_symlink() {
        if let Some(reason) = metadata_differs_from_entry(entry, &metadata, index_seconds) {
            return Err(changed(reason));
        }
        let target: PathBuf = read_link_at(source_dir, name).map_err(|error| { return format!("{shown} is unreadable ({error})"); })?;
        symlink_at(target.as_path(), destination_dir, name).map_err(|error| { return format!("{shown} cannot be linked ({error})"); })?;
        created.push(Created { names: all_names.to_vec(), directory: false });
        set_times_at(destination_dir, name, times_of(&metadata)).map_err(|error| { return format!("the times of {shown} cannot be set ({error})"); })?;
        let after: std::fs::Metadata = entry_metadata_at(source_dir, name).map_err(|error| { return format!("{shown} is unreadable ({error})"); })?;
        return if identity_of(&after) == identity_of(&metadata) { Ok(()) } else { Err(changed(format!("{shown} was replaced"))) };
    }
    if !metadata.is_file() {
        return Err(format!("{shown} is neither a file nor a link"));
    }
    (seams.after_lookup)();
    let source: File = open_file_at(source_dir, name).map_err(|error| { return format!("{shown} is unreadable ({error})"); })?;
    let held: std::fs::Metadata = source.metadata().map_err(|error| { return format!("{shown} is unreadable ({error})"); })?;
    if let Some(reason) = metadata_differs_from_entry(entry, &held, index_seconds) {
        return Err(changed(reason));
    }
    let destination: File = create_file_at(destination_dir, name, 0o600).map_err(|error| { return format!("{shown} cannot be created ({error})"); })?;
    created.push(Created { names: all_names.to_vec(), directory: false });
    if held.len() > 0 {
        clone_file(&source, &destination, Path::new(name)).map_err(|error| { return format!("the clone was refused ({error})"); })?;
    }
    (seams.after_clone)();
    let after: std::fs::Metadata = source.metadata().map_err(|error| { return format!("{shown} is unreadable ({error})"); })?;
    if identity_of(&after) != identity_of(&held) {
        return Err(changed(format!("{shown} was written during the clone")));
    }
    destination.set_permissions(std::fs::Permissions::from_mode(held.mode() & 0o7777)).map_err(|error| { return format!("the mode of {shown} cannot be set ({error})"); })?;
    let times: [libc::timespec; 2] = times_of(&held);
    // SAFETY: `destination` is an open descriptor and `times` holds the two timestamps the call reads.
    let result: libc::c_int = unsafe { libc::futimens(destination.as_raw_fd(), times.as_ptr()) };
    return if result == 0 { Ok(()) } else { Err(format!("the times of {shown} cannot be set ({})", std::io::Error::last_os_error())) };
}

/// The run itself: every tracked file, then the index; the caller removes `created` when this fails.
fn clone_all(source: &SnapshotSource, destination: &File, admin: &Path, created: &mut Vec<Created>) -> Result<(), String> {
    let mut seams: Seams<'_> = Seams { after_lookup: &mut || {}, after_clone: &mut || {} };
    let entries: Vec<StatEntry> = stored_stats(source.clean.index_bytes.as_slice(), source.clean.hash_length).ok_or_else(|| { return String::from("the source's index changed shape"); })?;
    let root: File = open_directory(source.root.as_path()).map_err(|error| { return format!("the source's root cannot be opened ({error})"); })?;
    for entry in &entries {
        let names: Vec<OsString> = entry_names(entry.path.as_slice())?;
        let source_dir: File = open_directory_beneath(&root, &names[..names.len() - 1]).map_err(|error| { return format!("the source directory of {} cannot be opened ({error})", String::from_utf8_lossy(entry.path.as_slice())); })?;
        let destination_dir: File = ensure_parents(destination, names.as_slice(), created)?;
        let name: &OsString = &names[names.len() - 1];
        clone_entry(&source_dir, &destination_dir, (name.as_os_str(), names.as_slice()), entry, source.clean.index_seconds, created, &mut seams)?;
    }
    let tree: PathBuf = handle_path(destination);
    let references: [PathBuf; 1] = [source.root.clone()];
    let report: String = install_index_file(admin.join("index").as_path(), tree.as_path(), source.clean.index_bytes.as_slice(), &RefreshOptions { lock: IndexLock::Acquire, references: &references, hash_length: Some(source.clean.hash_length) });
    return if report.starts_with("skipped") { Err(format!("the index could not be refreshed ({report})")) } else { Ok(()) };
}

/// What:
///  Clone every tracked file of the frozen index into `destination`, and install that index in `admin` with refreshed stat
///  fields. On an error the entries this run created are removed, and nothing else.
/// Why:
///   The index bytes are the ones proven clean, so what is installed is what was proven; the destination may hold files
///  of other writers, and they are not this run's to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cloned = await cloneTrackedFiles(source, destination, admin); // throws a reason
/// ```
pub fn clone_tracked_files(source: &SnapshotSource, destination: &Path, admin: &Path) -> Result<Cloned, String> {
    let held: File = open_directory(destination).map_err(|error| { return format!("{} cannot be opened ({error})", destination.display()); })?;
    let mut created: Vec<Created> = Vec::new();
    if let Err(reason) = clone_all(source, &held, admin, &mut created) {
        remove_created(&held, created.as_slice());
        return Err(reason);
    }
    return Ok(Cloned { destination: held, created });
}

/// Clone rung controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_clone_tests.rs"]
mod tests;
