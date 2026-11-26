//! What:
//!  The marker one command takes in the journal root before it stages an owed copy,
//!  the removal of the markers whose owners no longer run,
//!  and the name order the markers are judged in.
//! Why:
//!  Two commands that find the same dead add's claim would both stage into one name,
//! and one of them would remove the other's live stage.
//!  The marker is one exclusive create whose name carries its owner,
//! so exactly one command copies,
//! a marker whose owner died is taken over rather than left waiting,
//! and a marker a killed command left is swept by the next pass.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await removeDeadTakingMarks(root);
//! const taking = await takeTheCopy(root, claim); // undefined when another command holds it
//! ```

use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_owner::{CopyOwner, current_copy_owner, token_owner_is_dead};
use std::path::{Path, PathBuf};

/// What:
///  The marker one command writes in the journal root before it stages a copy,
/// and removes when it is done with that destination.
/// Why:
///   Two commands that find the same dead add's claim would both stage into one name and one of them would remove the
/// other's live stage;
/// the marker is one exclusive create whose name carries its owner,
/// so exactly one command copies and a marker
///  whose owner died is taken over rather than left waiting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using taking = await takeTheCopy(root, claim); // undefined when another command holds it
/// ```
pub(crate) struct Taking {
    /// The marker file this command created.
    path: PathBuf,
}

impl Drop for Taking {
    /// Removes the marker however this command ends,
    /// so a copy that fails leaves no claim that another command holds it.
    fn drop(&mut self) {
        let _removed: std::io::Result<()> = std::fs::remove_file(self.path.as_path());
    }
}

/// The suffix of that marker.
const TAKING_SUFFIX: &str = ".taking";

/// What:
///  The marks one add's identifier has been given,
/// in name order.
/// Why:
///   A command that finds a dead owner's marker removes it and takes the copy;
/// one that finds a live owner's leaves the copy to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const marks = takingMarksOf(root, id);
/// ```
fn taking_marks_of(root: &Path, identifier: &str) -> Result<Vec<PathBuf>, WorktreeCopyError> {
    let listing = match std::fs::read_dir(root) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(crate::worktree_copy_error::raw_failure(&error)),
    };
    let prefix: String = format!("{identifier}.");
    let mut marks: Vec<PathBuf> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    for listed in listing {
        let name: String = listed.map_err(crate::worktree_copy_error::raw_failure_of)?.file_name().to_string_lossy().into_owned();
        if name.starts_with(prefix.as_str()) && name.ends_with(TAKING_SUFFIX) {
            names.push(name);
        }
    }
    names.sort_by(|left, right| return by_mark_name(left, right));
    for name in names {
        marks.push(root.join(name.as_str()));
    }
    return Ok(marks);
}

/// What:
///  Order two mark names the way JavaScript's default sort does.
/// Why:
///   Which marker is judged first must not depend on the directory's order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// marks.sort(([left], [right]) => compareJs(left, right))
/// ```
fn by_mark_name(left: &str, right: &str) -> std::cmp::Ordering {
    return crate::worktree_copy_selection::js_order(left, right);
}

/// What:
///  Takes the copy of one add's destination,
/// or answers that another command holds it.
/// Why:
///   The take is one exclusive create whose name carries this command's owner,
/// so of two commands that find the same dead
///  claim exactly one copies;
/// a marker whose owner died is taken over,
/// because nothing of that command runs and
///  the worktree it would have filled is still unfilled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taking = await takeTheCopy(root, claim);
/// ```
pub(crate) fn take_the_copy(root: &Path, claim: &Path) -> Result<Option<Taking>, WorktreeCopyError> {
    let Some(stem) = claim.file_stem() else { return Ok(None) };
    let identifier: String = stem.to_string_lossy().into_owned();
    let owner: CopyOwner = current_copy_owner()?;
    // A marker whose owner is proven dead holds nothing, so it is removed; one whose owner lives keeps the copy.
    let mut held: bool = false;
    for mark in taking_marks_of(root, identifier.as_str())? {
        let token: Option<String> = match std::fs::read_to_string(mark.as_path()) {
            Ok(text) => Some(text.trim().to_string()),
            Err(_unreadable) => None,
        };
        if !token_owner_is_dead(token.as_deref(), mark.as_path())? {
            held = true;
            break;
        }
        // The name carries the owner, and the file is a marker of this namespace alone.
        let _removed: std::io::Result<()> = std::fs::remove_file(mark.as_path());
    }
    if held {
        return Ok(None);
    }
    let path: PathBuf = root.join(format!("{identifier}.{}{TAKING_SUFFIX}", owner.token()));
    // `match` on the exclusive create: the one command that made the file copies, and the rest read it as another's, so no
    // second attempt is needed, not even when two commands removed the same dead marker.
    return match std::fs::OpenOptions::new().write(true).create_new(true).open(path.as_path()) {
        Ok(mut written) => {
            use std::io::Write as _;
            written.write_all(owner.token().as_bytes()).map_err(crate::worktree_copy_error::raw_failure_of)?;
            written.sync_all().map_err(crate::worktree_copy_error::raw_failure_of)?;
            Ok(Some(Taking { path }))
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(None),
        Err(error) => Err(crate::worktree_copy_error::raw_failure(&error)),
    };
}

/// What:
///  Removes the taking markers of commands that no longer run.
/// Why:
///   A marker holds the copy of one destination back for as long as its owner lives,
/// and a command killed after making one
///  leaves it behind;
/// removing it loses nothing,
/// because the marker says only which command was copying,
/// and its
///  name carries the owner that is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeDeadTakingMarks(root);
/// ```
pub(crate) fn remove_dead_taking_marks(root: &Path) -> Result<(), WorktreeCopyError> {
    let listing = match std::fs::read_dir(root) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(crate::worktree_copy_error::raw_failure(&error)),
    };
    for listed in listing {
        let path: PathBuf = listed.map_err(crate::worktree_copy_error::raw_failure_of)?.path();
        let name: String = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
        if !name.ends_with(TAKING_SUFFIX) {
            continue;
        }
        // The name is `<identifier>.<owner>.taking`, and the owner field is the one before the suffix.
        let token: Option<String> = match name.strip_suffix(TAKING_SUFFIX) {
            Some(stem) => stem.rsplit_once('.').map(|(_item, owner)| { return owner.to_string(); }),
            None => None,
        };
        if token_owner_is_dead(token.as_deref(), path.as_path())? {
            let _removed: std::io::Result<()> = std::fs::remove_file(path.as_path());
        }
    }
    return Ok(());
}
