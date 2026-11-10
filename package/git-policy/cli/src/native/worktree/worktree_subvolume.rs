//! What:
//!  The empty btrfs subvolume the wrapper makes at the destination of a `git worktree add`,
//! before real Git runs,
//! and
//!  the cleanup of it when Git leaves it empty.
//! Why:
//!  A worktree that is a subvolume is a source of a later snapshot with no migration,
//! and it is removed in one
//!  call.
//! Git accepts an existing empty directory as the destination and fills it,
//! and an empty subvolume is an
//!  empty directory (`doc/planning/cli-git-btrfs-worktree.md`,
//! "Creating new worktrees as subvolumes").
//! Git removes the
//!  destination again only when it created it and failed late;
//! a failure before that point (a branch that exists,
//! a
//!  name Git rejects) leaves the directory,
//! so the wrapper removes the empty object it made itself.
//! An empty folder inside a repository is a trap and not a harmless leftover:
//! a Git command run in it acts on the
//!  enclosing checkout as if that were the new worktree.
//! A destination that is
//!  not on btrfs,
//! or a subvolume the kernel refuses,
//! leaves Git to make its ordinary directory without a word to the
//!  person,
//! with one diagnostic line for the log.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const made = await prepareSubvolumeDestination(plan); // undefined: Git makes the directory
//! await runGit();
//! if (made) await settleSubvolumeDestination(made); // rmdir when still empty
//! ```

use crate::diagnostic_log::{debug, warn};
use crate::fs_beneath::{ObjectId, handle_path, identity_at, identity_of, open_directory_at, remove_at};
use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::fs_kind::{FilesystemKind, filesystem_kind};
use crate::worktree_add_plan::AddPlan;
use std::fs::File;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeSubvolume";

/// What:
///  An empty subvolume this process created at a destination.
/// Why:
///   Only a destination the wrapper created is the wrapper's to remove again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CreatedSubvolume = { path: string; identity?: ObjectId };
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct CreatedSubvolume {
    /// The destination,
    /// an empty subvolume when it was created.
    pub path: PathBuf,
    /// What the created subvolume was when it was looked at right after its creation:
    /// a path proves nothing about which
    ///  object it names later.
    /// Absent when it could not be looked at,
    /// in which case settlement leaves it alone.
    pub identity: Option<ObjectId>,
}

/// What:
///  Create the destination of a planned `git worktree add` as an empty subvolume,
/// when it can be one.
///  `Option<CreatedSubvolume>` is the created subvolume or nothing.
/// Why:
///   The destination must be absent (an existing one is Git's to accept or refuse) and its parent must be on btrfs.
/// The
///  creation ioctl is unrestricted for a regular user,
/// so a refusal is a fault the log keeps and Git's ordinary directory
///  replaces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function prepareSubvolumeDestination(plan: AddPlan): Promise<CreatedSubvolume | undefined>;
/// ```
pub fn prepare_subvolume_destination(plan: &AddPlan) -> Option<CreatedSubvolume> {
    let destination: &Path = plan.destination.as_path();
    if std::fs::symlink_metadata(destination).is_ok() {
        debug(TAG, format!("{} exists already; Git decides whether it is acceptable", destination.display()).as_str());
        return None;
    }
    let parent: &Path = destination.parent()?;
    match filesystem_kind(parent) {
        Ok(FilesystemKind::Btrfs) => {}
        Ok(_) => return None,
        Err(error) => {
            debug(TAG, format!("the filesystem of {} is unknown ({error}); Git makes the directory", parent.display()).as_str());
            return None;
        }
    }
    if let Err(error) = subvolume_create_at(destination) {
        debug(TAG, format!("no subvolume at {} ({error}); Git makes the directory", destination.display()).as_str());
        return None;
    }
    debug(TAG, format!("created the empty subvolume {}", destination.display()).as_str());
    let read: std::io::Result<ObjectId> = match std::fs::File::open(parent) {
        Ok(held) => identity_at(&held, destination.file_name().unwrap_or_default()),
        Err(error) => Err(error),
    };
    let identity: Option<ObjectId> = match read {
        Ok(found) => Some(found),
        Err(error) => {
            debug(TAG, format!("the identity of the created subvolume {} cannot be read ({error}); settlement will leave it alone", destination.display()).as_str());
            None
        }
    };
    return Some(CreatedSubvolume { path: destination.to_path_buf(), identity });
}

/// What:
///  Who owns the destination of a creating command,
/// decided once,
/// before real Git runs.
/// Why:
///   A `git worktree add` that finds its destination unregistered cannot tell,
/// after its own Git
///  returned,
/// whether the registration there is the one its Git made:
/// a second add at the same path
///  leaves its registration exactly where this one's would be,
/// and Git accepts an empty directory any
///  command prepared.
/// One exclusive creation of the directory decides it before any Git runs:
/// the
///  command that made the directory owns the destination,
/// every other command finds it there and
///  copies nothing,
/// and Git's own status then says nothing about ownership.
/// Nothing waits and no lock
///  exists:
/// the creation either succeeds or finds the directory,
/// and a loser never removes what it did not
///  make.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reservation = { owned: CreatedDestination } | 'another commands' | 'not attempted';
/// ```
#[derive(Debug, Eq, PartialEq)]
pub enum Reservation {
    /// The destination was not there and this command made it.
    Mine(CreatedSubvolume),
    /// The destination was there already,
    /// so another command made it;
    /// the identity is what stood there when the reservation was
    ///  made,
    /// and a different object at the path later is a replacement.
    Anothers(Option<ObjectId>),
    /// Nothing was reserved:
    /// the command copies nothing,
    /// or the path could not be made.
    NotAttempted,
}

/// What:
///  The identity of the object at `path` now,
/// or nothing when it cannot be looked at.
/// Why:
///   A destination that stood there before this command's Git is another command's,
/// and the object that stood
///  there is what a later replacement is told apart from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function identityOfEntry(path: string): ObjectId | undefined;
/// ```
fn identity_of_entry(path: &Path) -> Option<ObjectId> {
    let parent: File = std::fs::File::open(path.parent()?).ok()?;
    return identity_at(&parent, path.file_name()?).ok();
}

/// What:
///  Reserve the destination of a creating command as an empty directory,
/// and record what it is.
/// Why:
///   Git accepts an existing empty directory as the destination of `git worktree add` and fills it,
/// so the
///  directory this command made is the one its worktree is made in,
/// and its identity proves it later.
/// An
///  existing directory is another command's reservation:
/// this one copies nothing and says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function reserveDestination(destination: string): Promise<Reservation>;
/// ```
pub fn reserve_destination(destination: &Path) -> Reservation {
    if destination.as_os_str().is_empty() {
        return Reservation::NotAttempted;
    }
    // `match` on the exclusive creation: the command that makes the directory owns the destination.
    match std::fs::DirBuilder::new().create(destination) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return Reservation::Anothers(identity_of_entry(destination)),
        Err(error) => {
            debug(TAG, format!("no reservation of {} ({error}); Git makes the directory and this command copies nothing", destination.display()).as_str());
            return Reservation::NotAttempted;
        }
    }
    let read: Option<ObjectId> = match destination.parent() {
        Some(parent) => match std::fs::File::open(parent) {
            Ok(held) => identity_at(&held, destination.file_name().unwrap_or_default()).ok(),
            Err(error) => {
                debug(TAG, format!("the identity of the reserved {} cannot be read ({error})", destination.display()).as_str());
                None
            }
        },
        None => None,
    };
    debug(TAG, format!("reserved the empty destination {}", destination.display()).as_str());
    return Reservation::Mine(CreatedSubvolume { path: destination.to_path_buf(), identity: read });
}

/// What:
///  What became of the empty object this command created at a destination,
/// when it was settled after real Git returned.
/// Why:
///   A destination that still holds something is the one case the person has to be told about,
/// and only
///  the settlement knows it;
/// a removed or absent object leaves nothing to say.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CreatedDestinationSettlement = 'removed' | 'gone' | 'holds something' | 'left alone';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CreatedDestinationSettlement {
    /// The empty object this command created was removed.
    Removed,
    /// Nothing stands at the path any more.
    Gone,
    /// The object at the path is the one this command created and it holds something,
    /// so it stays with everything in it.
    HoldsSomething,
    /// The object at the path is not the one this command created,
    /// or the path or its parent could not be read,
    /// so nothing was removed and nothing is said.
    LeftAlone,
}

/// What:
///  After Git ended,
/// remove the object this command created at a destination while it is still that object and still empty.
///  `rmdir` removes an empty btrfs subvolume without the `user_subvol_rm_allowed` mount option.
/// Why:
///   Git leaves an empty destination when it failed before it started to fill it,
/// and a leftover empty folder
///  inside a repository is acted on by any Git command run in it,
/// as if the enclosing checkout were the new
///  worktree;
/// a leftover empty subvolume would also make the next attempt at the same path look like an accepted empty
///  directory.
/// A destination that holds anything
///  is Git's result (or its late failure's, or another program's) and stays,
/// and so does an object that another process put at the path:
/// the
///  identity recorded at creation must still be the identity of what the name leads to,
/// and the emptiness is read through
///  the descriptor of that object.
/// The last step names the entry in the held parent,
/// and `rmdir` removes only an empty
///  directory,
/// so a swap in that instant could remove only an empty directory,
/// never data.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function settleCreatedDestination(made: CreatedSubvolume): Promise<CreatedDestinationSettlement>;
/// ```
pub fn settle_created_destination(created: &CreatedSubvolume) -> CreatedDestinationSettlement {
    let shown: &Path = created.path.as_path();
    let Some(expected) = created.identity else {
        debug(TAG, format!("{} was not identified when it was created; left alone", shown.display()).as_str());
        return CreatedDestinationSettlement::LeftAlone;
    };
    let (Some(parent_path), Some(name)) = (shown.parent(), shown.file_name()) else {
        debug(TAG, format!("{} has no parent and name; left alone", shown.display()).as_str());
        return CreatedDestinationSettlement::LeftAlone;
    };
    let parent: std::fs::File = match std::fs::File::open(parent_path) {
        Ok(parent) => parent,
        Err(error) => {
            debug(TAG, format!("{} cannot be settled ({error}); left alone", parent_path.display()).as_str());
            return CreatedDestinationSettlement::LeftAlone;
        }
    };
    let held: std::fs::File = match open_directory_at(&parent, name) {
        Ok(held) => held,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return CreatedDestinationSettlement::Gone,
        Err(error) => {
            debug(TAG, format!("{} cannot be opened ({error}); left alone", shown.display()).as_str());
            return CreatedDestinationSettlement::LeftAlone;
        }
    };
    if identity_of(&held).ok() != Some(expected) {
        warn(TAG, format!("{} is not the empty object this command created, so it is left in place; remove it yourself if it is not wanted", shown.display()).as_str());
        return CreatedDestinationSettlement::LeftAlone;
    }
    let mut listing = match std::fs::read_dir(handle_path(&held)) {
        Ok(listing) => listing,
        Err(error) => {
            debug(TAG, format!("{} cannot be listed ({error}); left alone", shown.display()).as_str());
            return CreatedDestinationSettlement::LeftAlone;
        }
    };
    if listing.next().is_some() {
        return CreatedDestinationSettlement::HoldsSomething;
    }
    match remove_at(&parent, name, true) {
        Ok(()) => {
            debug(TAG, format!("removed the empty destination {} that Git did not fill", shown.display()).as_str());
            return CreatedDestinationSettlement::Removed;
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return CreatedDestinationSettlement::Gone,
        Err(error) => {
            debug(TAG, format!("could not remove the empty destination {} ({error})", shown.display()).as_str());
            return CreatedDestinationSettlement::LeftAlone;
        }
    }
}

/// Subvolume destination controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_subvolume_tests.rs"]
mod tests;
