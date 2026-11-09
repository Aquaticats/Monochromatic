//! What:
//!  Reads one line of the protocol as a client message or a service message,
//! checking every field.
//! Why:
//!  The service refuses a damaged or hostile line rather than guessing,
//! and the client checks what it is told as strictly;
//!  the types and name checks come from `service_protocol_messages_and_name_checks.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reply = decodeResponse(await readLine(socket));
//! ```

use serde_json::Value;
use crate::path_bytes_in_records as path_bytes;
use crate::service_review_key::ReviewKey;
use std::path::PathBuf;
use crate::service_protocol_messages_and_name_checks::{PROTOCOL_VERSION, MAX_LINE_BYTES, MAX_CHANGES, MAX_ID_BYTES, ALLOWED_MODES, Change, Cleanup, CommitRequest, LandedCapture, LandingFigures, LandingSync, is_zero_object, FailureCode, ClientMessage, ServiceMessage, field, is_object_name, is_identity_byte, is_plain_path, is_scratch_path};

/// What:
///  Reads and checks one client line.
/// Why:
///   The service trusts nothing from a socket:
/// sizes,
/// names,
/// paths,
/// modes and the object names are all checked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// decodeClient(line) // throws on anything that is not a well-formed message
/// ```
pub fn decode_client(line: &str) -> Result<ClientMessage, String> {
    if line.len() > MAX_LINE_BYTES {
        return Err(String::from("line is too long"));
    }
    let value: Value = serde_json::from_str(line).map_err(error_text)?;
    // A request to retire carries no data a version could change the meaning of, and it is the one message a client of
    //  another protocol version must still be able to send, so that service can be replaced.
    if field(&value, "type")? == "retire" {
        return Ok(ClientMessage::Retire);
    }
    if value.get("version").and_then(Value::as_u64) != Some(u64::from(PROTOCOL_VERSION)) {
        return Err(String::from("unsupported version"));
    }
    match field(&value, "type")? {
        "retire" => return Ok(ClientMessage::Retire),
        "withdraw" => {
            let id: &str = field(&value, "id")?;
            if id.is_empty() || id.len() > MAX_ID_BYTES {
                return Err(String::from("withdraw request is not acceptable"));
            }
            return Ok(ClientMessage::Withdraw { id: id.to_string() });
        }
        "scratch" => {
            let git_dir: PathBuf = path_bytes::read_field(&value, "gitDir")?;
            let path: PathBuf = path_bytes::read_field(&value, "path")?;
            if !is_scratch_path(path.as_path(), git_dir.as_path()) {
                return Err(String::from("scratch directory is not acceptable"));
            }
            return Ok(ClientMessage::Scratch { git_dir, path });
        }
        "warm" => {
            let git_dir: PathBuf = path_bytes::read_field(&value, "gitDir")?;
            let repository: PathBuf = path_bytes::read_field(&value, "repository")?;
            for path in [&git_dir, &repository] {
                if !path.is_absolute() || path.to_string_lossy().contains('\0') {
                    return Err(String::from("warm request is not acceptable"));
                }
            }
            return Ok(ClientMessage::Warm { git_dir, repository });
        }
        "work" => {
            let git_dir: PathBuf = path_bytes::read_field(&value, "gitDir")?;
            if !git_dir.is_absolute() || git_dir.to_string_lossy().contains('\0') {
                return Err(String::from("work request is not acceptable"));
            }
            let review_key: Option<ReviewKey> = match value.get("reviewKey") {
                None | Some(Value::Null) => None,
                Some(Value::String(text)) => ReviewKey::new(text.as_str()),
                Some(_) => return Err(String::from("work request is not acceptable")),
            };
            return Ok(ClientMessage::Work { git_dir, review_key });
        }
        "commit" => {}
        other => return Err(format!("unknown message type {other}")),
    }
    let mut changes: Vec<Change> = Vec::new();
    for entry in value.get("changes").and_then(Value::as_array).ok_or("changes is missing")? {
        let path: PathBuf = path_bytes::read_field(entry, "path")?;
        let mode: u32 = match u32::try_from(entry.get("mode").and_then(Value::as_u64).ok_or("mode is missing")?) {
            Ok(found) => found,
            Err(_out_of_range) => return Err(String::from("mode is out of range")),
        };
        let blob: Option<String> = entry.get("blob").and_then(Value::as_str).map(str::to_string);
        if !is_plain_path(path.as_path()) || !ALLOWED_MODES.contains(&mode) || matches!(blob.as_deref(), Some(name) if !is_object_name(name)) {
            return Err(format!("change for {:?} is not acceptable", crate::path_bytes::path_message_text(crate::path_bytes::path_bytes_of(path.as_path()))));
        }
        changes.push(Change { path, mode, blob });
    }
    if changes.len() > MAX_CHANGES {
        return Err(String::from("number of changes is not acceptable"));
    }
    let request = CommitRequest {
        id: field(&value, "id")?.to_string(),
        repository: path_bytes::read_field(&value, "repository")?,
        common_dir: path_bytes::read_field(&value, "commonDir")?,
        git_dir: path_bytes::read_field(&value, "gitDir")?,
        reference: field(&value, "reference")?.to_string(),
        base: field(&value, "base")?.to_string(),
        changes,
        message: field(&value, "message")?.to_string(),
        author: value.get("author").and_then(Value::as_str).map(str::to_string),
        committer: field(&value, "committer")?.to_string(),
        amend: value.get("amend").and_then(Value::as_bool).unwrap_or(false),
        tree: value.get("tree").and_then(Value::as_str).map(str::to_string),
        commit: value.get("commit").and_then(Value::as_str).map(str::to_string),
        index: path_bytes::read_field(&value, "index")?,
        captured: value.get("captured").and_then(Value::as_u64).unwrap_or(0),
        // A `cleanup` that is absent or null asks for none.
        cleanup: match value.get("cleanup") {
            Some(entry) if !entry.is_null() => Some(Cleanup {
                git_dir: path_bytes::read_field(entry, "gitDir")?,
                scratch: path_bytes::read_field(entry, "scratch")?,
                reftable: entry.get("reftable").and_then(Value::as_bool).unwrap_or(false),
            }),
            _ => None,
        },
    };
    // An author, when given, has no NUL or line break in it; no author is fine.
    let author_clean: bool = match request.author.as_deref() {
        Some(author) => !author.contains(['\0', '\n']),
        None => true,
    };
    // A cleanup, when asked for, belongs to a commit built elsewhere and names absolute directories without NUL; none is fine.
    let cleanup_clean: bool = match request.cleanup.as_ref() {
        Some(cleanup) => {
            request.commit.is_some()
                && cleanup.git_dir.is_absolute()
                && cleanup.scratch.is_absolute()
                && cleanup.scratch.starts_with(&cleanup.git_dir)
                && !cleanup.git_dir.to_string_lossy().contains('\0')
                && !cleanup.scratch.to_string_lossy().contains('\0')
        }
        None => true,
    };
    let well_formed: bool = !request.id.is_empty()
        && request.id.as_bytes().iter().all(is_identity_byte)
        // A detached head is moved as `HEAD` itself.
        && (request.reference.starts_with("refs/") || request.reference == "HEAD")
        && !request.reference.contains(['\0', '\n', ' '])
        && is_object_name(&request.base)
        // A branch that does not exist yet has no commit to build the tree from, so only a commit built elsewhere names it.
        && (!is_zero_object(&request.base) || request.commit.is_some())
        && request.repository.is_absolute()
        && request.common_dir.is_absolute()
        && request.git_dir.is_absolute()
        && !request.git_dir.to_string_lossy().contains('\0')
        && request.index.is_absolute()
        && !request.index.to_string_lossy().contains('\0')
        && !request.committer.contains(['\0', '\n'])
        && author_clean
        // An amend the service builds names the tree it makes; a commit built elsewhere is named whole; any other request
        //  names neither, so the three cannot be mixed up.
        && (request.amend && request.commit.is_none()) == request.tree.is_some()
        && request.tree.as_deref().is_none_or(is_object_name)
        && request.commit.as_deref().is_none_or(is_object_name)
        && cleanup_clean;
    if !well_formed {
        return Err(String::from("request is not acceptable"));
    }
    return Ok(ClientMessage::Commit(Box::new(request)));
}

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
fn number_field(value: &Value, name: &str) -> Result<usize, String> {
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
        captures.push(LandedCapture { commit: commit.to_string(), captured });
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
fn error_text(error: serde_json::Error) -> String {
    return error.to_string();
}

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
    for each in value.get("syncs").and_then(Value::as_array).map(Vec::as_slice).unwrap_or_default() {
        syncs.push(LandingSync {
            what: field(each, "what")?.to_string(),
            waited: each.get("waited").and_then(Value::as_bool).ok_or_else(no_word_on_waiting)?,
            ms: number_field_value(each.get("ms").unwrap_or(&Value::Null), "a sync of a landing")?,
        });
    }
    return Ok(LandingFigures {
        lock_hold_ms: number_field_value(value.get("lockHoldMs").unwrap_or(&Value::Null), "the lock hold of a landing")?,
        queued_behind: number_field_value(value.get("queuedBehind").unwrap_or(&Value::Null), "the commits queued behind a landing")?,
        reads_ms: number_field_value(value.get("readsMs").unwrap_or(&Value::Null), "the reads of a landing")?,
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
fn number_field_value(value: &Value, what: &str) -> Result<u64, String> {
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
            release: value.get("release").and_then(Value::as_str).map(String::from),
        }),
        "position" => Ok(ServiceMessage::Position { number: number_field(&value, "number")?, of: number_field(&value, "of")? }),
        "landed" => Ok(ServiceMessage::Landed {
            id: field(&value, "id")?.to_string(),
            commit: field(&value, "commit")?.to_string(),
            already: value.get("already").and_then(Value::as_bool).unwrap_or(false),
            // A service of a release that does not send the figures leaves them absent, which the record states as none.
            landing: match value.get("landing") {
                None | Some(Value::Null) => None,
                Some(landing) => Some(landing_figures(landing)?),
            },
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
        "waiting" if value.get("inside_write").and_then(Value::as_bool) == Some(true) => Ok(ServiceMessage::InsideWrite { text: field(&value, "text")?.to_string() }),
        "waiting" => Ok(ServiceMessage::Waiting { text: field(&value, "text")?.to_string() }),
        "holder" => Ok(ServiceMessage::Holder { who: field(&value, "who")?.to_string() }),
        "heartbeat" => Ok(ServiceMessage::Heartbeat),
        "withdrawn" => Ok(ServiceMessage::Withdrawn { id: field(&value, "id")?.to_string() }),
        "failed" => Ok(ServiceMessage::Failed {
            id: field(&value, "id")?.to_string(),
            code: FailureCode::from_word(field(&value, "code")?).ok_or("unknown failure code")?,
        }),
        other => Err(format!("unknown message type {other}")),
    };
}
