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

use crate::conflict_side_source::{Stamp, WhenTheSourceChanges, copy_located, locate};
use crate::iso_time::format_iso_milliseconds;
use crate::private_storage::{create_private_directory, create_private_directory_all, sync_directory};
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};

pub use crate::conflict_side_decision::{SideDecision, decide_sides, literal_path};
pub use crate::conflict_side_notice::{clean_paths_note, saved_notice, still_being_written_note};
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
/// A source that another program is still writing is copied up to the size it had when the copy opened it,
///  that copy is kept,
///  and the caller says so:
///  a writer that never stops must not hold a command the person asked for,
///  and a torn copy that passes for the whole file would be worse than none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const saved = saveCopies({ root, commonDir, paths, milliseconds });
/// ```
pub fn save_copies(root: &Path, common_dir: &Path, paths: &[Vec<u8>], milliseconds: i64) -> Result<SavedWithWriters, SaveFailure> {
    return save_copies_watching(root, common_dir, paths, milliseconds, &mut no_pause);
}

/// What:
///  What one conflict-side save made of the paths it was given.
/// Why:
///   The notice names every copy,
///  and the paths another program was writing while the copy was made are named apart,
///  because their copy holds what stood when it opened the file and not what stood after.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SavedWithWriters = { copies: SavedCopy[]; stillBeingWritten: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SavedWithWriters {
    /// One entry per path that was saved,
    ///  in the order the paths were given.
    pub copies: Vec<SavedCopy>,
    /// The paths whose source another program was writing while the copy was made.
    pub still_being_written: Vec<String>,
}

/// What:
///  `save_copies`,
///  running `between` after each chunk of each copy.
/// Why:
///   The copy loop's own pause is where a control makes a source grow while it is being copied,
///  which is the case this save answers without waiting for the writer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const saved = saveCopiesWatching({ root, commonDir, paths, milliseconds, between });
/// ```
pub fn save_copies_watching(
    root: &Path,
    common_dir: &Path,
    paths: &[Vec<u8>],
    milliseconds: i64,
    between: &mut dyn FnMut(),
) -> Result<SavedWithWriters, SaveFailure> {
    let directory: PathBuf = match fresh_saved_directory(common_dir, milliseconds) {
        Ok(made) => made,
        Err(error) => return Err(SaveFailure::io(common_dir.join(SAVED_DIRECTORY).display().to_string(), error.to_string())),
    };
    let attempt: SaveAttempt = save_into_watching(root, directory.as_path(), paths, between, WhenTheSourceChanges::Keep)?;
    // The copy of a source that moved under it is kept, and the notice names each of them as one whose copy holds
    //  what stood when it was opened.
    let still_being_written: Vec<String> = attempt.changed.iter().map(|failure| { return failure.subject.clone(); }).collect();
    return Ok(SavedWithWriters { copies: attempt.copies, still_being_written });
}

/// What:
///  One save step's answer:
///  the copies it made,
///  and the sources that changed while they were copied.
/// Why:
///   A caller that only copies once must refuse such a source,
///  while a caller that may copy it again
///  needs to know which one it was and where it was read from.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaveAttempt {
    /// One entry per path that was saved,
    ///  in the order the paths were given.
    pub copies: Vec<SavedCopy>,
    /// One entry per path that was asked about,
    ///  in the same order,
    ///  saying what the worktree held there when it was read.
    pub sources: Vec<SavedSource>,
    /// The sources that changed while they were copied,
    ///  with their reasons,
    ///  in the order the paths were given.
    pub changed: Vec<SaveFailure>,
}

/// What:
///  One source the save read, and what stood there while the copy was made.
/// Why:
///   A source that changed again is compared with this,
///  and a path with nothing to copy is a source too:
///  a file that appears there before Git runs is work the command would discard.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SavedSource = { path: Buffer; stamp?: Stamp };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SavedSource {
    /// The repository-relative path,
    ///  in the bytes Git printed.
    pub path: Vec<u8>,
    /// What the entry at that path was when the copy was made,
    ///  or nothing when there was nothing to copy.
    pub stamp: Option<Stamp>,
}

/// What:
///  Save each path and report which source changed while it was copied,
///  without refusing anything.
/// Why:
///   The abort family may copy a changed source again,
///  so the reading and the refusal are separated:
///  this is the reading,
///  and `save_copies` is the refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attempt = saveCopiesReadingChanges({ root, commonDir, paths, milliseconds });
/// ```
pub fn save_copies_reading_changes(root: &Path, common_dir: &Path, paths: &[Vec<u8>], milliseconds: i64) -> Result<SaveAttempt, SaveFailure> {
    let directory: PathBuf = match fresh_saved_directory(common_dir, milliseconds) {
        Ok(made) => made,
        Err(error) => return Err(SaveFailure::io(common_dir.join(SAVED_DIRECTORY).display().to_string(), error.to_string())),
    };
    return save_into(root, directory.as_path(), paths);
}

/// What:
///  Save each path below `directory`, which the caller made,
///  and report which source changed while it was copied.
/// Why:
///   A caller that copies the same paths again keeps every copy it made:
///  each round writes into a folder of its own,
///  so a second copy never stands where the first one is,
///  and the person is told one folder that holds them all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attempt = saveInto({ directory, root, paths });
/// ```
pub fn save_into(root: &Path, directory: &Path, paths: &[Vec<u8>]) -> Result<SaveAttempt, SaveFailure> {
    return save_into_watching(root, directory, paths, &mut no_pause, WhenTheSourceChanges::Remove);
}

/// What:
///  `save_into`,
///  running `between` after each chunk of each copy.
/// Why:
///   A control that needs a source to change while it is being copied cannot time a writer;
///  the copy loop's own pause is where such a change belongs,
///  and the source module already offers the same pause to its own controls.
///  A shipped save passes a pause that does nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attempt = saveIntoWatching({ root, directory, paths, between });
/// ```
pub fn save_into_watching(
    root: &Path,
    directory: &Path,
    paths: &[Vec<u8>],
    between: &mut dyn FnMut(),
    when_the_source_changes: WhenTheSourceChanges,
) -> Result<SaveAttempt, SaveFailure> {
    let mut copies: Vec<SavedCopy> = Vec::new();
    let mut sources: Vec<SavedSource> = Vec::new();
    let mut changed: Vec<SaveFailure> = Vec::<SaveFailure>::new();
    let mut written_directories: BTreeSet<PathBuf> = BTreeSet::new();
    if let Err(error) = create_private_directory_all(directory) {
        return Err(SaveFailure::io(directory.display().to_string(), error.to_string()));
    }
    for path in paths {
        let shown: String = String::from_utf8_lossy(path).into_owned();
        let Some(located) = locate(root, path.as_slice())? else {
            sources.push(SavedSource { path: path.clone(), stamp: None });
            copies.push(SavedCopy { path: shown, saved_to: None });
            continue;
        };
        let destination: PathBuf = directory.join(OsString::from_vec(path.clone()));
        if let Some(parent) = destination.parent() {
            if let Err(error) = create_private_directory_all(parent) {
                return Err(SaveFailure::io(shown, error.to_string()));
            }
            let mut up: Option<&Path> = Some(parent);
            while let Some(each) = up {
                written_directories.insert(each.to_path_buf());
                if each == directory {
                    break;
                }
                up = each.parent();
            }
        }
        // A source that changed under the copy is reported, not refused here: an abort copies it again,
        //  and the copy the changed read left is already removed.
        match copy_located(&located, destination.as_path(), between, when_the_source_changes) {
            Ok(stamp) => {
                sources.push(SavedSource { path: path.clone(), stamp: Some(stamp) });
                copies.push(SavedCopy { path: shown, saved_to: Some(destination) });
            }
            Err(failure) if failure.kind == FailureKind::Changed => {
                // A caller that keeps the bytes the source held names its copy and hears that the source moved on;
                //  one that removes them leaves the path without a copy to point at.
                if when_the_source_changes == WhenTheSourceChanges::Keep {
                    sources.push(SavedSource { path: path.clone(), stamp: None });
                    copies.push(SavedCopy { path: shown, saved_to: Some(destination) });
                }
                changed.push(failure);
            }
            Err(failure) => return Err(failure),
        }
    }
    // Deepest first, then the directories that hold the copies' root, so every new entry is durable.
    let mut ordered: Vec<&PathBuf> = written_directories.iter().collect();
    ordered.sort_by_key(depth_deepest_first);
    for each in ordered {
        if let Err(error) = sync_directory(each.as_path()) {
            return Err(SaveFailure::io(each.display().to_string(), error.to_string()));
        }
    }
    for each in [directory] {
        if let Err(error) = sync_directory(each) {
            return Err(SaveFailure::io(each.display().to_string(), error.to_string()));
        }
    }
    return Ok(SaveAttempt { copies, sources, changed });
}

/// The decision,
/// the literal paths,
/// the copies and the notices.
#[cfg(test)]
#[path = "conflict_sides_tests.rs"]
mod tests;
