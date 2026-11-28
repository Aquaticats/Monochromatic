//! What:
//!  Every way the settling command stops,
//!  sorted into the caller's mistakes and the machine's states,
//!  and read back through the status each ends in.
//! Why:
//!  A new stop that nobody sorts would read `1` beside the others,
//!  and a person who mistyped a flag would learn that the machine failed,
//!  so this control names every arm of the type and the status each one owes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(stops.map(exitStatusOf)).toEqual([2, 2, 2, 1, 1, ...]);
//! ```

use crate::worktree_clone_record_settling::settling_refusal::SettlingStop;
use std::path::PathBuf;

/// What:
///  The stops the words and the two values a person types can make,
///  which end in the caller's own status.
/// Why:
///  These three are decided from the arguments alone,
///  before any record is read,
///  so they are the caller's and not the machine's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mistakes = [usage(), placeholder(), twice()];
/// ```
fn the_callers_mistakes() -> Vec<SettlingStop> {
    return vec![
        SettlingStop::Usage {
            said: String::from("the words are not this command's"),
        },
        SettlingStop::PlaceholderLeft {
            flag: String::from("--confirm-removal-of"),
            value: String::from("<slot>"),
        },
        SettlingStop::TheSlotIsNamedTwice {
            asked: String::from("one"),
            confirmed: String::from("two"),
        },
    ];
}

/// What:
///  One stop of every other arm of the type,
///  which ends in the status of a state that prevents the command.
/// Why:
///  The list is the type's own arms,
///  so a reader compares it with the enum and sees an arm nobody added here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const states = [notThisRecord(), guidDiffers(), noRecord(), ...];
/// ```
fn the_states_that_prevent_the_command() -> Vec<SettlingStop> {
    let path: PathBuf = PathBuf::from("/tmp/one.binding");
    let said: String = String::from("the read refused");
    return vec![
        SettlingStop::TheSlotIsNotThisRecord {
            asked: String::from("one"),
            recorded: String::from("two"),
        },
        SettlingStop::TheCloneGuidDiffers {
            typed: String::from("3102"),
            wanted: String::from("3103"),
        },
        SettlingStop::NoRecord {
            path: path.clone(),
        },
        SettlingStop::RecordUnreadable {
            path: path.clone(),
            said: said.clone(),
        },
        SettlingStop::NoCloneRecorded {
            slot: String::from("one"),
            phase: String::from("registered"),
        },
        SettlingStop::TheRecordIsIncomplete {
            said: String::from("the record holds no cloneGuid"),
        },
        SettlingStop::TheSlotCouldNotBeRead {
            path: path.clone(),
            said: said.clone(),
        },
        SettlingStop::TheClaimIsHeld {
            slot: String::from("one"),
            pid: 4321,
        },
        SettlingStop::TheClaimCannotBeTaken {
            slot: String::from("one"),
            said: said.clone(),
        },
        SettlingStop::TheCloneStillStands {
            clone: String::from("pool/clones/one"),
            guid: String::from("4821193048571903102"),
        },
        SettlingStop::TheNameHoldsAnother {
            name: String::from("pool/clones/one"),
            guid: String::from("4821193048571903102"),
        },
        SettlingStop::TheCloneAnswersElsewhere {
            found: String::from("pool/clones/two"),
        },
        SettlingStop::AMountAtTheSlot {
            described: said.clone(),
        },
        SettlingStop::TheMountTableCouldNotBeRead {
            said: said.clone(),
        },
        SettlingStop::ThePointerNamesAnother {
            found: String::from("another registration"),
        },
        SettlingStop::TheRegistrationIsAnother {
            recorded: String::from("1:2:3:4"),
            live: String::from("1:2:3:5"),
        },
        SettlingStop::TheRegistrationDoesNotAnswer {
            path: path.clone(),
            said: said.clone(),
        },
        SettlingStop::TheLockIsNotThisAdds {
            path: path.clone(),
        },
        SettlingStop::TheOriginCleanupDisagrees {
            field: String::from("originSnapshot"),
            recorded: String::from("pool/checkout@git-worktree-clone-origin-2f2f"),
            standing: String::from("pool/checkout@git-worktree-clone-origin-3a3a"),
        },
        SettlingStop::TheOriginCleanupCouldNotBeRead {
            path: path.clone(),
            said: said.clone(),
        },
        SettlingStop::ThePlanCouldNotBeRead {
            path: path.clone(),
            said: said.clone(),
        },
        SettlingStop::ThePlanIsTheFinishPasss {
            path: path.clone(),
        },
        SettlingStop::ThePlanDisagrees {
            field: String::from("head"),
            binding: String::from("present"),
            plan: String::from("gone"),
        },
        SettlingStop::TheRecordCouldNotGo {
            path: path.clone(),
            said: said.clone(),
        },
        SettlingStop::TheRecipeStopped {
            said: said.clone(),
        },
        SettlingStop::TheClaimCannotBeReleased {
            slot: String::from("one"),
            said,
        },
    ];
}

/// What:
///  That the words and the confirmations end in `2` and every state of the
///  machine ends in `1`.
/// Why:
///  One status per kind is the rule,
///  and a caller who cannot tell a typo from a refusing machine is the reason
///  the two exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(the_callers_mistakes().map(exitStatusOf)).toEqual([2, 2, 2]);
/// expect(the_states_that_prevent_the_command().every((stop) => exitStatusOf(stop) === 1)).toBe(true);
/// ```
#[test]
fn a_callers_mistake_ends_in_two_and_a_state_in_one() {
    for stop in the_callers_mistakes() {
        assert_eq!(
            super::exit_status_of(&stop),
            2,
            "a caller's own mistake reads the caller's status: {stop:?}"
        );
    }
    for stop in the_states_that_prevent_the_command() {
        assert_eq!(
            super::exit_status_of(&stop),
            1,
            "a state that prevents the command reads the failure status: {stop:?}"
        );
    }
}
