//! What:
//!  Controls of the report's folding:
//!  a pass,
//!  a failed arm that stops every later arm,
//!  an arm that could not run blocking the route,
//!  a child's ending,
//!  and the step in flight and the last one that finished.
//! Why:
//!  The verdict is the one word the wrapper gates on,
//!  and a fold that turned a failed arm into a pass,
//!  or a timeout into a refusal of nothing,
//!  would enable a route the probe could not exercise.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const report = new ProbeReport(); report.record(passed('unshare')); expect(report.verdict()).toEqual({ kind: 'blocked' });
//! ```

use super::super::probe_arms::{ProbeArm, ProbeStep};
use super::{ArmAnswer, ChildEnding, ProbeReport, ProbeVerdict};
use super::super::probe_status_line::StepLine;

/// Every worker step passed,
/// the holder's two lines arrived,
/// and the verdict
///  is a pass.
fn a_passing_report() -> ProbeReport {
    let mut report: ProbeReport = ProbeReport::new();
    for step in ProbeStep::WORKER_ORDER {
        if step == ProbeStep::CheckoutRoot {
            report.record(StepLine::not_exercised(step, "no-checkout-path", None));
        } else {
            report.record(StepLine::passed(step));
        }
    }
    for step in ProbeStep::HOLDER_ORDER {
        report.record(StepLine::passed(step));
    }
    return report;
}

/// A report whose checkout step did not run is blocked,
/// and the arm it names is the walk's.
#[test]
fn an_arm_that_did_not_run_blocks() {
    let report: ProbeReport = a_passing_report();
    assert_eq!(report.verdict(), ProbeVerdict::Blocked { arm: ProbeArm::Walk });
    assert!(!report.passed());
}

/// A failed step stops the probe:
/// the arms after it did not run,
/// and the verdict names that step.
#[test]
fn a_failed_step_stops_every_later_arm() {
    let mut report: ProbeReport = ProbeReport::new();
    for step in ProbeStep::WORKER_ORDER {
        if step == ProbeStep::AttachPlace {
            report.record(StepLine::failed(step, Some("EPERM")));
            break;
        }
        report.record(StepLine::passed(step));
    }
    assert_eq!(report.verdict(), ProbeVerdict::Failed { step: ProbeStep::AttachPlace });
    let attach: super::ArmOutcome = report.arm_outcome(ProbeArm::Attach);
    assert_eq!(attach.answer, ArmAnswer::Failed);
    assert_eq!(attach.errno, super::super::probe_errno_names::ErrnoWord::Named("EPERM"));
    assert_eq!(report.arm_outcome(ProbeArm::Verify).answer, ArmAnswer::NotExercised);
    assert_eq!(report.arm_outcome(ProbeArm::Teardown).answer, ArmAnswer::NotExercised);
}

/// A child that ended without a line makes the step in flight the failure.
#[test]
fn an_ending_without_a_line_fails_the_step_in_flight() {
    let mut report: ProbeReport = ProbeReport::new();
    report.record(StepLine::passed(ProbeStep::Unshare));
    report.record(StepLine::passed(ProbeStep::PrivatePropagation));
    report.finish_with(ChildEnding::NoStatusLine);
    assert_eq!(report.in_flight(), Some(ProbeStep::ScratchTmpfs));
    assert_eq!(report.last_finished(), Some(ProbeStep::PrivatePropagation));
    assert_eq!(report.verdict(), ProbeVerdict::Failed { step: ProbeStep::ScratchTmpfs });
}

/// A child that died of a signal is the same failure,
/// and a line the mode could not read is one too.
#[test]
fn a_signal_death_and_an_unreadable_line_fail_the_step_in_flight() {
    let mut signalled: ProbeReport = ProbeReport::new();
    signalled.record(StepLine::passed(ProbeStep::Unshare));
    signalled.finish_with(ChildEnding::Signal { number: libc::SIGKILL });
    assert_eq!(signalled.verdict(), ProbeVerdict::Failed { step: ProbeStep::PrivatePropagation });
    let mut unreadable: ProbeReport = ProbeReport::new();
    unreadable.record(StepLine::passed(ProbeStep::Unshare));
    unreadable.finish_with(ChildEnding::UnparsedLine { text: String::from("hello") });
    assert_eq!(unreadable.verdict(), ProbeVerdict::Failed { step: ProbeStep::PrivatePropagation });
}

/// A report with no line at all names the worker's first step as in flight.
#[test]
fn an_empty_report_has_the_first_step_in_flight() {
    let report: ProbeReport = ProbeReport::new();
    assert_eq!(report.in_flight(), Some(ProbeStep::Unshare));
    assert_eq!(report.last_finished(), None);
    assert_eq!(report.verdict(), ProbeVerdict::Blocked { arm: ProbeArm::Namespace });
}

/// A step that did not run names its own arm,
/// never a later one.
#[test]
fn a_step_that_did_not_run_names_its_arm() {
    let mut report: ProbeReport = ProbeReport::new();
    for step in [ProbeStep::Unshare, ProbeStep::PrivatePropagation, ProbeStep::ScratchTmpfs, ProbeStep::ScratchTree, ProbeStep::WalkSlot, ProbeStep::WalkRefusalControls, ProbeStep::CheckoutRoot] {
        report.record(StepLine::passed(step));
    }
    report.record(StepLine::not_exercised(ProbeStep::ReadSourceName, "no-source-name", None));
    assert_eq!(report.verdict(), ProbeVerdict::Blocked { arm: ProbeArm::Source });
}
