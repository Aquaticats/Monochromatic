//! Controls for the waits the forced arm would need:
//!  no constant has a value,
//!  the window refuses rather than defaulting,
//!  and a control's own window still computes the give-up as whole periods.

use crate::worktree_clone_forced_removal::forced_arm_numbers::*;
use std::time::Duration;

/// Every wait ships with no value,
/// which is what keeps the arm from running before step 22 sizes it.
#[test]
fn no_wait_has_a_value() {
    assert_eq!(TERMINATION_WAIT, None);
    assert_eq!(HEARTBEAT_PERIOD, None);
    assert_eq!(STOP_WAIT, None);
}

/// The window refuses,
/// naming the constant that stops it,
/// and the refusal's sentence names the same constant.
#[test]
fn the_window_refuses_and_names_the_constant() {
    let refused: RefusedClock =
        TimeoutWindow::of_the_constants().expect_err("no constant has a value");
    assert_eq!(refused, RefusedClock::Heartbeat);
    assert_eq!(refused.constant(), "T_h");
    let sentence: String = refused.to_string();
    assert!(sentence.contains("T_h"), "{sentence}");
    assert!(sentence.contains("step 22"), "{sentence}");
    assert_eq!(
        the_stop_wait().expect_err("T_stop has no value"),
        RefusedClock::StopWait
    );
    assert_eq!(RefusedClock::TerminationWait.constant(), "T_wait");
}

/// A control's own window computes its give-up window as whole heartbeat periods,
/// so the relation the design fixes is exercised even though no value ships.
#[test]
fn a_controls_window_counts_whole_periods() {
    let window: TimeoutWindow = TimeoutWindow::at(Duration::from_secs(10), Duration::from_secs(1));
    assert_eq!(window.termination(), Duration::from_secs(10));
    assert_eq!(window.heartbeat(), Duration::from_secs(1));
    assert_eq!(
        window.give_up(),
        Duration::from_secs(GIVE_UP_HEARTBEATS as u64)
    );
    assert_eq!(GIVE_UP_HEARTBEATS, 4);
}
