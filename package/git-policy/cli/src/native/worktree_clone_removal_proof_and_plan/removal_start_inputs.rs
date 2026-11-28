//! What:
//!  What one removal's start is made of:
//!  the six seams it reads and drives,
//!  and the two ways it can end.
//! Why:
//!  The start reads the binding,
//!  the plan and the clone's own absence,
//!  and whether it may enter at all depends on the force a person's command
//!  carried,
//!  so the count travels beside the seams rather than being read again inside the
//!  start.
//!  The two answers are what the rest of the order matches on,
//!  and they name the values each path carries rather than leaving a caller to
//!  find them again.
//!  They live together so `removal_resume.rs` holds the reads and these hold the
//!  shapes they answer with.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type TheStartingRemoval = { entry, reads, unmount, clock, probe, force };
//! type TheStart = { kind: 'present'; entered } | { kind: 'gone'; plan; planPath; lost; pool };
//! ```

use crate::worktree_clone_mount_handler::sweep_dataset_reader::DatasetReader;
use crate::worktree_clone_removal_proof_and_plan::loss_names::NamesToBeLost;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::removal_clock::RemovalClock;
use crate::worktree_clone_removal_proof_and_plan::removal_entry::{
    EnteredRemoval,
    RemovalEntry,
};
use crate::worktree_clone_removal_proof_and_plan::removal_force::RemovalForce;
use crate::worktree_clone_removal_proof_and_plan::removal_reads::RemovalPoolReads;
use crate::worktree_clone_removal_proof_and_plan::unmount_request::RemovalUnmount;
use std::path::PathBuf;

/// What:
///  Where one removal starts,
///  and what the rest of it needs from that start.
/// Why:
///  The two paths take different steps:
///  the head-present one unmounts,
/// proves and submits,
///  and the head-gone one settles and finishes without a clone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Start = { kind: 'present'; entered } | { kind: 'gone'; plan; planPath; lost; pool };
/// ```
pub enum TheStart {
    /// The clone stands,
    /// so the removal may destroy what its plan holds.
    Present {
        /// The plan this run writes or adopted,
        /// with the names it printed.
        entered: EnteredRemoval,
    },
    /// The clone is proved gone and its plan's head word says a destroy may have been sent.
    Gone {
        /// The plan,
        /// which every one of its names still records.
        plan: DestructionPlan,
        /// The plan's file.
        plan_path: PathBuf,
        /// The names a removal would lose.
        lost: NamesToBeLost,
        /// The pool its dataset names stand in.
        pool: String,
    },
}

/// What:
///  Everything one removal's start reads and drives.
/// Why:
///  The start reads the binding,
///  the plan and the clone's own absence,
///  and whether it may enter at all depends on the force a person's command
///  carried,
///  so the count travels beside the six seams rather than being read again inside
///  the start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheStartingRemoval = { entry, reads, unmount, clock, probe, force };
/// ```
pub struct TheStartingRemoval<'a> {
    /// The repository and slot this removal is about.
    pub entry: &'a RemovalEntry<'a>,
    /// The pool reads the proof and the absence check make.
    pub reads: &'a dyn RemovalPoolReads,
    /// The request that takes the clone down.
    pub unmount: &'a dyn RemovalUnmount,
    /// The process facts the plan's header carries.
    pub clock: &'a dyn RemovalClock,
    /// The reader the clone's own absence is proved through.
    pub probe: &'a dyn DatasetReader,
    /// How many forces this command carried.
    pub force: RemovalForce,
}
