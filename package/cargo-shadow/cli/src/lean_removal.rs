//! What:
//! The one way every lean-folder tool removes an entry:
//! check its identity,
//! move it into a trash directory on the same file system by an atomic rename that never replaces,
//! check the identity again,
//! and only then delete it.
//! Why:
//! Removal is one owner's action per item.
//! The rename claims the item (a second tool that races finds it gone),
//! the identity check before the rename and before the unlink (as the Git wrapper's lock removals do:
//! `package/git-policy/cli/src/native/lock/stale_ref_lock_own_ended.rs`) stops a path that was swapped for another entry from
//! being removed,
//! and a mismatch leaves what was moved in the trash for a person to look at:
//! "never discard user data".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function removeOne(item: Item, trash: string): Removed  // throws Refusal
//! ```

/// Files, directories and metadata.
use std::{fs, io, os::unix::fs::MetadataExt, path::{Path, PathBuf}};

/// The rename that fails instead of replacing.
use rustix::fs::{CWD, RenameFlags, renameat_with};

/// Device and inode.
use crate::lean_identity::{Identity, identity_of};

/// The name of the trash directory each tool keeps beside what it removes from.
pub const TRASH_DIRECTORY_NAME: &str = ".cargo-shadow-trash";

/// One thing to remove,
/// with the identity it had when it was classified.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Item {
    /// Where it is.
    pub path: PathBuf,
    /// What it was when judged.
    pub identity: Identity,
}

/// Why an item was not removed.
#[derive(Debug)]
pub enum Refusal {
    /// The path now holds another entry,
    /// or the moved entry is not the judged one.
    Changed {
        /// The path.
        path: PathBuf,
        /// What was judged.
        judged: Identity,
        /// What was found.
        found: Identity,
    },
    /// The entry vanished before it was claimed:
    /// another owner removed it.
    Gone(PathBuf),
    /// A system call failed.
    Io {
        /// The path.
        path: PathBuf,
        /// The error.
        error: io::Error,
    },
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Refusal::Changed { path, judged, found } => write!(
                formatter,
                "{} is not the entry that was judged (device {} inode {} then, device {} inode {} now); nothing was removed",
                path.display(),
                judged.device,
                judged.inode,
                found.device,
                found.inode
            ),
            Refusal::Gone(path) => write!(formatter, "{} was already removed by another owner", path.display()),
            Refusal::Io { path, error } => write!(formatter, "{}: {error}", path.display()),
        };
    }
}

/// What:
/// `pub fn item_of(path: &Path) -> io::Result<Item>` reads the identity an item has now.
/// Why:
/// Classification records it,
/// and the removal compares against it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const itemOf = (path: string): Item => ({ path, identity: identityOf(path) });
/// ```
pub fn item_of(path: &Path) -> io::Result<Item> {
    return Ok(Item { path: path.to_path_buf(), identity: identity_of(path)? });
}

/// What:
/// `fn trash_name(trash: &Path, item: &Path, serial: u64) -> PathBuf` is a name in the trash no other claim can share.
/// Why:
/// The process id, the clock and a serial make each claim unique, so the rename never has to replace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const trashName = (trash: string, item: string, serial: number) => join(trash, `${process.pid}-${Date.now()}-${serial}-${basename(item)}`);
/// ```
fn trash_name(trash: &Path, item: &Path, serial: u64) -> PathBuf {
    let nanos: u128 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|elapsed| { return elapsed.as_nanos(); }).unwrap_or(0);
    let base: String = item.file_name().map(|name| { return name.to_string_lossy().to_string(); }).unwrap_or_default();
    return trash.join(format!("{}-{nanos}-{serial}-{base}", std::process::id()));
}

/// What:
/// `pub fn claim_into_trash(item: &Item, trash: &Path, serial: u64) -> Result<PathBuf, Refusal>` moves the item into the trash
/// when it still is the judged entry.
/// Why:
/// It is the claim.
/// The trash must be on the item's file system (a sibling directory),
/// so the move is one atomic rename.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function claimIntoTrash(item: Item, trash: string, serial: number): string  // throws Refusal
/// ```
pub fn claim_into_trash(item: &Item, trash: &Path, serial: u64) -> Result<PathBuf, Refusal> {
    let before: Identity = match identity_of(&item.path) {
        Ok(found) => found,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Err(Refusal::Gone(item.path.clone())),
        Err(error) => return Err(Refusal::Io { path: item.path.clone(), error }),
    };
    if before != item.identity {
        return Err(Refusal::Changed { path: item.path.clone(), judged: item.identity, found: before });
    }
    fs::create_dir_all(trash).map_err(|error| { return Refusal::Io { path: trash.to_path_buf(), error }; })?;
    let target: PathBuf = trash_name(trash, &item.path, serial);
    match renameat_with(CWD, &item.path, CWD, &target, RenameFlags::NOREPLACE) {
        Ok(()) => {}
        Err(code) if code == rustix::io::Errno::NOENT => return Err(Refusal::Gone(item.path.clone())),
        Err(code) => return Err(Refusal::Io { path: item.path.clone(), error: io::Error::from(code) }),
    }
    let after: Identity = identity_of(&target).map_err(|error| { return Refusal::Io { path: target.clone(), error }; })?;
    if after != item.identity {
        // The path was swapped between the check and the rename: what moved is not what was judged. It stays in the trash.
        return Err(Refusal::Changed { path: target, judged: item.identity, found: after });
    }
    return Ok(target);
}

/// What:
/// `pub fn allocated_bytes_of(metadata: &fs::Metadata) -> u64` is the disk space an entry occupies (blocks of 512 bytes).
/// Why:
/// Freed space is reported by what the files occupy,
/// not by their apparent length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const allocatedBytesOf = (s: Stats): number => s.blocks * 512;
/// ```
pub fn allocated_bytes_of(metadata: &fs::Metadata) -> u64 {
    return metadata.blocks() * 512;
}

/// What:
/// `fn delete_entry(path: &Path, seen: &mut std::collections::HashSet<Identity>) -> io::Result<u64>` deletes one entry and
/// everything under it without following a link,
/// and returns the bytes of the files whose last link in this walk went.
/// Why:
/// A file with several hard links is counted once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deleteEntry(path: string, seen: Set<string>): number
/// ```
fn delete_entry(path: &Path, seen: &mut std::collections::HashSet<Identity>) -> io::Result<u64> {
    let metadata: fs::Metadata = match fs::symlink_metadata(path) {
        Ok(found) => found,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error),
    };
    let mut bytes: u64 = 0;
    if metadata.is_dir() {
        for child in fs::read_dir(path)? {
            bytes += delete_entry(&child?.path(), seen)?;
        }
        match fs::remove_dir(path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        return Ok(bytes);
    }
    let first: bool = metadata.nlink() <= 1 || seen.insert(Identity { device: metadata.dev(), inode: metadata.ino() });
    if first {
        bytes += allocated_bytes_of(&metadata);
    }
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    return Ok(bytes);
}

/// What:
/// `pub fn delete_claimed(claimed: &Path, judged: Identity) -> Result<u64, Refusal>` deletes a claimed entry after checking
/// its identity once more,
/// and returns the bytes freed.
/// Why:
/// The device and inode are checked just before the unlink,
/// so nothing but the judged entry is ever deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function deleteClaimed(claimed: string, judged: Identity): number  // throws Refusal
/// ```
pub fn delete_claimed(claimed: &Path, judged: Identity) -> Result<u64, Refusal> {
    let found: Identity = match identity_of(claimed) {
        Ok(identity) => identity,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Err(Refusal::Gone(claimed.to_path_buf())),
        Err(error) => return Err(Refusal::Io { path: claimed.to_path_buf(), error }),
    };
    if found != judged {
        return Err(Refusal::Changed { path: claimed.to_path_buf(), judged, found });
    }
    let mut seen: std::collections::HashSet<Identity> = std::collections::HashSet::new();
    return delete_entry(claimed, &mut seen).map_err(|error| { return Refusal::Io { path: claimed.to_path_buf(), error }; });
}

/// What:
/// `pub fn empty_trash(trash: &Path) -> Vec<String>` deletes what an earlier run claimed into this trash directory and did not
/// finish deleting (it was stopped between the move and the delete),
/// and returns a reason for each entry it could not delete.
/// Why:
/// Everything in a trash directory was judged removable and claimed by a rename;
/// the trash is the tools' own,
/// so finishing the deletion discards nothing that was not already judged.
/// Each entry is deleted under the same identity check as any claimed entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function emptyTrash(trash: string): string[]
/// ```
pub fn empty_trash(trash: &Path) -> Vec<String> {
    let mut failures: Vec<String> = Vec::new();
    let Ok(entries) = fs::read_dir(trash) else { return failures };
    for entry in entries.flatten() {
        let judged: Identity = match identity_of(&entry.path()) {
            Ok(found) => found,
            Err(error) => {
                failures.push(format!("{}: {error}", entry.path().display()));
                continue;
            }
        };
        match delete_claimed(&entry.path(), judged) {
            Ok(_) | Err(Refusal::Gone(_)) => {}
            Err(refusal) => failures.push(refusal.to_string()),
        }
    }
    return failures;
}
