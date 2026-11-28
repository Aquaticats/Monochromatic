//! What:
//!  Every fact plan step 18's `doctor` checks read of this machine,
//!  gathered once from the live system:
//!  the device,
//!  the layout gate,
//!  the pool and its datasets,
//!  the caller's mount table,
//!  the identity store and the identities it records,
//!  the delegation,
//!  the handler's installed paths,
//!  its readiness record,
//!  this boot,
//!  each clone's request and receipt,
//!  the staging folder,
//!  and the staging binds.
//!  Each managed clone's own removal facts are the sibling round's,
//!  so this value holds no clone group.
//! Why:
//!  `route_checks` takes one `RouteFacts` value and answers every check from
//!  it,
//!  so the facts are gathered once here and never re-read between two checks
//!  (`crate::doctor_zfs_route::route_checks`;
//!  plan step 18,
//!  "What it builds").
//!  A fact nobody could read does not stop the report:
//!  the facts that were read still answer their checks,
//!  and each check a failed read leaves unproved answers unknown in `unread`
//!  with the read's own reason,
//!  so one unreadable folder costs its own checks and no other.
//!  The route's own layout is the exception:
//!  without the checkout's path and the two datasets' names no route fact can
//!  be named at all,
//!  so the gather answers the refusal instead of a value whose names a check
//!  would print as the machine's own.
//!  Nothing here writes anything:
//!  the two staging forms a killed writer left are plan step 18's removal and a
//!  later round's call,
//!  and this report only names them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const gathered = the_machines_route_facts({ places, pool, store, readiness, receipts }); // { route, unread } | { why }
//! const report = routeChecks(gathered.route);
//! ```

/// This boot's identifier.
use super::the_boot_identity_read::the_live_boot;
/// Each clone's request and the receipt that answers it.
use super::the_clone_requests_read::the_request_facts;
/// The checkout,
/// the container and every clone below it.
use super::the_datasets_read::the_dataset_facts;
/// The delegation over the two datasets.
use super::the_delegation_read::the_delegation_facts;
/// The device,
/// the gate and the pool.
use super::the_device_the_gate_and_the_pool::{
    the_device_facts,
    the_pool_facts,
};
/// The handler's installed paths and ZED's wiring.
use super::the_handler_paths_read::{
    TheHandlerPaths,
    the_handler_paths_read,
};
/// The kernel's answers and the release's own.
use super::the_machines_pool_answers::{
    TheLivePool,
    TheMachinesPool,
};
/// The caller's mount table.
use super::the_mount_table_read::the_mount_facts;
/// The latest readiness record.
use super::the_readiness_record_read::the_readiness_facts;
/// The three names the route's layout holds,
/// read from the handler's configuration.
use super::the_route_layout::the_routes_layout;
/// The places every path of this gather is named by.
use super::the_route_places::TheRoutePlaces;
/// The staging folders under the account's cache.
use super::the_staging_folders_read::the_staging_facts;
/// The store,
/// the parent identity and each slot's identity.
use super::the_store_and_the_identities::{
    TheStoreAndTheIdentities,
    TheStoreReadInput,
    the_store_and_the_identities,
};
/// The unknown a failed read answers.
use super::the_unknown_a_failed_read_answers::the_unknown_of;
/// One check's stable name,
/// and the report line a check answers.
use crate::doctor_zfs_route::check_outcome::{
    Check,
    CheckReport,
};
/// What every check plan step 18 adds reads,
/// with each managed clone's group left to the sibling round.
use crate::doctor_zfs_route::route_checks::RouteFacts;
/// One live staging bind.
use crate::doctor_zfs_route::staging_folder_checks::StagingBind;
/// The staging binds the caller's rows hold,
/// read from the cache's own name.
use crate::doctor_zfs_route::staging_folder_checks::staging_binds;
/// The account this report is about,
/// which is the one the kernel judges a delegation against.
use crate::worktree_clone_add_route::add_grants::this_account;
/// The store folder the route reads under root control,
/// and the refusal of one that could not be opened.
use crate::worktree_clone_mount::{
    SlotIdentityStore,
    StoreRefusal,
};
/// The handler's own reader of the readiness record.
use crate::worktree_clone_mount_protocol::ReadinessFile;
/// The folder the handler files its receipts in.
use crate::worktree_clone_mount_protocol::ReceiptsFolder;

/// What:
///  This machine's route facts,
///  with no clone group,
///  and one unknown per check a failed read left unproved.
/// Why:
///  The checks read the facts,
///  and a read the machine refused has no fact to read,
///  so the unknowns travel beside the facts rather than as a value a check
///  would print as a pass.
///  A report prints each of these lines in place of the line `route_checks`
///  answers for the same check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MachineRouteFacts = { route: RouteFacts; unread: CheckReport[] };
/// ```
pub struct MachineRouteFacts {
    /// The machine's facts,
    /// with no clone group.
    pub route: RouteFacts,
    /// One unknown per check whose read this machine refused,
    /// in report order.
    pub unread: Vec<CheckReport>,
}

/// What:
///  Why the machine's route facts could not be gathered at all.
/// Why:
///  Without the route's own layout no dataset,
///  pool,
///  mount or delegation fact can be named,
///  so the report says which file it could not read and what settles that
///  rather than checking the route against names it invented.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class MachineFactsRefusal extends Error { message: string }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MachineFactsRefusal {
    /// The sentence the report prints.
    pub why: String,
}

/// What:
///  Everything one gather reads through:
///  where the paths are,
///  the kernel's answers,
///  the store the route opened,
///  the readiness record's own reader and the receipts' folder.
/// Why:
///  A store,
///  a readiness record and a receipt each carry the owner term their folder
///  must answer,
///  so the caller opens them and this bundle keeps one place for the parts a
///  control replaces:
///  a fixture's folder and a scripted pool in place of this machine's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheMachinesReads = { places, pool, store, readiness, receipts };
/// ```
pub(crate) struct TheMachinesReads<'a> {
    /// The places every path of this gather is named by.
    pub places: &'a TheRoutePlaces,
    /// The kernel's answers,
    /// which a control scripts.
    pub pool: &'a dyn TheMachinesPool,
    /// The identity store folder,
    /// or the refusal of one that could not be opened.
    pub store: Result<&'a SlotIdentityStore, &'a StoreRefusal>,
    /// The handler's reader of the latest readiness record.
    pub readiness: &'a ReadinessFile,
    /// The folder the handler files its receipts in.
    pub receipts: &'a ReceiptsFolder,
}

/// What:
///  This machine's route facts,
///  gathered once from the places and readers a caller hands in.
/// Why:
///  One body reads every fact in the design's order,
///  so a control drives the same code with a fixture's places and a scripted
///  pool,
///  and a check that reads one fact cannot see a second version of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gathered = the_machines_route_facts({ places, pool, store, readiness, receipts });
/// ```
pub(crate) fn the_machines_route_facts(
    reads: TheMachinesReads<'_>
) -> Result<MachineRouteFacts, MachineFactsRefusal> {
    let layout = the_routes_layout(reads.places).map_err(|why| {
        return MachineFactsRefusal {
            why,
        };
    })?;
    let places: &TheRoutePlaces = reads.places;
    let pool: &dyn TheMachinesPool = reads.pool;
    let mut unread: Vec<CheckReport> = Vec::new();
    let device = the_device_facts(places, pool);
    let abi: crate::doctor_zfs_route::zfs_checks::AbiFacts = pool.the_layouts_verdict();
    let pool_facts = the_pool_facts(&layout, pool);
    let datasets = the_dataset_facts(&layout, pool, &mut unread);
    let mounts = the_mount_facts(places, &layout, &mut unread);
    let staging_binds: Vec<StagingBind> = the_binds(places, &mounts.rows, &mut unread);
    let store: TheStoreAndTheIdentities = the_store_and_the_identities(TheStoreReadInput {
        places,
        layout: &layout,
        store: reads.store,
        unread: &mut unread,
    });
    let handler: TheHandlerPaths = the_handler_paths_read(places, &mut unread);
    let readiness: crate::doctor_zfs_route::handler_checks_readiness::ReadinessFacts =
        the_readiness_facts(reads.readiness);
    let live_boot: String = the_live_boot(places, &mut unread);
    let requests = the_request_facts(datasets.as_slice(), reads.receipts, &mut unread);
    let account: u32 = this_account();
    let staging: crate::doctor_zfs_route::staging_folder_checks::StagingFolderFacts =
        the_staging_facts(places, account);
    let delegation = the_delegation_facts(&layout, pool, &mut unread);
    unread.sort_by_key(|report| return report.check);
    unread.dedup_by_key(|report| return report.check);
    return Ok(MachineRouteFacts {
        route: RouteFacts {
            device,
            abi,
            pool: pool_facts,
            datasets,
            mounts,
            parent_identity: store.parent_identity,
            slots: store.slots,
            store: store.store,
            delegation,
            installation: handler.installation,
            configuration: handler.configuration,
            folders: handler.folders,
            wiring: handler.wiring,
            readiness,
            live_boot,
            requests,
            staging,
            staging_binds,
            clones: Vec::new(),
        },
        unread,
    });
}

/// What:
///  Every live staging bind the caller's rows hold,
///  or the unknown the staging bind's check answers when the account's cache
///  cannot be named.
/// Why:
///  A bind is found by the name of the folder it stands on,
///  and a machine whose cache has no name has no folder to look under,
///  so the check answers unknown rather than the pass an empty list gets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const binds = theBinds(places, rows, unread); // [{ mountPoint, source, sharing }]
/// ```
fn the_binds(
    places: &TheRoutePlaces,
    rows: &[crate::subvolume::fs_mount_options::MountFacts],
    unread: &mut Vec<CheckReport>,
) -> Vec<StagingBind> {
    let cache: std::path::PathBuf = match places.the_cache_or_why() {
        Ok(held) => held,
        Err(why) => {
            unread.push(the_unknown_of(Check::StagingBindPropagation, why.as_str()));
            return Vec::new();
        }
    };
    return staging_binds(rows, cache.as_path());
}

/// What:
///  This machine's route facts,
///  gathered from the paths the release installs at and the pool this machine
///  answers with.
/// Why:
///  This is the one entry a report calls:
///  it names the live places,
///  opens the store folder the route reads under root control,
///  and hands the live pool and readers to the gather.
///  No command reaches it yet;
///  the round that prints the report is the one that calls it
///  (`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`).
///
/// # Errors
/// The route's own layout could not be named,
/// so no dataset,
/// pool,
/// mount or delegation fact could be read,
/// and the sentence names the configuration file and the cause.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gathered = gatherTheMachineRouteFacts();
/// ```
pub fn gather_the_machine_route_facts() -> Result<MachineRouteFacts, MachineFactsRefusal> {
    let places: TheRoutePlaces = TheRoutePlaces::live();
    let pool: TheLivePool = TheLivePool::at(&places);
    let store: Result<SlotIdentityStore, StoreRefusal> =
        SlotIdentityStore::open_under_root_control(places.store.as_path());
    let readiness: ReadinessFile = ReadinessFile::live();
    let receipts: ReceiptsFolder = ReceiptsFolder::live();
    // What:     `as_ref` makes one `Result` of two borrows, so the opened store and its refusal are lent to the
    //           gather while both values stay here.
    // Why:      The store is opened once and read many times, so the gather never opens a second descriptor.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const held = store; // the same value, borrowed
    // ```
    let held: Result<&SlotIdentityStore, &StoreRefusal> = store.as_ref();
    return the_machines_route_facts(TheMachinesReads {
        places: &places,
        pool: &pool,
        store: held,
        readiness: &readiness,
        receipts: &receipts,
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_machines_route_facts_tests.rs"]
mod tests;
