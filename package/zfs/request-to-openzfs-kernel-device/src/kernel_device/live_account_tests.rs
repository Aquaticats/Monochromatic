//! What:
//!  The delegated account's arms against the pool the root run prepared:
//! the
//!  identity and snapshot reads,
//! the two-snapshot batch one request destroys,
//! the
//!  batch a dependent clone keeps whole,
//! the requests the grants refuse,
//! the
//!  user property set on its own clone,
//! and the same-name replacement whose
//!  identity changes.
//! Why:
//!  The design runs normal requests as the account with only the proposed
//!  grants,
//! because a root-only pass proves nothing about delegation.
//! Each arm
//!  asserts what the kernel decided for that account,
//! and a refusal names the
//!  permissions it would have needed.

use super::ZfsDevice;
use super::fixture::Fixture;
use super::fixture::properties;
use super::fixture_pool::FixturePool;
use super::fixture_reply::{listing_names, property_str, property_u64};
use super::live_account_fixture_tests::{
    BATCH_ARM, DEPENDENT_ARM, DEPENDENT_CLONE, FREE_SNAPSHOT, PINNED_SNAPSHOT, PROPERTY_ARM,
    READ_ARM, REPLACEMENT_ARM,
};
use crate::kernel_device::DatasetIdentity;
use crate::nvlist::NvList;

/// The guest account the arm runs as,
/// or a skip line naming the reason.
fn delegated_fixture(test: &str) -> Option<Fixture> {
    let fixture: Fixture = Fixture::from_environment(test)?;
    let (uid, _) = fixture.account_ids();
    if Fixture::current_uid() != uid {
        println!(
            "SKIP {test}: this process is uid {} and the delegated arm runs as the fixture \
             account '{}' (uid {uid}); the recipe runs the account arm as that account.",
            Fixture::current_uid(),
            fixture.account
        );
        return None;
    }
    return Some(fixture);
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_account_reads_the_prepared_clones_identity_and_snapshots() {
    let Some(fixture) =
        delegated_fixture("live_account_tests::the_account_reads_the_prepared_clone")
    else {
        return;
    };
    let account_pool: String = fixture.account_pool.clone();
    let pool: FixturePool<'_> = FixturePool::adopt(&fixture, &account_pool);
    let device: ZfsDevice = ZfsDevice::open().expect("the account opens the device");
    let read: String = format!("{}/{READ_ARM}", pool.clones);
    let identity: DatasetIdentity = device
        .objset_stats(&read)
        .expect("the account reads its clone's identity");
    assert_eq!(identity.stats.origin(), format!("{}@origin", pool.checkout));
    assert_eq!(property_u64(&identity.properties, "recordsize"), 32768);
    assert_eq!(property_u64(&identity.properties, "defer_destroy"), 0);
    let names: Vec<String> = listing_names(|cursor| {
        return device.snapshot_list_next(&read, cursor);
    });
    let mut expected: Vec<String> = ["own-one", "own-two"]
        .iter()
        .map(|name| {
            return format!("{read}@{name}");
        })
        .collect();
    let mut listed_names: Vec<String> = names;
    expected.sort();
    listed_names.sort();
    assert_eq!(listed_names, expected);
    let children: Vec<String> = listing_names(|cursor| {
        return device.dataset_list_next(&pool.clones, cursor);
    });
    for arm in [
        READ_ARM,
        BATCH_ARM,
        DEPENDENT_ARM,
        REPLACEMENT_ARM,
        PROPERTY_ARM,
    ] {
        assert!(
            children.contains(&format!("{}/{arm}", pool.clones)),
            "the account lists its prepared clone {arm}: {children:?}"
        );
    }
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_account_destroys_two_own_snapshots_in_one_batch() {
    let Some(fixture) =
        delegated_fixture("live_account_tests::the_account_destroys_two_own_snapshots")
    else {
        return;
    };
    let account_pool: String = fixture.account_pool.clone();
    let pool: FixturePool<'_> = FixturePool::adopt(&fixture, &account_pool);
    let device: ZfsDevice = ZfsDevice::open().expect("the account opens the device");
    let batch: String = format!("{}/{BATCH_ARM}", pool.clones);
    let first: String = format!("{batch}@own-one");
    let second: String = format!("{batch}@own-two");
    device
        .destroy_snaps(&pool.name, &[first.as_str(), second.as_str()])
        .expect("the grants let the account destroy its clone's own snapshots");
    let names: Vec<String> = listing_names(|cursor| {
        return device.snapshot_list_next(&batch, cursor);
    });
    assert!(names.is_empty(), "both names are gone: {names:?}");
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_account_batch_with_a_dependent_clone_keeps_both_names() {
    let Some(fixture) =
        delegated_fixture("live_account_tests::the_account_batch_with_a_dependent_clone")
    else {
        return;
    };
    let account_pool: String = fixture.account_pool.clone();
    let pool: FixturePool<'_> = FixturePool::adopt(&fixture, &account_pool);
    let device: ZfsDevice = ZfsDevice::open().expect("the account opens the device");
    let dependent: String = format!("{}/{DEPENDENT_ARM}", pool.clones);
    assert_eq!(
        device
            .objset_stats(&format!("{}/{DEPENDENT_CLONE}", pool.clones))
            .expect("the dependent clone is readable")
            .stats
            .origin(),
        format!("{dependent}@{PINNED_SNAPSHOT}")
    );
    let pinned: String = format!("{dependent}@{PINNED_SNAPSHOT}");
    let free: String = format!("{dependent}@{FREE_SNAPSHOT}");
    let error = device
        .destroy_snaps(&pool.name, &[pinned.as_str(), free.as_str()])
        .expect_err("the dependent clone refuses the whole set");
    let Some(failure) = error.request() else {
        panic!("the refusal is a request failure: {error}");
    };
    assert_eq!(failure.errno(), Some(libc::EEXIST));
    assert_eq!(failure.elements().len(), 1);
    assert_eq!(failure.elements()[0].name, pinned);
    let names: Vec<String> = listing_names(|cursor| {
        return device.snapshot_list_next(&dependent, cursor);
    });
    let mut kept: Vec<String> = names;
    let mut wanted: Vec<String> = vec![pinned, free];
    kept.sort();
    wanted.sort();
    assert_eq!(kept, wanted, "a refused batch keeps every name");
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_account_cannot_create_a_snapshot_of_the_checkout() {
    let Some(fixture) =
        delegated_fixture("live_account_tests::the_account_cannot_create_a_snapshot")
    else {
        return;
    };
    let account_pool: String = fixture.account_pool.clone();
    let pool: FixturePool<'_> = FixturePool::adopt(&fixture, &account_pool);
    let device: ZfsDevice = ZfsDevice::open().expect("the account opens the device");
    let snapshot: String = format!("{}@from-account", pool.checkout);
    let error = device
        .snapshot(&pool.name, &[snapshot.as_str()], None)
        .expect_err("no grant lets the account snapshot the checkout");
    assert_eq!(error.errno(), Some(libc::EPERM));
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_account_sets_a_user_property_on_its_clone_and_not_on_the_parent() {
    let Some(fixture) = delegated_fixture("live_account_tests::the_account_sets_a_user_property")
    else {
        return;
    };
    let account_pool: String = fixture.account_pool.clone();
    let pool: FixturePool<'_> = FixturePool::adopt(&fixture, &account_pool);
    let device: ZfsDevice = ZfsDevice::open().expect("the account opens the device");
    let own: String = format!("{}/{PROPERTY_ARM}", pool.clones);
    let wanted: NvList = properties(&[("monochromatic:sweep-nonce", "account-run")]);
    device
        .set_prop(&own, &wanted)
        .expect("the descendent userprop grant lets the account set it on its clone");
    let reply: NvList = device
        .objset_stats(&own)
        .expect("the clone is read")
        .properties;
    assert_eq!(
        property_str(&reply, "monochromatic:sweep-nonce"),
        "account-run"
    );
    let refused = device
        .set_prop(&pool.clones, &wanted)
        .expect_err("the parent's userprop grant is for descendants only");
    assert_eq!(refused.errno(), Some(libc::EPERM));
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_account_replaces_a_clone_name_and_gets_a_new_identity() {
    let Some(fixture) = delegated_fixture("live_account_tests::the_account_replaces_a_clone_name")
    else {
        return;
    };
    let account_pool: String = fixture.account_pool.clone();
    let pool: FixturePool<'_> = FixturePool::adopt(&fixture, &account_pool);
    let device: ZfsDevice = ZfsDevice::open().expect("the account opens the device");
    let target: String = format!("{}/{REPLACEMENT_ARM}", pool.clones);
    let before: DatasetIdentity = device.objset_stats(&target).expect("the clone is read");
    device
        .destroy(&target, false)
        .expect("the destroyed grant reaches the clone head");
    device
        .clone(&target, &format!("{}@origin", pool.checkout), None)
        .expect("the account clones the same name again");
    let after: DatasetIdentity = device
        .objset_stats(&target)
        .expect("the replacement is read");
    assert_ne!(before.stats.dds_guid, after.stats.dds_guid);
}
