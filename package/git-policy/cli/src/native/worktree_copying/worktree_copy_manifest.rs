//! What:
//!  The deterministic,
//! parent-first list of entries a copy transfers,
//!  read without following symbolic links,
//!  and the application of the retained permission bits to a staged copy.
//! Why:
//!  The stage and the source are compared entry by entry,
//!  so both sides must enumerate in the same order and classify the same way.
//!  Sockets,
//! FIFOs and device nodes stop the walk before any of them is opened.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const entries = await collectEntryManifest({ root, selectedRoots, excludedRoots });
//! await applyEntryModes({ root: stageRoot, entries });
//! ```

use crate::worktree_copy_error::raw_failure_of;
use crate::worktree_copy_error::{WorktreeCopyError, failure_with_cause};
use crate::worktree_copy_prune::Prune;
use crate::worktree_copy_selection::{by_repository_path_order, filesystem_path, join_repository_path};
use crate::path_bytes::path_bytes_of;
use crate::json_record::quote;
use std::path::{Path, PathBuf};

/// What:
///  The permission and special-mode bits the copy contract retains.
/// Why:
///   Only these bits are compared and applied;
/// file-type bits are never copied.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PERMISSION_BITS = 0o7777;
/// ```
#[cfg(unix)]
const PERMISSION_BITS: u32 = 0o7777;

/// What:
///  The kind of one copied entry.
/// Why:
///   Each kind is copied,
/// compared and modified differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeCopyEntryKind = 'directory' | 'file' | 'symlink';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKind {
    /// A directory,
    /// recreated rather than copied.
    Directory,
    /// A regular file,
    /// copied byte for byte.
    File,
    /// A symbolic link,
    /// recreated with its exact target text.
    Symlink,
}

/// What:
///  One entry of a manifest:
/// its kind,
/// its retained mode and its repository path.
/// Why:
///   A manifest is compared with another manifest entry by entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeCopyEntry = { kind: WorktreeCopyEntryKind; mode: number; relativePath: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorktreeCopyEntry {
    /// Copy kind of the entry.
    pub kind: EntryKind,
    /// Retained permission bits;
    ///  zero where the platform has no such bits.
    pub mode: u32,
    /// Slash-separated repository path,
    /// as the bytes the filesystem listed.
    pub relative_path: Vec<u8>,
}

/// What:
///  One pending walk item:
/// its native path and its repository path.
/// Why:
///   The walk keeps both,
/// because exclusion works on native paths and output on repository paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WalkItem = { filesystemPath: string; relativePath: string };
/// ```
struct WalkItem {
    /// Native path under the walked root.
    filesystem_path: PathBuf,
    /// Repository path retained in the manifest.
    relative_path: Vec<u8>,
}

/// What:
///  The copy kind of one no-follow filesystem entry,
///  or the failure for a socket,
/// FIFO or device node.
/// Why:
///   Only three kinds can be copied;
/// anything else must fail before it is opened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryKind({ stats, path }): WorktreeCopyEntryKind;
/// ```
fn entry_kind(file_type: std::fs::FileType, path: &Path) -> Result<EntryKind, WorktreeCopyError> {
    // `if` chains keep each supported kind next to its name; the last branch refuses the rest.
    if file_type.is_dir() {
        return Ok(EntryKind::Directory);
    }
    if file_type.is_file() {
        return Ok(EntryKind::File);
    }
    if file_type.is_symlink() {
        return Ok(EntryKind::Symlink);
    }
    let message: String = format!(
        "monochromatic-git-policy-cli: ignored path {} has unsupported filesystem type.",
        quote(path.to_string_lossy().as_ref())
    );
    return Err(failure_with_cause(message.as_str(), None));
}

/// What:
///  The permission bits of one no-follow entry,
/// as the copy contract retains them.
/// Why:
///   The mode is read from the entry itself,
/// never from what a symbolic link points to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stats.mode & 0o7777
/// ```
#[cfg(unix)]
pub fn retained_mode(metadata: &std::fs::Metadata) -> u32 {
    use std::os::unix::fs::MetadataExt;
    return metadata.mode() & PERMISSION_BITS;
}

/// What:
///  The permission bits of one entry on a platform without Unix mode bits.
/// Why:
///   The copy is Unix-only in this lane;
/// the zero mode keeps non-Unix builds compiling
///  without claiming the mode parity the incumbent provides there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stats.mode & 0o7777
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub fn retained_mode(_metadata: &std::fs::Metadata) -> u32 {
    return 0;
}

/// What:
///  Whether the running user can read an entry that the walk found,
/// as a file or as a directory to list.
/// Why:
///   A file or directory the user cannot read (a root-owned log in a dependency tree) cannot be part of a copy a user
///  makes,
/// so the walk leaves it out and says so instead of failing the whole copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await access(path, directory ? R_OK | X_OK : R_OK).then(() => true, () => false)
/// ```
#[cfg(unix)]
fn readable_by_user(path: &Path, kind: EntryKind) -> bool {
    use std::os::unix::ffi::OsStrExt;
    if kind == EntryKind::Symlink {
        return true;
    }
    let Ok(name) = std::ffi::CString::new(path.as_os_str().as_bytes()) else {
        return false;
    };
    let mode: libc::c_int = if kind == EntryKind::Directory { libc::R_OK | libc::X_OK } else { libc::R_OK };
    // SAFETY: `name` is a NUL-terminated string that outlives the call, and `access` only reads it.
    return unsafe { libc::access(name.as_ptr(), mode) } == 0;
}

/// What:
///  Every entry is readable on a platform without Unix permission checks in this lane.
/// Why:
///   Keeps the walk identical on every platform;
/// a refused open then fails the copy as before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// true
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn readable_by_user(_path: &Path, _kind: EntryKind) -> bool {
    return true;
}

/// What:
///  The complete manifest of a set of selected roots,
/// parent before child,
/// in JavaScript sort order.
/// Why:
///   A deterministic order makes the stage,
/// the comparison and the journal reproducible.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function collectEntryManifest({ root, selectedRoots, excludedRoots }): Promise<readonly WorktreeCopyEntry[]>;
/// ```
pub fn collect_entry_manifest(
    root: &Path,
    selected_roots: &[Vec<u8>],
    excluded_roots: &[PathBuf],
) -> Result<Vec<WorktreeCopyEntry>, WorktreeCopyError> {
    let mut skipped: Vec<Vec<u8>> = Vec::new();
    return collect_pruned_manifest(root, selected_roots, excluded_roots, Prune::Never, &mut skipped);
}

/// What:
///  The manifest of a set of selected roots under a leave-out rule,
/// with the unreadable entries it left out.
/// Why:
///   A copy from a main checkout leaves out caches and other checkouts,
/// and never fails over an entry the user cannot
///  read;
/// the caller reports those so the person sees what the new worktree lacks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function collectPrunedManifest({ root, selectedRoots, excludedRoots, prune, skipped }): Promise<readonly WorktreeCopyEntry[]>;
/// ```
pub fn collect_pruned_manifest(
    root: &Path,
    selected_roots: &[Vec<u8>],
    excluded_roots: &[PathBuf],
    prune: Prune,
    unreadable: &mut Vec<Vec<u8>>,
) -> Result<Vec<WorktreeCopyEntry>, WorktreeCopyError> {
    // The stack is filled in reverse so the lexically first root is popped first.
    let mut pending: Vec<WalkItem> = Vec::new();
    for relative_path in selected_roots.iter().rev() {
        pending.push(WalkItem {
            filesystem_path: filesystem_path(root, relative_path.as_slice())?,
            relative_path: relative_path.clone(),
        });
    }
    let mut entries: Vec<WorktreeCopyEntry> = Vec::new();
    while let Some(item) = pending.pop() {
        if lies_within_any(item.filesystem_path.as_path(), excluded_roots) || prune.skips_path(item.relative_path.as_slice()) {
            continue;
        }
        let metadata: std::fs::Metadata = std::fs::symlink_metadata(item.filesystem_path.as_path())
            .map_err(raw_failure_of)?;
        let kind: EntryKind = entry_kind(metadata.file_type(), item.filesystem_path.as_path())?;
        if !readable_by_user(item.filesystem_path.as_path(), kind) {
            unreadable.push(item.relative_path.clone());
            continue;
        }
        if kind != EntryKind::Directory {
            entries.push(WorktreeCopyEntry {
                kind,
                mode: retained_mode(&metadata),
                relative_path: item.relative_path.clone(),
            });
            continue;
        }
        let mut names: Vec<Vec<u8>> = Vec::new();
        for listed in std::fs::read_dir(item.filesystem_path.as_path())
            .map_err(raw_failure_of)?
        {
            let child: std::fs::DirEntry = listed.map_err(raw_failure_of)?;
            // The name is the bytes the filesystem listed; a name that is not text is still this worktree's entry.
            names.push(path_bytes_of(Path::new(child.file_name().as_os_str())).to_vec());
        }
        // A directory that is a checkout of its own is left out whole, with no entry of its own.
        if prune.skips_checkout(names.as_slice()) {
            continue;
        }
        entries.push(WorktreeCopyEntry {
            kind,
            mode: retained_mode(&metadata),
            relative_path: item.relative_path.clone(),
        });
        names.sort_by(by_repository_path_order);
        for name in names.iter().rev() {
            let child_relative: Vec<u8> = join_repository_path(item.relative_path.as_slice(), name.as_slice());
            pending.push(WalkItem {
                filesystem_path: filesystem_path(root, child_relative.as_slice())?,
                relative_path: child_relative,
            });
        }
    }
    return Ok(entries);
}

/// What:
///  The identity of a regular file's content as the filesystem reports it:
/// size,
/// modification time,
/// change time and inode.
/// Why:
///   A cloned file shares its data blocks with the source,
/// so equal stamps before and after the clone and again at the
///  end of the copy prove the source did not change in between without reading a byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FileStamp = { size: number; mtimeNs: bigint; ctimeNs: bigint; inode: bigint };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileStamp {
    /// Size in bytes.
    pub size: u64,
    /// Modification time,
    /// seconds and nanoseconds.
    pub modified: (i64, i64),
    /// Change time,
    /// seconds and nanoseconds;
    /// no program can set it,
    /// so a write always moves it.
    pub changed: (i64, i64),
    /// Inode number,
    /// so a replacement by rename is a difference.
    pub inode: u64,
}

/// What:
///  The stamp of one no-follow entry,
/// or nothing on a platform that reports no change time.
/// Why:
///   Without a stamp the caller compares bytes,
/// which is always correct.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stampOf(await lstat(path))
/// ```
#[cfg(unix)]
pub fn stamp_of(metadata: &std::fs::Metadata) -> Option<FileStamp> {
    use std::os::unix::fs::MetadataExt;
    return Some(FileStamp {
        size: metadata.len(),
        modified: (metadata.mtime(), metadata.mtime_nsec()),
        changed: (metadata.ctime(), metadata.ctime_nsec()),
        inode: metadata.ino(),
    });
}

/// What:
///  No stamp on a platform without a change time,
/// so every cloned file is compared byte for byte.
/// Why:
///   Keeps the call sites identical on every platform.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// undefined
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub fn stamp_of(_metadata: &std::fs::Metadata) -> Option<FileStamp> {
    return None;
}

/// What:
///  Whether a path is at or below any of the roots.
/// Why:
///   Excluded roots,
/// such as registered worktrees and the stage,
/// are never walked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// roots.some((root) => path.startsWith(root))
/// ```
fn lies_within_any(path: &Path, roots: &[PathBuf]) -> bool {
    for root in roots {
        if path.starts_with(root.as_path()) {
            return true;
        }
    }
    return false;
}

/// What:
///  Give one staged entry the mode its manifest entry recorded.
/// Why:
///   Modes are applied after the content so a restrictive mode cannot stop the copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// chmodSync(join(root, entry.relativePath), entry.mode)
/// ```
#[cfg(unix)]
fn set_entry_mode(root: &Path, entry: &WorktreeCopyEntry) -> Result<(), WorktreeCopyError> {
    use std::os::unix::fs::PermissionsExt;
    let path: PathBuf = filesystem_path(root, entry.relative_path.as_slice())?;
    return std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(entry.mode)).map_err(raw_failure_of);
}

/// What:
///  Applies each staged entry's retained mode,
/// files first and then directories deepest first.
/// Why:
///   A restrictive directory mode would block changes to its children,
/// so directories change last.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function applyEntryModes({ root, entries }): Promise<void>;
/// ```
#[cfg(unix)]
pub fn apply_entry_modes(
    root: &Path,
    entries: &[WorktreeCopyEntry],
) -> Result<(), WorktreeCopyError> {
    // Files first, in manifest order; then directories, deepest first, so a directory is made restrictive after its content.
    for entry in entries {
        if entry.kind == EntryKind::File {
            set_entry_mode(root, entry)?;
        }
    }
    for entry in entries.iter().rev() {
        if entry.kind == EntryKind::Directory {
            set_entry_mode(root, entry)?;
        }
    }
    return Ok(());
}

/// What:
///  Applies modes on a platform without Unix mode bits,
/// which leaves them untouched.
/// Why:
///   Keeps the call sites identical on every platform.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function applyEntryModes({ root, entries }): Promise<void>;
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub fn apply_entry_modes(
    _root: &Path,
    _entries: &[WorktreeCopyEntry],
) -> Result<(), WorktreeCopyError> {
    return Ok(());
}

/// Manifest controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_manifest_tests.rs"]
mod tests;
