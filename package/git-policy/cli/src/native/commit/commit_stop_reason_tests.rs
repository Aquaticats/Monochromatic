//! What:
//!  The reason one commit did not land,
//! its line,
//!  and the route each kind of stop is read from.
//! Why:
//!  A reason that names the wrong route is worse than none:
//! it sends the reader to a policy that did not report,
//!  or to a machine failure that never happened.

/// The reasons and the two readings that make them.
use super::*;
/// The finding a policy reports.
use crate::policy_events::FindingEvent;
/// The registry entry and the severity a finding carries.
use crate::policy_registry::{PolicyId, Severity};
/// The lifecycle point that ran a policy.
use crate::policy_trigger::Trigger;
/// The engine failure the wrapper reports.
use crate::diagnostics::EngineFailureCode;
/// The Git step the runner noted.
use crate::git_step_failure::FailedStep;

/// The status a control's failing step exits with.
const REFUSED_STATUS: i32 = 128;

/// The Git step a control's commit failed in.
const STEP: &str = "git add";

/// A control's finding,
/// at the severity given.
fn finding(severity: Severity) -> PolicyEvent {
    return PolicyEvent::Finding(FindingEvent {
        trigger: Trigger::PreForward,
        policy: PolicyId::FinalNewline,
        severity,
        code: "missing-newline",
        message: String::from("no newline"),
        path: None,
        location: None,
        fix_available: false,
    });
}

/// A control's fixed-transform rejection.
fn core_finding() -> PolicyEvent {
    return PolicyEvent::CoreFinding { core_id: "commit-only", code: "staged-deletion-worktree-present", message: String::from("see the message") };
}

/// A control's engine failure.
fn engine_failure() -> PolicyEvent {
    return PolicyEvent::EngineFailure {
        code: EngineFailureCode::PolicyIncomplete,
        hint: crate::policy_events::EngineFailureHint::OfCode,
        message: String::from("the policy could not finish"),
        trigger: Some(Trigger::PreForward),
        policy: None,
        path: None,
    };
}

/// A control's note of a step that failed.
fn noted(status: Option<i32>) -> FailedStep {
    return FailedStep { name: String::from(STEP), status };
}

/// A landed commit states null,
/// and a stopped one states the route,
/// the step and the status.
#[test]
fn the_line_states_the_reason_or_null() {
    assert_eq!(stop_json(None), "null");
    let refused: String = stop_json(Some(&CommitStopReason::git_step(STEP, Some(REFUSED_STATUS))));
    assert_eq!(refused, format!("{{\"kind\":\"{KIND_GIT_STEP}\",\"step\":\"{STEP}\",\"status\":{REFUSED_STATUS}}}"));
    // A step that never exited by itself carries no status at all, so a reader never reads zero as one.
    assert_eq!(stop_json(Some(&CommitStopReason::git_step(STEP, None))), format!("{{\"kind\":\"{KIND_GIT_STEP}\",\"step\":\"{STEP}\"}}"));
}

/// Each kind names the thing that stopped the commit and nothing else.
#[test]
fn each_kind_names_what_stopped_the_commit() {
    assert_eq!(stop_json(Some(&CommitStopReason::policy("final-newline"))), format!("{{\"kind\":\"{KIND_POLICY}\",\"policy\":\"final-newline\"}}"));
    assert_eq!(
        stop_json(Some(&CommitStopReason::core_finding("commit-only", "replay-conflict"))),
        format!("{{\"kind\":\"{KIND_CORE_FINDING}\",\"core\":\"commit-only\",\"code\":\"replay-conflict\"}}")
    );
    assert_eq!(
        stop_json(Some(&CommitStopReason::engine_failure("transaction-failed"))),
        format!("{{\"kind\":\"{KIND_ENGINE_FAILURE}\",\"code\":\"transaction-failed\"}}")
    );
    assert_eq!(
        stop_json(Some(&CommitStopReason::native_exit("git commit", REFUSED_STATUS))),
        format!("{{\"kind\":\"{KIND_NATIVE_EXIT}\",\"step\":\"git commit\",\"status\":{REFUSED_STATUS}}}")
    );
    assert_eq!(stop_json(Some(&CommitStopReason::unidentified())), format!("{{\"kind\":\"{KIND_UNIDENTIFIED}\"}}"));
}

/// A name with a quote or a separator stays one JSON string,
/// so a reason can never break the line it is written in.
#[test]
fn a_hostile_name_stays_one_json_string() {
    let hostile: String = String::from("git \"add\"\\x\n{\"k\":1}");
    let line: String = stop_json(Some(&CommitStopReason::git_step(hostile.as_str(), None)));
    let parsed: serde_json::Value = serde_json::from_str(line.as_str()).expect("one JSON object");
    assert_eq!(parsed["step"], hostile.as_str());
}

/// Real Git's own status is the reason,
/// with the step the runner noted and without one when it noted nothing.
#[test]
fn a_native_exit_names_git_and_its_status() {
    let with_note: CommitStopReason = reason_of_stop(&TransactionStop::NativeExit(REFUSED_STATUS), Some(noted(Some(1))));
    assert_eq!(with_note, CommitStopReason::native_exit(STEP, 1));
    let without: CommitStopReason = reason_of_stop(&TransactionStop::NativeExit(REFUSED_STATUS), None);
    assert_eq!(without, CommitStopReason::native_exit("git commit", REFUSED_STATUS));
}

/// A step that failed is the reason a `Message` stop and a `Failed` stop were made,
/// and a stop with no failed step behind it says so.
#[test]
fn a_failed_step_is_the_reason_and_an_unknown_route_is_named_as_one() {
    let message: CommitStopReason = reason_of_stop(&TransactionStop::Message(String::from("the words the person saw")), Some(noted(Some(REFUSED_STATUS))));
    assert_eq!(message, CommitStopReason::git_step(STEP, Some(REFUSED_STATUS)));
    let failed: CommitStopReason = reason_of_stop(
        &TransactionStop::Failed { code: EngineFailureCode::TransactionFailed, message: String::from("the words the person saw") },
        Some(noted(None)),
    );
    assert_eq!(failed, CommitStopReason::git_step(STEP, None));
    // A stop whose words are the wrapper's own, with no step and no finding to blame, is the gap the owner calls a defect.
    assert_eq!(reason_of_stop(&TransactionStop::Message(String::from("words")), None), CommitStopReason::unidentified());
    // A transaction failure no step explains keeps the engine's own code.
    let engine: CommitStopReason = reason_of_stop(
        &TransactionStop::Failed { code: EngineFailureCode::TransactionFailed, message: String::from("words") },
        None,
    );
    assert_eq!(engine, CommitStopReason::engine_failure("transaction-failed"));
}

/// The event that explains a non-zero exit is read first,
/// and a run of events that explains nothing is named as unidentified.
#[test]
fn the_events_name_the_route_that_explains_the_exit() {
    assert_eq!(reason_of_events(&[engine_failure(), core_finding(), finding(Severity::Error)]), CommitStopReason::engine_failure("policy-incomplete"));
    assert_eq!(reason_of_events(&[core_finding(), finding(Severity::Error)]), CommitStopReason::core_finding("commit-only", "staged-deletion-worktree-present"));
    assert_eq!(reason_of_events(&[finding(Severity::Error)]), CommitStopReason::policy("final-newline"));
    // A pass that proposed a correction without an error finding still names the policy that reported.
    assert_eq!(reason_of_events(&[finding(Severity::Warn)]), CommitStopReason::policy("final-newline"));
    assert_eq!(reason_of_events(&[]), CommitStopReason::unidentified());
}
