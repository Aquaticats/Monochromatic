//! What:
//!  Every fact plan step 18's `doctor` checks read of this machine,
//!  gathered once from the live system,
//!  each read behind a place or a seam a control can hand its own:
//!  the device,
//!  the layout gate,
//!  the pool and its datasets,
//!  the caller's mount table,
//!  the identity store and the identities it records,
//!  the delegation,
//!  the handler's installed paths and ZED's wiring,
//!  the readiness record,
//!  this boot,
//!  each clone's request and receipt,
//!  the staging folder and its binds.
//! Why:
//!  The checks were built to take their facts as a value and no code gathered
//!  one
//!  (`crate::doctor_zfs_route`,
//!  "The facts themselves are the wiring run's to gather once from the live
//!  system").
//!  This folder is that gather for everything true of the machine,
//!  and the sibling round that gathers each managed clone's removal facts
//!  completes the value:
//!  `MachineRouteFacts::route` is one `RouteFacts` whose clone group is empty
//!  here.
//!  A fact the machine refuses to let this run read does not stop the rest:
//!  the checks that could be answered are,
//!  and each check a failed read leaves unproved answers unknown in
//!  `MachineRouteFacts::unread` with the read's own reason,
//!  which is what the plan asks of a partial read.
//!  Nothing here writes anything,
//!  and nothing here opens `/dev/zfs` outside the live pool:
//!  a control hands in a scripted pool,
//!  so no control of this folder acts on a pool.
//!  The folder sits beside `health/` rather than inside
//!  `health/doctor_zfs_route/`:
//!  a child folder there would make that folder a group,
//!  whose sixty code files the layout rule then reports,
//!  and a use from it of a folder outside the set it already uses joins a
//!  tolerated loop the allow list may only lose
//!  (`doc/decision/cli-git-layout-rules-on-with-a-shrinking-allow-list.md`).
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `the_route_places.rs`:
//!   every path a machine fact is read from,
//!   the release's own and a control's fixture.
//! - `the_machines_pool_answers.rs`:
//!   the five questions the kernel and the pool answer,
//!   behind the one seam a control scripts.
//! - `the_unknown_a_failed_read_answers.rs`:
//!   the report line one failed read answers.
//! - `one_look_at_a_path.rs`:
//!   what one look at one path answered.
//! - `the_route_layout.rs`:
//!   the checkout's path and the two datasets,
//!   read from the handler's configuration.
//! - `the_device_the_gate_and_the_pool.rs`:
//!   the device's facts,
//!   the pool's name and its own identity.
//! - `the_datasets_read.rs`:
//!   the checkout,
//!   the container and every clone below it.
//! - `the_mount_table_read.rs`:
//!   the caller's own mount table.
//! - `the_boot_identity_read.rs`:
//!   this boot's identifier.
//! - `the_store_and_the_identities.rs`:
//!   the store's listing,
//!   the parent identity and each listed slot's identity.
//! - `the_handler_paths_read.rs`:
//!   the installed halves,
//!   the configuration,
//!   the two folders and ZED's wiring.
//! - `the_readiness_record_read.rs`:
//!   the latest readiness record.
//! - `the_clone_requests_read.rs`:
//!   each clone's request and the receipt that answers it.
//! - `the_staging_folders_read.rs`:
//!   the staging folders under the account's cache.
//! - `the_delegation_read.rs`:
//!   the account's effective delegation over the two datasets.
//! - `the_machines_route_facts.rs`:
//!   one value of every fact above,
//!   the checks a failed read leaves unproven,
//!   and the one entry a report calls.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const gathered = gatherTheMachineRouteFacts(); // { route, unread } | { why }
//! const report = routeChecks(gathered.route);
//! ```

/// What one look at one path answered.
mod one_look_at_a_path;
/// This boot's identifier.
mod the_boot_identity_read;
/// Each clone's request and the receipt that answers it.
mod the_clone_requests_read;
/// The datasets the checks judge.
mod the_datasets_read;
/// The account's effective delegation.
mod the_delegation_read;
/// The device,
/// the gate and the pool.
mod the_device_the_gate_and_the_pool;
/// The handler's installed paths and ZED's wiring.
mod the_handler_paths_read;
/// The kernel's answers and the release's own,
/// behind one seam.
mod the_machines_pool_answers;
/// Every fact of this machine the checks read,
/// gathered once from the live system.
pub mod the_machines_route_facts;
/// The caller's mount table.
mod the_mount_table_read;
/// The latest readiness record.
mod the_readiness_record_read;
/// The checkout's path and the two datasets.
mod the_route_layout;
/// Every path a machine fact is read from.
mod the_route_places;
/// The staging folders under the account's cache.
mod the_staging_folders_read;
/// The store,
/// the parent identity and each slot's identity.
mod the_store_and_the_identities;
/// The report line one failed read answers.
mod the_unknown_a_failed_read_answers;

/// The pool a control scripts,
/// never part of the release executable.
#[cfg(test)]
mod a_scripted_pool_for_each_control;

/// One fixture of the places and readers a control gathers over.
#[cfg(test)]
mod one_fixtures_places_and_readers;

/// This machine's route facts,
/// the checks a failed read leaves unproven,
/// and the sentence a gather that could not name the layout answers.
pub use the_machines_route_facts::{
    MachineFactsRefusal,
    MachineRouteFacts,
    gather_the_machine_route_facts,
};
