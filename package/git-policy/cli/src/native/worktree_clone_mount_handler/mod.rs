//! What:
//!  The root-run clone-mount handler of the ZFS worktree route:
//!  the second executable `mount-git-worktree-clone-datasets`,
//!  its mode grammar,
//!  the two paths it is installed at,
//!  the modes whose plan steps have not landed,
//!  the administrator's parent adoption,
//!  and the refusal both root modes answer a caller that is not root with.
//! Why:
//!  The design keeps the clone mount out of the Git wrapper:
//!  a root-installed program the ZED event and the boot and shutdown units start
//!  owns attachment,
//!  unmount and the recorded folder identities,
//!  and it shares the binding and the compatibility gate with the wrapper without
//!  becoming a package of its own
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Binding crate and compatibility",
//!  "ZED reconciliation and mount safety";
//!  the build order is `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 5).
//!  Each mode refuses until the plan step that builds it lands,
//!  except the modes whose steps are in:
//!  the default sweep,
//!  `boot-reconcile`,
//!  `start-probe` and the administrator's adoption,
//!  whose decision table is step 4's and whose end-to-end arm is step 5's.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `handler_install_paths.rs`:
//!   the two installed paths,
//!   the configuration's path and the program's own name.
//! - `handler_mode_arguments.rs`:
//!   the mode words,
//!   the adoption's three options,
//!   the refusals and the usage and version lines.
//! - `mode_awaiting_its_step.rs`:
//!   the refusal of a mode whose plan step has not landed.
//! - `root_mode_caller.rs`:
//!   the refusal a root mode answers a caller that is not root with.
//! - `sweep_checkout_mount.rs`:
//!   the checkout's own mount,
//!   its dataset and its propagation,
//!   read from the kernel's table.
//! - `sweep_configuration_refusal.rs`:
//!   why the sweep has no names to act on,
//!   and the words a person reads for each cause.
//! - `attaching_sweep_mode.rs`:
//!   the default invocation's sweep,
//!   which step 8 builds.
//! - `boot_reconcile_mode.rs`:
//!   the boot unit's reconciliation,
//!   which step 6 begins.
//! - `shutdown_unmount_mode.rs`:
//!   the shutdown unit's verified unmount,
//!   which step 9 builds.
//! - `adopt_worktree_parent_mode.rs`:
//!   the administrator's adoption,
//!   decided by step 4's table and recorded here.
//! - `per_slot_root_claim.rs`:
//!   the root-owned directory of one claim file per slot,
//!   and the exclusive lock a sweep holds while it acts.
//! - `handler_mode_dispatch.rs`:
//!   the command line,
//!   the exit statuses and the lines each mode writes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! process.exit(worktreeCloneMountHandler.run(process.argv.slice(2)));
//! ```

/// The two installed paths,
/// the configuration beside them and the program's own name.
pub mod handler_install_paths;

/// The mode words and the adoption's options.
pub mod handler_mode_arguments;

/// The refusal of a mode whose step has not landed.
pub mod mode_awaiting_its_step;

/// The refusal a root mode answers a caller that is not root with.
pub mod root_mode_caller;

/// The unmount arm's GUID read and refusal answer.
mod sweep_unmount_answers;

/// The default invocation:
/// the clone-mount sweep.
pub mod attaching_sweep_mode;

/// The checkout's own mount,
/// its dataset and its propagation.
pub mod sweep_checkout_mount;

/// The names the sweep takes from the configuration and the mount table.
pub mod sweep_configuration;

/// Why the sweep has no names to act on,
/// and the words for each cause.
pub mod sweep_configuration_refusal;

/// What one dataset the sweep reached says about itself.
pub mod sweep_dataset_facts;

/// The checks one child of the pinned parent passes before it is attached.
pub mod sweep_clone_check;

/// The attach step one clone goes through and its rollback.
pub mod sweep_attach_step;

/// The pool reads one sweep makes and the parent's child listing.
pub mod sweep_dataset_reader;

/// This boot's first attach,
/// claimed after the key check and written before the receipt.
pub mod sweep_first_attach;

/// One clone's whole turn of the sweep.
pub mod sweep_one_clone;

/// What one clone's turn answers and files.
pub mod sweep_one_clone_answers;

/// One whole sweep run and the lines it reports.
pub mod sweep_run;

/// What a run checks before any attach.
pub mod sweep_preconditions;

/// The boot unit's reconciliation.
pub mod boot_reconcile_mode;

/// The shutdown unit's verified unmount.
pub mod shutdown_unmount_mode;

/// The administrator's adoption of a replacement parent folder.
pub mod adopt_worktree_parent_mode;

/// The per-slot claim every sweep takes before it acts on a clone.
pub mod per_slot_root_claim;

/// The command line and the statuses.
pub mod handler_mode_dispatch;

/// The installed paths and the program's name.
pub use handler_install_paths::{CONFIGURATION_PATH, PROGRAM_NAME, SERVICE_INSTALL_PATH, ZEDLET_INSTALL_PATH};
/// The mode grammar,
/// its refusal and the request an adoption takes.
pub use handler_mode_arguments::{AdoptRequest, ArgumentRefusal, HandlerMode};
/// The refusal of a mode whose step has not landed.
pub use mode_awaiting_its_step::AwaitingItsStep;
/// The administrator's adoption and why it recorded nothing.
pub use adopt_worktree_parent_mode::AdoptFailure;
/// The per-slot claim and its refusal.
pub use per_slot_root_claim::{ClaimRefusal, HeldSlotClaim, SLOT_CLAIM_DIRECTORY, SlotClaimDirectory};
/// The command line and the two statuses.
pub use handler_mode_dispatch::{EXIT_DONE, EXIT_REFUSED, run};
