//! What:
//!  Reads one line of the protocol as a service message,
//!  checking every field:
//!  the version and the release a service greets with,
//!  the place of a batch,
//!  the figures of a landing and the captures of a `landed` list,
//!  and every answer a commit can end with.
//! Why:
//!   The client checks what it is told as strictly as the service checks what it is sent,
//!  so nothing a service sends can name a path,
//!  a number or a message kind this wrapper would not have written.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reply = decodeResponse(await readLine(socket));
//! ```

use crate::path_bytes_in_records as path_bytes;
use crate::service_protocol_decode_the_words_of_a_refused_line::error_text;
use crate::service_protocol_messages_and_name_checks::{
    FailureCode,
    LandedCapture,
    LandingFigures,
    LandingSync,
    MAX_LINE_BYTES,
    ServiceMessage,
    field,
    is_object_name,
};
use serde_json::Value;
/// What:
///  A numeric field of a service message as a count:
/// the field must be there and fit.
/// Why:
///  `decode_service` reads positions and versions with it;
/// a value too large is as unusable as a missing one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const numberField = (value, name) => { const n = value[name]; if (!Number.isSafeInteger(n)) throw new Error(`field ${name} is missing`); return n; };
/// ```
fn number_field(
    value: &Value,
    name: &str,
) -> Result<usize, String> {
    let Some(wide) = value.get(name).and_then(Value::as_u64) else {
        return Err(format!("field {name} is missing"));
    };
    return match usize::try_from(wide) {
        Ok(count) => Ok(count),
        Err(_too_large) => Err(format!("field {name} is missing")),
    };
}

/// What:
///  The captures of a `landed` list:
/// the entries that name an object and carry a capture number;
/// others are left out.
/// Why:
///  `decode_service` reads the list of a `retry-prepare` message with it,
/// and an entry it cannot use is skipped,
/// not an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entries.flatMap((entry) => isObjectName(entry.commit) && typeof entry.captured === 'number' ? [{ commit: entry.commit, captured: entry.captured }] : [])
/// ```
fn landed_captures(entries: &Vec<Value>) -> Vec<LandedCapture> {
    let mut captures: Vec<LandedCapture> = Vec::new();
    for entry in entries {
        let Some(commit) = entry.get("commit").and_then(Value::as_str) else {
            continue;
        };
        if !is_object_name(commit) {
            continue;
        }
        let Some(captured) = entry.get("captured").and_then(Value::as_u64) else {
            continue;
        };
        captures.push(LandedCapture {
            commit: commit.to_string(),
            captured,
        });
    }
    return captures;
}

/// What:
///  A JSON parse error as its message text.
/// Why:
///  The decoders report every failure as text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const errorText = (error: Error) => error.message;
/// ```
/// What:
///  The figures of a landing,
/// as the `landing` field of an answer.
/// Why:
///   The client trusts nothing from a socket:
/// every number here is checked,
/// and a field that is not one is refused
/// rather than guessed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const landingFigures = (v) => ({ lockHoldMs: num(v.lockHoldMs), queuedBehind: num(v.queuedBehind), readsMs: num(v.readsMs), reads: entries(v.reads), syncs: v.syncs.map(sync) });
/// ```
fn landing_figures(value: &Value) -> Result<LandingFigures, String> {
    let mut reads: Vec<(String, u64)> = Vec::new();
    let Some(named_reads) = value.get("reads") else {
        return Err(String::from("the figures of a landing name no reads"));
    };
    let Some(read_fields) = named_reads.as_object() else {
        return Err(String::from("the reads of a landing are not an object"));
    };
    for (name, ms) in read_fields {
        reads.push((name.clone(), number_field_value(ms, "a read of a landing")?));
    }
    let mut syncs: Vec<LandingSync> = Vec::new();
    for each in value
        .get("syncs")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        syncs.push(LandingSync {
            what: field(each, "what")?.to_string(),
            waited: each
                .get("waited")
                .and_then(Value::as_bool)
                .ok_or_else(no_word_on_waiting)?,
            ms: number_field_value(
                each.get("ms").unwrap_or(&Value::Null),
                "a sync of a landing",
            )?,
        });
    }
    return Ok(LandingFigures {
        lock_hold_ms: number_field_value(
            value.get("lockHoldMs").unwrap_or(&Value::Null),
            "the lock hold of a landing",
        )?,
        queued_behind: number_field_value(
            value.get("queuedBehind").unwrap_or(&Value::Null),
            "the commits queued behind a landing",
        )?,
        reads_ms: number_field_value(
            value.get("readsMs").unwrap_or(&Value::Null),
            "the reads of a landing",
        )?,
        reads,
        syncs,
    });
}

/// The sentence for a sync that does not say whether the landing waited for it.
fn no_word_on_waiting() -> String {
    return String::from("a sync of a landing does not say whether it was waited for");
}

/// What:
///  The number of one JSON value,
///  checked to be an unsigned whole number.
/// Why:
///   A figure that is not one,
///  which is a negative number,
///  a fraction or text,
///  is refused,
///  so a reader never takes a guess for a measurement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const numberFieldValue = (v, what) => Number.isInteger(v) && v >= 0 ? v : fail(`${what} is not a whole number`);
/// ```
fn number_field_value(
    value: &Value,
    what: &str,
) -> Result<u64, String> {
    let Some(number) = value.as_u64() else {
        return Err(format!("{what} is not a whole number"));
    };
    return Ok(number);
}

/// What:
///  Reads one service line.
/// Why:
///   The client checks what it is told as strictly as the service checks what it is sent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// decodeService(line)
/// ```
pub fn decode_service(line: &str) -> Result<ServiceMessage, String> {
    if line.len() > MAX_LINE_BYTES {
        return Err(String::from("line is too long"));
    }
    let value: Value = serde_json::from_str(line).map_err(error_text)?;
    return match field(&value, "type")? {
        "welcome" => Ok(ServiceMessage::Welcome {
            version: match u32::try_from(number_field(&value, "version")?) {
                Ok(found) => found,
                Err(_out_of_range) => return Err(String::from("version is out of range")),
            },
            release: value
                .get("release")
                .and_then(Value::as_str)
                .map(String::from),
        }),
        "drawn" => Ok(ServiceMessage::Drawn {
            tree: path_bytes::read_field(&value, "tree")?,
        }),
        "no-shadow" => Ok(ServiceMessage::NoShadow),
        "position" => Ok(ServiceMessage::Position {
            number: number_field(&value, "number")?,
            of: number_field(&value, "of")?,
        }),
        "landed" => Ok(ServiceMessage::Landed {
            id: field(&value, "id")?.to_string(),
            commit: field(&value, "commit")?.to_string(),
            already: value
                .get("already")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            // A service of a release that does not send the figures leaves them absent, which the record states as none.
            landing: match value.get("landing") {
                None | Some(Value::Null) => None,
                Some(landing) => Some(landing_figures(landing)?),
            },
        }),
        "held" => Ok(ServiceMessage::Held {
            id: field(&value, "id")?.to_string(),
            commit: field(&value, "commit")?.to_string(),
            words: field(&value, "words")?.to_string(),
        }),
        "retry-prepare" => Ok(ServiceMessage::RetryPrepare {
            id: field(&value, "id")?.to_string(),
            base: field(&value, "base")?.to_string(),
            // A service that sends no list tells nothing about the order of captures.
            landed: value
                .get("landed")
                .and_then(Value::as_array)
                .map(landed_captures)
                .unwrap_or_default(),
        }),
        "waiting" if value.get("inside_write").and_then(Value::as_bool) == Some(true) => {
            Ok(ServiceMessage::InsideWrite {
                text: field(&value, "text")?.to_string(),
            })
        }
        "waiting" => Ok(ServiceMessage::Waiting {
            text: field(&value, "text")?.to_string(),
        }),
        "holder" => Ok(ServiceMessage::Holder {
            who: field(&value, "who")?.to_string(),
        }),
        "heartbeat" => Ok(ServiceMessage::Heartbeat),
        "withdrawn" => Ok(ServiceMessage::Withdrawn {
            id: field(&value, "id")?.to_string(),
        }),
        "failed" => Ok(ServiceMessage::Failed {
            id: field(&value, "id")?.to_string(),
            code: FailureCode::from_word(field(&value, "code")?).ok_or("unknown failure code")?,
        }),
        other => Err(format!("unknown message type {other}")),
    };
}
