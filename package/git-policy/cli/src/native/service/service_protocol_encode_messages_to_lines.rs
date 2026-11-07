//! What:
//!  Writes a client message or a service message as one JSON object on one line.
//! Why:
//!  The line form needs no framing library;
//! the types come from `service_protocol_messages_and_name_checks.rs`
//!  and the decoder that reads these lines is in `service_protocol_decode_lines_to_messages.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! socket.write(JSON.stringify(encodeRequest(request)) + '\n');
//! ```

use serde_json::{Map, Value};
use crate::service_protocol_messages_and_name_checks::{PROTOCOL_VERSION, Change, Cleanup, LandedCapture, ClientMessage, ServiceMessage};

/// What:
///  One changed path of a commit request as a JSON object:
/// its path,
/// mode and blob (null for a deletion).
/// Why:
///  `encode_client` maps every change of a request through it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const changeJson = (change: Change) => ({ path: change.path, mode: change.mode, blob: change.blob ?? null });
/// ```
fn change_json(change: &Change) -> Value {
    return Value::Object(Map::from_iter([
        (String::from("path"), Value::from(change.path.as_str())),
        (String::from("mode"), Value::from(change.mode)),
        (String::from("blob"), change.blob.as_deref().map_or(Value::Null, Value::from)),
    ]));
}

/// What:
///  The cleanup a request asks for as a JSON object:
/// the Git directory,
/// the scratch directory and whether the store is a reftable.
/// Why:
///  `encode_client` writes it under the `cleanup` key,
/// and null when the request asks for none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cleanupJson = (cleanup: Cleanup) => ({ gitDir: cleanup.gitDir, scratch: cleanup.scratch, reftable: cleanup.reftable });
/// ```
fn cleanup_json(cleanup: &Cleanup) -> Value {
    return Value::Object(Map::from_iter([
        (String::from("gitDir"), Value::from(cleanup.git_dir.to_string_lossy().as_ref())),
        (String::from("scratch"), Value::from(cleanup.scratch.to_string_lossy().as_ref())),
        (String::from("reftable"), Value::from(cleanup.reftable)),
    ]));
}

/// What:
///  One landed capture as a JSON object:
/// the commit and the capture number.
/// Why:
///  `encode_service` writes the list of captures a retry must prepare against.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const landedJson = (entry: LandedCapture) => ({ commit: entry.commit, captured: entry.captured });
/// ```
fn landed_json(entry: &LandedCapture) -> Value {
    return Value::Object(Map::from_iter([
        (String::from("commit"), Value::from(entry.commit.as_str())),
        (String::from("captured"), Value::from(entry.captured)),
    ]));
}

/// One JSON object,
/// written on one line from its entries.
fn object(entries: Vec<(&str, Value)>) -> String {
    let mut map: Map<String, Value> = Map::new();
    for (key, value) in entries {
        map.insert(key.to_string(), value);
    }
    return Value::Object(map).to_string();
}

/// What:
///  The line that asks a service to retire,
/// written in the protocol version that service speaks.
/// Why:
///   A service reads a line only in its own version,
/// except that services built with this check read a retire at any
///  version.
/// A service of an earlier release (the ones that sent no release in their greeting) refuses a retire written in
///  a newer version as unsupported and stays,
/// so the line must carry the version the service announced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify({ type: 'retire', version: announcedVersion })
/// ```
pub fn encode_retire(version: u32) -> String {
    return object(vec![("type", Value::from("retire")), ("version", Value::from(version))]);
}

/// What:
///  The line for a client message.
/// Why:
///   One object per line,
/// no newline inside any field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify(message)
/// ```
pub fn encode_client(message: &ClientMessage) -> String {
    return match message {
        ClientMessage::Retire => encode_retire(PROTOCOL_VERSION),
        ClientMessage::Withdraw { id } => object(vec![
            ("type", Value::from("withdraw")),
            ("version", Value::from(PROTOCOL_VERSION)),
            ("id", Value::from(id.as_str())),
        ]),
        ClientMessage::Scratch { git_dir, path } => object(vec![
            ("type", Value::from("scratch")),
            ("version", Value::from(PROTOCOL_VERSION)),
            ("gitDir", Value::from(git_dir.to_string_lossy().as_ref())),
            ("path", Value::from(path.to_string_lossy().as_ref())),
        ]),
        ClientMessage::Work { git_dir, review_key } => {
            let mut entries: Vec<(&str, Value)> = vec![
                ("type", Value::from("work")),
                ("version", Value::from(PROTOCOL_VERSION)),
                ("gitDir", Value::from(git_dir.to_string_lossy().as_ref())),
            ];
            if let Some(key) = review_key {
                entries.push(("reviewKey", Value::from(key.expose())));
            }
            object(entries)
        }
        ClientMessage::Commit(request) => object(vec![
            ("type", Value::from("commit")),
            ("version", Value::from(PROTOCOL_VERSION)),
            ("id", Value::from(request.id.as_str())),
            ("repository", Value::from(request.repository.to_string_lossy().as_ref())),
            ("commonDir", Value::from(request.common_dir.to_string_lossy().as_ref())),
            ("gitDir", Value::from(request.git_dir.to_string_lossy().as_ref())),
            ("reference", Value::from(request.reference.as_str())),
            ("base", Value::from(request.base.as_str())),
            (
                "changes",
                Value::Array(
                    request
                        .changes
                        .iter()
                        .map(change_json)
                        .collect(),
                ),
            ),
            ("message", Value::from(request.message.as_str())),
            ("author", request.author.as_deref().map_or(Value::Null, Value::from)),
            ("committer", Value::from(request.committer.as_str())),
            ("amend", Value::from(request.amend)),
            ("tree", request.tree.as_deref().map_or(Value::Null, Value::from)),
            ("commit", request.commit.as_deref().map_or(Value::Null, Value::from)),
            ("index", Value::from(request.index.to_string_lossy().as_ref())),
            ("captured", Value::from(request.captured)),
            (
                "cleanup",
                request.cleanup.as_ref().map_or(Value::Null, cleanup_json),
            ),
        ]),
    };
}

/// What:
///  The line for a service message.
/// Why:
///   The same one-object form in the other direction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify(message)
/// ```
pub fn encode_service(message: &ServiceMessage) -> String {
    return match message {
        ServiceMessage::Welcome { version, release } => {
            let mut entries: Vec<(&str, Value)> = vec![("type", Value::from("welcome")), ("version", Value::from(*version))];
            if let Some(text) = release {
                entries.push(("release", Value::from(text.as_str())));
            }
            object(entries)
        }
        ServiceMessage::Position { number, of } => {
            object(vec![("type", Value::from("position")), ("number", Value::from(*number)), ("of", Value::from(*of))])
        }
        ServiceMessage::Landed { id, commit, already } => object(vec![
            ("type", Value::from("landed")),
            ("id", Value::from(id.as_str())),
            ("commit", Value::from(commit.as_str())),
            ("already", Value::from(*already)),
        ]),
        ServiceMessage::RetryPrepare { id, base, landed } => object(vec![
            ("type", Value::from("retry-prepare")),
            ("id", Value::from(id.as_str())),
            ("base", Value::from(base.as_str())),
            (
                "landed",
                Value::Array(
                    landed
                        .iter()
                        .map(landed_json)
                        .collect(),
                ),
            ),
        ]),
        ServiceMessage::Waiting { text } => object(vec![("type", Value::from("waiting")), ("text", Value::from(text.as_str()))]),
        ServiceMessage::InsideWrite { text } => object(vec![("type", Value::from("waiting")), ("text", Value::from(text.as_str())), ("inside_write", Value::from(true))]),
        ServiceMessage::Holder { who } => object(vec![("type", Value::from("holder")), ("who", Value::from(who.as_str()))]),
        ServiceMessage::Heartbeat => object(vec![("type", Value::from("heartbeat"))]),
        ServiceMessage::Withdrawn { id } => object(vec![("type", Value::from("withdrawn")), ("id", Value::from(id.as_str()))]),
        ServiceMessage::Failed { id, code } => {
            object(vec![("type", Value::from("failed")), ("id", Value::from(id.as_str())), ("code", Value::from(code.word()))])
        }
    };
}
