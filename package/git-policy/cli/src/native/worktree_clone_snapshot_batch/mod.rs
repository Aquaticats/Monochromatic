//! What:
//!  The verified snapshot set a ZFS worktree clone's removal destroys,
//! submitted as
//!  one `ZFS_IOC_DESTROY_SNAPS` request when it fits a caller's budget,
//! split by
//!  largest fitting prefix when it does not,
//! skipped when it is empty,
//! and the
//!  outcome the kernel answers with,
//! kept per name.
//! Why:
//!  Design step 3 of "Removal and recovery" destroys the clone's own snapshots
//!  in one request that omits the `defer` key,
//! keeps the per-name error list,
//!  skips an empty set
//! and,
//!  past the budget,
//! splits the set by the largest fitting prefix
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Removal and recovery";
//! `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 16).
//!  The request is the binding's own (`destroy_snaps`),
//!  sent through the
//!  wrapper's existing `/dev/zfs` layer,
//! so no second encoder and no `zfs` program
//!  exists between the verified names and the kernel.
//!  This folder is built ahead of its step and wired by no command:
//! step 16's wiring
//!  is what calls it,
//! and that wiring still owns the fresh lineage proof,
//! the
//!  deferred-origin check and the head destroy.
//!  The budget is a parameter and this piece picks no value:
//! the plan's measured budget
//!  is step 22's,
//! and until then the caller passes what step 16 has.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `request_budget.rs`:
//!   the largest encoded request input one request may carry.
//! - `request_refusal.rs`:
//!   why a set is not submitted at all,
//!   and what the caller changes.
//! - `snapshot_destroy_request.rs`:
//!   the one request,
//!   its encoded size,
//!   and the proof it names no `defer` key.
//! - `snapshot_batch_plan.rs`:
//!   the sort and the largest fitting prefix over the budget.
//! - `snapshot_name_outcome.rs`:
//!   one submitted name's answer as `destroyed`,
//!   `refused`,
//!   `intact` or
//!   `unknown`.
//! - `snapshot_batch_outcome.rs`:
//!   one request's outcome as a value,
//!   with the kernel's per-name list kept.
//! - `snapshot_batch_answer.rs`:
//!   reading a live answer into those outcomes.
//! - `snapshot_batch_send.rs`:
//!   the device seam,
//!   the empty-set skip and one request's outcome.
//!
//! The guest arms of the plan's step 16 live in `snapshot_batch_arm_tests.rs`,
//! declared for tests only,
//! and run in the throwaway `mvm` guest the recipe in
//! `package/zfs/request-to-openzfs-kernel-device/e2e/` makes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = planRequests(pool, verifiedNames, RequestBudget.ofInputBytes(measured));
//! for (const chunk of plan.chunks())
//!   const result = sendRequest(device, pool, chunk); // Skipped | Answered(BatchOutcome)
//! ```

pub mod request_budget;
pub mod request_refusal;
pub mod snapshot_batch_answer;
pub mod snapshot_batch_outcome;
pub mod snapshot_batch_plan;
pub mod snapshot_batch_send;
pub mod snapshot_destroy_request;
pub mod snapshot_name_outcome;

pub use request_budget::RequestBudget;
pub use request_refusal::BatchRefusal;
pub use snapshot_batch_answer::{
    MORE_ERRORS_KEY,
    outcome_of_answer,
};
pub use snapshot_batch_outcome::BatchOutcome;
pub use snapshot_batch_plan::{
    RequestPlan,
    plan_requests,
};
pub use snapshot_batch_send::{
    RequestResult,
    SnapshotDestroyDevice,
    send_request,
};
pub use snapshot_destroy_request::{
    DEFER_KEY,
    carries_no_defer,
    destroy_request,
    encoded_request_bytes,
};
pub use snapshot_name_outcome::{
    NameOutcome,
    SubmittedName,
};

/// The guest arms of step 16's snapshot batch,
/// compiled for tests only.
#[cfg(test)]
#[path = "snapshot_batch_arm_tests.rs"]
mod snapshot_batch_arm_tests;
