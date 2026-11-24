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
//!  the holders,
//!  and the checks later steps owe
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "What doctor checks and prints";
//!  plan step 18,
//!  "What it builds").
//!  Nothing here removes a staging form:
//!  the report reads first and the wiring run calls the removal after the
//!  report is written,
//!  which is why that one call is a separate function.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reports = route_checks(facts); // one CheckReport per check, in print order
//! ```

use super::check_outcome::CheckReport;
use super::delegation_checks::{DelegationFacts, check_delegation};
use super::handler_checks::{
    FolderFacts, HandlerConfigurationFacts, HandlerInstallationFacts, HandlerWiringFacts, ReadinessFacts, RequestFacts, check_handler_configuration, check_handler_folders, check_handler_installation, check_readiness,
    check_root_modes, check_unresolved_requests, check_zed_wiring,
};
use super::holder_checks::{HolderFacts, check_holders};
use super::identity_checks::{ParentIdentityFacts, SlotIdentityFacts, check_parent_identity, check_slot_identity};
use super::later_step_checks::{LaterStep, check_later_step};
use super::mount_checks::{MountCheckFacts, check_checkout_mount, check_checkout_propagation, check_nested_reachability};
use super::staging_folder_checks::{StagingBind, StagingFolderFacts, check_staging_bind_propagation, check_staging_folder};
use super::store_listing::{StoreListingFacts, check_staging_forms, check_store_entries, check_unknown_dot_entries};
use super::zfs_checks::{AbiFacts, DatasetFacts, DatasetRole, DeviceFacts, PoolFacts, check_abi, check_clone_properties, check_dataset, check_device, check_key, check_pool};
use super::check_outcome::Check;

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
/// type RouteFacts = { device; abi; pool; datasets; mounts; parent; slots; store; delegation; installation; configuration; folders; wiring; readiness; liveBoot; requests; staging; stagingBinds; holders };
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
    /// The scan's answer for one clone.
    pub holders: HolderFacts,
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
    }
    reports.push(CheckReport::new(Check::Key, check_key(&facts.datasets)));
    let clones: Vec<DatasetFacts> = facts.datasets.iter().filter(|dataset| return matches!(dataset.role, DatasetRole::Clone)).cloned().collect();
    reports.push(CheckReport::new(Check::CloneProperties, check_clone_properties(&clones)));
    reports.push(CheckReport::new(Check::CheckoutMount, check_checkout_mount(&facts.mounts)));
    reports.push(CheckReport::new(Check::CheckoutPropagation, check_checkout_propagation(&facts.mounts)));
    reports.push(CheckReport::new(Check::NestedReachability, check_nested_reachability(&facts.mounts)));
    reports.push(CheckReport::new(Check::ParentIdentity, check_parent_identity(&facts.parent_identity)));
    for slot in &facts.slots {
        reports.push(CheckReport::new(Check::SlotIdentity, check_slot_identity(slot)));
    }
    reports.push(CheckReport::new(Check::StoreEntries, check_store_entries(&facts.store)));
    reports.push(CheckReport::new(Check::StagingForms, check_staging_forms(&facts.store)));
    reports.push(CheckReport::new(Check::UnknownDotEntry, check_unknown_dot_entries(&facts.store)));
    reports.push(CheckReport::new(Check::Delegation, check_delegation(&facts.delegation)));
    reports.push(CheckReport::new(Check::HandlerInstallation, check_handler_installation(&facts.installation)));
    reports.push(CheckReport::new(Check::RootModes, check_root_modes()));
    reports.push(CheckReport::new(Check::HandlerConfiguration, check_handler_configuration(&facts.configuration)));
    reports.push(CheckReport::new(Check::HandlerFolders, check_handler_folders(&facts.folders)));
    reports.push(CheckReport::new(Check::ZedWiring, check_zed_wiring(&facts.wiring)));
    reports.push(CheckReport::new(Check::Readiness, check_readiness(&facts.readiness, facts.live_boot.as_str())));
    reports.push(CheckReport::new(Check::UnresolvedRequests, check_unresolved_requests(&facts.requests, facts.live_boot.as_str())));
    reports.push(CheckReport::new(Check::StagingFolder, check_staging_folder(&facts.staging)));
    reports.push(CheckReport::new(Check::StagingBindPropagation, check_staging_bind_propagation(&facts.staging_binds)));
    reports.push(CheckReport::new(Check::Holders, check_holders(&facts.holders)));
    for step in LaterStep::ALL {
        reports.push(CheckReport::new(step.check(), check_later_step(step)));
    }
    return reports;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "route_checks_tests.rs"]
mod tests;
