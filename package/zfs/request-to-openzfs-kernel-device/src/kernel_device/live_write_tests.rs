//! What:
//!  The live writes that do not batch:
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
//!  A clone under an unloaded key is the measurement's item 5 repeated through
//!  the binding:
//! the key decides mount readiness,
//! never dataset identity.

use super::ZfsDevice;
use super::fixture::Fixture;
use super::fixture::properties;
use super::fixture_pool::FixturePool;
use super::fixture_reply::{property_source, property_str, property_u64};
use crate::nvlist::NvList;

/// `ZFS_KEYSTATUS_UNAVAILABLE` (`include/sys/fs/zfs.h` at `zfs-2.4.4`).
const KEYSTATUS_UNAVAILABLE: u64 = 1;
/// `ZFS_KEYSTATUS_AVAILABLE` (`include/sys/fs/zfs.h` at `zfs-2.4.4`).
const KEYSTATUS_AVAILABLE: u64 = 2;

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
fn an_unknown_creation_time_property_is_refused_and_creates_nothing() {
    let Some(fixture) =
        Fixture::from_environment("live_write_tests::an_unknown_creation_time_property")
    else {
        return;
    };
    let pool: FixturePool<'_> = FixturePool::create(&fixture, "zfsstep2-refused-props");
    pool.assert_the_module_release();
    pool.snapshot(&pool.checkout, &["origin"]);
    let device: ZfsDevice = ZfsDevice::open().expect("the fixture has the device");
    let target: String = format!("{}/refused-props", pool.clones);
    let origin: String = format!("{}@origin", pool.checkout);
    let refused: NvList = properties(&[("monochromatic", "not-a-user-property")]);
    let error = device
        .clone(&target, &origin, Some(&refused))
        .expect_err("a property without a colon is no property the kernel knows");
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
