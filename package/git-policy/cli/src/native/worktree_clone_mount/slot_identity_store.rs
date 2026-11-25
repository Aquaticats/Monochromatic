//! What:
//!  The folder that records one identity for the `.claude/worktrees` parent folder and one for each
//!  worktree clone slot,
//!  held open by one descriptor,
//!  and the lookups and the listing a reader makes through it.
//! Why:
//!  The route must compare a folder it is about to attach a clone to against the identity it recorded,
//!  and it must never refresh that record just because a new inode or a new mode appeared
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount").
//!  The folder is under root control at `SLOT_IDENTITY_STORE_PATH`:
//!  `.parent` is published once by the setup run,
//!  a slot's record is published once when the sweep first verifies a clone,
//!  and only the administrator's adoption replaces the parent record.
//!  One file per slot is what lets each record be published with an exclusive creation and a link that
//!  replaces nothing,
//!  so the store takes no lock and no later writer can record an identity taken from a folder that moved.
//!  The folder is opened once and every record,
//!  rename and removal is named relative to that descriptor,
//!  so a name the account could change below the root-owned folder is never followed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const store = openStoreUnderRootControl('/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities');
//! const parent = store.parent(); const recorded = store.slot('my-worktree');
//! ```

use super::slot_identity_store_control::check_under_root_control;
use super::slot_identity_store_reading::{listed_slot_names, read_record};
use crate::entries_of_one_open_folder::open_folder;
use super::slot_identity_store_records::{PARENT_RECORD_NAME, StoredIdentity};
use super::slot_identity_store_refusals::StoreRefusal;
use super::slot_name::slot_name_refusal;
use crate::diagnostic_log::debug;
use std::path::Path;
use std::path::PathBuf;

/// Module tag for diagnostics.
const TAG: &str = "slotIdentityStore";

/// Where the route keeps the identities it recorded,
/// under root control:
/// a folder holding one record file per slot and `.parent` for the parent folder.
pub const SLOT_IDENTITY_STORE_PATH: &str = "/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities";

#[cfg(test)]
thread_local! {
    /// A control may catch every attempt to open a store folder,
    /// which is where a run that must read nothing stands,
    /// and which catches the attempt even where no folder is there to open.
    pub(crate) static BEFORE_THE_STORE_OPENS: std::cell::RefCell<Option<Box<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
}

/// What:
///  The store folder as one reader holds it:
///  its descriptor,
///  the path a refusal names,
///  and whether the records in it must answer the root-control terms.
/// Why:
///   Every record is named relative to the one descriptor,
///  so the folder judged is the folder held;
///  the route opens the folder under root control,
///  and the grammar's own open is what a control on a fixture folder uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Store = { folder: FileHandle; path: string; underRootControl: boolean };
/// ```
#[derive(Debug)]
pub struct SlotIdentityStore {
    /// The folder,
    /// held open for the whole read.
    folder: std::fs::File,
    /// The folder's path,
    /// which a refusal names.
    path: PathBuf,
    /// Whether every record read must answer the root-control terms.
    under_root_control: bool,
}

impl SlotIdentityStore {
    /// What:
    ///  Open the store folder at `path` and hold it,
    ///  refusing a link at that name and any other non-folder.
    /// Why:
    ///   The grammar's own reader,
    ///  which a control drives on a folder it owns;
    ///  the route reads through `open_under_root_control`,
    ///  which makes the same open and then checks whose folder it is.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const store = openStore(path);
    /// ```
    ///
    /// # Errors
    /// `Missing` when nothing is at the path,
    /// and `Unreadable` when the name holds a link or another non-folder.
    pub fn open(path: &Path) -> Result<SlotIdentityStore, StoreRefusal> {
        #[cfg(test)]
        BEFORE_THE_STORE_OPENS.with(|hook| { if let Some(before) = hook.borrow().as_ref() { before(); } });
        let folder: std::fs::File = open_folder(path).map_err(|error| return match error.kind() {
            std::io::ErrorKind::NotFound => StoreRefusal::Missing { path: path.to_path_buf() },
            _ => StoreRefusal::Unreadable { path: path.to_path_buf(), reason: error.to_string() },
        })?;
        debug(TAG, "opened the recorded slot identities folder");
        return Ok(SlotIdentityStore { folder, path: path.to_path_buf(), under_root_control: false });
    }

    /// What:
    ///  Open the store folder the route acts on,
    ///  refusing a folder this account could rewrite.
    /// Why:
    ///   The route runs as root,
    ///  so a store folder the attach's account could change is not the record the design rests on;
    ///  this is the reader every attach and every adoption uses,
    ///  and it also makes each record read answer the same two terms,
    ///  so a record another account could write decides nothing either.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const store = openStoreUnderRootControl('/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities');
    /// ```
    ///
    /// # Errors
    /// Everything `open` refuses,
    /// and a folder whose owner is not root or whose group or other write bit is set.
    pub fn open_under_root_control(path: &Path) -> Result<SlotIdentityStore, StoreRefusal> {
        let store: SlotIdentityStore = SlotIdentityStore::open(path)?;
        check_under_root_control(path, "the store folder", &store.folder)?;
        return Ok(SlotIdentityStore { folder: store.folder, path: store.path, under_root_control: true });
    }

    /// The store folder's path,
    /// which a refusal names.
    pub fn path(&self) -> &Path {
        return self.path.as_path();
    }

    /// The held folder,
    /// which the writers name every record relative to.
    pub(super) fn folder(&self) -> &std::fs::File {
        return &self.folder;
    }

    /// What:
    ///  The recorded identity of the `.claude/worktrees` parent folder,
    ///  or the refusal of a store that holds none.
    /// Why:
    ///   The attach compares the walked parent against this record,
    ///  and a store with no parent record leaves nothing to compare it against.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const parent = store.parent(); // throws NoParentRecord
    /// ```
    ///
    /// # Errors
    /// Everything the record's read refuses,
    /// and `NoParentRecord` when nothing is filed under `.parent`.
    pub fn parent(&self) -> Result<super::clone_folder_identity::CloneFolderIdentity, StoreRefusal> {
        return match self.read(PARENT_RECORD_NAME)? {
            Some(StoredIdentity::Parent(identity)) => Ok(identity),
            Some(other) => Err(StoreRefusal::RecordUnderAnotherName { path: self.path.join(PARENT_RECORD_NAME), filed_under: String::from(PARENT_RECORD_NAME), recorded: other.key().to_string() }),
            None => Err(StoreRefusal::NoParentRecord { path: self.path.clone() }),
        };
    }

    /// What:
    ///  The recorded identity of the slot named `name`,
    ///  or the refusal of a store that holds none.
    /// Why:
    ///   The attach compares the walked slot against the one record filed under its name,
    ///  and a slot with no record was never verified.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const recorded = store.slot('my-worktree'); // throws NoSlotRecord
    /// ```
    ///
    /// # Errors
    /// Everything the record's read refuses,
    /// a name that is no plain slot name,
    /// and `NoSlotRecord` when nothing is filed under it.
    pub fn slot(&self, name: &str) -> Result<super::clone_folder_identity::CloneFolderIdentity, StoreRefusal> {
        if let Some(reason) = slot_name_refusal(name) {
            return Err(StoreRefusal::SlotName { slot: name.to_string(), reason });
        }
        return match self.read(name)? {
            Some(StoredIdentity::Slot { identity, .. }) => Ok(identity),
            Some(other) => Err(StoreRefusal::RecordUnderAnotherName { path: self.path.join(name), filed_under: name.to_string(), recorded: other.key().to_string() }),
            None => Err(StoreRefusal::NoSlotRecord { path: self.path.clone(), slot: name.to_string() }),
        };
    }

    /// The record filed under `name`,
    /// or nothing when the folder holds no such file.
    fn read(&self, name: &str) -> Result<Option<StoredIdentity>, StoreRefusal> {
        return read_record(&self.folder, self.path.as_path(), name, self.under_root_control);
    }

    /// What:
    ///  Every slot name the folder lists,
    ///  sorted,
    ///  with `.parent` and every staging entry left out.
    /// Why:
    ///  The adoption needs the whole set and refuses a foreign name,
    ///  because a slot whose record a later read would miss must not pass for a record the adoption verified.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const names = store.slotNames();
    /// ```
    ///
    /// # Errors
    /// Everything the listing refuses.
    pub fn slot_names(&self) -> Result<Vec<String>, StoreRefusal> {
        return listed_slot_names(&self.folder, self.path.as_path());
    }

    /// What:
    ///  Every slot name the folder lists,
    ///  with every known staging name a killed writer left removed on the way,
    ///  from one listing of the folder.
    /// Why:
    ///  The adoption needs the whole set and the removal of the same instant:
    ///  its first listing keeps every name that is a slot record and refuses a foreign name,
    ///  and the same pass removes the known staging names before the adoption stages its own record,
    ///  so a killed writer's litter costs the folder one visit and not two.
    ///  The names the removal took are logged where they go and answered no further,
    ///  because the adoption decides on the set alone.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const names = store.listAndRemoveStagingEntries();
    /// ```
    ///
    /// # Errors
    /// Everything the listing or a removal refuses.
    pub(crate) fn list_and_remove_staging_entries(&self) -> Result<Vec<String>, StoreRefusal> {
        return super::slot_identity_store_reading::listed_slot_names_removing_staging_entries(&self.folder, self.path.as_path());
    }

    /// What:
    ///  Remove the staging names a killed writer left,
    ///  and answer their names.
    /// Why:
    ///  A kill between a staging file's bytes and its link leaves an entry no reader reads and no writer owns;
    ///  the adoption removes them as it lists the folder,
    ///  and plan step 18's `doctor` is to report and remove them
    ///  (`doc/decision/cli-git-zfs-slot-identity-store-format.md`,
    ///  "Who removes a killed writer's staging file").
    ///  No sweep run removes them,
    ///  because that would cost one listing of the folder per run,
    ///  which the accepted design's choice 9 does not take.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const removed = store.removeStagingEntries();
    /// ```
    ///
    /// # Errors
    /// Everything the listing or a removal refuses.
    pub fn remove_staging_entries(&self) -> Result<Vec<String>, StoreRefusal> {
        return super::slot_identity_store_reading::remove_known_staging_entries(&self.folder, self.path.as_path());
    }
}

#[cfg(test)]
#[path = "slot_identity_store_tests.rs"]
mod tests;
