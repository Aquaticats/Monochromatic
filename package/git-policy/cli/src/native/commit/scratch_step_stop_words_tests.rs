//! Controls for the words a stopped scratch `git commit` prints.

use super::*;
use crate::scratch_step_held_by_the_kernel::HeldProgram;
use crate::scratch_step_stall_decided_once::WhyTheStepWasWaitedFor;

/// One stop's words,
/// with the fields a control is not asking about left at their quietest.
fn a_stop(waited: u64, ceiling: u64, signs: bool, below_unlisted: bool, below_unended: u32) -> ScratchStepStop {
    return ScratchStepStop {
        waited: Duration::from_secs(waited),
        bound_that_ended_it: TheBoundThatEndedACommit::TheUsualCeiling(Duration::from_secs(ceiling)),
        signs,
        below_unlisted,
        below_unended,
    };
}

/// The words of a stop say what ended,
/// after how long,
/// why this long,
/// what became of the person's work,
/// and what to run.
#[test]
fn the_words_of_a_stop_say_what_ended_what_is_kept_and_what_to_run() {
    let said: String = stop_words(a_stop(1261, 1260, true, false, 0));
    assert!(said.starts_with("monochromatic-git-policy-cli: the `git commit` this commit runs in its scratch repository was stopped after 1261 seconds"), "{said}");
    assert!(said.contains("1260 seconds") && said.contains("three times"), "{said}");
    assert!(said.contains("signs the commit"), "a signed commit's words name the signer: {said}");
    assert!(said.contains("nothing was committed") && said.contains("index and files are as they were"), "{said}");
    assert!(said.contains("Run the same `git commit` command again"), "{said}");
    assert!(!stop_words(a_stop(1261, 1260, false, false, 0)).contains("signs the commit"));
}

/// A stop whose programs below could not be listed says one of them may still run,
/// and a stop that listed them says nothing about it.
#[test]
fn the_words_say_when_the_programs_below_could_not_be_listed() {
    let listed: String = stop_words(a_stop(1261, 1260, false, false, 0));
    let unlisted: String = stop_words(a_stop(1261, 1260, false, true, 0));
    assert!(!listed.contains("could not be listed"), "{listed}");
    assert!(unlisted.contains("could not be listed") && unlisted.contains("check with `ps`"), "{unlisted}");
}

/// What:
///  A stop that left a program running says so,
///  and how many it left.
/// Why:
///   Round six of the review:
///  the walk listed the programs below the step and the kernel refused a signal to one of them,
///  so the step was reported as ended with everything it started,
///  and the person was never told the signing program still holds their terminal.
#[test]
fn the_words_say_when_a_program_below_could_not_be_ended() {
    let quiet: String = stop_words(a_stop(1261, 1260, false, false, 0));
    let one: String = stop_words(a_stop(1261, 1260, false, false, 1));
    let several: String = stop_words(a_stop(1261, 1260, false, false, 3));
    assert!(!quiet.contains("could not be ended"), "{quiet}");
    assert!(one.contains("One program it started could not be ended, so it may still be running"), "{one}");
    assert!(several.contains("3 programs it started could not be ended, so they may still be running"), "{several}");
    assert!(one.contains("check with `ps`") && several.contains("check with `ps`"), "the person is told what to run: {one} {several}");
}

/// What:
///  A stop of a step the kernel held in a disk wait says the longer bound that ended it,
///  and that the step was already waited for.
/// Why:
///   The ceiling stood aside once before the step was ended,
///  and a person who is not told reads the stop as one that ended a step working on its own.
///  The longer bound is not the usual ceiling,
///  so the words may not explain it as the usual one
///  (finding 2 of the 2026-11-12 review of the stand-aside).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // expect(stopWords({ ...aStop(), boundThatEndedIt: stallBound(300, 'the-kernel-held-it') })).toContain('held it in a disk wait')
/// ```
#[test]
fn the_words_say_when_the_kernel_held_the_step_before_it_was_stopped() {
    let quiet: String = stop_words(a_stop(1261, 1260, false, false, 0));
    let held: String = stop_words(ScratchStepStop {
        bound_that_ended_it: TheBoundThatEndedACommit::TheStallBound {
            bound: Duration::from_secs(300),
            why: WhyTheStepWasWaitedFor::TheKernelHeldIt(HeldProgram { number: 4242, name: String::from("git") }),
        },
        ..a_stop(301, 1260, false, false, 0)
    });
    assert!(!quiet.contains("held it in a disk wait"), "{quiet}");
    assert!(held.contains("The kernel held it in a disk wait, so it was waited for to that bound before it was stopped."), "{held}");
    assert!(held.contains("(300 seconds)"), "the bound that ended it is the one named: {held}");
    assert!(!held.contains("three times"), "the longer bound is not explained as the usual ceiling: {held}");
}

/// What:
///  A stop of a step nothing could be learned about says so,
///  and never that the kernel held it.
/// Why:
///   The bound is the same longer one,
///  and the reason is different:
///  a look at `/proc` failed,
///  so no process was seen in uninterrupted sleep and naming one would state a fact no look found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // expect(stopWords({ ...aStop(), boundThatEndedIt: stallBound(300, 'its-state-could-not-be-read') })).not.toContain('The kernel held it')
/// ```
#[test]
fn the_words_say_when_nothing_could_be_learned_about_the_step() {
    let unknown: String = stop_words(ScratchStepStop {
        bound_that_ended_it: TheBoundThatEndedACommit::TheStallBound { bound: Duration::from_secs(300), why: WhyTheStepWasWaitedFor::ItsStateCouldNotBeRead },
        ..a_stop(301, 1260, false, false, 0)
    });
    assert!(!unknown.contains("The kernel held it"), "a state nothing read is not said as a disk wait: {unknown}");
    assert!(unknown.contains("Its state could not be read") && unknown.contains("waited for to that bound"), "{unknown}");
    assert!(unknown.contains("(300 seconds)"), "{unknown}");
}
