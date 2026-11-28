//! What:
//!  The manual way back from a removal that did not finish:
//!  the guarded `cancel-worktree-removal <slot>` command,
//!  and the state a removal restores when it cancels itself.
//! Why:
//!  Plan step 17 builds what a cancelled,
//!  timed-out or killed removal leaves and the command that ends it
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  "Step 17";
//!  `doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  "The cancel command" and "What a removal that cancels itself restores").
//!  The project's answer keeps the command as the manual way back for a removal whose wrapper was
//!  killed after it recorded the unmount or that ran past its deadline,
//!  and requires it to report every snapshot already destroyed and restore none
//!  (`doc/decision/cli-git-zfs-step-17-forced-and-cancelled-removal-designs-accepted.md`).
//!  This folder is reached in this release two ways:
//!  the management grammar admits `cancel-worktree-removal`,
//!  which runs `the_cancel_command::cancel_the_removal`,
//!  and the removal's own stop points call
//!  `the_cancelled_removal::the_removal_cancels_itself` through the removal's seam.
//!  The removal order may not name this folder,
//!  so its stop points ask it through the removal's own seam and
//!  `the_self_cancelling_answer.rs` is where the answer lives.
//!  The forced arm is another round's:
//!  this folder restores the state its failures leave and never ends a process.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `cancel_arguments.rs`:
//!   the command's grammar and every caller mistake.
//! - `cancel_refusal.rs`:
//!   every state that prevents the command,
//!   with its exit code and the sentence a person reads.
//! - `the_destroy_outcomes.rs`:
//!   the plan's tuples settled by the clone's live inventory.
//! - `the_plan_it_leaves.rs`:
//!   the one plan action a cancellation takes.
//! - `the_proof_inputs.rs`:
//!   what the proof reads and everything one proof establishes.
//! - `the_proof_reads.rs`:
//!   the records,
//!   the plan,
//!   the request and the clone's identity,
//!   each read and refused.
//! - `the_proof.rs`:
//!   which slot,
//!   which clone,
//!   which records,
//!   proved in the design's own order.
//! - `the_removal_claim.rs`:
//!   the per-clone removal claim,
//!   taken through the repository's own generation claim.
//! - `the_reversing_request.rs`:
//!   one `mount` request under a fresh nonce and the answer it must carry.
//! - `the_slot_mount.rs`:
//!   where the clone stands,
//!   read from the caller's mount table.
//! - `the_staged_state.rs`:
//!   the staging folder,
//!   its record and its claim settled on the answer.
//! - `the_cancelling_sequence.rs`:
//!   the order the two actors share.
//! - `the_cancel_command.rs`:
//!   the command itself,
//!   its report and its exit codes.
//! - `the_cancelled_removal.rs`:
//!   what a removal restores when it cancels itself,
//!   and what a forced removal's failure leaves.
//! - `the_self_cancelling_answer.rs`:
//!   the answer a removal that a holder stopped asks for,
//!   over this folder's own reversal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = cancelTheRemoval({ checkout, commonDir, home, request, reads, mounts, claim, mountRequest, staged });
//! console.error(outcome.reportLines().join('\n'));
//! ```

/// The command's grammar,
/// and every caller mistake it refuses.
pub mod cancel_arguments;

/// Every state that prevents the command,
/// with its exit code and its sentence.
pub mod cancel_refusal;

/// The per-clone removal claim one account takes for the whole removal.
pub mod the_removal_claim;

/// The plan's tuples settled by the live inventory.
pub mod the_destroy_outcomes;

/// Which slot,
/// which clone and which records the command proves.
pub mod the_proof;

/// One reversing `mount` request and the answer it must carry.
pub mod the_reversing_request;

/// The staging folder's settlement on that answer.
pub mod the_staged_state;

/// Where the clone stands,
/// read from the caller's mount table.
pub mod the_slot_mount;

/// The one plan action a cancellation takes.
pub mod the_plan_it_leaves;

/// The order the cancelling removal and the command share.
pub mod the_cancelling_sequence;

/// The guarded command,
/// its report and its exit codes.
pub mod the_cancel_command;

/// What a removal restores when it cancels itself.
pub mod the_cancelled_removal;

/// The answer a removal that a holder stopped asks for.
pub mod the_self_cancelling_answer;

/// The guest arms of the cancel command.
/// Compiled for tests only and run in the throwaway guest.
#[cfg(test)]
#[path = "the_cancel_command_guest_arm_tests.rs"]
mod the_cancel_command_guest_arm_tests;

/// The stand-ins the controls answer the seams with.
/// Never part of the release executable.
#[cfg(test)]
#[path = "cancel_stand_ins_tests.rs"]
pub(crate) mod cancel_stand_ins;
