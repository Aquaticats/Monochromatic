//! What:
//!  The post-commit policy gate over a landed commit's own changes.
//! Why:
//!  The gate checks the committed bytes of every path a landed commit changed,
//! and only a gate that passes lets the commit reach auto-push.
//! The push itself is
//!  the service's work item,
//! handed over by `commit_command.rs` after a passing gate.
//!  A blocked gate keeps the commit local and names it in a `commit-landed` event;
//! the command exits `2`.
//!  The landed commit is an input,
//! so the integrator can call this after any landing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = await runPostCommitLifecycle({ rawArgs, transformedArgs, gitPath, cwd, landedOid, ... });
//! ```

/// The landed-commit object name and the candidate layer's landed-delta source.
use crate::candidate_object::parse_object_id;
use crate::candidate_prediction::CandidateRequest;
/// The policy configuration the gate runs with.
use crate::config_schema::PolicyConfig;
/// The reading of the invocation's environment.
use crate::child_environment::environment_value;
/// The failure code and the event kinds the gate emits.
use crate::diagnostics::EngineFailureCode;
/// The linter the Markdown policy finds on,
/// and runs with,
/// the invocation's environment.
use crate::policy_checks::shipped_checks;
use crate::policy_content::LifecycleContent;
use crate::policy_engine::{StageEnd, StageRequest, pass_exit_code};
use crate::policy_events::PolicyEvent;
/// The effective severity of a finding,
/// and the registry row that names a policy the way the person configures it.
use crate::policy_registry::{Severity, policy_descriptor};
use crate::policy_pass::run_policy_pass;
use crate::policy_trigger::Trigger;
use crate::repository_facts::git_facts;
/// The variable naming the forbidden-strings rules file.
use crate::scanner_selection::RULES_VARIABLE;
use crate::wrapper_controls::Controls;
/// Owned OS strings for the replayed global options and the child environment.
use std::ffi::OsString;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// Exit status the command returns when the gate blocks the backup,
/// as the incumbent's `PolicyDecisionError(2)`.
pub const POST_COMMIT_BLOCKED_EXIT_CODE: i32 = 2;

/// What:
///  The inputs of one post-commit gate.
/// Why:
///   The landed commit names the delta;
/// the global prefix and the environment make every query read
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
    /// The landed commit object name,
    /// as Git printed it.
    pub landed_oid: &'request str,
    /// The arguments the command forwarded to Git.
    pub arguments: &'request [OsString],
    /// The global options of the invocation,
    /// kept before every query (`-C`,
    /// `--git-dir`,
    /// `-c`).
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
///  the landed commit,
/// every event it produced,
/// and whether the backup must not run.
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
    /// The landed commit object name,
    /// as given.
    pub oid: String,
    /// Every event the gate produced,
    /// in order.
    pub events: Vec<PolicyEvent>,
    /// Whether the backup must not run.
    pub blocked: bool,
}

/// What:
///  Run the post-commit policies over the landed commit's delta.
/// Why:
///   Only a gate with no blocking event,
/// no failure and no pending correction lets the backup run;
///  a warning-only result is allowed,
/// as in the incumbent.
///   A landed name that is not an object name is an engine failure,
/// so the commit is kept and reported.
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
                    hint: crate::policy_events::EngineFailureHint::OfCode,
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
    commit: crate::candidate_object::ObjectId,
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
///  The words the automatic push and every command's notice give for a gate that kept the commit local.
/// Why:
///   The person has to know which check refused the commit and what it found,
/// because the automatic push
///  stops at that commit until it is off the branch's way out.
///  Only the events that block are named,
/// in the order the gate produced them:
/// an error finding names its policy,
/// a fixed transform
///  that rejected the command names itself,
/// and a pass that could not finish says that on its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = (events) => events.filter(blocking).map(describe).join('; ').replace(/\s+/g, ' ');
/// ```
pub fn blocked_reason(events: &[PolicyEvent]) -> String {
    let mut said: Vec<String> = Vec::new();
    for event in events {
        match event {
            PolicyEvent::Finding(finding) if finding.severity == Severity::Error => {
                said.push(format!("{}: {}", policy_descriptor(finding.policy).name, finding.message));
            }
            PolicyEvent::CoreFinding { core_id, message, .. } => {
                said.push(format!("{core_id}: {message}"));
            }
            PolicyEvent::EngineFailure { policy, message, .. } => {
                said.push(match policy {
                    Some(known) => format!("{}: {message}", policy_descriptor(*known).name),
                    None => String::from(message.as_str()),
                });
            }
            PolicyEvent::Finding(_) | PolicyEvent::WarnUnsafe { .. } | PolicyEvent::FixSummary { .. } | PolicyEvent::CommitLanded { .. } | PolicyEvent::WordMerged { .. } => {}
        }
    }
    // A gate that stopped without a blocking event still has a cause in its own words; the caller words that case.
    let joined: String = said.join("; ");
    return joined.split_whitespace().collect::<Vec<&str>>().join(" ");
}

/// What:
///  The reason a gate that produced no blocking event is reported with.
/// Why:
///   A pass can end without an event that blocks (an empty finding list and a stage that did not finish),
/// and the
///  notice must still name something the person can act on rather than an empty clause.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NO_CAUSE = 'the post-commit check did not finish';
/// ```
pub const NO_BLOCKING_CAUSE: &str = "its post-commit check did not finish";

/// Post-commit gate behavior is controlled in the test module.
#[cfg(test)]
#[path = "post_commit_tests.rs"]
mod tests;
