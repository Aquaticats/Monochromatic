//! What:
//!  Controls of the device,
//!  ABI,
//!  pool,
//!  dataset,
//!  key and property checks.
//! Why:
//!  Each control changes exactly one fact of the design's healthy route and
//!  reads the one answer that moved,
//!  so a guard that stopped judging its field turns its control red.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(checkContainer(datasetWith({ canmount: 1 }))).toMatchObject({ kind: 'finding', field: 'canmount' });
//! ```

use crate::health::doctor_zfs_route::route_test_facts::{dataset, DatasetBuilder, CLONE_DATASET, CONTAINER_DATASET};
use super::{
    AbiFacts, CANMOUNT_OFF, DatasetFacts, DatasetRole, DeviceFacts, DeviceState, KEY_UNAVAILABLE, NOT_FOUND_ERRNO, PoolFacts, Reading, check_abi, check_clone_properties, check_dataset, check_device, check_key, check_pool, reading_of,
};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::ReadFailure;
use std::path::PathBuf;

/// One dataset reading,
/// for a check that reads several.
fn read(name: &str, role: DatasetRole, facts: crate::worktree_clone_mount_handler::sweep_dataset_facts::CloneObjectFacts) -> DatasetFacts {
    let mut dataset: DatasetFacts = dataset(role, facts);
    dataset.name = String::from(name);
    return dataset;
}

/// The reading the device facts carry.
fn device(state: DeviceState) -> DeviceFacts {
    return DeviceFacts { path: PathBuf::from("/dev/zfs"), state };
}

//region The device

/// A device this account opened passes.
#[test]
fn a_device_this_account_opened_passes() {
    let outcome: super::CheckOutcome = check_device(&device(DeviceState::Opens));
    assert!(matches!(outcome, super::CheckOutcome::Pass { .. }), "{outcome:?}");
}

/// A device that is not there is a finding naming the device.
#[test]
fn an_absent_device_is_a_finding_naming_the_device() {
    let outcome: super::CheckOutcome = check_device(&device(DeviceState::Absent));
    let super::CheckOutcome::Finding { field, detail } = outcome else {
        panic!("an absent device is a finding");
    };
    assert_eq!(field, "device");
    assert!(detail.contains("/dev/zfs"), "{detail}");
}

/// A device this account may not open is a finding naming the open.
#[test]
fn a_denied_open_is_a_finding_naming_the_open() {
    let outcome: super::CheckOutcome = check_device(&device(DeviceState::Denied { why: String::from("Permission denied (os error 13)") }));
    let super::CheckOutcome::Finding { field, detail } = outcome else {
        panic!("a refused open is a finding");
    };
    assert_eq!(field, "open");
    assert!(detail.contains("13"), "{detail}");
}

/// A look the caller was refused leaves the answer unknown.
#[test]
fn an_unreadable_device_is_unknown() {
    let outcome: super::CheckOutcome = check_device(&device(DeviceState::Unreadable { why: String::from("stat refused") }));
    assert!(matches!(outcome, super::CheckOutcome::Unknown { .. }), "{outcome:?}");
}

//endregion

//region The ABI verdict

/// A compatible layout passes with the module's release text.
#[test]
fn a_compatible_layout_passes() {
    let outcome: super::CheckOutcome = check_abi(&AbiFacts::Compatible { module: Some(String::from("2.4.4-1")), details: vec![String::from("module type information 99 bytes")] });
    let super::CheckOutcome::Pass { detail } = outcome else {
        panic!("a compatible layout passes");
    };
    assert!(detail.contains("2.4.4-1"), "{detail}");
}

/// A refused comparison is a finding naming the layout.
#[test]
fn a_refused_comparison_is_a_finding_naming_the_layout() {
    let outcome: super::CheckOutcome = check_abi(&AbiFacts::Refused { words: String::from("member refused zfs_cmd.zc_objset_stats.dds_flags") });
    let super::CheckOutcome::Finding { field, detail } = outcome else {
        panic!("a refusal is a finding");
    };
    assert_eq!(field, "module-layout");
    assert!(detail.contains("no flag or environment"), "{detail}");
}

/// A gate that could not compare leaves the answer unknown.
#[test]
fn a_gate_without_a_comparison_is_unknown() {
    let outcome: super::CheckOutcome = check_abi(&AbiFacts::Unreadable { why: String::from("base type information is not readable") });
    assert!(matches!(outcome, super::CheckOutcome::Unknown { .. }), "{outcome:?}");
}

//endregion

//region The pool

/// A pool that answers passes with its GUID.
#[test]
fn a_pool_that_answers_passes() {
    let outcome: super::CheckOutcome = check_pool(&PoolFacts { pool: String::from("zpcachyos"), reading: Reading::Read(9) });
    let super::CheckOutcome::Pass { detail } = outcome else {
        panic!("an answered pool passes");
    };
    assert!(detail.contains("GUID 9"), "{detail}");
}

/// A pool that is not there is a finding naming the pool.
#[test]
fn a_missing_pool_is_a_finding_naming_the_pool() {
    let outcome: super::CheckOutcome = check_pool(&PoolFacts { pool: String::from("zpcachyos"), reading: Reading::Missing { why: String::from("dataset does not exist") } });
    let super::CheckOutcome::Finding { field, detail } = outcome else {
        panic!("a missing pool is a finding");
    };
    assert_eq!(field, "pool");
    assert!(detail.contains("zpcachyos"), "{detail}");
}

/// A pool this run could not read leaves the answer unknown.
#[test]
fn an_unreadable_pool_is_unknown() {
    let outcome: super::CheckOutcome = check_pool(&PoolFacts { pool: String::from("zpcachyos"), reading: Reading::Unreadable { why: String::from("the device refused the request") } });
    assert!(matches!(outcome, super::CheckOutcome::Unknown { .. }), "{outcome:?}");
}

//endregion

//region The datasets

/// The design's checkout passes.
#[test]
fn the_designs_checkout_passes() {
    let facts: DatasetFacts = dataset(DatasetRole::Checkout { mount_point: PathBuf::from(crate::health::doctor_zfs_route::route_test_facts::CHECKOUT) }, DatasetBuilder::checkout().build());
    assert!(matches!(check_dataset(&facts), super::CheckOutcome::Pass { .. }));
}

/// A checkout mounted elsewhere is a finding naming the mountpoint.
#[test]
fn a_checkout_at_another_path_is_a_finding_naming_the_mountpoint() {
    let facts: DatasetFacts = dataset(DatasetRole::Checkout { mount_point: PathBuf::from(crate::health::doctor_zfs_route::route_test_facts::CHECKOUT) }, DatasetBuilder::checkout().mountpoint("/home/user/elsewhere").build());
    let super::CheckOutcome::Finding { field, .. } = check_dataset(&facts) else {
        panic!("a checkout at another path is a finding");
    };
    assert_eq!(field, "mountpoint");
}

/// A dataset that is not there is a finding naming the dataset.
#[test]
fn a_missing_dataset_is_a_finding_naming_the_dataset() {
    let facts: DatasetFacts = DatasetFacts { role: DatasetRole::Container, name: String::from(CONTAINER_DATASET), reading: Reading::Missing { why: String::from("dataset does not exist") } };
    let super::CheckOutcome::Finding { field, detail } = check_dataset(&facts) else {
        panic!("a missing dataset is a finding");
    };
    assert_eq!(field, "dataset");
    assert!(detail.contains(CONTAINER_DATASET), "{detail}");
}

/// A dataset this run could not read leaves the answer unknown.
#[test]
fn an_unreadable_dataset_is_unknown() {
    let facts: DatasetFacts = DatasetFacts { role: DatasetRole::Container, name: String::from(CONTAINER_DATASET), reading: Reading::Unreadable { why: String::from("permission denied") } };
    assert!(matches!(check_dataset(&facts), super::CheckOutcome::Unknown { .. }));
}

/// A container that could mount itself is a finding naming canmount.
#[test]
fn a_container_that_could_mount_itself_is_a_finding_naming_canmount() {
    let facts: DatasetFacts = dataset(DatasetRole::Container, DatasetBuilder::container().canmount(1).build());
    let super::CheckOutcome::Finding { field, detail } = check_dataset(&facts) else {
        panic!("a container that can mount is a finding");
    };
    assert_eq!(field, "canmount");
    assert!(detail.contains("off"), "{detail}");
}

/// A container whose mountpoint is not legacy is a finding naming the
/// mountpoint.
#[test]
fn a_container_that_is_not_legacy_is_a_finding_naming_the_mountpoint() {
    let facts: DatasetFacts = dataset(DatasetRole::Container, DatasetBuilder::container().mountpoint("/mnt").build());
    let super::CheckOutcome::Finding { field, .. } = check_dataset(&facts) else {
        panic!("a container at a path is a finding");
    };
    assert_eq!(field, "mountpoint");
}

/// The design's clone passes.
#[test]
fn the_designs_clone_passes() {
    let facts: DatasetFacts = dataset(DatasetRole::Clone, DatasetBuilder::clone().build());
    assert!(matches!(check_dataset(&facts), super::CheckOutcome::Pass { .. }));
}

/// A clone carrying setuid is a finding naming setuid.
#[test]
fn a_clone_carrying_setuid_is_a_finding_naming_setuid() {
    let facts: DatasetFacts = dataset(DatasetRole::Clone, DatasetBuilder::clone().setuid(1).build());
    let super::CheckOutcome::Finding { field, .. } = check_dataset(&facts) else {
        panic!("a clone with setuid is a finding");
    };
    assert_eq!(field, "setuid");
}

/// A clone carrying devices is a finding naming devices.
#[test]
fn a_clone_carrying_devices_is_a_finding_naming_devices() {
    let facts: DatasetFacts = dataset(DatasetRole::Clone, DatasetBuilder::clone().devices(1).build());
    let super::CheckOutcome::Finding { field, .. } = check_dataset(&facts) else {
        panic!("a clone with devices is a finding");
    };
    assert_eq!(field, "devices");
}

/// A snapshot read under a dataset's name is a finding naming the object type.
#[test]
fn a_snapshot_where_a_filesystem_belongs_is_a_finding() {
    let facts: DatasetFacts = dataset(DatasetRole::Checkout { mount_point: PathBuf::from(crate::health::doctor_zfs_route::route_test_facts::CHECKOUT) }, DatasetBuilder::checkout().object_type(crate::health::doctor_zfs_route::zfs_checks::FILESYSTEM_OBJECT_SET, true).build());
    let super::CheckOutcome::Finding { field, .. } = check_dataset(&facts) else {
        panic!("a snapshot is a finding");
    };
    assert_eq!(field, "object-type");
}

//endregion

//region The keys

/// Unencrypted datasets have no key to load and pass.
#[test]
fn unencrypted_datasets_pass_the_key_check() {
    let datasets: Vec<DatasetFacts> = crate::health::doctor_zfs_route::route_test_facts::healthy_datasets();
    assert!(matches!(check_key(&datasets), super::CheckOutcome::Pass { .. }));
}

/// An encrypted dataset whose key is loaded passes.
#[test]
fn an_encrypted_dataset_with_its_key_passes() {
    let datasets: Vec<DatasetFacts> = vec![dataset(DatasetRole::Container, DatasetBuilder::container().encrypted(0).build())];
    assert!(matches!(check_key(&datasets), super::CheckOutcome::Pass { .. }));
}

/// An encrypted dataset whose key is not loaded is a finding naming keystatus.
#[test]
fn an_unloaded_key_is_a_finding_naming_keystatus() {
    let datasets: Vec<DatasetFacts> = vec![dataset(DatasetRole::Container, DatasetBuilder::container().encrypted(KEY_UNAVAILABLE).build())];
    let super::CheckOutcome::Finding { field, detail } = check_key(&datasets) else {
        panic!("an unloaded key is a finding");
    };
    assert_eq!(field, "keystatus");
    assert!(detail.contains("not loaded"), "{detail}");
}

/// A dataset whose key state could not be read leaves the answer unknown.
#[test]
fn an_unreadable_key_state_is_unknown() {
    let datasets: Vec<DatasetFacts> = vec![DatasetFacts { role: DatasetRole::Container, name: String::from(CONTAINER_DATASET), reading: Reading::Unreadable { why: String::from("permission denied") } }];
    assert!(matches!(check_key(&datasets), super::CheckOutcome::Unknown { .. }));
}

//endregion

//region The clones' properties together

/// Every healthy clone passes with a count.
#[test]
fn healthy_clones_pass_with_a_count() {
    let clones: Vec<DatasetFacts> = vec![dataset(DatasetRole::Clone, DatasetBuilder::clone().build()), read(CLONE_DATASET, DatasetRole::Clone, DatasetBuilder::clone().build())];
    let outcome: super::CheckOutcome = check_clone_properties(&clones);
    let super::CheckOutcome::Pass { detail } = outcome else {
        panic!("healthy clones pass");
    };
    assert!(detail.contains('2'), "{detail}");
}

/// A broken clone is a finding that names it beside a healthy one.
#[test]
fn one_broken_clone_is_a_finding_beside_a_healthy_one() {
    let clones: Vec<DatasetFacts> = vec![dataset(DatasetRole::Clone, DatasetBuilder::clone().build()), read("pool/one-broken", DatasetRole::Clone, DatasetBuilder::clone().devices(1).build())];
    let super::CheckOutcome::Finding { field, detail } = check_clone_properties(&clones) else {
        panic!("one broken clone is a finding");
    };
    assert_eq!(field, "devices");
    assert!(detail.contains("pool/one-broken") || detail.contains(CLONE_DATASET), "{detail}");
}

/// A clone this run could not read leaves the answer unknown when none is
/// broken.
#[test]
fn an_unreadable_clone_is_unknown() {
    let clones: Vec<DatasetFacts> = vec![DatasetFacts { role: DatasetRole::Clone, name: String::from(CLONE_DATASET), reading: Reading::Unreadable { why: String::from("permission denied") } }];
    assert!(matches!(check_clone_properties(&clones), super::CheckOutcome::Unknown { .. }));
}

//endregion

//region The reading rule

/// `ENOENT` is the one answer that proves a dataset is gone.
#[test]
fn only_enotent_proves_a_dataset_is_gone() {
    let gone: Reading<u8> = reading_of(Err(ReadFailure { said: String::from("no such dataset"), errno: Some(NOT_FOUND_ERRNO) }));
    assert!(matches!(gone, Reading::Missing { .. }), "{gone:?}");
    let refused: Reading<u8> = reading_of(Err(ReadFailure { said: String::from("permission denied"), errno: Some(13) }));
    assert!(matches!(refused, Reading::Unreadable { .. }), "{refused:?}");
    let no_errno: Reading<u8> = reading_of(Err(ReadFailure { said: String::from("the reply had no properties"), errno: None }));
    assert!(matches!(no_errno, Reading::Unreadable { .. }), "{no_errno:?}");
    let read: Reading<u8> = reading_of(Ok(7));
    assert_eq!(read, Reading::Read(7));
}

/// The container's decided `canmount` is off.
#[test]
fn the_decided_canmount_is_off() {
    assert_eq!(CANMOUNT_OFF, 0);
}

//endregion
