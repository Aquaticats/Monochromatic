//! What:
//!  The fixture the delegated-account arms run against,
//! built by the root run before
//!  the account process starts:
//! one encrypted pool with the design's three grants,
//!  one clone per arm,
//! each with exactly the snapshots that arm needs,
//! and one of those
//!  clones promoted,
//! because item 17 measured the delegated batch on a promoted
//!  clone and item 18 on an unpromoted one.
//! Why:
//!  A delegated arm proves nothing if root made its requests for it,
//! and an
//!  account cannot snapshot,
//! so root must prepare the snapshots and then leave the
//!  pool alone.
//! Keeping the preparation in one test means the recipe has no
//!  knowledge of the layout,
//! and the account tests assert what it built.
//! The unpromoted arms clone the checkout's later snapshot,
//! so promoting the
//!  batch clone moves only its own origin history and leaves every other
//!  arm's
//!  origin where it was,
//! and the account keeps exactly the design's grants.

use super::ZfsDevice;
use super::fixture::Fixture;
use super::fixture_pool::{FIXTURE_POOL_PREFIX, FixturePool};
use super::write_requests::Promotion;

/// The clone names the account arms use,
/// and the snapshots each one needs.
pub(crate) const READ_ARM: &str = "read-arm";
/// The unpromoted clone the delegated batch destroys the own snapshots of.
pub(crate) const BATCH_ARM: &str = "batch-arm";
/// The promoted clone the delegated batch destroys the own snapshots of,
/// item 17's shape.
pub(crate) const PROMOTED_BATCH_ARM: &str = "promoted-batch-arm";
/// The clone one of whose snapshots has a dependent clone.
pub(crate) const DEPENDENT_ARM: &str = "dependent-arm";
/// The clone the account destroys and recreates.
pub(crate) const REPLACEMENT_ARM: &str = "replacement-arm";
/// The clone the account sets a user property on.
pub(crate) const PROPERTY_ARM: &str = "property-arm";
/// The dependent clone of [`DEPENDENT_ARM`]'s `@with-dependent`.
pub(crate) const DEPENDENT_CLONE: &str = "dependent-of-dependent";
/// The dependent clone of [`PROMOTED_BATCH_ARM`]'s `@with-dependent`,
/// which item 17's
/// mixed set names.
pub(crate) const PROMOTED_DEPENDENT_CLONE: &str = "promoted-dependent";
/// The snapshot of [`DEPENDENT_ARM`] that a clone pins.
pub(crate) const PINNED_SNAPSHOT: &str = "with-dependent";
/// The snapshot of [`DEPENDENT_ARM`] that nothing pins.
pub(crate) const FREE_SNAPSHOT: &str = "without-dependent";
/// The checkout snapshot every unpromoted arm clones from,
/// which the promotion
/// of [`PROMOTED_BATCH_ARM`] leaves on the checkout.
pub(crate) const CHECKOUT_SNAPSHOT: &str = "checkout-later";
/// The snapshots the promoted clone made itself,
/// which the delegated batch destroys.
pub(crate) const OWN_SNAPSHOTS: [&str; 2] = ["own-one", "own-two"];

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_root_run_prepares_the_delegated_fixture() {
    let Some(fixture) =
        Fixture::from_environment("live_account_fixture::the_root_run_prepares_the_fixture")
    else {
        return;
    };
    assert_eq!(
        Fixture::current_uid(),
        0,
        "the delegated fixture is prepared as root; the recipe runs this arm in the root suite \
         and the account arms after it"
    );
    let account_pool: String = fixture.account_pool.clone();
    assert!(
        account_pool.starts_with(FIXTURE_POOL_PREFIX),
        "the account pool is named '{FIXTURE_POOL_PREFIX}...'"
    );
    let pool: FixturePool<'_> = FixturePool::create(&fixture, &account_pool);
    pool.assert_the_module_release();
    pool.snapshot(&pool.checkout, &["earliest", "origin", CHECKOUT_SNAPSHOT]);
    let device: ZfsDevice = ZfsDevice::open().expect("the fixture has the device");

    // Item 17's shape: a clone promoted before the account touches it, so the
    // batch names a dataset that holds the checkout's transferred history.
    let origin: String = format!("{}@origin", pool.checkout);
    let promoted: String = pool.clone_of(&origin, PROMOTED_BATCH_ARM);
    assert!(
        matches!(device.promote(&promoted), Promotion::Promoted),
        "the batch clone is promoted"
    );
    assert_eq!(
        device
            .objset_stats(&pool.checkout)
            .expect("the checkout is read")
            .stats
            .origin(),
        format!("{promoted}@origin"),
        "the promotion leaves the checkout a clone of the promoted dataset"
    );
    pool.snapshot(&promoted, &[OWN_SNAPSHOTS[0], OWN_SNAPSHOTS[1]]);
    pool.snapshot(&promoted, &[PINNED_SNAPSHOT, FREE_SNAPSHOT]);
    let pinned: String = format!("{promoted}@{PINNED_SNAPSHOT}");
    pool.clone_of(&pinned, PROMOTED_DEPENDENT_CLONE);

    // Item 18's shape: every other arm clones the later snapshot, which stays
    // on the checkout, so the promotion above moved nothing they depend on.
    let later: String = format!("{}@{CHECKOUT_SNAPSHOT}", pool.checkout);
    let read: String = pool.clone_of(&later, READ_ARM);
    pool.snapshot(&read, &[OWN_SNAPSHOTS[0], OWN_SNAPSHOTS[1]]);
    let batch: String = pool.clone_of(&later, BATCH_ARM);
    pool.snapshot(&batch, &[OWN_SNAPSHOTS[0], OWN_SNAPSHOTS[1]]);
    let dependent: String = pool.clone_of(&later, DEPENDENT_ARM);
    pool.snapshot(&dependent, &[PINNED_SNAPSHOT, FREE_SNAPSHOT]);
    pool.clone_of(&format!("{dependent}@{PINNED_SNAPSHOT}"), DEPENDENT_CLONE);
    pool.clone_of(&later, REPLACEMENT_ARM);
    pool.clone_of(&later, PROPERTY_ARM);
    pool.grant_the_proposed_scope();
    for arm in [
        READ_ARM,
        BATCH_ARM,
        DEPENDENT_ARM,
        REPLACEMENT_ARM,
        PROPERTY_ARM,
    ] {
        let identity = device
            .objset_stats(&format!("{}/{arm}", pool.clones))
            .expect("every prepared clone is readable");
        assert_eq!(
            identity.stats.origin(),
            later,
            "the unpromoted arm {arm} clones the checkout's later snapshot"
        );
    }
    let promoted_identity = device
        .objset_stats(&promoted)
        .expect("the promoted arm is readable");
    assert_eq!(
        promoted_identity.stats.origin(),
        "",
        "the promoted arm has no origin"
    );
    pool.keep();
}
