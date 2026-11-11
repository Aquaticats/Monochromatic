//! What:
//!  Moving the nested Git repositories a clean would delete beside the audit log,
//!  before Git runs.
//! Why:
//!  A nested repository holds its own history and its own uncommitted files,
//!  and Git's clean with a second `-f` removes the whole directory,
//!  which no copy of the surrounding repository holds.
//!  The owner's answer for uncommitted work is "save,
//! then allow" (2026-11-02,
//!  `conflict_side_guard.rs`),
//! and a rename keeps every byte of the repository without copying it:
//!  the directory moves under `<common git dir>/monochromatic-git-policy-cli/discarded/<stamp>/`,
//!  the clean then runs as typed,
//!  and the audit line names the place.
//!  A move the system refuses is returned,
//!  and the caller stops the command naming the repository that stays where it is.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const copies = await setNestedRepositoriesAside({ commonDir, worktreeRoot, repositories, milliseconds });
//! ```

use crate::conflict_sides::{SavedCopy, SaveFailure, depth_deepest_first, fresh_saved_directory};
use crate::fs_beneath::plain_components;
use crate::private_storage::{create_private_directory_all, sync_directory};
/// `OsStr`/`OsString` are operating-system text of raw bytes.
use std::ffi::OsStr;
/// `OsStrExt` gives `OsStr::from_bytes`,
/// because a path is bytes and not text.
use std::os::unix::ffi::OsStrExt;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths,
///  and a set of them keeps every directory synced once.
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// What:
///  Move each repository below the place saved copies live,
///  keeping its path below the worktree,
///  and answer where each one went.
///  `&[Vec<u8>]` borrows the paths Git named,
/// relative to the worktree top level.
/// Why:
///   A rename is the whole save:
///  every byte,
///  every link and the repository's own `.git` travel without being read,
///  so a nested repository of any size costs one directory entry move,
///  and a kill leaves it whole in one of the two places.
///  The directories that received an entry are synced after the last move,
///  so the saved repository outlives the power loss that would otherwise lose the only copy.
///  The first refusal stops everything and is returned,
///  because a half-moved set would be a worktree nobody chose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function setNestedRepositoriesAside(commonDir, worktreeRoot, repositories, milliseconds): SavedCopy[]; // throws
/// ```
pub fn set_nested_repositories_aside(
    common_dir: &Path,
    worktree_root: &Path,
    repositories: &[Vec<u8>],
    milliseconds: i64,
) -> Result<Vec<SavedCopy>, SaveFailure> {
    let base: PathBuf = match fresh_saved_directory(common_dir, milliseconds) {
        Ok(created) => created,
        Err(error) => return Err(SaveFailure::io(common_dir.display().to_string(), error.to_string())),
    };
    let mut copies: Vec<SavedCopy> = Vec::<SavedCopy>::new();
    let mut written: BTreeSet<PathBuf> = BTreeSet::<PathBuf>::new();
    for path in repositories {
        let shown: String = String::from_utf8_lossy(path).into_owned();
        let relative: &Path = Path::new(OsStr::from_bytes(path));
        if let Err(error) = plain_components(relative) {
            return Err(SaveFailure::io(shown, error.to_string()));
        }
        let destination: PathBuf = base.join(relative);
        let Some(parent) = destination.parent() else {
            return Err(SaveFailure::io(shown, String::from("it has no directory to move the repository into")));
        };
        if let Err(error) = create_private_directory_all(parent) {
            return Err(SaveFailure::io(shown, error.to_string()));
        }
        written.insert(parent.to_path_buf());
        if let Err(error) = std::fs::rename(worktree_root.join(relative), destination.as_path()) {
            return Err(SaveFailure::io(shown, describe_rename_failure(error)));
        }
        copies.push(SavedCopy { path: shown, saved_to: Some(destination) });
    }
    // Deepest first, then the directories that hold the copies' root, so every moved entry is durable.
    let mut ordered: Vec<&PathBuf> = written.iter().collect();
    ordered.sort_by_key(depth_deepest_first);
    for each in ordered {
        if let Err(error) = sync_directory(each.as_path()) {
            return Err(SaveFailure::io(each.display().to_string(), error.to_string()));
        }
    }
    let discarded: &Path = base.parent().unwrap_or(base.as_path());
    for each in [base.as_path(), discarded] {
        if let Err(error) = sync_directory(each) {
            return Err(SaveFailure::io(each.display().to_string(), error.to_string()));
        }
    }
    return Ok(copies);
}

/// What:
///  The system's refusal of one move in words a person can act on.
/// Why:
///   One refusal a person meets has a remedy of its own:
///  a repository on another file system cannot be renamed into the saved copies,
///  and the wrapper does not copy a whole repository.
///  Every other refusal keeps the system's own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function describeRenameFailure(error: Error): string;
/// ```
fn describe_rename_failure(error: std::io::Error) -> String {
    return match error.raw_os_error() {
        Some(libc::EXDEV) => String::from(
            "it is on another file system than the place the wrapper keeps saved copies, and the wrapper does not copy a whole \
             repository",
        ),
        _ => error.to_string(),
    };
}

/// The move,
/// its refusals and the place it keeps.
#[cfg(test)]
#[path = "discarded_nested_repository_tests.rs"]
mod tests;
