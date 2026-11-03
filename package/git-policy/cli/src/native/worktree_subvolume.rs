//! What:
//!  The empty btrfs subvolume the wrapper makes at the destination of a `git worktree add`, before real Git runs, and
//!  the cleanup of it when Git leaves it empty.
//! Why:
//!  A worktree that is a subvolume is a source of a later snapshot with no migration, and it is removed in one
//!  call. Git accepts an existing empty directory as the destination and fills it, and an empty subvolume is an
//!  empty directory (`doc/planning/cli-git-btrfs-worktree.md`, "Creating new worktrees as subvolumes"). Git removes the
//!  destination again only when it created it and failed late; a failure before that point (a branch that exists, a
//!  name Git rejects) leaves the directory, so the wrapper removes the empty subvolume itself. A destination that is
//!  not on btrfs, or a subvolume the kernel refuses, leaves Git to make its ordinary directory without a word to the
//!  person, with one diagnostic line for the log.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const made = await prepareSubvolumeDestination(plan); // undefined: Git makes the directory
//! await runGit();
//! if (made) await settleSubvolumeDestination(made); // rmdir when still empty
//! ```

use super::diagnostic_log::debug;
use super::fs_btrfs_ioctl::subvolume_create_at;
use super::fs_kind::{FilesystemKind, filesystem_kind};
use super::worktree_add_plan::AddPlan;
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
/// type CreatedSubvolume = { path: string };
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct CreatedSubvolume {
    /// The destination, an empty subvolume when it was created.
    pub path: PathBuf,
}

/// What:
///  Create the destination of a planned `git worktree add` as an empty subvolume, when it can be one.
///  `Option<CreatedSubvolume>` is the created subvolume or nothing.
/// Why:
///   The destination must be absent (an existing one is Git's to accept or refuse) and its parent must be on btrfs. The
///  creation ioctl is unrestricted for a regular user, so a refusal is a fault the log keeps and Git's ordinary directory
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
    return Some(CreatedSubvolume { path: destination.to_path_buf() });
}

/// What:
///  After Git ended, remove the created subvolume if it is still empty.
///  `rmdir` deletes an empty subvolume without the `user_subvol_rm_allowed` mount option.
/// Why:
///   Git leaves an empty destination when it failed before it started to fill it, and a leftover empty subvolume would
///  make the next attempt at the same path look like an accepted empty directory. A destination that holds anything
///  is Git's result (or its late failure's) and stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function settleSubvolumeDestination(made: CreatedSubvolume): Promise<void>;
/// ```
pub fn settle_subvolume_destination(created: &CreatedSubvolume) {
    let mut listing = match std::fs::read_dir(created.path.as_path()) {
        Ok(listing) => listing,
        Err(error) => {
            debug(TAG, format!("{} cannot be listed ({error}); left alone", created.path.display()).as_str());
            return;
        }
    };
    if listing.next().is_some() {
        return;
    }
    match std::fs::remove_dir(created.path.as_path()) {
        Ok(()) => debug(TAG, format!("removed the empty subvolume {} that Git did not fill", created.path.display()).as_str()),
        Err(error) => debug(TAG, format!("could not remove the empty subvolume {} ({error})", created.path.display()).as_str()),
    }
}

/// Subvolume destination controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_subvolume_tests.rs"]
mod tests;
