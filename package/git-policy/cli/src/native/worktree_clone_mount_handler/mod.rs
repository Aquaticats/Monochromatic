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
//!  The owner answered "One program only,
//! ever",
//! so the release holds one
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
//! - `mode_awaiting_its_step.rs`:
//!   the refusal of a mode whose plan step has not landed.
//! - `per_slot_root_claim.rs`:
//!   the root-owned directory of one claim file per slot,
//!   and the exclusive lock a sweep holds while it acts.
//! - `root_mode_names.rs`:
//!   the one start name and the group word,
//!   and the decision each start is read as.
//! - `handler_install_paths.rs`:
//!   the paths one root-owned build serves,
//!   the checkout record's path,
//!   the words and the version line.
//! - `handler_mode_words.rs`:
//!   the mode words,
//!   the options each mode takes and the usage.
//! - `handler_mode_argument_refusals.rs`:
//!   why one invocation is not a mode,
//!   and the words each cause answers.
//! - `handler_mode_arguments_requests.rs`:
//!   what one invocation asks for,
//!   and the identities each mode that takes an argument is handed.
//! - `handler_mode_arguments_option_reads.rs`:
//!   each mode's options read,
//!   one mode at a time.
//! - `handler_mode_arguments.rs`:
//!   the mode word's read,
//!   and the words each mode takes.
//! - `root_mode_caller.rs`:
//!   the refusal a root mode answers a caller that is not root with.
//! - `adopt_worktree_parent_mode.rs`:
//!   the administrator's adoption,
//!   decided by step 4's table and recorded here.
//! - `shutdown_unmount_mode.rs`:
//!   the shutdown unit's verified unmount,
//!   which step 9 builds.
//! - `sweep_checkout_mount.rs`:
//!   the checkout's own mount,
//!   its dataset and its propagation,
//!   read from the kernel's table.
//! - `sweep_configuration_refusal.rs`:
//!   why the sweep has no names to act on,
//!   one cause per finding.
//! - `sweep_configuration.rs`:
//!   the names the sweep takes from the configuration and the mount table.
//! - `sweep_configuration_refusal_words.rs`:
//!   the words a person reads for each cause,
//!   and the entries a configuration file can carry.
//! - `sweep_dataset_facts.rs`:
//!   what one dataset the sweep reached says about itself.
//! - `sweep_clone_property_values.rs`:
//!   the values the design gives a clone's properties and its origin snapshot.
//! - `sweep_clone_refusals.rs`:
//!   why one child of the pinned parent is not a clone this sweep attaches,
//!   and the words each cause answers.
//! - `sweep_clone_check.rs`:
//!   the checks one child of the pinned parent passes before it is attached.
//! - `sweep_dataset_reader.rs`:
//!   the pool reads one sweep makes and the parent's child listing.
//! - `holder_scan_mode.rs`:
//!   the all-namespace holder scan of one clone,
//!   which step 9 builds.
//! - `record_checkout_mode_record.rs`:
//!   what the checkout record holds,
//!   the field a later run finds different,
//!   and the refusal each cause answers.
//! - `record_checkout_mode_record_file.rs`:
//!   the record as the file it is,
//!   its folder's proofs and the three lines an answer prints.
//! - `record_checkout_mode_places.rs`:
//!   where one run reads and writes,
//!   with the installed places by default.
//! - `record_checkout_mode_paths_and_datasets.rs`:
//!   what a checkout's path and a dataset's name have to be.
//! - `record_checkout_mode_record_comparison.rs`:
//!   the record the mode compares and writes,
//!   and the write a `--replace` token admits.
//! - `record_checkout_mode.rs`:
//!   the checkout's path,
//!   dataset,
//!   GUID,
//!   container and pool,
//!   recorded under root control,
//!   and the GUID a legitimate restore replaces.
//! - `mount_path_descendant_mode_refusals.rs`:
//!   why the mode mounted nothing,
//!   one variant per cause and the words each answers.
//! - `mount_path_descendant_mode_properties.rs`:
//!   the properties an explicit mount is refused by,
//!   and the attributes a dataset's own properties resolve to.
//! - `mount_path_descendant_mode_plan.rs`:
//!   every proof the explicit mount makes before it acts,
//!   and the two answers a proved state gives.
//! - `mount_path_descendant_mount_placement.rs`:
//!   the walk to the mountpoint through descriptors,
//!   and the mount placed on the folder that walk opened.
//! - `mount_path_descendant_mode.rs`:
//!   the mode's own entry:
//!   where a run reads and the lines it answers.
//! - `sweep_first_attach.rs`:
//!   this boot's first attach,
//!   claimed after the key check and written before the receipt.
//! - `sweep_probe_record.rs`:
//!   this boot's identifier,
//!   the start probe's record for it and the fingerprint it must agree with.
//! - `sweep_preconditions.rs`:
//!   what a run checks before any attach:
//!   the pool and the checkout.
//! - `sweep_standing_mount_source.rs`:
//!   one mount's source,
//!   asked of the kernel by the mount's unique number.
//! - `sweep_standing_mounts_kernel_table.rs`:
//!   the kernel's mount table and the poll that says it changed,
//!   and the seam a control answers with a table it built.
//! - `sweep_standing_mounts.rs`:
//!   whether a clone's own mount stands at its slot,
//!   from one polled read of the kernel's mount table and that one mount's source.
//! - `sweep_attach_step_answers.rs`:
//!   what one attach step is asked,
//!   the answers it gives,
//!   and the step and errno each refusal is written as.
//! - `sweep_attach_step_calls.rs`:
//!   the rollback's two calls and the one mount call,
//!   each as a seam a control answers.
//! - `sweep_attach_step_rollback.rs`:
//!   the rollback of a mount the attach made that landed beside its slot,
//!   read back before it is reported as taken down.
//! - `sweep_attach_step.rs`:
//!   the attach step one clone goes through,
//!   and the answer an occupied slot gets before the attach.
//! - `sweep_turn_context_and_outcome.rs`:
//!   what one clone's turn reads and writes,
//!   the outcome it answers with,
//!   and the step words a receipt names.
//! - `sweep_one_clone_answers.rs`:
//!   what one clone's turn files:
//!   a standing mount's `done` receipt,
//!   this boot's `first-attach` record,
//!   and the one receipt a request is answered with.
//! - `sweep_unmount_answers.rs`:
//!   the unmount arm's GUID read and refusal answer.
//! - `sweep_attach_answer.rs`:
//!   what one attach step's outcome files and answers.
//! - `sweep_refused_clone_answers.rs`:
//!   the answer to a clone the checks refused,
//!   and the receipt an unavailable key alone files.
//! - `sweep_unmount_under_the_claim.rs`:
//!   the unmount arm of one clone's turn,
//!   once its slot is claimed.
//! - `sweep_intent_read_again.rs`:
//!   the clone's identity and intent read again under the claim,
//!   and whether the intent changed after the request a turn acted on.
//! - `sweep_slot_first_observation.rs`:
//!   the slot's first observation:
//!   its record,
//!   its walk and its Git pointer.
//! - `sweep_one_clone.rs`:
//!   one clone's whole turn of the sweep.
//! - `sweep_run.rs`:
//!   one whole sweep run and the lines it reports.
//! - `attaching_sweep_mode.rs`:
//!   the default invocation's sweep,
//!   which step 8 builds.
//! - `boot_reconcile_mode.rs`:
//!   the boot unit's reconciliation,
//!   which step 6 begins.
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

/// The mode words,
/// the options each mode takes and the usage text.
pub mod handler_mode_words;

/// Why one invocation is not a mode,
/// and the words each cause answers.
pub mod handler_mode_argument_refusals;

/// What one invocation asks for,
/// and the identities each mode that takes an argument is handed.
pub mod handler_mode_arguments_requests;

/// Each mode's options read,
/// one mode at a time.
pub mod handler_mode_arguments_option_reads;

/// The mode word's read,
/// and the words each mode takes.
pub mod handler_mode_arguments;

/// The refusal of a mode whose step has not landed.
pub mod mode_awaiting_its_step;

/// The refusal a root mode answers a caller that is not root with.
pub mod root_mode_caller;

/// The unmount arm's GUID read and refusal answer.
mod sweep_unmount_answers;

/// What one clone's turn reads and writes,
/// the outcome it answers with,
/// and the step words a receipt names.
pub mod sweep_turn_context_and_outcome;

/// The unmount arm of one clone's turn,
/// once its slot is claimed.
pub mod sweep_unmount_under_the_claim;

/// What one clone's turn files.
pub mod sweep_one_clone_answers;

/// What one attach step's outcome files and answers.
pub mod sweep_attach_answer;

/// The answer to a clone the checks refused.
pub mod sweep_refused_clone_answers;

/// The clone's identity and intent read again under the slot's claim.
pub mod sweep_intent_read_again;

/// The slot's first observation:
/// its record,
/// its walk and its Git pointer.
pub mod sweep_slot_first_observation;

/// The default invocation:
/// the clone-mount sweep.
pub mod attaching_sweep_mode;

/// The checkout's own mount,
/// its dataset and its propagation.
pub mod sweep_checkout_mount;

/// Why the sweep has no names to act on,
/// one cause per finding.
pub mod sweep_configuration_refusal;

/// The names the sweep takes from the configuration and the mount table.
pub mod sweep_configuration;

/// The words a person reads for each cause,
/// and the entries a configuration file can carry.
pub mod sweep_configuration_refusal_words;

/// What one dataset the sweep reached says about itself.
pub mod sweep_dataset_facts;

/// The checks one child of the pinned parent passes before it is attached.
pub mod sweep_clone_check;

/// The values the design gives a clone's properties and its origin snapshot.
pub mod sweep_clone_property_values;

/// Why one child of the pinned parent is not a clone this sweep attaches.
pub mod sweep_clone_refusals;

/// The pool reads one sweep makes and the parent's child listing.
pub mod sweep_dataset_reader;

/// This boot's first attach,
/// claimed after the key check and written before the receipt.
pub mod sweep_first_attach;

/// One mount's source,
/// asked of the kernel by the mount's unique number.
pub mod sweep_standing_mount_source;

/// The kernel's mount table and the poll that says it changed,
/// and the seam a control answers with a table it built.
pub mod sweep_standing_mounts_kernel_table;

/// Whether a clone's own mount stands at its slot,
/// from one polled read of the kernel's mount table and that one mount's
/// source.
pub mod sweep_standing_mounts;

/// What one attach step is asked and the answers it gives,
/// with the step and errno each refusal is written as.
pub mod sweep_attach_step_answers;

/// The rollback's two calls and the one mount call,
/// each as a seam a control answers.
pub mod sweep_attach_step_calls;

/// The rollback of a mount the attach made that landed beside its slot.
pub mod sweep_attach_step_rollback;

/// The attach step one clone goes through,
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

/// One whole sweep run and the lines it reports.
pub mod sweep_run;

/// The start probe's record for this boot and the words it answers with.
pub mod sweep_probe_record;

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

/// What the checkout record holds,
/// the field a later run finds different and why the mode recorded nothing.
pub mod record_checkout_mode_record;

/// The record as the file it is:
/// its read,
/// its write and its folder's proofs.
pub mod record_checkout_mode_record_file;

/// Where one run reads and writes,
/// with the installed places by default.
pub mod record_checkout_mode_places;

/// What a checkout's path and a dataset's name have to be.
pub mod record_checkout_mode_paths_and_datasets;

/// The record the mode compares and writes,
/// and the write a `--replace` token admits.
pub mod record_checkout_mode_record_comparison;

/// The checkout's path,
/// dataset,
/// GUID,
/// container and pool,
/// and the GUID a legitimate restore replaces.
pub mod record_checkout_mode;

/// Why the `mount-path-descendant` mode mounted nothing,
/// one variant per cause and the words each answers.
pub mod mount_path_descendant_mode_refusals;

/// The properties an explicit mount is refused by,
/// and the attributes a dataset's own properties resolve to.
pub mod mount_path_descendant_mode_properties;

/// Every proof the explicit mount makes before it acts.
pub mod mount_path_descendant_mode_plan;

/// The walk to the mountpoint and the mount placed on it.
pub mod mount_path_descendant_mount_placement;

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

/// The administrator's adoption and why it recorded nothing.
pub use adopt_worktree_parent_mode::AdoptFailure;
/// The paths one root-owned build serves,
/// the checkout record's path,
/// the words and the version line.
pub use handler_install_paths::{
    CHECKOUT_RECORD_PATH,
    CONFIGURATION_PATH,
    ROOT_MODE_INSTALL_PATH,
    ZEDLET_INSTALL_PATH,
    root_modes_name,
    version_line,
};
/// The mode grammar,
/// its refusal and the requests the modes take.
pub use handler_mode_arguments::{
    AdoptRequest,
    ArgumentRefusal,
    HandlerMode,
    HolderScanInvocation,
    MountPathDescendantRequest,
    RecordCheckoutRequest,
};
/// The command line,
/// the two statuses and the refusal a root-mode misuse answers.
pub use handler_mode_dispatch::{
    EXIT_DONE,
    EXIT_REFUSED,
    refuse_root_mode_misuse,
    run,
};
/// The refusal of a mode whose step has not landed.
pub use mode_awaiting_its_step::AwaitingItsStep;
/// The per-slot claim and its refusal.
pub use per_slot_root_claim::{
    ClaimRefusal,
    HeldSlotClaim,
    SLOT_CLAIM_DIRECTORY,
    SlotClaimDirectory,
};
