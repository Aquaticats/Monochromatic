//! What:
//!  What one clone's turn reads and writes,
//!  the outcome it answers with,
//!  and the step words a receipt names:
//!  the paths and the seams of the sweep,
//!  as one value a caller cannot split,
//!  the answer a sweep reports for one clone,
//!  `identity-record`,
//!  `receipt` and `intent`.
//! Why:
//!  The turn itself is `sweep_one_clone.rs`'s,
//!  and the state it carries and the answer it gives are their own file so
//!  every file of this folder that needs one of them reads it here rather than
//!  through the turn
//!  (rule `MXL`).
//!  That is what makes the answers,
//!  the unmount's own file and the turn independent of one another:
//!  a file that only writes a receipt names these words without reaching the
//!  turn that files it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const context: SweepContext = { parentDataset, checkout, store, claims, receipts, boot, time, reader, attach, firstAttach, standing };
//! const outcome: CloneOutcome = { kind: 'attached', name, slot, mountId, canonicalSlot };
//! ```

use super::per_slot_root_claim::SlotClaimDirectory;
use super::sweep_attach_step::CloneAttachStep;
use super::sweep_dataset_reader::DatasetReader;
use super::sweep_first_attach::FirstAttachRecorder;
use super::sweep_standing_mounts::StandingMounts;
use crate::worktree_clone_mount_protocol::ReceiptsFolder;
use std::path::{
    Path,
    PathBuf,
};

/// The step a receipt and the first attach's record name when the slot's own
/// first observation could not be read or recorded.
pub const IDENTITY_RECORD_STEP: &str = "identity-record";

/// The step a receipt names when the receipt itself could not be filed.
pub const RECEIPT_STEP: &str = "receipt";

/// The step the sweep names when the durable intent changed while it waited for
/// the claim.
pub const INTENT_STEP: &str = "intent";

/// What:
///  Everything one clone's turn reads and writes.
/// Why:
///  The paths and the seams travel together so a caller cannot sweep one
///  checkout's children against another checkout's store,
///  and a control injects fixture folders and a scripted pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SweepContext = { parentDataset; checkout; checkoutDataset; store; claims; receipts; boot; time; reader; attach; firstAttach };
/// ```
pub struct SweepContext<'a> {
    /// The pinned clone parent dataset.
    pub parent_dataset: &'a str,
    /// The checkout the walk starts from.
    pub checkout: &'a Path,
    /// The checkout dataset a clone's origin must be a snapshot of.
    pub checkout_dataset: &'a str,
    /// The store of recorded identities.
    pub store: &'a Path,
    /// The per-slot claims.
    pub claims: &'a SlotClaimDirectory,
    /// The receipts folder.
    pub receipts: &'a ReceiptsFolder,
    /// This boot's kernel identifier.
    pub boot: &'a str,
    /// The clock a receipt's time is written from.
    pub time: &'a dyn Fn() -> String,
    /// The pool reads.
    pub reader: &'a dyn DatasetReader,
    /// The attach step.
    pub attach: &'a dyn CloneAttachStep,
    /// This boot's first attach,
    /// which the first attempt that passes the key check claims.
    pub first_attach: &'a dyn FirstAttachRecorder,
    /// What the canonical slot already holds.
    pub standing: &'a dyn StandingMounts,
}

/// What:
///  What one clone's turn answered.
/// Why:
///  A sweep reports every clone it reached,
///  and a clone it attached and a clone it refused must read differently in one
///  line an operator or a guest arm can find.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CloneOutcome = { kind: 'nothing-asked' | 'attached' | 'refused' | 'failed' | 'held'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloneOutcome {
    /// The clone asked for nothing,
    /// or the value it carries is inherited.
    NothingAsked {
        /// The dataset.
        name: String,
        /// The slot its name gave.
        slot: String,
        /// Why nothing was asked.
        why: String,
    },
    /// The clone's mount is at its slot.
    Attached {
        /// The dataset.
        name: String,
        /// The slot.
        slot: String,
        /// The mount the attach made,
        /// or the mount this clone already had at the slot.
        mount_id: u64,
        /// The canonical slot the mount was found at.
        canonical_slot: PathBuf,
    },
    /// Nothing was attached,
    /// and the line says why.
    Refused {
        /// The dataset.
        name: String,
        /// The slot,
        /// where the name gave one.
        slot: Option<String>,
        /// The step the answer names.
        step: String,
        /// The refusal in words.
        said: String,
    },
    /// A step failed after or during the attach,
    /// and the receipt says which,
    /// or nothing was filed when the receipt itself refused.
    Failed {
        /// The dataset.
        name: String,
        /// The slot,
        /// where the name gave one.
        slot: Option<String>,
        /// The step the receipt names.
        step: String,
        /// What the step answered.
        said: String,
    },
    /// The clone's mount and its staging bind left.
    Unmounted {
        /// The dataset.
        name: String,
        /// The slot.
        slot: String,
        /// The mount that left.
        mount_id: u64,
    },
    /// A busy unmount put the same mount back at its slot.
    Reattached {
        /// The dataset.
        name: String,
        /// The slot.
        slot: String,
        /// The clone's mount number.
        mount_id: u64,
        /// Whether the staging bind left too.
        bind_gone: bool,
    },
    /// Another sweep holds this slot.
    Held {
        /// The dataset.
        name: String,
        /// The slot another sweep holds.
        slot: String,
        /// What the claim answered.
        said: String,
    },
}

impl CloneOutcome {
    /// The dataset this outcome is about.
    pub fn name(&self) -> &str {
        return match self {
            CloneOutcome::NothingAsked {
                name,
                ..
            }
            | CloneOutcome::Attached {
                name,
                ..
            }
            | CloneOutcome::Unmounted {
                name,
                ..
            }
            | CloneOutcome::Reattached {
                name,
                ..
            }
            | CloneOutcome::Refused {
                name,
                ..
            }
            | CloneOutcome::Failed {
                name,
                ..
            }
            | CloneOutcome::Held {
                name,
                ..
            } => name.as_str(),
        };
    }

    /// The one line a sweep's report carries for this clone.
    /// It begins with the answer's own word,
    /// names the dataset and the slot,
    /// and says what a person does next when the answer was no.
    pub fn line(&self) -> String {
        return match self {
            CloneOutcome::NothingAsked {
                name,
                slot,
                why,
            } => format!("nothing {name} slot={slot} why={why}"),
            CloneOutcome::Attached {
                name,
                slot,
                mount_id,
                canonical_slot,
            } => format!(
                "attached {name} slot={slot} mount={mount_id} at={}",
                canonical_slot.display()
            ),
            CloneOutcome::Unmounted {
                name,
                slot,
                mount_id,
            } => format!("unmounted {name} slot={slot} mount={mount_id}"),
            CloneOutcome::Reattached {
                name,
                slot,
                mount_id,
                bind_gone,
            } => format!("reattached {name} slot={slot} mount={mount_id} bind-gone={bind_gone}"),
            CloneOutcome::Refused {
                name,
                slot,
                step,
                said,
            } => format!(
                "refused {name} slot={} step={step}: {said}",
                slot.as_deref().unwrap_or("none")
            ),
            CloneOutcome::Failed {
                name,
                slot,
                step,
                said,
            } => format!(
                "failed {name} slot={} step={step}: {said}",
                slot.as_deref().unwrap_or("none")
            ),
            CloneOutcome::Held {
                name,
                slot,
                said,
            } => format!("held {name} slot={slot}: {said}"),
        };
    }
}
