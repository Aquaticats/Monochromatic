//! What:
//!  The newline-delimited JSON-RPC 2.0 messages between the `file-enforcer` client and its per-worktree server.
//! Why:
//!  One JSON object per line over a Unix socket is simple to emit,
//! to read from any language and to test;
//! events
//!  stream back as `event` notifications and the final response carries the exit code.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Message = { jsonrpc: '2.0'; id?: number; method?: string; params?: unknown; result?: unknown; error?: { code: number; message: string; data?: unknown } };
//! ```

/// Imports `object` from `crate::json_build`.
use crate::json_build::object;
/// Imports `Value` from `serde_json`.
use serde_json::Value;

/// A JSON string value from text,
/// for the message builders below.
fn v(text: &str) -> Value {
    return Value::String(text.to_string());
}

/// The protocol version both sides must report in the handshake.
pub const PROTOCOL_VERSION: u64 = 1;

/// JSON-RPC error code:
/// the line was not valid JSON.
pub const ERROR_PARSE: i64 = -32700;
/// JSON-RPC error code:
/// the method is unknown or called before the handshake.
pub const ERROR_METHOD: i64 = -32601;
/// Server error code:
/// the handshake found a different protocol,
/// build or worktree.
pub const ERROR_MISMATCH: i64 = -32001;
/// Server error code:
/// the request could not be run.
pub const ERROR_FAILED: i64 = -32002;

/// What:
///  Render a request as one line.
///  `u64` is an unsigned 64-bit integer (sibling:
/// `i64`).
/// Why:
///  Every request carries an id so a response can never be mistaken for another request's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function requestLine(id: number, method: string, params: unknown): string
/// ```
pub fn request_line(id: u64, method: &str, params: Value) -> String {
    let pairs = [("jsonrpc", v("2.0")), ("id", Value::from(id)), ("method", v(method)), ("params", params)];
    let mut line = object(&pairs).to_string();
    line.push('\n');
    return line;
}

/// What:
///  Render a notification (a request without an id) as one line.
/// Why:
///  Events and keepalives need no response.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function notificationLine(method: string, params: unknown): string
/// ```
pub fn notification_line(method: &str, params: Value) -> String {
    let pairs = [("jsonrpc", v("2.0")), ("method", v(method)), ("params", params)];
    let mut line = object(&pairs).to_string();
    line.push('\n');
    return line;
}

/// What:
///  Render a success response as one line.
/// Why:
///  The `run` response carries `exitCode`;
/// the handshake response carries the server's pid.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resultLine(id: unknown, result: unknown): string
/// ```
pub fn result_line(id: &Value, result: Value) -> String {
    let pairs = [("jsonrpc", v("2.0")), ("id", id.clone()), ("result", result)];
    let mut line = object(&pairs).to_string();
    line.push('\n');
    return line;
}

/// What:
///  Render an error response as one line.
/// Why:
///  A mismatch error carries `data` with the reason and the server's pid.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function errorLine(id: unknown, code: number, message: string, data?: unknown): string
/// ```
pub fn error_line(id: &Value, code: i64, message: &str, data: Value) -> String {
    let detail = object(&[("code", Value::from(code)), ("message", v(message)), ("data", data)]);
    let pairs = [("jsonrpc", v("2.0")), ("id", id.clone()), ("error", detail)];
    let mut line = object(&pairs).to_string();
    line.push('\n');
    return line;
}
