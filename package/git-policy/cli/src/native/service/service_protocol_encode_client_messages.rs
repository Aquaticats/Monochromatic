//! What:
//!  The lines the wrapper writes to the service:
//!  a retire at a named protocol version,
//!  and every message of a client on one line.
//! Why:
//!   A service reads a line only in its own version,
//!  except that a retire is read at any version,
//!  so the line must carry the version the service announced;
//!  every other message carries the version of this wrapper
//!  and a commit is written by the one function that also measures it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! socket.write(JSON.stringify(encodeRequest(request)) + '\n');
//! ```

use crate::path_bytes_in_records as path_bytes;
use crate::service_protocol_encode_the_commit_line::commit_line;
use crate::service_protocol_encode_the_fields_of_a_message::{
    object,
    programs_json,
    put,
};
use crate::service_protocol_messages_and_name_checks::{
    ClientMessage,
    PROTOCOL_VERSION,
};
use serde_json::{
    Map,
    Value,
};
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
    return object(vec![
        ("type", Value::from("retire")),
        ("version", Value::from(version)),
    ]);
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
        ClientMessage::Withdraw {
            id,
        } => object(vec![
            ("type", Value::from("withdraw")),
            ("version", Value::from(PROTOCOL_VERSION)),
            ("id", Value::from(id.as_str())),
        ]),
        ClientMessage::Shadow {
            git_dir,
            scratch,
        } => {
            let mut fields: Map<String, Value> = Map::new();
            put(&mut fields, "type", Value::from("shadow"));
            put(&mut fields, "version", Value::from(PROTOCOL_VERSION));
            path_bytes::write_field(&mut fields, "gitDir", git_dir.as_path());
            path_bytes::write_field(&mut fields, "scratch", scratch.as_path());
            Value::Object(fields).to_string()
        }
        ClientMessage::Warm {
            git_dir,
            repository,
        } => {
            let mut fields: Map<String, Value> = Map::new();
            put(&mut fields, "type", Value::from("warm"));
            put(&mut fields, "version", Value::from(PROTOCOL_VERSION));
            path_bytes::write_field(&mut fields, "gitDir", git_dir.as_path());
            path_bytes::write_field(&mut fields, "repository", repository.as_path());
            Value::Object(fields).to_string()
        }
        ClientMessage::Work {
            git_dir,
            review_key,
            push_programs,
        } => {
            let mut fields: Map<String, Value> = Map::new();
            put(&mut fields, "type", Value::from("work"));
            put(&mut fields, "version", Value::from(PROTOCOL_VERSION));
            path_bytes::write_field(&mut fields, "gitDir", git_dir.as_path());
            if let Some(key) = review_key {
                put(&mut fields, "reviewKey", Value::from(key.expose()));
            }
            if !push_programs.is_empty() {
                let entries: Vec<Value> = push_programs
                    .iter()
                    .map(|(id, programs)| {
                        let mut entry: Map<String, Value> = Map::new();
                        put(&mut entry, "id", Value::from(id.as_str()));
                        put(&mut entry, "programs", programs_json(Some(programs)));
                        return Value::Object(entry);
                    })
                    .collect();
                put(&mut fields, "pushPrograms", Value::Array(entries));
            }
            Value::Object(fields).to_string()
        }
        ClientMessage::ForgetPrograms {
            git_dir,
            records,
        } => {
            let mut fields: Map<String, Value> = Map::new();
            put(&mut fields, "type", Value::from("forgetPrograms"));
            put(&mut fields, "version", Value::from(PROTOCOL_VERSION));
            path_bytes::write_field(&mut fields, "gitDir", git_dir.as_path());
            put(
                &mut fields,
                "records",
                Value::Array(
                    records
                        .iter()
                        .map(|id| {
                            return Value::from(id.as_str());
                        })
                        .collect(),
                ),
            );
            Value::Object(fields).to_string()
        }
        ClientMessage::Commit(request) => commit_line(request.as_ref()),
    };
}
