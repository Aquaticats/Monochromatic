//! What:
//!  The three stages of a pass in order,
//!  where each one can end the pass,
//!  and which
//!       arguments the pass hands back.
//! Why:
//!  A transform applied before a built-in policy read the command,
//!  or an optional
//!      policy run after a stopped stage,
//!  changes what a command is checked against.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect((await runPolicyPass(request, checks)).args).toEqual(['push', '--atomic']);
//! ```

/// The pass under test,
///  the engine types it returns and the scripted facts.
use super::{PassResult, policies_of_kind, run_policy_pass};
use crate::candidate_prediction::CandidateRequest;
use crate::command_test_support::os_arguments;
use crate::config_schema::PolicyConfig;
use crate::diagnostics::EngineFailureCode;
use crate::policy_checks::{ShippedChecks, shipped_checks};
use crate::policy_content::LifecycleContent;
use crate::policy_engine::{StageEnd, StageRequest, pass_exit_code};
use crate::policy_events::{FindingEvent, PolicyEvent};
use crate::policy_registry::{PolicyId, Severity};
use crate::policy_test_support::{ScriptedFacts, main_worktree, scripted_facts, with_severity};
use crate::policy_trigger::Trigger;
use crate::rule_commit_only_message::ALL_FLAG_MESSAGE;
use crate::wrapper_controls::{Controls, no_controls};
use std::path::PathBuf;

/// A request for `trigger` with default severities and the given controls.
fn request(trigger: Trigger, controls: Controls) -> StageRequest {
    return StageRequest {
        trigger,
        config: PolicyConfig::defaults(),
        controls,
        selected: Vec::<PolicyId>::new(),
    };
}

/// The exit code of a pass that does not reach Git:
///  0 means the command may proceed.
fn exit_code(result: &PassResult) -> i32 {
    return pass_exit_code(result.events.as_slice(), result.end);
}

/// Controls that only ask to keep going.
fn keep_going() -> Controls {
    let mut controls: Controls = no_controls();
    controls.keep_going = true;
    return controls;
}

/// Run one pass over a command and return it with the facts it asked for.
fn pass(
    stage: &StageRequest,
    values: &[&str],
    facts: ScriptedFacts,
    candidates: LifecycleContent,
) -> (PassResult, Vec<String>) {
    let mut checks: ShippedChecks<ScriptedFacts> = shipped_checks(
        facts,
        os_arguments(values),
        candidates,
        Vec::<PathBuf>::new(),
    );
    let result: PassResult = run_policy_pass(stage, &mut checks);
    // The pass never changes the arguments the checks read.
    assert_eq!(checks.arguments, os_arguments(values));
    return (result, checks.facts.asked);
}

/// Facts for a command run in `sub/` of the main worktree at `/r`.
fn below_root() -> ScriptedFacts {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.location = Ok(main_worktree("/r", "sub/"));
    return facts;
}

/// The require-root finding for a command run in `/r/sub`.
fn not_at_root() -> PolicyEvent {
    return PolicyEvent::Finding(FindingEvent {
        trigger: Trigger::PreForward,
        policy: PolicyId::RequireRoot,
        severity: Severity::Error,
        code: "not-at-root",
        message: String::from(
            "cli-git: not at the root of the git repository. Repo root is /r but effective cwd is /r/sub. This rule exists because agents and humans frequently forget which directory they are in. Tip: cd to /r or pass -C /r before the subcommand.",
        ),
        path: None,
        location: None,
        fix_available: false,
    });
}

/// The commit-only rejection of `git commit -a`.
fn all_flag_rejected() -> PolicyEvent {
    return PolicyEvent::CoreFinding {
        core_id: "commit-only",
        code: "all-flag",
        message: String::from(ALL_FLAG_MESSAGE),
    };
}

/// The registry splits into the five built-in and the four optional policies,
///  in order.
#[test]
fn the_registry_splits_into_two_stages() {
    assert_eq!(
        policies_of_kind(false),
        vec![
            PolicyId::RequireRoot,
            PolicyId::LinkedWorktreeOnly,
            PolicyId::BranchWorktreeOnly,
            PolicyId::AddExplicit,
            PolicyId::FinalNewline,
        ]
    );
    assert_eq!(
        policies_of_kind(true),
        vec![
            PolicyId::MarkdownAutofix,
            PolicyId::ForbiddenRootContext,
            PolicyId::DependentVersionBump,
            PolicyId::ForbiddenStrings,
        ]
    );
}

/// A clean forwarded command gets its transform and may proceed.
#[test]
fn a_clean_pass_hands_back_the_transformed_arguments() {
    let stage: StageRequest = request(Trigger::PreForward, no_controls());
    for (values, forwarded) in [
        (vec!["push", "origin"], vec!["push", "--atomic", "origin"]),
        (
            vec!["status"],
            vec!["-c", "advice.statusHints=false", "status"],
        ),
        (
            vec!["commit", "--dry-run", "-m", "x", "file"],
            vec!["commit", "-o", "--dry-run", "-m", "x", "file"],
        ),
        (vec!["add", "file"], vec!["add", "file"]),
    ] {
        let (result, asked) = pass(
            &stage,
            values.as_slice(),
            scripted_facts(),
            LifecycleContent::None,
        );
        assert_eq!(
            result,
            PassResult {
                arguments: os_arguments(forwarded.as_slice()),
                events: Vec::<PolicyEvent>::new(),
                end: StageEnd::Completed,
            },
            "{values:?}"
        );
        assert_eq!(exit_code(&result), 0, "{values:?}");
        assert_eq!(asked, vec![String::from("location")], "{values:?}");
    }
}

/// A built-in error ends the pass before any transform;
///  the input arguments come back.
#[test]
fn a_built_in_error_ends_the_pass_before_the_transforms() {
    let stage: StageRequest = request(Trigger::PreForward, no_controls());
    let (result, asked) = pass(
        &stage,
        &["commit", "-a", "-m", "x"],
        below_root(),
        LifecycleContent::None,
    );
    assert_eq!(
        result,
        PassResult {
            arguments: os_arguments(&["commit", "-a", "-m", "x"]),
            events: vec![not_at_root()],
            end: StageEnd::Stopped,
        }
    );
    assert_eq!(exit_code(&result), 1);
    assert_eq!(asked, vec![String::from("location")]);
}

/// With keep-going the transforms and optional policies still run,
///  and the pass still blocks.
#[test]
fn keep_going_collects_every_stage_and_still_blocks() {
    let stage: StageRequest = request(Trigger::PreForward, keep_going());
    let (rejected, _asked) = pass(
        &stage,
        &["commit", "-a", "-m", "x"],
        below_root(),
        LifecycleContent::None,
    );
    assert_eq!(
        rejected,
        PassResult {
            arguments: os_arguments(&["commit", "-a", "-m", "x"]),
            events: vec![not_at_root(), all_flag_rejected()],
            end: StageEnd::Completed,
        }
    );
    assert_eq!(exit_code(&rejected), 1);
    // The transform of a command the built-in stage rejected is still applied.
    let (pushed, _pushed_asked) = pass(&stage, &["push"], below_root(), LifecycleContent::None);
    assert_eq!(
        pushed,
        PassResult {
            arguments: os_arguments(&["push", "--atomic"]),
            events: vec![not_at_root()],
            end: StageEnd::Completed,
        }
    );
    assert_eq!(exit_code(&pushed), 1);
}

/// A transform rejection ends the pass unless keep-going;
///  a transform failure always does.
#[test]
fn a_transform_rejection_or_failure_ends_the_pass() {
    let stopped: StageRequest = request(Trigger::PreForward, no_controls());
    let (rejected, _asked) = pass(
        &stopped,
        &["commit", "-a", "-m", "x"],
        scripted_facts(),
        LifecycleContent::None,
    );
    assert_eq!(
        rejected,
        PassResult {
            arguments: os_arguments(&["commit", "-a", "-m", "x"]),
            events: vec![all_flag_rejected()],
            end: StageEnd::Stopped,
        }
    );
    assert_eq!(exit_code(&rejected), 1);
    for controls in [no_controls(), keep_going()] {
        let mut facts: ScriptedFacts = scripted_facts();
        facts.sequencer = Err(String::from("no sequencer answer"));
        let stage: StageRequest = request(Trigger::PreForward, controls);
        let (failed, asked) = pass(
            &stage,
            &["commit", "-m", "x"],
            facts,
            LifecycleContent::None,
        );
        assert_eq!(
            failed,
            PassResult {
                arguments: os_arguments(&["commit", "-m", "x"]),
                events: vec![PolicyEvent::EngineFailure {
                    code: EngineFailureCode::CoreIncomplete,
                    message: String::from("no sequencer answer"),
                    trigger: None,
                    policy: None,
                    path: None,
                }],
                end: StageEnd::Failed,
            }
        );
        assert_eq!(exit_code(&failed), 2);
        assert_eq!(
            asked,
            vec![String::from("location"), String::from("sequencer")]
        );
    }
    // The commit hatch skips the rejection, so the pass completes.
    let mut escaped: Controls = no_controls();
    escaped.commit_only_escaped = true;
    let (forwarded, _forwarded_asked) = pass(
        &request(Trigger::PreForward, escaped),
        &["commit", "-a", "-m", "x"],
        scripted_facts(),
        LifecycleContent::None,
    );
    assert_eq!(
        forwarded,
        PassResult {
            arguments: os_arguments(&["commit", "-a", "-m", "x"]),
            events: Vec::<PolicyEvent>::new(),
            end: StageEnd::Completed,
        }
    );
}

/// The candidates of `git add file`,
///  which these scripted facts cannot prepare.
fn add_candidates() -> LifecycleContent {
    return LifecycleContent::Requested(CandidateRequest::Add(os_arguments(&["file"])));
}

/// Candidates that cannot be prepared fail the pass at the first content policy that reads;
/// an unported content policy refuses without reading;
///  with none left the command proceeds.
#[test]
fn content_policies_read_fail_or_refuse_in_registry_order() {
    let (built_in, built_in_asked) = pass(
        &request(Trigger::PreForward, no_controls()),
        &["add", "file"],
        scripted_facts(),
        add_candidates(),
    );
    assert_eq!(
        built_in,
        PassResult {
            arguments: os_arguments(&["add", "file"]),
            events: vec![PolicyEvent::EngineFailure {
                code: EngineFailureCode::ContentUnavailable,
                message: String::from("the scripted facts prepare no candidates"),
                trigger: Some(Trigger::PreForward),
                policy: Some(PolicyId::FinalNewline),
                path: None,
            }],
            end: StageEnd::Failed,
        }
    );
    assert_eq!(
        built_in_asked,
        vec![String::from("location"), String::from("candidates")]
    );
    assert_eq!(exit_code(&built_in), 2);
    // With the built-in content policy escaped, a listed dependent-version policy completes `git add`.
    let config: PolicyConfig = with_severity(
        &PolicyConfig::defaults(),
        PolicyId::DependentVersionBump,
        Severity::Error,
    );
    let mut controls: Controls = no_controls();
    controls.escaped.push(PolicyId::FinalNewline);
    let optional: StageRequest = StageRequest {
        trigger: Trigger::PreForward,
        config,
        controls: controls.clone(),
        selected: Vec::<PolicyId>::new(),
    };
    let (listed, listed_asked) = pass(
        &optional,
        &["add", "file"],
        scripted_facts(),
        add_candidates(),
    );
    // The wired policy plans nothing for `git add`, so the listed command completes and asks no fact.
    assert_eq!(listed.end, StageEnd::Completed);
    assert_eq!(exit_code(&listed), 0);
    assert_eq!(listed_asked, vec![String::from("location")]);
    // With no content policy left, the same command may proceed and nothing is prepared.
    let (unlisted, unlisted_asked) = pass(
        &request(Trigger::PreForward, controls),
        &["add", "file"],
        scripted_facts(),
        add_candidates(),
    );
    assert_eq!(unlisted.end, StageEnd::Completed);
    assert_eq!(exit_code(&unlisted), 0);
    assert_eq!(unlisted_asked, vec![String::from("location")]);
}

/// A direct check runs both stages and no transform.
#[test]
fn a_direct_check_applies_no_transform() {
    let stage: StageRequest = request(Trigger::DirectCheck, no_controls());
    let (clean, asked) = pass(&stage, &["push"], scripted_facts(), LifecycleContent::None);
    assert_eq!(
        clean,
        PassResult {
            arguments: os_arguments(&["push"]),
            events: Vec::<PolicyEvent>::new(),
            end: StageEnd::Completed,
        }
    );
    assert_eq!(asked, vec![String::from("location")]);
    // The commit-only transform would reject `-a` and ask for the sequencer; a check does neither.
    let (unchanged, unchanged_asked) = pass(
        &stage,
        &["commit", "-a"],
        scripted_facts(),
        LifecycleContent::None,
    );
    assert_eq!(unchanged.events, Vec::<PolicyEvent>::new());
    assert_eq!(unchanged.arguments, os_arguments(&["commit", "-a"]));
    assert_eq!(unchanged_asked, vec![String::from("location")]);
}
