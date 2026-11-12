//! Controls for the ceilings of the waits for a foreign `index.lock`:
//! each wait takes the ceiling derived from its own measurement.
//! Why:
//!  On 2026-11-07 four commits waited 678 to 1,229 s because the landing waited a full unmeasured ten minutes;
//! the merge,
//! the settling
//!  of earlier batches and the index reconciliation after corrections waited the same ten minutes for the same kind of holder.

use super::*;
use crate::service_client_listen_and_ask_to_retire::START_WAIT;

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
/// The decision the loop asks is `landing/service_commit_rebuild_ceiling.rs` (`0bafaea12d`),
/// so `may_rebuild` is read where it is written.
#[test]
fn a_commit_that_moved_under_its_build_is_rebuilt_until_the_answer_ceiling() {
    assert_eq!(MOVED_BRANCH_REBUILD_CEILING, ANSWER_CEILING);
    let decision: &str = include_str!("../landing/service_commit_rebuild_ceiling.rs");
    assert!(decision.contains("MOVED_BRANCH_REBUILD_CEILING"), "the rebuilding decision takes this ceiling");
    assert!(decision.contains("may_rebuild"), "the rebuilding decision reads this ceiling");
    // The decision moved to the moved-branch file, and `service_commit.rs` keeps only the loop that runs it;
    // both are checked, because a count of builds must appear in neither.
    let answered: &str = include_str!("../landing/service_commit_after_a_moved_branch.rs");
    let commit: &str = include_str!("../landing/service_commit.rs");
    assert!(answered.contains("may_rebuild"), "the answer to a moved branch takes this ceiling");
    assert!(!answered.contains("REBUILD_ATTEMPTS") && !commit.contains("REBUILD_ATTEMPTS") && !decision.contains("REBUILD_ATTEMPTS"), "a count of builds no longer ends a commit");
}

/// The scratch repository's `git commit` is stopped at three times the slowest step record that ended by itself,
/// and never sooner than the floor a rotated log cannot lower,
/// so that an ordinary slow disk does not reach it.
/// Measured on 2026-11-12 over the eight generations the timing log held:
/// 3 x 12,735 ms is 38,205 ms,
/// the floor is 60,000 ms,
/// so the ceiling is the floor.
#[test]
fn the_scratch_commit_step_is_stopped_at_a_floor_or_three_times_the_slowest_step_that_ended() {
    assert_eq!(SLOWEST_COMMIT_STEP_THAT_ENDED_BY_ITSELF, Duration::from_millis(12_735), "the slowest step that landed, of 1,433 steps");
    assert_eq!(SCRATCH_COMMIT_STEP_MARGIN, 3);
    assert_eq!(SLOWEST_COMMIT_STEP_THAT_ENDED_BY_ITSELF * SCRATCH_COMMIT_STEP_MARGIN, Duration::from_millis(38_205));
    assert_eq!(SCRATCH_COMMIT_STEP_CEILING_FLOOR, Duration::from_secs(60));
    assert_eq!(SCRATCH_COMMIT_STEP_CEILING, SCRATCH_COMMIT_STEP_CEILING_FLOOR, "the floor holds this population above it, so the ceiling is the floor");
    assert_eq!(SCRATCH_COMMIT_STEP_CEILING, Duration::from_secs(60), "one minute, 4.7 times the slowest step that ended by itself");
    assert!(
        SCRATCH_COMMIT_STEP_CEILING >= SLOWEST_COMMIT_STEP_THAT_ENDED_BY_ITSELF * SCRATCH_COMMIT_STEP_MARGIN,
        "the ceiling is the later of the margin over the slowest ordinary step and the floor"
    );
    let source: &str = include_str!("../commit/commit_preparation.rs");
    assert!(source.contains("SCRATCH_COMMIT_STEP_CEILING"), "the step that runs the scratch commit takes this ceiling");
}

/// What:
///  A step the kernel holds in a disk wait is waited for to a bound far from an ordinary slow shelf,
///  and never sooner than the ceiling the commit ran under.
/// Why:
///   Ending a step that is waiting for the disk fails a commit that would have landed,
///  and on 2026-11-11 two commits failed that way,
///  their steps held for 37,153 and 47,978 ms under a 16,332 ms ceiling.
///  Five minutes is 6.2 times the slower of those two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(stallCeilingOf(ceiling).toBe(max(ceiling, 300_000)))
/// ```
#[test]
fn a_step_the_kernel_holds_is_waited_for_to_a_bound_far_from_an_ordinary_slow_disk() {
    assert_eq!(SCRATCH_COMMIT_STEP_STALL_CEILING, Duration::from_secs(300));
    assert!(SCRATCH_COMMIT_STEP_STALL_CEILING > SCRATCH_COMMIT_STEP_CEILING * 4, "the bound is far above the ceiling a step is noticed at");
    assert!(SCRATCH_COMMIT_STEP_STALL_CEILING > Duration::from_millis(47_978) * 6, "the bound is 6 times the longest step the kernel ever held here");
    assert_eq!(stall_ceiling_of(Duration::from_secs(1)), SCRATCH_COMMIT_STEP_STALL_CEILING, "a short ceiling is raised to the bound");
    assert_eq!(stall_ceiling_of(Duration::from_secs(600)), Duration::from_secs(600), "a ceiling a person set above the bound is never lowered");
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
///  A command that only has to tell the service about work whose record it already wrote waits the hand-over's greeting
///  bound,
/// and a command whose own next step depends on the answer keeps the longer start wait.
/// Why:
///   `git worktree remove` took 11.4 s and `git worktree add` 11 to 17 s with a service that answered nothing,
/// though the record was
///  already durable in both cases and 0.13 s and 0.5 s with a service that answered;
/// a wake that keeps the commit's
///  start wait pays it for an answer it does not need
/// (`~/.claude/../sonnet-audits/integration-tip-never-stuck/report.md`,
/// finding 4 of the after-alpha.9 audit).
/// The control reads the sources,
/// because the bound is chosen where the config is made,
/// and a later change back to the commit's wait
///  would meet nothing else.
#[test]
fn a_hand_over_waits_the_greeting_bound_and_never_the_commits_start_wait() {
    assert!(HAND_OVER_GREETING_WAIT < START_WAIT, "{HAND_OVER_GREETING_WAIT:?} must be shorter than {START_WAIT:?}");
    let stating: [(&str, &str); 6] = [
        ("../worktree/worktree_trash_hand_over.rs", include_str!("../worktree/worktree_trash_hand_over.rs")),
        ("../worktree/worktree_remove_hand_over.rs", include_str!("../worktree/worktree_remove_hand_over.rs")),
        ("../repair/package_repair_relink.rs", include_str!("../repair/package_repair_relink.rs")),
        ("../repair/package_repair_prewarm.rs", include_str!("../repair/package_repair_prewarm.rs")),
        ("../landing/held_push_release_wakes_the_service.rs", include_str!("../landing/held_push_release_wakes_the_service.rs")),
        ("../push/auto_push_trigger.rs", include_str!("../push/auto_push_trigger.rs")),
    ];
    for (path, source) in stating {
        assert!(source.contains("start_wait: HAND_OVER_GREETING_WAIT"), "{path} hands its record over with the hand-over's bound");
        assert!(!source.contains("start_wait: START_WAIT"), "{path} does not pay the commit's start wait for a record it already wrote");
        assert!(!source.contains("start_ceiling: START_CEILING"), "{path} does not pay the commit's start ceiling either");
        assert!(!source.contains("start_ceiling: crate::wait_ceilings::START_CEILING"), "{path} does not pay the commit's start ceiling either");
    }
    // The commit's own after-landing wake is one of these too: the service that answered the commit is running.
    let aftermath: &str = include_str!("../landing/service_landed_aftermath.rs");
    assert!(aftermath.contains("start_wait: HAND_OVER_GREETING_WAIT"), "the after-landing wake waits the hand-over's bound");
    // The commit's own request keeps the wait its start needs.
    let commit: &str = include_str!("../landing/service_commit_input_notes_and_embedded_commit.rs");
    assert!(commit.contains("start_wait: START_WAIT"), "a commit's own request keeps the start wait");
    assert!(commit.contains("start_ceiling: crate::wait_ceilings::START_CEILING"), "and the start ceiling");
}

/// What:
///  A unit the manager reports as running is given less than the start ceiling before it is asked to restart.
/// Why:
///   A service greets a connection before it restores anything,
/// so a running unit that greets nothing is one that stopped
///  inside its own process;
/// the wait for it is bounded by the greeting silence and never by the whole start ceiling,
/// which
///  would hold the commit for a minute.
#[test]
fn a_running_unit_that_greets_nothing_is_restarted_within_the_start_ceiling() {
    assert!(SERVICE_GREETING_SILENCE < START_CEILING, "{SERVICE_GREETING_SILENCE:?} must end before {START_CEILING:?}");
    assert_eq!(SERVICE_GREETING_SILENCE, Duration::from_secs(3), "nearly three times the slowest of 52 measured cold starts, and under 5 s");
    let reach: &str = include_str!("service_client_reach_service.rs");
    assert!(reach.contains("SERVICE_GREETING_SILENCE"), "the client is what asks for the restart");
    assert!(reach.contains("(watch.restart)("), "and the restart is a call of the service manager");
}

/// What:
///  A repair tool is announced before either of the two bounds that end it,
/// and long after a tool that works quickly has finished.
/// Why:
///   The quiet window is the owner's choice of 2026-11-04 and stays 600 s;
/// the notice says what runs and what ends it,
///  and it must come early enough that a person watching an add is not left without a word for ten minutes.
#[test]
fn a_repair_tool_is_announced_well_before_its_bounds_end_it() {
    assert_eq!(REPAIR_TOOL_NOTICE_AFTER, Duration::from_secs(2));
    assert!(REPAIR_TOOL_NOTICE_AFTER < REPAIR_TOOL_CEILING, "the notice comes long before the ceiling");
    let request: &str = include_str!("../repair/package_repair_request.rs");
    assert!(request.contains("const TOOL_QUIET_WINDOW: Duration = Duration::from_secs(600);"), "the owner's quiet window is still 600 s and this change leaves it alone");
    let command: &str = include_str!("../repair/package_repair_command.rs");
    assert!(command.contains("REPAIR_TOOL_NOTICE_AFTER"), "the run of every repair tool is what waits");
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

/// The start of one repair tool is bounded by the wait a client grants a service's own start:
/// both are one program's start on this host,
/// and the repair's run takes that bound from this file.
/// A start that does not answer waits for ever otherwise,
/// because the quiet window,
/// the ceiling and the notice are read from a clock the run reaches only after the start.
#[test]
fn a_repair_tool_start_is_bounded_by_the_service_start_wait() {
    assert_eq!(REPAIR_TOOL_START_CEILING, START_CEILING);
    let run: &str = include_str!("../repair/package_repair_command.rs");
    assert!(run.contains("REPAIR_TOOL_START_CEILING"), "the repair's run takes this bound");
    let start: &str = include_str!("../repair/package_repair_start.rs");
    assert!(start.contains("async fn start_within("), "the start of a tool is the bounded one");
}
