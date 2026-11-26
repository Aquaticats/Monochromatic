//! What:
//!  The one release of a managed Git lock,
//!  open only to the registration-cleanup step and only when it is handed the proof that the clone is gone.
//! Why:
//!  The lock protects the registration of a clone that is destroyed in the settle,
//!  so the release runs there and never in `doctor`,
//!  which holds no claim and is read-only
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  "How the wrapper's own removal releases the lock").
//!  The release is the design's:
//!  one `unlinkat` of `locked` through the held registration descriptor and one sync of the registration's directory,
//!  and before it the lock must be the add's own,
//!  whole or cut short,
//!  and the proof must name the slot and the clone the record names.
//!  A lock that does not stand refuses here rather than passing silently,
//!  because the decision of 2026-11-22 has the settle write a lost lock again first,
//!  so the settle releases a lock that was there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = releaseTheManagedLock(registration, registrationPath, slot, facts, proof);
//! // { kind: 'released' } | throws the refusal
//! ```

use super::clone_destruction_proof::CloneDestructionProof;
use super::lock_installation::LOCK_FILE_NAME;
use super::reading_a_bounded_file::ARefusedReading;
use super::recorded_lock_facts::RecordedLockFacts;
use crate::entries_of_one_open_folder::{
    open_entry_for_reading,
    read_entry_bytes,
};
use crate::private_storage::ReadRefusal;
use crate::publishing_entries_in_one_open_folder::sync_folder;
use crate::worktree_clone_binding_record::lock_words::{
    LOCK_WORDS_MAXIMUM_BYTES,
    LockStanding,
    lock_standing_of,
};
use std::ffi::OsStr;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Why the release refused.
/// Why:
///  A refusal names the fact that stopped it,
///  and no refusal removes anything:
///  a lock whose release is refused stands where it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { kind: 'proof-for-another-slot' | 'proof-for-another-clone' | 'no-lock-stands' | 'not-the-adds-lock' | 'refused'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReleaseRefusal {
    /// The proof was made for another slot.
    TheProofIsForAnotherSlot {
        /// The slot the proof names.
        proof: String,
        /// The slot being settled.
        settling: String,
    },
    /// The proof was made for another clone.
    TheProofIsForAnotherClone {
        /// The clone the proof names.
        proof: String,
        /// The clone the record names.
        recorded: String,
    },
    /// No lock stands to release,
    /// which the settle writes again first.
    NoLockStands,
    /// The lock's bytes are not this add's,
    /// or the read refused them.
    NotTheAddsLock {
        /// Why not.
        cause: ARefusedReading,
    },
    /// The system refused a call.
    Refused {
        /// What the call answered.
        reason: String,
    },
}

impl ReleaseRefusal {
    /// The words a person reads for one refusal.
    pub fn words(&self) -> String {
        return match self {
            ReleaseRefusal::TheProofIsForAnotherSlot {
                proof,
                settling,
            } => format!(
                "the destruction proof names the slot `{proof}` and this settle releases the lock of `{settling}`, so nothing was released"
            ),
            ReleaseRefusal::TheProofIsForAnotherClone {
                proof,
                recorded,
            } => {
                format!(
                    "the destruction proof names the clone `{proof}` and the record names `{recorded}`, so nothing was released"
                )
            }
            ReleaseRefusal::NoLockStands => String::from(
                "no managed Git lock stands in the registration, and a settle writes a lost lock again before it releases it, so nothing was released",
            ),
            ReleaseRefusal::NotTheAddsLock {
                cause,
            } => format!(
                "the lock is not this add's ({}), so nothing was released",
                cause.words()
            ),
            ReleaseRefusal::Refused {
                reason,
            } => format!("the release was refused ({reason}), so the lock stands where it was"),
        };
    }
}

/// What:
///  What one release did.
/// Why:
///  The settle records that it released the lock and from which registration,
///  so the answer names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Outcome = { kind: 'released', registration };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseOutcome {
    /// The registration the lock went from.
    pub registration: PathBuf,
}

/// The lock's bytes read through the held registration descriptor,
/// or the read's own refusal.
fn bytes_through_the_descriptor(registration: &std::fs::File) -> Result<Vec<u8>, ReadRefusal> {
    let mut file: std::fs::File = open_entry_for_reading(registration, OsStr::new(LOCK_FILE_NAME))?;
    return read_entry_bytes(&mut file, LOCK_WORDS_MAXIMUM_BYTES);
}

/// What:
///  Release one slot's managed Git lock after its clone's destruction is proven.
/// Why:
///  This is the only function that removes a lock the wrapper wrote,
///  and it refuses unless it is handed a destruction proof for this slot and this clone,
///  the lock stands as the add's own,
///  and then removes `locked` through the held descriptor and syncs the folder,
///  so a kill after the removal leaves no lock and a kill before it leaves one the finder still recognises.
///
/// # Errors
/// The proof naming another slot or clone,
/// no lock standing,
/// the lock being another's,
/// or the system refusing a call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (proof.slot !== slot) throw ...; if (proof.clone !== facts.clone.dataset) throw ...;
/// if (!lockStandsAsTheAddsOwn) throw ...; await unlinkat(registration, 'locked'); await syncFolder(registration);
/// ```
pub fn release_the_managed_lock(
    registration: &std::fs::File,
    registration_path: &Path,
    slot: &str,
    facts: &RecordedLockFacts,
    proof: &CloneDestructionProof,
) -> Result<ReleaseOutcome, ReleaseRefusal> {
    if proof.slot() != slot {
        return Err(ReleaseRefusal::TheProofIsForAnotherSlot {
            proof: String::from(proof.slot()),
            settling: String::from(slot),
        });
    }
    if proof.clone_dataset() != facts.clone.dataset || Some(proof.clone_guid()) != facts.clone.guid
    {
        return Err(ReleaseRefusal::TheProofIsForAnotherClone {
            proof: String::from(proof.clone_dataset()),
            recorded: facts.clone.dataset.clone(),
        });
    }
    let bytes: Vec<u8> = match bytes_through_the_descriptor(registration) {
        Ok(found) => found,
        Err(ReadRefusal::Missing) => return Err(ReleaseRefusal::NoLockStands),
        Err(ReadRefusal::NotRegular) => {
            return Err(ReleaseRefusal::NotTheAddsLock {
                cause: ARefusedReading::AFileOfAnotherKind,
            });
        }
        Err(ReadRefusal::TooLarge) => {
            return Err(ReleaseRefusal::NotTheAddsLock {
                cause: ARefusedReading::Unreadable {
                    reason: format!(
                        "it holds more than the {LOCK_WORDS_MAXIMUM_BYTES} bytes the sentence and Git's newline take"
                    ),
                },
            });
        }
        Err(ReadRefusal::Io(error)) => {
            return Err(ReleaseRefusal::Refused {
                reason: error.to_string(),
            });
        }
    };
    match lock_standing_of(bytes.as_slice(), &facts.nonce) {
        LockStanding::Verified
        | LockStanding::CutShort {
            ..
        } => {}
        LockStanding::Foreign => {
            return Err(ReleaseRefusal::NotTheAddsLock {
                cause: ARefusedReading::Unreadable {
                    reason: String::from(
                        "the bytes are not the sentence recomposed from the record's nonce",
                    ),
                },
            });
        }
    }
    return match crate::fs_beneath::remove_at(registration, OsStr::new(LOCK_FILE_NAME), false) {
        Ok(()) => match sync_folder(registration) {
            Ok(()) => Ok(ReleaseOutcome {
                registration: registration_path.to_path_buf(),
            }),
            Err(error) => Err(ReleaseRefusal::Refused {
                reason: error.to_string(),
            }),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Err(ReleaseRefusal::NoLockStands)
        }
        Err(error) => Err(ReleaseRefusal::Refused {
            reason: error.to_string(),
        }),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "lock_release_tests.rs"]
mod tests;
