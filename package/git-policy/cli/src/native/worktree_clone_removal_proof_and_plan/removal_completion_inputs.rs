//! What:
//!  Everything one whole removal drives,
//!  as named seams:
//!  the entry,
//!  the pool reads,
//!  the unmount,
//!  the clock,
//!  the two readers,
//!  the two devices,
//!  the holder scan,
//!  the settle,
//!  the byte budget and the moment.
//! Why:
//!  A control answers each seam without a pool,
//!  a namespace or a repository,
//!  and the batch reads and writes through one device,
//!  the head through another,
//!  the holder scan through a third,
//!  and the settle through a fourth,
//!  so a control can make any one of them refuse and leave the rest real.
//!  The value holds the seams alone:
//!  the order that drives them is `removal_completion.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Completion = { entry; reads; unmount; clock; check; probe; snapshots; head; holders; settle; budget; now };
//! ```

use crate::worktree_clone_mount_handler::sweep_dataset_reader::DatasetReader;
use crate::worktree_clone_removal_proof_and_plan::fresh_holder_scan::FreshHolderScan;
use crate::worktree_clone_removal_proof_and_plan::removal_entry::{
    RemovalClock,
    RemovalEntry,
};
use crate::worktree_clone_removal_proof_and_plan::removal_head_destroy::DatasetDestroyDevice;
use crate::worktree_clone_removal_proof_and_plan::removal_settle::RegistrationSettle;
use crate::worktree_clone_removal_proof_and_plan::unmount_request::RemovalUnmount;
use crate::worktree_clone_snapshot_batch::request_budget::RequestBudget;
use crate::worktree_clone_snapshot_batch::snapshot_batch_send::SnapshotDestroyDevice;
/// What:
///  Everything one whole removal drives,
///  as named seams so a control answers each without a pool,
/// a namespace or a repository.
/// Why:
///  The batch reads and writes through one device,
///  the head through another,
///  the holder scan through a third,
///  and the settle through a fourth,
///  so a control can make any one of them refuse and leave the rest real.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Completion = { entry; reads; unmount; clock; check; probe; snapshots; head; holders; settle; budget; now };
/// ```
pub struct RemovalCompletion<'a> {
    /// The repository and slot this removal is about.
    pub entry: &'a RemovalEntry<'a>,
    /// The pool reads the proofs make.
    pub reads:
        &'a dyn crate::worktree_clone_removal_proof_and_plan::removal_reads::RemovalPoolReads,
    /// The request that takes the clone's mount down.
    pub unmount: &'a dyn RemovalUnmount,
    /// The process facts a new plan or an amendment records.
    pub clock: &'a dyn RemovalClock,
    /// The reader the deferred-origin checks read through.
    pub check: &'a dyn DatasetReader,
    /// The reader the absence proofs and the live intent read through.
    pub probe: &'a dyn DatasetReader,
    /// The device the snapshot requests go to.
    pub snapshots: &'a dyn SnapshotDestroyDevice,
    /// The device the head destroy goes to.
    pub head: &'a dyn DatasetDestroyDevice,
    /// The fresh all-namespace scan each destroy is preceded by.
    pub holders: &'a dyn FreshHolderScan,
    /// The Git settle.
    pub settle: &'a dyn RegistrationSettle,
    /// The bytes one snapshot request may carry.
    pub budget: RequestBudget,
    /// The moment this run's records and attempts carry.
    pub now: &'a str,
}
