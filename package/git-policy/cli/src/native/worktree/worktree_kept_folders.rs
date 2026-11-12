//! What:
//!  The folder a taken worktree folder is kept in when the name it came from is taken,
//! and the cleanup that must
//!  leave such a folder alone.
//! Why:
//!   A worktree folder is taken whole into the trash behind one rename,
//! and the copy that arrives is looked at again
//!  (`worktree_snapshot_placeholders.rs`):
//! a folder that turns out to hold more than the stand-ins it was checked for is never
//!  deleted with them.
//! It goes back to the name it came from,
//! and when a directory has taken that name meanwhile it is kept
//!  under a name of this command's own (`never discard user data`).
//! The move stays inside the snapshot,
//! because a folder that holds a stand-in of a nested subvolume cannot be
//!  renamed out of the snapshot at all
//! (`doc/troubleshooting/btrfs-snapshot-keeps-an-empty-placeholder-for-a-nested-subvolume.md`),
//! so the folder is
//!  kept beside the trash,
//! in the trash parent,
//! where the repository's own exclude line keeps it out of every commit.
//! The stage is destroyed
//!  when the snapshot rung ends without serving the add,
//! so that cleanup asks this module first and leaves a stage that
//!  holds a kept folder in place,
//! with every kept path named (`worktree_ladder_run.rs`,
//! `discard_stage`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const kept = keptFolders(stage); // every <stage>/.claude/cli-git-trash/.cli-git-worktree-kept-*
//! ```

use crate::diagnostic_log::debug;
use crate::fs_beneath::ObjectId;
use crate::worktree_trash::{TRASH_PARENT, Trash};
use std::path::{Path, PathBuf};

/// What:
///  The prefix of a folder this command keeps when the name it came from is taken.
/// Why:
///   The name says which command made the folder,
/// and it is no name the wrapper reads as its own anywhere else:
/// no trash
///  directory is named this way,
/// so the trash's own scan carries it into a later worktree as a person's directory
///  would be.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const KEPT_PREFIX = '.cli-git-worktree-kept-';
/// ```
pub(crate) const KEPT_PREFIX: &str = ".cli-git-worktree-kept-";

/// What:
///  Move the folder the trash holds as `to` out of the trash,
/// under a name of this command's own in the trash parent,
/// and answer where it is now.
/// Why:
///   A folder that changed after it was checked keeps the bytes somebody put in it (`never discard user data`),
/// and its
///  trash entry name can be taken before it goes back,
/// so it cannot stay under the name a manifest would name;
/// one
///  rename inside the snapshot takes it out of that name without crossing a subvolume.
/// `None` means the trash no longer
///  holds the folder this call set aside,
/// so there is nothing of this command's to keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keepTheFolder({ trash, name: 'd3', moved: id, shown }); // '.claude/cli-git-trash/.cli-git-worktree-kept-<digits>'
/// ```
pub(crate) fn keep_the_folder(trash: &Trash, to: &str, moved: ObjectId, shown: &Path) -> Result<Option<PathBuf>, String> {
    // Only the entry this command moved is taken out: one that is not there any more,
    // or one that is another object, belongs to whoever replaced it.
    match trash.held_entry(to, moved) {
        Ok(Some(_the_entry_this_command_moved)) => {}
        Ok(None) => return Ok(None),
        Err(another_object) => {
            debug("worktreeKeptFolders", format!("{another_object}; the entry is left to whoever put it there").as_str());
            return Ok(None);
        }
    }
    let kept: String = kept_name()?;
    trash.keep_beside(to, kept.as_str(), shown)?;
    return Ok(Some(Path::new(TRASH_PARENT).join(kept.as_str())));
}

/// What:
///  The folders this command kept in the trash parent of the stage `stage`,
/// read from the path of the stage.
/// Why:
///   The stage is destroyed when the snapshot rung ends without serving the add,
/// and each of these folders holds bytes this
///  command decided never to delete,
/// so the cleanup asks this first and leaves the stage in place when anything is kept.
/// A trash parent that cannot be listed holds nothing this can name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keptFolders(stage) // every <stage>/.claude/cli-git-trash/.cli-git-worktree-kept-* path
/// ```
pub(crate) fn kept_folders(stage: &Path) -> Vec<PathBuf> {
    let Ok(listed) = std::fs::read_dir(stage.join(TRASH_PARENT)) else { return Vec::new() };
    let mut kept: Vec<PathBuf> = Vec::new();
    for entry in listed.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(KEPT_PREFIX) {
            kept.push(entry.path());
        }
    }
    return kept;
}

/// What:
///  A fresh name for a folder kept beside the trash;
/// the reason is the error.
/// Why:
///   Two commands can keep a folder of one repository in the same instant,
/// and a name nobody can guess keeps one from
///  ever naming the folder another kept;
/// the digits are the object-name alphabet,
/// so the name is one plain component of
///  printable bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const keptName = () => `.cli-git-worktree-kept-${randomHex(32)}`;
/// ```
fn kept_name() -> Result<String, String> {
    let made: String = match crate::random_id::random_uuid() {
        Ok(uuid) => uuid,
        Err(error) => return Err(format!("a name for the kept folder could not be made: {error:?}")),
    };
    let digits: String = made.chars().filter(char::is_ascii_hexdigit).collect();
    return Ok(format!("{KEPT_PREFIX}{}", digits.to_ascii_lowercase()));
}

/// Controls for the folders a changed worktree folder is kept in stay out of the release executable.
#[cfg(test)]
#[path = "worktree_kept_folders_tests.rs"]
mod tests;
