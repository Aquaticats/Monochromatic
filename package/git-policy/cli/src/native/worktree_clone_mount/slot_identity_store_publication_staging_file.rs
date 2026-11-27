//! What:
//!  The file a publication fills before its record has a name:
//!  the record's bytes and their bound,
//!  the two staging names,
//!  the staging file and how its bytes reach the disk,
//!  and the two ways that file is taken away again.
//! Why:
//!  A record must never be visible under its own name holding a prefix of its
//!  bytes,
//!  so every publication of `slot_identity_store_publication.rs` first fills a
//!  staging file no reader asks for,
//!  and the staging name of one publish can never be another's,
//!  so a failure removes only the file this call made.
//!  A record wider than one read accepts is refused here,
//!  before anything is created
//!  (`doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md`,
//!  "Two writers with two intents").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bytes = recordBytes(aRecord);                    // refused when wider than one read accepts
//! const staging = stagingName();                          // a dot and thirty-two hex digits
//! const file = createStagingEntry(store, staging, bytes);
//! removeOwnStagingEntry(store, staging);                   // on every way out
//! ```

use super::slot_identity_store::SlotIdentityStore;
use super::slot_identity_store_control::RECORD_MODE;
use super::slot_identity_store_reading::RECORD_MAXIMUM_BYTES;
use super::slot_identity_store_records::{
    PARENT_RECORD_NAME,
    StoredIdentity,
};
use super::slot_identity_store_refusals::{
    StoreRefusal,
    write_refusal,
};
use crate::diagnostic_log::warn;
use crate::hexadecimal_of_bytes::hexadecimal_of_bytes;
use crate::publishing_entries_in_one_open_folder::{
    create_new_entry,
    fill_entry_and_sync,
    remove_entry,
    sync_folder,
};
use std::ffi::{
    OsStr,
    OsString,
};
use std::path::Path;

/// Module tag for diagnostics.
const TAG: &str = "slotIdentityStore";

/// How many bytes one staging name's random part is made from:
/// sixteen,
/// which is the thirty-two lowercase hexadecimal digits the name holds.
const STAGING_RANDOM_BYTES: usize = 16;

/// The bytes of one record file:
/// the header line and the record line,
/// refused before anything is created when they outgrow one read.
pub(super) fn record_bytes(
    record: &StoredIdentity,
    store: &Path,
) -> Result<String, StoreRefusal> {
    let text: String = format!(
        "{}\n{}\n",
        super::slot_identity_store_records::STORE_HEADER,
        record.encode()
    );
    let bytes: u64 = text.len() as u64;
    if bytes > RECORD_MAXIMUM_BYTES {
        // The record's key is `-` for the parent folder, which is no file's name; the refusal names the file the
        // layout files that record under instead.
        let file_name: &str = match record {
            StoredIdentity::Parent(_identity) => PARENT_RECORD_NAME,
            StoredIdentity::Slot {
                name,
                ..
            } => name.as_str(),
        };
        return Err(StoreRefusal::RecordTooLarge {
            path: store.join(file_name),
            bytes,
            bound: RECORD_MAXIMUM_BYTES,
        });
    }
    return Ok(text);
}

/// A slot publication's staging name:
/// a dot and thirty-two lowercase hexadecimal digits.
pub(super) fn staging_name() -> OsString {
    let mut bytes: [u8; STAGING_RANDOM_BYTES] = [0; STAGING_RANDOM_BYTES];
    if getrandom::fill(&mut bytes).is_err() {
        warn(
            TAG,
            "the operating system's random source failed, so the staging name is made from this process's clock",
        );
        let now: u128 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|since| return since.as_nanos())
            .unwrap_or_default();
        bytes.copy_from_slice(&now.to_le_bytes()[..STAGING_RANDOM_BYTES]);
    }
    return OsString::from(format!(".{}", hexadecimal_of_bytes(&bytes)));
}

/// The parent replacement's staging name,
/// which is the form `private_storage::staging_beside` makes for `.parent`.
pub(super) fn parent_staging_name() -> OsString {
    let mut staging: OsString = OsString::from("..parent.");
    staging.push(staging_name().as_os_str());
    staging.push(".partial");
    return staging;
}

/// Create one staging file and fill it,
/// removing it again when the fill failed.
pub(super) fn create_staging_entry(
    store: &SlotIdentityStore,
    staging: &OsStr,
    bytes: &[u8],
    for_record: &str,
) -> Result<std::fs::File, StoreRefusal> {
    let mut file: std::fs::File = create_new_entry(store.folder(), staging, RECORD_MODE)
        .map_err(|error| return write_refusal(store.path(), error))?;
    if let Err(error) = fill_entry_and_sync(&mut file, bytes, RECORD_MODE) {
        remove_own_staging_entry(store, staging);
        return Err(write_refusal(
            store.path().join(for_record).as_path(),
            error,
        ));
    }
    return Ok(file);
}

/// Remove the staging name and flush the folder,
/// so the record's own name is durable and the staging name is gone.
pub(super) fn finish_the_publish(
    store: &SlotIdentityStore,
    staging: &OsStr,
) -> Result<(), StoreRefusal> {
    remove_own_staging_entry(store, staging);
    return sync_folder(store.folder()).map_err(|error| return write_refusal(store.path(), error));
}

/// Remove one staging name this call made,
/// and say so when the folder refused.
pub(super) fn remove_own_staging_entry(
    store: &SlotIdentityStore,
    staging: &OsStr,
) {
    match remove_entry(store.folder(), staging) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => warn(
            TAG,
            format!(
                "the staging name {} was left behind: {error}",
                staging.to_string_lossy()
            )
            .as_str(),
        ),
    }
}
