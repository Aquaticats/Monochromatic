//! What:
//!  What one worktree-clone slot is,
//!  read without changing anything:
//!  a known unmounted clone,
//!  a pending add,
//!  a pending removal,
//!  a mounted managed clone,
//!  a slot the route holds nothing in,
//!  or a state no reading proves;
//!  the report of the slot,
//!  its dataset and the one next safe action;
//!  and the live reads those two are made from.
//! Why:
//!  Plan step 12 builds the read-only distinction between a known unmounted
//!  clone and a pending add or removal,
//!  and the report of slot,
//!  dataset and next safe action,
//!  so no reader treats an unmounted clone's absent files as deleted tracked
//!  files and none offers an ordinary add over a pending slot
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees";
//!  `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 12).
//!  The sources are the add's binding record,
//!  the removal's plan folder,
//!  the clone's mount intent and this process's mount table,
//!  and each source keeps its own refusal,
//!  so a state nothing proves is reported as that rather than as either
//!  neighbour.
//!  Nothing here writes a record,
//!  changes an intent or touches a mount:
//!  the arms that act are other steps',
//!  and this folder's own step wires it in.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `slot_state_mounts.rs`:
//!   the mount table's rows,
//!   the slot's own mount,
//!   and where the clone dataset is mounted.
//! - `slot_state_readings.rs`:
//!   the binding record's reading,
//!   the removal plan's reading,
//!   the clone's intent facts,
//!   and the one value the classification and the report share.
//! - `slot_state_classification.rs`:
//!   the state one slot is in,
//!   in the design's order of proof.
//! - `slot_state_evidence.rs`:
//!   what the readings saw,
//!   one line per source.
//! - `slot_state_live_read.rs`:
//!   the gated intent read and the call that fills one slot's readings from
//!   the machine.
//! - `slot_state_next_actions.rs`:
//!   the one next safe action for each state.
//! - `slot_state_report.rs`:
//!   the report and the lines it prints.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { readSlot, GatedCloneIntentReads } from '@monochromatic-dev/git-policy-cli/worktree-clone-slot-state';
//! import { reportOf } from '@monochromatic-dev/git-policy-cli/worktree-clone-slot-state';
//! ```

pub mod slot_state_classification;
pub mod slot_state_evidence;
pub mod slot_state_live_read;
pub mod slot_state_mounts;
pub mod slot_state_next_actions;
pub mod slot_state_readings;
pub mod slot_state_report;

pub use slot_state_classification::{
    ProofStoppedBy,
    SlotRouteState,
    classify_slot,
    slot_name_of,
};
pub use slot_state_evidence::evidence;
pub use slot_state_live_read::{
    CloneIntentReads,
    GatedCloneIntentReads,
    SlotReadingSources,
    read_slot,
};
pub use slot_state_mounts::{
    CloneMount,
    MountRows,
    SlotMount,
    clone_mount_of,
    read_mount_rows,
    slot_mount_of,
};
pub use slot_state_next_actions::{
    next_safe_action,
    registration_sentence,
};
pub use slot_state_readings::{
    BindingReading,
    IntentFacts,
    PlanReading,
    SlotReadings,
    intent_facts_of,
    read_binding_reading,
    read_plan,
};
pub use slot_state_report::{
    NO_DATASET,
    SlotStateReport,
    report_of,
};

/// The binding record's own fixture,
///  shaped here so a control of this folder and of the move refusal build the
///  same record,
///  and never part of the release executable.
#[cfg(test)]
#[path = "slot_state_test_records.rs"]
pub(crate) mod slot_state_test_records;
