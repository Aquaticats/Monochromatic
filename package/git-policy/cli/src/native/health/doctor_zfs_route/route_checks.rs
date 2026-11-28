//! What:
//!  Every check plan step 18 adds,
//!  answered once from one set of facts and returned in report order.
//! Why:
//!  The wiring run gathers the facts once and prints one list,
//!  so a check cannot judge a state another check has already changed,
//!  and the order is the design's:
//!  the device and datasets first,
//!  then the checkout's mount,
//!  the recorded identities and the store,
//!  the delegation,
//!  the handler and its records,
//!  the staging folder,
//!  and then one group per managed clone:
//!  its state,
//!  its Git registration,
//!  its managed Git lock,
//!  its binding record,
//!  its destruction plan,
//!  its own snapshots,
//!  its origin,
//!  its retention,
//!  its blocked-removal state and the namespaces that hold it,
//!  the boot setup and the covered entry plan step 19 installs,
//!  the cancel command this build carries,
//!  the backup coverage step 19 still owes,
//!  and Sanoid's coverage step 20 decides
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "What doctor checks and prints";
//!  plan step 18,
//!  "What it builds").
//!  One clone's group follows the order a person reads the removal's own
//!  refusals in:
//!  what the slot is,
//!  what Git and the records say about it,
//!  what the plan did,
//!  what the pool holds,
//!  what it retains,
//!  and what stops a removal.
//!  Nothing here removes a staging form:
//!  the report reads first and the wiring run calls the removal after the
//!  report is written,
//!  which is why that one call is a separate function.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reports = route_checks(facts); // one CheckReport per check, in print order
//! ```

use super::binding_record_checks::check_binding_record;
use super::blocked_removal_checks::{
    BlockedRemovalFacts,
    check_blocked_removal,
};
use super::boot_protection_checks::BootProtectionFacts;
use super::boot_setup_facts::BootSetupFacts;
use super::cancel_command_checks::CancelCommandFacts;
use super::check_outcome::{
    Check,
    CheckOutcome,
    CheckReport,
};
use super::clone_origin_checks::{
    CloneOriginFacts,
    check_clone_origin,
};
use super::clone_retention_checks::{
    CloneRetentionFacts,
    check_clone_retention,
};
use super::clone_snapshot_checks::{
    CloneSnapshotFacts,
    check_clone_snapshots,
};
use super::clone_state_checks::{
    CloneStateFacts,
    check_clone_state,
};
use super::delegation_checks::check_delegation;
use super::delegation_reply::DelegationFacts;
use super::handler_checks::{
    FolderFacts,
    HandlerConfigurationFacts,
    HandlerInstallationFacts,
    HandlerWiringFacts,
    ReadinessFacts,
    RequestFacts,
    check_handler_configuration,
    check_handler_folders,
    check_handler_installation,
    check_readiness,
    check_root_modes,
    check_unresolved_requests,
    check_zed_wiring,
};
use super::holder_checks::{
    HolderFacts,
    check_holders,
};
use super::identity_checks::{
    ParentIdentityFacts,
    SlotIdentityFacts,
    check_parent_identity,
    check_slot_identity,
};
use super::later_checks_report_lines::{
    LaterCheckFacts,
    later_checks_reports,
};
use super::managed_lock_checks::{
    ManagedLockFacts,
    check_managed_lock,
};
use super::mount_checks::{
    MountCheckFacts,
    check_checkout_mount,
    check_checkout_propagation,
    check_nested_reachability,
};
use super::registration_checks::{
    RegistrationFacts,
    check_registration,
};
use super::removal_plan_checks::{
    RemovalPlanFacts,
    check_removal_plan,
};
use super::sanoid_report::SanoidReport;
use super::snapshot_directory_checks::snapdir_report;
use super::staging_folder_checks::{
    StagingBind,
    StagingFolderFacts,
    check_staging_bind_propagation,
    check_staging_folder,
};
use super::store_listing::{
    StoreListingFacts,
    check_staging_forms,
    check_store_entries,
    check_unknown_dot_entries,
};
use super::zfs_checks::{
    AbiFacts,
    DatasetFacts,
    DatasetRole,
    DeviceFacts,
    PoolFacts,
    check_abi,
    check_clone_properties,
    check_dataset,
    check_device,
    check_key,
    check_pool,
};

/// What:
///  Every removal-side fact one managed clone's checks read,
///  gathered once.
/// Why:
///  One clone's state,
///  records,
///  snapshots,
///  origin and holders belong to one
///  value,
///  so no line can describe one clone's plan beside another clone's pool answer,
///  and a control changes one field and sees exactly one answer move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CloneFacts = { state; registration; plan; snapshots; origin; retention; blocked };
/// ```
pub struct CloneFacts {
    /// What the slot is,
    /// and every live staging row the state is read with.
    pub state: CloneStateFacts,
    /// The Git registration the record names and what stands there.
    pub registration: RegistrationFacts,
    /// The managed Git lock that registration holds,
    /// in step 13's state.
    pub lock: ManagedLockFacts,
    /// The destruction plan that names this clone,
    /// when one stands.
    pub plan: RemovalPlanFacts,
    /// The clone's own snapshots beside the plan's states.
    pub snapshots: CloneSnapshotFacts,
    /// The origin the record names and its deferred-destroy mark.
    pub origin: CloneOriginFacts,
    /// What the clone and its snapshots retain.
    pub retention: CloneRetentionFacts,
    /// The blocked-removal state.
    pub blocked: BlockedRemovalFacts,
    /// The all-namespace scan's answer for this clone.
    pub holders: HolderFacts,
}

/// What:
///  Every fact plan step 18's checks read,
///  gathered once.
/// Why:
///  One value is the whole report's input,
///  so a control builds the healthy set,
///  changes one field and sees exactly one answer move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RouteFacts = { device; abi; pool; datasets; mounts; parent; slots; store; delegation; installation; configuration; folders; wiring; readiness; liveBoot; requests; staging; stagingBinds; clones };
/// ```
pub struct RouteFacts {
    /// `/dev/zfs` and this account's open of it.
    pub device: DeviceFacts,
    /// The layout gate's answer.
    pub abi: AbiFacts,
    /// The pool the checkout dataset names.
    pub pool: PoolFacts,
    /// The checkout,
    /// the container and each clone.
    pub datasets: Vec<DatasetFacts>,
    /// The caller's mount table.
    pub mounts: MountCheckFacts,
    /// The recorded and live parent identities.
    pub parent_identity: ParentIdentityFacts,
    /// Each listed slot's recorded identity against its live folder.
    pub slots: Vec<SlotIdentityFacts>,
    /// The store folder's listing.
    pub store: StoreListingFacts,
    /// The account's effective delegation.
    pub delegation: DelegationFacts,
    /// The handler's two installed halves.
    pub installation: HandlerInstallationFacts,
    /// The handler's configuration.
    pub configuration: HandlerConfigurationFacts,
    /// The readiness and results folders.
    pub folders: FolderFacts,
    /// ZED and the service wiring.
    pub wiring: HandlerWiringFacts,
    /// The latest readiness record.
    pub readiness: ReadinessFacts,
    /// This boot's identifier,
    /// which a record must match.
    pub live_boot: String,
    /// Each clone's request and the receipt that answers it.
    pub requests: Vec<RequestFacts>,
    /// The cache and the staging folders under it.
    pub staging: StagingFolderFacts,
    /// Every live mount under the cache with a staging name.
    pub staging_binds: Vec<StagingBind>,
    /// One group per managed clone the store lists.
    pub clones: Vec<CloneFacts>,
    /// The checkout's covered entry and the account that must own it,
    /// which plan step 19's protection is read from.
    pub boot_protection: BootProtectionFacts,
    /// The boot setup's plan,
    /// the files installed from it and the datasets measured under the
    /// checkout.
    pub boot_setup: BootSetupFacts,
    /// The cancel command this build carries,
    /// as its own grammar states it.
    pub cancel_command: CancelCommandFacts,
    /// Sanoid's coverage comparison as the coverage folder answered it,
    /// with the corrections below its line.
    pub sanoid: SanoidReport,
}

/// What:
///  Every report plan step 18's checks produce,
///  in print order.
/// Why:
///  The wiring run prints this list and nothing else decides the order,
///  so two runs of one machine are comparable and a control can find a line
///  by its check's name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reports = route_checks(facts); // [{ check, outcome }, ...]
/// ```
pub fn route_checks(facts: &RouteFacts) -> Vec<CheckReport> {
    let mut reports: Vec<CheckReport> = Vec::new();
    reports.push(CheckReport::new(Check::Device, check_device(&facts.device)));
    reports.push(CheckReport::new(Check::Abi, check_abi(&facts.abi)));
    reports.push(CheckReport::new(Check::Pool, check_pool(&facts.pool)));
    for dataset in &facts.datasets {
        if matches!(dataset.role, DatasetRole::Clone) {
            continue;
        }
        reports.push(CheckReport::new(dataset.check(), check_dataset(dataset)));
        if let Some(report) = snapdir_report(dataset) {
            reports.push(report);
        }
    }
    reports.push(CheckReport::new(Check::Key, check_key(&facts.datasets)));
    let clones: Vec<DatasetFacts> = facts
        .datasets
        .iter()
        .filter(|dataset| return matches!(dataset.role, DatasetRole::Clone))
        .cloned()
        .collect();
    reports.push(CheckReport::new(
        Check::CloneProperties,
        check_clone_properties(&clones),
    ));
    reports.push(CheckReport::new(
        Check::CheckoutMount,
        check_checkout_mount(&facts.mounts),
    ));
    reports.push(CheckReport::new(
        Check::CheckoutPropagation,
        check_checkout_propagation(&facts.mounts),
    ));
    reports.push(CheckReport::new(
        Check::NestedReachability,
        check_nested_reachability(&facts.mounts),
    ));
    reports.push(CheckReport::new(
        Check::ParentIdentity,
        check_parent_identity(&facts.parent_identity),
    ));
    for slot in &facts.slots {
        reports.push(CheckReport::new(
            Check::SlotIdentity,
            check_slot_identity(slot),
        ));
    }
    reports.push(CheckReport::new(
        Check::StoreEntries,
        check_store_entries(&facts.store),
    ));
    reports.push(CheckReport::new(
        Check::StagingForms,
        check_staging_forms(&facts.store),
    ));
    reports.push(CheckReport::new(
        Check::UnknownDotEntry,
        check_unknown_dot_entries(&facts.store),
    ));
    reports.push(CheckReport::new(
        Check::Delegation,
        check_delegation(&facts.delegation),
    ));
    reports.push(CheckReport::new(
        Check::HandlerInstallation,
        check_handler_installation(&facts.installation),
    ));
    reports.push(CheckReport::new(Check::RootModes, check_root_modes()));
    reports.push(CheckReport::new(
        Check::HandlerConfiguration,
        check_handler_configuration(&facts.configuration),
    ));
    reports.push(CheckReport::new(
        Check::HandlerFolders,
        check_handler_folders(&facts.folders),
    ));
    reports.push(CheckReport::new(
        Check::ZedWiring,
        check_zed_wiring(&facts.wiring),
    ));
    reports.push(CheckReport::new(
        Check::Readiness,
        check_readiness(&facts.readiness, facts.live_boot.as_str()),
    ));
    reports.push(CheckReport::new(
        Check::UnresolvedRequests,
        check_unresolved_requests(&facts.requests, facts.live_boot.as_str()),
    ));
    reports.push(CheckReport::new(
        Check::StagingFolder,
        check_staging_folder(&facts.staging),
    ));
    reports.push(CheckReport::new(
        Check::StagingBindPropagation,
        check_staging_bind_propagation(&facts.staging_binds),
    ));
    reports.extend(clone_reports(facts));
    reports.extend(later_checks_reports(&LaterCheckFacts {
        boot_protection: &facts.boot_protection,
        boot_setup: &facts.boot_setup,
        cancel_command: &facts.cancel_command,
        sanoid: &facts.sanoid,
    }));
    return reports;
}

/// What:
///  One group of lines per managed clone,
///  in the order a person reads one clone's removal state.
/// Why:
///  A repository can hold several managed clones,
///  and a plan,
///  a registration or a holder belongs to one of them,
///  so the lines are grouped per clone rather than mixed by check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = facts.clones.flatMap((clone) => [stateLine(clone), ...]);
/// ```
fn clone_reports(facts: &RouteFacts) -> Vec<CheckReport> {
    let mut reports: Vec<CheckReport> = Vec::new();
    for clone in &facts.clones {
        let binding: CheckOutcome = check_binding_record(&clone.state.readings.binding);
        reports.push(CheckReport::new(
            Check::CloneState,
            check_clone_state(&clone.state),
        ));
        reports.push(CheckReport::new(
            Check::Registration,
            check_registration(&clone.registration),
        ));
        reports.push(CheckReport::new(
            Check::ManagedLock,
            check_managed_lock(&clone.lock),
        ));
        reports.push(CheckReport::new(Check::BindingRecord, binding));
        reports.push(CheckReport::new(
            Check::RemovalPlan,
            check_removal_plan(&clone.plan),
        ));
        reports.push(CheckReport::new(
            Check::CloneSnapshots,
            check_clone_snapshots(&clone.snapshots),
        ));
        reports.push(CheckReport::new(
            Check::CloneOrigin,
            check_clone_origin(&clone.origin),
        ));
        reports.push(CheckReport::new(
            Check::CloneRetention,
            check_clone_retention(&clone.retention),
        ));
        reports.push(CheckReport::new(
            Check::BlockedRemoval,
            check_blocked_removal(&clone.blocked),
        ));
        reports.push(CheckReport::new(
            Check::Holders,
            check_holders(&clone.holders),
        ));
    }
    return reports;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "route_checks_tests.rs"]
mod tests;
