//! What:
//!  The client side:
//! find or start the server,
//! handshake,
//! send one request,
//! and print what comes back.
//! Why:
//!  Every command except the hidden `serve` is a client.
//! A request is never replayed after an ambiguous disconnect:
//!  a dropped connection mid-run is reported,
//! because the server may or may not have written.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export async function connect(options: ClientOptions): Promise<Connection>
//! ```

/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// Imports `Event`,
/// `EventSink` from `crate::event`.
use crate::event::{Event, EventSink};
/// Imports `build_identity`,
/// `worktree_identity` from `crate::identity`.
use crate::identity::{build_identity, worktree_identity};
/// Imports `ServerLaunch`,
/// `StartOutcome`,
/// `start_server_if_needed` from `crate::launch`.
use crate::launch::{ServerLaunch, StartOutcome, start_server_if_needed};
/// Imports `ERROR_MISMATCH`,
/// `PROTOCOL_VERSION`,
/// `request_line` from `crate::rpc_protocol`.
use crate::rpc_protocol::{ERROR_MISMATCH, PROTOCOL_VERSION, request_line};
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
/// Imports `Path`,
/// `PathBuf` from `std::path`.
use std::path::{Path, PathBuf};
/// Imports `Duration`,
/// `Instant` from `std::time`.
use std::time::{Duration, Instant};

/// What:
///  What a client needs to reach its server.
/// Why:
///  The CLI fills this from the command line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClientOptions = { root: string; socket: string; launch: ServerLaunch; deadline: number };
/// ```
#[derive(Clone, Debug)]
pub struct ClientOptions {
    /// Worktree root (or home directory for the per-user server).
    pub root: PathBuf,
    /// Socket path.
    pub socket: PathBuf,
    /// How to start a server.
    pub launch: ServerLaunch,
    /// How long to keep trying to reach or start a server.
    pub deadline: Duration,
}

/// What:
///  A connected,
/// handshaken connection.
/// Why:
///  The buffered reader and the writer must stay paired so no bytes are lost between requests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Connection = { reader: LineReader; writer: Socket };
/// ```
pub struct Connection {
    /// Buffered reading half of the socket;
    /// responses and event notifications arrive line by line.
    reader: BufReader<UnixStream>,
    /// Writing half of the socket;
    /// requests are sent on it.
    writer: UnixStream,
}

/// What:
///  Why a handshake did not succeed.
/// Why:
///  A mismatch means drain the old server and start a new one;
/// anything else is retried.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HandshakeFailure = { kind: 'mismatch'; pid: number } | { kind: 'other' };
/// ```
enum HandshakeFailure {
    /// The server reported a different protocol,
    /// build or worktree and is draining;
    /// the client waits for it to go and starts a new one.
    Mismatch,
    /// Any other handshake failure;
    /// the client retries after a short wait.
    Other,
}

/// What:
///  Send the handshake and read the reply.
/// Why:
///  The server compares protocol,
/// build and worktree identities before it serves anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function handshake(stream: Socket, root: string): Promise<Connection>
/// ```
fn handshake(stream: UnixStream, root: &Path) -> Result<Connection, HandshakeFailure> {
    let Ok(worktree) = worktree_identity(root) else { return Err(HandshakeFailure::Other) };
    let pairs = [
        ("protocolVersion", Value::from(PROTOCOL_VERSION)),
        ("buildId", Value::from(build_identity())),
        ("worktree", worktree),
    ];
    let params = object(&pairs);
    let Ok(reader_stream) = stream.try_clone() else { return Err(HandshakeFailure::Other) };
    let mut writer = stream;
    if writer.write_all(request_line(1, "handshake", params).as_bytes()).is_err() {
        return Err(HandshakeFailure::Other);
    }
    let mut reader = BufReader::new(reader_stream);
    let mut line = String::new();
    if reader.read_line(&mut line).unwrap_or(0) == 0 {
        return Err(HandshakeFailure::Other);
    }
    let Ok(reply) = serde_json::from_str::<Value>(&line) else { return Err(HandshakeFailure::Other) };
    if reply["error"]["code"].as_i64() == Some(ERROR_MISMATCH) {
        return Err(HandshakeFailure::Mismatch);
    }
    if reply["result"]["ok"].as_bool() == Some(true) {
        return Ok(Connection { reader, writer });
    }
    return Err(HandshakeFailure::Other);
}

/// What:
///  Wait for a draining server to remove its socket.
/// Why:
///  The replacement must not start while the old server still owns the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function waitForSocketGone(socket: string, deadline: number): Promise<void>
/// ```
fn wait_for_socket_gone(socket: &Path, deadline: Instant) {
    while Instant::now() < deadline && socket.exists() {
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// What:
///  Reach a server for this worktree,
/// starting one if none answers,
/// and complete the handshake.
/// Why:
///  A stale or mismatched server is replaced;
/// many callers at once end up on the same new server.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function connect(options: ClientOptions): Promise<Connection>
/// ```
pub fn connect(options: &ClientOptions) -> Result<Connection, Diagnostic> {
    let deadline = Instant::now() + options.deadline;
    while Instant::now() < deadline {
        match UnixStream::connect(&options.socket) {
            Ok(stream) => match handshake(stream, &options.root) {
                Ok(connection) => return Ok(connection),
                Err(HandshakeFailure::Mismatch) => wait_for_socket_gone(&options.socket, deadline),
                Err(HandshakeFailure::Other) => std::thread::sleep(Duration::from_millis(25)),
            },
            Err(_) => match start_server_if_needed(&options.socket, &options.launch, deadline)? {
                StartOutcome::Waiting => std::thread::sleep(Duration::from_millis(25)),
                StartOutcome::Started | StartOutcome::AlreadyRunning => {}
            },
        }
    }
    let message = format!("no server answered on {} within {:?}", options.socket.display(), options.deadline);
    return Err(Diagnostic::new("server-unavailable", &message));
}

/// What:
///   Sending a request on an established connection and asking the server to stop.
/// Why:
///   `request` streams the server's events into a sink and returns the exit code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Connection { request(method, params, sink): number; shutdown(): number | undefined }
/// ```
impl Connection {
    /// What:
    ///  Send one request and print its events until the response arrives;
    /// return the exit code.
    /// Why:
    ///  A disconnect before the response is reported and never retried:
    /// the server may have written.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async request(method: string, params: unknown, sink: EventSink): Promise<number>
    /// ```
    pub fn request(&mut self, method: &str, params: Value, sink: &mut dyn EventSink) -> i32 {
        if self.writer.write_all(request_line(2, method, params).as_bytes()).is_err() {
            sink.emit(disconnected());
            return 1;
        }
        loop {
            let mut line = String::new();
            if self.reader.read_line(&mut line).unwrap_or(0) == 0 {
                sink.emit(disconnected());
                return 1;
            }
            let Ok(message) = serde_json::from_str::<Value>(&line) else { continue };
            if message["method"] == "event" {
                if let Some(event) = Event::from_json(&message["params"]) {
                    sink.emit(event);
                }
                continue;
            }
            if message["id"] != 2 {
                continue;
            }
            if let Some(code) = message["result"]["exitCode"].as_i64() {
                return i32::try_from(code).unwrap_or(1);
            }
            let text = message["error"]["message"].as_str().unwrap_or("the server reported an error");
            sink.emit(Diagnostic::new("server-error", text).to_event("error"));
            return 1;
        }
    }

    /// What:
    ///  Ask the server to stop and return its process id.
    /// Why:
    ///  Tests and `Drop` guards stop their servers explicitly instead of waiting for the idle exit.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async shutdown(): Promise<number | undefined>
    /// ```
    pub fn shutdown(&mut self) -> Option<u64> {
        if self.writer.write_all(request_line(3, "shutdown", Value::Null).as_bytes()).is_err() {
            return None;
        }
        let mut line = String::new();
        if self.reader.read_line(&mut line).unwrap_or(0) == 0 {
            return None;
        }
        let reply: Value = serde_json::from_str(&line).ok()?;
        return reply["result"]["pid"].as_u64();
    }
}

/// What:
///  The event that reports a connection lost mid-request.
/// Why:
///  The message says plainly that the outcome is unknown and what to do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function disconnected(): Event
/// ```
fn disconnected() -> Event {
    let mut event = Event::new(
        "error",
        "server-disconnected",
        "the server closed the connection before it answered; whether it wrote any file is unknown",
    );
    event.hint = Some("run `file-enforcer check` to see the current state, then `file-enforcer write` again".to_string());
    return event;
}
