//! What:
//!  The proof that the clone one record names is gone,
//!  as a value only this module can make.
//! Why:
//!  The release of the managed Git lock belongs to the authorized registration-cleanup step and runs only after the
//!  clone's destruction is proven
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  "How the wrapper's own removal releases the lock",
//!  and the plan's step 16).
//!  A proof that any caller could write is no proof,
//!  so the fields are private,
//!  the one constructor takes the live answer the settle's pool read made,
//!  and it refuses a clone that still answers,
//!  a live read that could not be made,
//!  and a record that names no GUID to prove gone.
//!  What the settle reads and how it files the plan are steps 14 and 16;
//!  this value is the seam they hand over,
//!  and the release refuses a proof that names another slot or another clone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const proof = proveTheRecordedCloneIsGone(slot, recordedClone, liveAnswer); // throws unless the clone is gone
//! ```

use super::recorded_lock_facts::RecordedClone;
use crate::worktree_clone_mount_protocol::DatasetGuid;

/// What:
///  What one live read of the pool answered about the clone the record names.
/// Why:
///  The three answers are three different situations:
///  a clone that is gone,
///  a clone that still answers,
///  and a pool that could not be asked,
///  and only the first may end in a released lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Live = { kind: 'absent' } | { kind: 'answers', dataset, guid } | { kind: 'unreadable', reason };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TheLiveCloneAnswer {
    /// Nothing on the pool answers to the clone's dataset name or its GUID.
    Absent,
    /// The clone still answers.
    Answers {
        /// The dataset name that answered.
        dataset: String,
        /// The GUID that answered.
        guid: DatasetGuid,
    },
    /// The pool could not be read,
    /// so nothing is proven either way.
    Unreadable {
        /// What the read answered.
        reason: String,
    },
}

/// What:
///  Why no proof of the clone's destruction could be made.
/// Why:
///  Each of these stops the release,
///  and each names what the settle must resolve before a lock may go.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { kind: 'the-clone-still-answers' | 'no-guid-to-prove-gone' | 'unreadable'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DestructionRefusal {
    /// The clone still answers on the pool.
    TheCloneStillAnswers {
        /// The dataset name that answered.
        dataset: String,
        /// The GUID that answered.
        guid: DatasetGuid,
    },
    /// The record names no GUID,
    /// so nothing identifies the clone a proof would be about.
    NoGuidToProveGone {
        /// The dataset the record names.
        dataset: String,
    },
    /// The pool could not be read.
    Unreadable {
        /// What the read answered.
        reason: String,
    },
}

impl DestructionRefusal {
    /// The words a person reads for one refusal.
    pub fn words(&self) -> String {
        return match self {
            DestructionRefusal::TheCloneStillAnswers { dataset, guid } => format!(
                "the clone `{dataset}` still answers on the pool with GUID `{}`, so nothing was destroyed and the lock stands",
                guid.file_name()
            ),
            DestructionRefusal::NoGuidToProveGone { dataset } => format!(
                "the record names the clone `{dataset}` and no GUID, so nothing proves which clone a destruction ended; the lock stands"
            ),
            DestructionRefusal::Unreadable { reason } => format!("the pool could not be read ({reason}), so the clone's destruction is not proven and the lock stands"),
        };
    }
}

/// What:
///  The fact that one slot's recorded clone is gone,
///  carrying the slot and the clone it was proven for.
/// Why:
///  The release takes this value and compares it with the record it settles,
///  so a proof made for another slot or another clone releases nothing,
///  and no field is public,
///  so no caller can write one without the live answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CloneDestructionProof { private constructor(readonly slot: string, readonly dataset: string, readonly guid: DatasetGuid) {} }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloneDestructionProof {
    slot: String,
    clone_dataset: String,
    clone_guid: DatasetGuid,
}

impl CloneDestructionProof {
    /// The slot this proof was made for.
    pub fn slot(&self) -> &str {
        return self.slot.as_str();
    }

    /// The clone's recorded dataset.
    pub fn clone_dataset(&self) -> &str {
        return self.clone_dataset.as_str();
    }

    /// The clone's recorded GUID.
    pub fn clone_guid(&self) -> DatasetGuid {
        return self.clone_guid;
    }
}

/// What:
///  Make the proof that one slot's recorded clone is gone.
/// Why:
///  The settle's live read is the fact,
///  and the record's GUID is what the read answers about,
///  so a clone that answers,
///  a pool that cannot be read,
///  and a record with no GUID all refuse here rather than at the release.
///
/// # Errors
/// The clone still answering,
/// no GUID to prove gone,
/// or a pool that could not be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const proof = proveTheRecordedCloneIsGone(slot, clone, live);
/// ```
pub fn prove_the_recorded_clone_is_gone(slot: &str, clone: &RecordedClone, live: &TheLiveCloneAnswer) -> Result<CloneDestructionProof, DestructionRefusal> {
    let guid: DatasetGuid = clone.guid.ok_or_else(|| return DestructionRefusal::NoGuidToProveGone { dataset: clone.dataset.clone() })?;
    return match live {
        TheLiveCloneAnswer::Absent => Ok(CloneDestructionProof { slot: String::from(slot), clone_dataset: clone.dataset.clone(), clone_guid: guid }),
        TheLiveCloneAnswer::Answers { dataset, guid: answered } => {
            Err(DestructionRefusal::TheCloneStillAnswers { dataset: dataset.clone(), guid: *answered })
        }
        TheLiveCloneAnswer::Unreadable { reason } => Err(DestructionRefusal::Unreadable { reason: reason.clone() }),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "clone_destruction_proof_tests.rs"]
mod tests;
