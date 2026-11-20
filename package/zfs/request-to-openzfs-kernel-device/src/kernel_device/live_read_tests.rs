//! What:
//!  The live reads:
//! the child listing and its cursor,
//! the snapshot listing of an
//!  unpromoted clone,
//! the property values and sources one identity reply
//!  carries,
//! the delegations the fixture granted,
//! and a reply that outgrows the
//!  initial buffer.
//! Why:
//!  The design asks for the cursor,
//! property sources and output resize to be
//!  checked against a real module,
//! because only the kernel decides where a cursor
//!  stops and how big a reply is.
//! Each arm builds its own pool,
//! so a failure
//!  leaves no state another arm depends on.

use super::ZfsDevice;
use super::fixture::Fixture;
use super::fixture_pool::FixturePool;
use super::fixture_reply::{
    cli_value, expected_source, listing_names, property_source, property_str, property_u64,
};
use super::listed_object::ListedObject;
use super::read_reply::ReadAttempt;
use crate::nvlist::NvList;
use crate::requests::{INITIAL_REPLY_SIZE, Request, objset_stats};

/// The bytes each padding property's value holds,
/// so the count that outgrows
/// the initial reply follows from the two sizes.
const PAD_VALUE_BYTES: usize = 3_000;

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn listing_the_clone_parent_returns_every_child_and_then_its_end() {
    let Some(fixture) =
        Fixture::from_environment("live_read_tests::listing_the_clone_parent_returns_every_child")
    else {
        return;
    };
    let pool: FixturePool<'_> = FixturePool::create(&fixture, "zfsstep2-cursor");
    pool.assert_the_module_release();
    pool.snapshot(&pool.checkout, &["origin"]);
    let origin: String = format!("{}@origin", pool.checkout);
    let first: String = pool.clone_of(&origin, "first");
    let second: String = pool.clone_of(&origin, "second");
    let device: ZfsDevice = ZfsDevice::open().expect("the fixture has the device");
    let names: Vec<String> = listing_names(|cursor| {
        return device.dataset_list_next(&pool.clones, cursor);
    });
    assert_eq!(names.len(), 2);
    assert!(names.contains(&first));
    assert!(names.contains(&second));
    println!("children in kernel order: {names:?}");
    let listed: Option<ListedObject> = device
        .dataset_list_next(&pool.clones, 0)
        .expect("the first child is answered");
    let Some(object) = listed else {
        panic!("the clone parent has children");
    };
    assert_eq!(
        object.name, names[0],
        "the listing starts where the kernel does"
    );
    assert!(object.cursor > 0, "the kernel moved the cursor");
    assert_eq!(
        object.stats.dds_guid,
        cli_value(&fixture, &object.name, "guid")
    );
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn listing_an_unpromoted_clones_snapshots_returns_only_its_own() {
    let Some(fixture) =
        Fixture::from_environment("live_read_tests::listing_an_unpromoted_clones_snapshots")
    else {
        return;
    };
    let pool: FixturePool<'_> = FixturePool::create(&fixture, "zfsstep2-snapshots");
    pool.assert_the_module_release();
    pool.snapshot(&pool.checkout, &["earliest", "origin", "checkout-later"]);
    let clone: String = pool.clone_of(&format!("{}@origin", pool.checkout), "removal");
    pool.snapshot(&clone, &["own-one", "own-two"]);
    let device: ZfsDevice = ZfsDevice::open().expect("the fixture has the device");
    let names: Vec<String> = listing_names(|cursor| {
        return device.snapshot_list_next(&clone, cursor);
    });
    let mut expected: Vec<String> = ["own-one", "own-two"]
        .iter()
        .map(|name| {
            return format!("{clone}@{name}");
        })
        .collect();
    let mut listed_names: Vec<String> = names.clone();
    expected.sort();
    listed_names.sort();
    assert_eq!(
        listed_names, expected,
        "the clone lists exactly its own snapshots"
    );
    let listed: Option<ListedObject> = device
        .snapshot_list_next(&clone, 0)
        .expect("the first snapshot is answered");
    let Some(first) = listed else {
        panic!("the clone has own snapshots");
    };
    assert!(first.stats.dds_is_snapshot != 0);
    assert_eq!(
        first.stats.dds_guid,
        cli_value(&fixture, &first.name, "guid")
    );
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn identity_reports_each_property_value_with_its_source() {
    let Some(fixture) =
        Fixture::from_environment("live_read_tests::identity_reports_each_property_value")
    else {
        return;
    };
    let pool: FixturePool<'_> = FixturePool::create(&fixture, "zfsstep2-properties");
    pool.assert_the_module_release();
    fixture.zfs(&["set", "monochromatic:fixture=one", &pool.checkout]);
    let device: ZfsDevice = ZfsDevice::open().expect("the fixture has the device");
    let identity: super::listed_object::DatasetIdentity = device
        .objset_stats(&pool.checkout)
        .expect("the identity is read");
    let reply: &NvList = &identity.properties;
    let checkout: &str = pool.checkout.as_str();
    let (recordsize, recordsize_source) =
        super::fixture_reply::cli_value_and_source(&fixture, checkout, "recordsize");
    assert_eq!(property_u64(reply, "recordsize"), recordsize);
    assert_eq!(
        property_source(reply, "recordsize"),
        expected_source(&recordsize_source, checkout)
    );
    assert_eq!(
        property_u64(reply, "createtxg"),
        cli_value(&fixture, checkout, "createtxg")
    );
    assert_eq!(property_str(reply, "monochromatic:fixture"), "one");
    assert_eq!(property_source(reply, "monochromatic:fixture"), checkout);
    assert_eq!(
        property_u64(reply, "guid"),
        cli_value(&fixture, checkout, "guid")
    );
    assert_eq!(property_u64(reply, "defer_destroy"), 0);
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_granted_delegations_are_what_the_fixture_set() {
    let Some(fixture) =
        Fixture::from_environment("live_read_tests::the_granted_delegations_are_the_ones_set")
    else {
        return;
    };
    let pool: FixturePool<'_> = FixturePool::create(&fixture, "zfsstep2-grants");
    pool.assert_the_module_release();
    pool.grant_the_proposed_scope();
    let device: ZfsDevice = ZfsDevice::open().expect("the fixture has the device");
    let (uid, _) = fixture.account_ids();
    let local_key: String = format!("ul${uid}");
    let descendent_key: String = format!("ud${uid}");
    let parent: NvList = device
        .get_fsacl(&pool.clones)
        .expect("the clone parent's grants are read");
    let granted: &NvList = parent.get_list(&pool.clones).unwrap_or_else(|| {
        panic!("the reply carries no grants for the clone parent: {parent:?}");
    });
    let parent_local: Vec<String> = assert_permissions(granted, &local_key, &["create", "mount"]);
    let parent_descendent: Vec<String> =
        assert_permissions(granted, &descendent_key, &["destroy", "mount", "userprop"]);
    let checkout: NvList = device
        .get_fsacl(&pool.checkout)
        .expect("the checkout's grants are read");
    let on_checkout: &NvList = checkout.get_list(&pool.checkout).unwrap_or_else(|| {
        panic!("the reply carries no grants for the checkout: {checkout:?}");
    });
    let checkout_local: Vec<String> = assert_permissions(on_checkout, &local_key, &["snapshot"]);
    let checkout_descendent: Vec<String> =
        assert_permissions(on_checkout, &descendent_key, &["clone", "destroy", "mount"]);
    println!(
        "MEASUREMENT grants clone-parent {local_key}={parent_local:?} \
         {descendent_key}={parent_descendent:?} checkout {local_key}={checkout_local:?} \
         {descendent_key}={checkout_descendent:?}"
    );
}

/// Assert that the delegations under `key` are exactly `wanted`,
/// by name,
/// so a
/// permission the fixture did not grant cannot hide in the reply.
///
/// Returns the names in the reply,
/// for the measurement line.
///
/// # Panics
///
/// When `granted` has no `key` at all,
/// or the names under it are not `wanted`.
fn assert_permissions(granted: &NvList, key: &str, wanted: &[&str]) -> Vec<String> {
    let list: &NvList = granted.get_list(key).unwrap_or_else(|| {
        panic!("the reply carries no {key} grant: {granted:?}");
    });
    let mut found: Vec<String> = list
        .iter()
        .map(|pair| {
            return pair.name.clone();
        })
        .collect();
    found.sort();
    let mut expected: Vec<String> = wanted
        .iter()
        .map(|name| {
            return (*name).to_string();
        })
        .collect();
    expected.sort();
    assert_eq!(found, expected, "the delegations under {key}");
    return found;
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn a_reply_past_the_initial_buffer_is_read_after_one_growth() {
    let Some(fixture) =
        Fixture::from_environment("live_read_tests::a_reply_past_the_initial_buffer")
    else {
        return;
    };
    let pool: FixturePool<'_> = FixturePool::create(&fixture, "zfsstep2-resize");
    pool.assert_the_module_release();
    let padded: usize = INITIAL_REPLY_SIZE / PAD_VALUE_BYTES * 2;
    pool.pad_user_properties(&pool.checkout, padded, PAD_VALUE_BYTES);
    let device: ZfsDevice = ZfsDevice::open().expect("the fixture has the device");
    let checkout: String = pool.checkout.clone();
    let mut request: Request = objset_stats(&checkout).expect("the name fits");
    let first: ReadAttempt = device
        .attempt_read(&mut request)
        .expect("the first read is answered");
    let ReadAttempt::NeedsBiggerReply { requested } = first else {
        panic!("the property list is past the initial buffer");
    };
    assert!(requested > INITIAL_REPLY_SIZE as u64);
    let grown: usize = request
        .grow_reply(requested)
        .expect("the named size is under the cap");
    assert_eq!(grown, requested.next_power_of_two() as usize);
    let second: ReadAttempt = device
        .attempt_read(&mut request)
        .expect("the grown read is answered");
    let ReadAttempt::Reply { list: fitting } = second else {
        panic!("the grown buffer holds the reply");
    };
    assert!(fitting.get_list("monochromatic:pad0").is_some());
    assert!(request.reply().len() > INITIAL_REPLY_SIZE);
    let through: NvList = device
        .objset_stats(&checkout)
        .expect("the retrying read grows the buffer itself")
        .properties;
    let counted: usize = through
        .iter()
        .filter(|pair| {
            return pair.name.starts_with("monochromatic:pad");
        })
        .count();
    assert_eq!(counted, padded);
}
