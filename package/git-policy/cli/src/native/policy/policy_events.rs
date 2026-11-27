//! What:
//!  The JSON Lines rendering of the events one policy pass produces.
//!  The events themselves,
//!  their fields and the hints they carry,
//!  are in the file beside this one.
//! Why:
//!  Agents and scripts read these lines from wrapper standard error and from
//!      `git monochromatic-git-policy-cli check|fix` standard output.
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

/// Import JSON string escaping,
///  the failure-code spellings and the schema version.
use crate::diagnostics::{
    SCHEMA_VERSION,
    engine_failure_code_name,
    json_string,
};
/// The hints that follow a warning and a stopped commit.
use crate::event_hints::{
    POST_COMMIT_BLOCKED_HINT,
    WARN_UNSAFE_HINT,
    finding_hint,
};
/// A pathname as events report it,
///  and the encoding of its exact bytes.
use crate::event_path::{
    EventPath,
    base64_standard,
};
/// The hint of one engine-failure code,
///  and the field a line carries a hint in.
use crate::invocation_config::{
    engine_failure_hint,
    push_hint_field,
};
/// The rendering of the names a policy and a severity are printed as.
use crate::policy_registry::{
    policy_descriptor,
    severity_name,
};
/// The lifecycle point a fixed transform's rejection names,
///  and the name each point is printed as.
use crate::policy_trigger::{
    Trigger,
    trigger_name,
};

/// The events themselves,
///  their fields and the hints they carry,
///  which the sibling file defines and every reader of them names.
pub use crate::policy_events_kinds::{
    EngineFailureHint,
    FindingEvent,
    FindingLocation,
    LEFT_ALONE_HINT,
    PolicyEvent,
    STOPPED_READ_HINT,
    UNPROVEN_RIPPLE_HINT,
    WARN_UNSAFE_CODE,
    event_blocks,
};

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
fn event_head(
    sequence: u64,
    event_type: &str,
) -> String {
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
fn push_text_field(
    line: &mut String,
    name: &str,
    value: &str,
) {
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
fn push_path_field(
    line: &mut String,
    path: &EventPath,
) {
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
fn push_finding(
    line: &mut String,
    finding: &FindingEvent,
) {
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
    push_hint_field(
        line,
        finding_hint(policy_name, finding.code, finding.fix_available),
    );
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
fn path_array(
    paths: &[EventPath],
    entry: fn(&EventPath) -> String,
) -> String {
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
///  The readable text of one changed path,
///  as a `changedPaths` entry.
/// Why:
///   A named function for `path_array`,
///  because `rust/avoidable-closure` reports a closure argument that a named function could replace;
///  the syntax-only anonymous-function ban once cited here was removed on 2026-10-18
///  (`doc/decision/rust-closure-lint-dylint.md`).
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
fn push_path_list(
    line: &mut String,
    text_key: &str,
    bytes_key: &str,
    paths: &[EventPath],
) {
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
        PolicyEvent::WarnUnsafe {
            ..
        } => return "configuration-warning",
        PolicyEvent::CoreFinding {
            ..
        } => return "core-finding",
        PolicyEvent::EngineFailure {
            ..
        } => return "engine-failure",
        PolicyEvent::FixSummary {
            ..
        } => return "fix-summary",
        PolicyEvent::CommitLanded {
            ..
        } => return "commit-landed",
        PolicyEvent::WordMerged {
            ..
        } => return "word-merged",
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
pub fn render_policy_event(
    sequence: u64,
    event: &PolicyEvent,
) -> String {
    // `mut` allows appending the fields of the matched variant.
    let mut line: String = event_head(sequence, event_type(event));
    match event {
        // `&mut line` lends the line for appending.
        PolicyEvent::Finding(finding) => push_finding(&mut line, finding),
        PolicyEvent::WarnUnsafe {
            trigger,
            policy,
        } => {
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
            hint,
            message,
            trigger,
            policy,
            path,
        } => {
            let code_name: &str = engine_failure_code_name(*code);
            push_text_field(&mut line, "code", code_name);
            push_text_field(&mut line, "message", message.as_str());
            push_hint_field(
                &mut line,
                match hint {
                    EngineFailureHint::OfCode => engine_failure_hint(code_name),
                    EngineFailureHint::Chosen(chosen) => *chosen,
                },
            );
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
            push_path_list(
                &mut line,
                "changedPaths",
                "changedPathBytes",
                changed_paths.as_slice(),
            );
        }
        PolicyEvent::CommitLanded {
            oid,
        } => {
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
        PolicyEvent::WordMerged {
            lines,
        } => {
            push_text_field(&mut line, "outcome", "word-merged");
            push_text_field(&mut line, "message", lines.join("\n").as_str());
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
pub fn render_policy_events(
    first_sequence: u64,
    events: &[PolicyEvent],
) -> String {
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
