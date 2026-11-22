//! What:
//!  Publishing a record in the store folder:
//!  a slot's first record,
//!  the parent folder's first record,
//!  and the replacement the administrator's adoption publishes.
//! Why:
//!  A record must never be visible under its own name holding a prefix of its bytes,
//!  so a publication fills a staging file and gives it its record's name in one call that replaces nothing.
//!  `linkat` answers `EEXIST` to the second recorder of one name,
//!  which is the whole answer to two sweeps meeting on one slot:
//!  the loser removes its own staging name,
//!  syncs the folder and reads the record that won,
//!  and it answers `SlotAlreadyRecorded` carrying that record so the sweep carries on with it.
//!  A folder that refuses a hard link publishes with `renameat2` and `RENAME_NOREPLACE`,
//!  which answers `EEXIST` just the same,
//!  and a folder that refuses both calls refuses the write instead of publishing by a rename that replaces.
//!  A publication compares `.parent` against the walked parent before it creates anything,
//!  so no record is taken for a folder reached through a parent no check compared,
//!  and the setup run's second publication compares the record it finds before it makes any name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! publishSlotIdentity(store, 'add-cache-layer', identity, walkedParent); // SlotAlreadyRecorded carries the winner
//! publishParentIdentity(store, identity, walkedParent); replaceParentIdentity(store, adopted);
//! ```

use super::clone_folder_identity::{CloneFolderField, CloneFolderIdentity};
use crate::publishing_entries_in_one_open_folder::{
    create_new_entry, fill_entry_and_sync, link_entry, link_refused_because_the_folder_makes_none, remove_entry, rename_entry_never_replacing, rename_entry_replacing, sync_folder,
};
use super::slot_identity_store::SlotIdentityStore;
use super::slot_identity_store_control::RECORD_MODE;
use super::slot_identity_store_reading::RECORD_MAXIMUM_BYTES;
use super::slot_identity_store_records::{PARENT_RECORD_NAME, StoredIdentity};
use super::slot_name::slot_name_refusal;
use super::slot_identity_store_refusals::{StoreRefusal, write_refusal};
use crate::diagnostic_log::{debug, warn};
use crate::hexadecimal_of_bytes::hexadecimal_of_bytes;
use crate::rename_that_never_replaces::rename_unsupported;
use std::ffi::{OsStr, OsString};
use std::path::Path;

/// Module tag for diagnostics.
const TAG: &str = "slotIdentityStore";

/// How many bytes one staging name's random part is made from:
/// sixteen,
/// which is the thirty-two lowercase hexadecimal digits the name holds.
const STAGING_RANDOM_BYTES: usize = 16;

/// What:
///  Publish one slot's first record,
///  for a caller that drives the store folder by path.
/// Why:
///  The grammar's own writer,
///  which a control drives on a fixture folder;
///  the route writes through `publish_slot_identity_under_root_control`,
///  which opens the folder under root control first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// publishSlotIdentity(store, 'my-worktree', identity, walkedParent);
/// ```
///
/// # Errors
/// The store folder's open refusal,
/// a name that is no plain slot name,
/// a store with no parent record,
/// a walked parent that differs from that record,
/// a record wider than one read accepts,
/// the write's own failure,
/// or `SlotAlreadyRecorded` carrying the record that won.
pub fn publish_slot_identity(store: &Path, name: &str, identity: &CloneFolderIdentity, walked_parent: &CloneFolderIdentity) -> Result<(), StoreRefusal> {
    let held: SlotIdentityStore = SlotIdentityStore::open(store)?;
    return publish_into(&held, name, identity, walked_parent);
}

/// What:
///  Publish one slot's first record,
///  refusing a store folder or a record this account could rewrite.
/// Why:
///   A record decides which folder a clone may be attached to,
///  so the route publishes only into a folder that is root's and reads only records that are root's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// publishSlotIdentityUnderRootControl(store, 'my-worktree', identity, walkedParent);
/// ```
///
/// # Errors
/// Everything `publish_slot_identity` refuses,
/// and a folder or a record whose owner is not root or which another account may write.
pub fn publish_slot_identity_under_root_control(store: &Path, name: &str, identity: &CloneFolderIdentity, walked_parent: &CloneFolderIdentity) -> Result<(), StoreRefusal> {
    let held: SlotIdentityStore = SlotIdentityStore::open_under_root_control(store)?;
    return publish_into(&held, name, identity, walked_parent);
}

/// What:
///  The publication itself:
///  the name's grammar,
///  the parent's compare,
///  the bytes,
///  the staging file,
///  the link that names it,
///  and the loser's answer.
/// Why:
///  The parent's compare comes first because a slot record is what later lets an attach proceed under that parent,
///  and a record taken before the parent was compared would be a first observation under a parent nobody had compared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// publishInto(store, name, identity, walkedParent);
/// ```
fn publish_into(store: &SlotIdentityStore, name: &str, identity: &CloneFolderIdentity, walked_parent: &CloneFolderIdentity) -> Result<(), StoreRefusal> {
    if let Some(reason) = slot_name_refusal(name) {
        return Err(StoreRefusal::SlotName { slot: name.to_string(), reason });
    }
    let recorded_parent: CloneFolderIdentity = store.parent()?;
    let parent_fields: Vec<CloneFolderField> = recorded_parent.differences(walked_parent);
    if !parent_fields.is_empty() {
        return Err(StoreRefusal::ParentIdentity {
            path: store.path().to_path_buf(),
            recorded: Box::new(recorded_parent),
            found: Box::new(walked_parent.clone()),
            fields: parent_fields,
        });
    }
    let bytes: String = record_bytes(&StoredIdentity::Slot { name: name.to_string(), identity: identity.clone() }, store.path())?;
    let staging: OsString = staging_name();
    let created: std::fs::File = create_staging_entry(store, staging.as_os_str(), bytes.as_bytes(), name)?;
    let published: Result<(), StoreRefusal> = link_or_rename(store, staging.as_os_str(), OsStr::new(name), created, name);
    if published.is_ok() {
        debug(TAG, "published the identity of one clone slot");
    }
    return published;
}

/// What:
///  Publish the parent folder's first record,
///  and treat a second setup run's equal folder as success.
/// Why:
///  The setup run's boot unit runs on every boot,
///  so a second run is the normal case and must not report an error;
///  only a live folder that differs from the record refuses,
///  because the setup run never replaces an identity an administrator's adoption decided.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// publishParentIdentity(store, identity, walkedParent);
/// ```
///
/// # Errors
/// The store folder's open refusal,
/// a record wider than one read accepts,
/// the write's own failure,
/// or `ParentAlreadyRecorded` when the record and the live folder differ.
pub fn publish_parent_identity(store: &Path, identity: &CloneFolderIdentity, walked: &CloneFolderIdentity) -> Result<(), StoreRefusal> {
    let held: SlotIdentityStore = SlotIdentityStore::open(store)?;
    return publish_parent_into(&held, identity, walked);
}

/// What:
///  Publish the parent folder's first record,
///  refusing a store folder or a record this account could rewrite.
/// Why:
///  The setup run writes the record every attach compares against,
///  so it writes only into a folder that is root's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// publishParentIdentityUnderRootControl(store, identity, walkedParent);
/// ```
///
/// # Errors
/// Everything `publish_parent_identity` refuses,
/// and a folder or a record whose owner is not root or which another account may write.
pub fn publish_parent_identity_under_root_control(store: &Path, identity: &CloneFolderIdentity, walked: &CloneFolderIdentity) -> Result<(), StoreRefusal> {
    let held: SlotIdentityStore = SlotIdentityStore::open_under_root_control(store)?;
    return publish_parent_into(&held, identity, walked);
}

/// The parent's first publication,
/// which compares the live folder with the record the folder holds before it makes any name,
/// and publishes only where `.parent` is absent.
/// Why:
///  The setup run's boot unit runs on every boot,
///  so a second run is the normal case;
///  the design says the compare runs before the name is made
///  (`doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md`,
///  "Two writers with two intents"),
///  so the normal case pays no durable write,
///  no failed link,
///  no unlink and no folder `fsync`,
///  and a kill in that window leaves no staging name.
///  The `EEXIST` arm is kept for two first runs that both found `.parent` absent,
///  where the loser compares what the winner left.
fn publish_parent_into(store: &SlotIdentityStore, identity: &CloneFolderIdentity, walked: &CloneFolderIdentity) -> Result<(), StoreRefusal> {
    match store.parent() {
        Ok(recorded) => {
            let compared: Result<(), StoreRefusal> = compare_the_parent_identity_with(store, walked, &recorded);
            if compared.is_ok() {
                debug(TAG, "a second setup run found the identity the parent record holds");
            }
            return compared;
        }
        Err(StoreRefusal::NoParentRecord { .. }) => {}
        Err(other) => return Err(other),
    }
    let bytes: String = record_bytes(&StoredIdentity::Parent(identity.clone()), store.path())?;
    let staging: OsString = staging_name();
    let created: std::fs::File = create_staging_entry(store, staging.as_os_str(), bytes.as_bytes(), PARENT_RECORD_NAME)?;
    let published: Result<(), StoreRefusal> = link_or_rename(store, staging.as_os_str(), OsStr::new(PARENT_RECORD_NAME), created, PARENT_RECORD_NAME);
    return match published {
        Ok(()) => {
            debug(TAG, "published the identity of the worktrees parent folder");
            Ok(())
        }
        // Another run published `.parent` between this run's compare and its link, and its own answer is the record's.
        Err(StoreRefusal::SlotAlreadyRecorded { .. }) => compare_the_parent_record_with(store, walked),
        Err(other) => Err(other),
    };
}

/// Whether the record `.parent` now holds is the live folder,
/// which is a second setup run's success and any difference's refusal.
fn compare_the_parent_record_with(store: &SlotIdentityStore, walked: &CloneFolderIdentity) -> Result<(), StoreRefusal> {
    let recorded: CloneFolderIdentity = store.parent()?;
    return compare_the_parent_identity_with(store, walked, &recorded);
}

/// The compare itself,
/// over the record the folder holds.
fn compare_the_parent_identity_with(store: &SlotIdentityStore, walked: &CloneFolderIdentity, recorded: &CloneFolderIdentity) -> Result<(), StoreRefusal> {
    let fields: Vec<CloneFolderField> = recorded.differences(walked);
    if fields.is_empty() {
        return Ok(());
    }
    return Err(StoreRefusal::ParentAlreadyRecorded { path: store.path().to_path_buf(), recorded: Box::new(recorded.clone()), live: Box::new(walked.clone()), fields });
}

/// What:
///  Replace the parent record and keep every slot record as it is,
///  publishing the new file by one rename.
/// Why:
///  The administrator's adoption is the only write that may replace the parent's identity,
///  and it must never overwrite a surviving slot identity
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Descriptor mount and unmount");
///  the layout meets that with its shape:
///  `.parent` holds nothing but the parent's identity,
///  and this writer opens no slot file,
///  renames none and removes none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// replaceParentIdentity(store, adoptedIdentity);
/// ```
///
/// # Errors
/// The store folder's open refusal,
/// a store folder with no parent record,
/// a record wider than one read accepts,
/// or the write's own failure,
/// which leaves the record as it was.
pub fn replace_parent_identity(store: &Path, identity: &CloneFolderIdentity) -> Result<(), StoreRefusal> {
    let held: SlotIdentityStore = SlotIdentityStore::open(store)?;
    return replace_parent_into(&held, identity);
}

/// What:
///  Replace the parent record,
///  refusing a store folder or a record this account could rewrite.
/// Why:
///  The adoption is the one write that may replace the parent identity,
///  so it writes only into a folder that is root's,
///  and it reads the record it replaces through the same terms.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// replaceParentIdentityUnderRootControl(store, adoptedIdentity);
/// ```
///
/// # Errors
/// Everything `replace_parent_identity` refuses,
/// and a folder whose owner is not root or which another account may write.
pub fn replace_parent_identity_under_root_control(store: &Path, identity: &CloneFolderIdentity) -> Result<(), StoreRefusal> {
    let held: SlotIdentityStore = SlotIdentityStore::open_under_root_control(store)?;
    return replace_parent_into(&held, identity);
}

/// The replacing publication:
/// a staging file,
/// its bytes,
/// and one rename over `.parent`.
fn replace_parent_into(store: &SlotIdentityStore, identity: &CloneFolderIdentity) -> Result<(), StoreRefusal> {
    // The record must be there: the adoption refuses a store that never recorded a parent.
    let _recorded: CloneFolderIdentity = store.parent()?;
    let bytes: String = record_bytes(&StoredIdentity::Parent(identity.clone()), store.path())?;
    let staging: OsString = parent_staging_name();
    let _created: std::fs::File = create_staging_entry(store, staging.as_os_str(), bytes.as_bytes(), PARENT_RECORD_NAME)?;
    #[cfg(test)]
    BETWEEN_THE_PARENT_BYTES_AND_THE_RENAME.with(|hook| { if let Some(between) = hook.borrow().as_ref() { between(); } });
    #[cfg(test)]
    let forced_rename: Option<i32> = THE_REPLACING_RENAME_IS_ANSWERED_WITH.with(|answer| return answer.get());
    #[cfg(not(test))]
    let forced_rename: Option<i32> = None;
    let renamed: std::io::Result<()> = match a_forced_answer(forced_rename) {
        Some(answer) => Err(answer),
        None => rename_entry_replacing(store.folder(), staging.as_os_str(), OsStr::new(PARENT_RECORD_NAME)),
    };
    if let Err(error) = renamed {
        remove_own_staging_entry(store, staging.as_os_str());
        return Err(write_refusal(store.path(), error));
    }
    sync_folder(store.folder()).map_err(|error| return write_refusal(store.path(), error))?;
    debug(TAG, "replaced the recorded identity of the worktrees parent folder");
    return Ok(());
}

/// The bytes of one record file:
/// the header line and the record line,
/// refused before anything is created when they outgrow one read.
fn record_bytes(record: &StoredIdentity, store: &Path) -> Result<String, StoreRefusal> {
    let text: String = format!("{}\n{}\n", super::slot_identity_store_records::STORE_HEADER, record.encode());
    let bytes: u64 = text.len() as u64;
    if bytes > RECORD_MAXIMUM_BYTES {
        // The record's key is `-` for the parent folder, which is no file's name; the refusal names the file the
        // layout files that record under instead.
        let file_name: &str = match record {
            StoredIdentity::Parent(_identity) => PARENT_RECORD_NAME,
            StoredIdentity::Slot { name, .. } => name.as_str(),
        };
        return Err(StoreRefusal::RecordTooLarge { path: store.join(file_name), bytes, bound: RECORD_MAXIMUM_BYTES });
    }
    return Ok(text);
}

/// A slot publication's staging name:
/// a dot and thirty-two lowercase hexadecimal digits.
fn staging_name() -> OsString {
    let mut bytes: [u8; STAGING_RANDOM_BYTES] = [0; STAGING_RANDOM_BYTES];
    if getrandom::fill(&mut bytes).is_err() {
        warn(TAG, "the operating system's random source failed, so the staging name is made from this process's clock");
        let now: u128 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|since| return since.as_nanos()).unwrap_or_default();
        bytes.copy_from_slice(&now.to_le_bytes()[..STAGING_RANDOM_BYTES]);
    }
    return OsString::from(format!(".{}", hexadecimal_of_bytes(&bytes)));
}

/// The parent replacement's staging name,
/// which is the form `private_storage::staging_beside` makes for `.parent`.
fn parent_staging_name() -> OsString {
    let mut staging: OsString = OsString::from("..parent.");
    staging.push(staging_name().as_os_str());
    staging.push(".partial");
    return staging;
}

/// Create one staging file and fill it,
/// removing it again when the fill failed.
fn create_staging_entry(store: &SlotIdentityStore, staging: &OsStr, bytes: &[u8], for_record: &str) -> Result<std::fs::File, StoreRefusal> {
    let mut file: std::fs::File = create_new_entry(store.folder(), staging, RECORD_MODE).map_err(|error| return write_refusal(store.path(), error))?;
    if let Err(error) = fill_entry_and_sync(&mut file, bytes, RECORD_MODE) {
        remove_own_staging_entry(store, staging);
        return Err(write_refusal(store.path().join(for_record).as_path(), error));
    }
    return Ok(file);
}

/// What:
///  Publish the staging file under its record name,
///  by one link,
///  or by the rename that never replaces where the folder refuses a link.
/// Why:
///   The link is the accepted publication;
///  the rename is the fallback a folder without hard links needs,
///  and it answers `EEXIST` the same way,
///  so the one-winner property holds on both.
///  A folder that refuses both refuses the write,
///  and nothing falls back to a rename that replaces,
///  because a publication that may replace another's record is the refresh this layout forbids.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// linkOrRename(store, staging, name); // EEXIST becomes the loser's SlotAlreadyRecorded
/// ```
fn link_or_rename(store: &SlotIdentityStore, staging: &OsStr, name: &OsStr, staged: std::fs::File, for_record: &str) -> Result<(), StoreRefusal> {
    #[cfg(test)]
    BETWEEN_THE_BYTES_AND_THE_LINK.with(|hook| {
        // Taken out, so a second publication the hook runs does not run it again.
        let held = hook.borrow_mut().take();
        if let Some(between) = held { between(); }
    });
    drop(staged);
    #[cfg(test)]
    let forced_link: Option<i32> = THE_LINK_IS_ANSWERED_WITH.with(|answer| return answer.get());
    #[cfg(not(test))]
    let forced_link: Option<i32> = None;
    let linked: std::io::Result<()> = match a_forced_answer(forced_link) {
        Some(answer) => Err(answer),
        None => link_entry(store.folder(), staging, name),
    };
    return match linked {
        Ok(()) => finish_the_publish(store, staging),
        Err(error) if error.raw_os_error() == Some(libc::EEXIST) => lose_the_race(store, staging, name),
        Err(error) if link_refused_because_the_folder_makes_none(&error) => publish_by_the_rename(store, staging, name, for_record),
        Err(error) => {
            remove_own_staging_entry(store, staging);
            Err(StoreRefusal::Write {
                path: store.path().to_path_buf(),
                reason: format!("the folder refused the link that gives the record its name ({error}), and nothing was published"),
            })
        }
    };
}

/// What:
///  The refusal a control forces in place of one call's own answer,
///  or nothing in the release executable.
/// Why:
///  A folder on a test host makes hard links and the rename that never replaces,
///  so the two refusals a publication falls back from must be injectable;
///  the call is skipped rather than overridden afterwards,
///  because a link that ran would already have published the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const forced = forcedAnswer(); // undefined, or the error the control chose
/// ```
fn a_forced_answer(answer: Option<i32>) -> Option<std::io::Error> {
    #[cfg(not(test))]
    let _ = answer;
    return answer.map(|code| return std::io::Error::from_raw_os_error(code));
}

/// Remove the staging name and flush the folder,
/// so the record's own name is durable and the staging name is gone.
fn finish_the_publish(store: &SlotIdentityStore, staging: &OsStr) -> Result<(), StoreRefusal> {
    remove_own_staging_entry(store, staging);
    return sync_folder(store.folder()).map_err(|error| return write_refusal(store.path(), error));
}

/// The fallback publication of a folder without hard links.
fn publish_by_the_rename(store: &SlotIdentityStore, staging: &OsStr, name: &OsStr, for_record: &str) -> Result<(), StoreRefusal> {
    #[cfg(test)]
    let forced_rename: Option<i32> = THE_RENAME_IS_ANSWERED_WITH.with(|answer| return answer.get());
    #[cfg(not(test))]
    let forced_rename: Option<i32> = None;
    let renamed: std::io::Result<()> = match a_forced_answer(forced_rename) {
        Some(answer) => Err(answer),
        None => rename_entry_never_replacing(store.folder(), staging, name),
    };
    return match renamed {
        Ok(()) => sync_folder(store.folder()).map_err(|error| return write_refusal(store.path(), error)),
        Err(error) if error.raw_os_error() == Some(libc::EEXIST) => lose_the_race(store, staging, name),
        Err(error) => {
            remove_own_staging_entry(store, staging);
            let unsupported: bool = rename_unsupported(&error);
            Err(StoreRefusal::Write {
                path: store.path().join(for_record),
                reason: if unsupported {
                    format!("the folder refused both the link that gives the record its name and the rename that never replaces ({error}), so nothing was published; the folder makes no hard links and no no-replace rename, and nothing is published by a rename that could replace another run's record")
                } else {
                    format!("the folder refused the rename that never replaces ({error}), and nothing was published")
                },
            })
        }
    };
}

/// What:
///  The loser of two recorders meeting on one name:
///  it removes its own staging name,
///  flushes the folder,
///  reads the record that won,
///  and answers it.
/// Why:
///   The kernel answers exactly one writer,
///  and the winner's record is what every attach compares against,
///  so the loser carries on with it instead of failing the sweep;
///  the flush is what makes the winner's name durable before the loser reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// loseTheRace(store, staging, name); // throws SlotAlreadyRecorded carrying the winner
/// ```
fn lose_the_race(store: &SlotIdentityStore, staging: &OsStr, name: &OsStr) -> Result<(), StoreRefusal> {
    remove_own_staging_entry(store, staging);
    sync_folder(store.folder()).map_err(|error| return write_refusal(store.path(), error))?;
    let slot: String = name.to_string_lossy().into_owned();
    // `.parent` is the one other name this step can lose on, and its record holds the parent's identity.
    let winner: CloneFolderIdentity = if slot == PARENT_RECORD_NAME { store.parent()? } else { store.slot(slot.as_str())? };
    return Err(StoreRefusal::SlotAlreadyRecorded { path: store.path().join(&slot), slot, identity: winner });
}

/// Remove one staging name this call made,
/// and say so when the folder refused.
fn remove_own_staging_entry(store: &SlotIdentityStore, staging: &OsStr) {
    match remove_entry(store.folder(), staging) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => warn(TAG, format!("the staging name {} was left behind: {error}", staging.to_string_lossy()).as_str()),
    }
}

#[cfg(test)]
thread_local! {
    /// A control may run a second publication while this one holds every byte of its staging file and no record name yet,
    /// which is where two recorders of one slot meet.
    pub(crate) static BETWEEN_THE_BYTES_AND_THE_LINK: std::cell::RefCell<Option<Box<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
    /// A control may read the old parent record while the replacing writer holds its new bytes and has not renamed yet,
    /// which is where a reader racing an adoption stands.
    pub(crate) static BETWEEN_THE_PARENT_BYTES_AND_THE_RENAME: std::cell::RefCell<Option<Box<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
    /// A control may answer the link with a refusal no folder on a test host gives,
    /// which forces the fallback publication and the double refusal of its arms.
    pub(crate) static THE_LINK_IS_ANSWERED_WITH: std::cell::Cell<Option<i32>> = const { std::cell::Cell::new(None) };
    /// A control may answer the rename that never replaces with such a refusal too.
    pub(crate) static THE_RENAME_IS_ANSWERED_WITH: std::cell::Cell<Option<i32>> = const { std::cell::Cell::new(None) };
    /// A control may answer the adoption's replacing rename with a refusal,
    /// which is the arm that leaves the old record and removes the staging name.
    pub(crate) static THE_REPLACING_RENAME_IS_ANSWERED_WITH: std::cell::Cell<Option<i32>> = const { std::cell::Cell::new(None) };
}
