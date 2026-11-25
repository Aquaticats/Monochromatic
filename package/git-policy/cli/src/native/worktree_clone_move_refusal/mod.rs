//! What:
//!  The refusal of `git worktree move` for a slot a managed clone is bound to,
//!  mounted or not.
//! Why:
//!  Plan step 12 builds the refusal as a function the wrapper's `worktree move`
//!  handling will call
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 12;
//!  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees").
//!  The kernel's own `Device or resource busy` covers only a mounted clone,
//!  so an unmounted slot needs the wrapper's refusal,
//!  and the guest arm shows both arms on a real pool.
//!  The decision lives in `managed_move_refusal.rs`;
//!  this folder is separate from `worktree_clone_slot_state` because the wiring
//!  step calls it from the command path,
//!  which reads no report,
//!  and because the refusal reads only the binding record.
//!  Every forwarded `git worktree move` reaches it
//!  (`invocation/entry_zfs_slot_commands.rs`, `move_refusal_of`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { refuseMoveOfABoundSlot } from '@monochromatic-dev/git-policy-cli/worktree-clone-move-refusal';
//! ```

pub mod managed_move_refusal;

pub use managed_move_refusal::{ManagedMoveRefusal, ManagedMoveRefusalCause, refuse_move_of_a_bound_slot};
