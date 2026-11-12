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
/// The failures a command decides its own stop from,
/// and the platform one of them names.
use crate::service_messages::{ClientFailure, Platform, WhatTheBranchShows};
/// A duration the wait a failure states.
use std::time::Duration;

/// The status a control's failing step exits with.
const REFUSED_STATUS: i32 = 128;

/// The Git step a control's commit failed in.
const STEP: &str = "git add";

/// The process number a control's signal is sent by.
const SENDER: i32 = 4_242;

/// What:
///  The routes `reason_of_stop` looked for and found absent,
///  as a control states them.
/// Why:
///  The list is the record's own answer to "why does this say unidentified?":
///  a control that repeated the module's own constant could not notice that a route went missing from it.
const ROUTES_A_MESSAGE_IS_NOT: [&str; 7] = [KIND_POLICY, KIND_CORE_FINDING, KIND_ENGINE_FAILURE, KIND_GIT_STEP, KIND_NATIVE_EXIT, KIND_CLIENT_SIGNAL, KIND_CLIENT_REFUSAL];

/// The routes `reason_of_events` looked for and found absent.
const ROUTES_AN_EVENT_IS_NOT: [&str; 3] = [KIND_ENGINE_FAILURE, KIND_CORE_FINDING, KIND_POLICY];

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
    assert_eq!(
        stop_json(Some(&CommitStopReason::client_failure(&ClientFailure::OtherRelease))),
        format!("{{\"kind\":\"{KIND_CLIENT_REFUSAL}\",\"reason\":\"other-release\"}}")
    );
    assert_eq!(
        stop_json(Some(&CommitStopReason::client_signal(libc::SIGTERM, Some(SENDER)))),
        format!("{{\"kind\":\"{KIND_CLIENT_SIGNAL}\",\"signal\":{},\"sender\":{SENDER}}}", libc::SIGTERM)
    );
    // The terminal's own interrupt carries no sender, and the record says the signal alone rather than a number it guessed.
    assert_eq!(
        stop_json(Some(&CommitStopReason::client_signal(libc::SIGINT, None))),
        format!("{{\"kind\":\"{KIND_CLIENT_SIGNAL}\",\"signal\":{}}}", libc::SIGINT)
    );
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
    // A stop whose words are the wrapper's own, with no step and no finding to blame, is the gap the owner calls a defect,
    //  and it says which routes were looked for and were not it.
    assert_eq!(reason_of_stop(&TransactionStop::Message(String::from("words")), None), CommitStopReason::unidentified(&ROUTES_A_MESSAGE_IS_NOT));
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
    assert_eq!(reason_of_events(&[]), CommitStopReason::unidentified(&ROUTES_AN_EVENT_IS_NOT));
}

/// What:
///  Every failure a command decides its own stop from is named in the record by a word of its own,
///  and no two failures share one.
/// Why:
///  Seven stopped records of one release said only `unidentified` while the person had been told the cause,
///  and three of them were the service manager exchanging nothing with the wrapper:
///  a reader could not tell a release that would not retire from a bus that answered nothing.
///  The words are the record's own,
///  so each one is checked here against the failure it belongs to,
///  and a failure added later cannot silently take another's word.
#[test]
fn every_failure_a_command_decides_names_itself() {
    let failures: Vec<ClientFailure> = vec![
        ClientFailure::DidNotStart,
        ClientFailure::NothingAnswered { address: String::from("an address"), waited: Duration::from_secs(10) },
        ClientFailure::NoActivation(Platform::LinuxWithoutUserServices),
        ClientFailure::OtherAccount,
        ClientFailure::MayNotHaveLanded,
        ClientFailure::NoCommonVersion,
        ClientFailure::OtherRelease,
        ClientFailure::RepositoryFull(String::from("a repository")),
        ClientFailure::RepositoryReadOnly(String::from("a repository")),
        ClientFailure::PermissionDenied(String::from("a repository")),
        ClientFailure::RepositoryFrozen(String::from("a repository")),
        ClientFailure::BranchKeepsChanging { branch: String::from("main"), times: 2 },
        ClientFailure::BranchMoved { branch: String::from("main") },
        ClientFailure::ConclusionStateChanged { branch: String::from("main") },
        ClientFailure::BranchMovedMessageSaved { branch: String::from("main"), file: String::from("a file"), amend: false },
        ClientFailure::BranchSwitched { branch: String::from("main"), now: String::from("other"), file: None },
        ClientFailure::ServiceSilent,
        ClientFailure::ServiceSilentAfterPreparing { shows: WhatTheBranchShows::Unread },
        ClientFailure::StillWithService { minutes: 20, blocker: None },
        ClientFailure::WithdrawalUnconfirmed { blocker: None },
        ClientFailure::Interrupted { signal: libc::SIGINT, sender: None, blocker: None },
        ClientFailure::InterruptedNotKnown { signal: libc::SIGTERM, sender: None, blocker: None },
        ClientFailure::StepTooSlow,
        ClientFailure::Refused,
        ClientFailure::IndexInUse(String::from("another program")),
        ClientFailure::ReferenceLocked { branch: String::from("main"), words: String::new() },
        ClientFailure::Unavailable,
        ClientFailure::ProgramMissing(String::from("a program")),
        ClientFailure::CouldNotStart(String::from("a reason")),
        ClientFailure::SocketPathTooLong { path: String::from("a path"), length: 108, longest: 107, longest_runtime: 60 },
        ClientFailure::ServiceElsewhere { made: String::from("here"), looked_for: String::from("there"), manager_runtime: String::from("a"), client_runtime: String::from("b") },
        ClientFailure::UnitInTheWay(String::from("a unit")),
        ClientFailure::WorkNotHandedOver(String::from("a reason")),
        ClientFailure::ContentPruned,
    ];
    let mut seen: Vec<&'static str> = Vec::new();
    for failure in &failures {
        let reason: CommitStopReason = reason_of_stop(&crate::commit_transaction::client_stop(failure.clone()), None);
        assert_eq!(reason.kind, KIND_CLIENT_REFUSAL, "{failure:?}");
        let word: &'static str = crate::service_messages::failure_code(failure);
        assert!(!word.is_empty() && word.chars().all(|character| { return character.is_ascii_lowercase() || character == '-'; }), "{word}");
        assert!(!seen.contains(&word), "two failures share the word {word}");
        seen.push(word);
    }
    assert_eq!(seen.len(), failures.len(), "every failure is named, and by a word of its own");
    // The three the live log could not tell apart are named apart.
    let named: String = failures.iter().map(|failure| { return crate::service_messages::failure_code(failure); }).collect::<Vec<&str>>().join(" ");
    for expected in ["other-release", "no-activation", "did-not-start", "nothing-answered"] {
        assert!(named.contains(expected), "{expected} is not among the words: {named}");
    }
}

/// A stop the command decided is the reason even where a Git step failed beside it,
/// because the failure is what ended the commit.
#[test]
fn a_failure_the_command_decided_is_the_reason_whatever_else_failed() {
    let decided: CommitStopReason = reason_of_stop(&crate::commit_transaction::client_stop(ClientFailure::OtherRelease), Some(noted(Some(REFUSED_STATUS))));
    assert_eq!(decided, CommitStopReason::client_failure(&ClientFailure::OtherRelease));
}

/// The person's signal is named with the process that sent it,
/// so a record tells a keyboard's interrupt from a program's own bound.
#[test]
fn a_signal_names_its_sender_where_the_kernel_names_one() {
    let sent: CommitStopReason = reason_of_stop(&TransactionStop::EndedBySignal { signal: libc::SIGTERM, sender: Some(SENDER), message: String::from("words") }, None);
    assert_eq!(sent, CommitStopReason::client_signal(libc::SIGTERM, Some(SENDER)));
    let interrupted: CommitStopReason = reason_of_stop(&TransactionStop::EndedBySignal { signal: libc::SIGINT, sender: None, message: String::from("words") }, None);
    assert_eq!(interrupted, CommitStopReason::client_signal(libc::SIGINT, None));
}

/// A record with no route says which routes were ruled out,
/// so `unidentified` is a statement about the search and never a shrug.
#[test]
fn a_stop_with_no_route_says_what_was_ruled_out() {
    let line: String = stop_json(Some(&CommitStopReason::unidentified(&ROUTES_A_MESSAGE_IS_NOT)));
    let parsed: serde_json::Value = serde_json::from_str(line.as_str()).expect("one JSON object");
    let routes: Vec<&str> = parsed["ruledOut"].as_array().expect("a list of routes").iter().map(|route| { return route.as_str().expect("a route name"); }).collect();
    for kind in [KIND_POLICY, KIND_CORE_FINDING, KIND_ENGINE_FAILURE, KIND_GIT_STEP, KIND_NATIVE_EXIT, KIND_CLIENT_SIGNAL, KIND_CLIENT_REFUSAL] {
        assert!(routes.contains(&kind), "{kind} is not among the routes ruled out: {line}");
    }
    assert!(!routes.contains(&KIND_UNIDENTIFIED), "a record cannot rule itself out: {line}");
}
