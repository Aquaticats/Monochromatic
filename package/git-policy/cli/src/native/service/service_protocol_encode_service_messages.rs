//! What:
//!  The line for one service message,
//!  as the same one-object form in the other direction.
//! Why:
//!   The service's answers are written here and read by `decode_service`,
//!  and a landed answer carries the figures of its landing,
//!  which is what the client's timing record states.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! JSON.stringify(message)
//! ```

use crate::path_bytes_in_records as path_bytes;
use crate::service_protocol_encode_the_fields_of_a_message::{
    landed_json,
    landing_figures_json,
    object,
    put,
};
use crate::service_protocol_messages_and_name_checks::ServiceMessage;
use serde_json::{
    Map,
    Value,
};
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
        ServiceMessage::Welcome {
            version,
            release,
        } => {
            let mut entries: Vec<(&str, Value)> = vec![
                ("type", Value::from("welcome")),
                ("version", Value::from(*version)),
            ];
            if let Some(text) = release {
                entries.push(("release", Value::from(text.as_str())));
            }
            object(entries)
        }
        ServiceMessage::Drawn {
            tree,
        } => {
            let mut fields: Map<String, Value> = Map::new();
            put(&mut fields, "type", Value::from("drawn"));
            path_bytes::write_field(&mut fields, "tree", tree.as_path());
            Value::Object(fields).to_string()
        }
        ServiceMessage::NoShadow => object(vec![("type", Value::from("no-shadow"))]),
        ServiceMessage::Position {
            number,
            of,
        } => object(vec![
            ("type", Value::from("position")),
            ("number", Value::from(*number)),
            ("of", Value::from(*of)),
        ]),
        ServiceMessage::Landed {
            id,
            commit,
            already,
            landing,
        } => {
            let mut entries: Vec<(&str, Value)> = vec![
                ("type", Value::from("landed")),
                ("id", Value::from(id.as_str())),
                ("commit", Value::from(commit.as_str())),
                ("already", Value::from(*already)),
            ];
            if let Some(figures) = landing {
                entries.push(("landing", landing_figures_json(figures)));
            }
            object(entries)
        }
        ServiceMessage::Held {
            id,
            commit,
            words,
        } => object(vec![
            ("type", Value::from("held")),
            ("id", Value::from(id.as_str())),
            ("commit", Value::from(commit.as_str())),
            ("words", Value::from(words.as_str())),
        ]),
        ServiceMessage::RetryPrepare {
            id,
            base,
            landed,
        } => object(vec![
            ("type", Value::from("retry-prepare")),
            ("id", Value::from(id.as_str())),
            ("base", Value::from(base.as_str())),
            (
                "landed",
                Value::Array(landed.iter().map(landed_json).collect()),
            ),
        ]),
        ServiceMessage::Waiting {
            text,
        } => object(vec![
            ("type", Value::from("waiting")),
            ("text", Value::from(text.as_str())),
        ]),
        ServiceMessage::InsideWrite {
            text,
        } => object(vec![
            ("type", Value::from("waiting")),
            ("text", Value::from(text.as_str())),
            ("inside_write", Value::from(true)),
        ]),
        ServiceMessage::Holder {
            who,
        } => object(vec![
            ("type", Value::from("holder")),
            ("who", Value::from(who.as_str())),
        ]),
        ServiceMessage::Heartbeat => object(vec![("type", Value::from("heartbeat"))]),
        ServiceMessage::Withdrawn {
            id,
        } => object(vec![
            ("type", Value::from("withdrawn")),
            ("id", Value::from(id.as_str())),
        ]),
        ServiceMessage::Failed {
            id,
            code,
        } => object(vec![
            ("type", Value::from("failed")),
            ("id", Value::from(id.as_str())),
            ("code", Value::from(code.word())),
        ]),
    };
}
