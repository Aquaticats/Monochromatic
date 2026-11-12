//! What:
//!  The pools of snapshots the wrapper made ahead of an add,
//! and every pool a clear took and a kill left,
//!  taken out of the administrative directory of a worktree that is being removed,
//! before anything irreversible happens to
//!  that worktree.
//! Why:
//!  A worktree that was the source of one `git worktree add` holds
//! `<admin>/monochromatic-git-policy-cli/ahead`,
//!  a pool whose members are subvolumes of that worktree (`worktree_snapshot_ahead.rs`).
//! A removal destroys the
//!  worktree's own subvolume in one step and empties the administrative directory after it,
//! and that emptying refuses
//!  a nested subvolume,
//! because a tree that holds one holds data that is not part of the tree being removed
//!  (`subvolume_remove_walk.rs`).
//! So the pool was the one thing that made a finished removal end as "the files are
//!  gone and the registration is not",
//! with the whole pool left to `git worktree prune` (a Sonnet finding,
//! F1),
//! and the sweep after
//!  a merge reported that state as a worktree left in place.
//! Nothing else takes a pool:
//! the sweep of the repository scratch directory cannot reach a folder inside a worktree's
//!  administrative directory,
//! and a fill names only a worktree that still exists.
//! The pool is the wrapper's own cache of the worktree being
//!  removed,
//! exactly as the shadow pool of a removed worktree is (`worktree_removal_clears_the_pool.rs`),
//! and an add that
//!  finds no pool makes its own copy,
//! so nothing an add needs is lost with it.
//! What is taken is what this program wrote,
//! never a guess about the volumes:
//! the pool a fill reads,
//! and a
//!  pool a clear renamed to `removing-<identity>` and a kill left before it emptied it.
//! That second name is a
//!  record this program keeps of a pool it was taking,
//! and it is what makes an interrupted take resumable:
//! the next
//!  removal takes that pool rather than refusing a volume it made itself.
//! The data folder around them stays,
//!  because it also holds what a person reads:
//! the notices of a push that failed,
//! the files a pull kept out of
//!  the way,
//! and the service's own work queue,
//! none of which a removal may delete.
//! A pool is taken out of its own name
//!  with one rename before anything of it is taken:
//! a fill makes a member in the pool's own name,
//! so after the
//!  rename it finds no pool and makes its own,
//! while the members this clear was given stand under a name nothing
//!  else reads.
//! Every member is then taken out of the pool with one more rename and a durable record of the
//!  commit service's delete
//! (`worktree_remove_hand_over.rs`,
//! `leave_a_subvolume_for_the_service`),
//! never destroyed by the
//!  command a person waits on:
//! the btrfs destroy takes the filesystem's write lock against every snapshot in flight,
//!  so a person removing a worktree would wait for another add's snapshot to commit
//!  (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`,
//! a review finding,
//! F5).
//! The
//!  members leave for the folder this program keeps in the repository's common Git directory,
//!  which is on the pool's own
//! filesystem,
//! and a member that is no subvolume is a plain tree this program wrote and goes as one.
//! An older release spelled the data folder another way,
//! so both spellings are cleared and neither folder is
//!  moved.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! clearThePoolsAheadOfAWorktree(admin); // throws the reason nothing was removed
//! ```

use crate::fs_kind::is_subvolume_root;
use crate::private_storage::remove_tree;
use crate::wrapper_names_old_and_new::DATA_FOLDER;
use crate::worktree_removal_clears_the_pool::{REMOVING_PREFIX, RemoveAPool, take_the_pool_out_of_its_name};
use crate::worktree_remove_hand_over::leave_a_subvolume_for_the_service;
use crate::worktree_snapshot_ahead::POOL_FOLDER;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// What:
///  The one call that takes one member of the pool out of it,
/// behind a stand-in.
/// Why:
///   A member is a subvolume,
/// which no plain directory of a control is,
/// and a control has to see what a member that cannot go
///  comes to without a btrfs filesystem underneath it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TakeAMember = (member: string) => void; // throws the reason it did not
/// ```
pub(crate) type TakeAMember<'a> = dyn Fn(&Path) -> std::io::Result<()> + 'a;

/// What:
///  Remove every pool of snapshots ahead in the administrative directory `admin`,
/// and every pool a clear took and a kill left,
///  and answer the reason when something cannot go.
/// Why:
///   The removal is a refusal when this fails,
/// so it must run while the worktree is still there,
/// and the
///  answer is the words for that refusal.
///  What is taken is named by what this program wrote there,
/// never by a guess about the volumes:
/// the pool a fill reads,
///  and a pool a clear renamed to `removing-<identity>` and was killed before emptying.
/// Both are a
///  directory of the data folder the fill and the clear own,
/// and every other entry of that folder is left where it is,
///  because the same folder holds what a person reads:
/// the notices of a push that failed,
/// the files a
///  pull kept out of the way,
/// and the service's own work queue.
/// The data folder itself therefore stays,
/// and the
///  finish copies the notices it still holds before the registration goes.
///  An older release spelled the data folder another way,
/// so both spellings are cleared,
/// and neither folder is
///  moved by a command that does not own it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// clearThePoolsAheadOfAWorktree({ commonDir, admin }); // throws the reason
/// ```
///
/// # Errors
///
/// The words for a pool or a member that cannot be taken,
/// which is the reason the removal refuses while nothing of the
///  worktree has changed.
pub(crate) fn clear_the_pools_ahead_of_a_worktree(common_dir: &Path, admin: &Path) -> Result<(), String> {
    for folder in data_folders(admin) {
        clear_the_pools_ahead_in(common_dir, folder.as_path())?;
    }
    return Ok(());
}

/// What:
///  The folder in the repository's common Git directory that this program keeps its own state in,
/// which is where a
///  subvolume a removal leaves for the service is moved to.
/// Why:
///   A pool member is a snapshot of the worktree and sits in the pool the worktree's administrative directory
///  holds,
/// so it is on the common Git directory's filesystem,
/// and the folder beside the work queue is
///  the one place on that filesystem this program owns:
/// a move there crosses no device,
/// and no registration
///  holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folder = join(commonDir, DATA_FOLDER.current); // <common>/monochromatic-git-policy-cli
/// ```
pub(crate) fn common_data_folder(common_dir: &Path) -> PathBuf {
    return common_dir.join(DATA_FOLDER.current);
}

/// What:
///  The folders inside the administrative directory `admin` that hold this program's state,
/// both spellings
///  of the data folder's name.
/// Why:
///   A release before the program was renamed wrote the earlier name,
/// and a removal of that worktree takes its
///  leftovers as much as this release's,
/// while nothing that reads them ever moves a folder it does not own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folders = dataFolders(admin); // [<admin>/monochromatic-git-policy-cli, <admin>/cli-git]
/// ```
pub(crate) fn data_folders(admin: &Path) -> [PathBuf; 2] {
    return [admin.join(DATA_FOLDER.current), admin.join(DATA_FOLDER.earlier)];
}

/// What:
///  The same clear for one spelling of the data folder `folder`.
/// Why:
///   Both spellings need the same steps,
/// and a folder that is not there is nothing at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// clearThePoolsAheadIn(folder); // throws the reason
/// ```
fn clear_the_pools_ahead_in(common_dir: &Path, folder: &Path) -> Result<(), String> {
    // The pool a fill reads is taken out of its name first, so a fill that comes afterwards makes a pool of its own
    // instead of writing into the tree this clear was given.
    let mut taken: Vec<PathBuf> = Vec::new();
    if let Some(moved) = take_the_pool_out_of_its_name(folder.join(POOL_FOLDER).as_path(), REMOVING_PREFIX)? {
        taken.push(moved);
    }
    taken.extend(left_over_pools(folder)?);
    let destination: PathBuf = common_data_folder(common_dir);
    for pool in taken {
        clear_one_pool_ahead(
            pool.as_path(),
            &|member: &Path| { return leave_a_member_to_the_service(common_dir, destination.as_path(), member); },
            &remove_tree,
        )?;
    }
    // The folder goes when nothing put anything back in it, and stays when it holds what a person reads. A fill that made a
    // pool of its own in these moments keeps that pool, and the emptying of the registration after the destroy answers it.
    let _kept = std::fs::remove_dir(folder);
    return Ok(());
}

/// What:
///  The pools a clear renamed to a name no fill reads and a kill left behind,
/// in the data folder `folder`.
/// Why:
///   The rename comes before every deletion,
/// so the name a killed clear leaves is this program's own record of a pool it
///  was taking,
/// and the body after its prefix is the identity that name was made of.
/// A directory called
///  `removing-<identity>` is the one shape taken by name,
/// and it is a record this program wrote rather than a guess about
///  what a volume is:
/// nothing else in the data folder is ever renamed to that shape.
/// A link at such a name is not that record:
/// a pool is a directory this program made,
///  and taking the link would move it and then read every entry below it through
///  its target.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pools = leftOverPools(folder); // string[]
/// ```
fn left_over_pools(folder: &Path) -> Result<Vec<PathBuf>, String> {
    let listing = match std::fs::read_dir(folder) {
        Ok(listing) => listing,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("the folder {folder} this program keeps for the worktree cannot be read ({error})", folder = folder.display())),
    };
    let mut pools: Vec<PathBuf> = Vec::new();
    for found in listing {
        let entry: std::fs::DirEntry = match found {
            Ok(entry) => entry,
            Err(error) => return Err(format!("the folder {folder} this program keeps for the worktree cannot be read ({error})", folder = folder.display())),
        };
        let name: std::ffi::OsString = entry.file_name();
        let Some(text) = name.to_str() else { continue };
        let Some(identity) = text.strip_prefix(REMOVING_PREFIX) else { continue };
        if !is_an_identity(identity) {
            continue;
        }
        // What the entry is, never what it is named: `DirEntry::file_type` answers for the entry itself and does not
        // follow a link.
        match entry.file_type() {
            Ok(kind) if kind.is_dir() => pools.push(entry.path()),
            Ok(_not_a_directory) => {}
            Err(error) => return Err(format!("the folder {folder} this program keeps for the worktree cannot be read ({error})", folder = folder.display())),
        }
    }
    return Ok(pools);
}

/// Whether one byte of a name this program made is a hexadecimal digit.
fn is_a_hexadecimal_digit(byte: u8) -> bool {
    return byte.is_ascii_hexdigit();
}

/// What:
///  Whether `text` is an identity this program makes for a name:
/// eight hexadecimal digits and then four,
///  four,
///  four and twelve more around four hyphens.
/// Why:
///   A leftover pool is named `removing-<identity>`,
/// and an identity is a shape this program alone writes,
/// so a
///  directory a person named anything else is never taken for one of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identity = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(text);
/// ```
fn is_an_identity(text: &str) -> bool {
    let shape: [usize; 5] = [8, 4, 4, 4, 12];
    let mut parts = text.split('-');
    for digits in shape {
        let Some(part) = parts.next() else { return false };
        if part.len() != digits || !part.bytes().all(is_a_hexadecimal_digit) {
            return false;
        }
    }
    return parts.next().is_none();
}

/// What:
///  Leave one member of the pool for the commit service to delete when it is a subvolume,
///  and remove one that is no subvolume at all.
/// Why:
///   Every member the service makes is a subvolume,
/// and a btrfs destroy takes the filesystem's
///  write lock against every snapshot in flight,
/// so it is the service's work and never the person's:
/// the removal
///  is the one command a person waits on,
/// and the member leaves the registration with one rename and
///  a durable record
/// (`leave_a_subvolume_for_the_service`).
/// An entry that is no subvolume is a name this
///  program wrote and goes as a plain tree, which waits on nothing;
/// a member another process claimed first
///  is gone,
/// which is the same end state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// leaveAMemberToTheService({ commonDir, destination, member }); // throws the reason
/// ```
fn leave_a_member_to_the_service(common_dir: &Path, destination: &Path, member: &Path) -> std::io::Result<()> {
    // A pool holds a subvolume per member; a name that is no directory is a file this program wrote once, and it goes
    // without a delete that would have to walk it.
    return match std::fs::symlink_metadata(member) {
        Ok(found) if !found.is_dir() => std::fs::remove_file(member),
        Ok(_directory) if is_subvolume_root(member).unwrap_or(false) => {
            return leave_a_subvolume_for_the_service(common_dir, member, destination).map_err(std::io::Error::other).map(|_left| {});
        }
        Ok(_directory) => remove_tree(member),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    };
}

/// What:
///  Take every member of the pool `taken` out of it,
/// then remove the folder itself,
/// with the two calls a control replaces.
/// Why:
///   The deletion is given a name no fill reads,
/// so a member that appears afterwards is made in a pool of its own,
/// and
///  one that cannot go is the reason the whole removal refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// clearOnePoolAhead(taken, destroy, remove); // throws the reason
/// ```
pub(crate) fn clear_one_pool_ahead(taken: &Path, take_member: &TakeAMember<'_>, remove: &RemoveAPool<'_>) -> Result<(), String> {
    let listing = match std::fs::read_dir(taken) {
        Ok(listing) => listing,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("the folder {taken} of the snapshots the wrapper made ahead of an add cannot be read ({error})", taken = taken.display())),
    };
    for found in listing {
        let member: PathBuf = match found {
            Ok(entry) => entry.path(),
            Err(error) => return Err(format!("the folder {taken} of the snapshots the wrapper made ahead of an add cannot be read ({error})", taken = taken.display())),
        };
        match take_member(member.as_path()) {
            Ok(()) => {}
            // Another process claimed the member between the listing and the destroy: it is not in the pool any more.
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "the snapshot {member} the wrapper made ahead of an add cannot be taken for the commit service to delete ({error})",
                    member = member.display()
                ));
            }
        }
    }
    return match remove(taken) {
        Ok(()) => Ok(()),
        // Another command's clear took it between the listing and this call.
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("the folder {taken} of the snapshots the wrapper made ahead of an add cannot be removed ({error})", taken = taken.display())),
    };
}

/// Controls of the clear stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_clears_the_ahead_pool_tests.rs"]
mod tests;
