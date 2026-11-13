//! Controls for the one decision two enforcers of a step's ceiling share.

use super::*;
use crate::scratch_step_held_by_the_kernel::HeldProgram;

/// A stand-aside of a step the kernel held,
/// counted from the step's start.
fn held_at(waited: Duration, bound: Duration) -> TheStallItWaitsThrough {
    return TheStallItWaitsThrough {
        stood_aside_after: waited,
        bound,
        why: WhyTheStepWasWaitedFor::TheKernelHeldIt(HeldProgram { number: 4242, name: String::from("git") }),
    };
}

/// What:
///  A ceiling nothing is decided about stands aside when one enforcer sees the kernel holding the step,
///  and the other enforcer reads that same stand-aside.
/// Why:
///   Two enforcers that each decided for themselves could disagree,
///  and the one that did not see the hold ended the step at the usual ceiling
///  (findings 1 and 2 of the 2026-11-12 review of the stand-aside).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const stall = new TheStallDecidedOnce(); expect(stall.state()).toBe('undecided'); const after = stall.standAside(a); expect(after).toEqual({ stoodAside: a }); expect(stall.state()).toEqual({ stoodAside: a });
/// ```
#[test]
fn a_ceiling_nothing_is_decided_about_stands_aside_once_and_both_enforcers_read_it() {
    let stall: TheStallDecidedOnce = TheStallDecidedOnce::new();
    assert_eq!(stall.state(), TheStall::Undecided, "a new ceiling is decided about nothing");
    let stood: TheStallItWaitsThrough = held_at(Duration::from_secs(60), Duration::from_secs(300));
    assert_eq!(stall.stand_aside(stood.clone()), TheStall::StoodAside(stood.clone()), "the enforcer that saw the hold stands aside");
    assert_eq!(stall.state(), TheStall::StoodAside(stood.clone()), "the other enforcer reads the same stand-aside");
    assert!(!stall.the_usual_ceiling_ends_it(), "a ceiling that stood aside cannot have the usual ceiling end the step afterwards");
    assert_eq!(stall.state(), TheStall::StoodAside(stood), "and the stand-aside it keeps is the one that was made");
}

/// What:
///  The first stand-aside is the one the whole step keeps,
///  and a second one does not replace its bound.
/// Why:
///   The bound the words name and the wait the stop records must be one wait and not two,
///  whoever of the two enforcers saw the hold first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // stall.standAside(a); expect(stall.standAside(b)).toEqual({ stoodAside: a });
/// ```
#[test]
fn the_first_stand_aside_is_the_one_the_step_keeps() {
    let stall: TheStallDecidedOnce = TheStallDecidedOnce::new();
    let first: TheStallItWaitsThrough = held_at(Duration::from_secs(60), Duration::from_secs(300));
    let second: TheStallItWaitsThrough = held_at(Duration::from_millis(1), Duration::from_secs(600));
    let _stood: TheStall = stall.stand_aside(first.clone());
    assert_eq!(stall.stand_aside(second), TheStall::StoodAside(first), "the whole step runs under one bound");
}

/// What:
///  A ceiling already ending the step cannot stand aside afterwards.
/// Why:
///   The words of a stop name the bound that really ended it:
///  a stand-aside decided after the usual ceiling was claimed would print and record a wait
///  that never happened (finding 2 of the 2026-11-12 review).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // expect(stall.theUsualCeilingEndsIt()).toBe(true); expect(stall.standAside(a)).toBe('the-usual-ceiling-ends-it');
/// ```
#[test]
fn a_ceiling_already_ending_the_step_cannot_stand_aside() {
    let stall: TheStallDecidedOnce = TheStallDecidedOnce::new();
    assert!(stall.the_usual_ceiling_ends_it(), "nothing was decided, so the usual ceiling ends the step");
    assert_eq!(stall.state(), TheStall::TheUsualCeilingEndsIt);
    assert_eq!(
        stall.stand_aside(held_at(Duration::from_secs(60), Duration::from_secs(300))),
        TheStall::TheUsualCeilingEndsIt,
        "a step the usual ceiling ends is never said to be waited for"
    );
}

/// What:
///  A ceiling that already stood aside stays so when the enforcer that found nothing asks.
/// Why:
///   That is the disagreement that failed a commit at the usual ceiling while the person had been told
///  it would be waited for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // stall.standAside(a); expect(stall.theUsualCeilingEndsIt()).toBe(false);
/// ```
#[test]
fn a_stand_aside_is_not_undone_by_the_enforcer_that_found_nothing() {
    let stall: TheStallDecidedOnce = TheStallDecidedOnce::new();
    let stood: TheStallItWaitsThrough = held_at(Duration::from_secs(60), Duration::from_secs(300));
    let _decided: TheStall = stall.stand_aside(stood.clone());
    assert!(!stall.the_usual_ceiling_ends_it(), "the ceiling that stood aside stays aside");
    assert_eq!(stall.state(), TheStall::StoodAside(stood));
}
