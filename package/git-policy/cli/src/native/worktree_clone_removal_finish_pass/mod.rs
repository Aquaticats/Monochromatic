//! What:
//!  The finish pass that removes a destruction plan whose head is `gone`,
//!  settles the Git registration that plan names,
//!  removes the binding's files and then the plan,
//!  and releases the slot's claim last.
//! Why:
//!  A removal killed between its head destroy and its end leaves a plan whose head is `gone`,
//!  a registration and a binding,
//!  and no next removal of that slot can run once the registration is gone,
//!  so one command has to finish that tail without a person
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The finish pass that settles a registration";
//!  `doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "What a kill at each point leaves,
//! and who finishes it").
//!  The pass is the wrapper's own driver in the place it already finishes an interrupted destroy,
//!  with the plan folder in place of the destroy records
//!  (`worktree/worktree_removal_finishes_an_interrupted_destroy.rs`).
//!  It settles nothing on a record's word:
//!  it reads the pool,
//!  proves the clone gone by its name and by no child of the pinned parent carrying its GUID,
//!  and writes nothing into any plan.
//!  This folder is built ahead of the plan's order and is wired by no command:
//!  plan step 16's own switch calls it,
//!  and no `git` command,
//! `doctor`,
//! mount handler or boot unit reaches it
//!  (`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`,
//!  "Building later pieces ahead of the plan's order").
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `pass_done_lines.rs`:
//!   the sentences for a plan and a removal the pass finished,
//!   and what a settle's answer says.
//! - `pass_refusal_lines.rs`:
//!   every sentence for a plan the pass left standing,
//!   and for entry it did not read.
//! - `the_plans_binding.rs`:
//!   the binding read and compared,
//!   its two inventories first and the binding last,
//!   each proved gone.
//! - `the_plans_file.rs`:
//!   the plan's own file,
//!   unlinked and proved gone.
//! - `the_plans_folder.rs`:
//!   the one look at the plan folder,
//!   and the empty folder's removal.
//! - `the_pass_seams.rs`:
//!   the folder look,
//!   the gate,
//!   the pool reader and the settle the pass runs through.
//! - `the_plans_registration.rs`:
//!   which registration the plan's own name holds,
//!   and the settle of it under the destruction proof.
//! - `the_slots_claim.rs`:
//!   the per-slot claim,
//!   taken under the slot the plan's `targetHex` names,
//!   and given back last.
//! - `pass_on_one_plan.rs`:
//!   one plan through the pass,
//!   in the design's order.
//! - `pass_over_the_plans.rs`:
//!   the folder's listing read as plans,
//!   and the pass's entry point.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const lines = finishTheRemovalPlans({ commonDir, folder, gate, probe, settle });
//! for (const line of lines) console.error(line);
//! ```

pub mod pass_done_lines;
pub mod pass_on_one_plan;
pub mod pass_over_the_plans;
pub mod pass_refusal_lines;
pub mod the_pass_seams;
pub mod the_plans_binding;
pub mod the_plans_file;
pub mod the_plans_folder;
pub mod the_plans_registration;
pub mod the_slots_claim;

pub use pass_over_the_plans::finish_the_removal_plans;
pub use the_pass_seams::FinishPass;

/// The repository,
/// the plan and the seams a finish-pass control starts from.
/// Never part of the release executable.
#[cfg(test)]
#[path = "pass_fixtures_tests.rs"]
pub(crate) mod pass_fixtures;

/// The guest arm of this folder's piece,
/// compiled for tests only and run in the throwaway guest.
#[cfg(test)]
#[path = "pass_guest_arm_tests.rs"]
mod pass_guest_arm_tests;
