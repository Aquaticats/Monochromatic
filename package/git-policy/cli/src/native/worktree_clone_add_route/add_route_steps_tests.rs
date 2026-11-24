//! Controls for the steps' own interface:
//!  a refusal keeps the wrapper's refused status and a Git failure keeps Git's,
//!  and the stop switch's steps are named in the add's order.

use super::StepRefusal;
use crate::subvolume_command::REFUSED_EXIT_CODE;
use crate::worktree_clone_add_route::add_route_plan::STOP_STEPS;

#[test]
fn a_refusal_and_a_git_failure_keep_their_statuses() {
    let refused: StepRefusal = StepRefusal::refused(String::from("the slot holds entries"));
    assert_eq!(refused.code, REFUSED_EXIT_CODE);
    assert_eq!(refused.message, "the slot holds entries");
    let failed: StepRefusal = StepRefusal::git(128);
    assert_eq!(failed.code, 128, "Git's own status reaches the person");
    assert!(failed.message.is_empty(), "a Git failure prints Git's own words");
}

#[test]
fn the_stop_switch_names_the_steps_the_guest_arms_kill_at() {
    for step in ["claim", "slot-identity", "registered", "origin-inventory", "clone", "clone-inventory", "mount", "exchange", "filled"] {
        assert!(STOP_STEPS.contains(&step), "{step} is a step the switch knows");
    }
    let ordered: Vec<&str> = STOP_STEPS.to_vec();
    let at = |step: &str| -> usize { return ordered.iter().position(|found| return *found == step).expect("the step is named"); };
    assert!(at("claim") < at("registered"));
    assert!(at("registered") < at("clone"));
    assert!(at("clone") < at("mount"));
    assert!(at("mount") < at("exchange"));
    assert!(at("exchange") < at("filled"));
}
