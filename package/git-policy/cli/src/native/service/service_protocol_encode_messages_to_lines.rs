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
use crate::path_bytes_in_records as path_bytes;
use std::path::Path;
use crate::service_protocol_messages_and_name_checks::{PROTOCOL_VERSION, MAX_LINE_BYTES, Change, Cleanup, CommitRequest, LandedCapture, ClientMessage, LandingFigures, LandingSync, ServiceMessage};

/// What:
///  One field of a message,
/// by its name.
/// Why:
///  A message that carries paths is built here rather than from a list of pairs,
/// because a path takes two fields (see `path_bytes::write_field`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const put = (fields, key, value) => { fields[key] = value; };
/// ```
fn put(fields: &mut Map<String, Value>, key: &str, value: Value) {
    fields.insert(String::from(key), value);
}

/// What:
///  One changed path of a commit request as a JSON object:
/// its path,
/// mode and blob (null for a deletion).
/// Why:
///  `encode_client` maps every change of a request through it,
/// and the path takes two fields (see `path_bytes::write_field`),
/// so a file name that is not text reaches the service as the name Git listed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const changeJson = (change: Change) => ({ path: shown, pathHex: hex(change.path), mode: change.mode, blob: change.blob ?? null });
/// ```
fn change_json(change: &Change) -> Value {
    let mut fields: Map<String, Value> = Map::new();
    let path: &Path = change.path.as_path();
    // A name that is text is exactly the plain field, so the bytes are written only when the plain field cannot spell them:
    //  a request of `MAX_CHANGES` changes stays inside `MAX_LINE_BYTES` at the path length that bound was measured for.
    put(&mut fields, "path", Value::from(path.display().to_string()));
    if path.to_str().is_none() {
        put(&mut fields, "pathHex", Value::from(path_bytes::hex_of_path(path)));
    }
    put(&mut fields, "mode", Value::from(change.mode));
    put(&mut fields, "blob", change.blob.as_deref().map_or(Value::Null, Value::from));
    return Value::Object(fields);
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
    let mut fields: Map<String, Value> = Map::new();
    path_bytes::write_field(&mut fields, "gitDir", cleanup.git_dir.as_path());
    path_bytes::write_field(&mut fields, "scratch", cleanup.scratch.as_path());
    put(&mut fields, "reftable", Value::from(cleanup.reftable));
    return Value::Object(fields);
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

/// What:
///  What a landing spent and held,
/// as the `landing` field of an answer.
/// Why:
///   The client's record states these figures beside its own phases,
/// so a slow commit explains its landing without a second file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const landingJson = (f) => ({ lockHoldMs: f.lockHoldMs, queuedBehind: f.queuedBehind, readsMs: f.readsMs, reads: Object.fromEntries(f.reads), syncs: f.syncs });
/// ```
/// One sync of a landing as a JSON object.
fn one_sync_json(sync: &LandingSync) -> Value {
    return Value::Object(Map::from_iter([
        (String::from("what"), Value::from(sync.what.as_str())),
        (String::from("waited"), Value::from(sync.waited)),
        (String::from("ms"), Value::from(sync.ms)),
    ]));
}

fn landing_figures_json(figures: &LandingFigures) -> Value {
    let mut reads: Map<String, Value> = Map::new();
    for (name, ms) in &figures.reads {
        reads.insert(name.clone(), Value::from(*ms));
    }
    let syncs: Vec<Value> = figures.syncs.iter().map(one_sync_json).collect();
    return Value::Object(Map::from_iter([
        (String::from("lockHoldMs"), Value::from(figures.lock_hold_ms)),
        (String::from("queuedBehind"), Value::from(figures.queued_behind)),
        (String::from("readsMs"), Value::from(figures.reads_ms)),
        (String::from("reads"), Value::Object(reads)),
        (String::from("syncs"), Value::Array(syncs)),
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
        ClientMessage::Warm { git_dir, repository } => {
            let mut fields: Map<String, Value> = Map::new();
            put(&mut fields, "type", Value::from("warm"));
            put(&mut fields, "version", Value::from(PROTOCOL_VERSION));
            path_bytes::write_field(&mut fields, "gitDir", git_dir.as_path());
            path_bytes::write_field(&mut fields, "repository", repository.as_path());
            Value::Object(fields).to_string()
        }
        ClientMessage::Work { git_dir, review_key } => {
            let mut fields: Map<String, Value> = Map::new();
            put(&mut fields, "type", Value::from("work"));
            put(&mut fields, "version", Value::from(PROTOCOL_VERSION));
            path_bytes::write_field(&mut fields, "gitDir", git_dir.as_path());
            if let Some(key) = review_key {
                put(&mut fields, "reviewKey", Value::from(key.expose()));
            }
            Value::Object(fields).to_string()
        }
        ClientMessage::Commit(request) => commit_line(request.as_ref()),
    };
}

/// What:
///  The line for one commit request.
/// Why:
///   `encode_client` writes it,
/// and the preparation asks the same function whether the request fits the ceiling both sides accept,
/// so the line the service is given and the line that was measured cannot differ.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commitLine = (request: CommitRequest) => JSON.stringify({ type: 'commit', version: PROTOCOL_VERSION, ...request });
/// ```
pub fn commit_line(request: &CommitRequest) -> String {
    let mut fields: Map<String, Value> = Map::new();
    put(&mut fields, "type", Value::from("commit"));
    put(&mut fields, "version", Value::from(PROTOCOL_VERSION));
    put(&mut fields, "id", Value::from(request.id.as_str()));
    path_bytes::write_field(&mut fields, "repository", request.repository.as_path());
    path_bytes::write_field(&mut fields, "commonDir", request.common_dir.as_path());
    path_bytes::write_field(&mut fields, "gitDir", request.git_dir.as_path());
    path_bytes::write_field(&mut fields, "index", request.index.as_path());
    path_bytes::write_field(&mut fields, "scratch", request.scratch.as_path());
    put(&mut fields, "reference", Value::from(request.reference.as_str()));
    put(&mut fields, "base", Value::from(request.base.as_str()));
    put(&mut fields, "changes", Value::Array(request.changes.iter().map(change_json).collect()));
    put(&mut fields, "message", Value::from(request.message.as_str()));
    put(&mut fields, "author", request.author.as_deref().map_or(Value::Null, Value::from));
    put(&mut fields, "committer", Value::from(request.committer.as_str()));
    put(&mut fields, "amend", Value::from(request.amend));
    put(&mut fields, "tree", request.tree.as_deref().map_or(Value::Null, Value::from));
    put(&mut fields, "commit", request.commit.as_deref().map_or(Value::Null, Value::from));
    put(&mut fields, "captured", Value::from(request.captured));
    put(&mut fields, "holder", Value::from(request.holder.as_str()));
    put(&mut fields, "cleanup", request.cleanup.as_ref().map_or(Value::Null, cleanup_json));
    return Value::Object(fields).to_string();
}

/// What:
///  Whether the line that carries `request` is one the service accepts.
/// Why:
///   The reader refuses a line over `MAX_LINE_BYTES` (`decode_client`),
/// and the client writes the line and one line break,
/// so a request at the ceiling is one whose line and break fit together.
/// A request that does not fit is a commit the wrapper builds itself instead (`service_commit.rs`):
/// "Commits should never fail."
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commitRequestFits = (request: CommitRequest) => Buffer.byteLength(commitLine(request)) + 1 <= MAX_LINE_BYTES;
/// ```
pub fn commit_request_fits(request: &CommitRequest) -> bool {
    // The reader counts the line break it reads as part of the line, so the break is part of the budget here too.
    return commit_line(request).len() + 1 <= MAX_LINE_BYTES;
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
        ServiceMessage::Landed { id, commit, already, landing } => {
            let mut entries: Vec<(&str, Value)> =
                vec![("type", Value::from("landed")), ("id", Value::from(id.as_str())), ("commit", Value::from(commit.as_str())), ("already", Value::from(*already))];
            if let Some(figures) = landing {
                entries.push(("landing", landing_figures_json(figures)));
            }
            object(entries)
        }
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
