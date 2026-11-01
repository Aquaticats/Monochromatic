//! What:
//!  The post-commit policy gate over a landed commit's own changes, and the backup that follows it.
//! Why:
//!  The incumbent (`src/policy-engine/post-commit-lifecycle.ts`) checks the committed bytes of
//!  every path a landed commit changed, and only a gate that passes lets the commit reach auto-push.
//!  A blocked gate keeps the commit local and names it in a `commit-landed` event; the command exits `2`.
//!  The landed commit is an input, so the integrator can call this after any landing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = await runPostCommitLifecycle({ rawArgs, transformedArgs, gitPath, cwd, landedOid, ... });
//! if (!result.blocked) await autoPush({ gitPath, cwd, landedOid: result.oid });
//! ```

/// The auto-push a clean gate hands the landed commit to, and the Git request it runs with.
use super::auto_push::{AutoPushResult, auto_push};
use super::auto_push_single_flight::{PushGit, SingleFlightError};
/// The landed-commit object name and the candidate layer's landed-delta source.
use super::candidate_object::parse_object_id;
use super::candidate_prediction::CandidateRequest;
/// The policy configuration the gate runs with.
use super::config_schema::PolicyConfig;
/// The reading of the invocation's environment.
use super::child_environment::environment_value;
/// The failure code and the event kinds the gate emits.
use super::diagnostics::EngineFailureCode;
/// The linter the Markdown policy finds on, and runs with, the invocation's environment.
use super::markdown_linter::executable_linter;
use super::policy_checks::shipped_checks;
use super::policy_content::LifecycleContent;
use super::policy_engine::{StageEnd, StageRequest, pass_exit_code};
use super::policy_events::{PolicyEvent, render_policy_events};
use super::policy_pass::run_policy_pass;
use super::policy_trigger::Trigger;
use super::repository_facts::git_facts;
/// The variable naming the forbidden-strings rules file.
use super::scanner_selection::RULES_VARIABLE;
use super::wrapper_controls::Controls;
/// Owned OS strings for the replayed global options and the child environment.
use std::ffi::OsString;
/// `Write` is the stderr sink the gate's JSONL lines go to.
use std::io::Write;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// Exit status the command returns when the gate blocks the backup, as the incumbent's `PolicyDecisionError(2)`.
pub const POST_COMMIT_BLOCKED_EXIT_CODE: i32 = 2;

/// What:
///  The inputs of one post-commit gate.
/// Why:
///   The landed commit names the delta; the global prefix and the environment make every query read
///  the repository the commit landed in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PostCommitInput = { gitPath; cwd; landedOid; arguments; environment; policySeverities; policyOptions };
/// ```
#[derive(Clone, Debug)]
pub struct PostCommitRequest<'request> {
    /// Absolute real Git executable.
    pub real_git: &'request Path,
    /// The landed commit object name, as Git printed it.
    pub landed_oid: &'request str,
    /// The arguments the command forwarded to Git.
    pub arguments: &'request [OsString],
    /// The global options of the invocation, kept before every query (`-C`, `--git-dir`, `-c`).
    pub global_prefix: &'request [OsString],
    /// Variables added to every query.
    pub overlay: &'request [(OsString, OsString)],
    /// The invocation's whole environment:
    ///  the Markdown linter is found on its `PATH` and runs with it,
    ///  and the forbidden-strings rules variable is read from it.
    pub environment: &'request [(OsString, OsString)],
    /// The policy severities and options the gate runs with.
    pub config: PolicyConfig,
    /// Escapes and flags the invocation carries.
    pub controls: Controls,
}

/// What:
///  The settled gate:
///  the landed commit, every event it produced, and whether the backup must not run.
/// Why:
///   `events` already carries the `commit-landed` event when `blocked` is true,
///  so the caller renders one list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PostCommitLifecycleResult = { oid: string; events: PolicyEvent[]; blocked: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostCommitGate {
    /// The landed commit object name, as given.
    pub oid: String,
    /// Every event the gate produced, in order.
    pub events: Vec<PolicyEvent>,
    /// Whether the backup must not run.
    pub blocked: bool,
}

/// What:
///  Run the post-commit policies over the landed commit's delta.
/// Why:
///   Only a gate with no blocking event, no failure and no pending correction lets the backup run;
///  a warning-only result is allowed, as in the incumbent.
///   A landed name that is not an object name is an engine failure, so the commit is kept and reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runPostCommitLifecycle(input): Promise<PostCommitLifecycleResult>;
/// ```
pub fn run_post_commit_gate(request: &PostCommitRequest) -> PostCommitGate {
    let oid = request.landed_oid.to_owned();
    let (mut events, end): (Vec<PolicyEvent>, StageEnd) =
        match parse_object_id(request.landed_oid.as_bytes()) {
            Some(commit) => run_staged_gate(request, commit),
            None => (
                vec![PolicyEvent::EngineFailure {
                    code: EngineFailureCode::ContentUnavailable,
                    message: format!("landed commit {} is not an object name", request.landed_oid),
                    trigger: Some(Trigger::PostCommit),
                    policy: None,
                    path: None,
                }],
                StageEnd::Failed,
            ),
        };
    let blocked = pass_exit_code(events.as_slice(), end) != 0;
    if blocked {
        events.push(PolicyEvent::CommitLanded { oid: oid.clone() });
    }
    return PostCommitGate {
        oid,
        events,
        blocked,
    };
}

/// Run the post-commit stage over the landed delta of a parsed commit name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await dependencies.runPolicyEngine({ trigger: 'post-commit', gitFacts: createPostCommitGitFacts(...) })
/// ```
fn run_staged_gate(
    request: &PostCommitRequest,
    commit: super::candidate_object::ObjectId,
) -> (Vec<PolicyEvent>, StageEnd) {
    let facts = git_facts(request.real_git, request.global_prefix, request.overlay);
    let mut checks = shipped_checks(
        facts,
        request.arguments.to_vec(),
        LifecycleContent::Requested(CandidateRequest::Landed(commit)),
        Vec::new(),
    );
    // The scanner and Markdown options are the configuration's, as the pre-commit stage's checks hold them.
    checks.scanner_settings.options = request.config.forbidden_strings.clone();
    checks.markdown.options = request.config.markdown_autofix.clone();
    // The forbidden-strings rules variable is read from this invocation's environment.
    checks.scanner_settings.rules_variable = environment_value(request.environment, RULES_VARIABLE);
    // The Markdown linter is found on, and runs with, this invocation's environment.
    checks.markdown.linter = Box::new(executable_linter(request.environment));
    let stage: StageRequest = StageRequest {
        trigger: Trigger::PostCommit,
        config: request.config.clone(),
        controls: request.controls.clone(),
        selected: Vec::new(),
    };
    let pass = run_policy_pass(&stage, &mut checks);
    return (pass.events, pass.end);
}

/// What:
///  The result of the gate and the backup it allowed or prevented.
/// Why:
///   The exit status and the auto-push result come from one place, so the caller only reports them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { gate: PostCommitLifecycleResult; autoPush?: AutoPushResult }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostCommitOutcome {
    /// The gate result, including the `commit-landed` event when blocked.
    pub gate: PostCommitGate,
    /// `0` after a backup or a warning-only gate; `2` when the gate blocked the backup.
    pub exit_code: i32,
    /// The auto-push result, absent when the gate blocked the backup.
    pub auto_push: Option<AutoPushResult>,
}

/// What:
///  Run the gate, print its events, and back up the landed commit only when the gate allows it.
/// Why:
///   The incumbent prints the events before the backup and never backs up a blocked commit;
///  a failed backup is reported by `auto_push` and never changes the outcome's exit code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const result = await runPostCommitLifecycle(input);
/// process.stderr.write(renderPolicyEvents(result.events));
/// if (!result.blocked) await autoPush({ gitPath, cwd, landedOid: result.oid });
/// ```
pub fn run_post_commit(
    request: &PostCommitRequest,
    git: &PushGit,
    err: &mut dyn Write,
) -> Result<PostCommitOutcome, SingleFlightError> {
    let gate = run_post_commit_gate(request);
    let rendered = render_policy_events(0, gate.events.as_slice());
    if !rendered.is_empty() {
        err.write_all(rendered.as_bytes())
            .map_err(SingleFlightError::Io)?;
    }
    if gate.blocked {
        return Ok(PostCommitOutcome {
            gate,
            exit_code: POST_COMMIT_BLOCKED_EXIT_CODE,
            auto_push: None,
        });
    }
    let landed = gate.oid.clone();
    let pushed = auto_push(git, landed.as_str(), err)?;
    return Ok(PostCommitOutcome {
        gate,
        exit_code: 0,
        auto_push: Some(pushed),
    });
}

/// Post-commit gate behavior is controlled in the test module.
#[cfg(test)]
#[path = "post_commit_tests.rs"]
mod tests;
