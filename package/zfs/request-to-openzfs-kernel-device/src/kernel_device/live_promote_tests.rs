//! What:
//!  The live promote controls item 11 measured through the binding:
//! a promotion
//!  moves the origin and the older checkout snapshots into the clone without
//!  changing their GUIDs or transaction groups,
//! the checkout becomes a clone of
//!  it,
//! a bookmark travels with the moved history,
//! and the reverse promotion
//!  restores the snapshot map.
//! Why:
//!  The project included promote as a test-only primitive,
//! because promoting a
//!  worktree clone moves protected checkout history into a dataset ordinary
//!  removal destroys.
//! These arms prove the primitive does exactly that and no
//!  more,
//! and that a name conflict is refused with the conflicting snapshot named.

use super::ZfsDevice;
use super::fixture::Fixture;
use super::fixture_pool::FixturePool;
use super::fixture_reply::bookmark_identity;
use super::write_requests::Promotion;
use crate::kernel_device::DatasetIdentity;
use crate::nvlist::NvList;

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn promotion_moves_the_older_history_without_changing_identity() {
    let Some(fixture) = Fixture::from_environment("live_promote_tests::promotion_moves") else {
        return;
    };
    let pool: FixturePool<'_> = FixturePool::create(&fixture, "zfsstep2-promote");
    pool.assert_the_module_release();
    pool.snapshot(&pool.checkout, &["earliest", "origin", "checkout-later"]);
    let bookmark: String = format!("{}#from-earliest", pool.checkout);
    fixture.zfs(&[
        "bookmark",
        &format!("{}@earliest", pool.checkout),
        &bookmark,
    ]);
    let promotion: String = pool.clone_of(&format!("{}@origin", pool.checkout), "promotion");
    pool.snapshot(&promotion, &["worktree-own"]);
    let device: ZfsDevice = ZfsDevice::open().expect("the fixture has the device");
    let checkout_before: Vec<(String, u64, u64)> = pool.identity_rows("snapshot", &pool.checkout);
    let promotion_before: Vec<(String, u64, u64)> = pool.identity_rows("snapshot", &promotion);
    let bookmark_before: NvList = device
        .get_bookmarks(&pool.checkout, &[])
        .expect("the checkout's bookmarks are read");
    let bookmark_id: (u64, u64) = bookmark_identity(&bookmark_before, "from-earliest");
    assert!(matches!(device.promote(&promotion), Promotion::Promoted));
    let promoted: DatasetIdentity = device
        .objset_stats(&promotion)
        .expect("the promoted clone is read");
    assert_eq!(
        promoted.stats.origin(),
        "",
        "the promoted clone has no origin"
    );
    let checkout_after: DatasetIdentity = device
        .objset_stats(&pool.checkout)
        .expect("the checkout is read");
    assert_eq!(
        checkout_after.stats.origin(),
        format!("{promotion}@origin"),
        "the checkout now depends on the promoted clone"
    );
    let promotion_after: Vec<(String, u64, u64)> = pool.identity_rows("snapshot", &promotion);
    for row in &checkout_before {
        if row.0.ends_with("@origin") || row.0.ends_with("@earliest") {
            let moved: String = format!(
                "{promotion}@{}",
                row.0
                    .rsplit('@')
                    .next()
                    .expect("a snapshot name has a tail")
            );
            let found: &(String, u64, u64) = promotion_after
                .iter()
                .find(|candidate| {
                    return candidate.0 == moved;
                })
                .unwrap_or_else(|| {
                    panic!("the promoted clone lists no {moved}: {promotion_after:?}");
                });
            assert_eq!(
                found,
                &(moved.clone(), row.1, row.2),
                "the moved snapshot keeps its GUID and creation transaction group"
            );
        }
    }
    let checkout_now: Vec<(String, u64, u64)> = pool.identity_rows("snapshot", &pool.checkout);
    let later: &(String, u64, u64) = checkout_now
        .iter()
        .find(|row| {
            return row.0.ends_with("@checkout-later");
        })
        .unwrap_or_else(|| {
            panic!("the checkout lost its later snapshot: {checkout_now:?}");
        });
    assert_eq!(
        later.1,
        checkout_before
            .iter()
            .find(|row| {
                return row.0.ends_with("@checkout-later");
            })
            .expect("the later snapshot existed before")
            .1
    );
    let bookmarks_after: NvList = device
        .get_bookmarks(&promotion, &[])
        .expect("the promoted clone's bookmarks are read");
    assert_eq!(
        bookmark_identity(&bookmarks_after, "from-earliest"),
        bookmark_id
    );
    let checkout_bookmarks: NvList = device
        .get_bookmarks(&pool.checkout, &[])
        .expect("the checkout's bookmarks are read");
    assert!(checkout_bookmarks.get_list("from-earliest").is_none());
    assert!(matches!(
        device.promote(&pool.checkout),
        Promotion::Promoted
    ));
    assert_eq!(
        pool.identity_rows("snapshot", &pool.checkout),
        checkout_before
    );
    assert_eq!(pool.identity_rows("snapshot", &promotion), promotion_before);
    let restored: NvList = device
        .get_bookmarks(&pool.checkout, &[])
        .expect("the restored checkout's bookmarks are read");
    assert_eq!(bookmark_identity(&restored, "from-earliest"), bookmark_id);
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn a_conflicting_snapshot_name_is_refused_and_named() {
    let Some(fixture) = Fixture::from_environment("live_promote_tests::a_conflict_is_named") else {
        return;
    };
    let pool: FixturePool<'_> = FixturePool::create(&fixture, "zfsstep2-promote-conflict");
    pool.assert_the_module_release();
    pool.snapshot(&pool.checkout, &["earliest", "origin"]);
    let promotion: String = pool.clone_of(&format!("{}@origin", pool.checkout), "promotion");
    let device: ZfsDevice = ZfsDevice::open().expect("the fixture has the device");
    assert!(matches!(device.promote(&promotion), Promotion::Promoted));
    fixture.zfs(&["snapshot", &format!("{}@earliest", pool.checkout)]);
    match device.promote(&pool.checkout) {
        Promotion::Promoted => panic!("the name 'earliest' exists on both sides"),
        Promotion::Refused {
            conflicting_snapshot,
            error,
        } => {
            assert_eq!(conflicting_snapshot, "earliest");
            assert_eq!(error.errno(), Some(libc::EEXIST));
        }
    }
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn a_dataset_that_is_not_a_clone_is_refused_without_a_name() {
    let Some(fixture) = Fixture::from_environment("live_promote_tests::not_a_clone") else {
        return;
    };
    let pool: FixturePool<'_> = FixturePool::create(&fixture, "zfsstep2-promote-plain");
    pool.assert_the_module_release();
    let device: ZfsDevice = ZfsDevice::open().expect("the fixture has the device");
    match device.promote(&pool.checkout) {
        Promotion::Promoted => panic!("the checkout is nobody's clone yet"),
        Promotion::Refused {
            conflicting_snapshot,
            error,
        } => {
            assert!(conflicting_snapshot.is_empty());
            assert_eq!(error.errno(), Some(libc::EINVAL));
        }
    }
}
