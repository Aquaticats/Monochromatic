//! What:
//!  The fields a protocol message is written from:
//!  one field by its name,
//!  one changed path,
//!  one cleanup,
//!  one landed capture and one sync of a landing,
//!  the figures of a landing,
//!  the programs a command carried,
//!  and the one object each message is written as.
//! Why:
//!  Every message kind of both directions is written from these,
//!  and a path takes two fields (see `path_bytes::write_field`),
//!  so a file name that is not text reaches the service as the name Git listed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const object = (entries) => JSON.stringify(Object.fromEntries(entries));
//! ```

use crate::path_bytes_in_records as path_bytes;
use crate::service_protocol_messages_and_name_checks::{
    Change,
    Cleanup,
    LandedCapture,
    LandingFigures,
    LandingSync,
};
use serde_json::{
    Map,
    Value,
};
use std::path::Path;
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
pub(crate) fn put(
    fields: &mut Map<String, Value>,
    key: &str,
    value: Value,
) {
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
pub(crate) fn change_json(change: &Change) -> Value {
    let mut fields: Map<String, Value> = Map::new();
    let path: &Path = change.path.as_path();
    // A name that is text is exactly the plain field, so the bytes are written only when the plain field cannot spell them:
    //  a request of `MAX_CHANGES` changes stays inside `MAX_LINE_BYTES` at the path length that bound was measured for.
    put(&mut fields, "path", Value::from(path.display().to_string()));
    if path.to_str().is_none() {
        put(
            &mut fields,
            "pathHex",
            Value::from(path_bytes::hex_of_path(path)),
        );
    }
    put(&mut fields, "mode", Value::from(change.mode));
    put(
        &mut fields,
        "blob",
        change.blob.as_deref().map_or(Value::Null, Value::from),
    );
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
pub(crate) fn cleanup_json(cleanup: &Cleanup) -> Value {
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
pub(crate) fn landed_json(entry: &LandedCapture) -> Value {
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
pub(crate) fn one_sync_json(sync: &LandingSync) -> Value {
    return Value::Object(Map::from_iter([
        (String::from("what"), Value::from(sync.what.as_str())),
        (String::from("waited"), Value::from(sync.waited)),
        (String::from("ms"), Value::from(sync.ms)),
    ]));
}

pub(crate) fn landing_figures_json(figures: &LandingFigures) -> Value {
    let mut reads: Map<String, Value> = Map::new();
    for (name, ms) in &figures.reads {
        reads.insert(name.clone(), Value::from(*ms));
    }
    let syncs: Vec<Value> = figures.syncs.iter().map(one_sync_json).collect();
    return Value::Object(Map::from_iter([
        (
            String::from("lockHoldMs"),
            Value::from(figures.lock_hold_ms),
        ),
        (
            String::from("queuedBehind"),
            Value::from(figures.queued_behind),
        ),
        (String::from("readsMs"), Value::from(figures.reads_ms)),
        (String::from("reads"), Value::Object(reads)),
        (String::from("syncs"), Value::Array(syncs)),
    ]));
}

/// One JSON object,
/// written on one line from its entries.
pub(crate) fn object(entries: Vec<(&str, Value)>) -> String {
    let mut map: Map<String, Value> = Map::new();
    for (key, value) in entries {
        map.insert(key.to_string(), value);
    }
    return Value::Object(map).to_string();
}
/// What:
///  The programs and transport settings of a request as one JSON object,
///  or null when it carries none.
/// Why:
///   The names travel as pairs rather than as an object's keys,
/// so their order is the order the push's overlay adds them in,
/// and a name cannot be lost to a
///  duplicate key;
/// `transport` is the flag alone,
/// never a value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const programsJson = (programs) => programs === null ? null : ({ transport: programs.transport, variables: programs.variables, notText: programs.notText });
/// ```
pub(crate) fn programs_json(
    programs: Option<&crate::service_push_programs_held_in_memory::PushPrograms>
) -> Value {
    let Some(carried) = programs else {
        return Value::Null;
    };
    let variables: Value = Value::Array(
        carried
            .variables
            .iter()
            .map(|(name, value)| {
                return Value::Array(vec![
                    Value::from(name.as_str()),
                    Value::from(value.as_str()),
                ]);
            })
            .collect(),
    );
    // The names whose value this release cannot write in a line, never the values themselves: the service keeps the
    //  manager's own value for such a name, and a value no line can hold cannot be written here either.
    let not_text: Value = Value::Array(
        carried
            .not_text
            .iter()
            .map(|name| {
                return Value::from(name.as_str());
            })
            .collect(),
    );
    return Value::Object(Map::from_iter([
        (String::from("transport"), Value::from(carried.transport)),
        (String::from("variables"), variables),
        (String::from("not_text"), not_text),
    ]));
}
