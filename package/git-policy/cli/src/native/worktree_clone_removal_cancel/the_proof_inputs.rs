//! What:
//!  What the removal command's proof reads,
//!  and everything one proof establishes:
//!  the repository's pinned parent,
//!  its common Git directory,
//!  the account's home,
//!  the slot,
//!  the pool reads and the absence probe,
//!  and the values the act and the report read afterwards.
//! Why:
//!  The claim is taken between the records' proof and the act,
//!  and the caller repeats these reads under it,
//!  so one value carries the records,
//!  the plan and the settled destroyed set
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  its "The proof" section).
//!  A control answers the pool reads and the absence probe and leaves the records
//!  real,
//!  so the proof's order is the same code with a scripted pool.
//!  These are the values alone,
//!  and the reads that fill or refuse them sit in the file beside this one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Proof = { checkout; commonDir; home; slot; reads; absence };
//! type Proved = { recorded; slotPath; plan?; intent; destroyed };
//! ```

use crate::worktree_clone_mount_handler::sweep_dataset_reader::DatasetReader;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountIntent;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::removal_identities::RecordedBinding;
use crate::worktree_clone_removal_proof_and_plan::removal_reads::RemovalPoolReads;
use std::path::{
    Path,
    PathBuf,
};

/// The folder this repository's worktrees live in,
/// below its pinned parent.
pub const WORKTREES_FOLDER: &str = ".claude/worktrees";

/// What:
///  Everything the proof reads.
/// Why:
///  A control answers the pool reads and the absence probe and leaves the records real,
///  so the proof's order is the same code with a scripted pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Proof = { checkout; commonDir; home; slot; reads; absence };
/// ```
pub struct TheRecordsProof<'a> {
    /// The repository's pinned parent,
    /// the checkout the slot's pointer must name a registration of.
    pub checkout: &'a Path,
    /// The repository's common Git directory.
    pub common_dir: &'a Path,
    /// The account's home,
    /// where a staging record's presence marks a removal.
    pub home: &'a Path,
    /// The slot the command was given.
    pub slot: &'a str,
    /// The pool reads the clone's identity and snapshots.
    pub reads: &'a dyn RemovalPoolReads,
    /// The reader the clone's absence is proved through,
    /// which is the same device in the release.
    pub absence: &'a dyn DatasetReader,
}

/// What:
///  Everything one proof establishes,
///  together with the reads the act needs.
/// Why:
///  The claim is taken between the records' proof and the act,
///  and the caller repeats these reads under it,
///  so one value carries the records,
///  the plan and the settled destroyed set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Proved = { recorded; slotPath; plan?; intent; destroyed };
/// ```
#[derive(Debug)]
pub struct WhatTheRecordsProve {
    /// The identities the add recorded,
    /// which the act and the report read.
    pub recorded: RecordedBinding,
    /// The slot's own folder.
    pub slot_path: PathBuf,
    /// The plan that stands,
    /// where one does.
    pub plan: Option<DestructionPlan>,
    /// The request the clone's own property carried.
    pub intent: MountIntent,
    /// The snapshots the plan's tuples prove destroyed.
    pub destroyed: Vec<String>,
}
