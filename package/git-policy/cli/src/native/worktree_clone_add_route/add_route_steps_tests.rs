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

/// What:
///  The clone step reports a clone whose `mountpoint` is not the container's
///  `legacy`,
///  every time the clone answers,
///  before it publishes the clone's inventory.
/// Why:
///  The report is the add's half of the design's two checks,
///  and a step that skipped it would record and mount a clone the handler's
///  sweep refuses
///  (`doc/decision/cli-git-zfs-clone-inherits-legacy-from-the-container.md`).
///  The step opens the kernel's device, so the shape a release file must have is
///  read from the source,
///  as the preflight's own controls do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(source.match(/refuseAClone/)).toHaveLength(2); // the definition and one call, in stepClone
/// ```
#[test]
fn the_clone_step_reports_the_clones_mountpoint_before_it_publishes_the_inventory() {
    let source: &str = include_str!("add_route_steps.rs");
    let code: Vec<&str> = source
        .lines()
        .map(str::trim_start)
        .filter(|line| return !line.starts_with("//"))
        .collect();
    let joined: String = code.join("\n");
    assert_eq!(joined.matches("refuse_a_clone_that_is_not_legacy_from_the_container(").count(), 2, "the report is defined once and called once: {joined}");
    let clone_step: usize = joined.find("pub fn step_clone(").expect("the clone step is a function of this file");
    let call: usize = clone_step + joined[clone_step..].find("refuse_a_clone_that_is_not_legacy_from_the_container(").expect("the call is in the clone step");
    let inventory: usize = joined.find("let inventory = steps.device.clone_inventory(").expect("the clone step publishes the clone's inventory");
    assert!(call < inventory, "the report stands before the clone is recorded, so a clone the add cannot use is never written down as its own");
}
