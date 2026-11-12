//! Controls for the record of one stop of the scratch `git commit`.

use super::*;
use crate::scratch_step_held_by_the_kernel::HeldProgram;
use crate::scratch_step_stall_decided_once::{TheStallItWaitsThrough, WhyTheStepWasWaitedFor};
use crate::scratch_step_stop_words::stop_words;

/// The usual ceiling a control gives a commit.
const THE_USUAL_CEILING: Duration = Duration::from_secs(60);
/// The longer bound a control stands aside for.
const A_STALL_BOUND: Duration = Duration::from_secs(300);

/// The stand-aside a ceiling makes for a step the kernel held.
fn stood_aside() -> TheStallItWaitsThrough {
    return TheStallItWaitsThrough {
        stood_aside_after: THE_USUAL_CEILING,
        bound: A_STALL_BOUND,
        why: WhyTheStepWasWaitedFor::TheKernelHeldIt(HeldProgram { number: 4242, name: String::from("gpg") }),
    };
}

/// A stop the wait that owns the child answered.
fn the_waits_stop(waited: Duration, stall: Option<TheStallItWaitsThrough>) -> CeilingStop {
    return CeilingStop { waited, below_listed: true, below_unended: 0, stall };
}

/// What:
///  A step the wait ended at the usual ceiling names the usual bound,
///  even when the notice's record carries a bound a ceiling stood aside for.
/// Why:
///   Finding 2 of the 2026-11-12 review:
///  the record merge kept the notice's stall ceiling and took the wait's stop,
///  so the person read that the step was stopped after 60 seconds at a 300 second bound and that it was waited for.
///  A stop is taken from the enforcer that ended the step,
///  and the bound is the one decision both enforcers share.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const facts = { stoppedAfter: undefined, stallCeiling: 300_000, heldAfterCeiling: 60_000 }; const stop = theStopOfTheStep({ facts, theWaitsStop: { waited: 60_000, stall: undefined }, theStall: 'the-usual-ceiling-ends-it', theUsualCeiling: 60_000 });
/// // expect(stop.boundThatEndedIt).toEqual({ theUsualCeiling: 60_000 })
/// ```
#[test]
fn a_stop_the_wait_ended_at_the_usual_ceiling_names_the_usual_bound() {
    let facts: ScratchStepFacts = ScratchStepFacts {
        stopped_after: None,
        stall_ceiling: Some(A_STALL_BOUND),
        held_after_ceiling: Some(THE_USUAL_CEILING),
        below_unlisted: false,
        ..ScratchStepFacts::unarmed(Some(THE_USUAL_CEILING))
    };
    let stop: CeilingStop = the_waits_stop(THE_USUAL_CEILING, None);
    let the_stop: ScratchStepStop = the_stop_of_the_step(TheEnforcersOfAStep {
        facts: &facts,
        the_waits_stop: Some(&stop),
        the_stall: &TheStall::TheUsualCeilingEndsIt,
        the_usual_ceiling: THE_USUAL_CEILING,
        signs: false,
    })
    .expect("the wait ended the step");
    assert_eq!(the_stop.waited, THE_USUAL_CEILING, "the words say how long the step lasted");
    assert_eq!(
        the_stop.bound_that_ended_it,
        TheBoundThatEndedACommit::TheUsualCeiling(THE_USUAL_CEILING),
        "the words name the bound that ended it and never one a ceiling stood aside for"
    );
    let said: String = stop_words(the_stop);
    assert!(!said.contains("waited for to that bound"), "a step killed at the usual ceiling was not waited for: {said}");
    assert!(!said.contains("(300 seconds)"), "{said}");
}

/// What:
///  A step the wait ended at the bound its ceiling stood aside for names that bound and why.
/// Why:
///   The person is told the bound the step really ran under,
///  and the reason is part of it:
///  a step the kernel held and one nothing could be learned about are different things to check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const stop = theStopOfTheStep({ facts, theWaitsStop: { waited: 300_000, stall: stoodAside }, theStall: { stoodAside }, theUsualCeiling: 60_000 });
/// // expect(stopStop.boundThatEndedIt).toEqual({ theStallBound: { bound: 300_000, why: 'the-kernel-held-it' } })
/// ```
#[test]
fn a_stop_the_wait_ended_at_the_bound_it_waited_for_names_that_bound() {
    let stood: TheStallItWaitsThrough = stood_aside();
    let facts: ScratchStepFacts = ScratchStepFacts::unarmed(Some(THE_USUAL_CEILING));
    let stop: CeilingStop = the_waits_stop(A_STALL_BOUND, Some(stood.clone()));
    let the_stop: ScratchStepStop = the_stop_of_the_step(TheEnforcersOfAStep {
        facts: &facts,
        the_waits_stop: Some(&stop),
        the_stall: &TheStall::StoodAside(stood.clone()),
        the_usual_ceiling: THE_USUAL_CEILING,
        signs: false,
    })
    .expect("the wait ended the step");
    assert_eq!(
        the_stop.bound_that_ended_it,
        TheBoundThatEndedACommit::TheStallBound { bound: A_STALL_BOUND, why: stood.why },
        "the words name the bound the step ran under and why it was waited for"
    );
    let said: String = stop_words(the_stop);
    assert!(said.contains("held it in a disk wait") && said.contains("(300 seconds)"), "{said}");
}

/// What:
///  A step the notice's thread ended names the bound that really ended it and the programs it could not list.
/// Why:
///   The notice's own stop is the record of that stop:
///  the wait that owns the child answered nothing, because the step ended at the notice's signal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const facts = { stoppedAfter: 60_000, belowUnlisted: true, belowUnended: 2 }; const stop = theStopOfTheStep({ facts, theWaitsStop: undefined, theStall: 'the-usual-ceiling-ends-it', theUsualCeiling: 60_000 });
/// // expect(stopStop.belowUnlisted).toBe(true)
/// ```
#[test]
fn a_stop_the_notices_thread_ended_is_the_notices_record() {
    let facts: ScratchStepFacts = ScratchStepFacts {
        stopped_after: Some(THE_USUAL_CEILING),
        below_unlisted: true,
        below_unended: 2,
        ..ScratchStepFacts::unarmed(Some(THE_USUAL_CEILING))
    };
    let the_stop: ScratchStepStop = the_stop_of_the_step(TheEnforcersOfAStep {
        facts: &facts,
        the_waits_stop: None,
        the_stall: &TheStall::TheUsualCeilingEndsIt,
        the_usual_ceiling: THE_USUAL_CEILING,
        signs: true,
    })
    .expect("the notice's thread ended the step");
    assert_eq!(the_stop.waited, THE_USUAL_CEILING);
    assert!(the_stop.below_unlisted && the_stop.below_unended == 2, "a program that may still run is named");
    assert!(the_stop.signs, "the words name the signing program");
}

/// What:
///  A step nothing stopped has no words.
/// Why:
///   A commit that landed prints no stop,
///  and a stop invented for it would fail a commit that succeeded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // expect(theStopOfTheStep({ facts: { stoppedAfter: undefined }, theWaitsStop: undefined, ... })).toBeUndefined()
/// ```
#[test]
fn a_step_nothing_stopped_has_no_words() {
    let facts: ScratchStepFacts = ScratchStepFacts::unarmed(Some(THE_USUAL_CEILING));
    assert_eq!(
        the_stop_of_the_step(TheEnforcersOfAStep { facts: &facts, the_waits_stop: None, the_stall: &TheStall::Undecided, the_usual_ceiling: THE_USUAL_CEILING, signs: false }),
        None
    );
}
