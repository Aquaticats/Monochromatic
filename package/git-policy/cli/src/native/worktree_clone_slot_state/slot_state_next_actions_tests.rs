//! Controls for the sentences a report is made of:
//!  the registration line for each of its three arms,
//!  the evidence lines for a record with no clone yet,
//!  the mounting intent's own placeholder,
//!  the settling command's sentence,
//!  and the sentence a slot with a standing destruction plan gets.

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
///  A slot a destruction plan names says what `git worktree remove` there does:
///  the wrapper's own removal,
///  which resumes the plan,
///  so the sentence neither warns a person off it nor promises it in the words
///  every other state uses.
/// Why:
///  The removal's command path takes a slot a destruction plan names,
///  because the design's resume path reads the plan and the live pinned parent
///  and no other fact decides it
///  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
///  "Which resume path a removal takes"),
///  and Git's own removal there would drop the registration and the slot folder
///  while the plan and the clone stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const action = nextSafeAction(classifySlot(pendingRemoval), pendingRemoval);
/// ```
#[test]
fn a_standing_plan_says_what_the_removal_resumes() {
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
        action.contains("A `git worktree remove` of this slot runs the wrapper's own removal"),
        "the sentence names what runs there: {action}"
    );
    for said in [
        "reads the plan",
        "proves every name the plan records again",
        "sends no request for a name the plan records as destroyed",
        "ends the plan with the removal's own records",
        "`-f -f` among them",
        "an add here is refused while the plan stands",
    ] {
        assert!(action.contains(said), "the sentence says {said}: {action}");
    }
    assert!(
        !action.contains("is Git's own removal"),
        "Git's own removal is not what runs here: {action}"
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

/// What:
///  The two states the owner's answers of 2026-11-28 added name their own way on.
/// Why:
///  The sentence a person reads beside a slot says what is safe next,
///  and for these two states the safe things are the settling command and a
///  forced removal for the plan,
///  and the cancel command and a forced removal for the staged clone
///  (`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`).
#[test]
fn the_two_new_readings_name_the_commands_that_end_them() {
    let readings: SlotReadings = readings_at(Phase::Filled);
    let plan: String = next_safe_action(
        &SlotRouteState::CannotProve {
            reason: String::from("the destruction plan at /plans/1 could not be read"),
            stopped_by: ProofStoppedBy::ADestructionPlanThatCannotBeRead,
        },
        &readings,
    );
    for said in [
        "git monochromatic-git-policy-cli settle-worktree-clone-record topic",
        "git worktree remove --force topic",
        "from the pool's own list",
    ] {
        assert!(plan.contains(said), "the sentence says {said}: {plan}");
    }
    let staged: String = next_safe_action(
        &SlotRouteState::CannotProve {
            reason: String::from("the clone is mounted on the staging folder"),
            stopped_by: ProofStoppedBy::ACloneAtTheStagingFolder,
        },
        &readings,
    );
    for said in [
        "git monochromatic-git-policy-cli cancel-worktree-removal topic",
        "git worktree remove --force topic",
    ] {
        assert!(staged.contains(said), "the sentence says {said}: {staged}");
    }
}
