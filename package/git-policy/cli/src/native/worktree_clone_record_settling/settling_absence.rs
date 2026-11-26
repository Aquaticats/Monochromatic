//! What:
//!  What the pool answers about the recorded clone's existence,
//!  read by its name and by its GUID under the pinned parent.
//! Why:
//!  The state this command exists for is a clone whose absence cannot be proved,
//!  so a refused or partial listing is an answer and never a stop:
//!  it is kept,
//!  printed as evidence,
//!  and the person confirms against it
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The settling command",
//!  read 3,
//!  and W3 of the fourth removal-side check).
//!  A clone that answers,
//!  a name reused by another GUID and a GUID carried by another name are stops:
//!  each proves the recorded clone may stand,
//!  and no removal and no settling command acts on a slot whose clone stands.
//!  The proof itself is the removal's own,
//!  so the settling command and a removal can never disagree about what
//!  "gone" means.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const presence = theClonesPresence(probe, 'tank/wt/topic', guid);
//! ```

use crate::worktree_clone_mount_handler::sweep_dataset_reader::DatasetReader;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::removal_head_destroy::{
    CloneAbsence,
    the_clone_absence,
};

/// What:
///  The four answers a pool gives about one recorded clone.
/// Why:
///  Each leads one way:
///  a clone that stands and a name or GUID that answers elsewhere refuse,
///  a proved absence goes on to confirm the removals,
///  and an absence the pool could not prove goes on too,
///  with the refusal kept as the evidence a person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Presence = { kind: 'stands'; name; guid } | { kind: 'elsewhere'; found } | { kind: 'gone' } | { kind: 'unprovable'; evidence };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TheClonesPresence {
    /// The recorded name answers with the recorded GUID.
    Stands {
        /// The name that answered.
        name: String,
        /// The GUID it answered.
        guid: DatasetGuid,
    },
    /// The recorded name answers with another GUID.
    TheNameHoldsAnother {
        /// The name that answered.
        name: String,
        /// The GUID it answered.
        guid: DatasetGuid,
    },
    /// Another name under the pinned parent carries the recorded GUID.
    AnswersElsewhere {
        /// The dataset that carries it.
        found: String,
    },
    /// No child of the pinned parent carries the name or the GUID.
    ProvedGone,
    /// The pool refused the listing or answered it only in part,
    /// and this is what it said.
    Unprovable {
        /// The refusal,
        /// kept for the lines a person reads.
        evidence: String,
    },
}

impl TheClonesPresence {
    /// Whether this answer proves the recorded clone may stand.
    pub fn stops(&self) -> bool {
        return matches!(
            self,
            TheClonesPresence::Stands { .. }
                | TheClonesPresence::TheNameHoldsAnother { .. }
                | TheClonesPresence::AnswersElsewhere { .. }
        );
    }

    /// The sentence a line prints for a stop,
    /// or nothing for an answer the run goes on from.
    pub fn stop_sentence(
        &self
    ) -> Option<crate::worktree_clone_record_settling::settling_refusal::SettlingStop> {
        return match self {
            TheClonesPresence::Stands { name, guid } => Some(crate::worktree_clone_record_settling::settling_refusal::SettlingStop::TheCloneStillStands { clone: name.clone(), guid: guid.file_name() }),
            TheClonesPresence::TheNameHoldsAnother { name, guid } => {
                Some(crate::worktree_clone_record_settling::settling_refusal::SettlingStop::TheNameHoldsAnother { name: name.clone(), guid: guid.file_name() })
            }
            TheClonesPresence::AnswersElsewhere { found } => Some(crate::worktree_clone_record_settling::settling_refusal::SettlingStop::TheCloneAnswersElsewhere { found: found.clone() }),
            TheClonesPresence::ProvedGone | TheClonesPresence::Unprovable { .. } => None,
        };
    }
}

/// What:
///  The pool's answer about one recorded clone,
///  as the settling command reads it.
/// Why:
///  The removal's proof answers with a stop where a listing refused,
///  and this command reads that same answer as the state it exists for,
///  so the mapping is stated once here rather than in the command's steps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const presence = theClonesPresence(probe, clone, guid);
/// ```
pub fn the_clones_presence(
    probe: &dyn DatasetReader,
    clone_dataset: &str,
    clone_guid: DatasetGuid,
) -> TheClonesPresence {
    return match the_clone_absence(probe, clone_dataset, clone_guid) {
        Ok(CloneAbsence::There {
            name,
            guid,
        }) => match name == clone_dataset {
            true => TheClonesPresence::Stands {
                name,
                guid,
            },
            false => TheClonesPresence::AnswersElsewhere {
                found: name,
            },
        },
        Ok(CloneAbsence::Reused {
            guid,
        }) => TheClonesPresence::TheNameHoldsAnother {
            name: String::from(clone_dataset),
            guid,
        },
        Ok(CloneAbsence::Gone) => TheClonesPresence::ProvedGone,
        Err(stop) => TheClonesPresence::Unprovable {
            evidence: stop.message(),
        },
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "settling_absence_tests.rs"]
mod tests;
