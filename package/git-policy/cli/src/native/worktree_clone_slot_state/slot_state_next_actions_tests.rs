//! Controls for the sentences a report is made of:
//!  the registration line for each of its three arms,
//!  the evidence lines for a record with no clone yet,
//!  the mounting intent's own placeholder,
//!  and the settling command's sentence,
//!  which says the command cannot be run in this release.

use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_slot_state::slot_state_classification::{
    ProofStoppedBy,
    SlotRouteState,
    classify_slot,
};
use crate::worktree_clone_slot_state::slot_state_evidence::evidence;
use crate::worktree_clone_slot_state::slot_state_mounts::{
    MountRows,
    SlotMount,
};
use crate::worktree_clone_slot_state::slot_state_next_actions::{
    next_safe_action,
    registration_sentence,
};
use crate::worktree_clone_slot_state::slot_state_readings::{
    BindingReading,
    IntentFacts,
    PlanReading,
    SlotReadings,
};
use crate::worktree_clone_slot_state::slot_state_test_records::{
    CLONE_DATASET,
    record_at,
};
use std::path::PathBuf;

/// The fixture's slot name.
fn slot() -> &'static str {
    return "topic";
}

/// The fixture's slot path.
fn slot_path() -> PathBuf {
    return PathBuf::from("/repo/.claude/worktrees/topic");
}

/// The readings of a record at one phase with nothing mounted.
fn readings_at(phase: Phase) -> SlotReadings {
    return SlotReadings {
        slot: String::from(slot()),
        slot_path: slot_path(),
        binding: BindingReading::Record(Box::new(record_at(phase, slot_path().as_path()))),
        plan: PlanReading::NoPlan,
        intent: IntentFacts::Absent,
        mounts: MountRows::Rows(Vec::new()),
        slot_mount: SlotMount::Nothing,
        registration_stands: None,
    };
}

/// Each registration arm says a different thing about Git's own list.
#[test]
fn each_registration_arm_says_what_git_lists() {
    assert!(
        registration_sentence(Some(true)).contains("lists this worktree"),
        "{}",
        registration_sentence(Some(true))
    );
    assert!(
        registration_sentence(Some(false)).contains("no longer lists"),
        "{}",
        registration_sentence(Some(false))
    );
    assert!(
        registration_sentence(None).contains("names no Git registration yet"),
        "{}",
        registration_sentence(None)
    );
}

/// The evidence of an unfinished add names its phase and says the intent was not asked.
#[test]
fn an_unfinished_adds_evidence_names_its_phase() {
    let readings: SlotReadings = SlotReadings {
        intent: IntentFacts::Unasked,
        ..readings_at(Phase::Registered)
    };
    let lines: Vec<String> = evidence(&readings);
    assert!(lines.iter().any(|line| return line == "binding: phase=registered cloneDataset=`tank/monochromatic-git-worktree-clone-datasets/topic` pool=`tank`"), "{lines:?}");
    assert!(
        lines.iter().any(|line| return line
            == "intent: not asked, because the record names no clone dataset yet"),
        "{lines:?}"
    );
    assert!(
        lines
            .iter()
            .any(|line| return line == "plan: no destruction plan names this clone"),
        "{lines:?}"
    );
    assert!(
        lines.iter().any(|line| return line
            == "registration: not read, and the record names /repo/.git/worktrees/topic"),
        "{lines:?}"
    );
}

/// A slot left at `phase=settling` names the add again, Git's removal and the
/// settling command,
/// and says the settling command cannot be run in this release,
/// so a person is not sent to one that answers with the usage text.
#[test]
fn the_settling_arm_says_the_settling_command_cannot_be_run_yet() {
    let readings: SlotReadings = SlotReadings {
        intent: IntentFacts::Unasked,
        ..readings_at(Phase::Settling)
    };
    let state: SlotRouteState = classify_slot(&readings);
    let action: String = next_safe_action(&state, &readings);
    assert!(
        action.contains("settle-worktree-clone-record"),
        "the command the state needs is named: {action}"
    );
    assert!(
        action.contains("no management grammar admits it in this release"),
        "and the sentence says it cannot be run: {action}"
    );
    assert!(
        action.contains("cannot be run yet"),
        "with what a person does instead: {action}"
    );
    assert!(
        action.contains(CLONE_DATASET),
        "and the clone's dataset: {action}"
    );
}

/// The unmounted arm's action names the property and the sweep,
/// so a person can act on it without reading the code.
#[test]
fn the_unmounted_action_names_the_property_and_the_sweep() {
    let readings: SlotReadings = readings_at(Phase::Filled);
    let state: SlotRouteState = classify_slot(&readings);
    let action: String = next_safe_action(&state, &readings);
    assert!(
        action.contains("org.monochromatic:git-worktree-clone-mount-desired-state"),
        "{action}"
    );
    assert!(action.contains(CLONE_DATASET), "{action}");
    let waiting: SlotReadings = SlotReadings {
        intent: IntentFacts::Mounting {
            nonce: String::from("9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f"),
        },
        ..readings_at(Phase::Filled)
    };
    let waited: String = next_safe_action(&classify_slot(&waiting), &waiting);
    assert!(
        waited.contains("9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f"),
        "{waited}"
    );
    assert!(
        waited.contains("journalctl -u reconcile-git-worktree-clone-mounts.service"),
        "{waited}"
    );
}

/// A state nothing proves tells the reader to change nothing and names the source.
#[test]
fn a_state_nothing_proves_changes_nothing() {
    let readings: SlotReadings = readings_at(Phase::Filled);
    let unproved: SlotRouteState = SlotRouteState::CannotProve {
        reason: String::from("the mount table could not be read"),
        stopped_by: ProofStoppedBy::AMountThatCannotBeRead,
    };
    let action: String = next_safe_action(&unproved, &readings);
    assert!(
        action.contains("the mount table could not be read"),
        "{action}"
    );
    assert!(
        action.contains("do not move or remove the slot"),
        "{action}"
    );
}
