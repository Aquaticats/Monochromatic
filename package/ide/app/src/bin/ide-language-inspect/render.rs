//! JSON rendering of what the Language module publishes,
//!  for the inspection record.

/// `json_value!` is `json!` with the one lint allowance the repository's `unwrap` ban needs.
use ide_app::json_value;
/// The published types being rendered.
use ide_app::language::{
    diagnostics::DiagnosticsSnapshot,
    hints::HintsSnapshot,
    identity::ServerIdentity,
    reply::{LanguageReply, RequestOutcome, Target},
    status::LanguageStatus,
};
/// What:
///  `Value` is any JSON value.
/// Why:
///  Observations are printed as one JSON object per line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Value = unknown;
/// ```
use serde_json::Value;
/// Debug formatting names the states and kinds the record shows.
use std::fmt::Debug;

/// Longest text copied into the record,
///  in characters.
const SHOWN_CHARS: usize = 2000;

/// Shorten text for the record.
fn shown(text: &str) -> String {
    // `chars().take(n).collect()` keeps at most `n` characters in a new owned `String`.
    return text.chars().take(SHOWN_CHARS).collect();
}

/// Format a value that may be absent with its debug formatting,
///  keeping the absence.
fn debugged<T: Debug>(value: Option<T>) -> Option<String> {
    // The trailing `?` returns `None` when the value is absent.
    let present = value?;
    return Some(format!("{present:?}"));
}

/// Name the server that answered as `name#instance`,
///  keeping the absence of one.
fn identified(server: Option<&ServerIdentity>) -> Option<String> {
    // The trailing `?` returns `None` when no server answered.
    let answering = server?;
    return Some(format!("{}#{}", answering.name, answering.instance));
}

/// Render the outcome of one reply.
fn outcome(found: &RequestOutcome) -> Value {
    // What: `match` unpacks the tagged union; each arm builds the JSON for one variant.
    // Why: The record must show exactly which state a real server produced.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (found.kind) { case 'hover': return { state: 'hover', text: found.hover.text }; }
    // ```
    return match found {
        RequestOutcome::Locations(targets) => {
            // `Vec::new()` creates an empty growable list for the rendered targets.
            let mut rendered = Vec::new();
            for target in targets {
                rendered.push(match target {
                    Target::Open(open) => json_value!({
                        "path": open.path,
                        "outsideProject": open.outside_project,
                        "sameDocument": open.same_document,
                        "line": open.line,
                        "range": open.range,
                    }),
                    Target::Unavailable { uri, refusal } => json_value!({
                        "unavailable": uri,
                        "reason": refusal.to_string(),
                    }),
                });
            }
            json_value!({ "state": "locations", "targets": rendered })
        }
        RequestOutcome::Hover(hover) => json_value!({
            "state": "hover",
            "markdown": hover.markdown,
            "range": hover.range,
            "text": shown(&hover.text),
        }),
        RequestOutcome::Empty => json_value!({ "state": "empty" }),
        RequestOutcome::Unsupported => json_value!({ "state": "unsupported" }),
        RequestOutcome::Starting => json_value!({ "state": "starting" }),
        RequestOutcome::Unsynchronized => json_value!({ "state": "unsynchronized" }),
        RequestOutcome::NoServer => json_value!({ "state": "no-server" }),
        RequestOutcome::Superseded => json_value!({ "state": "superseded" }),
        // `{failure:?}` writes the failure with its debug formatting.
        RequestOutcome::Failed(failure) => {
            json_value!({ "state": "failed", "failure": format!("{failure:?}") })
        }
    };
}

/// Render one reply with the identities it carries.
pub fn reply(found: &LanguageReply) -> Value {
    return json_value!({
        "request": found.request,
        "file": found.stamp.file,
        "revision": found.stamp.revision,
        // The server is rendered only when one answered.
        "server": identified(found.server.as_ref()),
        "remaining": found.remaining,
        "outcome": outcome(&found.outcome),
    });
}

/// Render the status rows.
pub fn status(found: &LanguageStatus) -> Value {
    let mut servers = Vec::new();
    for row in &found.servers {
        servers.push(json_value!({
            "server": format!("{}#{}", row.server.name, row.server.instance),
            "state": format!("{:?}", row.state),
            "features": debugged(row.features),
            "progress": row.progress,
        }));
    }
    return json_value!({
        "language": found.language,
        "document": format!("{:?}", found.document),
        "servers": servers,
    });
}

/// Render diagnostics with the text each one marks.
pub fn diagnostics(found: &DiagnosticsSnapshot, text: &helix_core::Rope) -> Value {
    let mut items = Vec::new();
    for group in &found.groups {
        for item in &group.items {
            // `min` keeps the quoted range inside the text even if the record is read late.
            let end = item.end.min(text.len_chars());
            let start = item.start.min(end);
            items.push(json_value!({
                "source": group.source,
                "severity": debugged(item.severity),
                "code": item.code,
                "freshness": format!("{:?}", item.freshness),
                "server": format!("{}#{}", item.server.name, item.server.instance),
                "range": [item.start, item.end],
                "marked": shown(&text.slice(start..end).to_string()),
                "message": shown(&item.message),
            }));
        }
    }
    return json_value!({ "revision": found.stamp.revision, "items": items });
}

/// Render hints with the character each one precedes.
pub fn hints(found: &HintsSnapshot, text: &helix_core::Rope) -> Value {
    let mut items = Vec::new();
    for hint in &found.hints {
        let position = hint.position.min(text.len_chars());
        let line = text.char_to_line(position);
        items.push(json_value!({
            "position": hint.position,
            "line": line,
            "label": hint.label,
            "kind": debugged(hint.kind),
            "paddingLeft": hint.padding_left,
            "paddingRight": hint.padding_right,
            // The text from the line start to the hint shows where it is drawn.
            "before": shown(&text.slice(text.line_to_char(line)..position).to_string()),
        }));
    }
    return json_value!({
        "revision": found.stamp.revision,
        "lines": [found.first_line, found.last_line],
        "items": items,
    });
}
