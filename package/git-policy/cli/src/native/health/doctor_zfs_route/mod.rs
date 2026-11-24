//! What:
//!  The checks plan step 18 of the ZFS route adds to
//!  `git monochromatic-git-policy-cli doctor`:
//!  the device,
//!  ABI and dataset facts,
//!  the checkout mount and its propagation,
//!  the recorded parent and slot identities,
//!  the slot identity store's folder listing,
//!  the removal of the two staging forms a killed writer left,
//!  the effective delegation,
//!  the handler's installation and its folders,
//!  ZED and service wiring,
//!  the latest readiness and unresolved requests,
//!  the staging folder and its bind's propagation,
//!  the holders of a clone,
//!  and the checks later steps still owe.
//! Why:
//!  The step's checks are read-only apart from the two staging forms it
//!  removes,
//!  and each answers one of three things:
//!  a pass,
//!  a finding naming the failing field or path component,
//!  or unknown with the reason it could not read or prove the fact
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 18,
//!  "What it builds";
//!  `package/git-policy/cli/SPEC.md`,
//!  "### Doctor").
//!  Every check takes its facts as a value,
//!  so the wording and the decision are tested on built facts and a control
//!  never opens this machine's `/dev/zfs`, store or receipts.
//!  The gathering half reads the live system once and is what the wiring run
//!  calls;
//!  nothing in this folder is reached from the wrapper's command entry,
//!  the handler or a boot unit until step 18 is wired,
//!  which is the project's answer of 2026-11-23
//!  (`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`,
//!  "Building later pieces ahead of the plan's order").
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `check_outcome.rs`:
//!   what one check answered,
//!   the checks' stable names and the line each report prints.
//! - `zfs_checks.rs`:
//!   the device,
//!   the ABI verdict,
//!   the pool,
//!   the checkout and container datasets,
//!   their keys and the clones' properties.
//! - `mount_checks.rs`:
//!   the checkout's mount,
//!   its propagation,
//!   the caller's visibility and the nested mounts' path reachability.
//! - `store_listing.rs`:
//!   the store folder's entry count,
//!   a misfiled record,
//!   a foreign entry,
//!   the two known staging forms with their count and age,
//!   and a dot entry that matches neither form.
//! - `store_staging_removal.rs`:
//!   the removal of the two known staging forms,
//!   which only `doctor` calls.
//! - `identity_checks.rs`:
//!   the recorded parent and slot identities against the live folders,
//!   naming the failing field or path component.
//! - `delegation_checks.rs`:
//!   the effective delegation including ancestor grants,
//!   and the absence of the lineage-changing grants.
//! - `handler_checks.rs`:
//!   the handler's installation,
//!   configuration,
//!   root-owned folders,
//!   ZED and service wiring,
//!   the latest readiness and the unresolved requests.
//! - `staging_folder_checks.rs`:
//!   the staging folder under the account's cache and the propagation of its
//!   live bind.
//! - `holder_checks.rs`:
//!   the namespaces that hold a clone,
//!   from the all-namespace scan's result.
//! - `later_step_checks.rs`:
//!   every check a later step builds,
//!   answered unknown and naming that step.
//! - `route_checks.rs`:
//!   one list of every check's answer for the report the wiring run prints.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reports = routeChecks(facts); // [{ check: 'checkout-mount', outcome: { kind: 'pass', detail } }]
//! ```

/// What one check answered,
/// the checks' stable names and the printed line.
pub mod check_outcome;

/// The device,
/// ABI,
/// pool,
/// dataset,
/// key and property checks.
pub mod zfs_checks;

/// The checkout's mount,
/// its propagation and the nested mounts' reachability.
pub mod mount_checks;

/// The slot identity store's folder listing.
pub mod store_listing;

/// The removal of the two known staging forms.
pub mod store_staging_removal;

/// The recorded parent and slot identities against the live folders.
pub mod identity_checks;

/// The effective delegation and the lineage-changing grants' absence.
pub mod delegation_checks;

/// The handler's installation,
/// folders,
/// wiring,
/// readiness and unresolved requests.
pub mod handler_checks;

/// The staging folder and its live bind's propagation.
pub mod staging_folder_checks;

/// The namespaces that hold a clone.
pub mod holder_checks;

/// The checks later steps still owe.
pub mod later_step_checks;

/// Every check's answer in one list.
pub mod route_checks;

/// The facts a control builds for every check in this folder,
/// never part of the release executable.
#[cfg(test)]
#[path = "route_test_facts.rs"]
pub(crate) mod route_test_facts;

/// What one check answered.
pub use check_outcome::{Check, CheckOutcome, CheckReport, finding, pass, unknown};
/// The device,
/// ABI,
/// pool,
/// dataset,
/// key and property checks and what they read.
pub use zfs_checks::{
    AbiFacts, DatasetFacts, DatasetRole, DeviceFacts, DeviceState, PoolFacts, Reading, check_abi, check_clone_properties, check_dataset, check_device, check_key, check_pool, reading_of,
};
/// The checkout mount,
/// its propagation and the nested mounts' reachability.
pub use mount_checks::{MountCheckFacts, NestedMount, check_checkout_mount, check_checkout_propagation, check_nested_reachability, nested_mounts};
/// The store's folder listing.
pub use store_listing::{
    ListedEntry, RecordFacts, SlotRecordFacts, StoreListingFacts, check_staging_forms, check_store_entries, check_unknown_dot_entries, is_foreign_entry, is_unknown_dot_entry, known_staging_entries,
};
/// The removal of the two known staging forms.
pub use store_staging_removal::{StagingRemoval, StagingRemovalRefusal, StagingRemovalRequest, remove_known_staging_forms};
/// The recorded parent and slot identities.
pub use identity_checks::{LiveIdentity, ParentIdentityFacts, RecordedIdentity, SlotIdentityFacts, check_parent_identity, check_slot_identity, walk_refusal_is_unproved};
/// The effective delegation.
pub use delegation_checks::{DelegationFacts, FORBIDDEN_PERMISSIONS, Grant, GrantPlace, Inheritance, REQUIRED_GRANTS, RequiredGrant, Who, allow_command, check_delegation, grants_of, missing_grants, place_dataset};
/// The handler's installation and folders.
pub use handler_checks::{
    FolderFacts, HandlerConfigurationFacts, HandlerInstallationFacts, HandlerWiringFacts, PLACEHOLDER_EXECUTABLE_PATH, PLACEHOLDER_ZEDLET_PATH, PathFacts, PathState, REQUIRED_ROOT_MODES, ReadinessFacts, RequestFacts,
    check_handler_configuration, check_handler_folders, check_handler_installation, check_modes, check_readiness, check_root_modes, check_unresolved_requests, check_zed_wiring,
};
/// The staging folder and its bind.
pub use staging_folder_checks::{StagedFolder, StagingBind, StagingFolderFacts, check_staging_bind_propagation, check_staging_folder, is_derived_folder_name, staging_binds};
/// The namespaces that hold a clone.
pub use holder_checks::{HolderFacts, NamespaceHold, check_holders};
/// The checks later steps still owe.
pub use later_step_checks::{LaterStep, check_later_step};
/// Every check's answer in one list.
pub use route_checks::{RouteFacts, route_checks};
