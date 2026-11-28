//! What:
//!  The checks plan step 18 of the ZFS route adds to
//!  `git monochromatic-git-policy-cli doctor`:
//!  the device,
//!  ABI and dataset facts,
//!  the checkout mount and its propagation,
//!  the snapshot directories' visibility on the checkout dataset and the clone
//!  container,
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
//!  never opens this machine's `/dev/zfs`,
//! store or receipts.
//!  The facts themselves are the wiring run's to gather once from the live
//!  system,
//!  as the add route's preflight gathers the same delegation through
//!  `grants_of` and as the guest arm's helpers gather the rest.
//!  `doctor`'s own section reaches none of them today:
//!  the one caller outside this folder is the add route's own delegation check,
//!  which takes `check_delegation` from here
//!  (`worktree_clone_add_route::add_grants`),
//!  and every other check is reached by its tests and its guest arms alone
//!  until step 18 is wired,
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
//! - `zfs_checks_limits.rs`:
//!   the numbers and words the kernel and the design fix.
//! - `zfs_checks_reading.rs`:
//!   what a read of one dataset answered,
//!   and the rule that reads a failed read.
//! - `zfs_checks_machine_prerequisites.rs`:
//!   the device,
//!   the ABI verdict and the pool.
//! - `zfs_checks_dataset_roles.rs`:
//!   a dataset's role and the facts a read of it answered.
//! - `zfs_checks_dataset_properties.rs`:
//!   one dataset against the properties its role requires.
//! - `zfs_checks_key_and_clones.rs`:
//!   the encryption key of each dataset,
//!   and every clone's attach properties.
//! - `zfs_checks.rs`:
//!   the device,
//!   the ABI verdict,
//!   the pool,
//!   the checkout and container datasets,
//!   their keys and the clones' properties,
//!   one report per dataset.
//! - `snapshot_directory_checks.rs`:
//!   the `snapdir` of the checkout dataset and of the clone container,
//!   and the command that sets it where it is not `disabled`.
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
//! - `delegation_reply.rs`:
//!   the kernel's delegation reply and the grant model it becomes:
//!   who a grant names,
//!   how widely it reaches,
//!   and the dataset that carries it.
//! - `delegation_checks_permissions.rs`:
//!   the permissions the route's account needs,
//!   where the design grants each of them,
//!   and the permissions the route must not have.
//! - `delegation_checks_grant_commands.rs`:
//!   how a required permission was met,
//!   and the `zfs allow` and `zfs unallow` commands that move a grant.
//! - `delegation_checks_narrowing.rs`:
//!   the wider grants a person narrows,
//!   and the lines that name them.
//! - `delegation_checks_forbidden_grants.rs`:
//!   the lineage-changing grants the account holds or would come to hold.
//! - `delegation_checks.rs`:
//!   the required and forbidden permissions,
//!   the effective delegation including ancestor grants,
//!   and the commands that settle each finding,
//!   judged here.
//! - `handler_checks_paths_and_modes.rs`:
//!   the paths,
//!   owners and modes every handler check compares against,
//!   and what one look at a path answered.
//! - `handler_checks_installation.rs`:
//!   the installed executable and zedlet,
//!   and the root modes the one executable must answer.
//! - `handler_checks_configuration.rs`:
//!   the handler's configuration file:
//!   its owner,
//!   its mode and whether its text parsed.
//! - `handler_checks_folder_modes.rs`:
//!   the readiness and results folders against root ownership and the modes
//!   the design fixes.
//! - `handler_checks_zed_wiring.rs`:
//!   ZED's unit,
//!   its enablement and the other zedlets an enablement runs.
//! - `handler_checks_readiness.rs`:
//!   the latest readiness record and whether it admits an attach.
//! - `handler_checks_unresolved_requests.rs`:
//!   the clone requests no receipt of this boot answers.
//! - `handler_checks.rs`:
//!   the handler's installation,
//!   configuration,
//!   root-owned folders,
//!   ZED and service wiring,
//!   the latest readiness and the unresolved requests,
//!   named here.
//! - `staging_folder_checks.rs`:
//!   the staging folder under the account's cache and the propagation of its
//!   live bind.
//! - `holder_checks.rs`:
//!   the namespaces that hold a clone,
//!   from the all-namespace scan's result.
//! - `clone_state_checks.rs`:
//!   what one managed slot is:
//!   the clone mounted at it,
//!   unmounted from it,
//!   staged under the cache,
//!   and the add or removal that stopped part-way.
//! - `registration_checks.rs`:
//!   the Git registration one slot's record names,
//!   complete,
//!   pruned or pointing elsewhere.
//! - `managed_lock_checks.rs`:
//!   the sentence one slot's managed Git lock prints in each state,
//!   and the missing pointer's sentence beside it.
//! - `binding_record_checks.rs`:
//!   the fields of the binding record one add wrote.
//! - `removal_plan_checks.rs`:
//!   the destruction plan one removal left and the outcome of each name in it.
//! - `clone_snapshot_checks.rs`:
//!   the clone's own snapshots beside the plan's states.
//! - `clone_origin_checks.rs`:
//!   the origin each clone names and its deferred-destroy mark.
//! - `clone_retention_checks.rs`:
//!   what one clone and its snapshots retain,
//!   in named accounting fields.
//! - `blocked_removal_checks.rs`:
//!   the blocked-removal state:
//!   the standing intent,
//!   the snapshots already destroyed,
//!   the holders,
//!   and what may end them.
//! - `later_step_checks.rs`:
//!   every check a later step builds,
//!   answered unknown and naming that step.
//! - `route_checks.rs`:
//!   one list of every check's answer for the report the wiring run prints.
//! - `clone_facts_reads.rs`:
//!   every read one managed clone's lines take,
//!   as one seam a control scripts.
//! - `clone_facts_gathering_holders.rs`:
//!   the namespaces that hold one clone and its reader's own mounts.
//! - `clone_facts_gathering_records.rs`:
//!   the registration one record names and the origin with its mark.
//! - `clone_facts_gathering_retention.rs`:
//!   what one clone and its own snapshots retain.
//! - `clone_facts_gathering.rs`:
//!   one managed clone's fact set,
//!   gathered from one request and one set of reads.
//! - `clone_facts_live_pool.rs`:
//!   the two shapes the gatherer's pool reads take of the one gated device.
//! - `clone_facts_live_answers.rs`:
//!   what stands at a path,
//!   and which tasks the caller's own session keeps.
//! - `clone_facts_live_reads.rs`:
//!   the release answers to that seam,
//!   from this machine's own readers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reports = routeChecks(facts); // [{ check: 'checkout-mount', outcome: { kind: 'pass', detail } }]
//! ```

/// What one check answered,
/// the checks' stable names and the printed line.
pub mod check_outcome;

/// The numbers and words the kernel and the design fix.
pub mod zfs_checks_limits;

/// What a read of one dataset answered.
pub mod zfs_checks_reading;

/// The device,
/// the ABI verdict and the pool.
pub mod zfs_checks_machine_prerequisites;

/// A dataset's role and the facts a read of it answered.
pub mod zfs_checks_dataset_roles;

/// One dataset against the properties its role requires.
pub mod zfs_checks_dataset_properties;

/// The encryption key of each dataset and every clone's properties.
pub mod zfs_checks_key_and_clones;

/// The device,
/// ABI,
/// pool,
/// dataset,
/// key and property checks,
/// one report per dataset.
pub mod zfs_checks;

/// The `snapdir` of the checkout dataset and of the clone container.
pub mod snapshot_directory_checks;

/// The checkout's mount,
/// its propagation and the nested mounts' reachability.
pub mod mount_checks;

/// The slot identity store's folder listing.
pub mod store_listing;

/// The removal of the two known staging forms.
pub mod store_staging_removal;

/// The recorded parent and slot identities against the live folders.
pub mod identity_checks;

/// The kernel's delegation reply and the grant model it becomes.
pub mod delegation_reply;

/// The permissions the design requires and forbids.
pub mod delegation_checks_permissions;

/// How a required permission was met,
/// and the commands that move a grant.
pub mod delegation_checks_grant_commands;

/// The wider grants a person narrows.
pub mod delegation_checks_narrowing;

/// The lineage-changing grants the account holds or would come to hold.
pub mod delegation_checks_forbidden_grants;

/// The effective delegation and the lineage-changing grants' absence.
pub mod delegation_checks;

/// What one look at a path answered,
/// and the paths,
/// owners and modes every handler check compares against.
pub mod handler_checks_paths_and_modes;

/// The installed halves and the root modes the one executable must answer.
pub mod handler_checks_installation;

/// The handler's configuration file.
pub mod handler_checks_configuration;

/// The two root-owned folders.
pub mod handler_checks_folder_modes;

/// ZED's unit,
/// its enablement and the other zedlets.
pub mod handler_checks_zed_wiring;

/// The latest readiness record.
pub mod handler_checks_readiness;

/// The clone requests no receipt of this boot answers.
pub mod handler_checks_unresolved_requests;

/// The handler's installation,
/// folders,
/// wiring,
/// readiness and unresolved requests.
pub mod handler_checks;

/// The staging folder and its live bind's propagation.
pub mod staging_folder_checks;

/// The namespaces that hold a clone.
pub mod holder_checks;

/// What one managed slot is.
pub mod clone_state_checks;

/// The Git registration one slot's record names.
pub mod registration_checks;

/// The sentence one slot's managed Git lock prints.
pub mod managed_lock_checks;

/// The binding record one add wrote.
pub mod binding_record_checks;

/// The destruction plan one removal left.
pub mod removal_plan_checks;

/// The clone's own snapshots beside the plan's states.
pub mod clone_snapshot_checks;

/// The origin each clone names and its deferred-destroy mark.
pub mod clone_origin_checks;

/// What one clone and its snapshots retain.
pub mod clone_retention_checks;

/// The blocked-removal state of one clone.
pub mod blocked_removal_checks;

/// The checks later steps still owe.
pub mod later_step_checks;

/// Every check's answer in one list.
pub mod route_checks;

/// Every read one managed clone's lines take,
/// as one seam a control scripts.
pub mod clone_facts_reads;

/// The namespaces that hold one clone.
pub mod clone_facts_gathering_holders;

/// The registration one record names,
/// and the origin with its mark.
pub mod clone_facts_gathering_records;

/// What one clone and its own snapshots retain.
pub mod clone_facts_gathering_retention;

/// One managed clone's fact set,
/// gathered from one request and one set of reads.
pub mod clone_facts_gathering;

/// The two answers this machine gives without a pool.
pub mod clone_facts_live_answers;

/// The two shapes the gatherer's pool reads take of one gated device.
pub mod clone_facts_live_pool;

/// The release answers to that seam.
pub mod clone_facts_live_reads;

/// The facts a control builds for every check in this folder,
/// never part of the release executable:
/// the names,
/// the paths,
/// the datasets,
/// the mount table with the staging folder,
/// the store and its identities,
/// the delegation,
/// the handler,
/// one healthy clone,
/// and the one fact set every control starts from.
#[cfg(test)]
#[path = "route_test_facts.rs"]
pub(crate) mod route_test_facts;

/// The names every healthy fact of this folder is built from.
#[cfg(test)]
mod route_test_facts_names;

/// One path at the state a healthy route shows.
#[cfg(test)]
mod route_test_facts_paths;

/// One healthy dataset of each role.
#[cfg(test)]
mod route_test_facts_datasets;

/// The mount table and the staging folder.
#[cfg(test)]
mod route_test_facts_mounts_and_staging_folder;

/// The store's listing and the identities it records.
#[cfg(test)]
mod route_test_facts_store_and_identities;

/// The decided grants and the delegation they make.
#[cfg(test)]
mod route_test_facts_delegation;

/// The handler's halves,
/// folders,
/// wiring,
/// readiness and requests.
#[cfg(test)]
mod route_test_facts_handler;

/// One managed clone at the design's healthy state.
#[cfg(test)]
mod route_test_facts_clone;

/// The removal-side lines' guest arm,
/// compiled for tests only and run in the throwaway guest.
#[cfg(test)]
#[path = "clone_lines_guest_arm_tests.rs"]
mod clone_lines_guest_arm_tests;

/// The gatherer's controls,
/// and the guest arm that reads six clones of one repository.
#[cfg(test)]
#[path = "clone_facts_gathering_tests.rs"]
mod clone_facts_gathering_tests;

/// The gatherer's guest arm,
/// compiled for tests only and run in the throwaway guest.
#[cfg(test)]
#[path = "clone_facts_guest_arm_tests.rs"]
mod clone_facts_guest_arm_tests;

/// The binding record one add wrote.
pub use binding_record_checks::{
    NOT_RECORDED,
    check_binding_record,
};
/// The blocked-removal state of one clone.
pub use blocked_removal_checks::{
    BlockedRemovalFacts,
    check_blocked_removal,
};
/// What one check answered.
pub use check_outcome::{
    Check,
    CheckOutcome,
    CheckReport,
    finding,
    pass,
    unknown,
};
/// The origin each clone names and its deferred-destroy mark.
pub use clone_origin_checks::{
    CloneOriginFacts,
    RecordedOrigin,
    check_clone_origin,
};
/// What one clone and its snapshots retain.
pub use clone_retention_checks::{
    AccountingFacts,
    CloneRetentionFacts,
    LOGICAL_REFERENCED_PROPERTY,
    REFERENCED_PROPERTY,
    USED_PROPERTY,
    WRITTEN_PROPERTY,
    check_clone_retention,
};
/// The clone's own snapshots beside the plan's states.
pub use clone_snapshot_checks::{
    CloneSnapshotFacts,
    check_clone_snapshots,
};
/// What one managed slot is.
pub use clone_state_checks::{
    CloneStateFacts,
    check_clone_state,
};
/// The effective delegation.
pub use delegation_checks::{
    FORBIDDEN_PERMISSIONS,
    GrantPlace,
    REQUIRED_GRANTS,
    RequiredGrant,
    allow_command,
    check_delegation,
    missing_grants,
    place_dataset,
};
/// The kernel's delegation reply and the grant model it becomes.
pub use delegation_reply::{
    DelegationFacts,
    Grant,
    Inheritance,
    Who,
    grants_of,
};
/// The handler's installation and folders.
pub use handler_checks::{
    FolderFacts,
    HandlerConfigurationFacts,
    HandlerInstallationFacts,
    HandlerWiringFacts,
    PathFacts,
    PathState,
    REQUIRED_ROOT_MODES,
    ROOT_MODE_EXECUTABLE_PATH,
    ReadinessFacts,
    RequestFacts,
    ZEDLET_EVENT_PATH,
    check_handler_configuration,
    check_handler_folders,
    check_handler_installation,
    check_modes,
    check_readiness,
    check_root_modes,
    check_unresolved_requests,
    check_zed_wiring,
};
/// The namespaces that hold a clone.
pub use holder_checks::{
    HolderFacts,
    NamespaceHold,
    check_holders,
};
/// The recorded parent and slot identities.
pub use identity_checks::{
    LiveIdentity,
    ParentIdentityFacts,
    RecordedIdentity,
    SlotIdentityFacts,
    check_parent_identity,
    check_slot_identity,
    walk_refusal_is_unproved,
};
/// The checks later steps still owe.
pub use later_step_checks::{
    LaterStep,
    check_later_step,
};
/// The sentence one slot's managed Git lock prints.
pub use managed_lock_checks::{
    ManagedLockFacts,
    check_managed_lock,
};
/// The checkout mount,
/// its propagation and the nested mounts' reachability.
pub use mount_checks::{
    MountCheckFacts,
    NestedMount,
    check_checkout_mount,
    check_checkout_propagation,
    check_nested_reachability,
    nested_mounts,
};
/// The Git registration one slot's record names.
pub use registration_checks::{
    RegistrationFacts,
    check_registration,
};
/// The destruction plan one removal left.
pub use removal_plan_checks::{
    PlanFileReading,
    RecordedSnapshot,
    RemovalPlanFacts,
    check_removal_plan,
    recorded_outcomes,
};
/// Every check's answer in one list.
pub use route_checks::{
    CloneFacts,
    RouteFacts,
    route_checks,
};
/// The `snapdir` of the checkout dataset and of the clone container.
pub use snapshot_directory_checks::{
    SNAPDIR_DISABLED,
    SNAPDIR_HIDDEN,
    SNAPDIR_PROPERTY,
    SNAPDIR_VISIBLE,
    SnapdirSetting,
    check_snapdir,
    snapdir_report,
    snapdir_setting,
    snapdir_word,
};
/// The staging folder and its bind.
pub use staging_folder_checks::{
    StagedFolder,
    StagingBind,
    StagingFolderFacts,
    check_staging_bind_propagation,
    check_staging_folder,
    is_derived_folder_name,
    staging_binds,
};
/// The store's folder listing.
pub use store_listing::{
    ListedEntry,
    RecordFacts,
    SlotRecordFacts,
    StoreListingFacts,
    check_staging_forms,
    check_store_entries,
    check_unknown_dot_entries,
    is_foreign_entry,
    is_unknown_dot_entry,
    known_staging_entries,
};
/// The removal of the two known staging forms.
pub use store_staging_removal::{
    StagingRemoval,
    StagingRemovalRefusal,
    StagingRemovalRequest,
    remove_known_staging_forms,
};
/// The device,
/// ABI,
/// pool,
/// dataset,
/// key and property checks and what they read.
pub use zfs_checks::{
    AbiFacts,
    DatasetFacts,
    DatasetRole,
    DeviceFacts,
    DeviceState,
    PoolFacts,
    Reading,
    check_abi,
    check_clone_properties,
    check_dataset,
    check_device,
    check_key,
    check_pool,
    reading_of,
};
