//! What:
//!  The exclusive creation of one destination entry from the private stage,
//!  the identity captured right after it exists,
//!  and the creation of the unselected parent directories an entry needs.
//! Why:
//!  A regular file is installed as a hard link to its staged copy,
//!  so it appears with its final bytes and mode in one step;
//!  the identity captured after creation is the only proof later rollback accepts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await ensureParents({ destinationRoot, entry, created });
//! await createSelectedEntry({ snapshot, destinationRoot, entry });
//! created.push(await captureInstalledPath({ destinationPath, relativePath, selected: true }));
//! ```

use super::worktree_copy_compare::lstat_or_absent;
use super::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure};
use super::worktree_copy_manifest::{EntryKind, WorktreeCopyEntry};
use super::worktree_copy_selection::filesystem_path;
use super::worktree_copy_stage::copy_file_exclusive;
use crate::json_record::quote;
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::path::{Path, PathBuf};

/// What:
///  The permission bits of a directory cli-git creates for a selected entry, before modes are applied.
/// Why:
///   Restrictive modes can block the children installation still has to create, so modes come last.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PRIVATE_DIRECTORY_MODE = 0o700;
/// ```
const PRIVATE_DIRECTORY_MODE: u32 = 0o700;

/// What:
///  The operating-system errors that mean the filesystem cannot hard-link the staged file.
/// Why:
///   Only these refusals fall back to a copy; any other failure is reported as it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LINK_UNSUPPORTED_CODES = new Set(['EMLINK', 'ENOSYS', 'ENOTSUP', 'EOPNOTSUPP', 'EPERM', 'EXDEV']);
/// ```
const LINK_UNSUPPORTED_CODES: [i32; 6] = [
    libc::EMLINK,
    libc::ENOSYS,
    libc::ENOTSUP,
    libc::EOPNOTSUPP,
    libc::EPERM,
    libc::EXDEV,
];

/// What:
///  One path cli-git created, with the identity the filesystem gave it.
/// Why:
///   Rollback removes a path only when its device and inode still match this record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InstalledWorktreePath = { device: string; inode: string; relativePath: string; selected: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledPath {
    /// Device number reported by the filesystem after creation.
    pub device: u64,
    /// Inode number reported by the filesystem after creation.
    pub inode: u64,
    /// Slash-separated repository path.
    pub relative_path: String,
    /// Whether the path came from the selected source manifest rather than being parent scaffolding.
    pub selected: bool,
}

/// What:
///  The no-follow identity of a path, read right after its creation.
/// Why:
///   The identity names the exact object cli-git made, not whatever later sits at the same path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function captureInstalledPath({ destinationPath, relativePath, selected }): Promise<InstalledWorktreePath>;
/// ```
pub fn capture_installed_path(
    destination_path: &Path,
    relative_path: &str,
    selected: bool,
) -> Result<InstalledPath, WorktreeCopyError> {
    let metadata: std::fs::Metadata =
        std::fs::symlink_metadata(destination_path).map_err(|error| return raw_failure(&error))?;
    return Ok(InstalledPath {
        device: metadata.dev(),
        inode: metadata.ino(),
        relative_path: String::from(relative_path),
        selected,
    });
}

/// What:
///  Creates each missing unselected parent directory of one entry, shallow first, recording what it created.
/// Why:
///   An entry's parents are scaffolding that rollback must be able to remove again, so each one is recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function ensureParents({ destinationRoot, entry, created }): Promise<void>;
/// ```
pub fn ensure_parents(
    destination_root: &Path,
    entry: &WorktreeCopyEntry,
    created: &mut Vec<InstalledPath>,
) -> Result<(), WorktreeCopyError> {
    let components: Vec<&str> = entry.relative_path.split('/').collect();
    let parent_count: usize = components.len().saturating_sub(1);
    for depth in 1..=parent_count {
        let current: String = components[..depth].join("/");
        let destination_path: PathBuf = filesystem_path(destination_root, current.as_str())?;
        // `match` on the existing entry: a directory is kept, anything else is a refusal.
        match lstat_or_absent(destination_path.as_path())? {
            Some(metadata) => {
                if !metadata.is_dir() {
                    let message: String = format!(
                        "cli-git: ignored-state parent is not a directory: {} in {}.",
                        quote(current.as_str()),
                        quote(destination_root.to_string_lossy().as_ref())
                    );
                    return Err(failure_with_cause(message.as_str(), None));
                }
            }
            None => {
                std::fs::create_dir(destination_path.as_path())
                    .map_err(|error| return raw_failure(&error))?;
                created.push(capture_installed_path(
                    destination_path.as_path(),
                    current.as_str(),
                    false,
                )?);
            }
        }
    }
    return Ok(());
}

/// What:
///  Whether a hard-link failure means the filesystem cannot link the staged file.
/// Why:
///   Only a refusal of the link itself falls back to a copy; a missing source or a full disk does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isLinkUnsupported(error: unknown): boolean;
/// ```
fn is_link_unsupported(error: &std::io::Error) -> bool {
    return error
        .raw_os_error()
        .is_some_and(|code| return LINK_UNSUPPORTED_CODES.contains(&code));
}

/// What:
///  Installs one staged regular file: a hard link when the filesystem allows it, otherwise an exclusive copy with the entry's mode.
/// Why:
///   The link shares the staged bytes and mode in one step; the copy is the fallback the contract requires.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installFile({ stagePath, destinationPath, mode }): Promise<void>;
/// ```
fn install_file(
    stage_path: &Path,
    destination_path: &Path,
    mode: u32,
) -> Result<(), WorktreeCopyError> {
    // `match` on the link result: success ends here, a refusal copies, and any other failure is reported.
    match std::fs::hard_link(stage_path, destination_path) {
        Ok(()) => return Ok(()),
        Err(error) if is_link_unsupported(&error) => {}
        Err(error) => return Err(raw_failure(&error)),
    }
    copy_file_exclusive(stage_path, destination_path)?;
    let applied: std::io::Result<()> = std::fs::set_permissions(
        destination_path,
        std::os::unix::fs::PermissionsExt::from_mode(mode),
    );
    if let Err(error) = applied {
        // A copy that cannot take its mode is removed, so the next attempt starts from the absent path.
        let _ignored_removal: std::io::Result<()> = std::fs::remove_file(destination_path);
        return Err(raw_failure(&error));
    }
    return Ok(());
}

/// What:
///  Creates one absent selected entry from the private stage, as its kind requires.
/// Why:
///   Directories are created private and changed later; links are recreated with their exact target text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function createSelectedEntry({ snapshot, destinationRoot, entry }): Promise<void>;
/// ```
pub fn create_selected_entry(
    stage_root: &Path,
    destination_root: &Path,
    entry: &WorktreeCopyEntry,
) -> Result<(), WorktreeCopyError> {
    let stage_path: PathBuf = filesystem_path(stage_root, entry.relative_path.as_str())?;
    let destination_path: PathBuf =
        filesystem_path(destination_root, entry.relative_path.as_str())?;
    // `match` on the kind selects the one creation that applies to it.
    match entry.kind {
        EntryKind::Directory => {
            return std::fs::DirBuilder::new()
                .mode(PRIVATE_DIRECTORY_MODE)
                .create(destination_path.as_path())
                .map_err(|error| return raw_failure(&error));
        }
        EntryKind::File => {
            return install_file(stage_path.as_path(), destination_path.as_path(), entry.mode);
        }
        EntryKind::Symlink => {
            let target: PathBuf = std::fs::read_link(stage_path.as_path())
                .map_err(|error| return raw_failure(&error))?;
            return std::os::unix::fs::symlink(target, destination_path.as_path())
                .map_err(|error| return raw_failure(&error));
        }
    }
}

/// Entry-creation controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_install_entry_tests.rs"]
mod tests;
