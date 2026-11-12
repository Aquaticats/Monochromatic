//! What:
//!  The owner's writing on the staged directories above one entry,
//! for the moment the settling removes that entry and copies it
//!  again,
//! and the modes those directories had before.
//! Why:
//!  The settling applies the source's modes to the stage before it compares,
//! so a changed entry sits below a directory
//!  whose mode may grant reading and searching and no writing,
//! and removing the staged copy of that entry needs the writing its
//!  owner withheld.
//! The stage is this copy's own folder and nothing else writes in it,
//! so the widening is one `chmod` per
//!  directory,
//! and the recorded mode goes back before the next comparison reads it:
//! a stage left wider than its source
//!  would be installed with modes the source never had.
//! A directory the source removed meanwhile is already gone and needs nothing;
//! a
//!  directory whose mode the owner may not change keeps its mode and the removal reports what the system answered,
//! as
//!  it does everywhere else in this folder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const opened = openTheStagedDirectories(stageRoot, 'cache/a.txt'); // chmod cache 0700, recording 0500
//! rmSync(join(stageRoot, 'cache/a.txt')); copyOneLeaf(/* ... */); opened.putTheModesBack(); // chmod cache 0500
//! ```

use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_selection::{filesystem_path, is_separator_byte};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// The owner's read,
/// write and search bits,
/// which a directory needs before an entry inside it can go or be made.
const OWNER_ACCESS: u32 = 0o700;

/// What:
///  One staged directory whose mode was widened for a removal or a copy,
/// with the mode it had.
/// Why:
///   The mode is put back by path once the step is done,
/// and the list holds the directories deepest last,
///  so putting them back in reverse leaves every parent as writable as the child expects until the child is done.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Widened = { path: string; mode: number };
/// ```
struct Widened {
    /// The staged directory.
    path: PathBuf,
    /// The permission bits it had before the widening.
    mode: u32,
}

/// What:
///  The staged directories one step may write in,
/// each carrying the mode it had before this value widened it.
/// Why:
///   The caller widens once,
/// does the removal and the copy it needs,
/// and puts every mode back;
/// a directory already wide
///  enough is not recorded and not touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OpenedForWriting = { widened: Widened[] };
/// ```
pub struct OpenedForWriting {
    /// The directories this value widened,
    /// in the order they were walked.
    widened: Vec<Widened>,
}

/// What:
///  Give the owner writing on every staged directory above one entry,
/// and on the entry itself when it is a directory a
///  removal takes whole,
/// recording each mode this changes.
/// Why:
///   The entry's own directory has to be writable too before its children can go;
/// a path that is not there is
///  the removal's own answer and nothing to widen,
/// and a name that is not a directory is no directory to widen.
/// A refusal part-way puts back what this call already widened,
/// so a stage is never left wider than its source by
///  a call that failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const components = repositoryPath.split('/'); const opened = []; for (const prefix of prefixesIncludingTheEntry) { const path = join(stageRoot, prefix); const stat = lstat(path) ?? continue; if (!stat.isDirectory()) continue; if ((stat.mode & 0o700) === 0o700) continue; chmod(path, stat.mode | 0o700); opened.push({ path, mode: stat.mode & 0o7777 }); }
/// ```
///
/// # Errors
///  When a staged directory's mode cannot be read or changed,
///  as every other refusal of the copy's own folder is reported.
pub fn open_the_staged_directories(
    stage_root: &Path,
    repository_path: &[u8],
) -> Result<OpenedForWriting, WorktreeCopyError> {
    let mut opened: OpenedForWriting = OpenedForWriting { widened: Vec::new() };
    let mut prefix: Vec<u8> = Vec::new();
    for component in repository_path.split(is_separator_byte) {
        if !prefix.is_empty() {
            prefix.push(b'/');
        }
        prefix.extend_from_slice(component);
        if let Err(error) = opened.widen_one(stage_root, prefix.as_slice()) {
            // The directories this call widened go back before the refusal travels, so a failed call leaves no mode wider than the source's.
            let _put_back: Result<(), WorktreeCopyError> = opened.put_the_modes_back();
            return Err(error);
        }
    }
    return Ok(opened);
}

impl OpenedForWriting {
    /// What:
    ///  Widen one staged directory,
    /// this value's own refusals included,
    /// and remember the mode it had.
    /// Why:
    ///   A path that is not there yet is the step's own answer and no refusal of the widening;
    /// every other answer about the
    ///  name is one,
    /// because the step that follows would fail with the system's words and no path.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const stat = lstat(path); if (stat === undefined) return; if (!stat.isDirectory()) return; if ((stat.mode & 0o700) === 0o700) return; chmod(path, stat.mode | 0o700); widened.push({ path, mode: stat.mode & 0o7777 });
    /// ```
    fn widen_one(&mut self, stage_root: &Path, prefix: &[u8]) -> Result<(), WorktreeCopyError> {
        let path: PathBuf = filesystem_path(stage_root, prefix)?;
        let metadata: std::fs::Metadata = match std::fs::symlink_metadata(path.as_path()) {
            Ok(found) => found,
            // A directory the source removed meanwhile, or a name nothing holds yet, leaves nothing to widen; the step that follows reports what it finds.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(crate::worktree_copy_error::raw_failure(&error)),
        };
        if !metadata.is_dir() {
            return Ok(());
        }
        let mode: u32 = metadata.permissions().mode() & 0o7777;
        if mode & OWNER_ACCESS == OWNER_ACCESS {
            return Ok(());
        }
        std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(mode | OWNER_ACCESS)).map_err(as_a_copy_failure)?;
        self.widened.push(Widened { path, mode });
        return Ok(());
    }

    /// What:
    ///  Put every recorded mode back,
    /// deepest first,
    /// and answer the first refusal after every mode has been tried.
    /// Why:
    ///   The stage must hold the source's modes again before the next comparison reads them,
    /// so one refusal does not
    ///  leave the other directories wider than their source;
    /// a directory the source removed meanwhile has no mode to put
    ///  back.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// let first = undefined; for (const { path, mode } of widened.toReversed()) { try { chmod(path, mode) } catch (e) { if (e.code !== 'ENOENT' && first === undefined) first = e } } if (first !== undefined) throw first;
    /// ```
    ///
    /// # Errors
    ///  When a recorded mode cannot be put back.
    pub fn put_the_modes_back(self) -> Result<(), WorktreeCopyError> {
        let mut refusal: Option<WorktreeCopyError> = None;
        for widened in self.widened.iter().rev() {
            match std::fs::set_permissions(widened.path.as_path(), std::fs::Permissions::from_mode(widened.mode)) {
                Ok(()) => {}
                // A directory the source removed while the step ran holds no mode to put back, and the copy answers for it on its next pass.
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    if refusal.is_none() {
                        refusal = Some(crate::worktree_copy_error::raw_failure(&error));
                    }
                }
            }
        }
        return match refusal {
            Some(error) => Err(error),
            None => Ok(()),
        };
    }
}

/// What:
///  One refusal of a staged directory's own mode,
/// as the copy's folder words it everywhere else.
/// Why:
///   The widening and its restoration both answer in the same type as the step they serve,
/// so a refusal of
///  either reaches the caller as one error and never as a bare operating-system answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const asACopyFailure = (error) => new WorktreeCopyError(error.message);
/// ```
fn as_a_copy_failure(error: std::io::Error) -> WorktreeCopyError {
    return crate::worktree_copy_error::raw_failure(&error);
}

/// What:
///  Give the owner writing on every directory of one staged tree that a removal takes whole,
/// and record nothing
///  to put back.
/// Why:
///   `remove_dir_all` cannot unlink an entry inside a directory of mode 0500,
/// so a staged directory whose own
///  tree holds such a directory fails the removal that takes the entry with it;
/// the whole tree is going,
///  so there is no mode to restore and the widening is the removal's alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const directory of directoriesBelow(root)) chmod(directory, (stat(directory).mode & 0o7777) | 0o700);
/// ```
///
/// # Errors
///  When a directory below the entry cannot be read or given its owner's writing.
pub fn open_the_staged_tree_for_removal(root: &Path) -> Result<(), WorktreeCopyError> {
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        match std::fs::symlink_metadata(directory.as_path()) {
            Ok(metadata) if metadata.is_dir() => {
                let mode: u32 = metadata.permissions().mode() & 0o7777;
                if mode & OWNER_ACCESS != OWNER_ACCESS {
                    std::fs::set_permissions(directory.as_path(), std::fs::Permissions::from_mode(mode | OWNER_ACCESS)).map_err(as_a_copy_failure)?;
                }
            }
            // A directory another step removed at the same moment holds nothing to widen.
            Ok(_) => continue,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(crate::worktree_copy_error::raw_failure(&error)),
        }
        let listing: std::fs::ReadDir = match std::fs::read_dir(directory.as_path()) {
            Ok(listing) => listing,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(crate::worktree_copy_error::raw_failure(&error)),
        };
        for listed in listing {
            let entry: std::fs::DirEntry = match listed {
                Ok(found) => found,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(crate::worktree_copy_error::raw_failure(&error)),
            };
            // Only directories below the entry need the owner's writing; a file inside them is unlinked by the removal that holds the directory it sits in.
            match entry.file_type() {
                Ok(kind) if kind.is_dir() => pending.push(entry.path()),
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(crate::worktree_copy_error::raw_failure(&error)),
            }
        }
    }
    return Ok(());
}
