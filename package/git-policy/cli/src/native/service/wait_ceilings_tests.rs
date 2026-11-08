//! Controls for the ceilings of the waits for a foreign `index.lock`:
//! each wait takes the ceiling derived from its own measurement,
//!  and none still takes the unmeasured ten minutes.
//! Why:
//!  On 2026-11-07 four commits waited 678 to 1,229 s because the landing waited a full unmeasured ten minutes;
//! the merge,
//! the settling
//!  of earlier batches and the index reconciliation after corrections waited the same ten minutes for the same kind of holder.

use super::*;

/// The margin every ceiling of this file keeps over its longest measurement.
const MARGIN: u32 = 5;

/// One minute,
/// the unit the ceilings are rounded up to.
const MINUTE: Duration = Duration::from_secs(60);

/// The smallest whole number of minutes that is at least five times `measured`.
fn derived(measured: Duration) -> Duration {
    return whole_minutes_at_least(measured * MARGIN);
}

/// Whether `source` names the unmeasured `LOCK_CEILING` as a word of its own (`LANDING_LOCK_CEILING` and the like are other names).
fn names_the_unmeasured_ceiling(source: &str) -> bool {
    return source.match_indices("LOCK_CEILING").any(|(at, _)| {
        let before: Option<char> = source[..at].chars().next_back();
        return !before.is_some_and(|character| { return character.is_ascii_uppercase() || character == '_'; });
    });
}

/// The merge's wait for a foreign holder is five times the longest self-released foreign wait,
/// in whole minutes.
#[test]
fn the_merge_waits_the_measured_ceiling_not_ten_minutes() {
    assert_eq!(MERGE_LOCK_CEILING, derived(LONGEST_SELF_RELEASED_FOREIGN_LOCK_WAIT));
    assert!(MERGE_LOCK_CEILING < LOCK_CEILING);
    let source: &str = include_str!("../lock/index_holder_wait.rs");
    assert!(!names_the_unmeasured_ceiling(source), "the merge's wait (`wait_for_index_release`) still takes LOCK_CEILING");
    assert!(source.contains("MERGE_LOCK_CEILING"));
}

/// The settling of earlier batches waits five times the longest settle measured,
/// in whole minutes.
#[test]
fn the_settling_of_earlier_batches_waits_the_measured_ceiling_not_ten_minutes() {
    assert_eq!(SETTLE_LOCK_CEILING, derived(LONGEST_SETTLE_WAIT));
    assert!(SETTLE_LOCK_CEILING < LOCK_CEILING);
    let source: &str = include_str!("../invocation/management_service_settle_earlier_batches.rs");
    assert!(!names_the_unmeasured_ceiling(source), "the settle still takes LOCK_CEILING");
    assert!(source.contains("SETTLE_LOCK_CEILING"));
}

/// The index reconciliation after corrections waits five times the longest self-released foreign wait,
/// in whole minutes.
#[test]
fn the_index_reconciliation_waits_the_measured_ceiling_not_ten_minutes() {
    assert_eq!(RECONCILE_LOCK_CEILING, derived(LONGEST_SELF_RELEASED_FOREIGN_LOCK_WAIT));
    assert!(RECONCILE_LOCK_CEILING < LOCK_CEILING);
    let source: &str = include_str!("../landing/service_land_follow.rs");
    assert!(!names_the_unmeasured_ceiling(source), "the reconciliation (`update_index`) still takes LOCK_CEILING");
    assert!(source.contains("RECONCILE_LOCK_CEILING"));
}

/// No wait is left on the unmeasured ceiling:
/// it stays only as the basis of the foreign-owner patience of another PID namespace.
#[test]
fn no_wait_is_left_on_the_unmeasured_ceiling() {
    for (name, source) in [
        ("the landing", include_str!("../landing/service_index_lock.rs")),
        ("the service's default", include_str!("service_main.rs")),
    ] {
        assert!(!names_the_unmeasured_ceiling(source), "{name} takes LOCK_CEILING");
    }
    assert!(MINUTE <= LANDING_LOCK_CEILING);
}

/// A commit that moved under its own build is built again until the answer ceiling,
/// not for a count of builds,
/// and the commit path names the ceiling.
#[test]
fn a_commit_that_moved_under_its_build_is_rebuilt_until_the_answer_ceiling() {
    assert_eq!(MOVED_BRANCH_REBUILD_CEILING, ANSWER_CEILING);
    let decision: &str = include_str!("../landing/service_commit_rebuild_ceiling.rs");
    assert!(decision.contains("MOVED_BRANCH_REBUILD_CEILING"), "the rebuilding decision takes this ceiling");
    let commit: &str = include_str!("../landing/service_commit.rs");
    assert!(commit.contains("may_rebuild") && !commit.contains("REBUILD_ATTEMPTS"), "a count of builds no longer ends a commit");
}

/// The scratch repository's `git commit` is stopped at three times the slowest `git commit` step record:
/// the owner's answer of 2026-11-08 ("About 3 s now"),
/// 3 x 1,002 ms,
/// with no rounding up.
#[test]
fn the_scratch_commit_step_is_stopped_at_three_times_the_slowest_step_record() {
    assert_eq!(SLOWEST_MEASURED_COMMIT_STEP, Duration::from_millis(1_002), "the figure of the 14 step records");
    assert_eq!(SCRATCH_COMMIT_STEP_MARGIN, 3);
    assert_eq!(SCRATCH_COMMIT_STEP_CEILING, SLOWEST_MEASURED_COMMIT_STEP * SCRATCH_COMMIT_STEP_MARGIN);
    let source: &str = include_str!("../commit/commit_preparation.rs");
    assert!(source.contains("SCRATCH_COMMIT_STEP_CEILING"), "the step that runs the scratch commit takes this ceiling");
}
