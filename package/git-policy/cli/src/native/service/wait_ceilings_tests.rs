//! Controls for the ceilings of the waits for a foreign `index.lock`:
//! each wait takes the ceiling derived from its own measurement.
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

/// The merge's wait for a foreign holder is five times the longest self-released foreign wait,
/// in whole minutes.
#[test]
fn the_merge_waits_the_measured_ceiling() {
    assert_eq!(MERGE_LOCK_CEILING, derived(LONGEST_SELF_RELEASED_FOREIGN_LOCK_WAIT));
    let source: &str = include_str!("../lock/index_holder_wait.rs");
    assert!(source.contains("MERGE_LOCK_CEILING"));
}

/// The settling of earlier batches waits five times the longest settle measured,
/// in whole minutes.
#[test]
fn the_settling_of_earlier_batches_waits_the_measured_ceiling() {
    assert_eq!(SETTLE_LOCK_CEILING, derived(LONGEST_SETTLE_WAIT));
    let source: &str = include_str!("../invocation/management_service_settle_earlier_batches.rs");
    assert!(source.contains("SETTLE_LOCK_CEILING"));
}

/// The index reconciliation after corrections waits five times the longest self-released foreign wait,
/// in whole minutes.
#[test]
fn the_index_reconciliation_waits_the_measured_ceiling() {
    assert_eq!(RECONCILE_LOCK_CEILING, derived(LONGEST_SELF_RELEASED_FOREIGN_LOCK_WAIT));
    let source: &str = include_str!("../landing/service_land_follow.rs");
    assert!(source.contains("RECONCILE_LOCK_CEILING"));
}

/// Every foreign-lock wait of the wrapper is the one measured whole minute,
/// the landing's and the three that follow it.
#[test]
fn every_foreign_lock_wait_is_the_measured_whole_minute() {
    assert_eq!(LANDING_LOCK_CEILING, derived(LONGEST_SELF_RELEASED_FOREIGN_LOCK_WAIT));
    assert_eq!(MERGE_LOCK_CEILING, LANDING_LOCK_CEILING);
    assert_eq!(RECONCILE_LOCK_CEILING, LANDING_LOCK_CEILING);
    assert_eq!(SETTLE_LOCK_CEILING, derived(LONGEST_SETTLE_WAIT));
    assert!(MINUTE <= LANDING_LOCK_CEILING);
    assert!(SETTLE_LOCK_CEILING <= LANDING_LOCK_CEILING);
}

/// A commit that moved under its own build is built again until the answer ceiling,
/// not for a count of builds,
/// and the commit path's answer to a moved branch names the ceiling.
/// The answer to a moved branch lives in its own file
/// (`landing/service_commit_after_a_moved_branch.rs`),
/// so that the loop in `landing/service_commit.rs` stays under the line limit;
/// that move is d5af173797,
/// which kept every condition and every message as it was.
#[test]
fn a_commit_that_moved_under_its_build_is_rebuilt_until_the_answer_ceiling() {
    assert_eq!(MOVED_BRANCH_REBUILD_CEILING, ANSWER_CEILING);
    let decision: &str = include_str!("../landing/service_commit_rebuild_ceiling.rs");
    assert!(decision.contains("MOVED_BRANCH_REBUILD_CEILING"), "the rebuilding decision takes this ceiling");
    assert!(decision.contains("may_rebuild"), "the rebuilding decision reads this ceiling");
    let answered: &str = include_str!("../landing/service_commit_after_a_moved_branch.rs");
    let commit: &str = include_str!("../landing/service_commit.rs");
    assert!(answered.contains("may_rebuild"), "the answer to a moved branch takes this ceiling");
    assert!(!answered.contains("REBUILD_ATTEMPTS") && !commit.contains("REBUILD_ATTEMPTS"), "a count of builds no longer ends a commit");
}

/// The scratch repository's `git commit` is stopped at three times the slowest `git commit` step record:
/// the owner's answer of 2026-11-08 ("About 16 seconds"),
/// 3 x 5,444 ms,
/// with no rounding up.
#[test]
fn the_scratch_commit_step_is_stopped_at_three_times_the_slowest_step_record() {
    assert_eq!(SLOWEST_MEASURED_COMMIT_STEP, Duration::from_millis(5_444), "the figure of the records below 16 s");
    assert_eq!(SCRATCH_COMMIT_STEP_MARGIN, 3);
    assert_eq!(SCRATCH_COMMIT_STEP_CEILING, SLOWEST_MEASURED_COMMIT_STEP * SCRATCH_COMMIT_STEP_MARGIN);
    assert_eq!(SCRATCH_COMMIT_STEP_CEILING, Duration::from_millis(16_332), "about 16 s, the owner's answer");
    let source: &str = include_str!("../commit/commit_preparation.rs");
    assert!(source.contains("SCRATCH_COMMIT_STEP_CEILING"), "the step that runs the scratch commit takes this ceiling");
}

/// What:
///  The wait the standing notice uses is the shipped one unless a positive number of seconds is set,
/// and a setting that means nothing leaves it.
/// Why:
///   A control reads the variable to move the notice past its own bound,
/// and a misspelling must not silence the telling:
/// a setting that is missing,
/// is not a
///  number,
/// is zero or is a sign leaves the shipped wait.
#[test]
fn the_standing_notice_takes_a_positive_setting_and_the_shipped_wait_otherwise() {
    assert_eq!(notice_after_of(Some("30")), Duration::from_secs(30));
    assert_eq!(notice_after_of(Some(" 1 ")), Duration::from_secs(1));
    for meaningless in [None, Some(""), Some("abc"), Some("0"), Some("-5"), Some("1.5")] {
        assert_eq!(notice_after_of(meaningless), WAITING_NOTICE_AFTER, "{meaningless:?}");
    }
    assert_eq!(notice_after_of(None), WAITING_NOTICE_AFTER);
}

/// The variable the control of the standing notice sets is the one the client reads.
#[test]
fn the_standing_notice_variable_is_the_one_the_client_reads() {
    assert_eq!(WAITING_NOTICE_VARIABLE, "CLI_GIT_STANDING_NOTICE_SECONDS");
    let source: &str = include_str!("../landing/service_commit_input_notes_and_embedded_commit.rs");
    assert!(source.contains("waiting_notice_after: crate::wait_ceilings::waiting_notice_after()"), "the client's setting is read where the config is made");
}

/// What:
///  The wait for a snapshot made ahead is the owner's 60 s bound for an add,
/// and the snapshot rung names it.
/// Why:
///   The ceiling covers the whole wait for a btrfs call the add does not make itself,
/// so it is the one number that
///  decides when Git's own checkout makes the worktree instead;
/// a ceiling that drifted from the owner's bound,
/// or a
///  rung that stopped taking it,
/// would leave an add waiting with no end again.
#[test]
fn the_wait_for_a_snapshot_made_ahead_is_the_owners_bound_for_an_add() {
    assert_eq!(SNAPSHOT_AHEAD_CEILING, Duration::from_secs(60), "the owner, 2026-11-10: \"60s is fine for adding worktrees\"");
    let rung: &str = include_str!("../worktree/worktree_ladder_finish.rs");
    assert!(rung.contains("SNAPSHOT_AHEAD_CEILING"), "the snapshot rung takes this ceiling");
}
