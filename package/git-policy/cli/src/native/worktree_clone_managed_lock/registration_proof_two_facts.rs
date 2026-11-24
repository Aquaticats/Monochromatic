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
//!  so the comparison refuses there rather than calling the clone's own root another folder;
//!  what the mounted case compares is measurement `M2`'s and the wiring step's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const proof = proveTheRegistration(facts, repository, registration); // throws the fact that differed
//! ```

use super::lexical_path_resolution::the_lexical_path;
use super::reading_a_bounded_file::{ARefusedReading, Reading, read_a_bounded_file};
use super::recorded_lock_facts::RecordedLockFacts;
use crate::hooks_path_values::without_trailing_newlines;
use crate::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use crate::worktree_clone_mount::clone_slot_git_pointer::POINTER_BOUND;
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
    TheSlotIsNotTheRecordedFolder {
        /// The identity the record holds.
        recorded: FolderIdentity,
        /// The identity the folder answers now.
        found: FolderIdentity,
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
///
/// # Errors
/// The fact that differed,
/// with the two values it compared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// proveTheRegistration(facts, repository, registration);
/// ```
pub fn prove_the_registration(facts: &RecordedLockFacts, repository: &Path, registration: &Path) -> Result<(), TwoFactRefusal> {
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
    let found: FolderIdentity = FolderIdentity::of_folder(facts.slot.as_path()).map_err(|error| {
        return TwoFactRefusal::Unreadable { what: format!("the slot `{}`", facts.slot.display()), cause: ARefusedReading::Unreadable { reason: error.to_string() } };
    })?;
    if found != recorded {
        return Err(TwoFactRefusal::TheSlotIsNotTheRecordedFolder { recorded, found });
    }
    return Ok(());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "registration_proof_two_facts_tests.rs"]
mod tests;
