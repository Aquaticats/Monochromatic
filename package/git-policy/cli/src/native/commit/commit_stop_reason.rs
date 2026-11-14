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
/// The failure a command decided its own stop from,
/// and the word the record names it by.
use crate::service_messages::{ClientFailure, failure_code};

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
///  A signal stopped the command while it waited for the commit service,
///  so the commit was not made.
/// Why:
///  The record states the signal and,
///  where the kernel names the process that sent it,
///  that process:
///  a keyboard's interrupt and the stop a caller sends at its own bound are the same `SIGTERM`,
///  and only the sender tells them apart.
pub const KIND_CLIENT_SIGNAL: &str = "client-signal";

/// What:
///  The command itself did not make the commit,
///  and the record says which failure it was.
/// Why:
///  These stops are the ones the client knows best and the record named worst:
///  a service of another release that would not stop,
///  a service manager that answered nothing within its bound,
///  a service that answered `unavailable`,
///  a wait that reached its ceiling.
///  Every one of them used to land in `unidentified`,
///  which the owner's rule reads as a defect,
///  although the person had been told the cause in words.
///  The name matches the type the client decides with (`ClientFailure`),
///  and the `reason` member carries `service_messages::failure_code`.
///  "Refusal" would be the wrong word for the ones that are a wait that ran out.
pub const KIND_CLIENT_FAILURE: &str = "client-failure";

/// What:
///  What the record states when no step,
/// no policy,
/// no transform and no engine failure explains the stop.
/// Why:
///   The routes that stop a commit with words the wrapper did not name (a Git failure that noted no step,
/// a run of events that explains nothing) report
///  no step and no finding,
///  so their records say this rather than nothing:
/// it is where the owner's rule looks for a defect,
///  "If we can't figure it out,
/// then we have a defect and should log more."
///  A record that says only this cannot say how far the wrapper looked,
///  so the reason names the routes that were checked and were not it (`ruledOut`).
pub const KIND_UNIDENTIFIED: &str = "unidentified";

/// The routes a stop in the wrapper's own words was checked against and is none of.
/// Why:
///   A reader of an `unidentified` record needs to know what was ruled out:
///  every route a commit can stop by is named,
///  and the record says they were all absent.
const EVERY_ROUTE_A_MESSAGE_IS_NOT: [&str; 7] = [KIND_POLICY, KIND_CORE_FINDING, KIND_ENGINE_FAILURE, KIND_GIT_STEP, KIND_NATIVE_EXIT, KIND_CLIENT_SIGNAL, KIND_CLIENT_FAILURE];

/// What:
///  The routes a run of events that explains nothing was checked against.
/// Why:
///  `reason_of_events` reads the events of a commit that reported a result,
///  so only the routes an event can carry were looked for.
const EVERY_ROUTE_AN_EVENT_IS_NOT: [&str; 3] = [KIND_ENGINE_FAILURE, KIND_CORE_FINDING, KIND_POLICY];

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
/// type Stop = { kind: string; policy?: string; core?: string; code?: string; step?: string; status?: number; signal?: number; sender?: number; reason?: string; ruledOut?: string[] };
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
    /// The signal that stopped the command;
    /// absent unless a signal did.
    signal: Option<i32>,
    /// The process that sent that signal,
    /// where the kernel said which one did;
    /// absent for a signal from a terminal or from the kernel,
    /// and for every stop that is not a signal.
    sender: Option<i32>,
    /// What the command's own failure came to,
    /// in one word from `service_messages::failure_code`;
    /// absent unless the command decided the stop itself.
    reason: Option<&'static str>,
    /// The routes that were checked and are not what stopped the commit,
    /// named where no route could be:
    /// absent unless the record says `unidentified`.
    ruled_out: &'static [&'static str],
}

impl CommitStopReason {
    /// A reason of one kind,
    /// with the member each kind fills.
    fn only(kind: &'static str) -> Self {
        return Self { kind, policy: None, core: None, code: None, step: None, status: None, signal: None, sender: None, reason: None, ruled_out: &[] };
    }

    /// A policy's error finding.
    pub fn policy(name: &str) -> Self {
        return Self { policy: Some(name.to_string()), ..Self::only(KIND_POLICY) };
    }

    /// A fixed transform's rejection,
    /// named by the transform and by its own code.
    pub fn core_finding(core: &'static str, code: &'static str) -> Self {
        return Self { core: Some(core), code: Some(code), ..Self::only(KIND_CORE_FINDING) };
    }

    /// The wrapper's own engine failing,
    /// named by its stable code.
    pub fn engine_failure(code: &'static str) -> Self {
        return Self { code: Some(code), ..Self::only(KIND_ENGINE_FAILURE) };
    }

    /// A Git step that failed,
    /// named by the step and by the status it exited with.
    pub fn git_step(step: &str, status: Option<i32>) -> Self {
        return Self { step: Some(step.to_string()), status, ..Self::only(KIND_GIT_STEP) };
    }

    /// Real Git declining the command,
    /// named by the step it ran and by its own status.
    pub fn native_exit(step: &str, status: i32) -> Self {
        return Self { step: Some(step.to_string()), status: Some(status), ..Self::only(KIND_NATIVE_EXIT) };
    }

    /// A commit whose route named nothing a reader can check,
    /// with the routes that were looked for and were not it.
    pub fn unidentified(ruled_out: &'static [&'static str]) -> Self {
        return Self { ruled_out, ..Self::only(KIND_UNIDENTIFIED) };
    }

    /// The person stopping the command with a signal,
    /// named by the signal and,
    /// where the kernel said which process sent it,
    /// by that process.
    pub fn client_signal(signal: i32, sender: Option<i32>) -> Self {
        return Self { signal: Some(signal), sender, ..Self::only(KIND_CLIENT_SIGNAL) };
    }

    /// The command's own failure,
    /// named by the word that failure goes by.
    pub fn client_failure(failure: &ClientFailure) -> Self {
        return Self { reason: Some(failure_code(failure)), ..Self::only(KIND_CLIENT_FAILURE) };
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
        if let Some(signal) = self.signal {
            members.push(format!("\"signal\":{signal}"));
        }
        if let Some(sender) = self.sender {
            members.push(format!("\"sender\":{sender}"));
        }
        if let Some(reason) = self.reason {
            members.push(format!("\"reason\":{}", quote(reason)));
        }
        if !self.ruled_out.is_empty() {
            let routes: Vec<String> = self.ruled_out.iter().map(quote_route).collect();
            members.push(format!("\"ruledOut\":[{}]", routes.join(",")));
        }
        return format!("{{{}}}", members.join(","));
    }
}

/// One route of the list an `unidentified` reason was checked against,
/// as a JSON string.
/// Why:
///  The names are this crate's own literals,
///  and `quote` escapes them for the line;
///  a named function keeps the map from a closure that captures nothing (`rust/avoidable-closure`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const quoteRoute = (route: string) => JSON.stringify(route);
/// ```
fn quote_route(route: &&'static str) -> String {
    return quote(route);
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
/// function reasonOfStop(stop, failed) { if (stop.nativeExit) return nativeExit(failed ?? { name: 'git commit', status: stop.code }); if (stop.client) return clientFailure(stop.client); if (failed) return gitStep(failed); if (stop.failed) return engineFailure(stop.code); if (stop.signal) return clientSignal(stop.signal, stop.sender); return unidentified(everyRouteAMessageIsNot); }
/// ```
pub fn reason_of_stop(stop: &TransactionStop, failed: Option<FailedStep>) -> CommitStopReason {
    return match stop {
        TransactionStop::NativeExit(code) => match failed {
            Some(step) => CommitStopReason::native_exit(step.name.as_str(), step.status.unwrap_or(*code)),
            None => CommitStopReason::native_exit(SCRATCH_COMMIT_STEP, *code),
        },
        // What the command decided is what the record states, and a step that failed beside it never explains it: the failure
        //  names the route, and a later step's note is the one the runner happened to keep.
        TransactionStop::Client(failure) => CommitStopReason::client_failure(failure),
        TransactionStop::Message(_) => match failed {
            Some(step) => CommitStopReason::git_step(step.name.as_str(), step.status),
            None => CommitStopReason::unidentified(&EVERY_ROUTE_A_MESSAGE_IS_NOT),
        },
        TransactionStop::Failed { code, .. } => match failed {
            Some(step) => CommitStopReason::git_step(step.name.as_str(), step.status),
            None => CommitStopReason::engine_failure(engine_failure_code_name(*code)),
        },
        TransactionStop::EndedBySignal { signal, sender, .. } => CommitStopReason::client_signal(*signal, *sender),
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
    return CommitStopReason::unidentified(&EVERY_ROUTE_AN_EVENT_IS_NOT);
}

/// The reasons,
/// the routes that stop a commit and the events that explain one are controls in the test module.
#[cfg(test)]
#[path = "commit_stop_reason_tests.rs"]
mod tests;
