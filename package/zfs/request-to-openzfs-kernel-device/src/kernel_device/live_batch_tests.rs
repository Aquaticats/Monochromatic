//! What:
//!  The live snapshot batches:
//! a mixed set one name with a dependent clone refuses,
//!  with per-name errors and every name kept,
//! an own-snapshot set destroys
//!  both names in one request while the checkout's history keeps its
//!  identity,
//!  and both sets again on a promoted clone,
//! where the checkout's older
//!  history has moved into the dataset the batch names.
//! Why:
//!  Removal destroys a clone's own snapshots in one request and must know
//!  exactly which name survived;
//! items 14,
//! 17 and 18 measured that through a helper,
//! item 17 on a
//!  promoted clone and item 18 on an unpromoted one,
//! and these arms measure the
//!  same requests through the binding,
//! the delegated ones in the account arm beside
//!  it.
//! A batch that only ever runs on an unpromoted clone never exercises the
//!  transferred snapshots the promotion leaves in the dataset a removal
//!  destroys.

use super::ZfsDevice;
use super::fixture::Fixture;
use super::fixture_pool::FixturePool;
use super::fixture_reply::listing_names;
use super::write_requests::Promotion;
use crate::requests::Operation;

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn a_mixed_batch_keeps_every_name_and_names_the_dependent_one() {
    let Some(fixture) = Fixture::from_environment("live_batch_tests::a_mixed_batch") else {
        return;
    };
    let pool: FixturePool<'_> = FixturePool::create(&fixture, "zfsstep2-mixed-batch");
    pool.assert_the_module_release();
    pool.snapshot(&pool.checkout, &["earliest", "origin", "checkout-later"]);
    let removal: String = pool.clone_of(&format!("{}@origin", pool.checkout), "removal");
    pool.snapshot(&removal, &["with-dependent", "without-dependent"]);
    let dependent_origin: String = format!("{removal}@with-dependent");
    let dependent: String = pool.clone_of(&dependent_origin, "dependent");
    assert_eq!(
        cli_origin(&fixture, &dependent),
        dependent_origin,
        "the dependent clone is what makes the name undestroyable"
    );
    let checkout_before: Vec<(String, u64, u64)> = pool.identity_rows("snapshot", &pool.checkout);
    let device: ZfsDevice = ZfsDevice::open().expect("the fixture has the device");
    let dependent_name: String = format!("{removal}@with-dependent");
    let free_name: String = format!("{removal}@without-dependent");
    let error = device
        .destroy_snaps(&pool.name, &[dependent_name.as_str(), free_name.as_str()])
        .expect_err("a name with a dependent clone refuses the whole set");
    let Some(failure) = error.request() else {
        panic!("the refusal is a request failure: {error}");
    };
    assert_eq!(failure.operation(), Operation::DestroySnaps);
    assert_eq!(failure.errno(), Some(libc::EEXIST));
    assert_eq!(failure.elements().len(), 1);
    assert_eq!(failure.elements()[0].name, dependent_name);
    assert_eq!(failure.elements()[0].errno, libc::EEXIST);
    let names: Vec<String> = listing_names(|cursor| {
        return device.snapshot_list_next(&removal, cursor);
    });
    let mut kept: Vec<String> = names;
    let mut wanted: Vec<String> = vec![dependent_name.clone(), free_name.clone()];
    kept.sort();
    wanted.sort();
    assert_eq!(kept, wanted, "a refused batch keeps every name");
    assert_eq!(
        pool.identity_rows("snapshot", &pool.checkout),
        checkout_before
    );
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn an_own_snapshot_batch_destroys_both_and_keeps_the_checkout_history() {
    let Some(fixture) = Fixture::from_environment("live_batch_tests::an_own_snapshot_batch") else {
        return;
    };
    let pool: FixturePool<'_> = FixturePool::create(&fixture, "zfsstep2-own-batch");
    pool.assert_the_module_release();
    pool.snapshot(&pool.checkout, &["earliest", "origin", "checkout-later"]);
    let removal: String = pool.clone_of(&format!("{}@origin", pool.checkout), "removal");
    pool.snapshot(&removal, &["own-one", "own-two"]);
    let checkout_before: Vec<(String, u64, u64)> = pool.identity_rows("snapshot", &pool.checkout);
    let own_before: Vec<(String, u64, u64)> = pool.identity_rows("snapshot", &removal);
    let device: ZfsDevice = ZfsDevice::open().expect("the fixture has the device");
    let first: String = format!("{removal}@own-one");
    let second: String = format!("{removal}@own-two");
    device
        .destroy_snaps(&pool.name, &[first.as_str(), second.as_str()])
        .expect("both own snapshots are destroyed in one request");
    let names: Vec<String> = listing_names(|cursor| {
        return device.snapshot_list_next(&removal, cursor);
    });
    assert!(
        names.is_empty(),
        "the clone has no snapshots left: {names:?}"
    );
    assert_eq!(own_before.len(), 2);
    assert_eq!(
        pool.identity_rows("snapshot", &pool.checkout),
        checkout_before
    );
}

/// The origin a clone reports,
/// read from `zfs get`,
/// which the fixture uses as its own
/// control of the dependency the batch refuses.
fn cli_origin(fixture: &Fixture, dataset: &str) -> String {
    let printed: String = fixture.zfs(&["get", "-H", "-o", "value", "origin", dataset]);
    return printed.trim().to_string();
}

/// The snapshot names a dataset holds,
/// in the kernel's own order.
fn snapshots(device: &ZfsDevice, dataset: &str) -> Vec<String> {
    return listing_names(|cursor| {
        return device.snapshot_list_next(dataset, cursor);
    });
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn a_promoted_clone_destroys_its_own_snapshots_and_refuses_the_dependent_one() {
    let Some(fixture) = Fixture::from_environment("live_batch_tests::a_promoted_batch") else {
        return;
    };
    let pool: FixturePool<'_> = FixturePool::create(&fixture, "zfsstep2-promoted-batch");
    pool.assert_the_module_release();
    pool.snapshot(&pool.checkout, &["earliest", "origin", "checkout-later"]);
    let removal: String = pool.clone_of(&format!("{}@origin", pool.checkout), "removal");
    pool.snapshot(&removal, &["own-one", "own-two"]);
    let device: ZfsDevice = ZfsDevice::open().expect("the fixture has the device");
    assert!(matches!(device.promote(&removal), Promotion::Promoted));
    let moved_history: Vec<(String, u64, u64)> = pool.identity_rows("snapshot", &removal);
    pool.snapshot(&removal, &["with-dependent", "without-dependent"]);
    let dependent: String = pool.clone_of(&format!("{removal}@with-dependent"), "dependent");
    assert_eq!(
        cli_origin(&fixture, &dependent),
        format!("{removal}@with-dependent"),
        "the dependent clone is what makes the name undestroyable"
    );

    // The mixed set first: it must leave every name, including the own ones.
    let before: Vec<String> = snapshots(&device, &removal);
    let dependent_name: String = format!("{removal}@with-dependent");
    let free_name: String = format!("{removal}@without-dependent");
    let error = device
        .destroy_snaps(&pool.name, &[dependent_name.as_str(), free_name.as_str()])
        .expect_err("a name with a dependent clone refuses the whole set");
    let Some(failure) = error.request() else {
        panic!("the refusal is a request failure: {error}");
    };
    assert_eq!(failure.operation(), Operation::DestroySnaps);
    assert_eq!(failure.errno(), Some(libc::EEXIST));
    assert_eq!(failure.elements().len(), 1);
    assert_eq!(failure.elements()[0].name, dependent_name);
    let mut kept: Vec<String> = snapshots(&device, &removal);
    let mut wanted: Vec<String> = before.clone();
    kept.sort();
    wanted.sort();
    assert_eq!(kept, wanted, "a refused batch keeps every name");

    // The own set next: the two own names go and the moved history stays.
    let own_first: String = format!("{removal}@own-one");
    let own_second: String = format!("{removal}@own-two");
    let reply: Option<crate::nvlist::NvList> = device
        .destroy_snaps(&pool.name, &[own_first.as_str(), own_second.as_str()])
        .expect("both own snapshots are destroyed in one request");
    let carried = reply
        .as_ref()
        .map_or_else(Vec::new, crate::requests::element_errors);
    assert!(
        carried.is_empty(),
        "the destroyed set carries no per-name failure: {carried:?}"
    );
    let after: Vec<String> = snapshots(&device, &removal);
    assert!(
        !after.contains(&own_first) && !after.contains(&own_second),
        "the two own names are gone: {after:?}"
    );
    for (name, guid, txg) in &moved_history {
        if name.ends_with("@own-one") || name.ends_with("@own-two") {
            continue;
        }
        assert!(
            pool.identity_rows("snapshot", &removal)
                .contains(&(name.clone(), *guid, *txg)),
            "the promoted clone keeps {name} with its GUID and transaction group"
        );
    }
}
