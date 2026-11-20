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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const store = readStore('/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities');
//! const recorded = store.slot('my-worktree'); // or the parent record
//! ```

use super::clone_folder_identity::CloneFolderIdentity;
use super::slot_identity_store_records::{STORE_HEADER, StoredIdentity, parse_records, slot_name_refusal};
use super::slot_identity_store_refusals::{StoreRefusal, unreadable, write_refusal};
use crate::diagnostic_log::{debug, warn};
use crate::private_storage::{open_regular_file, replace_private_file, sync_directory};
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

/// Module tag for diagnostics.
const TAG: &str = "slotIdentityStore";

/// Where the route keeps the identities it recorded,
/// under root control.
pub const SLOT_IDENTITY_STORE_PATH: &str = "/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities";

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
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const store = readStore(path);
    /// ```
    ///
    /// # Errors
    /// The file's absence,
    /// an unreadable or non-regular file,
    /// a line that is not one this module wrote,
    /// or a key recorded twice.
    pub fn read(path: &Path) -> Result<SlotIdentityStore, StoreRefusal> {
        let mut file: std::fs::File = open_regular_file(path).map_err(|refusal| return unreadable(path, refusal))?;
        let mut text: String = String::new();
        file.read_to_string(&mut text).map_err(|error| return StoreRefusal::Unreadable { path: path.to_path_buf(), reason: error.to_string() })?;
        let records: Vec<StoredIdentity> = parse_records(path, text.as_str())?;
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

/// What:
///  Record the identity of a slot that has none.
/// Why:
///   The first sweep that verifies a clone records its slot,
/// and a later run must never rewrite that record:
///  a slot identity is not refreshed because a new inode or mode appeared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// appendSlotIdentity(store, 'my-worktree', identity); // throws when the slot already has one
/// ```
///
/// # Errors
/// The read refusal of the store,
/// the refusal of a name that is no plain name,
/// or the refusal of a slot that is already recorded.
pub fn append_slot_identity(store: &Path, name: &str, identity: &CloneFolderIdentity) -> Result<(), StoreRefusal> {
    if let Some(reason) = slot_name_refusal(name) {
        return Err(StoreRefusal::SlotName { slot: name.to_string(), reason });
    }
    let read: SlotIdentityStore = SlotIdentityStore::read(store)?;
    if read.slot(name).is_some() {
        return Err(StoreRefusal::SlotAlreadyRecorded { path: store.to_path_buf(), slot: name.to_string() });
    }
    if read.parent().is_none() {
        return Err(StoreRefusal::NoParentRecord { path: store.to_path_buf() });
    }
    let line: String = StoredIdentity::Slot { name: name.to_string(), identity: identity.clone() }.encode();
    debug(TAG, "record the identity of one clone slot");
    return append_one_line(store, line.as_str());
}

/// What:
///  Replace the parent folder's record,
///  keeping every slot record as it is.
/// Why:
///   The administrator's adoption is the only write that may replace the parent's identity,
///  and it must never overwrite a surviving slot identity
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Descriptor mount and unmount").
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

/// What:
///  Append one line to the store in one write,
///  never following a link at its name.
/// Why:
///   The store is read by other invocations while one writes,
/// so a line lands whole or not at all,
///  and a short write is reported rather than completed silently.
fn append_one_line(store: &Path, line: &str) -> Result<(), StoreRefusal> {
    let mut options: std::fs::OpenOptions = std::fs::OpenOptions::new();
    options.append(true).create(false).custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let mut file: std::fs::File = options.open(store).map_err(|error| return write_refusal(store, error))?;
    let bytes: Vec<u8> = format!("{line}\n").into_bytes();
    let written: usize = file.write(&bytes).map_err(|error| return write_refusal(store, error))?;
    if written != bytes.len() {
        warn(TAG, "an append to the recorded slot identities was short");
        return Err(StoreRefusal::Unreadable { path: store.to_path_buf(), reason: format!("the append wrote {written} of {} bytes, so the file holds a torn record", bytes.len()) });
    }
    file.sync_all().map_err(|error| return write_refusal(store, error))?;
    if let Some(directory) = store.parent() {
        sync_directory(directory).map_err(|error| return write_refusal(store, error))?;
    }
    return Ok(());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "slot_identity_store_tests.rs"]
mod tests;
