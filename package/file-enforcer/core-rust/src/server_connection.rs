//! What:
//!  One client connection:
//! the handshake,
//! then `run`,
//! `watch` and `shutdown` requests.
//! Why:
//!  The handshake (protocol version,
//! build identity,
//! worktree identity) comes first,
//! so a server from another build
//!  or serving another directory never answers a request.
//! A mismatch makes the server drain and exit so the client
//!  can start a fresh one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! async function handleConnection(shared: Shared, socket: Socket): Promise<void>
//! ```

/// Imports `Event`,
/// `EventSink` from `crate::event`.
use crate::event::{Event, EventSink};
/// Imports  from `crate::rpc_protocol`.
use crate::rpc_protocol::{
    ERROR_FAILED, ERROR_METHOD, ERROR_MISMATCH, ERROR_PARSE, PROTOCOL_VERSION, error_line, notification_line, result_line,
};
/// Imports `RunMode`,
/// `RunRequest`,
/// `run` from `crate::run`.
use crate::run::{RunMode, RunRequest, run};
/// Imports `Shared` from `crate::server`.
use crate::server::Shared;
/// Imports `watch_connection` from `crate::watch`.
use crate::watch::watch_connection;
/// Imports `object` from `crate::json_build`.
use crate::json_build::object;
/// Imports `Value` from `serde_json`.
use serde_json::Value;
/// Imports `BufRead`,
/// `BufReader`,
/// `Write` from `std::io`.
use std::io::{BufRead, BufReader, Write};
/// Imports `UnixStream` from `std::os::unix::net`.
use std::os::unix::net::UnixStream;
/// Imports `PathBuf` from `std::path`.
use std::path::PathBuf;
/// Imports `Ordering` from `std::sync::atomic`.
use std::sync::atomic::Ordering;

/// What:
///  An event sink that sends each event to the client as an `event` notification.
/// Why:
///  The client prints the events with its own sequence numbers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RpcSink implements EventSink { socket: Socket }
/// ```
pub struct RpcSink {
    /// The client's socket;
    /// events are written to a clone of it.
    stream: UnixStream,
}

/// What:
///   Constructing the sink and sending a keep-alive.
/// Why:
///   `watch` pings so a vanished client is noticed within one tick.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RpcSink { constructor(stream); ping(): boolean }
/// ```
impl RpcSink {
    /// What:
    ///  Wrap a stream.
    /// Why:
    ///  Clones of one connection's stream share the socket.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// new RpcSink(socket)
    /// ```
    pub fn new(stream: UnixStream) -> RpcSink {
        return RpcSink { stream };
    }

    /// What:
    ///  Send a keepalive notification;
    /// `false` when the client is gone.
    /// Why:
    ///  A write to a closed peer fails,
    /// which is how a long-lived `watch` notices its client left.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ping(): boolean
    /// ```
    pub fn ping(&mut self) -> bool {
        return self.stream.write_all(notification_line("ping", Value::Null).as_bytes()).is_ok();
    }
}

/// What:
///   How events leave the server:
/// each becomes one `event` notification line on the socket.
/// Why:
///   A write failure is ignored because the client may have gone;
/// the run still completes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// emit(event: Event): void { socket.write(notification('event', event)); }
/// ```
impl EventSink for RpcSink {
    /// Send the event;
    /// a closed client is ignored so the run still finishes.
    fn emit(&mut self, event: Event) {
        let line = notification_line("event", event.to_json(0));
        let _ = self.stream.write_all(line.as_bytes());
    }
}

/// The `{"ok": true, "pid": <this process>}` result of the handshake and of `shutdown`.
fn ok_with_pid() -> Value {
    return object(&[("ok", Value::Bool(true)), ("pid", Value::from(std::process::id()))]);
}

/// The `{"exitCode": <code>}` result of `run` and `watch`.
fn exit_code_object(code: i32) -> Value {
    return object(&[("exitCode", Value::from(code))]);
}

/// What:
///  Compare the client's handshake with this server;
/// `Some(reason)` on a mismatch.
/// Why:
///  Each of the three identities can differ independently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function handshakeMismatch(shared: Shared, params: unknown): string | undefined
/// ```
fn handshake_mismatch(shared: &Shared, params: &Value) -> Option<String> {
    if params["protocolVersion"].as_u64() != Some(PROTOCOL_VERSION) {
        return Some("the protocol version differs".to_string());
    }
    if params["buildId"].as_str() != Some(shared.build.as_str()) {
        return Some("the build identity differs".to_string());
    }
    if params["worktree"] != shared.worktree {
        return Some("the worktree identity differs".to_string());
    }
    return None;
}

/// What:
///  Read the `run` parameters into a request.
/// Why:
///  Each request carries its canonical config path and options;
/// the config is re-read every run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runRequestOf(shared: Shared, params: unknown): RunRequest | string
/// ```
fn run_request_of(shared: &Shared, params: &Value) -> Result<RunRequest, String> {
    let Some(config) = params["configPath"].as_str() else {
        return Err("the run request has no configPath".to_string());
    };
    let mode = match params["mode"].as_str() {
        Some("write") => RunMode::Write,
        Some("check") => RunMode::Check { expected_dir: params["expectedDir"].as_str().map(PathBuf::from) },
        Some("explain") => RunMode::Explain { path: params["path"].as_str().unwrap_or_default().to_string() },
        _ => return Err("the run request has an unknown mode".to_string()),
    };
    return Ok(RunRequest {
        root: shared.options.root.clone(),
        config_path: PathBuf::from(config),
        scope: shared.options.scope,
        mode,
    });
}

/// What:
///  Serve one connection until it closes or asks the server to stop.
/// Why:
///  Runs execute under the shared run lock,
/// one at a time across all connections.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function handleConnection(shared: Shared, socket: Socket): Promise<void>
/// ```
pub fn handle_connection(shared: &Shared, stream: UnixStream) {
    let Ok(reader_stream) = stream.try_clone() else { return };
    let mut writer = stream;
    let mut shaken = false;
    for line in BufReader::new(reader_stream).lines() {
        let Ok(text) = line else { return };
        if text.trim().is_empty() {
            continue;
        }
        let request: Value = match serde_json::from_str(&text) {
            Ok(request) => request,
            Err(error) => {
                let reply = error_line(&Value::Null, ERROR_PARSE, &format!("not valid JSON: {error}"), Value::Null);
                let _ = writer.write_all(reply.as_bytes());
                return;
            }
        };
        let id = request["id"].clone();
        let method = request["method"].as_str().unwrap_or_default().to_string();
        let params = request["params"].clone();
        if method == "handshake" {
            if let Some(reason) = handshake_mismatch(shared, &params) {
                let data = object(&[("reason", Value::from(reason)), ("pid", Value::from(std::process::id()))]);
                let _ = writer.write_all(error_line(&id, ERROR_MISMATCH, "handshake mismatch", data).as_bytes());
                shared.shutdown.store(true, Ordering::SeqCst);
                return;
            }
            shaken = true;
            let _ = writer.write_all(result_line(&id, ok_with_pid()).as_bytes());
            continue;
        }
        if !shaken {
            let reply = error_line(&id, ERROR_METHOD, "the first request must be the handshake", Value::Null);
            let _ = writer.write_all(reply.as_bytes());
            return;
        }
        match method.as_str() {
            "run" => {
                let reply = match run_request_of(shared, &params) {
                    Ok(run_request) => {
                        let Ok(sink_stream) = writer.try_clone() else { return };
                        let mut sink = RpcSink::new(sink_stream);
                        let guard = shared.run_lock.lock();
                        let code = run(&run_request, &shared.registry, &mut sink);
                        drop(guard);
                        result_line(&id, exit_code_object(code))
                    }
                    Err(message) => error_line(&id, ERROR_FAILED, &message, Value::Null),
                };
                let _ = writer.write_all(reply.as_bytes());
            }
            "watch" => {
                let Some(config) = params["configPath"].as_str() else {
                    let reply = error_line(&id, ERROR_FAILED, "the watch request has no configPath", Value::Null);
                    let _ = writer.write_all(reply.as_bytes());
                    return;
                };
                let Ok(sink_stream) = writer.try_clone() else { return };
                watch_connection(shared, RpcSink::new(sink_stream), &PathBuf::from(config));
                let _ = writer.write_all(result_line(&id, exit_code_object(0)).as_bytes());
                return;
            }
            "shutdown" => {
                let _ = writer.write_all(result_line(&id, ok_with_pid()).as_bytes());
                shared.shutdown.store(true, Ordering::SeqCst);
                return;
            }
            other => {
                let reply = error_line(&id, ERROR_METHOD, &format!("unknown method {other}"), Value::Null);
                let _ = writer.write_all(reply.as_bytes());
            }
        }
    }
}
