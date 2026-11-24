//! What:
//!  The facts the managed lock's proof needs,
//!  read from a slot's binding record whatever else that record is refused for.
//! Why:
//!  The sentence a lock holds is recomputed from the record's `nonce=`,
//!  and the check order takes that nonce from any well-formed record of the slot,
//!  whatever its phase
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  "How doctor and removal tell the states apart",
//!  step 1,
//!  and its limit `L12`).
//!  The scan of the record's lines is `binding_record_fields`,
//!  and this file decides which of them the lock's proof reads:
//!  the nonce,
//!  the slot and its identity,
//!  the registration and its identity,
//!  and the clone.
//!  A record it cannot read means no sentence can be recomposed at all,
//!  which is the state S3 and never a guess.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = readTheRecordedFacts(commonDir, slot); // { nonce, slot, slotIdentity?, registration?, registrationIdentity?, clone }
//! ```

use super::binding_record_fields::{
    CLONE_DATASET_KEY, CLONE_GUID_KEY, Fields, NONCE_KEY, REGISTRATION_IDENTITY_KEY, REGISTRATION_KEY, RecordedFactsRefusal, SLOT_IDENTITY_KEY, SLOT_KEY, fields_of, identity_field, path_field, required,
};
use super::reading_a_bounded_file::{ARefusedReading, Reading, read_a_bounded_file};
use crate::earlier_service_process_look::effective_user;
use crate::worktree_clone_binding_record::binding_bound::BINDING_MAXIMUM_BYTES;
use crate::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use crate::worktree_clone_binding_record::registration_home::{binding_name, home_folder};
use crate::worktree_clone_mount_protocol::{DatasetGuid, RequestNonce};
use std::path::{Path, PathBuf};

/// What:
///  The clone one record names:
///  its dataset and,
///  once the clone answered,
///  its GUID.
/// Why:
///  The release refuses unless the destruction proof names the clone the record names,
///  so the record's own two facts travel together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordedClone = { dataset: string; guid?: DatasetGuid };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedClone {
    /// The clone's dataset name,
    /// which the add chose before the clone existed.
    pub dataset: String,
    /// The clone's GUID,
    /// absent until the phase that proves it.
    pub guid: Option<DatasetGuid>,
}

/// What:
///  The record facts this route's readers take,
///  and nothing the add's own resume decides from.
/// Why:
///  A lock's proof reads the nonce,
///  the slot's path and identity,
///  the registration's path and identity,
///  and the clone,
///  and stops there:
///  the phase,
///  the claim and the request are the add's and the removal's to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordedLockFacts = { nonce: RequestNonce; slot: PathBuf; slotIdentity?: FolderIdentity; registration?: PathBuf; registrationIdentity?: FolderIdentity; clone: RecordedClone };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedLockFacts {
    /// The random number every lock sentence is recomputed from.
    pub nonce: RequestNonce,
    /// The slot's recorded whole path.
    pub slot: PathBuf,
    /// The underlay folder's identity,
    /// recorded while the slot was empty.
    pub slot_identity: Option<FolderIdentity>,
    /// The registration's recorded path,
    /// absent before the add wrote it.
    pub registration: Option<PathBuf>,
    /// The registration directory's recorded identity.
    pub registration_identity: Option<FolderIdentity>,
    /// The clone the record names.
    pub clone: RecordedClone,
}


/// What:
///  Read one slot's binding record for the facts the lock's proof needs,
///  checked for the account and the mode a record owes.
/// Why:
///  A record belongs to the account that wrote it,
///  and one another account could rewrite is not a fact about this add,
///  so both checks come before any field is read.
///
/// # Errors
/// The record's own refusal,
/// its grammar's,
/// or a field the lock's proof cannot do without.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = readTheRecordedFacts(commonDir, slot); // RecordedLockFacts or a refusal
/// ```
pub fn read_the_recorded_facts(common_dir: &Path, slot: &str) -> Result<RecordedLockFacts, RecordedFactsRefusal> {
    use std::os::unix::fs::MetadataExt as _;
    let home: PathBuf = home_folder(common_dir);
    let path: PathBuf = home.join(binding_name(slot));
    let bytes: Vec<u8> = match read_a_bounded_file(path.as_path(), BINDING_MAXIMUM_BYTES) {
        Reading::Absent => return Err(RecordedFactsRefusal::Absent),
        Reading::Refused(cause) => return Err(RecordedFactsRefusal::Refused(cause)),
        Reading::Bytes(found) => found,
    };
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path.as_path()).map_err(|error| return RecordedFactsRefusal::Refused(ARefusedReading::Unreadable { reason: error.to_string() }))?;
    let reader: u32 = effective_user();
    if metadata.uid() != reader {
        return Err(RecordedFactsRefusal::AnotherAccount { owner: metadata.uid(), reader });
    }
    let mode: u32 = metadata.mode() & 0o7777;
    if mode & 0o022 != 0 {
        return Err(RecordedFactsRefusal::WritableByOthers { mode });
    }
    let text: &str = std::str::from_utf8(bytes.as_slice()).map_err(|_utf8| {
        return RecordedFactsRefusal::NotThisGrammar { line: 1, found: String::from("bytes that are not UTF-8") };
    })?;
    let fields: Fields = fields_of(text)?;
    let nonce_text: &String = required(fields.nonce.as_ref(), NONCE_KEY)?;
    let nonce: RequestNonce = RequestNonce::parse(nonce_text.as_str()).map_err(|_refusal| {
        return RecordedFactsRefusal::Malformed { key: NONCE_KEY, found: nonce_text.clone() };
    })?;
    let slot_path: PathBuf = path_field(fields.slot.as_ref(), SLOT_KEY)?.ok_or(RecordedFactsRefusal::MissingField { key: SLOT_KEY })?;
    let clone_text: &String = required(fields.clone_dataset.as_ref(), CLONE_DATASET_KEY)?;
    let clone: RecordedClone = RecordedClone {
        dataset: clone_text.clone(),
        guid: match fields.clone_guid.as_ref() {
            None => None,
            Some(guid) => Some(DatasetGuid::parse(guid.as_str()).map_err(|_refusal| {
                return RecordedFactsRefusal::Malformed { key: CLONE_GUID_KEY, found: guid.clone() };
            })?),
        },
    };
    return Ok(RecordedLockFacts {
        nonce,
        slot: slot_path,
        slot_identity: identity_field(fields.slot_identity.as_ref(), SLOT_IDENTITY_KEY)?,
        registration: path_field(fields.registration.as_ref(), REGISTRATION_KEY)?,
        registration_identity: identity_field(fields.registration_identity.as_ref(), REGISTRATION_IDENTITY_KEY)?,
        clone,
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "recorded_lock_facts_tests.rs"]
mod tests;
