//! What:
//!  The file that records one identity for the `.claude/worktrees` parent folder and one for each
//!  worktree clone slot,
//!  and what reading,
//!  adding and replacing a record do.
//! Why:
//!  The route must compare a folder it is about to attach a clone to against the identity it recorded,
//!  and it must never refresh that record just because a new inode or a new mode appeared
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount").
//!  The file is under root control at `SLOT_IDENTITY_STORE_PATH`:
//!  the parent record is written once by the setup run,
//!  a slot record is written once when the sweep first verifies a clone,
//!  and only the administrator's adoption decision replaces the parent record.
//!  The route reads it through `read_under_root_control`,
//!  which refuses a file or a folder this account could rewrite,
//!  because an identity the account can change decides nothing,
//!  and it writes through `append_slot_identity_under_root_control` and
//!  `record_parent_identity_under_root_control`,
//!  which make the same refusal inside the step that holds the lock.
//!  The lock that keeps two appends apart stands on a file beside the store that only root may open,
//!  because an exclusive `flock` needs no write bit:
//!  a lock on the store itself is one the account that may read it could hold against root's own sweep.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const store = readStoreUnderRootControl('/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities');
//! const recorded = store.slot('my-worktree'); // or the parent record
//! ```

use super::clone_folder_identity::{CloneFolderIdentity, PERMISSION_BITS};
use super::slot_identity_store_records::{STORE_HEADER, StoredIdentity, parse_records, slot_name_refusal};
use super::slot_identity_store_refusals::{StoreRefusal, unreadable, write_refusal};
use crate::diagnostic_log::{debug, warn};
use crate::private_storage::{open_regular_file, read_regular_file, replace_private_file, sync_directory};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

/// Module tag for diagnostics.
const TAG: &str = "slotIdentityStore";

/// Where the route keeps the identities it recorded,
/// under root control.
pub const SLOT_IDENTITY_STORE_PATH: &str = "/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities";

/// What:
///  The largest store one read accepts.
/// Why:
///   A store is a header and one line per slot of one checkout,
///  and a runaway or replaced file must be named rather than read into memory
///  (`private_storage::read_regular_file`).
///  A slot line holds a folder name of at most 255 bytes and an identity whose ACLs are in practice tens of bytes,
///  so 4096 slots of 256 bytes fit with the header inside one mebibyte;
///  a checkout with more worktrees than that needs this bound raised,
///  not a larger read.
pub const STORE_MAXIMUM_BYTES: u64 = 1 << 20;

/// The owner of a store,
/// and of the folder that holds it:
/// root,
/// because only the root-run setup and adopt modes write either.
pub const STORE_OWNER: u32 = 0;

/// The permission bits that let an account other than the owner write:
/// group write and other write.
pub const WRITABLE_BY_OTHERS: u32 = 0o022;

/// What:
///  Why a file or folder this route reads is not the one under root control,
///  or nothing when it is.
/// Why:
///   A record decides which folder a clone may be attached to,
///  so a store the attach's account could rewrite would make every recorded identity advisory.
///  A store owned by root and holding no group or other write bit is one only root writes;
///  the account the attach runs as may still read it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = storeControlRefusal(owner, mode & 0o7777); // string | undefined
/// ```
pub fn store_control_refusal(owner: u32, mode: u32) -> Option<String> {
    if owner != STORE_OWNER {
        return Some(format!(
            "its owner is user {owner} and not root, so the account that owns it could rewrite the identities this route compares against"
        ));
    }
    if mode & WRITABLE_BY_OTHERS != 0 {
        return Some(format!(
            "its permission bits are {mode:04o}, and {WRITABLE_BY_OTHERS:04o} would let another account write it"
        ));
    }
    return None;
}

/// What:
///  The store as it was read:
///  the identities it held and the file they came from.
/// Why:
///   A refusal quotes the file and a lookup answers from one read,
/// so two lookups of one attach see one state of the store.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Store = { path: string; records: StoredIdentity[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlotIdentityStore {
    /// The file the records were read from.
    path: PathBuf,
    /// The records,
    /// the parent first when it is there.
    records: Vec<StoredIdentity>,
}

impl SlotIdentityStore {
    /// What:
    ///  Read the store at `path`.
    /// Why:
    ///   A record decides whether a folder may be attached to,
    /// so it is read whole and any line this module cannot read refuses the read:
    ///  a skipped line could hide a slot that would then be treated as new.
    ///  The read is bounded (`STORE_MAXIMUM_BYTES`) and never follows a link at the name.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const store = readStore(path);
    /// ```
    ///
    /// # Errors
    /// The file's absence,
    /// a read larger than `STORE_MAXIMUM_BYTES`,
    /// an unreadable or non-regular file,
    /// text that is not UTF-8,
    /// a line that is not one this module wrote,
    /// or a key recorded twice.
    pub fn read(path: &Path) -> Result<SlotIdentityStore, StoreRefusal> {
        let bytes: Vec<u8> = read_regular_file(path, STORE_MAXIMUM_BYTES).map_err(|refusal| return unreadable(path, refusal, STORE_MAXIMUM_BYTES))?;
        let text: String = String::from_utf8(bytes).map_err(|_error| return StoreRefusal::Unreadable { path: path.to_path_buf(), reason: String::from("the file is not UTF-8 text") })?;
        return SlotIdentityStore::from_text(path, text.as_str());
    }

    /// What:
    ///  Read the store the route acts on:
    ///  the file and the folder that holds it must be under root control,
    ///  then the store reads as `read` reads it.
    /// Why:
    ///   The route runs as root,
    /// so a store the attach's account could rewrite is not the record the design rests on;
    ///  this is the reader every attach and every adoption uses,
    ///  and `read` stays the grammar's own reader for a caller that already holds such a file.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const store = readStoreUnderRootControl('/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities');
    /// ```
    ///
    /// # Errors
    /// Everything `read` refuses,
    /// and a file or folder whose owner is not root or whose group or other write bit is set.
    pub fn read_under_root_control(path: &Path) -> Result<SlotIdentityStore, StoreRefusal> {
        let file: std::fs::File = open_regular_file(path).map_err(|refusal| return unreadable(path, refusal, STORE_MAXIMUM_BYTES))?;
        check_control(path, "the store file", &file)?;
        check_the_folder_control(path)?;
        return SlotIdentityStore::read(path);
    }

    /// The store `text` describes,
    /// with `path` as the file it came from.
    fn from_text(path: &Path, text: &str) -> Result<SlotIdentityStore, StoreRefusal> {
        let records: Vec<StoredIdentity> = parse_records(path, text)?;
        debug(TAG, "read the recorded slot identities");
        return Ok(SlotIdentityStore { path: path.to_path_buf(), records });
    }

    /// The file these records were read from.
    pub fn path(&self) -> &Path {
        return self.path.as_path();
    }

    /// The recorded identity of the `.claude/worktrees` parent folder,
    /// when the setup run wrote one.
    pub fn parent(&self) -> Option<&CloneFolderIdentity> {
        for record in &self.records {
            if let StoredIdentity::Parent(identity) = record {
                return Some(identity);
            }
        }
        return None;
    }

    /// The recorded identity of the slot named `name`,
    /// when the sweep recorded one.
    pub fn slot(&self, name: &str) -> Option<&CloneFolderIdentity> {
        for record in &self.records {
            if let StoredIdentity::Slot { name: recorded, identity } = record && recorded == name {
                return Some(identity);
            }
        }
        return None;
    }

    /// The names of every recorded slot.
    pub fn slot_names(&self) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        for record in &self.records {
            if let StoredIdentity::Slot { name, .. } = record {
                names.push(name.clone());
            }
        }
        return names;
    }
}

/// The folder that holds the store at `path`,
/// which is the working directory for a bare file name.
fn store_folder(path: &Path) -> PathBuf {
    return match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    };
}

/// Whether the folder that holds the store at `path` is under root control,
/// and the refusal naming what is not when it is not.
fn check_the_folder_control(path: &Path) -> Result<(), StoreRefusal> {
    let folder: PathBuf = store_folder(path);
    let held: std::fs::File = std::fs::File::open(&folder).map_err(|error| return unreadable(folder.as_path(), crate::private_storage::ReadRefusal::Io(error), STORE_MAXIMUM_BYTES))?;
    return check_control(folder.as_path(), "the folder that holds the store", &held);
}

/// Whether the open file at `path` is under root control,
/// and the refusal naming what is not when it is not.
fn check_control(path: &Path, what: &str, held: &std::fs::File) -> Result<(), StoreRefusal> {
    let metadata: std::fs::Metadata = held.metadata().map_err(|error| return StoreRefusal::Unreadable { path: path.to_path_buf(), reason: error.to_string() })?;
    let refusal: Option<String> = store_control_refusal(metadata.uid(), metadata.mode() & PERMISSION_BITS);
    return match refusal {
        Some(reason) => Err(StoreRefusal::NotUnderRootControl { path: path.to_path_buf(), what: what.to_string(), reason }),
        None => Ok(()),
    };
}

/// What:
///  Record the identity of a slot that has none,
///  for a caller that already holds a store under root control.
/// Why:
///   The first sweep that verifies a clone records its slot,
///  and a later run must never rewrite that record:
///  a slot identity is not refreshed because a new inode or mode appeared.
///  The read,
///  the check and the append are one step under one exclusive lock of the lock file beside the store,
///  so two sweeps that meet on one slot record it once:
///  without that,
///  both would read no record,
///  both would append,
///  and the duplicate would refuse every later
///  read of the store until an administrator edited a root-owned file.
///  This is the grammar's own writer,
///  which a control drives on a fixture store;
///  the route writes through `append_slot_identity_under_root_control`,
///  which makes the root-control check inside the same step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// appendSlotIdentity(store, 'my-worktree', identity); // throws when the slot already has one
/// ```
///
/// # Errors
/// The read refusal of the store,
/// the refusal of a name that is no plain name,
/// the refusal of a slot that is already recorded,
/// the refusal of a lock another run holds past the wait,
/// or the write's own failure.
pub fn append_slot_identity(store: &Path, name: &str, identity: &CloneFolderIdentity) -> Result<(), StoreRefusal> {
    let mut file: std::fs::File = open_the_store(store)?;
    let _lock: std::fs::File = hold_the_store_lock(store)?;
    return append_under_the_lock(&mut file, store, name, identity);
}

/// What:
///  Record the identity of a slot that has none,
///  refusing a store or a lock this account could rewrite or hold.
/// Why:
///   A record decides which folder a clone may be attached to,
///  so the route writes only a store whose file and folder are root's
///  and a lock only root may open.
///  The check happens inside the step that holds the lock,
///  so a swap between a caller's own read and this append cannot change what is written;
///  a caller that read first covers nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// appendSlotIdentityUnderRootControl(store, 'my-worktree', identity); // refuses a store or a lock this account could change
/// ```
///
/// # Errors
/// Everything `append_slot_identity` refuses,
/// and a store file,
/// a folder or a lock file whose owner is not root or which another account may write or open.
pub fn append_slot_identity_under_root_control(store: &Path, name: &str, identity: &CloneFolderIdentity) -> Result<(), StoreRefusal> {
    let mut file: std::fs::File = open_the_store(store)?;
    check_control(store, "the store file", &file)?;
    check_the_folder_control(store)?;
    let lock: std::fs::File = hold_the_store_lock(store)?;
    check_lock_control(store_lock_path(store).as_path(), &lock)?;
    return append_under_the_lock(&mut file, store, name, identity);
}

/// The read,
/// the checks and the append of one slot record,
/// which every caller makes while it holds the store's lock.
fn append_under_the_lock(file: &mut std::fs::File, store: &Path, name: &str, identity: &CloneFolderIdentity) -> Result<(), StoreRefusal> {
    if let Some(reason) = slot_name_refusal(name) {
        return Err(StoreRefusal::SlotName { slot: name.to_string(), reason });
    }
    let text: String = read_through(file, store)?;
    let read: SlotIdentityStore = SlotIdentityStore::from_text(store, text.as_str())?;
    if read.slot(name).is_some() {
        return Err(StoreRefusal::SlotAlreadyRecorded { path: store.to_path_buf(), slot: name.to_string() });
    }
    if read.parent().is_none() {
        return Err(StoreRefusal::NoParentRecord { path: store.to_path_buf() });
    }
    let line: String = StoredIdentity::Slot { name: name.to_string(), identity: identity.clone() }.encode();
    #[cfg(test)]
    BETWEEN_THE_READ_AND_THE_APPEND.with(|hook| { if let Some(between) = hook.borrow().as_ref() { between(); } });
    debug(TAG, "record the identity of one clone slot");
    return append_line(file, store, line.as_str());
}

/// What:
///  Replace the parent folder's record,
///  keeping every slot record as it is.
/// Why:
///   The administrator's adoption is the only write that may replace the parent's identity,
///  and it must never overwrite a surviving slot identity
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Descriptor mount and unmount").
///  Its own read and the rename that publishes the new file are not one step,
///  so an append another process made between them is dropped by the rename:
///  the mode runs with managed operations quiesced,
///  which is what keeps a surviving slot record from being lost.
///  This is the grammar's own writer,
///  which a control drives on a fixture store;
///  the route writes through `record_parent_identity_under_root_control`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordParentIdentity(store, adoptedIdentity);
/// ```
///
/// # Errors
/// The read refusal of the store,
/// or the write's own failure,
/// which leaves the file as it was.
pub fn record_parent_identity(store: &Path, identity: &CloneFolderIdentity) -> Result<(), StoreRefusal> {
    let read: SlotIdentityStore = SlotIdentityStore::read(store)?;
    return replace_the_parent_record(store, &read, identity);
}

/// What:
///  Replace the parent folder's record,
///  refusing a store whose file or folder this account could rewrite.
/// Why:
///   The adoption is the one write that may replace the parent identity,
///  so the route makes it only on a store that is root's;
///  reading first and then writing would check a file the caller no longer looks at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordParentIdentityUnderRootControl(store, adoptedIdentity);
/// ```
///
/// # Errors
/// Everything `record_parent_identity` refuses,
/// and a store file or folder whose owner is not root or which another account may write.
pub fn record_parent_identity_under_root_control(store: &Path, identity: &CloneFolderIdentity) -> Result<(), StoreRefusal> {
    let read: SlotIdentityStore = SlotIdentityStore::read_under_root_control(store)?;
    return replace_the_parent_record(store, &read, identity);
}

/// Write the store `read` describes with the parent record `identity`,
/// keeping every slot record and publishing the whole file by rename.
fn replace_the_parent_record(store: &Path, read: &SlotIdentityStore, identity: &CloneFolderIdentity) -> Result<(), StoreRefusal> {
    let mut lines: Vec<String> = vec![String::from(STORE_HEADER), StoredIdentity::Parent(identity.clone()).encode()];
    for record in &read.records {
        if matches!(record, StoredIdentity::Parent(_identity)) {
            continue;
        }
        lines.push(record.encode());
    }
    let mut text: String = lines.join("\n");
    text.push('\n');
    debug(TAG, "replace the recorded identity of the worktrees parent folder");
    replace_private_file(store, text.as_bytes()).map_err(|error| return write_refusal(store, error))?;
    return Ok(());
}

/// Where the lock that keeps two appends apart stands:
/// a file beside the store.
/// Why:
///  A lock on the store's own descriptor is a lock on a file the attach's account may read:
///  an exclusive `flock` needs no write bit,
///  so that account would hold it and stall root's sweep.
///  This file is made root's own with no other bit (`STORE_LOCK_MODE`),
///  so an account that may read the store cannot open it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lock = store + '.lock';
/// ```
pub fn store_lock_path(store: &Path) -> PathBuf {
    let mut name: std::ffi::OsString = store.as_os_str().to_os_string();
    name.push(".lock");
    return PathBuf::from(name);
}

/// The mode the lock file is made with:
/// root's own,
/// with no group or other bit,
/// so no other account may even open it.
pub const STORE_LOCK_MODE: u32 = 0o600;

/// The permission bits that let an account other than the owner open a lock file.
const READABLE_OR_WRITABLE_BY_OTHERS: u32 = 0o077;

/// How long an append waits for another run's lock before it refuses.
/// Why:
///  The only holder is another append or the adoption's rewrite of this same store,
///  which is one line written and two `fsync`s;
///  the bound is the same two seconds the removal's claim wait ships with
///  (`lock/abandoned_lock_proving.rs`),
///  and it ends the wait rather than holding a command open.
pub const STORE_LOCK_WAIT: std::time::Duration = std::time::Duration::from_secs(2);

/// The pause between two asks for the lock,
/// which is short enough that a held lock is taken again as soon as it is free.
pub const STORE_LOCK_PAUSE: std::time::Duration = std::time::Duration::from_millis(2);

/// The wait this run allows for the lock,
/// which a control may shorten.
fn the_lock_wait_allowed() -> std::time::Duration {
    #[cfg(test)]
    {
        if let Some(allowed) = THE_LOCK_WAIT_ALLOWED.with(|allowed| { return allowed.borrow().clone(); }) {
            return allowed;
        }
    }
    return STORE_LOCK_WAIT;
}

/// Open the store for the read and the append that record one slot,
/// refusing a link at its name and any non-regular file.
fn open_the_store(store: &Path) -> Result<std::fs::File, StoreRefusal> {
    let mut options: std::fs::OpenOptions = std::fs::OpenOptions::new();
    options.read(true).append(true).custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let file: std::fs::File = options.open(store).map_err(|error| return match error.raw_os_error() {
        Some(libc::ELOOP) => StoreRefusal::Write { path: store.to_path_buf(), reason: String::from("the name holds a symbolic link, and nothing is written through a link") },
        _ => write_refusal(store, error),
    })?;
    // The descriptor's own type, so a swap between the open and the check cannot change it.
    let metadata: std::fs::Metadata = file.metadata().map_err(|error| return write_refusal(store, error))?;
    if !metadata.is_file() {
        return Err(StoreRefusal::Write { path: store.to_path_buf(), reason: String::from("the name holds a link, a directory or another non-regular file") });
    }
    return Ok(file);
}

/// What:
///  Open the lock file beside the store and hold it exclusively,
///  asking again until the wait this run allows has passed.
/// Why:
///   The read and the append of one slot are one step,
///  and one exclusive `flock` of one small file is the smallest unit that covers it.
///  The lock file is root's own (`STORE_LOCK_MODE`),
///  so the account that may read the store cannot open it and cannot hold the lock.
///  It is asked for without waiting (`LOCK_NB`) under `STORE_LOCK_WAIT`,
///  because the kernel's `flock` has no timeout on Linux and a holder that never finishes must refuse rather than
///  hold the command open.
///  The kernel releases the lock when the descriptor closes however this call ends,
///  so a run that is killed leaves no lock behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lock = open(store + '.lock', O_RDWR | O_CREAT | O_NOFOLLOW, 0o600); flock(lock.fd, LOCK_EX | LOCK_NB) // until the wait passes
/// ```
fn hold_the_store_lock(store: &Path) -> Result<std::fs::File, StoreRefusal> {
    let lock: PathBuf = store_lock_path(store);
    let mut options: std::fs::OpenOptions = std::fs::OpenOptions::new();
    options.read(true).write(true).create(true).mode(STORE_LOCK_MODE).custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let file: std::fs::File = options.open(lock.as_path()).map_err(|error| return match error.raw_os_error() {
        Some(libc::ELOOP) => StoreRefusal::Write { path: lock.clone(), reason: String::from("the name holds a symbolic link, and nothing is written through a link") },
        _ => write_refusal(lock.as_path(), error),
    })?;
    let metadata: std::fs::Metadata = file.metadata().map_err(|error| return write_refusal(lock.as_path(), error))?;
    if !metadata.is_file() {
        return Err(StoreRefusal::Write { path: lock.clone(), reason: String::from("the name holds a directory or another non-regular file") });
    }
    let began: std::time::Instant = std::time::Instant::now();
    loop {
        // Lock kept: one append of one slot record, held while the store is read and added to; the lock file is
        // root's own (mode 0600), so an account that may read the store cannot take it, and the ask is bounded by
        // `STORE_LOCK_WAIT` (`AGENTS.md`, rule NLK).
        // SAFETY: `flock` reads the descriptor and the flag only, and touches no memory of ours.
        let taken: libc::c_int = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        if taken == 0 {
            return Ok(file);
        }
        let error: std::io::Error = std::io::Error::last_os_error();
        // Only a lock another run holds is worth waiting for; a file system without `flock` answers something else.
        if error.raw_os_error() != Some(libc::EWOULDBLOCK) {
            return Err(write_refusal(lock.as_path(), error));
        }
        if began.elapsed() >= the_lock_wait_allowed() {
            return Err(StoreRefusal::Locked { path: lock, waited: the_lock_wait_allowed() });
        }
        std::thread::sleep(STORE_LOCK_PAUSE);
    }
}

/// Whether the lock file at `path` is one only root may open,
/// and the refusal naming what lets another account open it when it is not.
fn check_lock_control(path: &Path, held: &std::fs::File) -> Result<(), StoreRefusal> {
    let metadata: std::fs::Metadata = held.metadata().map_err(|error| return StoreRefusal::Unreadable { path: path.to_path_buf(), reason: error.to_string() })?;
    let mode: u32 = metadata.mode() & PERMISSION_BITS;
    let reason: Option<String> = if metadata.uid() != STORE_OWNER {
        Some(format!("its owner is user {} and not root, so that account could open it and hold the lock and stall this run", metadata.uid()))
    } else if mode & READABLE_OR_WRITABLE_BY_OTHERS != 0 {
        Some(format!("its permission bits are {mode:04o}, and {READABLE_OR_WRITABLE_BY_OTHERS:04o} would let another account open it and hold the lock"))
    } else {
        None
    };
    return match reason {
        Some(reason) => Err(StoreRefusal::NotUnderRootControl { path: path.to_path_buf(), what: String::from("the lock that keeps two appends apart"), reason }),
        None => Ok(()),
    };
}

/// The text of the store,
/// read through the descriptor the append holds,
/// up to `STORE_MAXIMUM_BYTES`.
fn read_through(file: &std::fs::File, store: &Path) -> Result<String, StoreRefusal> {
    let mut text: String = String::new();
    file.take(STORE_MAXIMUM_BYTES + 1)
        .read_to_string(&mut text)
        .map_err(|error| return StoreRefusal::Unreadable { path: store.to_path_buf(), reason: error.to_string() })?;
    if text.len() as u64 > STORE_MAXIMUM_BYTES {
        return Err(StoreRefusal::Unreadable {
            path: store.to_path_buf(),
            reason: format!("the file is larger than the {STORE_MAXIMUM_BYTES} bytes one store read accepts"),
        });
    }
    return Ok(text);
}

/// What:
///  Append one written line to the store,
///  then flush it and the folder that holds it.
/// Why:
///   The store is read by other invocations while one writes,
/// so a line lands whole or not at all,
///  and a short write is reported rather than completed silently.
fn append_line(file: &mut std::fs::File, store: &Path, line: &str) -> Result<(), StoreRefusal> {
    let bytes: Vec<u8> = format!("{line}\n").into_bytes();
    let written: usize = file.write(&bytes).map_err(|error| return write_refusal(store, error))?;
    if written != bytes.len() {
        warn(TAG, "an append to the recorded slot identities was short");
        return Err(StoreRefusal::Write { path: store.to_path_buf(), reason: format!("the append wrote {written} of {} bytes, so the file holds a torn record", bytes.len()) });
    }
    file.sync_all().map_err(|error| return write_refusal(store, error))?;
    if let Some(directory) = store.parent() {
        sync_directory(directory).map_err(|error| return write_refusal(store, error))?;
    }
    return Ok(());
}

#[cfg(test)]
thread_local! {
    /// A control may run another append in another thread while this one holds the store's lock,
    /// which is where two sweeps recording one slot meet,
    /// and may shorten the wait for a lock another run holds.
    pub(crate) static BETWEEN_THE_READ_AND_THE_APPEND: std::cell::RefCell<Option<Box<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
    /// The wait for the lock this run allows,
    /// which a control shortens so a refusal is reached without holding the suite for the shipped bound.
    pub(crate) static THE_LOCK_WAIT_ALLOWED: std::cell::RefCell<Option<std::time::Duration>> = const { std::cell::RefCell::new(None) };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "slot_identity_store_tests.rs"]
mod tests;
