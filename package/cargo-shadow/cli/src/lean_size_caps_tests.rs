//! The tests of mechanism E:
//! a profile directory over its cap loses its oldest entries until it fits,
//! one under its cap loses nothing,
//! and an entry within the grace period is never evicted.

use std::{path::PathBuf, time::Duration};

use crate::lean_identity::Deadline;
use crate::lean_size_caps::SizeCap;
use crate::lean_sizes::tree_bytes;
use crate::lean_test_support::{DAY, HOUR, NOW, add_incremental, add_old_unit, make_profile, real_environment};
use crate::lean_unit_removal::{ProfileOutcome, apply_removals};
use crate::test_scratch::scratch;

/// The grace period of these tests.
const GRACE: i64 = DAY;

/// Applies a cap to a profile directory for real.
fn cap(profile: &std::path::Path, cap_bytes: u64) -> ProfileOutcome {
    let planner: SizeCap = SizeCap { cap_bytes, grace_seconds: GRACE, deadline: Deadline::after(Duration::from_secs(60)) };
    let mut log: Vec<String> = Vec::new();
    return apply_removals(profile, &planner, &real_environment(), NOW, false, &Deadline::after(Duration::from_secs(60)), &mut log);
}

#[test]
fn the_oldest_entries_go_first_until_the_directory_fits() {
    let root = scratch();
    let profile: PathBuf = make_profile(root.path(), "debug");
    let oldest: Vec<PathBuf> = add_old_unit(&profile, "a", "00000000000000a1", 7, NOW - 30 * DAY);
    let middle: Vec<PathBuf> = add_old_unit(&profile, "b", "00000000000000b2", 7, NOW - 20 * DAY);
    let newest: Vec<PathBuf> = add_old_unit(&profile, "c", "00000000000000c3", 7, NOW - 10 * DAY);
    let mut two_units: u64 = 0;
    for path in middle.iter().chain(newest.iter()) {
        two_units += tree_bytes(path, &Deadline::after(Duration::from_secs(60))).bytes;
    }
    // The cap fits the two newer units exactly,
    // so the oldest must go and nothing more.
    let outcome: ProfileOutcome = cap(&profile, two_units);
    assert_eq!(outcome.units, 1, "{outcome:?}");
    assert!(oldest.iter().all(|path| { return !path.exists(); }));
    assert!(middle.iter().all(|path| { return path.exists(); }));
    assert!(newest.iter().all(|path| { return path.exists(); }));
}

#[test]
fn a_directory_under_its_cap_loses_nothing() {
    let root = scratch();
    let profile: PathBuf = make_profile(root.path(), "debug");
    let only: Vec<PathBuf> = add_old_unit(&profile, "a", "00000000000000a1", 7, NOW - 30 * DAY);
    let outcome: ProfileOutcome = cap(&profile, 10_000_000);
    assert_eq!((outcome.units, outcome.incremental), (0, 0));
    assert!(only.iter().all(|path| { return path.exists(); }));
}

#[test]
fn an_entry_within_the_grace_period_is_never_evicted_even_over_the_cap() {
    let root = scratch();
    let profile: PathBuf = make_profile(root.path(), "debug");
    let fresh: Vec<PathBuf> = add_old_unit(&profile, "a", "00000000000000a1", 7, NOW - HOUR);
    let fresh_incremental: PathBuf = add_incremental(&profile, "a-1", NOW - HOUR);
    let outcome: ProfileOutcome = cap(&profile, 1);
    assert_eq!((outcome.units, outcome.incremental), (0, 0));
    assert!(fresh.iter().all(|path| { return path.exists(); }));
    assert!(fresh_incremental.exists());
}

#[test]
fn old_incremental_state_is_evicted_with_the_units_by_age() {
    let root = scratch();
    let profile: PathBuf = make_profile(root.path(), "debug");
    let unit: Vec<PathBuf> = add_old_unit(&profile, "a", "00000000000000a1", 7, NOW - 5 * DAY);
    let old_incremental: PathBuf = add_incremental(&profile, "a-1", NOW - 40 * DAY);
    let mut unit_bytes: u64 = 0;
    for path in &unit {
        unit_bytes += tree_bytes(path, &Deadline::after(Duration::from_secs(60))).bytes;
    }
    // The cap fits the unit exactly and not the incremental directory beside it.
    let outcome: ProfileOutcome = cap(&profile, unit_bytes);
    assert_eq!(outcome.incremental, 1, "{outcome:?}");
    assert!(!old_incremental.exists());
    assert!(unit.iter().all(|path| { return path.exists(); }));
}
