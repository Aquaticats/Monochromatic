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
//! A claimed entry may hold a directory its owner cannot write (`0500`,
//! `0555`,
//! `0000`):
//! unlinking a file needs write and search on the directory that holds it,
//! and listing a directory needs read on it,
//! so the deletion gives the directory those three bits first when this user owns it,
//! and reports that it did.
//! A directory of another user is left as it is and named.
//! Only directories the walk enters are changed,
//! and a symbolic link is always removed as a link,
//! so nothing outside the claimed entry is touched.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function removeOne(item: Item, trash: string): Deleted  // throws Refusal
//! ```

/// Files,
/// directories and metadata,
/// and the effective user id that decides whose directory may be changed.
use std::{fs, io, os::unix::fs::{MetadataExt, PermissionsExt}, path::{Path, PathBuf}};

/// The effective user id,
/// which is the user a removal acts as.
use rustix::process::geteuid;

/// The rename that fails instead of replacing.
use rustix::fs::{CWD, RenameFlags, renameat_with};

/// Device and inode.
use crate::lean_identity::{Identity, identity_of};
/// The final component of a path as text.
use crate::lean_text::name_of;

/// The name of the trash directory each tool keeps beside what it removes from.
pub const TRASH_DIRECTORY_NAME: &str = ".cargo-shadow-trash";

/// The owner's read,
/// write and search bits:
/// what a directory needs before the walk can name what it holds and unlink it.
/// All three are added together because a directory this user can list but not unlink in (`0555`) and one this user
/// can neither list nor unlink in (`0000`) need the same walk to get through.
const OWNER_ACCESS_BITS: u32 = 0o700;

/// How many changed directories a proof line names before it counts the rest.
const NAMED_MODES: usize = 5;

/// One thing to remove,
/// with the identity it had when it was classified.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Item {
    /// Where it is.
    pub path: PathBuf,
    /// What it was when judged.
    pub identity: Identity,
}

/// What a completed deletion did.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Deleted {
    /// The disk bytes of the files whose last link in this walk went.
    pub bytes: u64,
    /// The directories whose owner read,
    /// write and search permission this deletion added so it could remove what they held.
    pub modes_added: Vec<PathBuf>,
    /// One reason for each part of the entry that stayed.
    pub remaining: Vec<String>,
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
    Gone(
        /// The path that was already removed.
        PathBuf,
    ),
    /// A system call failed.
    Io {
        /// The path.
        path: PathBuf,
        /// The error.
        error: io::Error,
    },
    /// Part of the entry is gone and part of it stayed:
    /// a directory the walk reached could not be emptied,
    /// so what lies under it is still there.
    PartlyRemoved {
        /// The claimed entry.
        path: PathBuf,
        /// What went and what stayed.
        deleted: Deleted,
    },
}

/// The text a refusal shows.
impl std::fmt::Display for Refusal {
    /// Writes the refusal as text.
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
            Refusal::PartlyRemoved { path, deleted } => write!(
                formatter,
                "{} was only partly removed: {} disk bytes are gone, {} part(s) of it stay{}, and each stayed because: {}",
                path.display(),
                deleted.bytes,
                deleted.remaining.len(),
                modes_named(&deleted.modes_added),
                deleted.remaining.join("; ")
            ),
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
/// The process id,
/// the clock and a serial make each claim unique,
/// so the rename never has to replace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const trashName = (trash: string, item: string, serial: number) => join(trash, `${process.pid}-${Date.now()}-${serial}-${basename(item)}`);
/// ```
fn trash_name(trash: &Path, item: &Path, serial: u64) -> PathBuf {
    let nanos: u128 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_nanos(),
        Err(_) => 0,
    };
    let base: String = name_of(item);
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
    if let Err(error) = fs::create_dir_all(trash) {
        return Err(Refusal::Io { path: trash.to_path_buf(), error });
    }
    let target: PathBuf = trash_name(trash, &item.path, serial);
    match renameat_with(CWD, &item.path, CWD, &target, RenameFlags::NOREPLACE) {
        Ok(()) => {}
        Err(code) if code == rustix::io::Errno::NOENT => return Err(Refusal::Gone(item.path.clone())),
        Err(code) => return Err(Refusal::Io { path: item.path.clone(), error: io::Error::from(code) }),
    }
    let after: Identity = match identity_of(&target) {
        Ok(found) => found,
        Err(error) => return Err(Refusal::Io { path: target, error }),
    };
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
/// `pub fn modes_named(modes: &[PathBuf]) -> String` is the clause a proof line carries when a removal changed the mode of
/// a directory,
/// naming a few of them and counting the rest.
/// Why:
/// Changing a mode is a mutation of the tree before it goes,
/// so the line says it happened,
/// as it says which locks were taken.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const modesNamed = (modes: string[]): string => modes.length === 0 ? '' : `; mode changed on ${modes.length} directories this user could not write: ${modes.slice(0, 5).join(', ')}${modes.length > 5 ? ` and ${modes.length - 5} more` : ''}`;
/// ```
pub fn modes_named(modes: &[PathBuf]) -> String {
    if modes.is_empty() {
        return String::new();
    }
    let mut named: Vec<String> = Vec::with_capacity(NAMED_MODES);
    for mode in modes.iter().take(NAMED_MODES) {
        named.push(mode.display().to_string());
    }
    let rest: String = match modes.len().checked_sub(NAMED_MODES) {
        Some(more) if more > 0 => format!(" and {more} more"),
        _ => String::new(),
    };
    return format!("; mode changed on {} directories this user could not write: {}{rest}", modes.len(), named.join(", "));
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
fn open_directory(path: &Path, metadata: &fs::Metadata, walk: &mut Walk<'_>) -> Option<String> {
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
        Err(error) => Some(format!("{} could not be made writable (mode {mode:04o}): {error}", path.display())),
    };
}

/// What a walk carries from one entry to the next.
struct Walk<'a> {
    /// The user the removal runs as,
    /// which decides whose directory mode it may change.
    this_uid: u32,
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
fn delete_entry(path: &Path, walk: &mut Walk<'_>) {
    let metadata: fs::Metadata = match fs::symlink_metadata(path) {
        Ok(found) => found,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return,
        Err(error) => {
            walk.deleted.remaining.push(format!("{} could not be read: {error}", path.display()));
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
                Err(error) if error.kind() == io::ErrorKind::DirectoryNotEmpty => walk.deleted.remaining.push(reason),
                Err(error) => walk.deleted.remaining.push(format!("{reason}, and it could not be removed either: {error}")),
            }
            return;
        }
        match fs::read_dir(path) {
            Ok(entries) => {
                for child in entries {
                    match child {
                        Ok(entry) => delete_entry(&entry.path(), walk),
                        Err(error) => walk.deleted.remaining.push(format!("{} holds an entry that could not be read: {error}", path.display())),
                    }
                }
            }
            Err(error) => walk.deleted.remaining.push(format!("{} could not be listed: {error}", path.display())),
        }
        match fs::remove_dir(path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) if error.kind() == io::ErrorKind::DirectoryNotEmpty => {}
            Err(error) => walk.deleted.remaining.push(format!("{} could not be removed: {error}", path.display())),
        }
        return;
    }
    let first: bool = metadata.nlink() <= 1 || walk.seen.insert(Identity { device: metadata.dev(), inode: metadata.ino() });
    if first {
        walk.deleted.bytes += allocated_bytes_of(&metadata);
    }
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => walk.deleted.remaining.push(format!("{} could not be removed: {error}", path.display())),
    }
}

/// What:
/// `pub fn delete_claimed(claimed: &Path, judged: Identity) -> Result<Deleted, Refusal>` deletes a claimed entry after
/// checking its identity once more,
/// and reports what it removed.
/// Why:
/// The device and inode are checked just before the unlink,
/// so nothing but the judged entry is ever deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function deleteClaimed(claimed: string, judged: Identity): Deleted  // throws Refusal
/// ```
pub fn delete_claimed(claimed: &Path, judged: Identity) -> Result<Deleted, Refusal> {
    return delete_claimed_as(claimed, judged, geteuid().as_raw());
}

/// What:
/// `pub(crate) fn delete_claimed_as(claimed: &Path, judged: Identity, this_uid: u32) -> Result<Deleted, Refusal>` deletes a
/// claimed entry as the given user.
/// Why:
/// The directory whose mode a removal may change is the one this user owns,
/// and a test can act as a user who owns nothing of the fixture.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function deleteClaimedAs(claimed: string, judged: Identity, thisUid: number): Deleted  // throws Refusal
/// ```
pub(crate) fn delete_claimed_as(claimed: &Path, judged: Identity, this_uid: u32) -> Result<Deleted, Refusal> {
    let found: Identity = match identity_of(claimed) {
        Ok(identity) => identity,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Err(Refusal::Gone(claimed.to_path_buf())),
        Err(error) => return Err(Refusal::Io { path: claimed.to_path_buf(), error }),
    };
    if found != judged {
        return Err(Refusal::Changed { path: claimed.to_path_buf(), judged, found });
    }
    let mut seen: std::collections::HashSet<Identity> = std::collections::HashSet::new();
    let mut walk: Walk<'_> = Walk { this_uid, deleted: Deleted::default(), seen: &mut seen };
    delete_entry(claimed, &mut walk);
    return match walk.deleted.remaining.is_empty() {
        true => Ok(walk.deleted),
        false => Err(Refusal::PartlyRemoved { path: claimed.to_path_buf(), deleted: walk.deleted }),
    };
}

/// What:
/// `pub struct TrashSweep` is what finishing an earlier run's claims did.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrashSweep {
    /// The disk bytes the finished claims freed.
    pub bytes: u64,
    /// The directories whose owner access this sweep widened.
    pub modes_added: Vec<PathBuf>,
    /// One reason for each entry it could not finish removing.
    pub failures: Vec<String>,
}

/// What one finished deletion adds to a sweep.
impl TrashSweep {
    /// Adds what a finished deletion did.
    fn add(&mut self, deleted: Deleted) {
        self.bytes += deleted.bytes;
        self.modes_added.extend(deleted.modes_added);
    }
}

/// What:
/// `pub fn empty_trash(trash: &Path) -> TrashSweep` deletes what an earlier run claimed into this trash directory and did not
/// finish deleting (it was stopped between the move and the delete),
/// and reports what it freed and what it could not finish.
/// Why:
/// Everything in a trash directory was judged removable and claimed by a rename;
/// the trash is the tools' own,
/// so finishing the deletion discards nothing that was not already judged.
/// Each entry is deleted under the same identity check as any claimed entry,
/// and by the same walk,
/// so what a stopped run could not get through is finished here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function emptyTrash(trash: string): TrashSweep
/// ```
pub fn empty_trash(trash: &Path) -> TrashSweep {
    let mut sweep: TrashSweep = TrashSweep::default();
    let Ok(entries) = fs::read_dir(trash) else { return sweep };
    for entry in entries.flatten() {
        let judged: Identity = match identity_of(&entry.path()) {
            Ok(found) => found,
            Err(error) => {
                sweep.failures.push(format!("{}: {error}", entry.path().display()));
                continue;
            }
        };
        match delete_claimed(&entry.path(), judged) {
            Ok(deleted) => sweep.add(deleted),
            Err(Refusal::Gone(_)) => {}
            Err(Refusal::PartlyRemoved { deleted, .. }) => {
                let reason: String = Refusal::PartlyRemoved { path: entry.path(), deleted: deleted.clone() }.to_string();
                sweep.add(deleted);
                sweep.failures.push(reason);
            }
            Err(refusal) => sweep.failures.push(refusal.to_string()),
        }
    }
    return sweep;
}
