//! What:
//!  The live writes that do not batch:
//! a snapshot set one request creates with a
//!  user property,
//! a clone's accepted and refused
//!  creation-time user properties,
//! a post-create `SET_PROP`,
//! a head destroy and
//!  the name reused afterwards,
//! and a clone taken while the pool's key is unloaded.
//! Why:
//!  The design asks for native name-value list encoding to be exercised on
//!  creation and after it,
//! and for a refused property to leave nothing behind.
//!  The snapshot request is the add route's first write,
//! so its creation and the
//!  reply it answers with are exercised here too.
//! A clone under an unloaded key is the measurement's item 5 repeated through
//!  the binding:
//! the key decides mount readiness,
//! never dataset identity.

use super::ZfsDevice;
use super::fixture::Fixture;
use super::fixture::properties;
use super::fixture_pool::FixturePool;
use super::fixture_reply::{cli_value, listing_names, property_source, property_str, property_u64};
use crate::nvlist::NvList;

/// `ZFS_KEYSTATUS_UNAVAILABLE` (`include/sys/fs/zfs.h` at `zfs-2.4.4`).
const KEYSTATUS_UNAVAILABLE: u64 = 1;
/// `ZFS_KEYSTATUS_AVAILABLE` (`include/sys/fs/zfs.h` at `zfs-2.4.4`).
const KEYSTATUS_AVAILABLE: u64 = 2;

/// The snapshot name the root arm creates through the binding.
const SNAPSHOT_NAME: &str = "from-binding";
/// The user property the root arm sets at snapshot creation.
const SNAPSHOT_PROPERTY: (&str, &str) = ("monochromatic:request", "snapshot");
/// `ZAP_MAXVALUELEN` (`include/sys/fs/zfs.h` at `zfs-2.4.4`),
/// the value length
/// `zfs_check_userprops` refuses with `E2BIG` at or past.
const PROPERTY_VALUE_LIMIT: usize = 1024 * 8;

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn a_snapshot_set_with_a_user_property_creates_each_and_answers_with_no_failure() {
    let Some(fixture) =
        Fixture::from_environment("live_write_tests::a_snapshot_set_with_a_user_property")
    else {
        return;
    };
    let pool: FixturePool<'_> = FixturePool::create(&fixture, "zfsstep2-snapshot");
    pool.assert_the_module_release();
    pool.snapshot(&pool.checkout, &["origin"]);
    let clone: String = pool.clone_of(&format!("{}@origin", pool.checkout), "snapshot-pair");
    let device: ZfsDevice = ZfsDevice::open().expect("the fixture has the device");
    // One snapshot of each dataset: the kernel refuses two snapshots of one
    // filesystem in one request (`zfs_ioc_snapshot`, `module/zfs/zfs_ioctl.c` of
    // `zfs-2.4.4`).
    let on_checkout: String = format!("{}@{SNAPSHOT_NAME}", pool.checkout);
    let on_clone: String = format!("{clone}@{SNAPSHOT_NAME}");
    let wanted: NvList = properties(&[SNAPSHOT_PROPERTY]);
    let reply: Option<NvList> = device
        .snapshot(
            &pool.name,
            &[on_checkout.as_str(), on_clone.as_str()],
            Some(&wanted),
        )
        .expect("the kernel creates one snapshot of each dataset");
    let carried: Vec<crate::requests::ElementError> = reply
        .as_ref()
        .map_or_else(Vec::new, crate::requests::element_errors);
    assert!(
        carried.is_empty(),
        "the created snapshots carry no per-name failure: {carried:?}"
    );
    for name in [&on_checkout, &on_clone] {
        let identity: crate::kernel_device::DatasetIdentity =
            device.objset_stats(name).expect("the new snapshot is read");
        assert_ne!(identity.stats.dds_is_snapshot, 0, "{name} is a snapshot");
        assert_eq!(
            property_u64(&identity.properties, "guid"),
            cli_value(&fixture, name, "guid")
        );
        assert_eq!(
            property_str(&identity.properties, SNAPSHOT_PROPERTY.0),
            SNAPSHOT_PROPERTY.1
        );
        assert_eq!(
            property_source(&identity.properties, SNAPSHOT_PROPERTY.0),
            name.as_str(),
            "the property was set on the snapshot itself"
        );
    }
    let listed: Vec<String> = listing_names(|cursor| {
        return device.snapshot_list_next(&pool.checkout, cursor);
    });
    assert!(
        listed.contains(&on_checkout),
        "the checkout lists the snapshot the request created: {listed:?}"
    );
    println!(
        "MEASUREMENT snapshot-ioctl created=2 carried={} names={on_checkout},{on_clone}",
        carried.len()
    );
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn a_creation_time_user_property_is_accepted_and_reads_back() {
    let Some(fixture) =
        Fixture::from_environment("live_write_tests::a_creation_time_user_property")
    else {
        return;
    };
    let pool: FixturePool<'_> = FixturePool::create(&fixture, "zfsstep2-create-props");
    pool.assert_the_module_release();
    pool.snapshot(&pool.checkout, &["origin"]);
    let device: ZfsDevice = ZfsDevice::open().expect("the fixture has the device");
    let target: String = format!("{}/with-props", pool.clones);
    let origin: String = format!("{}@origin", pool.checkout);
    let wanted: NvList = properties(&[("monochromatic:desired-state", "mounted")]);
    device
        .clone(&target, &origin, Some(&wanted))
        .expect("the native list the clone carries is accepted");
    let reply: NvList = device
        .objset_stats(&target)
        .expect("the new clone is read")
        .properties;
    assert_eq!(
        property_str(&reply, "monochromatic:desired-state"),
        "mounted"
    );
    assert_eq!(
        property_source(&reply, "monochromatic:desired-state"),
        target
    );
    assert_eq!(property_u64(&reply, "defer_destroy"), 0);
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn a_creation_time_user_property_past_the_value_limit_is_refused_and_creates_nothing() {
    let Some(fixture) =
        Fixture::from_environment("live_write_tests::a_creation_time_user_property_past_the_limit")
    else {
        return;
    };
    let pool: FixturePool<'_> = FixturePool::create(&fixture, "zfsstep2-refused-props");
    pool.assert_the_module_release();
    pool.snapshot(&pool.checkout, &["origin"]);
    let device: ZfsDevice = ZfsDevice::open().expect("the fixture has the device");
    let target: String = format!("{}/refused-props", pool.clones);
    let origin: String = format!("{}@origin", pool.checkout);
    // A real user property, past the value length the kernel accepts: `E2BIG`,
    // not the `EINVAL` an unknown property name gets.
    let too_long: String = "v".repeat(PROPERTY_VALUE_LIMIT);
    let refused: NvList = properties(&[("monochromatic:too-long", too_long.as_str())]);
    let error = device
        .clone(&target, &origin, Some(&refused))
        .expect_err("the kernel refuses a user property value past its limit");
    assert_eq!(error.errno(), Some(libc::E2BIG));
    assert!(!pool.dataset_exists(&target));
    assert!(
        error.to_string().contains("CLONE"),
        "the refusal names the request: {error}"
    );
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn a_post_create_set_prop_sets_the_user_property() {
    let Some(fixture) = Fixture::from_environment("live_write_tests::a_post_create_set_prop")
    else {
        return;
    };
    let pool: FixturePool<'_> = FixturePool::create(&fixture, "zfsstep2-set-prop");
    pool.assert_the_module_release();
    pool.snapshot(&pool.checkout, &["origin"]);
    let clone: String = pool.clone_of(&format!("{}@origin", pool.checkout), "sweep-nudge");
    let device: ZfsDevice = ZfsDevice::open().expect("the fixture has the device");
    device
        .set_prop(
            &clone,
            &properties(&[("monochromatic:sweep-nonce", "2026-11-19T02:00:00-05:00")]),
        )
        .expect("the property set is accepted");
    let reply: NvList = device
        .objset_stats(&clone)
        .expect("the clone is read")
        .properties;
    assert_eq!(
        property_str(&reply, "monochromatic:sweep-nonce"),
        "2026-11-19T02:00:00-05:00"
    );
    assert_eq!(property_source(&reply, "monochromatic:sweep-nonce"), clone);
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn a_head_destroy_removes_the_clone_and_the_name_can_be_reused() {
    let Some(fixture) =
        Fixture::from_environment("live_write_tests::a_head_destroy_removes_the_clone")
    else {
        return;
    };
    let pool: FixturePool<'_> = FixturePool::create(&fixture, "zfsstep2-replacement");
    pool.assert_the_module_release();
    pool.snapshot(&pool.checkout, &["origin"]);
    let device: ZfsDevice = ZfsDevice::open().expect("the fixture has the device");
    let target: String = format!("{}/replacement", pool.clones);
    let origin: String = format!("{}@origin", pool.checkout);
    device
        .clone(&target, &origin, None)
        .expect("the first clone is made");
    let before: u64 = property_u64(
        &device
            .objset_stats(&target)
            .expect("the clone is read")
            .properties,
        "guid",
    );
    device
        .destroy(&target, false)
        .expect("the unmounted clone is destroyed");
    assert!(!pool.dataset_exists(&target));
    device
        .clone(&target, &origin, None)
        .expect("the same name is free again");
    let after: u64 = property_u64(
        &device
            .objset_stats(&target)
            .expect("the replacement is read")
            .properties,
        "guid",
    );
    assert_ne!(before, after, "a reused name is a different object");
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn a_clone_under_an_unloaded_key_succeeds_and_reports_the_key_state() {
    let Some(fixture) =
        Fixture::from_environment("live_write_tests::a_clone_under_an_unloaded_key")
    else {
        return;
    };
    let pool: FixturePool<'_> = FixturePool::create(&fixture, "zfsstep2-key");
    pool.assert_the_module_release();
    pool.snapshot(&pool.checkout, &["origin"]);
    let device: ZfsDevice = ZfsDevice::open().expect("the fixture has the device");
    let origin: String = format!("{}@origin", pool.checkout);
    // The key belongs to the pool's root,
    // and the kernel refuses to unload it while a
    // dataset under it is mounted,
    // so the child is unmounted first: `zfs unmount` has no
    // recursive flag on this release.
    fixture.zfs(&["unmount", "-f", &pool.checkout]);
    fixture.zfs(&["unmount", "-f", &pool.home]);
    fixture.zfs(&["unload-key", &pool.name]);
    assert_eq!(
        property_u64(
            &device
                .objset_stats(&pool.checkout)
                .expect("the checkout is read")
                .properties,
            "keystatus"
        ),
        KEYSTATUS_UNAVAILABLE
    );
    let target: String = format!("{}/key-unavailable", pool.clones);
    device
        .clone(&target, &origin, None)
        .expect("a clone of an encrypted dataset needs no loaded key");
    let identity: crate::kernel_device::DatasetIdentity =
        device.objset_stats(&target).expect("the clone is read");
    assert_eq!(
        property_u64(&identity.properties, "keystatus"),
        KEYSTATUS_UNAVAILABLE
    );
    assert_eq!(identity.stats.origin(), origin);
    fixture.zfs(&["load-key", &pool.name]);
    assert_eq!(
        property_u64(
            &device
                .objset_stats(&target)
                .expect("the clone is read again")
                .properties,
            "keystatus"
        ),
        KEYSTATUS_AVAILABLE
    );
}
