//! What:
//!  Controls of the one sentence a refusal prints about what the plan records,
//!  at both check points and for one, two and no destroyed snapshots.
//! Why:
//!  A refusal after the snapshot batch must name every snapshot already gone and never claim
//!  that nothing was destroyed,
//!  and a refusal before the batch must say that nothing was
//!  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  "The refusals" and control 11).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(RemovalHistory.ofDestroyed([]).sentence()).toBe('This removal has not destroyed anything yet.');
//! ```

use super::RemovalHistory;

/// The two snapshots the design's own example plan records.
fn two_snapshots() -> RemovalHistory {
    return RemovalHistory::of_destroyed(vec![String::from("fix-flaky-test@manual-2026-11-21"), String::from("fix-flaky-test@sanoid-hourly-2026-11-22")]);
}

#[test]
fn an_empty_plan_says_nothing_was_destroyed() {
    assert_eq!(RemovalHistory::NothingDestroyed.sentence(), "This removal has not destroyed anything yet.");
    assert_eq!(RemovalHistory::of_destroyed(Vec::new()).sentence(), "This removal has not destroyed anything yet.");
}

#[test]
fn one_destroyed_snapshot_is_named_with_the_singular_sentence() {
    let history: RemovalHistory = RemovalHistory::of_destroyed(vec![String::from("fix-flaky-test@manual-2026-11-21")]);
    assert_eq!(
        history.sentence(),
        "This clone has already lost `fix-flaky-test@manual-2026-11-21`, in this removal or an earlier one, and it does not come back."
    );
}

#[test]
fn two_destroyed_snapshots_are_both_named_and_joined() {
    assert_eq!(
        two_snapshots().sentence(),
        "This clone has already lost `fix-flaky-test@manual-2026-11-21` and `fix-flaky-test@sanoid-hourly-2026-11-22`, in this removal or an earlier one, and they do not come back."
    );
}

#[test]
fn three_destroyed_snapshots_separate_with_commas_and_join_the_last() {
    let history: RemovalHistory = RemovalHistory::of_destroyed(vec![String::from("a"), String::from("b"), String::from("c")]);
    assert!(history.sentence().contains("`a`, `b` and `c`"), "every name is there and the last is joined: {}", history.sentence());
}

#[test]
fn the_snapshots_are_readable_and_the_empty_history_holds_none() {
    assert_eq!(two_snapshots().lost_snapshots().len(), 2);
    assert!(RemovalHistory::NothingDestroyed.lost_snapshots().is_empty());
}

#[test]
fn the_sentence_never_says_this_run() {
    assert!(!two_snapshots().sentence().contains("this run"));
    assert!(two_snapshots().sentence().contains("in this removal or an earlier one"));
}
