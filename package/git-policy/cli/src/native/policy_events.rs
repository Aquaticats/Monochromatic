//! What:
//!  The events one policy pass produces,
//!  and their JSON Lines rendering.
//! Why:
//!  Agents and scripts read these lines from wrapper standard error and from
//!      `git cli-git check|fix` standard output.
//!  Each event is one compact JSON object with
//!      the incumbent's field names and field order,
//!  so existing readers parse native
//!      events unchanged.
//!  Events are numbered when rendered,
//!  so a list of events always
//!      prints with consecutive numbers in emission order.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // process.stderr.write(renderPolicyEvents(result.events));
//! ```

/// What:
///  `use` brings names from sibling files into this file;
///  `super::` means "the parent
///       module",
///  where every sibling file of this crate is declared.
/// Why:
///   JSON string escaping,
///  the failure-code spellings and the schema version already
///       live in `diagnostics.rs`;
///  policy and severity names live in the registry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { jsonString, SCHEMA_VERSION } from './diagnostics.ts';
/// ```
use super::diagnostics::{
    EngineFailureCode, SCHEMA_VERSION, engine_failure_code_name, json_string,
};
/// A pathname as events report it,
///  and the encoding of its exact bytes.
use super::event_hints::{
    POST_COMMIT_BLOCKED_HINT, REPLAY_HEADERS_DROPPED_HINT, WARN_UNSAFE_HINT, engine_failure_hint, finding_hint,
    push_hint_field,
};
use super::event_path::{EventPath, base64_standard};
use super::policy_registry::{PolicyId, Severity, policy_descriptor, severity_name};
use super::policy_trigger::{Trigger, trigger_name};

/// The code of the warning for a `warn` severity on a policy whose protection `warn` removes.
pub const WARN_UNSAFE_CODE: &str = "warn-unsafe";

/// What:
///  A byte range inside a candidate.
///  A `struct` is a record with named fields;
///  `u64`
///       is an unsigned 64-bit integer (siblings `u32`,
///  `usize`).
/// Why:
///   Content policies point at the exact bytes of a finding.
///  `u64` holds any file
///       offset on every platform,
///  where `usize` would be 32 bits on a 32-bit system.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FindingLocation = { byteStart: number; byteEnd: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FindingLocation {
    /// Offset of the first byte.
    pub byte_start: u64,
    /// Offset one past the last byte.
    pub byte_end: u64,
}

/// What:
///  One policy finding as it is reported.
///  `String` is owned UTF-8 text (sibling `&str`
///       borrows);
///  `Option<T>` is "a value or nothing";
///  `&'static str` is text baked into
///       the program.
/// Why:
///   The record owns its message and path because events outlive the check that made
///       them.
///  The code is compiled-in text,
///  so a shipped policy cannot emit an empty or
///       misspelled code at run time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FindingEvent = { trigger; policyId; severity: 'warn' | 'error'; code; message; path?; pathBytes?; location?; fix: 'none' | 'available' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FindingEvent {
    /// The lifecycle point that ran the policy.
    pub trigger: Trigger,
    /// The policy that reported.
    pub policy: PolicyId,
    /// The effective severity,
    ///  `Warn` or `Error`.
    pub severity: Severity,
    /// The policy-local code;
    ///  rendering prefixes the policy name.
    pub code: &'static str,
    /// The explanation for the person who ran the command.
    pub message: String,
    /// The repository path the finding is about,
    ///  when it has one.
    pub path: Option<EventPath>,
    /// The byte range the finding is about,
    ///  when it has one.
    pub location: Option<FindingLocation>,
    /// Whether the engine holds a correction for this finding.
    pub fix_available: bool,
}

/// What:
///  Every event a policy pass can produce.
///  An `enum` is a closed set of named
///       alternatives;
///  each variant carries its own fields.
///  `Vec<EventPath>` is an owned
///       list of pathnames in their event form.
/// Why:
///   One list of events is rendered in order,
///  whatever mixture of kinds it holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PolicyEvent = FindingEvent | ConfigurationWarningEvent | CoreFindingEvent | EngineFailureEvent | FixSummaryEvent;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyEvent {
    /// A policy reported something.
    Finding(FindingEvent),
    /// A policy that `warn` does not protect is configured as `warn`.
    WarnUnsafe {
        /// The lifecycle point that ran the policy.
        trigger: Trigger,
        /// The policy configured as `warn`.
        policy: PolicyId,
    },
    /// A fixed transform rejected the command;
    ///  it has no severity and cannot be configured.
    CoreFinding {
        /// The transform,
        ///  for example `commit-only`.
        core_id: &'static str,
        /// The transform-local code;
        ///  rendering prefixes the transform name.
        code: &'static str,
        /// The explanation for the person who ran the command.
        message: String,
    },
    /// The pass could not complete.
    EngineFailure {
        /// The stable cause.
        code: EngineFailureCode,
        /// The explanation for the person who ran the command.
        message: String,
        /// The lifecycle point,
        ///  when the failure belongs to one.
        trigger: Option<Trigger>,
        /// The policy that could not complete,
        ///  when one is responsible.
        policy: Option<PolicyId>,
        /// The repository path responsible,
        ///  when the failure is about one path.
        path: Option<EventPath>,
    },
    /// Corrections were applied.
    FixSummary {
        /// The fixing lifecycle point.
        trigger: Trigger,
        /// How many passes changed candidate content before it settled.
        passes: u64,
        /// The changed paths,
        ///  each once,
        ///  in Git's byte order.
        changed_paths: Vec<EventPath>,
    },
    /// The landed commit whose automatic backup the post-commit gate stopped.
    ///  The commit stays local; the event names it so the user can push it deliberately.
    CommitLanded {
        /// The exact landed commit object ID.
        oid: String,
    },
    /// A landing found the target moved (`landing-race-lost`).
    LandingRaceLost {
        /// This transaction's lost races so far, from 1.
        attempt: u64,
        /// The target value that won.
        winning_oid: String,
    },
    /// A commit that lost enough races holds the next landing slot (`landing-reserved`).
    LandingReserved {
        /// Lost races when the reservation was granted.
        lost_races: u64,
    },
    /// A prepared commit was replayed onto the moved target (`commit-replayed`).
    CommitReplayed {
        /// The prepared commit.
        prepared_oid: String,
        /// The preparation base, absent when the branch was unborn.
        from_base: Option<String>,
        /// The target value the replay used as parent.
        onto: String,
        /// The replayed commit, after revalidation.
        oid: String,
    },
    /// A signed replay could not carry some custom headers (`replay-headers-dropped`).
    ReplayHeadersDropped {
        /// The prepared commit carrying the headers.
        prepared_oid: String,
        /// The re-signed commit lacking them.
        oid: String,
        /// The dropped header names in prepared-commit order, each once.
        headers: Vec<String>,
    },
    /// The core finding `concurrent-commit/replay-conflict`, with the data a person needs to resolve it.
    ReplayConflict {
        /// The target ref the commit aimed at.
        target_ref: String,
        /// The conflicting paths in Git path byte order, as the exact bytes Git reported.
        paths: Vec<EventPath>,
        /// The earliest target commit touching a conflicting path.
        winning_oid: String,
        /// The prepared commit kept for cherry-picking.
        prepared_oid: String,
    },
}

/// What:
///  Whether an event blocks the command.
///  `&PolicyEvent` borrows the event read-only.
/// Why:
///   An error finding and a fixed-transform rejection keep Git from running;
///  a
///       warning finding and a configuration warning do not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const blocks = event.type === 'core-finding' || (event.type === 'finding' && event.severity === 'error');
/// ```
pub fn event_blocks(event: &PolicyEvent) -> bool {
    // `match` picks one arm per variant; `{ .. }` ignores the fields a variant carries.
    match event {
        PolicyEvent::Finding(finding) => return finding.severity == Severity::Error,
        PolicyEvent::CoreFinding { .. } | PolicyEvent::ReplayConflict { .. } => return true,
        PolicyEvent::WarnUnsafe { .. }
        | PolicyEvent::EngineFailure { .. }
        | PolicyEvent::FixSummary { .. }
        | PolicyEvent::CommitLanded { .. }
        | PolicyEvent::LandingRaceLost { .. }
        | PolicyEvent::LandingReserved { .. }
        | PolicyEvent::CommitReplayed { .. }
        | PolicyEvent::ReplayHeadersDropped { .. } => return false,
    }
}

/// What:
///  The opening every event shares:
///  schema version,
///  sequence and type.
/// Why:
///   Readers branch on these three fields first,
///  and they always come first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const head = `{"schemaVersion":1,"sequence":${sequence},"type":${JSON.stringify(type)}`;
/// ```
fn event_head(sequence: u64, event_type: &str) -> String {
    // `format!` builds owned text; `{{` is a literal brace.
    return format!(
        "{{\"schemaVersion\":{SCHEMA_VERSION},\"sequence\":{sequence},\"type\":{}",
        json_string(event_type)
    );
}

/// What:
///  Append `,"name":"value"` to a line.
///  `&mut String` lends the line for appending.
/// Why:
///   Every text field is encoded at this final step,
///  so no value can end its JSON
///       string or its line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// line += `,${JSON.stringify(name)}:${JSON.stringify(value)}`;
/// ```
fn push_text_field(line: &mut String, name: &str, value: &str) {
    // `.as_str()` lends the owned text to `push_str`.
    line.push_str(format!(",\"{name}\":{}", json_string(value)).as_str());
}

/// What:
///  Append `,"path":"..."` and,
///  for a name that is not UTF-8,
///  `,"pathBytes":"..."`.
///       `&EventPath` borrows the pathname's event form.
/// Why:
///   `path` keeps its meaning for every existing reader;
///  `pathBytes` is the optional
///       field schema version 1 allows (`SPEC.md`,
///  "Unknown fields may be added only in a
///       backward-compatible schema revision"),
///  present only when `path` is not exact.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// line.path = path.text; if (path.exact) line.pathBytes = base64(path.exact);
/// ```
fn push_path_field(line: &mut String, path: &EventPath) {
    push_text_field(line, "path", path.text());
    // `if let Some(exact) = ...` runs only for a name that kept its bytes.
    if let Some(exact) = path.exact() {
        push_text_field(line, "pathBytes", base64_standard(exact).as_str());
    }
}

/// What:
///  Render a finding's fields after the shared opening.
/// Why:
///   The order is the incumbent's:
///  trigger,
///  policy,
///  severity,
///  code,
///  message,
///  then the
///       optional path (with its exact bytes) and location,
///  then the fix state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify({ trigger, policyId, severity, code: `${policyId}/${code}`, message, path, location, fix })
/// ```
fn push_finding(line: &mut String, finding: &FindingEvent) {
    let policy_name: &str = policy_descriptor(finding.policy).name;
    push_text_field(line, "trigger", trigger_name(finding.trigger));
    push_text_field(line, "policyId", policy_name);
    push_text_field(line, "severity", severity_name(finding.severity));
    push_text_field(
        line,
        "code",
        format!("{policy_name}/{}", finding.code).as_str(),
    );
    push_text_field(line, "message", finding.message.as_str());
    push_hint_field(line, finding_hint(policy_name, finding.code, finding.fix_available));
    // `if let Some(path) = &finding.path` borrows the path only when one is present.
    if let Some(path) = &finding.path {
        push_path_field(line, path);
    }
    if let Some(location) = finding.location {
        line.push_str(
            format!(
                ",\"location\":{{\"byteStart\":{},\"byteEnd\":{}}}",
                location.byte_start, location.byte_end
            )
            .as_str(),
        );
    }
    let fix: &str = if finding.fix_available {
        "available"
    } else {
        "none"
    };
    push_text_field(line, "fix", fix);
}

/// What:
///  Render one JSON string per changed path:
///  its text,
///  or the base64 of its bytes.
///       `fn(&EventPath) -> String` is a plain function pointer choosing what each entry holds.
/// Why:
///   A fix summary lists every path it changed,
///  and its byte list lines up with that
///       list entry for entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify(changedPaths.map(entry))
/// ```
fn path_array(paths: &[EventPath], entry: fn(&EventPath) -> String) -> String {
    // `String::from` copies the borrowed text into an owned, growable line.
    let mut array: String = String::from("[");
    // `.iter().enumerate()` yields `(index, path)` pairs in order.
    for (index, path) in paths.iter().enumerate() {
        if index > 0 {
            array.push(',');
        }
        array.push_str(json_string(entry(path).as_str()).as_str());
    }
    array.push(']');
    return array;
}

/// What:
///  A JSON array of strings.
/// Why:
///   The replay events carry lists of names in the incumbent's `JSON.stringify` form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify(names);
/// ```
fn string_array(names: &[String]) -> String {
    let mut array: String = String::from("[");
    for (index, name) in names.iter().enumerate() {
        if index > 0 {
            array.push(',');
        }
        array.push_str(json_string(name.as_str()).as_str());
    }
    array.push(']');
    return array;
}

/// What:
///  The readable text of one changed path,
///  as a `changedPaths` entry.
/// Why:
///   A named function for `path_array`,
///  because the repository bans anonymous functions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pathText = (path: EventPath) => path.text;
/// ```
fn path_text(path: &EventPath) -> String {
    return String::from(path.text());
}

/// What:
///  The base64 of one changed path's bytes,
///  as a `changedPathBytes` entry.
/// Why:
///   Names that are UTF-8 are listed too,
///  so index `n` of both lists is the same file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pathBytes = (path: EventPath) => base64(path.bytes);
/// ```
fn path_bytes(path: &EventPath) -> String {
    return base64_standard(path.bytes());
}

/// What:
///  Append a list of paths as `,"<text_key>":[...]` and,
///  when any of them is not UTF-8,
///       `,"<bytes_key>":[...]` with one base64 entry per path.
/// Why:
///   An array cannot hold an absent entry without JSON `null`,
///  which events never
///       carry,
///  so the byte list names every path once it is needed at all,
///  and is
///       absent while every name is exact.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// line[textKey] = paths.map(p => p.text); if (paths.some(p => p.exact)) line[bytesKey] = paths.map(p => base64(p.bytes));
/// ```
fn push_path_list(line: &mut String, text_key: &str, bytes_key: &str, paths: &[EventPath]) {
    line.push_str(format!(",\"{text_key}\":{}", path_array(paths, path_text)).as_str());
    for path in paths {
        if path.exact().is_some() {
            line.push_str(format!(",\"{bytes_key}\":{}", path_array(paths, path_bytes)).as_str());
            return;
        }
    }
}

/// What:
///  The wire spelling of an event's kind.
/// Why:
///   The `type` field is the third field of every line and decides how a reader
///       interprets the rest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const type = event.type;
/// ```
fn event_type(event: &PolicyEvent) -> &'static str {
    match event {
        PolicyEvent::Finding(_) => return "finding",
        PolicyEvent::WarnUnsafe { .. } => return "configuration-warning",
        PolicyEvent::CoreFinding { .. } => return "core-finding",
        PolicyEvent::EngineFailure { .. } => return "engine-failure",
        PolicyEvent::FixSummary { .. } => return "fix-summary",
        PolicyEvent::CommitLanded { .. } => return "commit-landed",
        PolicyEvent::LandingRaceLost { .. } => return "landing-race-lost",
        PolicyEvent::LandingReserved { .. } => return "landing-reserved",
        PolicyEvent::CommitReplayed { .. } => return "commit-replayed",
        PolicyEvent::ReplayHeadersDropped { .. } => return "replay-headers-dropped",
        PolicyEvent::ReplayConflict { .. } => return "core-finding",
    }
}

/// What:
///  Render one event as a line-terminated JSON object.
///  `&PolicyEvent` borrows the
///       event;
///  the result is owned text.
/// Why:
///   One function owns every field order,
///  so two events of one kind can never differ
///       in shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function renderPolicyEvent(sequence: number, event: PolicyEvent): string { return JSON.stringify({ ...event, sequence }) + '\n'; }
/// ```
pub fn render_policy_event(sequence: u64, event: &PolicyEvent) -> String {
    // `mut` allows appending the fields of the matched variant.
    let mut line: String = event_head(sequence, event_type(event));
    match event {
        // `&mut line` lends the line for appending.
        PolicyEvent::Finding(finding) => push_finding(&mut line, finding),
        PolicyEvent::WarnUnsafe { trigger, policy } => {
            let policy_name: &str = policy_descriptor(*policy).name;
            // `*trigger` copies the small value out of the borrowed event.
            push_text_field(&mut line, "trigger", trigger_name(*trigger));
            push_text_field(&mut line, "policyId", policy_name);
            push_text_field(&mut line, "code", WARN_UNSAFE_CODE);
            push_text_field(
                &mut line,
                "message",
                format!("Policy {policy_name} is warn-unsafe but configured as warn.").as_str(),
            );
            push_hint_field(&mut line, Some(WARN_UNSAFE_HINT));
        }
        PolicyEvent::CoreFinding {
            core_id,
            code,
            message,
        } => {
            push_text_field(&mut line, "trigger", trigger_name(Trigger::PreForward));
            push_text_field(&mut line, "coreId", core_id);
            push_text_field(&mut line, "code", format!("{core_id}/{code}").as_str());
            push_text_field(&mut line, "message", message.as_str());
        }
        PolicyEvent::EngineFailure {
            code,
            message,
            trigger,
            policy,
            path,
        } => {
            let code_name: &str = engine_failure_code_name(*code);
            push_text_field(&mut line, "code", code_name);
            push_text_field(&mut line, "message", message.as_str());
            push_hint_field(&mut line, engine_failure_hint(code_name));
            if let Some(found) = trigger {
                push_text_field(&mut line, "trigger", trigger_name(*found));
            }
            if let Some(found) = policy {
                push_text_field(&mut line, "policyId", policy_descriptor(*found).name);
            }
            if let Some(found) = path {
                push_path_field(&mut line, found);
            }
        }
        PolicyEvent::FixSummary {
            trigger,
            passes,
            changed_paths,
        } => {
            push_text_field(&mut line, "trigger", trigger_name(*trigger));
            line.push_str(format!(",\"passes\":{passes}").as_str());
            push_path_list(&mut line, "changedPaths", "changedPathBytes", changed_paths.as_slice());
        }
        PolicyEvent::CommitLanded { oid } => {
            push_text_field(&mut line, "oid", oid);
            push_text_field(&mut line, "outcome", "post-commit-blocked");
            push_text_field(
                &mut line,
                "message",
                format!("Commit {oid} remains local; post-commit gate blocked automatic backup.")
                    .as_str(),
            );
            push_hint_field(&mut line, Some(POST_COMMIT_BLOCKED_HINT));
        }
        PolicyEvent::LandingRaceLost { attempt, winning_oid } => {
            line.push_str(format!(",\"attempt\":{attempt}").as_str());
            push_text_field(&mut line, "winningOid", winning_oid);
        }
        PolicyEvent::LandingReserved { lost_races } => {
            line.push_str(format!(",\"lostRaces\":{lost_races}").as_str());
        }
        PolicyEvent::CommitReplayed {
            prepared_oid,
            from_base,
            onto,
            oid,
        } => {
            push_text_field(&mut line, "preparedOid", prepared_oid);
            if let Some(base) = from_base {
                push_text_field(&mut line, "fromBase", base);
            }
            push_text_field(&mut line, "onto", onto);
            push_text_field(&mut line, "oid", oid);
        }
        PolicyEvent::ReplayHeadersDropped {
            prepared_oid,
            oid,
            headers,
        } => {
            push_text_field(&mut line, "preparedOid", prepared_oid);
            push_text_field(&mut line, "oid", oid);
            line.push_str(format!(",\"headers\":{}", string_array(headers.as_slice())).as_str());
            let plural: &str = if headers.len() == 1 { "" } else { "s" };
            push_text_field(
                &mut line,
                "message",
                format!(
                    "Replaying signed commit {prepared_oid} as {oid} dropped its {} header{plural}, because Git can re-sign only the headers git commit-tree writes.",
                    headers.join(", ")
                )
                .as_str(),
            );
            push_hint_field(&mut line, Some(REPLAY_HEADERS_DROPPED_HINT));
        }
        PolicyEvent::ReplayConflict {
            target_ref,
            paths,
            winning_oid,
            prepared_oid,
        } => {
            push_text_field(&mut line, "trigger", trigger_name(Trigger::PreForward));
            push_text_field(&mut line, "coreId", "concurrent-commit");
            push_text_field(&mut line, "code", "concurrent-commit/replay-conflict");
            push_text_field(
                &mut line,
                "message",
                format!(
                    "{target_ref} moved while this commit was prepared, and replaying it conflicts with {winning_oid} in {}; nothing landed. The prepared commit {prepared_oid} stays in the object store until gc expires it: resolve with git cherry-pick {prepared_oid}.",
                    paths.iter().map(EventPath::text).collect::<Vec<&str>>().join(", ")
                )
                .as_str(),
            );
            push_path_list(&mut line, "paths", "pathsBytes", paths.as_slice());
            push_text_field(&mut line, "winningOid", winning_oid);
            push_text_field(&mut line, "preparedOid", prepared_oid);
        }
    }
    line.push_str("}\n");
    return line;
}

/// What:
///  Render a list of events,
///  numbered consecutively from `first_sequence`.
///       `&[PolicyEvent]` borrows the list.
/// Why:
///   An invocation numbers its events once:
///  events printed before the pass (a legacy
///       configuration notice) take the first numbers and the pass continues after them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function renderPolicyEvents(firstSequence: number, events: PolicyEvent[]): string;
/// ```
pub fn render_policy_events(first_sequence: u64, events: &[PolicyEvent]) -> String {
    // `String::new()` is empty owned text.
    let mut rendered: String = String::new();
    for (index, event) in events.iter().enumerate() {
        // `as u64` widens the list index to the event number type.
        rendered.push_str(render_policy_event(first_sequence + index as u64, event).as_str());
    }
    return rendered;
}

/// Field-order,
///  escaping and numbering controls stay out of the release executable.
#[cfg(test)]
#[path = "policy_events_tests.rs"]
mod tests;
