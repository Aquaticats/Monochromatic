//! What:
//!  The deterministic, parent-first list of entries a copy transfers,
//!  read without following symbolic links,
//!  and the application of the retained permission bits to a staged copy.
//! Why:
//!  The stage and the source are compared entry by entry,
//!  so both sides must enumerate in the same order and classify the same way.
//!  Sockets, FIFOs and device nodes stop the walk before any of them is opened.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const entries = await collectEntryManifest({ root, selectedRoots, excludedRoots });
//! await applyEntryModes({ root: stageRoot, entries });
//! ```

use super::worktree_copy_error::raw_failure;
use super::worktree_copy_error::{WorktreeCopyError, failure_with_cause};
use super::worktree_copy_selection::{filesystem_path, js_order};
use crate::json_record::quote;
use std::path::{Path, PathBuf};

/// What:
///  The permission and special-mode bits the copy contract retains.
/// Why:
///   Only these bits are compared and applied; file-type bits are never copied.
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
///   Each kind is copied, compared and modified differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeCopyEntryKind = 'directory' | 'file' | 'symlink';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKind {
    /// A directory, recreated rather than copied.
    Directory,
    /// A regular file, copied byte for byte.
    File,
    /// A symbolic link, recreated with its exact target text.
    Symlink,
}

/// What:
///  One entry of a manifest: its kind, its retained mode and its repository path.
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
    /// Slash-separated repository path.
    pub relative_path: String,
}

/// What:
///  One pending walk item: its native path and its repository path.
/// Why:
///   The walk keeps both, because exclusion works on native paths and output on repository paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WalkItem = { filesystemPath: string; relativePath: string };
/// ```
struct WalkItem {
    /// Native path under the walked root.
    filesystem_path: PathBuf,
    /// Repository path retained in the manifest.
    relative_path: String,
}

/// What:
///  The copy kind of one no-follow filesystem entry,
///  or the failure for a socket, FIFO or device node.
/// Why:
///   Only three kinds can be copied; anything else must fail before it is opened.
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
        "cli-git: ignored path {} has unsupported filesystem type.",
        quote(path.to_string_lossy().as_ref())
    );
    return Err(failure_with_cause(message.as_str(), None));
}

/// What:
///  The permission bits of one no-follow entry, as the copy contract retains them.
/// Why:
///   The mode is read from the entry itself, never from what a symbolic link points to.
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
///   The copy is Unix-only in this lane; the zero mode keeps non-Unix builds compiling
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
///  The complete manifest of a set of selected roots, parent before child, in JavaScript sort order.
/// Why:
///   A deterministic order makes the stage, the comparison and the journal reproducible.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function collectEntryManifest({ root, selectedRoots, excludedRoots }): Promise<readonly WorktreeCopyEntry[]>;
/// ```
pub fn collect_entry_manifest(
    root: &Path,
    selected_roots: &[String],
    excluded_roots: &[PathBuf],
) -> Result<Vec<WorktreeCopyEntry>, WorktreeCopyError> {
    // The stack is filled in reverse so the lexically first root is popped first.
    let mut pending: Vec<WalkItem> = Vec::new();
    for relative_path in selected_roots.iter().rev() {
        pending.push(WalkItem {
            filesystem_path: filesystem_path(root, relative_path.as_str())?,
            relative_path: relative_path.clone(),
        });
    }
    let mut entries: Vec<WorktreeCopyEntry> = Vec::new();
    while let Some(item) = pending.pop() {
        if excluded_roots
            .iter()
            .any(|excluded| return item.filesystem_path.starts_with(excluded.as_path()))
        {
            continue;
        }
        let metadata: std::fs::Metadata = std::fs::symlink_metadata(item.filesystem_path.as_path())
            .map_err(|error| return raw_failure(&error))?;
        let kind: EntryKind = entry_kind(metadata.file_type(), item.filesystem_path.as_path())?;
        entries.push(WorktreeCopyEntry {
            kind,
            mode: retained_mode(&metadata),
            relative_path: item.relative_path.clone(),
        });
        if kind != EntryKind::Directory {
            continue;
        }
        let mut names: Vec<String> = Vec::new();
        for listed in std::fs::read_dir(item.filesystem_path.as_path())
            .map_err(|error| return raw_failure(&error))?
        {
            let child: std::fs::DirEntry = listed.map_err(|error| return raw_failure(&error))?;
            names.push(child.file_name().to_string_lossy().into_owned());
        }
        names.sort_by(|left, right| return js_order(left, right));
        for name in names.iter().rev() {
            let child_relative: String = format!("{}/{name}", item.relative_path);
            pending.push(WalkItem {
                filesystem_path: filesystem_path(root, child_relative.as_str())?,
                relative_path: child_relative,
            });
        }
    }
    return Ok(entries);
}

/// What:
///  Applies each staged entry's retained mode, files first and then directories deepest first.
/// Why:
///   A restrictive directory mode would block changes to its children, so directories change last.
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
    use std::os::unix::fs::PermissionsExt;
    let files = entries
        .iter()
        .filter(|entry| return entry.kind == EntryKind::File);
    let directories = entries
        .iter()
        .rev()
        .filter(|entry| return entry.kind == EntryKind::Directory);
    for entry in files.chain(directories) {
        let path: PathBuf = filesystem_path(root, entry.relative_path.as_str())?;
        std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(entry.mode))
            .map_err(|error| return raw_failure(&error))?;
    }
    return Ok(());
}

/// What:
///  Applies modes on a platform without Unix mode bits, which leaves them untouched.
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
