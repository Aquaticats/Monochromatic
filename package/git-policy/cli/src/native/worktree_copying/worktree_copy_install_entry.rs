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

use crate::worktree_copy_compare::lstat_or_absent;
use crate::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure, raw_failure_of};
use crate::worktree_copy_manifest::{EntryKind, WorktreeCopyEntry};
use crate::worktree_copy_selection::{filesystem_path, is_separator_byte, parent_of_repository_path, shown_repository_path};
use crate::json_record::quote;
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::path::{Path, PathBuf};

/// What:
///  The permission bits of a directory monochromatic-git-policy-cli creates for a selected entry,
/// before modes are applied.
/// Why:
///   Restrictive modes can block the children installation still has to create,
/// so modes come last.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PRIVATE_DIRECTORY_MODE = 0o700;
/// ```
const PRIVATE_DIRECTORY_MODE: u32 = 0o700;

/// What:
///  The operating-system errors that mean the filesystem cannot hard-link the staged file.
/// Why:
///   Only these refusals fall back to a copy;
/// any other failure is reported as it is.
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
///  One path monochromatic-git-policy-cli created,
/// with the identity the filesystem gave it.
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
    /// Slash-separated repository path,
    /// as the bytes the filesystem listed.
    pub relative_path: Vec<u8>,
    /// Whether the path came from the selected source manifest rather than being parent scaffolding.
    pub selected: bool,
}

/// What:
///  The no-follow identity of a path,
/// read right after its creation.
/// Why:
///   The identity names the exact object monochromatic-git-policy-cli made,
/// not whatever later sits at the same path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function captureInstalledPath({ destinationPath, relativePath, selected }): Promise<InstalledWorktreePath>;
/// ```
pub fn capture_installed_path(
    destination_path: &Path,
    relative_path: &[u8],
    selected: bool,
) -> Result<InstalledPath, WorktreeCopyError> {
    let metadata: std::fs::Metadata =
        std::fs::symlink_metadata(destination_path).map_err(raw_failure_of)?;
    return Ok(InstalledPath {
        device: metadata.dev(),
        inode: metadata.ino(),
        relative_path: relative_path.to_vec(),
        selected,
    });
}

/// What:
///  Creates each missing unselected parent directory of one entry,
/// shallow first,
/// recording what it created.
/// Why:
///   An entry's parents are scaffolding that rollback must be able to remove again,
/// so each one is recorded.
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
    return ensure_parents_remembering(destination_root, entry, created, &mut std::collections::HashSet::new());
}

/// What:
///  Creates each missing unselected parent directory of one entry,
/// skipping every parent an earlier call already proved.
/// Why:
///   Entries arrive parent first,
/// so almost every entry's parent was just proved or created;
/// asking the filesystem about
///  each ancestor of each of hundreds of thousands of entries costs more than the clones themselves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function ensureParentsRemembering({ destinationRoot, entry, created, known }): Promise<void>;
/// ```
pub fn ensure_parents_remembering(
    destination_root: &Path,
    entry: &WorktreeCopyEntry,
    created: &mut Vec<InstalledPath>,
    known: &mut std::collections::HashSet<Vec<u8>>,
) -> Result<(), WorktreeCopyError> {
    let Some(parent) = parent_of_repository_path(entry.relative_path.as_slice()) else {
        return Ok(());
    };
    if known.contains(parent) {
        return Ok(());
    }
    let components: Vec<&[u8]> = entry
        .relative_path
        .split(is_separator_byte)
        .collect();
    let parent_count: usize = components.len().saturating_sub(1);
    for depth in 1..=parent_count {
        let current: Vec<u8> = joined_components(components[..depth].as_ref());
        let destination_path: PathBuf = filesystem_path(destination_root, current.as_slice())?;
        // `match` on the existing entry: a directory is kept, anything else is a refusal.
        match lstat_or_absent(destination_path.as_path())? {
            Some(metadata) => {
                if !metadata.is_dir() {
                    let message: String = format!(
                        "monochromatic-git-policy-cli: ignored-state parent is not a directory: {} in {}.",
                        quote(shown_repository_path(current.as_slice()).as_str()),
                        quote(destination_root.to_string_lossy().as_ref())
                    );
                    return Err(failure_with_cause(message.as_str(), None));
                }
            }
            None => {
                std::fs::create_dir(destination_path.as_path())
                    .map_err(raw_failure_of)?;
                created.push(capture_installed_path(
                    destination_path.as_path(),
                    current.as_slice(),
                    false,
                )?);
            }
        }
    }
    for depth in 1..=parent_count {
        known.insert(joined_components(components[..depth].as_ref()));
    }
    return Ok(());
}

/// What:
///  One repository path made of already validated components.
/// Why:
///   The scaffolding loop names each ancestor of an entry,
/// so it needs the same join the walk uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// components.join('/')
/// ```
fn joined_components(components: &[&[u8]]) -> Vec<u8> {
    let mut joined: Vec<u8> = Vec::new();
    for component in components {
        if !joined.is_empty() {
            joined.push(b'/');
        }
        joined.extend_from_slice(component);
    }
    return joined;
}

/// What:
///  Whether a hard-link failure means the filesystem cannot link the staged file.
/// Why:
///   Only a refusal of the link itself falls back to a copy;
/// a missing source or a full disk does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isLinkUnsupported(error: unknown): boolean;
/// ```
fn is_link_unsupported(error: &std::io::Error) -> bool {
    return match error.raw_os_error() {
        Some(code) => LINK_UNSUPPORTED_CODES.contains(&code),
        None => false,
    };
}

/// What:
///  Installs one staged regular file:
/// a hard link when the filesystem allows it,
/// otherwise a copy published at the destination name only once its bytes and mode are whole.
/// Why:
///   The link shares the staged bytes and mode in one step;
/// the copy is the fallback the contract requires,
/// and  it can never leave a partial file with the destination's name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installFile({ stageRoot, stagePath, destinationPath, entry }): Promise<void>;
/// ```
fn install_file(
    stage_root: &Path,
    stage_path: &Path,
    destination_path: &Path,
    entry: &WorktreeCopyEntry,
) -> Result<(), WorktreeCopyError> {
    // The link publishes the staged object itself, so the staged file takes the entry's mode first: the destination's
    // first instant is never wider than its source. A stage that cannot take it is copied instead, and the copy sets
    // the mode on the file it fills.
    if std::fs::set_permissions(stage_path, std::os::unix::fs::PermissionsExt::from_mode(entry.mode)).is_ok() {
        // `match` on the link result: success ends here, a refusal copies, and any other failure is reported.
        match std::fs::hard_link(stage_path, destination_path) {
            Ok(()) => return Ok(()),
            Err(error) if is_link_unsupported(&error) => {}
            Err(error) => return Err(raw_failure(&error)),
        }
    }
    let temporary: std::ffi::OsString = crate::worktree_copy_publish_file::temporary_name_of(
        stage_container_name(stage_root).as_str(),
        entry.relative_path.as_slice(),
    );
    return crate::worktree_copy_publish_file::publish_copy_of_file(
        stage_path,
        destination_path,
        entry.mode,
        temporary.as_os_str(),
    );
}

/// What:
///  The name of the private stage container one payload belongs to.
/// Why:
///   The name carries the random identifier of this copy,
/// so a temporary file a killed run left is told from every other copy's by its name alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stageContainerName = (stageRoot) => path.basename(path.dirname(stageRoot));
/// ```
fn stage_container_name(stage_root: &Path) -> String {
    let Some(container) = stage_root.parent().and_then(Path::file_name) else {
        return String::new();
    };
    return container.to_string_lossy().into_owned();
}

/// What:
///  Creates one absent selected entry from the private stage,
/// as its kind requires.
/// Why:
///   Directories are created private and changed later;
/// links are recreated with their exact target text.
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
    let stage_path: PathBuf = filesystem_path(stage_root, entry.relative_path.as_slice())?;
    let destination_path: PathBuf =
        filesystem_path(destination_root, entry.relative_path.as_slice())?;
    // `match` on the kind selects the one creation that applies to it.
    match entry.kind {
        EntryKind::Directory => {
            return std::fs::DirBuilder::new()
                .mode(PRIVATE_DIRECTORY_MODE)
                .create(destination_path.as_path())
                .map_err(raw_failure_of);
        }
        EntryKind::File => {
            return install_file(stage_root, stage_path.as_path(), destination_path.as_path(), entry);
        }
        EntryKind::Symlink => {
            let target: PathBuf = std::fs::read_link(stage_path.as_path())
                .map_err(raw_failure_of)?;
            return std::os::unix::fs::symlink(target, destination_path.as_path())
                .map_err(raw_failure_of);
        }
    }
}

/// Entry-creation controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_install_entry_tests.rs"]
mod tests;
