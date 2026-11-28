//! What:
//!  The proof of a registration that needs no pointer:
//!  one registration's two records,
//!  the folder identity the slot's record holds,
//!  and the clone-name or clone-GUID fact that names the slot while a mount
//!  covers it.
//! Why:
//!  The managed lock outlives the slot's pointer,
//!  so the lock may not be judged through it:
//!  the record's slot path and identity,
//!  the `gitdir` back-link and the `commondir` name the registration,
//!  and a folder whose identity differs is not the folder the add recorded
//!  (`doc/planning/cli-git-zfs-managed-lock-design.md`,
//!  and `doc/decisions`' record of the second gate review's `N5`).
//!  While a mount covers the slot that folder is the clone's root and not the
//!  underlay the record identified,
//!  so a record whose clone the mount table shows at the slot is proved through
//!  the clone's own recorded GUID instead.
//!  The refusals it answers with are `registration_proof_two_facts_refusals.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const proof = proveTheRegistration(facts, repository, registration, mounts, guids); // throws the fact that differed
//! ```

use super::dataset_guid_reads::DatasetGuidReads;
use super::lexical_path_resolution::the_lexical_path;
use super::reading_a_bounded_file::{
    ARefusedReading,
    Reading,
    read_a_bounded_file,
};
use super::recorded_lock_facts::RecordedLockFacts;
use crate::hooks_path_values::without_trailing_newlines;
use crate::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use crate::worktree_clone_mount::clone_slot_git_pointer::POINTER_BOUND;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use crate::worktree_clone_slot_state::slot_state_mounts::{
    CloneMount,
    MountRows,
    clone_mount_of,
    read_mount_rows,
    slot_mount_of,
};
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasets;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{
    Path,
    PathBuf,
};

// What:     the refusal a caller of the proof names through this module, whose
//           enum and sentences now live in a sibling, re-exported where it was
//           declared before the split.
// Why:      the lock's own classification and another folder's link facts name
//           it through this module's path, and the control file that is named
//           for this module reaches it with `super::`, so a split that moved it
//           would break each of them.
//
// In TS you'd write (pseudocode):
// ```ts
// export { TwoFactRefusal } from './registrationProofTwoFactsRefusals.js';   // for the controls
// ```
pub use super::registration_proof_two_facts_refusals::TwoFactRefusal;

/// The name of a registration's back-link to its worktree.
pub const GITDIR_FILE_NAME: &str = "gitdir";

/// The name of a registration's record of the common directory it belongs to.
pub const COMMONDIR_FILE_NAME: &str = "commondir";

/// The most bytes this route reads of a registration's own record,
/// which is the bound the clone mount route gives the slot's pointer,
/// because both hold one path and one newline.
pub const REGISTRATION_RECORD_MAXIMUM_BYTES: u64 = POINTER_BOUND as u64;

/// Read one record of a registration,
/// with a refused read returned as the refusal it is.
fn record_bytes(
    registration: &Path,
    name: &str,
) -> Result<Reading, TwoFactRefusal> {
    let path: PathBuf = registration.join(name);
    return match read_a_bounded_file(path.as_path(), REGISTRATION_RECORD_MAXIMUM_BYTES) {
        Reading::Refused(cause) => Err(TwoFactRefusal::Unreadable {
            what: format!("`{}`", path.display()),
            cause,
        }),
        reading => Ok(reading),
    };
}

/// What:
///  One path a registration's record names,
///  with its ending newline trimmed and a relative spelling resolved against the registration.
/// Why:
///  Git writes the worktree's whole path in `gitdir` and `../..` in `commondir`,
///  and a registration another command wrote may hold either spelling,
///  so both are read as the files they name.
fn named_in_the_registration(
    reading: &Reading,
    registration: &Path,
) -> Option<PathBuf> {
    let Reading::Bytes(bytes) = reading else {
        return None;
    };
    let named: PathBuf = PathBuf::from(std::ffi::OsStr::from_bytes(without_trailing_newlines(
        bytes.as_slice(),
    )));
    let spelled: PathBuf = if named.is_absolute() {
        named
    } else {
        registration.join(named)
    };
    return Some(the_lexical_path(spelled.as_path()));
}

/// What:
///  Prove a registration one slot's record explains without reading the slot's pointer.
/// Why:
///  The managed lock outlives the pointer,
///  so the proof may not need it:
///  the record's slot path and identity,
///  the `gitdir` back-link,
///  and the `commondir` name the registration,
///  and a folder whose identity differs is not the folder the add recorded.
///  While a mount covers the slot that folder is the clone's root and not the underlay
///  the record identified,
///  so a record whose clone the mount table shows at the slot is proved through the
///  clone's own recorded GUID instead,
///  and any other mount there,
///  a GUID the pool does not answer,
///  or a record with no GUID to compare
///  is refused rather than compared.
///
/// # Errors
/// The fact that differed,
/// with the two values it compared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// proveTheRegistration(facts, repository, registration, mounts, guids);
/// ```
pub fn prove_the_registration(
    facts: &RecordedLockFacts,
    repository: &Path,
    registration: &Path,
    mounts: &dyn MountedDatasets,
    guids: &dyn DatasetGuidReads,
) -> Result<(), TwoFactRefusal> {
    let gitdir: Reading = record_bytes(registration, GITDIR_FILE_NAME)?;
    if let Reading::Refused(cause) = gitdir {
        return Err(TwoFactRefusal::Unreadable {
            what: format!("`{}`", registration.join(GITDIR_FILE_NAME).display()),
            cause,
        });
    }
    let spelled: PathBuf = facts.slot.join(".git");
    let wanted: PathBuf = the_lexical_path(spelled.as_path());
    let Some(back_link) = named_in_the_registration(&gitdir, registration) else {
        return Err(TwoFactRefusal::NoGitdir);
    };
    if back_link != wanted {
        return Err(TwoFactRefusal::GitdirNamesAnotherPath {
            found: back_link,
            wanted,
        });
    }
    let commondir: Reading = record_bytes(registration, COMMONDIR_FILE_NAME)?;
    if let Reading::Refused(cause) = commondir {
        return Err(TwoFactRefusal::Unreadable {
            what: format!("`{}`", registration.join(COMMONDIR_FILE_NAME).display()),
            cause,
        });
    }
    if let Some(common) = named_in_the_registration(&commondir, registration)
        && !crate::real_git_candidate::same_file(common.as_path(), repository)
    {
        return Err(TwoFactRefusal::CommondirNamesAnotherRepository {
            found: common,
            repository: repository.to_path_buf(),
        });
    }
    let recorded: FolderIdentity = facts
        .slot_identity
        .ok_or(TwoFactRefusal::NoRecordedSlotIdentity)?;
    return match the_cover_at(facts, mounts) {
        // A mount of the record's clone stands at the slot, so the folder there is the clone's root
        // and not the underlay the add identified: the clone's own GUID is what names this slot,
        // because a dataset made again under the same name answers another one.
        CloneMount::AtItsSlot => proven_through_the_clone(facts, guids),
        // Anything else standing at the slot is not this record's clone, and a table nothing read
        // proves neither the mounted fact nor the unmounted one.
        CloneMount::SomethingElseAtTheSlot {
            described,
        } => Err(TwoFactRefusal::AMountThatIsNotTheRecords {
            found: described,
            recorded: facts.clone.dataset.clone(),
        }),
        CloneMount::Unreadable {
            reason,
        } => Err(TwoFactRefusal::TheMountTableWasNotRead {
            reason,
        }),
        // Nothing covers the slot, or the clone is mounted somewhere else, so the recorded
        // underlay identity is what the folder at this path must answer.
        CloneMount::Nowhere
        | CloneMount::Elsewhere {
            ..
        }
        | CloneMount::AtTheStagingFolder {
            ..
        } => {
            let found: FolderIdentity =
                FolderIdentity::of_folder(facts.slot.as_path()).map_err(|error| {
                    return TwoFactRefusal::Unreadable {
                        what: format!("the slot `{}`", facts.slot.display()),
                        cause: ARefusedReading::Unreadable {
                            reason: error.to_string(),
                        },
                    };
                })?;
            match found == recorded {
                true => Ok(()),
                false => Err(TwoFactRefusal::TheSlotIsNotTheRecordedFolder {
                    recorded,
                    found,
                }),
            }
        }
    };
}

/// What:
///  The mounted fact:
///  the clone dataset the mount's source names answers the record's `cloneGuid=`.
/// Why:
///  The mount table compares a name,
///  and a name alone passes a dataset made again under it after the recorded one was destroyed,
///  so the pool's own answer about that name is the second fact,
///  and a record with no GUID,
///  a pool that does not answer,
///  and an answer that differs
///  are three refusals rather than a pass.
fn proven_through_the_clone(
    facts: &RecordedLockFacts,
    guids: &dyn DatasetGuidReads,
) -> Result<(), TwoFactRefusal> {
    let dataset: String = facts.clone.dataset.clone();
    let recorded: DatasetGuid = facts.clone.guid.ok_or_else(|| {
        return TwoFactRefusal::NoRecordedCloneGuid {
            dataset: dataset.clone(),
        };
    })?;
    let found: DatasetGuid = guids.guid_of(dataset.as_str()).map_err(|reason| {
        return TwoFactRefusal::TheClonesGuidWasNotRead {
            dataset: dataset.clone(),
            reason,
        };
    })?;
    return match found == recorded {
        true => Ok(()),
        false => Err(TwoFactRefusal::TheDatasetAtTheSlotHasAnotherGuid {
            dataset,
            recorded,
            found,
        }),
    };
}

/// What:
///  Where the record's clone stands,
///  judged by the caller's mount table.
/// Why:
///  A mount of another dataset,
///  a bind mount of a subtree,
///  a mount at another path,
///  and a table nothing read
///  are four different answers,
///  and only the clone's own root at the slot makes the recorded underlay identity
///  unreadable there.
fn the_cover_at(
    facts: &RecordedLockFacts,
    mounts: &dyn MountedDatasets,
) -> CloneMount {
    let rows: MountRows = read_mount_rows(mounts);
    let slot_mount = slot_mount_of(mounts, facts.slot.as_path());
    return clone_mount_of(
        &rows,
        &slot_mount,
        facts.clone.dataset.as_str(),
        facts.slot.as_path(),
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "registration_proof_two_facts_tests.rs"]
mod tests;
