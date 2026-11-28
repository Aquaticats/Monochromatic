//! Controls for the sentences a report is made of:
//!  the registration line for each of its three arms,
//!  the evidence lines for a record with no clone yet,
//!  the mounting intent's own placeholder,
//!  the settling command's sentence,
//!  and the sentence a slot with a standing destruction plan gets,
//!  which names no removal this release runs there.

use crate::worktree_clone_binding_record::add_state_commands::THE_REMOVALS_WORDS;
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

/// A slot left at `phase=settling` names the add again,
/// Git's removal and the
/// settling command,
/// and claims no grammar refuses the settling command,
/// so a person is not sent to one that answers with the usage text.
#[test]
fn the_settling_arm_names_the_settling_command_and_what_it_removes() {
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
        !action.contains("no management grammar admits it"),
        "the sentence no longer claims no grammar admits the command, because one does: {action}"
    );
    assert!(
        action.contains("once you have confirmed the slot"),
        "with what the command asks a person to confirm: {action}"
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

/// What:
///  A slot a destruction plan names is warned off `git worktree remove`,
///  and the words that say the wrapper's removal runs there are not printed.
/// Why:
///  The command path takes only a slot it proves holds a whole clone,
///  and a slot with a standing plan is a class of its own,
///  so the sentence cannot say the wrapper's removal is what runs there:
///  Git's own removal is,
///  and it would drop the registration and the slot folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const action = nextSafeAction(classifySlot(pendingRemoval), pendingRemoval);
/// ```
#[test]
fn a_standing_plan_warns_off_git_rather_than_naming_the_removal() {
    let readings: SlotReadings = SlotReadings {
        plan: PlanReading::NamesTheClone {
            plan: PathBuf::from(
                "/repo/.git/monochromatic-git-policy-cli/clone-destruction-plans/topic",
            ),
        },
        ..readings_at(Phase::Filled)
    };
    let state: SlotRouteState = classify_slot(&readings);
    let action: String = next_safe_action(&state, &readings);
    assert!(
        action.contains("do not run `git worktree remove`"),
        "the warning comes first: {action}"
    );
    assert!(!action.contains(THE_REMOVALS_WORDS), "{action}");
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
