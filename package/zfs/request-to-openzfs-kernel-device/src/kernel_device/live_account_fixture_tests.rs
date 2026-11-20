//! What:
//!  The fixture the delegated-account arms run against,
//! built by the root run before
//!  the account process starts:
//! one encrypted pool with the design's three grants and
//!  one clone per arm,
//! each with exactly the snapshots that arm needs.
//! Why:
//!  A delegated arm proves nothing if root made its requests for it,
//! and an
//!  account cannot snapshot,
//! so root must prepare the snapshots and then leave the
//!  pool alone.
//! Keeping the preparation in one test means the recipe has no
//!  knowledge of the layout,
//! and the account tests assert what it built.

use super::ZfsDevice;
use super::fixture::Fixture;
use super::fixture_pool::{FIXTURE_POOL_PREFIX, FixturePool};

/// The clone names the account arms use,
/// and the snapshots each one needs.
pub(crate) const READ_ARM: &str = "read-arm";
/// The clone the delegated batch destroys the own snapshots of.
pub(crate) const BATCH_ARM: &str = "batch-arm";
/// The clone one of whose snapshots has a dependent clone.
pub(crate) const DEPENDENT_ARM: &str = "dependent-arm";
/// The clone the account destroys and recreates.
pub(crate) const REPLACEMENT_ARM: &str = "replacement-arm";
/// The clone the account sets a user property on.
pub(crate) const PROPERTY_ARM: &str = "property-arm";
/// The dependent clone of [`DEPENDENT_ARM`]'s `@with-dependent`.
pub(crate) const DEPENDENT_CLONE: &str = "dependent-of-dependent";
/// The snapshot of [`DEPENDENT_ARM`] that a clone pins.
pub(crate) const PINNED_SNAPSHOT: &str = "with-dependent";
/// The snapshot of [`DEPENDENT_ARM`] that nothing pins.
pub(crate) const FREE_SNAPSHOT: &str = "without-dependent";

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_root_run_prepares_the_delegated_fixture() {
    let Some(fixture) =
        Fixture::from_environment("live_account_fixture::the_root_run_prepares_the_fixture")
    else {
        return;
    };
    if Fixture::current_uid() != 0 {
        println!(
            "SKIP live_account_fixture::the_root_run_prepares_the_fixture: this process is not \
             root, so the account run does not prepare what it reads."
        );
        return;
    }
    let account_pool: String = fixture.account_pool.clone();
    assert!(
        account_pool.starts_with(FIXTURE_POOL_PREFIX),
        "the account pool is named '{FIXTURE_POOL_PREFIX}...'"
    );
    let pool: FixturePool<'_> = FixturePool::create(&fixture, &account_pool);
    pool.assert_the_module_release();
    pool.snapshot(&pool.checkout, &["earliest", "origin", "checkout-later"]);
    let origin: String = format!("{}@origin", pool.checkout);
    let read: String = pool.clone_of(&origin, READ_ARM);
    pool.snapshot(&read, &["own-one", "own-two"]);
    let batch: String = pool.clone_of(&origin, BATCH_ARM);
    pool.snapshot(&batch, &["own-one", "own-two"]);
    let dependent: String = pool.clone_of(&origin, DEPENDENT_ARM);
    pool.snapshot(&dependent, &[PINNED_SNAPSHOT, FREE_SNAPSHOT]);
    let pinned: String = format!("{dependent}@{PINNED_SNAPSHOT}");
    pool.clone_of(&pinned, DEPENDENT_CLONE);
    pool.clone_of(&origin, REPLACEMENT_ARM);
    pool.clone_of(&origin, PROPERTY_ARM);
    pool.grant_the_proposed_scope();
    let device: ZfsDevice = ZfsDevice::open().expect("the fixture has the device");
    let listed: Vec<String> = ["read-arm", "batch-arm", "dependent-arm", "replacement-arm"]
        .iter()
        .map(|name| {
            return format!("{}/{name}", pool.clones);
        })
        .collect();
    for dataset in &listed {
        let identity = device
            .objset_stats(dataset)
            .expect("every prepared clone is readable");
        assert_eq!(identity.stats.origin(), origin);
    }
    pool.keep();
}
