//! Controls for when a stop may force the end of its tree: inside the grace it waits, past it with every member interruptible it
//!  forces, and past it with a member in uninterruptible sleep it waits until the ceiling and then forces.

use super::*;

/// A member that the controls call blocked or not by their own rule; no real process is looked at.
const MEMBER: TreeMember = TreeMember { pid: 1, started: 1 };

/// A grace and a ceiling for the controls.
const GRACE: Duration = Duration::from_secs(5);

/// A ceiling past the grace for the controls.
const CEILING: Duration = Duration::from_secs(18);

/// Every member is blocked.
fn blocked(_member: &TreeMember) -> bool {
    return true;
}

/// No member is blocked.
fn not_blocked(_member: &TreeMember) -> bool {
    return false;
}

/// Inside the grace a stop waits, whatever the members' states.
#[test]
fn inside_the_grace_a_stop_always_waits() {
    let _rules = rules_for_this_thread(ControlRules { blocked: not_blocked, ceiling: CEILING });
    assert!(escalation_waits(Duration::ZERO, GRACE, &[MEMBER]));
    assert!(escalation_waits(GRACE - Duration::from_millis(1), GRACE, &[MEMBER]));
}

/// Past the grace, with every member able to act on a signal, the stop forces the end at once, as it did before.
#[test]
fn past_the_grace_with_no_member_blocked_the_end_is_forced() {
    let _rules = rules_for_this_thread(ControlRules { blocked: not_blocked, ceiling: CEILING });
    assert!(!escalation_waits(GRACE, GRACE, &[MEMBER]));
    assert!(!escalation_waits(GRACE, GRACE, &[]));
}

/// Past the grace, with a member blocked, the stop waits until the ceiling and forces the end at it, so nothing waits for good.
#[test]
fn a_member_that_never_leaves_uninterruptible_sleep_is_forced_at_the_ceiling() {
    let _rules = rules_for_this_thread(ControlRules { blocked, ceiling: CEILING });
    assert!(escalation_waits(GRACE, GRACE, &[MEMBER]));
    assert!(escalation_waits(CEILING - Duration::from_millis(1), GRACE, &[MEMBER]));
    assert!(!escalation_waits(CEILING, GRACE, &[MEMBER]));
    assert!(!escalation_waits(CEILING * 100, GRACE, &[MEMBER]));
}

/// The real ceiling is the owner's multiple of the longest delay measured, and lies past the stop's grace.
#[test]
fn the_real_ceiling_is_three_times_the_worst_delay_and_lies_past_the_grace() {
    assert_eq!(STOP_CEILING, crate::worst_delay_seen_on_this_host::LONGEST_DELAY_SEEN * 3);
    assert!(STOP_CEILING > crate::bounded_push::STOP_GRACE);
}

/// The real state of this test's own process is not uninterruptible sleep while it runs the check, and a process that is gone is not.
#[cfg(target_os = "linux")]
#[test]
fn the_state_look_reads_a_real_process_and_refuses_a_gone_one() {
    let me: TreeMember = crate::process_tree::member_of(i32::try_from(std::process::id()).expect("pid")).expect("this process");
    assert!(!in_uninterruptible_sleep(&me), "a process running this check is not in uninterruptible sleep");
    assert!(crate::process_tree::state_letter_of(&me).is_some());
    assert!(!in_uninterruptible_sleep(&TreeMember { pid: i32::MAX, started: 0 }));
}
