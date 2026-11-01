//! What:
//!  Stage order,
//!  trigger and filter skipping,
//!  severities,
//!  stopping,
//!  and the three ways
//!       a pass can fail to decide.
//! Why:
//!  Each rule here is a way a policy could silently not run or a blocked command could
//!      be let through:
//!  a wrong trigger set,
//!  an ignored severity,
//!  a swallowed failure.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect((await runPolicyStage({ ... })).events.map(e => e.policyId)).toEqual(['require-root']);
//! ```

/// The stage under test and the types it reads and produces.
use super::{
    BLOCKED_EXIT_CODE, PolicyChecks, PolicyFinding, PolicyOutcome, StageEnd, StageRequest,
    StageResult, Unavailable, any_policy_applies, pass_exit_code, run_policy_stage,
};
use crate::config_schema::PolicyConfig;
use crate::diagnostics::EngineFailureCode;
/// The event form of a pathname.
use crate::event_path::EventPath;
use crate::policy_events::{FindingEvent, FindingLocation, PolicyEvent};
use crate::policy_registry::{POLICY_REGISTRY, PolicyId, Severity, policy_descriptor};
use crate::policy_trigger::Trigger;
use crate::wrapper_controls::{Controls, no_controls};

/// A provider that answers from a script and records every check it was asked for.
struct Scripted {
    /// Outcomes by policy;
    ///  a policy without an entry finds nothing.
    outcomes: Vec<(PolicyId, PolicyOutcome)>,
    /// Every check in call order.
    calls: Vec<(PolicyId, Trigger)>,
}

impl PolicyChecks for Scripted {
    fn check(&mut self, policy: PolicyId, trigger: Trigger) -> PolicyOutcome {
        self.calls.push((policy, trigger));
        for (scripted, outcome) in &self.outcomes {
            if *scripted == policy {
                return outcome.clone();
            }
        }
        return PolicyOutcome::Findings(Vec::<PolicyFinding>::new());
    }
}

/// A provider with the given script and no calls yet.
fn scripted(outcomes: Vec<(PolicyId, PolicyOutcome)>) -> Scripted {
    return Scripted {
        outcomes,
        calls: Vec::<(PolicyId, Trigger)>::new(),
    };
}

/// One finding with only its required fields.
fn finding(code: &'static str) -> PolicyFinding {
    return PolicyFinding {
        code,
        message: format!("message of {code}"),
        path: None,
        location: None,
        fix_available: false,
    };
}

/// The outcome "found exactly this one thing".
fn found(code: &'static str) -> PolicyOutcome {
    return PolicyOutcome::Findings(vec![finding(code)]);
}

/// Every shipped policy in registry order.
fn every_policy() -> Vec<PolicyId> {
    let mut ids: Vec<PolicyId> = Vec::<PolicyId>::new();
    for descriptor in POLICY_REGISTRY {
        ids.push(descriptor.id);
    }
    return ids;
}

/// The settings of a repository whose configuration names all four optional policies at
/// the severity the incumbent declares as their default.
fn all_listed() -> PolicyConfig {
    let mut config: PolicyConfig = PolicyConfig::defaults();
    for setting in &mut config.settings {
        setting.severity = policy_descriptor(setting.id).default_severity;
    }
    return config;
}

/// A copy of `config` with one policy's severity replaced.
fn with_severity(config: &PolicyConfig, policy: PolicyId, severity: Severity) -> PolicyConfig {
    let mut changed: PolicyConfig = config.clone();
    for setting in &mut changed.settings {
        if setting.id == policy {
            setting.severity = severity;
        }
    }
    return changed;
}

/// Run every shipped policy for one trigger and return the result with the recorded calls.
fn run(
    trigger: Trigger,
    config: &PolicyConfig,
    controls: &Controls,
    selected: &[PolicyId],
    outcomes: Vec<(PolicyId, PolicyOutcome)>,
) -> (StageResult, Vec<PolicyId>) {
    let mut checks: Scripted = scripted(outcomes);
    let request: StageRequest = StageRequest {
        trigger,
        config: config.clone(),
        controls: controls.clone(),
        selected: selected.to_vec(),
    };
    let result: StageResult = run_policy_stage(&request, every_policy().as_slice(), &mut checks);
    let mut called: Vec<PolicyId> = Vec::<PolicyId>::new();
    for (policy, asked) in &checks.calls {
        assert_eq!(*asked, trigger, "a check is asked for the stage's trigger");
        called.push(*policy);
    }
    return (result, called);
}

/// The finding event the stage must build from `finding(code)`.
fn event(
    trigger: Trigger,
    policy: PolicyId,
    severity: Severity,
    code: &'static str,
) -> PolicyEvent {
    return PolicyEvent::Finding(FindingEvent {
        trigger,
        policy,
        severity,
        code,
        message: format!("message of {code}"),
        path: None,
        location: None,
        fix_available: false,
    });
}

/// A policy runs only for the triggers it declares,
///  in the order it was given.
#[test]
fn policies_run_in_order_for_their_triggers() {
    let configured: PolicyConfig = all_listed();
    let none: Controls = no_controls();
    let (pre_forward, called) = run(Trigger::PreForward, &configured, &none, &[], vec![]);
    assert_eq!(called, every_policy());
    assert_eq!(pre_forward.end, StageEnd::Completed);
    assert_eq!(pre_forward.events, Vec::<PolicyEvent>::new());
    let (_, direct_check) = run(Trigger::DirectCheck, &configured, &none, &[], vec![]);
    assert_eq!(
        direct_check,
        [
            PolicyId::RequireRoot,
            PolicyId::FinalNewline,
            PolicyId::MarkdownAutofix,
            PolicyId::ForbiddenRootContext,
            PolicyId::DependentVersionBump,
            PolicyId::ForbiddenStrings,
        ]
    );
    let (_, direct_fix) = run(Trigger::DirectFix, &configured, &none, &[], vec![]);
    assert_eq!(
        direct_fix,
        [
            PolicyId::FinalNewline,
            PolicyId::MarkdownAutofix,
            PolicyId::DependentVersionBump,
        ]
    );
    // The order is the caller's list, not the registry's.
    let mut checks: Scripted = scripted(vec![]);
    let request: StageRequest = StageRequest {
        trigger: Trigger::PreForward,
        config: configured,
        controls: none,
        selected: Vec::<PolicyId>::new(),
    };
    run_policy_stage(
        &request,
        &[PolicyId::AddExplicit, PolicyId::RequireRoot],
        &mut checks,
    );
    assert_eq!(
        checks.calls,
        [
            (PolicyId::AddExplicit, Trigger::PreForward),
            (PolicyId::RequireRoot, Trigger::PreForward)
        ]
    );
}

/// A policy that is off,
///  escaped or outside the direct-command filter is never asked.
#[test]
fn off_escaped_and_unselected_policies_are_skipped() {
    let none: Controls = no_controls();
    // Until the configuration names them, the four optional policies are off.
    let (_, unlisted) = run(
        Trigger::PreForward,
        &PolicyConfig::defaults(),
        &none,
        &[],
        vec![],
    );
    assert_eq!(
        unlisted,
        [
            PolicyId::RequireRoot,
            PolicyId::LinkedWorktreeOnly,
            PolicyId::BranchWorktreeOnly,
            PolicyId::AddExplicit,
            PolicyId::FinalNewline,
        ]
    );
    let off: PolicyConfig = with_severity(
        &PolicyConfig::defaults(),
        PolicyId::RequireRoot,
        Severity::Off,
    );
    let (_, without_root) = run(Trigger::PreForward, &off, &none, &[], vec![]);
    assert_eq!(without_root.first(), Some(&PolicyId::LinkedWorktreeOnly));
    assert_eq!(without_root.len(), 4);
    let mut escaped: Controls = no_controls();
    escaped.escaped = vec![PolicyId::AddExplicit, PolicyId::RequireRoot];
    let (_, after_escape) = run(
        Trigger::PreForward,
        &PolicyConfig::defaults(),
        &escaped,
        &[],
        vec![],
    );
    assert_eq!(
        after_escape,
        [
            PolicyId::LinkedWorktreeOnly,
            PolicyId::BranchWorktreeOnly,
            PolicyId::FinalNewline,
        ]
    );
    let (_, selected) = run(
        Trigger::DirectCheck,
        &all_listed(),
        &none,
        &[PolicyId::ForbiddenStrings, PolicyId::RequireRoot],
        vec![],
    );
    assert_eq!(
        selected,
        [PolicyId::RequireRoot, PolicyId::ForbiddenStrings]
    );
    // A selected policy that does not declare the trigger is still skipped.
    let (_, wrong_trigger) = run(
        Trigger::DirectFix,
        &all_listed(),
        &none,
        &[PolicyId::RequireRoot],
        vec![],
    );
    assert_eq!(wrong_trigger, Vec::<PolicyId>::new());
}

/// Without keep-going the first error finding ends the pass;
///  with it,
///  later policies still run.
#[test]
fn first_error_stops_unless_keep_going() {
    let config: PolicyConfig = PolicyConfig::defaults();
    let script: Vec<(PolicyId, PolicyOutcome)> = vec![
        (PolicyId::RequireRoot, found("not-at-root")),
        (PolicyId::BranchWorktreeOnly, found("creates")),
    ];
    let (stopped, called) = run(
        Trigger::PreForward,
        &config,
        &no_controls(),
        &[],
        script.clone(),
    );
    assert_eq!(stopped.end, StageEnd::Stopped);
    assert_eq!(called, [PolicyId::RequireRoot]);
    assert_eq!(
        stopped.events,
        [event(
            Trigger::PreForward,
            PolicyId::RequireRoot,
            Severity::Error,
            "not-at-root"
        )]
    );
    assert_eq!(
        pass_exit_code(stopped.events.as_slice(), stopped.end),
        BLOCKED_EXIT_CODE
    );
    let mut keep_going: Controls = no_controls();
    keep_going.keep_going = true;
    let (continued, all_called) = run(Trigger::PreForward, &config, &keep_going, &[], script);
    assert_eq!(continued.end, StageEnd::Completed);
    assert_eq!(all_called.len(), 5);
    assert_eq!(
        continued.events,
        [
            event(
                Trigger::PreForward,
                PolicyId::RequireRoot,
                Severity::Error,
                "not-at-root"
            ),
            event(
                Trigger::PreForward,
                PolicyId::BranchWorktreeOnly,
                Severity::Error,
                "creates"
            ),
        ]
    );
    assert_eq!(
        pass_exit_code(continued.events.as_slice(), continued.end),
        BLOCKED_EXIT_CODE
    );
}

/// A warning finding never stops the pass and never blocks;
///  the event carries the configured severity.
#[test]
fn warning_findings_do_not_stop_or_block() {
    let config: PolicyConfig = with_severity(
        &PolicyConfig::defaults(),
        PolicyId::BranchWorktreeOnly,
        Severity::Warn,
    );
    let (result, called) = run(
        Trigger::PreForward,
        &config,
        &no_controls(),
        &[],
        vec![
            (PolicyId::BranchWorktreeOnly, found("creates")),
            (PolicyId::FinalNewline, found("missing")),
        ],
    );
    assert_eq!(result.end, StageEnd::Completed);
    assert_eq!(called.len(), 5);
    // Both policies are warn-safe, so no configuration warning follows their findings.
    assert_eq!(
        result.events,
        [
            event(
                Trigger::PreForward,
                PolicyId::BranchWorktreeOnly,
                Severity::Warn,
                "creates"
            ),
            event(
                Trigger::PreForward,
                PolicyId::FinalNewline,
                Severity::Warn,
                "missing"
            ),
        ]
    );
    assert_eq!(pass_exit_code(result.events.as_slice(), result.end), 0);
}

/// `warn` on a policy it does not protect is reported after that policy's findings,
///  also when it found nothing.
#[test]
fn unsafe_warn_is_reported_even_when_clean() {
    let config: PolicyConfig = with_severity(
        &PolicyConfig::defaults(),
        PolicyId::AddExplicit,
        Severity::Warn,
    );
    let warning: PolicyEvent = PolicyEvent::WarnUnsafe {
        trigger: Trigger::PreForward,
        policy: PolicyId::AddExplicit,
    };
    let (clean, _) = run(Trigger::PreForward, &config, &no_controls(), &[], vec![]);
    assert_eq!(clean.events.as_slice(), std::slice::from_ref(&warning));
    assert_eq!(clean.end, StageEnd::Completed);
    let (dirty, _) = run(
        Trigger::PreForward,
        &config,
        &no_controls(),
        &[],
        vec![(PolicyId::AddExplicit, found("bulk"))],
    );
    assert_eq!(
        dirty.events,
        [
            event(
                Trigger::PreForward,
                PolicyId::AddExplicit,
                Severity::Warn,
                "bulk"
            ),
            warning,
        ]
    );
    assert_eq!(pass_exit_code(dirty.events.as_slice(), dirty.end), 0);
    // At `error` the same policy earns no warning.
    let (error, _) = run(
        Trigger::PreForward,
        &PolicyConfig::defaults(),
        &no_controls(),
        &[],
        vec![],
    );
    assert_eq!(error.events, Vec::<PolicyEvent>::new());
}

/// A finding's optional fields reach its event unchanged.
#[test]
fn finding_details_reach_the_event() {
    let detailed: PolicyFinding = PolicyFinding {
        code: "missing",
        message: String::from("no final newline"),
        path: Some(EventPath::from_git_bytes("a.txt".as_bytes())),
        location: Some(FindingLocation {
            byte_start: 4,
            byte_end: 5,
        }),
        fix_available: true,
    };
    let (result, _) = run(
        Trigger::DirectCheck,
        &PolicyConfig::defaults(),
        &no_controls(),
        &[],
        vec![(
            PolicyId::FinalNewline,
            PolicyOutcome::Findings(vec![detailed, finding("second")]),
        )],
    );
    assert_eq!(
        result.events,
        [
            PolicyEvent::Finding(FindingEvent {
                trigger: Trigger::DirectCheck,
                policy: PolicyId::FinalNewline,
                severity: Severity::Warn,
                code: "missing",
                message: String::from("no final newline"),
                path: Some(EventPath::from_git_bytes("a.txt".as_bytes())),
                location: Some(FindingLocation {
                    byte_start: 4,
                    byte_end: 5,
                }),
                fix_available: true,
            }),
            event(
                Trigger::DirectCheck,
                PolicyId::FinalNewline,
                Severity::Warn,
                "second"
            ),
        ]
    );
}

/// A policy that could not finish ends the pass with one engine failure naming trigger,
/// policy and the outcome's own cause,
///  for either cause.
#[test]
fn failed_check_ends_the_pass_with_an_engine_failure() {
    let mut keep_going: Controls = no_controls();
    keep_going.keep_going = true;
    for code in [
        EngineFailureCode::ContentUnavailable,
        EngineFailureCode::PolicyIncomplete,
    ] {
        let (result, called) = run(
            Trigger::PreForward,
            &PolicyConfig::defaults(),
            &keep_going,
            &[],
            vec![
                (PolicyId::RequireRoot, found("not-at-root")),
                (
                    PolicyId::LinkedWorktreeOnly,
                    PolicyOutcome::Failed {
                        code,
                        message: String::from("git could not be asked"),
                        path: None,
                    },
                ),
            ],
        );
        assert_eq!(result.end, StageEnd::Failed);
        assert_eq!(
            called,
            [PolicyId::RequireRoot, PolicyId::LinkedWorktreeOnly]
        );
        assert_eq!(
            result.events,
            [
                event(
                    Trigger::PreForward,
                    PolicyId::RequireRoot,
                    Severity::Error,
                    "not-at-root"
                ),
                PolicyEvent::EngineFailure {
                    code,
                    message: String::from("git could not be asked"),
                    trigger: Some(Trigger::PreForward),
                    policy: Some(PolicyId::LinkedWorktreeOnly),
                    path: None,
                },
            ]
        );
        // The failure decides the exit code even though an error finding precedes it.
        assert_eq!(pass_exit_code(result.events.as_slice(), result.end), 2);
    }
}

/// A policy that cannot be evaluated ends the pass as unavailable,
///  keeping earlier events and inventing none.
#[test]
fn unavailable_policy_ends_the_pass_without_a_clean_result() {
    let (result, called) = run(
        Trigger::PreForward,
        &with_severity(
            &PolicyConfig::defaults(),
            PolicyId::AddExplicit,
            Severity::Warn,
        ),
        &no_controls(),
        &[],
        vec![(
            PolicyId::FinalNewline,
            PolicyOutcome::Unavailable("candidate content"),
        )],
    );
    assert_eq!(
        result.end,
        StageEnd::Unavailable(Unavailable::Policy {
            policy: PolicyId::FinalNewline,
            needs: "candidate content",
        })
    );
    assert_eq!(called.len(), 5);
    assert_eq!(
        result.events,
        [PolicyEvent::WarnUnsafe {
            trigger: Trigger::PreForward,
            policy: PolicyId::AddExplicit,
        }]
    );
    assert_eq!(pass_exit_code(result.events.as_slice(), result.end), 2);
}

/// A manual-push request over the given settings,
///  controls and filter.
fn manual_push(config: &PolicyConfig, controls: &Controls, selected: &[PolicyId]) -> StageRequest {
    return StageRequest {
        trigger: Trigger::ManualPush,
        config: config.clone(),
        controls: controls.clone(),
        selected: selected.to_vec(),
    };
}

/// Applicability follows the same trigger,
///  filter,
///  escape and severity tests as the stage.
#[test]
fn applicability_matches_the_stage() {
    let built_ins: PolicyConfig = PolicyConfig::defaults();
    let none: Controls = no_controls();
    let all: Vec<PolicyId> = every_policy();
    let request: StageRequest = manual_push(&built_ins, &none, &[]);
    // Only `final-newline` declares manual push among the policies that run unlisted.
    assert!(any_policy_applies(&request, all.as_slice()));
    assert!(!any_policy_applies(
        &request,
        &[PolicyId::RequireRoot, PolicyId::AddExplicit]
    ));
    assert!(!any_policy_applies(&request, &[]));
    let off: PolicyConfig = with_severity(&built_ins, PolicyId::FinalNewline, Severity::Off);
    assert!(!any_policy_applies(
        &manual_push(&off, &none, &[]),
        all.as_slice()
    ));
    let mut escaped: Controls = no_controls();
    escaped.escaped = vec![PolicyId::FinalNewline];
    assert!(!any_policy_applies(
        &manual_push(&built_ins, &escaped, &[]),
        all.as_slice()
    ));
    assert!(!any_policy_applies(
        &manual_push(&built_ins, &none, &[PolicyId::RequireRoot]),
        all.as_slice()
    ));
    // Once listed, the scanner also declares manual push.
    let configured: PolicyConfig =
        with_severity(&all_listed(), PolicyId::FinalNewline, Severity::Off);
    assert!(any_policy_applies(
        &manual_push(&configured, &none, &[]),
        all.as_slice()
    ));
}

/// An error finding of `require-root`.
fn error_event() -> PolicyEvent {
    return event(
        Trigger::PreForward,
        PolicyId::RequireRoot,
        Severity::Error,
        "not-at-root",
    );
}

/// A warning finding of `final-newline`.
fn warning_event() -> PolicyEvent {
    return event(
        Trigger::PreForward,
        PolicyId::FinalNewline,
        Severity::Warn,
        "missing",
    );
}

/// The exit code is 2 for an undecided pass,
///  1 for a blocking event,
///  and 0 otherwise.
#[test]
fn exit_code_follows_ending_then_events() {
    let core: PolicyEvent = PolicyEvent::CoreFinding {
        core_id: "commit-only",
        code: "all-flag",
        message: String::from("m"),
    };
    assert_eq!(BLOCKED_EXIT_CODE, 1);
    assert_eq!(pass_exit_code(&[], StageEnd::Completed), 0);
    assert_eq!(pass_exit_code(&[warning_event()], StageEnd::Completed), 0);
    assert_eq!(
        pass_exit_code(&[warning_event(), error_event()], StageEnd::Completed),
        1
    );
    assert_eq!(pass_exit_code(&[error_event()], StageEnd::Stopped), 1);
    assert_eq!(
        pass_exit_code(&[warning_event(), core], StageEnd::Completed),
        1
    );
    assert_eq!(pass_exit_code(&[error_event()], StageEnd::Failed), 2);
    assert_eq!(
        pass_exit_code(
            &[error_event()],
            StageEnd::Unavailable(Unavailable::Lifecycle(Trigger::PostCommit))
        ),
        2
    );
    // A pending correction blocks even when every event is a warning.
    assert_eq!(pass_exit_code(&[warning_event()], StageEnd::Proposed), 1);
    assert_eq!(pass_exit_code(&[], StageEnd::Proposed), 1);
}

/// The finding `finding(code)` offering a fix.
fn fixable(code: &'static str) -> PolicyFinding {
    let mut offered: PolicyFinding = finding(code);
    offered.fix_available = true;
    return offered;
}

/// A policy that proposes a correction ends the stage after its own findings,
///  whatever
/// its severity and even under keep-going;
///  a finding without a fix does not.
#[test]
fn a_proposed_correction_ends_the_stage() {
    let mut keep_going: Controls = no_controls();
    keep_going.keep_going = true;
    for (severity, controls) in [
        (Severity::Warn, no_controls()),
        (Severity::Error, keep_going.clone()),
    ] {
        let config: PolicyConfig = with_severity(&all_listed(), PolicyId::FinalNewline, severity);
        let (result, called) = run(
            Trigger::DirectFix,
            &config,
            &controls,
            &[],
            vec![(
                PolicyId::FinalNewline,
                PolicyOutcome::Findings(vec![finding("plain"), fixable("fixed")]),
            )],
        );
        assert_eq!(called, vec![PolicyId::FinalNewline], "{severity:?}");
        assert_eq!(result.end, StageEnd::Proposed, "{severity:?}");
        let mut expected_fixed: PolicyEvent = event(
            Trigger::DirectFix,
            PolicyId::FinalNewline,
            severity,
            "fixed",
        );
        if let PolicyEvent::Finding(offered) = &mut expected_fixed {
            offered.fix_available = true;
        }
        assert_eq!(
            result.events,
            vec![
                event(
                    Trigger::DirectFix,
                    PolicyId::FinalNewline,
                    severity,
                    "plain"
                ),
                expected_fixed,
            ]
        );
    }
    // Without a fix the stage goes on to the next policy.
    let (plain, called) = run(
        Trigger::DirectFix,
        &all_listed(),
        &no_controls(),
        &[],
        vec![(PolicyId::FinalNewline, found("plain"))],
    );
    assert_eq!(plain.end, StageEnd::Completed);
    assert!(called.len() > 1, "{called:?}");
}
