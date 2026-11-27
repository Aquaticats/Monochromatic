//! What:
//!  Publishing a record in the store folder:
//!  a slot's first record,
//!  the parent folder's first record,
//!  and the replacement the administrator's adoption publishes.
//! Why:
//!  A publication compares `.parent` against the walked parent before it
//!  creates anything,
//!  so no record is taken for a folder reached through a parent no check
//!  compared,
//!  and the setup run's second publication compares the record it finds before
//!  it makes any name.
//!  Each of them fills a staging file and gives it the record's name in one
//!  call that replaces nothing,
//!  which `slot_identity_store_publication_staging_file.rs` and
//!  `slot_identity_store_publication_one_winner.rs` hold
//!  (`doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md`,
//!  "Two writers with two intents").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! publishSlotIdentity(store, 'add-cache-layer', identity, walkedParent); // SlotAlreadyRecorded carries the winner
//! publishParentIdentity(store, identity, walkedParent); replaceParentIdentity(store, adopted);
//! ```

use super::clone_folder_identity::{
    CloneFolderField,
    CloneFolderIdentity,
};
use super::slot_identity_store::SlotIdentityStore;
use super::slot_identity_store_publication_one_winner::{
    a_forced_answer,
    link_or_rename,
};
use super::slot_identity_store_publication_staging_file::{
    create_staging_entry,
    parent_staging_name,
    record_bytes,
    remove_own_staging_entry,
    staging_name,
};
use super::slot_identity_store_records::{
    PARENT_RECORD_NAME,
    StoredIdentity,
};
use super::slot_identity_store_refusals::{
    StoreRefusal,
    write_refusal,
};
use super::slot_name::slot_name_refusal;
use crate::diagnostic_log::debug;
use crate::publishing_entries_in_one_open_folder::{
    rename_entry_replacing,
    sync_folder,
};
use std::ffi::{
    OsStr,
    OsString,
};
use std::path::Path;

/// Import the tag this module's diagnostics carry.
use crate::worktree_clone_mount::slot_identity_store_reading::TAG;

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
pub fn publish_slot_identity(
    store: &Path,
    name: &str,
    identity: &CloneFolderIdentity,
    walked_parent: &CloneFolderIdentity,
) -> Result<(), StoreRefusal> {
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
pub fn publish_slot_identity_under_root_control(
    store: &Path,
    name: &str,
    identity: &CloneFolderIdentity,
    walked_parent: &CloneFolderIdentity,
) -> Result<(), StoreRefusal> {
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
fn publish_into(
    store: &SlotIdentityStore,
    name: &str,
    identity: &CloneFolderIdentity,
    walked_parent: &CloneFolderIdentity,
) -> Result<(), StoreRefusal> {
    if let Some(reason) = slot_name_refusal(name) {
        return Err(StoreRefusal::SlotName {
            slot: name.to_string(),
            reason,
        });
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
    let bytes: String = record_bytes(
        &StoredIdentity::Slot {
            name: name.to_string(),
            identity: identity.clone(),
        },
        store.path(),
    )?;
    let staging: OsString = staging_name();
    let created: std::fs::File =
        create_staging_entry(store, staging.as_os_str(), bytes.as_bytes(), name)?;
    let published: Result<(), StoreRefusal> =
        link_or_rename(store, staging.as_os_str(), OsStr::new(name), created, name);
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
pub fn publish_parent_identity(
    store: &Path,
    identity: &CloneFolderIdentity,
    walked: &CloneFolderIdentity,
) -> Result<(), StoreRefusal> {
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
pub fn publish_parent_identity_under_root_control(
    store: &Path,
    identity: &CloneFolderIdentity,
    walked: &CloneFolderIdentity,
) -> Result<(), StoreRefusal> {
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
fn publish_parent_into(
    store: &SlotIdentityStore,
    identity: &CloneFolderIdentity,
    walked: &CloneFolderIdentity,
) -> Result<(), StoreRefusal> {
    match store.parent() {
        Ok(recorded) => {
            let compared: Result<(), StoreRefusal> =
                compare_the_parent_identity_with(store, walked, &recorded);
            if compared.is_ok() {
                debug(
                    TAG,
                    "a second setup run found the identity the parent record holds",
                );
            }
            return compared;
        }
        Err(StoreRefusal::NoParentRecord {
            ..
        }) => {}
        Err(other) => return Err(other),
    }
    let bytes: String = record_bytes(&StoredIdentity::Parent(identity.clone()), store.path())?;
    let staging: OsString = staging_name();
    let created: std::fs::File = create_staging_entry(
        store,
        staging.as_os_str(),
        bytes.as_bytes(),
        PARENT_RECORD_NAME,
    )?;
    let published: Result<(), StoreRefusal> = link_or_rename(
        store,
        staging.as_os_str(),
        OsStr::new(PARENT_RECORD_NAME),
        created,
        PARENT_RECORD_NAME,
    );
    return match published {
        Ok(()) => {
            debug(TAG, "published the identity of the worktrees parent folder");
            Ok(())
        }
        // Another run published `.parent` between this run's compare and its link, and its own answer is the record's.
        Err(StoreRefusal::SlotAlreadyRecorded {
            ..
        }) => compare_the_parent_record_with(store, walked),
        Err(other) => Err(other),
    };
}

/// Whether the record `.parent` now holds is the live folder,
/// which is a second setup run's success and any difference's refusal.
fn compare_the_parent_record_with(
    store: &SlotIdentityStore,
    walked: &CloneFolderIdentity,
) -> Result<(), StoreRefusal> {
    let recorded: CloneFolderIdentity = store.parent()?;
    return compare_the_parent_identity_with(store, walked, &recorded);
}

/// The compare itself,
/// over the record the folder holds.
fn compare_the_parent_identity_with(
    store: &SlotIdentityStore,
    walked: &CloneFolderIdentity,
    recorded: &CloneFolderIdentity,
) -> Result<(), StoreRefusal> {
    let fields: Vec<CloneFolderField> = recorded.differences(walked);
    if fields.is_empty() {
        return Ok(());
    }
    return Err(StoreRefusal::ParentAlreadyRecorded {
        path: store.path().to_path_buf(),
        recorded: Box::new(recorded.clone()),
        live: Box::new(walked.clone()),
        fields,
    });
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
pub fn replace_parent_identity(
    store: &Path,
    identity: &CloneFolderIdentity,
) -> Result<(), StoreRefusal> {
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
pub fn replace_parent_identity_under_root_control(
    store: &Path,
    identity: &CloneFolderIdentity,
) -> Result<(), StoreRefusal> {
    let held: SlotIdentityStore = SlotIdentityStore::open_under_root_control(store)?;
    return replace_parent_into(&held, identity);
}

/// The replacing publication:
/// a staging file,
/// its bytes,
/// and one rename over `.parent`.
fn replace_parent_into(
    store: &SlotIdentityStore,
    identity: &CloneFolderIdentity,
) -> Result<(), StoreRefusal> {
    // The record must be there: the adoption refuses a store that never recorded a parent.
    let _recorded: CloneFolderIdentity = store.parent()?;
    let bytes: String = record_bytes(&StoredIdentity::Parent(identity.clone()), store.path())?;
    let staging: OsString = parent_staging_name();
    let _created: std::fs::File = create_staging_entry(
        store,
        staging.as_os_str(),
        bytes.as_bytes(),
        PARENT_RECORD_NAME,
    )?;
    #[cfg(test)]
    BETWEEN_THE_PARENT_BYTES_AND_THE_RENAME.with(|hook| {
        if let Some(between) = hook.borrow().as_ref() {
            between();
        }
    });
    #[cfg(test)]
    let forced_rename: Option<i32> =
        THE_REPLACING_RENAME_IS_ANSWERED_WITH.with(|answer| return answer.get());
    #[cfg(not(test))]
    let forced_rename: Option<i32> = None;
    let renamed: std::io::Result<()> = match a_forced_answer(forced_rename) {
        Some(answer) => Err(answer),
        None => rename_entry_replacing(
            store.folder(),
            staging.as_os_str(),
            OsStr::new(PARENT_RECORD_NAME),
        ),
    };
    if let Err(error) = renamed {
        remove_own_staging_entry(store, staging.as_os_str());
        return Err(write_refusal(store.path(), error));
    }
    sync_folder(store.folder()).map_err(|error| return write_refusal(store.path(), error))?;
    debug(
        TAG,
        "replaced the recorded identity of the worktrees parent folder",
    );
    return Ok(());
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
