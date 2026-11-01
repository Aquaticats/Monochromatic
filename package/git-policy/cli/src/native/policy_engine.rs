//! What:
//!  Run shipped policies in registry order for one lifecycle point and settle the
//!       outcome:
//!  which events were produced and whether the command may proceed.
//! Why:
//!  Every lifecycle shares one execution contract:
//!  a policy runs only for the triggers
//!      it declares,
//!  at the configured severity,
//!  unless it is filtered out,
//!  escaped or off;
//!      the first error finding stops the pass unless the caller asked to keep going;
//!  and a
//!      policy or lifecycle this executable cannot evaluate is reported as unavailable,
//!      never as clean.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const stage = await runPolicyStage({ policies, context, trigger, severities, selectedPolicyIds, escapedPolicyIds, keepGoing });
//! ```

/// What:
///  `use` brings names from sibling files into this file;
///  `super::` means "the parent
///       module",
///  where every sibling file of this crate is declared.
/// Why:
///   The stage combines configuration,
///  controls,
///  the registry and the event types.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { PolicyEvent } from './policy_events.ts';
/// ```
use super::action::ENGINE_FAILURE_EXIT_CODE;
use super::config_schema::PolicyConfig;
use super::diagnostics::EngineFailureCode;
use super::event_path::EventPath;
use super::policy_events::{FindingEvent, FindingLocation, PolicyEvent, event_blocks};
use super::policy_registry::{PolicyId, Severity, policy_descriptor};
use super::policy_trigger::{Trigger, policy_runs_on, trigger_is_ported};
use super::wrapper_controls::{Controls, is_escaped};

/// What:
///  Exit code when a pass ends with an error finding or a fixed-transform rejection.
///       `i32` is a signed 32-bit integer,
///  the type of process exit codes.
/// Why:
///   Callers branch on 1 for "policy said no" and on 2 for "could not decide".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BLOCKED_EXIT_CODE = 1;
/// ```
pub const BLOCKED_EXIT_CODE: i32 = 1;

/// What:
///  One thing a policy reports.
///  A `struct` is a record with named fields;
///  `String` is
///       owned UTF-8 text;
///  `Option<T>` is "a value or nothing";
///  `&'static str` is text
///       baked into the program.
/// Why:
///   A policy describes what it found;
///  the engine adds the trigger,
///  the policy name
///       and the configured severity when it turns the finding into an event.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PolicyFinding = { code: string; message: string; path?: EventPath; location?: FindingLocation; patch?: PolicyPatch };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyFinding {
    /// The policy-local code,
    ///  lower-case words joined by hyphens.
    pub code: &'static str,
    /// The explanation for the person who ran the command.
    pub message: String,
    /// The repository path the finding is about,
    ///  when it has one.
    pub path: Option<EventPath>,
    /// The byte range the finding is about,
    ///  when it has one.
    pub location: Option<FindingLocation>,
    /// Whether the policy proposed a correction for this finding.
    pub fix_available: bool,
}

/// What:
///  What one policy check returned.
///  An `enum` is a closed set of named alternatives;
///       `Failed { code, message }` carries named fields.
/// Why:
///   "Nothing found",
///  "cannot be evaluated here" and "could not finish" are three
///       different answers;
///  folding the last two into an empty finding list would let an
///       unchecked command through.
///  A policy that could not finish names its cause:
///       `content-unavailable` when candidate bytes or a repository fact could not be
///       read,
///  `policy-incomplete` when its own machinery failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PolicyOutcome = { kind: 'findings'; findings: PolicyFinding[] } | { kind: 'unavailable'; needs: string } | { kind: 'failed'; code: EngineFailureCode; message: string; path?: EventPath };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyOutcome {
    /// The policy was evaluated;
    ///  an empty list means it found nothing.
    Findings(Vec<PolicyFinding>),
    /// The policy needs something this executable does not implement yet;
    ///  the text names it.
    Unavailable(&'static str),
    /// The policy could not finish.
    Failed {
        /// The cause,
        ///  as the engine-failure event reports it.
        code: EngineFailureCode,
        /// What failed,
        ///  for the person who ran the command;
        ///  never a candidate's pathname.
        message: String,
        /// The candidate the failure is about,
        ///  when one file is responsible;
        ///  the event
        /// names it in its own field,
        ///  never inside the message.
        path: Option<EventPath>,
    },
}

/// What:
///  The policies behind the stage.
///  A `trait` is a named set of methods a type promises
///       to provide,
///  like a TS `interface`;
///  `&mut self` lends the provider for writing,
///       because a check may cache repository facts.
/// Why:
///   The stage owns order,
///  severity and stopping;
///  this seam owns what each policy
///       reads.
///  Tests drive the stage with a scripted provider,
///  the executable with real Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface PolicyChecks { check(policy: PolicyId, trigger: PolicyTrigger): Promise<PolicyOutcome> }
/// ```
pub trait PolicyChecks {
    /// Evaluate one policy for one lifecycle point.
    fn check(&mut self, policy: PolicyId, trigger: Trigger) -> PolicyOutcome;
}

/// What:
///  The settings of one stage.
///  A `struct` that owns its values:
///  `PolicyConfig`,
///       `Controls` and `Vec<PolicyId>` are copies made once per invocation.
/// Why:
///   Owning small copies keeps the record free of borrowing rules,
///  so it can be
///       handed to every stage of an invocation as one plain value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StageRequest = { trigger: PolicyTrigger; config: PolicyConfig; controls: Controls; selected: PolicyId[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StageRequest {
    /// The lifecycle point being checked.
    pub trigger: Trigger,
    /// Effective severities.
    pub config: PolicyConfig,
    /// Keep-going and per-invocation escapes.
    pub controls: Controls,
    /// The `--policy` filter of a direct command;
    ///  empty selects every policy.
    pub selected: Vec<PolicyId>,
}

/// What:
///  Why a pass could not be evaluated by this executable.
/// Why:
///   The refusal names either a whole lifecycle or one policy and what it needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Unavailable = { kind: 'lifecycle'; trigger } | { kind: 'policy'; policy; needs: string };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Unavailable {
    /// The lifecycle of this trigger is not ported.
    Lifecycle(Trigger),
    /// This policy needs something that is not ported.
    Policy {
        /// The policy that could not be evaluated.
        policy: PolicyId,
        /// What it needs,
        ///  in words for the refusal notice.
        needs: &'static str,
    },
}

/// What:
///  How a stage ended.
/// Why:
///   The caller continues only after `Completed`;
///  each other ending has its own exit code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StageEnd = 'completed' | 'stopped' | 'proposed' | 'failed' | Unavailable;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StageEnd {
    /// Every applicable policy was evaluated and none stopped the pass.
    Completed,
    /// An error finding stopped the pass.
    Stopped,
    /// A policy proposed a correction,
    ///  so the pass ends and restarts on corrected content.
    Proposed,
    /// A policy could not finish;
    ///  the last event is the engine failure naming its cause.
    Failed,
    /// A policy or the lifecycle cannot be evaluated by this executable.
    Unavailable(Unavailable),
}

/// What:
///  The events of one stage and how it ended.
///  `Vec<PolicyEvent>` is an owned list.
/// Why:
///   Events gathered before a stop are still reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StageResult = { events: PolicyEvent[]; end: StageEnd };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StageResult {
    /// Events in emission order.
    pub events: Vec<PolicyEvent>,
    /// How the stage ended.
    pub end: StageEnd,
}

/// What:
///  Whether the stage evaluates this policy.
///  `&StageRequest` borrows the settings.
/// Why:
///   The order of the tests is the incumbent's:
///  trigger,
///  direct-command filter,
///       escape,
///  then severity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const applies = policy.triggers.includes(trigger) && (selected.size === 0 || selected.has(id)) && !escaped.has(id) && severity !== 'off';
/// ```
fn policy_applies(request: &StageRequest, policy: PolicyId) -> bool {
    if !policy_runs_on(policy, request.trigger) {
        return false;
    }
    // `.contains(&policy)` borrows the identity for the lookup.
    if !request.selected.is_empty() && !request.selected.contains(&policy) {
        return false;
    }
    if is_escaped(&request.controls, policy) {
        return false;
    }
    return request.config.setting(policy).severity != Severity::Off;
}

/// What:
///  Whether any of `policies` would be evaluated for this request.
/// Why:
///   A lifecycle that only exists for its policies (manual push) is not applicable
///       when none is enabled,
///  and the caller must know that before asking the stage.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const any = policies.some(policy => applies(policy));
/// ```
pub fn any_policy_applies(request: &StageRequest, policies: &[PolicyId]) -> bool {
    // `for policy in policies` borrows each identity; `*policy` copies it out.
    for policy in policies {
        if policy_applies(request, *policy) {
            return true;
        }
    }
    return false;
}

/// What:
///  Run `policies` in the given order.
///  `&mut dyn PolicyChecks` lends "any provider of
///       checks" for writing (`dyn` means the concrete type is chosen at run time,
///  like a
///       TS interface value).
/// Why:
///   One policy at a time,
///  in registry order,
///  so findings and their numbers are
///       reproducible.
///  A trigger whose lifecycle is not ported ends the stage before any
///       policy runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runPolicyStage(request: StageRequest, policies: PolicyId[], checks: PolicyChecks): Promise<StageResult>;
/// ```
pub fn run_policy_stage(
    request: &StageRequest,
    policies: &[PolicyId],
    checks: &mut dyn PolicyChecks,
) -> StageResult {
    // `Vec::<PolicyEvent>::new()` is an empty owned list; `mut` allows pushing.
    let mut events: Vec<PolicyEvent> = Vec::<PolicyEvent>::new();
    if !trigger_is_ported(request.trigger) {
        return StageResult {
            events,
            end: StageEnd::Unavailable(Unavailable::Lifecycle(request.trigger)),
        };
    }
    for policy in policies {
        if !policy_applies(request, *policy) {
            continue;
        }
        let severity: Severity = request.config.setting(*policy).severity;
        // `match` unpacks the three outcomes; only `Findings` continues past it.
        let findings: Vec<PolicyFinding> = match checks.check(*policy, request.trigger) {
            PolicyOutcome::Findings(found) => found,
            PolicyOutcome::Unavailable(needs) => {
                return StageResult {
                    events,
                    end: StageEnd::Unavailable(Unavailable::Policy {
                        policy: *policy,
                        needs,
                    }),
                };
            }
            PolicyOutcome::Failed {
                code,
                message,
                path,
            } => {
                events.push(PolicyEvent::EngineFailure {
                    code,
                    message,
                    // `Some(x)` is the "present" case of `Option`.
                    trigger: Some(request.trigger),
                    policy: Some(*policy),
                    path,
                });
                return StageResult {
                    events,
                    end: StageEnd::Failed,
                };
            }
        };
        let found_any: bool = !findings.is_empty();
        let mut proposed: bool = false;
        // `for finding in findings` moves each finding out of the list into its event.
        for finding in findings {
            // A finding offers a fix exactly when its policy proposed a correction.
            if finding.fix_available {
                proposed = true;
            }
            events.push(PolicyEvent::Finding(FindingEvent {
                trigger: request.trigger,
                policy: *policy,
                severity,
                code: finding.code,
                message: finding.message,
                path: finding.path,
                location: finding.location,
                fix_available: finding.fix_available,
            }));
        }
        // A `warn` that removes the policy's protection is reported even when nothing was found.
        if severity == Severity::Warn && !policy_descriptor(*policy).warn_safe {
            events.push(PolicyEvent::WarnUnsafe {
                trigger: request.trigger,
                policy: *policy,
            });
        }
        // A proposal ends the stage whatever the severity, as `patchProposed` does in the
        // installed wrapper's `policy-stage.ts`: later policies read the corrected content.
        if proposed {
            return StageResult {
                events,
                end: StageEnd::Proposed,
            };
        }
        if severity == Severity::Error && found_any && !request.controls.keep_going {
            return StageResult {
                events,
                end: StageEnd::Stopped,
            };
        }
    }
    return StageResult {
        events,
        end: StageEnd::Completed,
    };
}

/// What:
///  The exit code of a pass that did not reach Git,
///  from its events and its ending.
///       `&[PolicyEvent]` borrows the events of every stage of the pass.
/// Why:
///   0 means clean or warnings only,
///  1 means an error finding,
///  a fixed-transform
///       rejection or a pending correction,
///  and 2 means the pass could not decide.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const exitCode = !complete ? 2 : (hasError || patchProposed) ? 1 : 0;
/// ```
pub fn pass_exit_code(events: &[PolicyEvent], end: StageEnd) -> i32 {
    // `Unavailable(_)` matches that variant whatever it carries; `{}` does nothing and goes on.
    match end {
        StageEnd::Failed | StageEnd::Unavailable(_) => return ENGINE_FAILURE_EXIT_CODE,
        StageEnd::Proposed => return BLOCKED_EXIT_CODE,
        StageEnd::Completed | StageEnd::Stopped => {}
    }
    for event in events {
        if event_blocks(event) {
            return BLOCKED_EXIT_CODE;
        }
    }
    return 0;
}

/// Order,
///  severity,
///  stopping and unavailability controls stay out of the release executable.
#[cfg(test)]
#[path = "policy_engine_tests.rs"]
mod tests;
