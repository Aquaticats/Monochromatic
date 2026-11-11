//! What:
//!  Why one wrapped commit did not land,
//!  as the timing record states it:
//! the policy that refused the content,
//! the fixed transform that rejected the command,
//! the engine failure the wrapper raised,
//! the Git step that failed and the status it exited with,
//! or real Git
//!  declining the command in its own words.
//! Why:
//!   A record that states only where the time went cannot say why a commit stopped,
//!  so a policy that correctly refused content read exactly like a machine failure the owner's rules forbid
//!  ("Commits should never fail.").
//! The owner's answer to a cause nobody could name is a defect that logs more
//!  ("If we can't figure it out,
//! then we have a defect and should log more."),
//! so the record names the route that stopped the commit and,
//!  where a Git step is to blame,
//! the step and its status,
//! and never the content,
//! a path or a message.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Stop = { kind: 'policy' | 'core-finding' | 'engine-failure' | 'git-step' | 'native-exit' | 'unidentified'; policy?: string; core?: string; code?: string; step?: string; status?: number };
//! ```

/// Escaping of a name inside a JSON string.
use crate::json_record::quote;
/// The wire spelling of an engine failure code.
use crate::diagnostics::engine_failure_code_name;
/// The registry name of the policy that reported.
use crate::policy_registry::policy_descriptor;
/// The Git step that failed last,
/// as the runner noted it.
use crate::git_step_failure::FailedStep;
/// The events a refused or ended commit reports.
use crate::policy_events::{PolicyEvent, event_blocks};
/// How a commit can stop without landing.
use crate::commit_transaction::TransactionStop;

/// A policy's error finding stopped the commit.
pub const KIND_POLICY: &str = "policy";

/// A fixed transform rejected the command,
/// so no policy and no Git step is to blame.
pub const KIND_CORE_FINDING: &str = "core-finding";

/// The wrapper's own engine failed.
pub const KIND_ENGINE_FAILURE: &str = "engine-failure";

/// A Git step failed and the command could not go on.
pub const KIND_GIT_STEP: &str = "git-step";

/// Real Git declined the command and exited with its own status.
pub const KIND_NATIVE_EXIT: &str = "native-exit";

/// What:
///  What the record states when no step,
/// no policy,
/// no transform and no engine failure explains the stop.
/// Why:
///   The routes that stop a commit with words of their own (the branch kept moving,
/// the content was pruned,
/// the service was not there) report
///  no step and no finding,
///  so their records say this rather than nothing:
/// it is where the owner's rule looks for a defect,
///  "If we can't figure it out,
/// then we have a defect and should log more.",
pub const KIND_UNIDENTIFIED: &str = "unidentified";

/// The step name the wrapper gives the `git commit` it builds in a scratch repository.
/// It is the step a commit Git declined names when the runner noted none.
const SCRATCH_COMMIT_STEP: &str = "git commit";

/// What:
///  Why one commit did not land,
///  as the record states it.
/// Why:
///  Every field is something the wrapper compiled in or a number:
/// a policy's registry name,
/// a transform's own identifiers,
/// an engine code,
/// the program and subcommand of a step,
/// and an exit status,
/// so no field can carry what the person typed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Stop = { kind: string; policy?: string; core?: string; code?: string; step?: string; status?: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitStopReason {
    /// The route that stopped the commit,
    /// one of the `KIND_` constants.
    kind: &'static str,
    /// The registry name of the policy that refused the content.
    policy: Option<String>,
    /// The fixed transform that rejected the command.
    core: Option<&'static str>,
    /// The transform's own code,
    /// or the engine failure's stable code.
    code: Option<&'static str>,
    /// The program and the subcommand of the step that failed,
    /// for example `git add`.
    step: Option<String>,
    /// The status that step exited with;
    /// absent when the step never exited by itself.
    status: Option<i32>,
}

impl CommitStopReason {
    /// A policy's error finding.
    pub fn policy(name: &str) -> Self {
        return Self { kind: KIND_POLICY, policy: Some(name.to_string()), core: None, code: None, step: None, status: None };
    }

    /// A fixed transform's rejection,
    /// named by the transform and by its own code.
    pub fn core_finding(core: &'static str, code: &'static str) -> Self {
        return Self { kind: KIND_CORE_FINDING, policy: None, core: Some(core), code: Some(code), step: None, status: None };
    }

    /// The wrapper's own engine failing,
    /// named by its stable code.
    pub fn engine_failure(code: &'static str) -> Self {
        return Self { kind: KIND_ENGINE_FAILURE, policy: None, core: None, code: Some(code), step: None, status: None };
    }

    /// A Git step that failed,
    /// named by the step and by the status it exited with.
    pub fn git_step(step: &str, status: Option<i32>) -> Self {
        return Self { kind: KIND_GIT_STEP, policy: None, core: None, code: None, step: Some(step.to_string()), status };
    }

    /// Real Git declining the command,
    /// named by the step it ran and by its own status.
    pub fn native_exit(step: &str, status: i32) -> Self {
        return Self { kind: KIND_NATIVE_EXIT, policy: None, core: None, code: None, step: Some(step.to_string()), status: Some(status) };
    }

    /// A commit whose route named nothing a reader can check.
    pub fn unidentified() -> Self {
        return Self { kind: KIND_UNIDENTIFIED, policy: None, core: None, code: None, step: None, status: None };
    }

    /// What the reason says,
    /// as one JSON object:
    /// `{"kind":"git-step","step":"git add","status":128}`.
    /// Why:
    ///  A field of a reason is written only when it holds something,
    /// so a reader never has to tell an empty field from an absent one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stopJson(reason) // '{"kind":"policy","policy":"final-newline"}'
    /// ```
    fn json(&self) -> String {
        let mut members: Vec<String> = vec![format!("\"kind\":{}", quote(self.kind))];
        if let Some(policy) = self.policy.as_ref() {
            members.push(format!("\"policy\":{}", quote(policy.as_str())));
        }
        if let Some(core) = self.core {
            members.push(format!("\"core\":{}", quote(core)));
        }
        if let Some(code) = self.code {
            members.push(format!("\"code\":{}", quote(code)));
        }
        if let Some(step) = self.step.as_ref() {
            members.push(format!("\"step\":{}", quote(step.as_str())));
        }
        if let Some(status) = self.status {
            members.push(format!("\"status\":{status}"));
        }
        return format!("{{{}}}", members.join(","));
    }
}

/// What:
///  The reason as the record states it,
/// or the JSON word for no value.
/// Why:
///  A commit that landed has no reason,
/// and the record says null rather than leaving the field out,
/// so a reader never tells an old record from a landed one by a missing key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stopJson = (reason) => reason === null ? 'null' : JSON.stringify(reason);
/// ```
pub fn stop_json(reason: Option<&CommitStopReason>) -> String {
    let Some(stated) = reason else {
        return String::from("null");
    };
    return stated.json();
}

/// What:
///  Why the command stopped,
///  given how it stopped and the last Git step that failed.
/// Why:
///  Each ending of the wrapper carries a different amount of evidence:
/// real Git's own status names its exit code,
/// a failed step its step and status,
/// and an engine failure its code;
/// the step is preferred over the code because the owner asked for what stopped the commit,
/// and a step that never exited by itself says so by carrying no status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reasonOfStop(stop, failed) { if (stop.nativeExit) return nativeExit(failed ?? { name: 'git commit', status: stop.code }); if (failed) return gitStep(failed); if (stop.failed) return engineFailure(stop.code); return unidentified(); }
/// ```
pub fn reason_of_stop(stop: &TransactionStop, failed: Option<FailedStep>) -> CommitStopReason {
    return match stop {
        TransactionStop::NativeExit(code) => match failed {
            Some(step) => CommitStopReason::native_exit(step.name.as_str(), step.status.unwrap_or(*code)),
            None => CommitStopReason::native_exit(SCRATCH_COMMIT_STEP, *code),
        },
        TransactionStop::Message(_) => match failed {
            Some(step) => CommitStopReason::git_step(step.name.as_str(), step.status),
            None => CommitStopReason::unidentified(),
        },
        TransactionStop::Failed { code, .. } => match failed {
            Some(step) => CommitStopReason::git_step(step.name.as_str(), step.status),
            None => CommitStopReason::engine_failure(engine_failure_code_name(*code)),
        },
    };
}

/// What:
///  Why a commit that reported events did not land.
/// Why:
///  A commit the wrapper declines ends with the events that say why,
/// and the first event that explains a non-zero exit is the reason:
/// an engine failure,
/// then a fixed transform's rejection,
/// then the policy that reported at error severity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reasonOfEvents(events) { return events.find(isEngine) ?? events.find(isCore) ?? events.find(blocks) ?? events.find(isFinding) ?? { kind: 'unidentified' }; }
/// ```
pub fn reason_of_events(events: &[PolicyEvent]) -> CommitStopReason {
    for event in events {
        if let PolicyEvent::EngineFailure { code, .. } = event {
            return CommitStopReason::engine_failure(engine_failure_code_name(*code));
        }
    }
    for event in events {
        if let PolicyEvent::CoreFinding { core_id, code, .. } = event {
            return CommitStopReason::core_finding(core_id, code);
        }
    }
    for event in events {
        if let PolicyEvent::Finding(finding) = event
            && event_blocks(event)
        {
            return CommitStopReason::policy(policy_descriptor(finding.policy).name);
        }
    }
    for event in events {
        if let PolicyEvent::Finding(finding) = event {
            return CommitStopReason::policy(policy_descriptor(finding.policy).name);
        }
    }
    return CommitStopReason::unidentified();
}

/// The reasons,
/// the routes that stop a commit and the events that explain one are controls in the test module.
#[cfg(test)]
#[path = "commit_stop_reason_tests.rs"]
mod tests;
