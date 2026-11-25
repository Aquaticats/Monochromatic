//! What:
//!  The binding record the settling command acts on:
//!  its reading,
//!  the identities a removal compares,
//!  and the two values the person's flags must confirm.
//! Why:
//!  The record is the witness the command removes last,
//!  so an unreadable one,
//!  a foreign one,
//!  one of another account's and one whose fields the grammar refuses
//!  each stop the run by name and change nothing
//!  (`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
//!  "A record that cannot be read").
//!  The slot's derived name and the GUID's last digits are what the two flags
//!  confirm,
//!  and both come from the record,
//!  so a pasted line confirms nothing
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The settling command").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const recorded = readTheRecord(commonDir, request);
//! confirmTheRecord(recorded, request);
//! ```

use crate::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_binding_record::record_publication::read_binding;
use crate::worktree_clone_binding_record::registration_home::{binding_name, home_folder};
use crate::worktree_clone_mount_protocol::RequestNonce;
use crate::worktree_clone_record_settling::settling_refusal::SettlingStop;
use crate::worktree_clone_record_settling::settling_request::SettlingRequest;
use crate::worktree_clone_record_settling::settling_words::the_last_four_digits;
use crate::worktree_clone_removal_proof_and_plan::removal_identities::{RecordedBinding, read_the_recorded_binding};
use std::path::{Path, PathBuf};

/// What:
///  The binding record the command acts on,
///  as the raw record and the identities a removal compares.
/// Why:
///  The lines print the phase and the underlay's recorded identity,
///  the lock's sentence is recomputed from the nonce,
///  and every removal step compares the typed identities,
///  so both readings of one file travel together and cannot disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheRecord = { recorded: RecordedBinding; phase: Phase; slotIdentity?: FolderIdentity; nonce: RequestNonce };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TheRecord {
    /// The identities a removal compares.
    pub recorded: RecordedBinding,
    /// How far the add got.
    pub phase: Phase,
    /// The underlay folder's identity,
    /// recorded while the slot was empty.
    pub slot_identity: Option<FolderIdentity>,
    /// The random number the lock's sentence is recomputed from.
    pub nonce: RequestNonce,
}

/// What:
///  The binding record and the identities a removal compares,
///  read for one slot,
///  or the stop the first of them names.
/// Why:
///  The command's second read is the record,
///  and its third is the pool,
///  so this is the last point at which a mistyped slot or a mistyped GUID can
///  refuse before anything is asked about the pool.
///
/// # Throws
/// No record,
/// a record that cannot be read,
/// one that names no clone,
/// or one that lacks a field the removals compare.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recorded = readTheRecord(commonDir, 'topic');
/// ```
pub fn read_the_record(common_dir: &Path, slot: &str) -> Result<TheRecord, SettlingStop> {
    let home: PathBuf = home_folder(common_dir);
    let path: PathBuf = home.join(binding_name(slot));
    let record: BindingRecord = read_binding(home.as_path(), slot)
        .map_err(|refusal| return SettlingStop::RecordUnreadable { path: path.clone(), said: refusal.message(path.as_path()) })?;
    if record.clone_guid.is_none() {
        return Err(SettlingStop::NoCloneRecorded { slot: String::from(slot), phase: String::from(record.phase.word()) });
    }
    let recorded: RecordedBinding = read_the_recorded_binding(common_dir, slot).map_err(|stop| return SettlingStop::TheRecordIsIncomplete { said: stop.message() })?;
    return Ok(TheRecord { recorded, phase: record.phase, slot_identity: record.slot_identity, nonce: record.nonce });
}

/// What:
///  Whether the two flags name the slot and the digits this record holds.
/// Why:
///  The slot is named twice so a mistyped flag refuses,
///  and both must equal the name the record's own `slot=` derives,
///  so a slot typed for another record refuses even when its file was found.
///  The digits compare against the record's own `cloneGuid=`,
///  which no slot name supplies.
///
/// # Throws
/// The flag that names another slot or another clone's digits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// confirmTheRecord(recorded, request);
/// ```
pub fn confirm_the_record(recorded: &RecordedBinding, request: &SettlingRequest) -> Result<(), SettlingStop> {
    let recorded_name: String = recorded.slot_path.file_name().map_or_else(String::new, |name| return name.to_string_lossy().into_owned());
    if recorded_name != request.slot {
        return Err(SettlingStop::TheSlotIsNotThisRecord { asked: request.slot.clone(), recorded: recorded_name });
    }
    let wanted: String = the_last_four_digits(recorded.clone_guid);
    if wanted != request.digits {
        return Err(SettlingStop::TheCloneGuidDiffers { typed: request.digits.clone(), wanted });
    }
    return Ok(());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "settling_record_tests.rs"]
mod tests;
