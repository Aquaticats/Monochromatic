//! What:
//!  Controls for the catalog-specifier policy inside the policy stage:
//!  which severity runs it,
//! what each severity does to the command,
//! and the points where it is skipped.
//! Why:
//!  The policy is optional and ships switched off,
//! so a configuration that never names it must run nothing for it,
//!  and the severities `warn` and `error` must differ exactly in whether the command may go on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(runPolicyStage({ trigger: 'pre-forward', config: off, ... }).events).toEqual([]);
//! ```
#![cfg(unix)]

use super::{os, write};
use crate::candidate_prediction::CandidateRequest;
use crate::config_schema::PolicyConfig;
use crate::policy_checks::{ShippedChecks, shipped_checks};
use crate::policy_content::LifecycleContent;
use crate::policy_engine::{StageEnd, StageRequest, StageResult, pass_exit_code, run_policy_stage};
use crate::policy_events::PolicyEvent;
use crate::policy_registry::{PolicyId, Severity};
use crate::policy_test_support::{ScriptedFacts, scripted_facts, with_severity};
use crate::policy_trigger::Trigger;
use crate::test_support::{fixture, remove, repository};
use crate::wrapper_controls::{Controls, no_controls};
use std::path::{Path, PathBuf};

/// A repository with one manifest that holds a literal version,
/// and the shipped checks over `git add package.json`.
fn checks_over(repo: &Path) -> ShippedChecks<ScriptedFacts> {
    write(repo, "package.json", br#"{"dependencies": {"zod": "^4.6.7"}}"#);
    let mut facts: ScriptedFacts = scripted_facts();
    facts.candidates_repository = Some(repo.to_path_buf());
    return shipped_checks(
        facts,
        os(&["add", "package.json"]),
        LifecycleContent::Requested(CandidateRequest::Add(os(&["package.json"]))),
        Vec::<PathBuf>::new(),
    );
}

/// The stage over this one policy at this trigger,
/// severity and controls.
fn stage(
    checks: &mut ShippedChecks<ScriptedFacts>,
    trigger: Trigger,
    severity: Option<Severity>,
    controls: Controls,
) -> StageResult {
    let defaults: PolicyConfig = PolicyConfig::defaults();
    let config: PolicyConfig = match severity {
        Some(chosen) => with_severity(&defaults, PolicyId::CatalogSpecifiers, chosen),
        None => defaults,
    };
    let request: StageRequest = StageRequest { trigger, config, controls, selected: Vec::new() };
    return run_policy_stage(&request, &[PolicyId::CatalogSpecifiers], checks);
}

/// The policy ships off:
/// the configuration defaults run nothing for it and never read a candidate,
///  at every lifecycle point that offers content;
/// `off` listed explicitly does the same.
#[test]
fn an_unlisted_or_off_policy_runs_nothing_and_reads_nothing() {
    let root: PathBuf = fixture("catalog-stage-off");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let mut checks: ShippedChecks<ScriptedFacts> = checks_over(repo.as_path());
    for trigger in [Trigger::PreForward, Trigger::PostCommit, Trigger::ManualPush, Trigger::DirectCheck] {
        for severity in [None, Some(Severity::Off)] {
            let result: StageResult = stage(&mut checks, trigger, severity, no_controls());
            assert_eq!(result.events, Vec::<PolicyEvent>::new(), "{trigger:?} {severity:?}");
            assert_eq!(result.end, StageEnd::Completed);
        }
    }
    assert_eq!(checks.facts.asked, Vec::<String>::new(), "a skipped policy must not prepare candidates");
    remove(root.as_path());
}

/// `error` blocks the command at the first finding;
/// `warn` reports the same finding with the unsafe-warn notice and lets
///  the command go on;
/// the finding keeps the file and the rule's code.
#[test]
fn warn_reports_and_error_blocks() {
    let root: PathBuf = fixture("catalog-stage-severity");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let mut checks: ShippedChecks<ScriptedFacts> = checks_over(repo.as_path());
    let blocked: StageResult = stage(&mut checks, Trigger::PreForward, Some(Severity::Error), no_controls());
    assert_eq!(blocked.end, StageEnd::Stopped);
    assert_eq!(blocked.events.len(), 1);
    assert_eq!(pass_exit_code(blocked.events.as_slice(), blocked.end), 1);
    let PolicyEvent::Finding(finding) = &blocked.events[0] else {
        panic!("a finding event, not {:?}", blocked.events[0]);
    };
    assert_eq!((finding.policy, finding.severity, finding.code), (PolicyId::CatalogSpecifiers, Severity::Error, "disallowed-specifier"));
    let warned: StageResult = stage(&mut checks, Trigger::PreForward, Some(Severity::Warn), no_controls());
    assert_eq!(warned.end, StageEnd::Completed);
    assert_eq!(warned.events.len(), 2, "the finding and the unsafe-warn notice");
    assert_eq!(pass_exit_code(warned.events.as_slice(), warned.end), 0);
    remove(root.as_path());
}

/// A direct fix has no correction to offer,
/// and the escape written before the command skips the policy,
///  with or without keep-going.
#[test]
fn a_direct_fix_and_an_escape_skip_the_policy() {
    let root: PathBuf = fixture("catalog-stage-skips");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let mut checks: ShippedChecks<ScriptedFacts> = checks_over(repo.as_path());
    let fix: StageResult = stage(&mut checks, Trigger::DirectFix, Some(Severity::Error), no_controls());
    assert_eq!((fix.events, fix.end), (Vec::<PolicyEvent>::new(), StageEnd::Completed));
    let mut escaped: Controls = no_controls();
    escaped.escaped.push(PolicyId::CatalogSpecifiers);
    let result: StageResult = stage(&mut checks, Trigger::PreForward, Some(Severity::Error), escaped);
    assert_eq!((result.events, result.end), (Vec::<PolicyEvent>::new(), StageEnd::Completed));
    assert_eq!(checks.facts.asked, Vec::<String>::new());
    remove(root.as_path());
}
