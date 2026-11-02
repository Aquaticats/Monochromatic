//! What:
//!  Rung 2 of the cheap-worktree ladder: each tracked file of the clean source is cloned (`FICLONE`) into the new worktree
//!  with the source's mode and times, and the source's index is installed with fresh stat fields.
//! Why:
//!  A clone shares every data block with the source on btrfs, on XFS with reflink and inside an overlay, which the
//!  snapshot rung cannot reach (overlayfs and XFS have reflink but no subvolumes). It takes no less time than Git's checkout
//!  on btrfs (`doc/planning/cli-git-btrfs-worktree.md`, option B), so it saves disk, not time, and it is the accelerated
//!  rung where a snapshot is impossible. The wrapper's own index writer is `rewrite_index_stat_with`: with each file's
//!  size and modification time kept, only the inode, change time and device of every entry change. The first failure ends
//!  the rung with its reason, the caller clears the partial worktree, and Git's own checkout completes it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await cloneTrackedFiles(source, destination, admin); // throws a reason
//! ```

use super::fs_clone_exchange::clone_file;
use super::subvolume_index_files::{IndexLock, RefreshOptions, refresh_index_file_with};
use super::subvolume_remove::remove_tree;
use super::worktree_clean::{StatEntry, stored_stats, tree_differs_from_index};
use super::worktree_snapshot_source::SnapshotSource;
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};

/// What:
///  The path of a tracked entry below a root, refused unless every component is a plain name and none is `.git`.
/// Why:
///   The index is the source's own, but a path that climbs out of the destination or enters `.git` is never created.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryPath(root: string, path: Uint8Array): string; // throws
/// ```
fn entry_path(root: &Path, path: &[u8]) -> Result<PathBuf, String> {
    let relative: &Path = Path::new(OsStr::from_bytes(path));
    let plain: bool = relative.components().all(|part: Component<'_>| { return matches!(part, Component::Normal(name) if name != ".git"); });
    return if plain && !path.is_empty() { Ok(root.join(relative)) } else { Err(format!("the index path {} is not a plain path", String::from_utf8_lossy(path))) };
}

/// What:
///  Set a path's access and modification times to the source's, to the nanosecond, without following a symbolic link.
/// Why:
///   The index entry keeps the source's modification time, and Git trusts a file whose size and time equal it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await lutimes(path, atime, mtime);
/// ```
fn copy_times(from: &std::fs::Metadata, to: &Path) -> Result<(), String> {
    let name = std::ffi::CString::new(to.as_os_str().as_bytes()).map_err(|_error| { return String::from("a path holds a NUL byte"); })?;
    let times: [libc::timespec; 2] = [
        libc::timespec { tv_sec: from.atime() as libc::time_t, tv_nsec: from.atime_nsec() as libc::c_long },
        libc::timespec { tv_sec: from.mtime() as libc::time_t, tv_nsec: from.mtime_nsec() as libc::c_long },
    ];
    // SAFETY: `name` is a NUL-terminated string that outlives the call and `times` holds the two timestamps the call reads.
    let result: libc::c_int = unsafe { libc::utimensat(libc::AT_FDCWD, name.as_ptr(), times.as_ptr(), libc::AT_SYMLINK_NOFOLLOW) };
    return if result == 0 { Ok(()) } else { Err(format!("the times of {} cannot be set ({})", to.display(), std::io::Error::last_os_error())) };
}

/// What:
///  Create one destination entry from its source: a clone of a regular file with the source's mode and times, or a link
///  with the same target.
/// Why:
///   An entry of any other type is not a tracked file Git checks out this way, so the rung ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await cloneEntry(from, to);
/// ```
fn clone_entry(from: &Path, to: &Path) -> Result<(), String> {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(from).map_err(|error| { return format!("{} is unreadable ({error})", from.display()); })?;
    if metadata.file_type().is_symlink() {
        let target: PathBuf = std::fs::read_link(from).map_err(|error| { return format!("{} is unreadable ({error})", from.display()); })?;
        std::os::unix::fs::symlink(target, to).map_err(|error| { return format!("{} cannot be linked ({error})", to.display()); })?;
        return copy_times(&metadata, to);
    }
    if !metadata.is_file() {
        return Err(format!("{} is neither a file nor a link", from.display()));
    }
    let source: File = File::open(from).map_err(|error| { return format!("{} is unreadable ({error})", from.display()); })?;
    let destination: File = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(to).map_err(|error| { return format!("{} cannot be created ({error})", to.display()); })?;
    if metadata.len() > 0 {
        clone_file(&source, &destination, from).map_err(|error| { return format!("the clone was refused ({error})"); })?;
    }
    destination.set_permissions(std::fs::Permissions::from_mode(metadata.mode() & 0o7777)).map_err(|error| { return format!("the mode of {} cannot be set ({error})", to.display()); })?;
    drop(destination);
    return copy_times(&metadata, to);
}

/// What:
///  Remove everything in a destination except Git's `.git` file.
/// Why:
///   A partial clone must not stay under the checkout that completes the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await clearDestination(destination);
/// ```
pub fn clear_destination(destination: &Path) -> Result<(), String> {
    for entry in std::fs::read_dir(destination).map_err(|error| { return format!("{} is unreadable ({error})", destination.display()); })?.flatten() {
        if entry.file_name() != ".git" {
            remove_tree(entry.path().as_path()).map_err(|error| { return format!("{} cannot be cleared ({error})", entry.path().display()); })?;
        }
    }
    return Ok(());
}

/// What:
///  Clone every tracked file of the source into `destination`, check that the source did not change meanwhile, and install
///  the source's index in `admin` with refreshed stat fields.
/// Why:
///   The check reuses the clean check's comparison against the source's own files, so a file written during the clone is
///  caught by its size or time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await cloneTrackedFiles(source, destination, admin); // throws a reason
/// ```
pub fn clone_tracked_files(source: &SnapshotSource, destination: &Path, admin: &Path) -> Result<(), String> {
    let bytes: Vec<u8> = std::fs::read(source.git_dir.join("index")).map_err(|error| { return format!("the source's index is unreadable ({error})"); })?;
    let entries: Vec<StatEntry> = stored_stats(bytes.as_slice(), source.clean.hash_length).ok_or_else(|| { return String::from("the source's index changed shape"); })?;
    let mut made: BTreeSet<PathBuf> = BTreeSet::new();
    for entry in &entries {
        let to: PathBuf = entry_path(destination, entry.path.as_slice())?;
        if let Some(parent) = to.parent() && made.insert(parent.to_path_buf()) {
            std::fs::create_dir_all(parent).map_err(|error| { return format!("{} cannot be created ({error})", parent.display()); })?;
        }
        clone_entry(entry_path(source.root.as_path(), entry.path.as_slice())?.as_path(), to.as_path())?;
    }
    if let Some(reason) = tree_differs_from_index(source.root.as_path(), entries.as_slice(), source.clean.index_seconds) {
        return Err(format!("the source changed while it was cloned ({reason})"));
    }
    let target = admin.join("index");
    std::fs::write(target.as_path(), bytes.as_slice()).map_err(|error| { return format!("the index cannot be written ({error})"); })?;
    let references: [PathBuf; 1] = [source.root.clone()];
    let report: String = refresh_index_file_with(target.as_path(), destination, &RefreshOptions { lock: IndexLock::Acquire, references: &references, hash_length: Some(source.clean.hash_length) });
    return if report.starts_with("skipped") { Err(format!("the index could not be refreshed ({report})")) } else { Ok(()) };
}

/// Clone rung controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_clone_tests.rs"]
mod tests;
