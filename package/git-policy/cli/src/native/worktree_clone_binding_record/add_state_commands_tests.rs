//! Controls for the command each resume state names:
//!  a clone the pool cannot prove gone names the settling command, as one it can,
//!  a clone that stands names the removal,
//!  a settling record with no clone GUID names the add again,
//!  a standing plan names the removal the plan belongs to,
//!  and the printed settling line carries placeholders rather than the digits.

use crate::worktree_clone_binding_record::add_state_commands::{ClonePresence, NextCommand, SETTLING_COMMAND_WORDS, StateFacts, command_for};
use crate::worktree_clone_binding_record::binding_words::Phase;

/// The facts of one state,
/// with the arms a test does not name left absent.
fn facts(phase: Phase) -> StateFacts {
    return StateFacts {
        phase,
        clone: None,
        registration_stands: true,
        plan_stands: false,
        checkout_begun: false,
        holds_clone_guid: false,
    };
}

/// A clone at `cloned` that is gone and cannot be proved gone names the settling command,
/// exactly as one that is proved gone does.
#[test]
fn an_unprovable_absence_names_the_settling_command_as_a_proved_one_does() {
    for phase in [Phase::Cloned, Phase::Mounted] {
        let mut proved: StateFacts = facts(phase);
        proved.clone = Some(ClonePresence::GoneProved);
        proved.holds_clone_guid = true;
        let mut unprovable: StateFacts = facts(phase);
        unprovable.clone = Some(ClonePresence::GoneUnprovable { evidence: String::from("the pool refused the listing") });
        unprovable.holds_clone_guid = true;
        assert_eq!(command_for(&proved), NextCommand::TheSettlingCommand, "phase `{}`", phase.word());
        assert_eq!(command_for(&unprovable), NextCommand::TheSettlingCommand, "phase `{}`", phase.word());
        assert_eq!(command_for(&proved), command_for(&unprovable));
    }
}

/// A verified record whose clone is gone names the settling command too,
/// while one whose clone stands continues.
#[test]
fn a_verified_record_names_the_settling_command_only_when_the_clone_is_gone() {
    let mut gone: StateFacts = facts(Phase::Filled);
    gone.clone = Some(ClonePresence::GoneUnprovable { evidence: String::from("the pool answered in part") });
    gone.holds_clone_guid = true;
    assert_eq!(command_for(&gone), NextCommand::TheSettlingCommand);
    let mut stands: StateFacts = facts(Phase::Filled);
    stands.clone = Some(ClonePresence::Stands);
    stands.holds_clone_guid = true;
    assert_eq!(command_for(&stands), NextCommand::None);
}

/// A settling record whose clone stands names the removal,
/// and one that holds no clone GUID names the add again.
#[test]
fn a_settling_record_names_the_removal_or_the_add_again() {
    let mut stands: StateFacts = facts(Phase::Settling);
    stands.clone = Some(ClonePresence::Stands);
    stands.holds_clone_guid = true;
    assert_eq!(command_for(&stands), NextCommand::GitWorktreeRemove);
    let mut no_clone: StateFacts = facts(Phase::Settling);
    no_clone.holds_clone_guid = false;
    assert_eq!(command_for(&no_clone), NextCommand::TheAddAgain);
    let mut proved_gone: StateFacts = facts(Phase::Settling);
    proved_gone.clone = Some(ClonePresence::GoneProved);
    proved_gone.holds_clone_guid = true;
    proved_gone.registration_stands = true;
    assert_eq!(command_for(&proved_gone), NextCommand::TheAddAgain);
    let mut proved_gone_no_registration: StateFacts = proved_gone.clone();
    proved_gone_no_registration.registration_stands = false;
    assert_eq!(command_for(&proved_gone_no_registration), NextCommand::TheSettlingCommand);
}

/// A standing plan decides before the phase does,
/// so the add never removes what a removal's plan holds.
#[test]
fn a_standing_plan_names_the_removal() {
    let mut with_plan: StateFacts = facts(Phase::Settling);
    with_plan.plan_stands = true;
    with_plan.clone = Some(ClonePresence::GoneProved);
    with_plan.holds_clone_guid = true;
    assert_eq!(command_for(&with_plan), NextCommand::TheRemoval);
}

/// The printed settling line names the command and leaves both flags as placeholders,
/// so the line a person copies carries no value they must have read.
#[test]
fn the_settling_line_leaves_its_confirmations_as_placeholders() {
    let line: String = NextCommand::TheSettlingCommand.printed_line("add-cache-layer");
    assert!(line.contains(SETTLING_COMMAND_WORDS), "{line}");
    assert!(line.contains("--confirm-removal-of <slot>"), "{line}");
    assert!(line.contains("--confirm-clone-guid <the last four decimal digits of cloneGuid=>"), "{line}");
    assert!(!line.contains("3102"), "the printed line carries no digit of the record's GUID: {line}");
}
