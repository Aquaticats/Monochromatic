//! What:
//!  The facts a control builds for the ZFS route's `doctor` checks:
//!  one healthy dataset of each role,
//!  one mount table with the checkout on it,
//!  one store listing and one delegation that hold the decided shape,
//!  and the one fact set every control starts from (`healthy_route_facts`).
//!  Each part of those facts is a sibling file of its own,
//!  and this file re-exports them,
//!  so a control keeps one path for all of them.
//! Why:
//!  Every check takes its facts as a value,
//!  so one builder of a healthy route lets each control change exactly the
//!  field it judges and read the one answer that moved.
//!  The values are the design's own:
//!  the checkout dataset mounted at the checkout path,
//!  the clone container beside it with `mountpoint=legacy`,
//!  `canmount=off`,
//!  `setuid=off` and `devices=off`,
//!  and a clone below it
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Dataset and path layout").
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = healthyRouteFacts(); facts.datasets[1].reading.value.mountpoint = { kind: 'text', text: '/' };
//! const report = route_checks(facts).find((entry) => entry.check === 'container-dataset');
//! ```

use super::handler_checks::PathState;
use super::mount_checks::MountCheckFacts;
use super::route_checks::RouteFacts;
/// The covered entry,
/// the boot setup and the Sanoid comparison of a healthy machine.
pub(crate) use super::route_test_facts_boot_and_sanoid::{
    healthy_boot_protection,
    healthy_boot_setup,
    healthy_cancel_command,
    healthy_sanoid,
};
/// One managed clone at the design's healthy state.
pub(crate) use super::route_test_facts_clone::healthy_clone;
/// One healthy dataset of each role,
/// with the fields a control changes.
pub(crate) use super::route_test_facts_datasets::{
    DatasetBuilder,
    dataset,
    healthy_datasets,
};
/// The nine decided grants and the delegation they make.
pub(crate) use super::route_test_facts_delegation::{
    grant,
    healthy_delegation,
    healthy_grants,
};
/// The handler's halves,
/// configuration,
/// folders,
/// wiring,
/// readiness and requests.
pub(crate) use super::route_test_facts_handler::{
    healthy_configuration,
    healthy_folders,
    healthy_installation,
    healthy_readiness,
    healthy_request,
    healthy_wiring,
};
/// One mount table with the checkout on it,
/// and the staging folder under the account's cache.
pub(crate) use super::route_test_facts_mounts_and_staging_folder::{
    healthy_mount_text,
    healthy_mounts,
    healthy_staging,
    mount_row,
};
/// The checkout the design names,
/// the pool it lives in,
/// the checkout's and the clone container's datasets,
/// one clone below the container,
/// the managed account and its group,
/// this boot,
/// the staging folder's prefix,
/// the add nonce the healthy clone's lock names,
/// and the origin snapshot every fixture clone names.
pub(crate) use super::route_test_facts_names::{
    ACCOUNT,
    ACCOUNT_GROUP,
    CHECKOUT,
    CHECKOUT_DATASET,
    CLONE_DATASET,
    CONTAINER_DATASET,
    LIVE_BOOT,
    POOL,
    STAGING_PREFIX,
};
/// One path at the state a healthy route shows,
/// and the helper that gives a root-owned file.
pub(crate) use super::route_test_facts_paths::{
    path_facts,
    root_file,
};
/// The store's listing and the parent identities it records.
pub(crate) use super::route_test_facts_store_and_identities::{
    folder_identity,
    healthy_parent_identity,
    healthy_store,
};
use super::staging_folder_checks::{
    StagingBind,
    StagingFolderFacts,
    staging_binds,
};
use super::zfs_checks::{
    AbiFacts,
    DeviceFacts,
    DeviceState,
    PoolFacts,
    Reading,
};
use std::path::PathBuf;

/// The healthy route's whole fact set,
/// which every control starts from.
pub(crate) fn healthy_route_facts() -> RouteFacts {
    let mounts: MountCheckFacts = healthy_mounts();
    let staging_binds_value: Vec<StagingBind> =
        staging_binds(&mounts.rows, mounts.checkout_path.as_path());
    let cache: String = String::from("/home/user/.cache");
    return RouteFacts {
        device: DeviceFacts {
            path: PathBuf::from(monochromatic_zfs_request_to_openzfs_kernel_device::DEVICE_PATH),
            state: DeviceState::Opens,
        },
        abi: AbiFacts::Compatible {
            module: Some(String::from("2.4.4-1")),
            details: vec![String::from("base type information 1234 bytes")],
        },
        pool: PoolFacts {
            pool: String::from(POOL),
            reading: Reading::Read(9),
        },
        datasets: healthy_datasets(),
        mounts,
        parent_identity: healthy_parent_identity(),
        slots: Vec::new(),
        store: healthy_store(),
        delegation: healthy_delegation(),
        installation: healthy_installation(),
        configuration: healthy_configuration(),
        folders: healthy_folders(),
        wiring: healthy_wiring(),
        readiness: healthy_readiness(),
        live_boot: String::from(LIVE_BOOT),
        requests: vec![healthy_request()],
        staging: StagingFolderFacts {
            cache: path_facts(
                cache.as_str(),
                PathState::Directory {
                    owner: ACCOUNT,
                    mode: 0o700,
                },
            ),
            account: ACCOUNT,
            folders: Vec::new(),
        },
        staging_binds: staging_binds_value,
        clones: vec![healthy_clone()],
        boot_protection: healthy_boot_protection(),
        boot_setup: healthy_boot_setup(),
        cancel_command: healthy_cancel_command(),
        sanoid: healthy_sanoid(),
    };
}
