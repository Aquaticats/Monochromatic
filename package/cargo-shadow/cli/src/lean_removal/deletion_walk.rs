//! What:
//! The deletion of a claimed entry:
//! the walk that removes what an entry holds without following a link,
//! the owner access it adds to a directory this user owns so it can be emptied,
//! and the identity check just before the unlink.
//! Why:
//! A directory the walk may not empty stops that part of the tree and not the rest,
//! so a tree that holds a directory of another user is still removed as far as it can be,
//! and the caller can say what is left and why.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function allocatedBytesOf(s: Stats): number
//! export function widenedMode(mode: number): number
//! function openDirectory(path: string, metadata: Stats, walk: Walk): string | undefined
//! function deleteEntry(path: string, walk: Walk): void
//! export function deleteClaimed(claimed: string, judged: Identity, countBytes: ByteCount): Deleted  // throws Refusal
//! export function deleteClaimedAs(claimed: string, judged: Identity, thisUid: number, countBytes: ByteCount): Deleted  // throws Refusal
//! ```
/// Imports what a completed deletion did.
use super::item_and_deletion_record::Deleted;
/// Imports the refusals a deletion answers with.
use super::refusal_text::Refusal;
/// Imports the device and inode every deletion is checked against.
use crate::lean_identity::{
    Identity,
    identity_of,
};
/// Imports how a walk counts an entry's bytes.
use crate::lean_sizes::ByteCount;
/// Imports the effective user id,
/// which is the user a removal acts as.
use rustix::process::geteuid;
/// Imports the borrowed path type the walk works with.
use std::path::Path;
/// Imports the file system,
/// the error kinds and the metadata fields the walk reads.
use std::{
    fs,
    io,
    os::unix::fs::{
        MetadataExt,
        PermissionsExt,
    },
};
/// The owner's read,
/// write and search bits:
/// what a directory needs before the walk can name what it holds and unlink it.
/// All three are added together because a directory this user can list but not unlink in (`0555`) and one this user
/// can neither list nor unlink in (`0000`) need the same walk to get through.
const OWNER_ACCESS_BITS: u32 = 0o700;

/// What:
/// `pub fn allocated_bytes_of(metadata: &fs::Metadata) -> u64` is the disk space an entry occupies (blocks of 512 bytes).
/// Why:
/// Freed space is reported by what the files occupy,
/// not by their apparent length,
/// so this is the rule every tool of this package counts by.
/// Gotcha:
/// A file system need not have accounted a fresh file's blocks yet:
/// on this host's ZFS a file written a moment ago reports one 512-byte block until the pool's transaction group commits
/// (measured 2026-11-27, issue 6552),
/// which is why a control that sizes what it just wrote counts by `length_bytes` instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const allocatedBytesOf = (s: Stats): number => s.blocks * 512;
/// ```
pub fn allocated_bytes_of(metadata: &fs::Metadata) -> u64 {
    return metadata.blocks() * 512;
}

/// What:
/// `pub(crate) fn widened_mode(mode: u32) -> u32` is a directory's mode with this user's read,
/// write and search added.
/// Why:
/// The removal widens what this user may do and nothing else:
/// a group bit and an other-user bit a person set stay as they were.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const widenedMode = (mode: number): number => mode | 0o700;
/// ```
pub(crate) fn widened_mode(mode: u32) -> u32 {
    return mode | OWNER_ACCESS_BITS;
}

/// What:
/// `fn open_directory(path: &Path, metadata: &fs::Metadata, walk: &mut Walk<'_>) -> Option<String>` gives the directory this
/// user's read,
/// write and search permission when they are missing and this user owns it,
/// answering nothing when the walk may go on into it and the reason when it may not.
/// Why:
/// Unlinking what a directory holds needs write and search on that directory,
/// and listing it needs read;
/// a directory of another user stays as it is and is named instead,
/// because this removal may only change what this user owns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openDirectory(path: string, metadata: Stats, walk: Walk): string | undefined
/// ```
fn open_directory(
    path: &Path,
    metadata: &fs::Metadata,
    walk: &mut Walk<'_>,
) -> Option<String> {
    let mode: u32 = metadata.mode() & 0o7777;
    if mode & OWNER_ACCESS_BITS == OWNER_ACCESS_BITS {
        return None;
    }
    if metadata.uid() != walk.this_uid {
        return Some(format!(
            "{} is owned by uid {} and the removal runs as uid {}, so its mode {mode:04o} was not changed",
            path.display(),
            metadata.uid(),
            walk.this_uid
        ));
    }
    let widened: fs::Permissions = fs::Permissions::from_mode(widened_mode(mode));
    return match fs::set_permissions(path, widened) {
        Ok(()) => {
            walk.deleted.modes_added.push(path.to_path_buf());
            None
        }
        Err(error) => Some(format!(
            "{} could not be made writable (mode {mode:04o}): {error}",
            path.display()
        )),
    };
}

/// What a walk carries from one entry to the next.
struct Walk<'a> {
    /// The user the removal runs as,
    /// which decides whose directory mode it may change.
    this_uid: u32,
    /// How the walk counts an entry's bytes,
    /// which its caller sets.
    count_bytes: ByteCount,
    /// What it did.
    deleted: Deleted,
    /// The files already counted,
    /// so a file with several hard links is counted once.
    seen: &'a mut std::collections::HashSet<Identity>,
}

/// What:
/// `fn delete_entry(path: &Path, walk: &mut Walk<'_>)` deletes one entry and everything under it without following a link,
/// and records what it did and what stayed.
/// Why:
/// One directory it may not empty stops that part of the tree and not the rest,
/// so a tree that holds a directory of another user is still removed as far as it can be,
/// and the caller can say what is left and why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deleteEntry(path: string, walk: Walk): void
/// ```
fn delete_entry(
    path: &Path,
    walk: &mut Walk<'_>,
) {
    let metadata: fs::Metadata = match fs::symlink_metadata(path) {
        Ok(found) => found,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return,
        Err(error) => {
            walk.deleted
                .remaining
                .push(format!("{} could not be read: {error}", path.display()));
            return;
        }
    };
    if metadata.is_dir() {
        if let Some(reason) = open_directory(path, &metadata, walk) {
            // A directory the walk may not enter can still be removed when it holds nothing,
            // and only what stays behind is worth a reason.
            match fs::remove_dir(path) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) if error.kind() == io::ErrorKind::DirectoryNotEmpty => {
                    walk.deleted.remaining.push(reason)
                }
                Err(error) => walk.deleted.remaining.push(format!(
                    "{reason}, and it could not be removed either: {error}"
                )),
            }
            return;
        }
        match fs::read_dir(path) {
            Ok(entries) => {
                for child in entries {
                    match child {
                        Ok(entry) => delete_entry(&entry.path(), walk),
                        Err(error) => walk.deleted.remaining.push(format!(
                            "{} holds an entry that could not be read: {error}",
                            path.display()
                        )),
                    }
                }
            }
            Err(error) => walk
                .deleted
                .remaining
                .push(format!("{} could not be listed: {error}", path.display())),
        }
        match fs::remove_dir(path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) if error.kind() == io::ErrorKind::DirectoryNotEmpty => {}
            Err(error) => walk
                .deleted
                .remaining
                .push(format!("{} could not be removed: {error}", path.display())),
        }
        return;
    }
    let first: bool = metadata.nlink() <= 1
        || walk.seen.insert(Identity {
            device: metadata.dev(),
            inode: metadata.ino(),
        });
    if first {
        walk.deleted.bytes += (walk.count_bytes)(&metadata);
    }
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => walk
            .deleted
            .remaining
            .push(format!("{} could not be removed: {error}", path.display())),
    }
}

/// What:
/// `pub fn delete_claimed(claimed: &Path, judged: Identity, count_bytes: ByteCount) -> Result<Deleted, Refusal>` deletes a
/// claimed entry after checking its identity once more,
/// and reports what it removed.
/// Why:
/// The device and inode are checked just before the unlink,
/// so nothing but the judged entry is ever deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function deleteClaimed(claimed: string, judged: Identity, countBytes: ByteCount): Deleted  // throws Refusal
/// ```
pub fn delete_claimed(
    claimed: &Path,
    judged: Identity,
    count_bytes: ByteCount,
) -> Result<Deleted, Refusal> {
    return delete_claimed_as(claimed, judged, geteuid().as_raw(), count_bytes);
}

/// What:
/// `pub(crate) fn delete_claimed_as(claimed: &Path, judged: Identity, this_uid: u32, count_bytes: ByteCount) -> Result<Deleted, Refusal>`
/// deletes a claimed entry as the given user.
/// Why:
/// The directory whose mode a removal may change is the one this user owns,
/// and a test can act as a user who owns nothing of the fixture.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function deleteClaimedAs(claimed: string, judged: Identity, thisUid: number, countBytes: ByteCount): Deleted  // throws Refusal
/// ```
pub(crate) fn delete_claimed_as(
    claimed: &Path,
    judged: Identity,
    this_uid: u32,
    count_bytes: ByteCount,
) -> Result<Deleted, Refusal> {
    let found: Identity = match identity_of(claimed) {
        Ok(identity) => identity,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(Refusal::Gone(claimed.to_path_buf()));
        }
        Err(error) => {
            return Err(Refusal::Io {
                path: claimed.to_path_buf(),
                error,
            });
        }
    };
    if found != judged {
        return Err(Refusal::Changed {
            path: claimed.to_path_buf(),
            judged,
            found,
        });
    }
    let mut seen: std::collections::HashSet<Identity> = std::collections::HashSet::new();
    let mut walk: Walk<'_> = Walk {
        this_uid,
        count_bytes,
        deleted: Deleted::default(),
        seen: &mut seen,
    };
    delete_entry(claimed, &mut walk);
    return match walk.deleted.remaining.is_empty() {
        true => Ok(walk.deleted),
        false => Err(Refusal::PartlyRemoved {
            path: claimed.to_path_buf(),
            deleted: walk.deleted,
        }),
    };
}
