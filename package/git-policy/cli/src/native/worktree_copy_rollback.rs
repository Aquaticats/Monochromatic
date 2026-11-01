//! What:
//!  Removal of the paths one failed installation created,
//!  child first, and only where the filesystem still proves they are cli-git's own.
//! Why:
//!  A failed installation must not leave a partial worktree, but it also must never delete a path
//!  the user or another process put there after cli-git created it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const retained = await rollbackCreated({ snapshot, destinationRoot, created });
//! ```

use super::worktree_copy_compare::{entry_matches, lstat_or_absent};
use super::worktree_copy_error::WorktreeCopyError;
use super::worktree_copy_install_entry::InstalledPath;
use super::worktree_copy_manifest::{EntryKind, WorktreeCopyEntry};
use super::worktree_copy_selection::filesystem_path;
use std::collections::BTreeMap;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// What:
///  Where a recorded path stands relative to the identity cli-git recorded for it.
/// Why:
///   Only an exact match may be removed; an absent path has nothing to remove and a changed one is someone else's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IdentityState = 'absent' | 'changed' | 'match';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IdentityState {
    /// Nothing exists at the path.
    Absent,
    /// Something exists with a different device or inode.
    Changed,
    /// The object cli-git created is still there.
    Match,
}

/// What:
///  The current identity state of one recorded path, read without following links.
/// Why:
///   Checked immediately before each removal, so a path replaced during rollback is never removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installedIdentityState({ path, installed }): Promise<'absent' | 'changed' | 'match'>;
/// ```
fn identity_state(
    path: &Path,
    installed: &InstalledPath,
) -> Result<IdentityState, WorktreeCopyError> {
    return match lstat_or_absent(path)? {
        None => Ok(IdentityState::Absent),
        Some(metadata) => {
            if metadata.dev() == installed.device && metadata.ino() == installed.inode {
                Ok(IdentityState::Match)
            } else {
                Ok(IdentityState::Changed)
            }
        }
    };
}

/// What:
///  The one removal of a proven path: a directory only when empty, a file or link by unlinking it.
/// Why:
///   A directory is removed with `rmdir`, which refuses a non-empty one, so user files inside it survive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await ((expected?.kind === 'directory') || (!installed.selected) ? rmdir(path) : unlink(path));
/// ```
fn remove_proven(path: &Path, is_directory: bool) -> std::io::Result<()> {
    if is_directory {
        return std::fs::remove_dir(path);
    }
    return std::fs::remove_file(path);
}

/// What:
///  Removes every path the failed installation created that still carries cli-git's identity and content,
///  and returns a diagnostic for each path it had to keep.
/// Why:
///   Returning the kept paths lets the user see exactly what remains, instead of a silent partial removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function rollbackCreated({ snapshot, destinationRoot, created }): Promise<readonly string[]>;
/// ```
pub fn rollback_created(
    stage_root: &Path,
    destination_root: &Path,
    entries: &[WorktreeCopyEntry],
    created: &[InstalledPath],
) -> Result<Vec<String>, WorktreeCopyError> {
    let selected_by_path: BTreeMap<&str, &WorktreeCopyEntry> = entries
        .iter()
        .map(|entry| return (entry.relative_path.as_str(), entry))
        .collect();
    let mut retained: Vec<String> = Vec::new();
    for installed in created.iter().rev() {
        let destination_path: std::path::PathBuf =
            filesystem_path(destination_root, installed.relative_path.as_str())?;
        let expected: Option<&&WorktreeCopyEntry> =
            selected_by_path.get(installed.relative_path.as_str());
        let outcome: Result<Option<String>, WorktreeCopyError> = rollback_one(
            stage_root,
            destination_root,
            destination_path.as_path(),
            installed,
            expected.copied(),
        );
        match outcome {
            Ok(None) => {}
            Ok(Some(reason)) => retained.push(reason),
            Err(error) => retained.push(format!("{}: {}", installed.relative_path, error.0)),
        }
    }
    return Ok(retained);
}

/// What:
///  The rollback decision for one created path.
/// Why:
///   Each proof is checked in order, and the result names the first one that failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // one iteration of the rollbackCreated loop
/// ```
fn rollback_one(
    stage_root: &Path,
    destination_root: &Path,
    destination_path: &Path,
    installed: &InstalledPath,
    expected: Option<&WorktreeCopyEntry>,
) -> Result<Option<String>, WorktreeCopyError> {
    let initial: IdentityState = identity_state(destination_path, installed)?;
    // `match` on the first identity state decides whether the path is ours to examine.
    match initial {
        IdentityState::Absent => return Ok(None),
        IdentityState::Changed => {
            return Ok(Some(format!(
                "{}: filesystem identity changed",
                installed.relative_path
            )));
        }
        IdentityState::Match => {}
    }
    // A directory keeps its private installation mode until modes are applied, and `rmdir` removes it only when empty,
    // so its identity is the whole proof; a file or link must still equal the stage.
    let content_differs: bool = match expected {
        Some(entry) if entry.kind != EntryKind::Directory => {
            !entry_matches(stage_root, destination_root, entry)?
        }
        _ => false,
    };
    if content_differs {
        return Ok(Some(installed.relative_path.clone()));
    }
    // The identity is read again immediately before removal, to close the race with a replacement.
    let final_state: IdentityState = identity_state(destination_path, installed)?;
    // `match` on the recheck: only an exact match proceeds; a changed path is named, an absent one is done.
    match final_state {
        IdentityState::Match => {}
        IdentityState::Absent => return Ok(None),
        IdentityState::Changed => {
            return Ok(Some(format!(
                "{}: filesystem identity changed",
                installed.relative_path
            )));
        }
    }
    let is_directory: bool = expected
        .is_some_and(|entry| return entry.kind == EntryKind::Directory)
        || !installed.selected;
    // The removal's own text is kept as the reason, as the incumbent reports it.
    remove_proven(destination_path, is_directory)
        .map_err(|error| return WorktreeCopyError(error.to_string()))?;
    return Ok(None);
}

/// Rollback controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_rollback_tests.rs"]
mod tests;
