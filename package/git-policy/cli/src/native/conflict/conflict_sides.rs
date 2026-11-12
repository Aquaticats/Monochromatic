//! What:
//!  The saved copy of each file a conflict-side command replaces,
//! and the one place that names the pieces of that carve-out:
//!  the operation and conflicted paths (`conflict_side_state.rs`),
//! the decision on a command line
//!  (`conflict_side_decision.rs`),
//! the notices (`conflict_side_notice.rs`) and the finding and reading of a file
//!  (`conflict_side_source.rs`).
//! Why:
//!  `linked-worktree-only` rejects `git checkout --theirs -- a.rs` because it overwrites the worktree file.
//! During a
//!  merge,
//! a rebase,
//! a cherry-pick or a revert that command is how a conflict is settled,
//! and the file it overwrites is the
//!  half-resolved copy a person may have edited by hand.
//! The owner's decision is to let it through for conflicted paths
//!  after the current file is copied beside the audit log (`discard_audit.rs`),
//! so nothing is lost and the copy is named.
//!  Everything here is read in process:
//! the operation from the state files in the Git directory,
//! the conflicted paths from
//!  the unmerged entries of the index file.
//! The copy is written and synced before the caller lets Git run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const conflicted = unmergedPaths(await readIndex(gitDir));
//! const decision = decideSides({ pathspecs, prefix, conflicted });
//! if (decision.kind === 'allow') await saveCopies({ root, commonDir, paths: decision.paths });
//! ```

use crate::conflict_side_source::{copy_located, locate};
use crate::iso_time::format_iso_milliseconds;
use crate::private_storage::{create_private_directory, create_private_directory_all, sync_directory};
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};

pub use crate::conflict_side_decision::{SideDecision, decide_sides, literal_path};
pub use crate::conflict_side_notice::{clean_paths_note, saved_notice};
pub use crate::conflict_side_source::{FailureKind, SaveFailure};
pub use crate::conflict_side_state::{Operation, conflicted_paths, operation_in_progress};

/// Where the copies live,
/// below the common Git directory,
/// beside `monochromatic-git-policy-cli-audit.jsonl`.
pub const SAVED_DIRECTORY: &str = "monochromatic-git-policy-cli/discarded";

/// One saved file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SavedCopy {
    /// The repository-relative path.
    pub path: String,
    /// Where the copy is,
    /// or nothing when the path is deleted in the worktree and has nothing to copy.
    pub saved_to: Option<PathBuf>,
}

/// The name of a directory for copies made at `milliseconds`:
/// `2026-11-02T14-01-07`.
fn stamp(milliseconds: i64) -> String {
    let mut name: String = String::new();
    for each in format_iso_milliseconds(milliseconds).chars().take(19) {
        name.push(if each == ':' { '-' } else { each });
    }
    return name;
}

/// The pause between chunks of a copy when nothing needs to happen between them.
fn no_pause() {}

/// The sort key that puts the deepest directory first.
pub(crate) fn depth_deepest_first(path: &&PathBuf) -> std::cmp::Reverse<usize> {
    return std::cmp::Reverse(path.components().count());
}

/// Create the directory for this invocation's copies,
/// never one that exists:
/// a second invocation in the same second
///  gets a numbered sibling,
/// so no copy overwrites another.
fn fresh_directory(saved_root: &Path, name: &str) -> std::io::Result<PathBuf> {
    create_private_directory_all(saved_root)?;
    for number in 0_u32..1000 {
        let candidate: PathBuf = if number == 0 { saved_root.join(name) } else { saved_root.join(format!("{name}-{number}")) };
        match create_private_directory(candidate.as_path()) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    return Err(std::io::Error::other("a thousand copies were saved in this second"));
}

/// What:
///  A fresh directory for the copies one invocation saves,
///  named for `milliseconds`.
/// Why:
///  The conflicted files and the nested repositories a command saves go under the same folder,
/// so both name a place beside the audit log the same way and both leave the age and size limits one folder to prune.
///  A directory that already exists is never reused,
///  so no copy overwrites another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function freshSavedDirectory(commonDir: string, milliseconds: number): string; // throws
/// ```
pub(crate) fn fresh_saved_directory(common_dir: &Path, milliseconds: i64) -> std::io::Result<PathBuf> {
    let saved_root: PathBuf = common_dir.join(SAVED_DIRECTORY);
    return fresh_directory(saved_root.as_path(), stamp(milliseconds).as_str());
}

/// What:
///  Save the current worktree file of each path below the common Git directory and sync everything written.
/// Why:
///   The command that follows overwrites these files,
/// so each is copied first with its content,
/// its permission bits or
///  its symbolic-link target.
/// A path with no file in the worktree has nothing to copy and is reported as such.
/// The
///  first failure stops the work and is returned,
/// so the caller refuses the command instead of discarding unsaved work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const copies = await saveCopies({ root, commonDir, paths, milliseconds });
/// ```
pub fn save_copies(root: &Path, common_dir: &Path, paths: &[Vec<u8>], milliseconds: i64) -> Result<Vec<SavedCopy>, SaveFailure> {
    let saved_root: PathBuf = common_dir.join(SAVED_DIRECTORY);
    let mut copies: Vec<SavedCopy> = Vec::new();
    let mut directory: Option<PathBuf> = None;
    let mut written_directories: BTreeSet<PathBuf> = BTreeSet::new();
    for path in paths {
        let shown: String = String::from_utf8_lossy(path).into_owned();
        let Some(located) = locate(root, path.as_slice())? else {
            copies.push(SavedCopy { path: shown, saved_to: None });
            continue;
        };
        let base: PathBuf = match &directory {
            Some(existing) => existing.clone(),
            None => {
                let created: PathBuf = match fresh_directory(saved_root.as_path(), stamp(milliseconds).as_str()) {
                    Ok(made) => made,
                    Err(error) => return Err(SaveFailure::io(saved_root.display().to_string(), error.to_string())),
                };
                directory = Some(created.clone());
                created
            }
        };
        let destination: PathBuf = base.join(OsString::from_vec(path.clone()));
        if let Some(parent) = destination.parent() {
            if let Err(error) = create_private_directory_all(parent) {
                return Err(SaveFailure::io(shown, error.to_string()));
            }
            let mut up: Option<&Path> = Some(parent);
            while let Some(each) = up {
                written_directories.insert(each.to_path_buf());
                if each == base.as_path() {
                    break;
                }
                up = each.parent();
            }
        }
        copy_located(&located, destination.as_path(), &mut no_pause)?;
        copies.push(SavedCopy { path: shown, saved_to: Some(destination) });
    }
    if directory.is_some() {
        // Deepest first, then the directories that hold the copies' root, so every new entry is durable.
        let mut ordered: Vec<&PathBuf> = written_directories.iter().collect();
        ordered.sort_by_key(depth_deepest_first);
        for each in ordered {
            if let Err(error) = sync_directory(each.as_path()) {
                return Err(SaveFailure::io(each.display().to_string(), error.to_string()));
            }
        }
        let parent: PathBuf = common_dir.join("monochromatic-git-policy-cli");
        for each in [saved_root.as_path(), parent.as_path()] {
            if let Err(error) = sync_directory(each) {
                return Err(SaveFailure::io(each.display().to_string(), error.to_string()));
            }
        }
    }
    return Ok(copies);
}

/// The decision,
/// the literal paths,
/// the copies and the notices.
#[cfg(test)]
#[path = "conflict_sides_tests.rs"]
mod tests;
