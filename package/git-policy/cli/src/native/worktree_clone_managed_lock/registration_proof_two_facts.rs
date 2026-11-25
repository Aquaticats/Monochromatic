//! What:
//!  The two-fact proof that a registration belongs to a slot whose pointer may be gone.
//! Why:
//!  The wrapper's existing proof starts from the worktree's pointer and ends with a `same_file` against its path,
//!  so it cannot answer for a slot with no `.git` and answers nothing there
//!  (`F14` of the design's facts).
//!  A slot with no pointer is the state S5,
//!  and the managed lock is what keeps its registration,
//!  so the lock's own reader needs a proof that survives the pointer's loss:
//!  the record's slot path and identity,
//!  the registration's `gitdir` file naming exactly `<slot>/.git`,
//!  and,
//!  when the `commondir` file stands,
//!  its `commondir` naming this repository
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  "What proves ownership").
//!  While a mount covers the slot,
//!  the recorded underlay identity cannot be read at that path,
//!  so the proof compares the mount the kernel shows there instead:
//!  a mount of the clone the record names is this slot's,
//!  and its source dataset must answer the record's own `cloneGuid=`,
//!  because a dataset made again under the same name carries another GUID;
//!  any other mount,
//!  a GUID the pool does not answer,
//!  or a mount table that cannot be read
//!  refuses rather than calling the clone's own root another folder
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  "What proves ownership",
//!  measurement `M2`,
//!  and `doc/decisions`' record of the second gate review's `N5`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const proof = proveTheRegistration(facts, repository, registration, mounts, guids); // throws the fact that differed
//! ```

use super::dataset_guid_reads::DatasetGuidReads;
use super::lexical_path_resolution::the_lexical_path;
use super::reading_a_bounded_file::{ARefusedReading, Reading, read_a_bounded_file};
use super::recorded_lock_facts::RecordedLockFacts;
use crate::hooks_path_values::without_trailing_newlines;
use crate::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use crate::worktree_clone_mount::clone_slot_git_pointer::POINTER_BOUND;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use crate::worktree_clone_slot_state::slot_state_mounts::{CloneMount, MountRows, clone_mount_of, read_mount_rows, slot_mount_of};
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasets;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

/// The name of a registration's back-link to its worktree.
pub const GITDIR_FILE_NAME: &str = "gitdir";

/// The name of a registration's record of the common directory it belongs to.
pub const COMMONDIR_FILE_NAME: &str = "commondir";

/// The most bytes this route reads of a registration's own record,
/// which is the bound the clone mount route gives the slot's pointer,
/// because both hold one path and one newline.
pub const REGISTRATION_RECORD_MAXIMUM_BYTES: u64 = POINTER_BOUND as u64;

/// What:
///  Which fact of the two-fact proof differed.
/// Why:
///  A lock whose registration is not this slot's is never released,
///  and the sentence a person reads names the fact that failed,
///  so a `gitdir` naming another worktree and a `commondir` naming another repository read differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { kind: 'no-gitdir' | 'gitdir-names-another-path' | 'commondir-names-another-repository' | 'the-slot-is-not-the-recorded-folder' | 'no-recorded-slot-identity' | 'unreadable'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TwoFactRefusal {
    /// The registration holds no `gitdir` file.
    NoGitdir,
    /// The registration's `gitdir` names another path.
    GitdirNamesAnotherPath {
        /// The path the back-link names.
        found: PathBuf,
        /// The path this slot's `.git` holds.
        wanted: PathBuf,
    },
    /// The registration's `commondir` names another repository.
    CommondirNamesAnotherRepository {
        /// The common directory the record names.
        found: PathBuf,
        /// The repository the caller is acting in.
        repository: PathBuf,
    },
    /// The folder at the recorded slot path is not the folder the record names.
    /// This is the unmounted case,
    /// where no mount covers the slot.
    TheSlotIsNotTheRecordedFolder {
        /// The identity the record holds.
        recorded: FolderIdentity,
        /// The identity the folder answers now.
        found: FolderIdentity,
    },
    /// A mount stands at the recorded slot path and it is not a mount of the clone the record names.
    AMountThatIsNotTheRecords {
        /// What the mount table says stands there.
        found: String,
        /// The clone the record names.
        recorded: String,
    },
    /// The mount table could not be read,
    /// so nothing says whether a mount covers the slot.
    TheMountTableWasNotRead {
        /// What the read answered.
        reason: String,
    },
    /// A mount of the record's clone stands at the slot,
    /// and the record holds no GUID to compare the mount's source with.
    NoRecordedCloneGuid {
        /// The clone the record names.
        dataset: String,
    },
    /// A mount of the record's clone stands at the slot,
    /// and the pool did not answer the GUID of the dataset its source names.
    TheClonesGuidWasNotRead {
        /// The clone the record names.
        dataset: String,
        /// What the read answered.
        reason: String,
    },
    /// The dataset the mount's source names answers another GUID than the record holds.
    TheDatasetAtTheSlotHasAnotherGuid {
        /// The clone the record names.
        dataset: String,
        /// The GUID the record holds.
        recorded: DatasetGuid,
        /// The GUID the pool answers for that name now.
        found: DatasetGuid,
    },
    /// The record holds no slot identity to compare,
    /// which every phase after `claimed` writes.
    NoRecordedSlotIdentity,
    /// A record of the registration,
    /// or the slot folder,
    /// could not be read.
    Unreadable {
        /// The name read.
        what: String,
        /// Why it was refused.
        cause: ARefusedReading,
    },
}

impl TwoFactRefusal {
    /// The words a person reads for one failed fact.
    pub fn words(&self, registration: &Path) -> String {
        return match self {
            TwoFactRefusal::NoGitdir => format!("the registration `{}` holds no `gitdir` file", registration.display()),
            TwoFactRefusal::GitdirNamesAnotherPath { found, wanted } => {
                format!("the registration `{}` names `{}` in its `gitdir` and this slot's `.git` lives at `{}`", registration.display(), found.display(), wanted.display())
            }
            TwoFactRefusal::CommondirNamesAnotherRepository { found, repository } => {
                format!("the registration `{}` names `{}` as its common directory and this repository is `{}`", registration.display(), found.display(), repository.display())
            }
            TwoFactRefusal::TheSlotIsNotTheRecordedFolder { recorded, found } => {
                format!("the folder at the recorded slot path answers the identity `{}` and the record holds `{}`", found.encode(), recorded.encode())
            }
            TwoFactRefusal::AMountThatIsNotTheRecords { found, recorded } => {
                format!("a mount stands at the recorded slot path and it is not the clone `{recorded}` this slot's record names ({found})")
            }
            TwoFactRefusal::TheMountTableWasNotRead { reason } => {
                format!("the mount table could not be read ({reason}), so nothing says whether a mount covers the slot and the record's own folder is not compared through one")
            }
            TwoFactRefusal::NoRecordedCloneGuid { dataset } => format!(
                "a mount stands at the recorded slot path and names the clone `{dataset}` this slot's record names, and the record holds no `cloneGuid=`, so nothing was removed or rewritten; Ways forward: read the record's fields, then run the add again"
            ),
            TwoFactRefusal::TheClonesGuidWasNotRead { dataset, reason } => format!(
                "a mount stands at the recorded slot path and names the clone `{dataset}`, and the pool did not answer that dataset's GUID ({reason}), so nothing was removed or rewritten; Ways forward: run `zfs get -H -o value guid {dataset}` on the machine that holds the pool, then run the add again"
            ),
            TwoFactRefusal::TheDatasetAtTheSlotHasAnotherGuid { dataset, recorded, found } => format!(
                "a mount stands at the recorded slot path and names the clone `{dataset}`, and the pool answers GUID `{}` for that name while this slot's record holds `{}`, so a dataset made again under the same name stands where the recorded clone stood and nothing was removed or rewritten; Ways forward: check `zfs get -H -o value guid {dataset}` against the record's `cloneGuid=`, then run the add again",
                found.file_name(),
                recorded.file_name()
            ),
            TwoFactRefusal::NoRecordedSlotIdentity => String::from("the record holds no slot identity, which every phase after the claim writes"),
            TwoFactRefusal::Unreadable { what, cause } => format!("{what} could not be read ({})", cause.words()),
        };
    }
}

/// Read one record of a registration,
/// with a refused read returned as the refusal it is.
fn record_bytes(registration: &Path, name: &str) -> Result<Reading, TwoFactRefusal> {
    let path: PathBuf = registration.join(name);
    return match read_a_bounded_file(path.as_path(), REGISTRATION_RECORD_MAXIMUM_BYTES) {
        Reading::Refused(cause) => Err(TwoFactRefusal::Unreadable { what: format!("`{}`", path.display()), cause }),
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
fn named_in_the_registration(reading: &Reading, registration: &Path) -> Option<PathBuf> {
    let Reading::Bytes(bytes) = reading else { return None };
    let named: PathBuf = PathBuf::from(std::ffi::OsStr::from_bytes(without_trailing_newlines(bytes.as_slice())));
    let spelled: PathBuf = if named.is_absolute() { named } else { registration.join(named) };
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
pub fn prove_the_registration(facts: &RecordedLockFacts, repository: &Path, registration: &Path, mounts: &dyn MountedDatasets, guids: &dyn DatasetGuidReads) -> Result<(), TwoFactRefusal> {
    let gitdir: Reading = record_bytes(registration, GITDIR_FILE_NAME)?;
    if let Reading::Refused(cause) = gitdir {
        return Err(TwoFactRefusal::Unreadable { what: format!("`{}`", registration.join(GITDIR_FILE_NAME).display()), cause });
    }
    let spelled: PathBuf = facts.slot.join(".git");
    let wanted: PathBuf = the_lexical_path(spelled.as_path());
    let Some(back_link) = named_in_the_registration(&gitdir, registration) else {
        return Err(TwoFactRefusal::NoGitdir);
    };
    if back_link != wanted {
        return Err(TwoFactRefusal::GitdirNamesAnotherPath { found: back_link, wanted });
    }
    let commondir: Reading = record_bytes(registration, COMMONDIR_FILE_NAME)?;
    if let Reading::Refused(cause) = commondir {
        return Err(TwoFactRefusal::Unreadable { what: format!("`{}`", registration.join(COMMONDIR_FILE_NAME).display()), cause });
    }
    if let Some(common) = named_in_the_registration(&commondir, registration)
        && !crate::real_git_candidate::same_file(common.as_path(), repository)
    {
        return Err(TwoFactRefusal::CommondirNamesAnotherRepository { found: common, repository: repository.to_path_buf() });
    }
    let recorded: FolderIdentity = facts.slot_identity.ok_or(TwoFactRefusal::NoRecordedSlotIdentity)?;
    return match the_cover_at(facts, mounts) {
        // A mount of the record's clone stands at the slot, so the folder there is the clone's root
        // and not the underlay the add identified: the clone's own GUID is what names this slot,
        // because a dataset made again under the same name answers another one.
        CloneMount::AtItsSlot => proven_through_the_clone(facts, guids),
        // Anything else standing at the slot is not this record's clone, and a table nothing read
        // proves neither the mounted fact nor the unmounted one.
        CloneMount::SomethingElseAtTheSlot { described } => Err(TwoFactRefusal::AMountThatIsNotTheRecords { found: described, recorded: facts.clone.dataset.clone() }),
        CloneMount::Unreadable { reason } => Err(TwoFactRefusal::TheMountTableWasNotRead { reason }),
        // Nothing covers the slot, or the clone is mounted somewhere else, so the recorded
        // underlay identity is what the folder at this path must answer.
        CloneMount::Nowhere | CloneMount::Elsewhere { .. } => {
            let found: FolderIdentity = FolderIdentity::of_folder(facts.slot.as_path()).map_err(|error| {
                return TwoFactRefusal::Unreadable { what: format!("the slot `{}`", facts.slot.display()), cause: ARefusedReading::Unreadable { reason: error.to_string() } };
            })?;
            match found == recorded {
                true => Ok(()),
                false => Err(TwoFactRefusal::TheSlotIsNotTheRecordedFolder { recorded, found }),
            }
        }
    };
}

/// What:
///  The mounted fact: the clone dataset the mount's source names answers the record's `cloneGuid=`.
/// Why:
///  The mount table compares a name,
///  and a name alone passes a dataset made again under it after the recorded one was destroyed,
///  so the pool's own answer about that name is the second fact,
///  and a record with no GUID,
///  a pool that does not answer,
///  and an answer that differs
///  are three refusals rather than a pass.
fn proven_through_the_clone(facts: &RecordedLockFacts, guids: &dyn DatasetGuidReads) -> Result<(), TwoFactRefusal> {
    let dataset: String = facts.clone.dataset.clone();
    let recorded: DatasetGuid = facts.clone.guid.ok_or_else(|| return TwoFactRefusal::NoRecordedCloneGuid { dataset: dataset.clone() })?;
    let found: DatasetGuid = guids.guid_of(dataset.as_str()).map_err(|reason| return TwoFactRefusal::TheClonesGuidWasNotRead { dataset: dataset.clone(), reason })?;
    return match found == recorded {
        true => Ok(()),
        false => Err(TwoFactRefusal::TheDatasetAtTheSlotHasAnotherGuid { dataset, recorded, found }),
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
fn the_cover_at(facts: &RecordedLockFacts, mounts: &dyn MountedDatasets) -> CloneMount {
    let rows: MountRows = read_mount_rows(mounts);
    let slot_mount = slot_mount_of(mounts, facts.slot.as_path());
    return clone_mount_of(&rows, &slot_mount, facts.clone.dataset.as_str(), facts.slot.as_path());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "registration_proof_two_facts_tests.rs"]
mod tests;
