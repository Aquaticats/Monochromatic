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
        let path: PathBuf = filesystem_path(stage_root, prefix.as_slice())?;
        // A directory the source removed meanwhile, or a name that is no directory, leaves nothing to widen; the step that follows reports what it finds.
        let Ok(metadata) = std::fs::symlink_metadata(path.as_path()) else {
            continue;
        };
        if !metadata.is_dir() {
            continue;
        }
        let mode: u32 = metadata.permissions().mode() & 0o7777;
        if mode & OWNER_ACCESS == OWNER_ACCESS {
            continue;
        }
        let widened: std::fs::Permissions = std::fs::Permissions::from_mode(mode | OWNER_ACCESS);
        if let Err(error) = std::fs::set_permissions(path.as_path(), widened) {
            return Err(crate::worktree_copy_error::raw_failure(&error));
        }
        opened.widened.push(Widened { path, mode });
    }
    return Ok(opened);
}

impl OpenedForWriting {
    /// What:
    ///  Put every recorded mode back,
    /// deepest first,
    /// and answer what the first refusal was.
    /// Why:
    ///   The stage must hold the source's modes again before the next comparison reads them,
    /// and a directory the
    ///  source removed meanwhile has no mode to put back.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// for (const { path, mode } of widened.toReversed()) { try { chmod(path, mode) } catch (e) { if (e.code !== 'ENOENT') return failure(e) } } return ok;
    /// ```
    ///
    /// # Errors
    ///  When a recorded mode cannot be put back.
    pub fn put_the_modes_back(self) -> Result<(), WorktreeCopyError> {
        for widened in self.widened.iter().rev() {
            match std::fs::set_permissions(widened.path.as_path(), std::fs::Permissions::from_mode(widened.mode)) {
                Ok(()) => {}
                // A directory the source removed while the step ran holds no mode to put back, and the copy answers for it on its next pass.
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(crate::worktree_copy_error::raw_failure(&error)),
            }
        }
        return Ok(());
    }
}
