//! What:
//!  Which fact of the two-fact proof differed,
//!  and the sentence a person reads for each.
//! Why:
//!  A lock whose registration is not this slot's is never released,
//!  and the sentence names the fact that failed,
//!  so a `gitdir` naming another worktree and a `commondir` naming another
//!  repository read differently.
//!  Every way forward is in the same sentence the handler prints,
//!  because a person meets the refusal on one line of a journal
//!  (`doc/planning/cli-git-zfs-managed-lock-design.md`,
//!  and `doc/decisions`' record of the second gate review's `N5`).
//!  The proof that answers with them is `registration_proof_two_facts.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Refusal = { kind: 'no-gitdir' | 'gitdir-names-another-path' | 'commondir-names-another-repository' | 'the-slot-is-not-the-recorded-folder' | 'no-recorded-slot-identity' | 'unreadable'; ... };
//! const said = refusal.words(registration);
//! ```

use super::reading_a_bounded_file::ARefusedReading;
use crate::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::path::{
    Path,
    PathBuf,
};

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
    pub fn words(
        &self,
        registration: &Path,
    ) -> String {
        return match self {
            TwoFactRefusal::NoGitdir => format!(
                "the registration `{}` holds no `gitdir` file",
                registration.display()
            ),
            TwoFactRefusal::GitdirNamesAnotherPath {
                found,
                wanted,
            } => {
                format!(
                    "the registration `{}` names `{}` in its `gitdir` and this slot's `.git` lives at `{}`",
                    registration.display(),
                    found.display(),
                    wanted.display()
                )
            }
            TwoFactRefusal::CommondirNamesAnotherRepository {
                found,
                repository,
            } => {
                format!(
                    "the registration `{}` names `{}` as its common directory and this repository is `{}`",
                    registration.display(),
                    found.display(),
                    repository.display()
                )
            }
            TwoFactRefusal::TheSlotIsNotTheRecordedFolder {
                recorded,
                found,
            } => {
                format!(
                    "the folder at the recorded slot path answers the identity `{}` and the record holds `{}`",
                    found.encode(),
                    recorded.encode()
                )
            }
            TwoFactRefusal::AMountThatIsNotTheRecords {
                found,
                recorded,
            } => {
                format!(
                    "a mount stands at the recorded slot path and it is not the clone `{recorded}` this slot's record names ({found})"
                )
            }
            TwoFactRefusal::TheMountTableWasNotRead {
                reason,
            } => {
                format!(
                    "the mount table could not be read ({reason}), so nothing says whether a mount covers the slot and the record's own folder is not compared through one"
                )
            }
            TwoFactRefusal::NoRecordedCloneGuid {
                dataset,
            } => format!(
                "a mount stands at the recorded slot path and names the clone `{dataset}` this slot's record names, and the record holds no `cloneGuid=`, so nothing was removed or rewritten; Ways forward: read the record's fields, then run the add again"
            ),
            TwoFactRefusal::TheClonesGuidWasNotRead {
                dataset,
                reason,
            } => format!(
                "a mount stands at the recorded slot path and names the clone `{dataset}`, and the pool did not answer that dataset's GUID ({reason}), so nothing was removed or rewritten; Ways forward: run `zfs get -H -o value guid {dataset}` on the machine that holds the pool, then run the add again"
            ),
            TwoFactRefusal::TheDatasetAtTheSlotHasAnotherGuid {
                dataset,
                recorded,
                found,
            } => format!(
                "a mount stands at the recorded slot path and names the clone `{dataset}`, and the pool answers GUID `{}` for that name while this slot's record holds `{}`, so a dataset made again under the same name stands where the recorded clone stood and nothing was removed or rewritten; Ways forward: check `zfs get -H -o value guid {dataset}` against the record's `cloneGuid=`, then run the add again",
                found.file_name(),
                recorded.file_name()
            ),
            TwoFactRefusal::NoRecordedSlotIdentity => String::from(
                "the record holds no slot identity, which every phase after the claim writes",
            ),
            TwoFactRefusal::Unreadable {
                what,
                cause,
            } => format!("{what} could not be read ({})", cause.words()),
        };
    }
}
