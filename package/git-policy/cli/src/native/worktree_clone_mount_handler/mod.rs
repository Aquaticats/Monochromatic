//! What:
//!  The root modes of the one wrapper executable:
//!  the words `monochromatic-git-policy-cli zfs-root <mode>`,
//!  the name the ZED history event selects,
//!  the paths one root-owned build serves,
//!  the modes whose plan steps have not landed,
//!  the administrator's parent adoption,
//!  the checkout record and the nested dataset the route places under it,
//!  and the refusal both root modes answer a caller that is not root with.
//! Why:
//!  The owner answered "One program only, ever", so the release holds one
//!  executable and the root modes are modes of the wrapper
//!  (`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`).
//!  A root-owned program the ZED event and the boot and shutdown units start
//!  owns attachment,
//!  unmount and the recorded folder identities,
//!  and it shares the binding and the compatibility gate with the wrapper
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Binding crate and compatibility",
//!  "ZED reconciliation and mount safety";
//!  the build order is `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 5,
//!  and how the modes are reached is
//!  `doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`).
//!  Each mode refuses until the plan step that builds it lands,
//!  except the modes whose steps are in:
//!  the default sweep,
//!  `boot-reconcile`,
//!  `start-probe`,
//!  `holder-scan` and the administrator's adoption,
//!  whose decision table is step 4's and whose end-to-end arm is step 5's.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `root_mode_names.rs`:
//!   the one start name and the group word,
//!   and the decision each start is read as.
//! - `handler_install_paths.rs`:
//!   the paths one root-owned build serves,
//!   the checkout record's path,
//!   the words and the version line.
//! - `handler_mode_arguments.rs`:
//!   the mode words,
//!   the adoption's three options,
//!   the replacement a record admits,
//!   the refusals and the usage.
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
//! - `holder_scan_mode.rs`:
//!   the all-namespace holder scan of one clone,
//!   which step 9 builds.
//! - `adopt_worktree_parent_mode.rs`:
//!   the administrator's adoption,
//!   decided by step 4's table and recorded here.
//! - `record_checkout_mode.rs`:
//!   the checkout's path, dataset, GUID, container and pool,
//!   recorded under root control,
//!   and the GUID a legitimate restore replaces.
//! - `mount_path_descendant_mode.rs`:
//!   one nested dataset mounted at its own mountpoint beneath the recorded
//!   checkout.
//! - `per_slot_root_claim.rs`:
//!   the root-owned directory of one claim file per slot,
//!   and the exclusive lock a sweep holds while it acts.
//! - `handler_mode_dispatch.rs`:
//!   the command line,
//!   the exit statuses and the lines each mode writes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! process.exit(worktreeCloneMountHandler.run(words));
//! ```

/// The one start name and the group word,
/// and the decision each start is read as.
pub mod root_mode_names;

/// The paths one root-owned build serves,
/// the checkout record's path,
/// the words and the version line.
pub mod handler_install_paths;

/// The mode words, the adoption's options and the replacement a record admits.
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

/// The pool reads one sweep makes and the parent's child listing.
pub mod sweep_dataset_reader;

/// This boot's first attach,
/// claimed after the key check and written before the receipt.
pub mod sweep_first_attach;

/// One mount's source,
/// asked of the kernel by the mount's unique number.
pub mod sweep_standing_mount_source;

/// Whether a clone's own mount stands at its slot,
/// from one polled read of the kernel's mount table and that one mount's
/// source.
pub mod sweep_standing_mounts;

/// The attach step one clone goes through and its rollback,
/// and the answer an occupied slot gets before the attach.
pub mod sweep_attach_step;

/// The seccomp filter a control installs so the mount-source call answers
/// nothing,
/// and the statement a run makes about its containers' profile,
/// never in the release build.
#[cfg(all(test, target_os = "linux"))]
pub(crate) mod standing_mount_source_call_refusal;

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

/// The all-namespace holder scan of one clone,
/// and the line that counts what it found.
pub mod holder_scan_mode;

/// The administrator's adoption of a replacement parent folder.
pub mod adopt_worktree_parent_mode;

/// The checkout's path, dataset, GUID, container and pool,
/// and the GUID a legitimate restore replaces.
pub mod record_checkout_mode;

/// One nested dataset mounted beneath the recorded checkout.
pub mod mount_path_descendant_mode;

/// The per-slot claim every sweep takes before it acts on a clone.
pub mod per_slot_root_claim;

/// The command line and the statuses.
pub mod handler_mode_dispatch;

/// The one start name's decisions,
/// which read the module the pin holds line for line,
/// so the module itself carries no test declaration.
#[cfg(all(test, target_os = "linux"))]
#[path = "root_mode_names_tests.rs"]
mod root_mode_names_tests;

/// The paths one root-owned build serves,
/// the checkout record's path,
/// the words and the version line.
pub use handler_install_paths::{CHECKOUT_RECORD_PATH, CONFIGURATION_PATH, ROOT_MODE_INSTALL_PATH, ZEDLET_INSTALL_PATH, root_modes_name, version_line};
/// The mode grammar,
/// its refusal and the requests the modes take.
pub use handler_mode_arguments::{AdoptRequest, ArgumentRefusal, HandlerMode, HolderScanInvocation, MountPathDescendantRequest, RecordCheckoutRequest};
/// The refusal of a mode whose step has not landed.
pub use mode_awaiting_its_step::AwaitingItsStep;
/// The administrator's adoption and why it recorded nothing.
pub use adopt_worktree_parent_mode::AdoptFailure;
/// The per-slot claim and its refusal.
pub use per_slot_root_claim::{ClaimRefusal, HeldSlotClaim, SLOT_CLAIM_DIRECTORY, SlotClaimDirectory};
/// The command line, the two statuses and the refusal a root-mode misuse answers.
pub use handler_mode_dispatch::{EXIT_DONE, EXIT_REFUSED, refuse_root_mode_misuse, run};
