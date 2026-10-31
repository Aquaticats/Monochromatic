//! What:
//!  The JSON event every output line is, and where events go.
//! Why:
//!  Every line file-enforcer writes is one JSON object, so the engine emits `Event`s to an `EventSink` and only the
//!  outermost layer decides whether they become lines on standard output, test assertions or socket notifications.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Event = { type: string; code: string; message: string; hint?: string; fields: Record<string, unknown> };
//! ```

use serde_json::{Map, Value};

/// What:
///  The version number every event line carries as `schemaVersion`.
///  `u64` is an unsigned 64-bit integer (siblings: `u32`, `i64`).
/// Why:
///  Consumers branch on it when the event shape changes; `u64` matches the number type `serde_json` writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SCHEMA_VERSION = 1;
/// ```
pub const SCHEMA_VERSION: u64 = 1;

/// What:
///  One output line before it is numbered: its type, code, message, optional hint and extra fields.
/// Why:
///  The sequence number belongs to whoever prints the lines (one counter per process), not to the engine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Event = { type: string; code: string; message: string; hint?: string; fields: Record<string, unknown> };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    /// The `type` member: `drift`, `warning`, `error`, `help`, `done`, and so on.
    pub event_type: String,
    /// Stable kebab-case handle for the specific occurrence.
    pub code: String,
    /// Human-readable explanation.
    pub message: String,
    /// What a person can do next.
    pub hint: Option<String>,
    /// Further members, written after `hint` in insertion order.
    pub fields: Map<String, Value>,
}

impl Event {
    /// What:
    ///  Build an event with no hint and no extra fields.
    /// Why:
    ///  The common core of every line.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function event(type: string, code: string, message: string): Event;
    /// ```
    pub fn new(event_type: &str, code: &str, message: &str) -> Event {
        return Event {
            event_type: event_type.to_string(),
            code: code.to_string(),
            message: message.to_string(),
            hint: None,
            fields: Map::new(),
        };
    }

    /// What:
    ///  Add or replace one extra member.
    /// Why:
    ///  Drift and explain events carry `path`, `rule` and similar members.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// setField(key: string, value: unknown): void;
    /// ```
    pub fn set_field(&mut self, key: &str, value: Value) {
        self.fields.insert(key.to_string(), value);
    }

    /// What:
    ///  Render the event as a JSON object value with the given sequence number.
    /// Why:
    ///  The same object travels as a line on standard output and inside a JSON-RPC notification.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toJson(sequence: number): object;
    /// ```
    pub fn to_json(&self, sequence: u64) -> Value {
        let mut object = Map::new();
        object.insert("schemaVersion".to_string(), Value::from(SCHEMA_VERSION));
        object.insert("sequence".to_string(), Value::from(sequence));
        object.insert("type".to_string(), Value::String(self.event_type.clone()));
        object.insert("code".to_string(), Value::String(self.code.clone()));
        object.insert("message".to_string(), Value::String(self.message.clone()));
        if let Some(hint) = &self.hint {
            object.insert("hint".to_string(), Value::String(hint.clone()));
        }
        for (key, value) in &self.fields {
            object.insert(key.clone(), value.clone());
        }
        return Value::Object(object);
    }

    /// What:
    ///  Render the event as one newline-terminated JSON line.
    /// Why:
    ///  Standard output is JSON lines only.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toLine(sequence: number): string { return JSON.stringify(this.toJson(sequence)) + '\n'; }
    /// ```
    pub fn to_line(&self, sequence: u64) -> String {
        let mut line = self.to_json(sequence).to_string();
        line.push('\n');
        return line;
    }

    /// What:
    ///  Read an event back from the object a JSON-RPC notification carries.
    /// Why:
    ///  The client re-numbers server events with its own counter before printing them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static fromJson(value: unknown): Event | undefined;
    /// ```
    pub fn from_json(value: &Value) -> Option<Event> {
        let object = value.as_object()?;
        let event_type = object.get("type")?.as_str()?;
        let code = object.get("code")?.as_str()?;
        let message = object.get("message")?.as_str()?;
        let mut event = Event::new(event_type, code, message);
        if let Some(hint) = object.get("hint").and_then(Value::as_str) {
            event.hint = Some(hint.to_string());
        }
        for (key, member) in object {
            if key == "schemaVersion" || key == "sequence" || key == "type" || key == "code" {
                continue;
            }
            if key == "message" || key == "hint" {
                continue;
            }
            event.fields.insert(key.clone(), member.clone());
        }
        return Some(event);
    }
}

/// What:
///  Where events go: a printer, a collector in a test, or a socket connection.
///  `dyn` trait objects are values chosen at run time (TS: an interface-typed parameter).
/// Why:
///  The engine emits events without knowing whether a person, a test or a remote client reads them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface EventSink { emit(event: Event): void }
/// ```
pub trait EventSink {
    /// Receive one event; sinks that can fail keep the failure to themselves.
    fn emit(&mut self, event: Event);
}

/// What:
///  A sink that keeps every event in a list.
/// Why:
///  Tests and the in-process runner read the list after the run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CollectingSink implements EventSink { events: Event[] = []; }
/// ```
#[derive(Debug, Default)]
pub struct CollectingSink {
    /// Events in emission order.
    pub events: Vec<Event>,
}

impl EventSink for CollectingSink {
    /// What:
    ///  Append the event to the list.
    /// Why:
    ///  Keeps emission order.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// emit(event: Event) { this.events.push(event); }
    /// ```
    fn emit(&mut self, event: Event) {
        self.events.push(event);
    }
}
