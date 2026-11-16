//! What:
//!  The controls of one add's own allowance:
//!  a step clamped into what is left of it,
//!  a spent allowance refusing a step and
//!  naming the one it waited for,
//!  the clock that stops for the person's own command,
//!  and the words of the add's ending.
//! Why:
//!   The allowance is what makes an add end when its own steps keep being ended and started
//!  again,
//!  so the clamping is checked without a Git at all:
//!  a bound that is not clamped is the whole defect.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! using clock = arm(Duration.fromMillis(200));
//! expect(whatIsLeft()).toBeLessThanOrEqual(200); expect(spentAfterAwait()).toBeDefined();
//! ```

use super::{
    ALLOWANCE_VARIABLE, Spent, TheAddsOwnClock, a_bound_within_the_allowance, a_clock_runs_here, arm, before_a_step, the_allowance_is_spent, the_persons_command_began,
    what_is_left, within_the_allowance,
};
use crate::child_class::ChildClass;
use crate::child_watch::{StepBound, WatchTerms};
use std::path::Path;
use std::time::Duration;

/// The quiet window a step of a bulk Git step carries before the allowance clamps it,
/// in milliseconds.
/// The shipped window is minutes; a control's is a moment, so the clamp is what the control sees.
const CONTROL_WINDOW: Duration = Duration::from_millis(400);

/// The ceiling a step of a bulk Git step carries before the allowance clamps it,
/// in milliseconds.
const CONTROL_CEILING: Duration = Duration::from_millis(800);

/// The allowance a control arms,
/// in milliseconds,
/// long enough to see what is left and short enough to spend.
const CONTROL_ALLOWANCE: Duration = Duration::from_millis(300);

/// The terms and the bound a control hands to the clamp:
/// a bulk step's window,
/// its notice grace,
/// and its ceiling.
fn a_bulk_step() -> (WatchTerms, StepBound) {
    let mut terms: WatchTerms = WatchTerms::for_class(ChildClass::Object);
    terms.limit = CONTROL_WINDOW;
    // A sink for notices, so a control prints none.
    terms.notices = Box::new(std::io::sink());
    return (terms, StepBound::EndAt(CONTROL_CEILING));
}

/// The pause a control takes while it checks what a stopped clock does.
fn pause_for_a_moment(milliseconds: u64) {
    std::thread::sleep(Duration::from_millis(milliseconds));
}

/// What:
///  A step's window,
///  its notice grace and its bound all come out of the allowance when the allowance is the shorter.
/// Why:
///   A bound left at its class's number is the defect:
///  the add would wait its ten minutes for one step and then start
///  another.
#[test]
fn a_step_is_clamped_into_what_is_left_of_the_allowance() {
    let _clock: TheAddsOwnClock = arm(CONTROL_ALLOWANCE);
    assert!(a_clock_runs_here(), "the clock of the add is armed");
    let left: Duration = what_is_left().expect("an add runs here");
    assert!(left <= CONTROL_ALLOWANCE, "the allowance is what is left, and not the class's ten minutes: {left:?}");
    let (terms, bound): (WatchTerms, StepBound) = within_the_allowance(a_bulk_step().0, a_bulk_step().1, "git read-tree -m -u, a bulk step");
    assert!(terms.limit <= left, "the quiet window is the allowance's when that is shorter: {:?}", terms.limit);
    assert!(terms.cadence.grace <= terms.limit / 2, "the notice grace stays within half the window it announces: {:?}", terms.cadence.grace);
    match bound {
        StepBound::EndAt(ceiling) => assert!(ceiling <= left, "the ceiling is the allowance's when that is shorter: {ceiling:?}"),
        other => panic!("a step that may be run twice is still ended at a ceiling, not {other:?}"),
    }
}

/// What:
///  A step that is never ended is still never ended,
///  and the command stops waiting for it when the allowance is gone.
/// Why:
///   The owner's "Never cut a branch write (Recommended)" holds whatever the allowance says,
///  and "nothing should ever
///  wait forever or indefinitely" holds too:
///  the command stops waiting and leaves the write running.
#[test]
fn a_step_that_is_never_ended_is_never_ended_by_the_allowance() {
    let _clock: TheAddsOwnClock = arm(CONTROL_ALLOWANCE);
    let left: Duration = what_is_left().expect("an add runs here");
    let (_, bound): (WatchTerms, StepBound) = within_the_allowance(a_bulk_step().0, StepBound::Unbounded, "git update-ref refs/heads/main, a branch write");
    match bound {
        StepBound::LeaveRunning { after, extra } => {
            assert!(after <= left, "the command stops waiting at the allowance: {after:?}");
            assert_eq!(extra, Duration::ZERO, "nothing is waited for on top of an allowance that is spent");
        }
        other => panic!("a write that is never ended is left running and not ended, not {other:?}"),
    }
}

/// What:
///  Once the allowance is spent,
///  no step starts,
///  and the refusal names the step it refused,
///  while the add's ending names the last step that did start.
/// Why:
///   A step started after the allowance is gone would spend a process and print a notice that says it was
///  ended,
///  when what happened is that the add had no time left;
///  and a step refused after the time went is not the
///  one that spent it.
#[test]
fn a_spent_allowance_refuses_a_step_and_names_what_it_waited_for() {
    let _clock: TheAddsOwnClock = arm(CONTROL_ALLOWANCE);
    assert!(before_a_step("git ls-files --others, a bulk step").is_ok(), "a step with time left starts");
    pause_for_a_moment(CONTROL_ALLOWANCE.as_millis() as u64 + 50);
    let refused: std::io::Error = before_a_step("git status --porcelain, a bulk step").expect_err("a step with no time left is refused");
    assert_eq!(refused.kind(), std::io::ErrorKind::TimedOut, "a refused step is a step that did not answer");
    assert!(refused.to_string().contains(ALLOWANCE_VARIABLE), "the refusal names the variable that sets the allowance: {refused}");
    assert!(refused.to_string().contains("git status --porcelain, a bulk step"), "the refusal names the step it refused: {refused}");
    let spent: Spent = the_allowance_is_spent().expect("the allowance is spent");
    assert_eq!(spent.ceiling(), CONTROL_ALLOWANCE, "the ending names the allowance it was armed with");
    assert_eq!(spent.waiting_for(), Some("git ls-files --others, a bulk step"), "the ending names the last step that started");
    let line: String = spent.the_add_stopped_line(Some(Path::new("/fixture/topic")));
    assert!(line.contains(ALLOWANCE_VARIABLE), "the line names the variable: {line}");
    assert!(line.contains("was not made"), "the line says the worktree was not made: {line}");
    assert!(line.contains("git ls-files --others, a bulk step"), "the line names the step it waited for: {line}");
    assert!(line.contains("/fixture/topic"), "the line names the destination: {line}");
}

/// What:
///  The clock stops while the person's own command runs,
///  so its time is not the wrapper's own,
///  and runs again when the command returns.
/// Why:
///   A checkout the person's Git ran for twenty minutes would otherwise fail the add's copy,
///  which is the
///  wrapper failing a healthy add for Git's own time.
#[test]
fn the_clock_stops_while_the_persons_own_command_runs() {
    let _clock: TheAddsOwnClock = arm(CONTROL_ALLOWANCE);
    {
        let _person = the_persons_command_began();
        assert!(what_is_left().is_none_or(|left: Duration| { return left <= CONTROL_ALLOWANCE; }), "the clock still reports the add it belongs to");
        pause_for_a_moment(CONTROL_ALLOWANCE.as_millis() as u64 + 100);
        assert!(the_allowance_is_spent().is_none(), "the person's own command spends none of the allowance");
    }
    pause_for_a_moment(CONTROL_ALLOWANCE.as_millis() as u64 + 100);
    assert!(the_allowance_is_spent().is_some(), "the clock runs again once the person's command returned");
}

/// What:
///  A process that runs no add has no allowance at all:
///  nothing is clamped and no step is refused.
/// Why:
///   Every Git step of every command asks,
///  and a commit's steps must keep the bounds their classes give them.
#[test]
fn a_process_that_runs_no_add_clamps_nothing() {
    assert!(!a_clock_runs_here(), "no add runs in a process that armed no clock");
    assert!(what_is_left().is_none(), "a process with no add has no allowance to report");
    assert!(the_allowance_is_spent().is_none(), "a process with no add has no allowance to spend");
    assert!(before_a_step("git rev-parse HEAD").is_ok(), "a step of a command that is not an add starts");
    let (terms, bound): (WatchTerms, StepBound) = within_the_allowance(a_bulk_step().0, a_bulk_step().1, "git status, a bulk step");
    assert_eq!(terms.limit, CONTROL_WINDOW, "no add means no clamping of the quiet window");
    match bound {
        StepBound::EndAt(ceiling) => assert_eq!(ceiling, CONTROL_CEILING, "no add means no clamping of a step's ceiling"),
        other => panic!("a step that may be run twice is ended at a ceiling, not {other:?}"),
    }
    assert_eq!(a_bound_within_the_allowance(Some(CONTROL_CEILING), "git read-tree, a bulk step"), Some(CONTROL_CEILING), "no add means no clamping of a ladder step's ceiling");
}

/// What:
///  A ladder step's ceiling is clamped into the allowance,
///  and a step with no ceiling keeps none.
/// Why:
///  The ladder's own Git starts through the forwarded runner and carries a plain span of time,
///  so the clamp is a
///  different face than the watched step's terms and bound.
#[test]
fn a_ladder_steps_ceiling_is_clamped_and_an_absent_one_stays_absent() {
    let _clock: TheAddsOwnClock = arm(CONTROL_ALLOWANCE);
    let left: Duration = what_is_left().expect("an add runs here");
    let clamped: Option<Duration> = a_bound_within_the_allowance(Some(CONTROL_CEILING), "git reset --hard --no-recurse-submodules");
    assert!(clamped.is_some_and(|ceiling: Duration| { return ceiling <= left; }), "the ladder's ceiling is the allowance's when that is shorter: {clamped:?}");
    assert_eq!(a_bound_within_the_allowance(None, "git rev-parse HEAD"), None, "a step with no ceiling keeps none");
}

/// What:
///  The ending says the allowance in whole seconds and the step it waited for,
///  and nothing about a destination when the add named none.
/// Why:
///   The line is what a person reads when their add made nothing,
///  so it must make sense for an add whose destination the wrapper could not read.
#[test]
fn the_ending_names_the_numbers_and_the_step_with_or_without_a_destination() {
    let spent: Spent = Spent::for_a_control(Duration::from_secs(12 * 60), Duration::from_secs(721), Some(String::from("git ls-files --others, a bulk step")));
    let with_a_destination: String = spent.the_add_stopped_line(Some(Path::new("/fixture/topic")));
    assert!(with_a_destination.contains("721 seconds of 720 seconds"), "the line names what was spent and the allowance: {with_a_destination}");
    assert!(with_a_destination.contains("/fixture/topic"), "the line names the destination: {with_a_destination}");
    let without: String = spent.the_add_stopped_line(None);
    assert!(without.contains("No worktree was made"), "the line says what it left when it cannot name the destination: {without}");
    assert!(without.contains("git ls-files --others, a bulk step"), "the line names the step: {without}");
}
