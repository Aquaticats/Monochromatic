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
    BATCH_ARM, CHECKOUT_SNAPSHOT, DEPENDENT_ARM, DEPENDENT_CLONE, FREE_SNAPSHOT, OWN_SNAPSHOTS,
    PINNED_SNAPSHOT, PROMOTED_BATCH_ARM, PROMOTED_DEPENDENT_CLONE, PROPERTY_ARM, READ_ARM,
    REPLACEMENT_ARM,
};
use crate::kernel_device::DatasetIdentity;
use crate::nvlist::NvList;

/// The guest account the arm runs as,
/// or `None` on a host run,
/// where no live
/// request may be made.
///
/// # Panics
///
/// When the guest is named but this process is not the delegated account:
/// an arm that
/// cannot run in the guest must fail,
/// not look like a passing test that skipped its
/// assertions.
fn delegated_fixture(test: &str) -> Option<Fixture> {
    let fixture: Fixture = Fixture::from_environment(test)?;
    let (uid, _) = fixture.account_ids();
    assert_eq!(
        Fixture::current_uid(),
        uid,
        "{test}: this process is uid {} and the delegated arm runs as the fixture account \
         '{}' (uid {uid}); the recipe's account suite runs it as that account, and the root suite \
         excludes it",
        Fixture::current_uid(),
        fixture.account
    );
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
    assert_eq!(
        identity.stats.origin(),
        format!("{}@{CHECKOUT_SNAPSHOT}", pool.checkout)
    );
    assert_eq!(property_u64(&identity.properties, "recordsize"), 32768);
    assert_eq!(property_u64(&identity.properties, "defer_destroy"), 0);
    let names: Vec<String> = listing_names(|cursor| {
        return device.snapshot_list_next(&read, cursor);
    });
    let mut expected: Vec<String> = OWN_SNAPSHOTS
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
        PROMOTED_BATCH_ARM,
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
    let first: String = format!("{batch}@{}", OWN_SNAPSHOTS[0]);
    let second: String = format!("{batch}@{}", OWN_SNAPSHOTS[1]);
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
fn the_account_destroys_its_own_snapshots_on_a_promoted_clone() {
    let Some(fixture) =
        delegated_fixture("live_account_tests::the_account_destroys_two_own_snapshots_promoted")
    else {
        return;
    };
    let account_pool: String = fixture.account_pool.clone();
    let pool: FixturePool<'_> = FixturePool::adopt(&fixture, &account_pool);
    let device: ZfsDevice = ZfsDevice::open().expect("the account opens the device");
    let promoted: String = format!("{}/{PROMOTED_BATCH_ARM}", pool.clones);
    assert_eq!(
        device
            .objset_stats(&promoted)
            .expect("the account reads the promoted clone")
            .stats
            .origin(),
        "",
        "the arm names a promoted clone, which has no origin"
    );
    let before: Vec<String> = listing_names(|cursor| {
        return device.snapshot_list_next(&promoted, cursor);
    });
    let own: Vec<String> = OWN_SNAPSHOTS
        .iter()
        .map(|name| {
            return format!("{promoted}@{name}");
        })
        .collect();
    for name in &own {
        assert!(
            before.contains(name),
            "the promoted clone holds {name} before the batch: {before:?}"
        );
    }
    let reply: Option<NvList> = device
        .destroy_snaps(&pool.name, &[own[0].as_str(), own[1].as_str()])
        .expect("the grants let the account destroy its clone's own snapshots");
    let carried = reply
        .as_ref()
        .map_or_else(Vec::new, crate::requests::element_errors);
    assert!(
        carried.is_empty(),
        "the destroyed set carries no per-name failure: {carried:?}"
    );
    let after: Vec<String> = listing_names(|cursor| {
        return device.snapshot_list_next(&promoted, cursor);
    });
    assert!(
        after.iter().all(|name| {
            return !own.contains(name);
        }),
        "both own names are gone: {after:?}"
    );
    for name in before.iter().filter(|name| {
        return !own.contains(name);
    }) {
        assert!(
            after.contains(name),
            "the batch left {name}, including the history the promotion moved: {after:?}"
        );
    }
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_account_batch_with_a_dependent_clone_on_a_promoted_clone_keeps_both_names() {
    let Some(fixture) = delegated_fixture("live_account_tests::the_account_mixed_batch_promoted")
    else {
        return;
    };
    let account_pool: String = fixture.account_pool.clone();
    let pool: FixturePool<'_> = FixturePool::adopt(&fixture, &account_pool);
    let device: ZfsDevice = ZfsDevice::open().expect("the account opens the device");
    let promoted: String = format!("{}/{PROMOTED_BATCH_ARM}", pool.clones);
    assert_eq!(
        device
            .objset_stats(&format!("{}/{PROMOTED_DEPENDENT_CLONE}", pool.clones))
            .expect("the dependent clone is readable")
            .stats
            .origin(),
        format!("{promoted}@{PINNED_SNAPSHOT}")
    );
    let pinned: String = format!("{promoted}@{PINNED_SNAPSHOT}");
    let free: String = format!("{promoted}@{FREE_SNAPSHOT}");
    let before: Vec<String> = listing_names(|cursor| {
        return device.snapshot_list_next(&promoted, cursor);
    });
    let error = device
        .destroy_snaps(&pool.name, &[pinned.as_str(), free.as_str()])
        .expect_err("the dependent clone refuses the whole set");
    let Some(failure) = error.request() else {
        panic!("the refusal is a request failure: {error}");
    };
    assert_eq!(failure.errno(), Some(libc::EEXIST));
    assert_eq!(failure.elements().len(), 1);
    assert_eq!(failure.elements()[0].name, pinned);
    let after: Vec<String> = listing_names(|cursor| {
        return device.snapshot_list_next(&promoted, cursor);
    });
    let mut kept: Vec<String> = after.clone();
    let mut wanted: Vec<String> = before;
    kept.sort();
    wanted.sort();
    assert_eq!(kept, wanted, "a refused batch keeps every name");
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
        .clone(
            &target,
            &format!("{}@{CHECKOUT_SNAPSHOT}", pool.checkout),
            None,
        )
        .expect("the account clones the same name again");
    let after: DatasetIdentity = device
        .objset_stats(&target)
        .expect("the replacement is read");
    assert_ne!(before.stats.dds_guid, after.stats.dds_guid);
}
