//! What:
//!  One arm that prints what the plan's VM items 5, 11 to 18 returned when
//!  the same requests were re-run through this crate:
//! versions,
//! layout sizes,
//!  GUIDs,
//! transaction groups,
//! per-name errors,
//! a grown reply's sizes and the key
//!  state,
//! each read from the kernel's own answer.
//! Why:
//!  A measurement record quotes values,
//! not only a green suite;
//! the other arms assert
//!  and stay quiet,
//! so this one prints one block the record can cite,
//! and it runs
//!  only in the guest.

use super::ZfsDevice;
use super::fixture::Fixture;
use super::fixture_pool::FixturePool;
use super::fixture_reply::{bookmark_identity, listing_names, property_u64};
use super::read_reply::ReadAttempt;
use super::write_requests::Promotion;
use crate::kernel_device::DatasetIdentity;
use crate::nvlist::NvList;
use crate::requests::{INITIAL_REPLY_SIZE, Request, objset_stats};

/// The bytes each padding property's value holds.
const PAD_VALUE_BYTES: usize = 3_000;
/// `ZFS_KEYSTATUS_UNAVAILABLE` and `ZFS_KEYSTATUS_AVAILABLE`
/// (`include/sys/fs/zfs.h` at `zfs-2.4.4`).
const KEYSTATUS_UNAVAILABLE: u64 = 1;

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_plan_items_report_what_the_crate_received() {
    let Some(fixture) = Fixture::from_environment("live_measurement_record_tests::the_plan_items")
    else {
        return;
    };
    let release: String = std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .expect("the guest's release is read")
        .trim()
        .to_string();
    let module: String = std::fs::read_to_string("/sys/module/zfs/version")
        .expect("the module release is read")
        .trim()
        .to_string();
    let srcversion: String = std::fs::read_to_string("/sys/module/zfs/srcversion")
        .expect("the module source version is read")
        .trim()
        .to_string();
    println!("MEASUREMENT guest kernel={release} module={module} srcversion={srcversion}");
    println!(
        "MEASUREMENT userspace={}",
        fixture.zfs(&["version"]).lines().collect::<Vec<&str>>().join(" ")
    );
    let pool: FixturePool<'_> = FixturePool::create(&fixture, "zfsstep2-record");
    pool.assert_the_module_release();
    let device: ZfsDevice = ZfsDevice::open().expect("the fixture has the device");
    let checkout: String = pool.checkout.clone();

    // The reply that outgrows its buffer.
    let padded: usize = INITIAL_REPLY_SIZE / PAD_VALUE_BYTES * 2;
    pool.pad_user_properties(&checkout, padded, PAD_VALUE_BYTES);
    let mut request: Request = objset_stats(&checkout).expect("the name fits");
    let first: ReadAttempt = device.attempt_read(&mut request).expect("the read is answered");
    if let ReadAttempt::NeedsBiggerReply { requested } = first {
        println!(
            "MEASUREMENT reply initial={INITIAL_REPLY_SIZE} requested={requested} \
             properties={padded} value_bytes={PAD_VALUE_BYTES}"
        );
        request.grow_reply(requested).expect("the size is under the cap");
    }
    let through = device.objset_stats(&checkout).expect("the grown read is answered");
    println!(
        "MEASUREMENT recordsize={} source={}",
        property_u64(&through.properties, "recordsize"),
        super::fixture_reply::property_source(&through.properties, "recordsize")
    );

    // Item 11 and 13: promote, the moved history and the bookmark.
    pool.snapshot(&checkout, &["earliest", "origin", "checkout-later"]);
    let bookmark: String = format!("{checkout}#from-earliest");
    fixture.zfs(&["bookmark", &format!("{checkout}@earliest"), &bookmark]);
    let promotion: String = pool.clone_of(&format!("{checkout}@origin"), "promotion");
    pool.snapshot(&promotion, &["worktree-own"]);
    let before: Vec<(String, u64, u64)> = pool.identity_rows("snapshot", &checkout);
    let bookmarks: NvList = device
        .get_bookmarks(&checkout, &[])
        .expect("the checkout's bookmarks are read");
    println!(
        "MEASUREMENT before-promote checkout snapshots={before:?} bookmark={:?}",
        bookmark_identity(&bookmarks, "from-earliest")
    );
    assert!(matches!(device.promote(&promotion), Promotion::Promoted));
    let promoted: DatasetIdentity = device.objset_stats(&promotion).expect("the clone is read");
    let moved: Vec<(String, u64, u64)> = pool.identity_rows("snapshot", &promotion);
    println!(
        "MEASUREMENT after-promote checkout origin={} promoted origin='{}' moved={moved:?}",
        device
            .objset_stats(&checkout)
            .expect("the checkout is read")
            .stats
            .origin(),
        promoted.stats.origin()
    );

    // Item 14 and 18: the own set and the mixed set, through the binding.
    pool.snapshot(&promotion, &["own-one", "own-two", "with-dependent", "without-dependent"]);
    let dependent: String = pool.clone_of(&format!("{promotion}@with-dependent"), "dependent");
    let own_first: String = format!("{promotion}@own-one");
    let own_second: String = format!("{promotion}@own-two");
    let destroyed = device.destroy_snaps(&pool.name, &[own_first.as_str(), own_second.as_str()]);
    println!("MEASUREMENT own-batch result={destroyed:?}");
    let pinned: String = format!("{promotion}@with-dependent");
    let free: String = format!("{promotion}@without-dependent");
    let refused = device.destroy_snaps(&pool.name, &[pinned.as_str(), free.as_str()]);
    if let Err(error) = refused {
        println!("MEASUREMENT mixed-batch refused: {error}");
    }
    let kept: Vec<String> = listing_names(|cursor| {
        return device.snapshot_list_next(&promotion, cursor);
    });
    println!(
        "MEASUREMENT dependent clone origin={} kept={kept:?}",
        device
            .objset_stats(&dependent)
            .expect("the dependent clone is read")
            .stats
            .origin()
    );

    // Item 12: the same name replaced, with the identity the kernel reports.
    // After the promotion the checkout is a clone of the promotion dataset,
    // so the origin snapshot it clones from now lives there.
    let promoted_origin: String = format!("{promotion}@origin");
    let replacement: String = pool.clone_of(&promoted_origin, "replacement");
    let first_guid: u64 = property_u64(
        &device
            .objset_stats(&replacement)
            .expect("the replacement is read")
            .properties,
        "guid",
    );
    device
        .destroy(&replacement, false)
        .expect("the unmounted clone is destroyed");
    pool.clone_of(&promoted_origin, "replacement");
    let second_guid: u64 = property_u64(
        &device
            .objset_stats(&replacement)
            .expect("the new replacement is read")
            .properties,
        "guid",
    );
    println!("MEASUREMENT replacement guids before={first_guid} after={second_guid}");

    // Item 5: a clone while the key is unloaded.
    fixture.zfs(&["unmount", "-f", &checkout]);
    fixture.zfs(&["unmount", "-f", &pool.home]);
    fixture.zfs(&["unload-key", &pool.name]);
    let key_clone: String = format!("{}/key-unavailable", pool.clones);
    device
        .clone(&key_clone, &promoted_origin, None)
        .expect("a clone needs no loaded key");
    let key_identity: DatasetIdentity = device.objset_stats(&key_clone).expect("the clone is read");
    println!(
        "MEASUREMENT key-unloaded clone keystatus={} origin={}",
        property_u64(&key_identity.properties, "keystatus"),
        key_identity.stats.origin()
    );
    fixture.zfs(&["load-key", &pool.name]);
    let loaded: DatasetIdentity = device
        .objset_stats(&key_clone)
        .expect("the clone is read after the key load");
    println!(
        "MEASUREMENT key-loaded keystatus={} ({} is unavailable)",
        property_u64(&loaded.properties, "keystatus"),
        KEYSTATUS_UNAVAILABLE
    );
    println!(
        "MEASUREMENT encryption={} keyformat={}",
        property_u64(&loaded.properties, "encryption"),
        property_u64(&loaded.properties, "keyformat")
    );
}
