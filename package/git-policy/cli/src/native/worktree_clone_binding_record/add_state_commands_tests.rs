//! Controls for the command each resume state names:
//!  a clone the pool cannot prove gone names the settling command, as one it can,
//!  a clone that stands names the removal,
//!  a settling record with no clone GUID names the add again,
//!  each head word of a standing plan names the command the plan design gives
//!  it,
//!  a plan's recorded name answering another clone names no command,
//!  and
//!  the printed settling line carries placeholders rather than the digits.

use crate::worktree_clone_binding_record::add_state_commands::{
    ClonePresence,
    NextCommand,
    PlanHeadWord,
    SETTLING_COMMAND_WORDS,
    StateFacts,
    command_for,
};
use crate::worktree_clone_binding_record::binding_words::Phase;

/// The facts of one state,
/// with the arms a test does not name left absent.
fn facts(phase: Phase) -> StateFacts {
    return StateFacts {
        phase,
        clone: None,
        registration_stands: true,
        plan_head: None,
        checkout_begun: false,
        holds_clone_guid: false,
    };
}

/// The facts of one state beside a standing plan,
/// with the head word the test names and a clone the pool proved gone.
fn facts_beside_a_plan(
    phase: Phase,
    head: PlanHeadWord,
    clone: Option<ClonePresence>,
) -> StateFacts {
    let mut found: StateFacts = facts(phase);
    found.plan_head = Some(head);
    found.clone = clone;
    found.holds_clone_guid = true;
    return found;
}

/// The clone presence a pool answers with,
/// named so a control reads as the fact it supplies.
const PROVED_GONE: Option<ClonePresence> = Some(ClonePresence::GoneProved);

/// An absence the pool cannot prove,
/// which is its own arm of the plan design's rule.
fn unprovable() -> Option<ClonePresence> {
    return Some(ClonePresence::GoneUnprovable {
        evidence: String::from("the pool refused the listing"),
    });
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
        unprovable.clone = Some(ClonePresence::GoneUnprovable {
            evidence: String::from("the pool refused the listing"),
        });
        unprovable.holds_clone_guid = true;
        assert_eq!(
            command_for(&proved),
            NextCommand::TheSettlingCommand,
            "phase `{}`",
            phase.word()
        );
        assert_eq!(
            command_for(&unprovable),
            NextCommand::TheSettlingCommand,
            "phase `{}`",
            phase.word()
        );
        assert_eq!(command_for(&proved), command_for(&unprovable));
    }
}

/// A verified record whose clone is gone names the settling command too,
/// while one whose clone stands continues.
#[test]
fn a_verified_record_names_the_settling_command_only_when_the_clone_is_gone() {
    let mut gone: StateFacts = facts(Phase::Filled);
    gone.clone = Some(ClonePresence::GoneUnprovable {
        evidence: String::from("the pool answered in part"),
    });
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
    assert_eq!(
        command_for(&proved_gone_no_registration),
        NextCommand::TheSettlingCommand
    );
}

/// A standing plan decides before the phase does,
/// so the add never removes what a removal's plan holds.
#[test]
fn a_standing_plan_names_the_removal() {
    let with_plan: StateFacts =
        facts_beside_a_plan(Phase::Settling, PlanHeadWord::Requested, PROVED_GONE);
    assert_eq!(command_for(&with_plan), NextCommand::TheRemoval);
}

/// Each head word of a standing plan names the command the plan design's
/// "Which resume path a removal takes" gives it beside a clone the pool proved
/// gone:
/// the removal for `requested`,
/// `unknown` and `gone`,
/// and the settling command for the outside destroy,
/// `present` and `refused`.
#[test]
fn each_head_word_names_the_command_the_plan_design_gives_it() {
    let wanted: [(PlanHeadWord, NextCommand); 5] = [
        (PlanHeadWord::Present, NextCommand::TheSettlingCommand),
        (PlanHeadWord::Requested, NextCommand::TheRemoval),
        (PlanHeadWord::Gone, NextCommand::TheRemoval),
        (PlanHeadWord::Refused, NextCommand::TheSettlingCommand),
        (PlanHeadWord::Unknown, NextCommand::TheRemoval),
    ];
    for (head, command) in wanted {
        for phase in [
            Phase::Cloned,
            Phase::Mounted,
            Phase::Filled,
            Phase::Settling,
        ] {
            assert_eq!(
                command_for(&facts_beside_a_plan(phase, head, PROVED_GONE)),
                command,
                "head `{}` at phase `{}`",
                head.word(),
                phase.word()
            );
        }
    }
}

/// A clone that answers under the pinned parent takes the removal's
/// head-present path whatever the plan's head word says,
/// because the design decides that before it reads the word.
#[test]
fn a_clone_that_stands_names_the_removal_for_every_head_word() {
    for head in [
        PlanHeadWord::Present,
        PlanHeadWord::Requested,
        PlanHeadWord::Gone,
        PlanHeadWord::Refused,
        PlanHeadWord::Unknown,
    ] {
        assert_eq!(
            command_for(&facts_beside_a_plan(
                Phase::Settling,
                head,
                Some(ClonePresence::Stands)
            )),
            NextCommand::TheRemoval,
            "head `{}`",
            head.word()
        );
    }
}

/// An absence the pool cannot prove names the settling command for every head
/// word but `gone`,
/// whose route is the finish pass and which no command accepts.
#[test]
fn an_unprovable_absence_names_the_settling_command_unless_the_head_is_gone() {
    for head in [
        PlanHeadWord::Present,
        PlanHeadWord::Requested,
        PlanHeadWord::Refused,
        PlanHeadWord::Unknown,
    ] {
        assert_eq!(
            command_for(&facts_beside_a_plan(Phase::Settling, head, unprovable())),
            NextCommand::TheSettlingCommand,
            "head `{}`",
            head.word()
        );
    }
    assert_eq!(
        command_for(&facts_beside_a_plan(
            Phase::Settling,
            PlanHeadWord::Gone,
            unprovable()
        )),
        NextCommand::TheFinishPass
    );
}

/// A reader that did not look for the clone,
/// which is the add's smallest plan read,
/// names the command by the head word alone:
/// the settling command for `present` and `refused`,
/// and the removal for the other three.
#[test]
fn a_reader_that_did_not_look_for_the_clone_names_the_command_by_the_head_word() {
    let wanted: [(PlanHeadWord, NextCommand); 5] = [
        (PlanHeadWord::Present, NextCommand::TheSettlingCommand),
        (PlanHeadWord::Requested, NextCommand::TheRemoval),
        (PlanHeadWord::Gone, NextCommand::TheRemoval),
        (PlanHeadWord::Refused, NextCommand::TheSettlingCommand),
        (PlanHeadWord::Unknown, NextCommand::TheRemoval),
    ];
    for (head, command) in wanted {
        assert_eq!(
            command_for(&facts_beside_a_plan(Phase::Claimed, head, None)),
            command,
            "head `{}`",
            head.word()
        );
    }
}

/// Each head word round-trips through the stored spelling,
/// and a word outside the plan design's five reads as no head word at all.
#[test]
fn a_head_word_reads_back_from_the_spelling_a_plan_stores() {
    for head in [
        PlanHeadWord::Present,
        PlanHeadWord::Requested,
        PlanHeadWord::Gone,
        PlanHeadWord::Refused,
        PlanHeadWord::Unknown,
    ] {
        assert_eq!(
            PlanHeadWord::parse(head.word()),
            Some(head),
            "`{}` reads back",
            head.word()
        );
    }
    for refused in ["", "finished", "Present", "present "] {
        assert_eq!(
            PlanHeadWord::parse(refused),
            None,
            "`{refused}` is no head word"
        );
    }
}

/// A recorded name that answers another clone's GUID is the one state no
/// command settles:
/// the removal refuses it,
/// so does the settling command,
/// and the finish pass leaves it standing,
/// so a person reads the records whatever the head word says.
#[test]
fn a_reused_name_names_no_command_for_every_head_word() {
    let reused: Option<ClonePresence> = Some(ClonePresence::TheNameAnswersAnotherClone {
        live_guid: String::from("4821193048571903103"),
    });
    for head in [
        PlanHeadWord::Present,
        PlanHeadWord::Requested,
        PlanHeadWord::Gone,
        PlanHeadWord::Refused,
        PlanHeadWord::Unknown,
    ] {
        assert_eq!(
            command_for(&facts_beside_a_plan(Phase::Settling, head, reused.clone())),
            NextCommand::APersonReadsTheRecords,
            "head `{}`",
            head.word()
        );
    }
}

/// The line for a reused name sends the person to the records and names no
/// command that would act on the slot.
#[test]
fn the_reused_names_line_names_the_doctor_and_no_command() {
    let line: String = NextCommand::APersonReadsTheRecords.printed_line("add-cache-layer");
    assert!(
        line.contains("no command of this release settles the state"),
        "{line}"
    );
    assert!(line.contains("doctor"), "{line}");
    for command in [
        SETTLING_COMMAND_WORDS,
        "git worktree remove",
        "cancel-worktree-removal",
    ] {
        assert!(
            !line.contains(command),
            "no command is named (`{command}`): {line}"
        );
    }
}

/// The printed settling line names the command and leaves both flags as placeholders,
/// so the line a person copies carries no value they must have read.
#[test]
fn the_settling_line_leaves_its_confirmations_as_placeholders() {
    let line: String = NextCommand::TheSettlingCommand.printed_line("add-cache-layer");
    assert!(line.contains(SETTLING_COMMAND_WORDS), "{line}");
    assert!(line.contains("--confirm-removal-of <slot>"), "{line}");
    assert!(
        line.contains("--confirm-clone-guid <the last four decimal digits of cloneGuid=>"),
        "{line}"
    );
    assert!(
        !line.contains("3102"),
        "the printed line carries no digit of the record's GUID: {line}"
    );
}
