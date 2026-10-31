//! What:
//!  Tests for `watch` (an absent directory that gets created, and the per-user server)
//!  and for a client whose server vanishes in the middle of a request.
//! Why:
//!  Both paths depend on timing and on a peer that misbehaves, so they need a client that can watch in the
//!  background and a fake server that answers the handshake and then hangs up.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('watch and disconnects', () => { it('notices a directory being created', async () => { ... }); });
//! ```

use crate::client::{ClientOptions, connect};
use crate::config_decode::ConfigScope;
use crate::event::{CollectingSink, Event, EventSink};
use crate::json_build::object;
use crate::launch::ServerLaunch;
use crate::rpc_protocol::{notification_line, result_line};
use crate::server_tests::{TestServer, short_runtime_dir};
use crate::test_support::{read_text, scratch_dir, write_files};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// What:
///  An event sink that forwards every event into a channel.
///  `Sender<Event>` is the sending half of a thread-safe queue (TS: an `EventTarget` another thread can dispatch on).
/// Why:
///  The watch request blocks its thread until the server stops, so the test reads events from another thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ChannelSink implements EventSink { emit(event: Event) { this.sender.send(event); } }
/// ```
struct ChannelSink {
    /// Where events go.
    sender: Sender<Event>,
}

/// What:
///  The sink's behavior.
/// Why:
///  A closed receiver means the test is over; dropped events are fine then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ChannelSink { emit(event: Event): void }
/// ```
impl EventSink for ChannelSink {
    /// Send the event; ignore a closed channel.
    fn emit(&mut self, event: Event) {
        let _ = self.sender.send(event);
    }
}

/// What:
///  A `watch` request running on its own thread, and the events it has produced.
/// Why:
///  The test reads events as they arrive and finally waits for the request to end after the server shuts down.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WatchClient = { events: AsyncIterable<Event>; done: Promise<number> };
/// ```
struct WatchClient {
    /// Events in arrival order.
    events: Receiver<Event>,
    /// Yields the exit code once the request returns.
    done: Receiver<i32>,
    /// The request thread, joined at the end.
    thread: Option<JoinHandle<()>>,
}

/// What:
///  Start and stop a watch, and wait for events by code.
/// Why:
///  Every test below needs the same three moves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class WatchClient { static start(server, config): WatchClient; waitFor(code, seconds): Event; finish(): number }
/// ```
impl WatchClient {
    /// Connect to the server and send a `watch` request for `config` from a new thread.
    fn start(options: &ClientOptions, config: &Path) -> WatchClient {
        let mut connection = connect(options).expect("the watch client handshakes");
        let params = object(&[("mode", Value::from("write")), ("configPath", Value::from(config.display().to_string()))]);
        let (sender, events) = channel();
        let (done_sender, done) = channel();
        let thread = std::thread::spawn(move || {
            let code = connection.request("watch", params, &mut ChannelSink { sender });
            let _ = done_sender.send(code);
        });
        return WatchClient { events, done, thread: Some(thread) };
    }

    /// Wait up to `seconds` for an event with `code`, returning it; earlier events are skipped.
    fn wait_for(&self, code: &str, seconds: u64) -> Event {
        return self.try_wait_for(code, Duration::from_secs(seconds)).unwrap_or_else(|| panic!("no {code} event within {seconds} seconds"));
    }

    /// Like `wait_for` but returns `None` on timeout.
    fn try_wait_for(&self, code: &str, limit: Duration) -> Option<Event> {
        let deadline = Instant::now() + limit;
        while Instant::now() < deadline {
            if let Ok(event) = self.events.recv_timeout(Duration::from_millis(100))
                && event.code == code
            {
                return Some(event);
            }
        }
        return None;
    }

    /// Wait for the request to return after the server was asked to stop, and join its thread.
    fn finish(mut self) -> i32 {
        let code = self.done.recv_timeout(Duration::from_secs(15)).expect("the watch request returns after shutdown");
        if let Some(thread) = self.thread.take() {
            thread.join().expect("the watch thread ends cleanly");
        }
        return code;
    }
}

/// What:
///  Shut a test server down through a second connection and wait for its thread.
/// Why:
///  The watch loop only ends when the server's shutdown flag is set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stop(server: TestServer): void
/// ```
fn stop(server: TestServer) {
    drop(server);
}

#[test]
fn watch_notices_an_absent_directory_being_created_and_then_a_file_inside_it() {
    let runtime = short_runtime_dir();
    let dir = scratch_dir();
    let config = dir.path().join("file-enforcer.config.jsonc");
    let text = r#"{"rules": [{"kind": "concat", "id": "c", "output": "out.txt", "parts": [{"file": "later/in.txt"}]}]}"#;
    write_files(dir.path(), &[("file-enforcer.config.jsonc", text)]);
    let server = TestServer::start(dir.path(), runtime.path(), Duration::from_secs(60));
    let watch = WatchClient::start(&server.client(), &config);
    watch.wait_for("watch-started", 10);
    watch.wait_for("input-missing", 10);
    // Creating the absent directory is a change to the nearest existing ancestor (the worktree root).
    std::fs::create_dir_all(dir.path().join("later")).expect("create the absent directory");
    watch.wait_for("input-missing", 10);
    // The new directory is watched only after the next loop pass, so a write made too early is missed;
    // rewriting the file until the output appears keeps the test free of a fixed sleep.
    let deadline = Instant::now() + Duration::from_secs(20);
    let mut written = false;
    let mut attempt = 0;
    while !written && Instant::now() < deadline {
        attempt += 1;
        std::fs::write(dir.path().join("later/in.txt"), format!("created {attempt}")).expect("write the input");
        written = watch.try_wait_for("file-written", Duration::from_millis(1500)).is_some();
    }
    assert!(written, "the file created inside the new directory was never noticed");
    assert!(read_text(dir.path(), "out.txt").starts_with("created "), "{}", read_text(dir.path(), "out.txt"));
    stop(server);
    assert_eq!(watch.finish(), 0);
}

#[test]
fn watch_on_the_per_user_server_follows_the_user_config() {
    let runtime = short_runtime_dir();
    let home = scratch_dir();
    let config = home.path().join("user.config.jsonc");
    std::fs::write(&config, r#"{"rules": []}"#).expect("the user config");
    let server = TestServer::start_scoped(home.path(), runtime.path(), Duration::from_secs(60), ConfigScope::User);
    let watch = WatchClient::start(&server.client(), &config);
    watch.wait_for("watch-started", 10);
    // A repository-only kind in the per-user config is a decode problem only the user scope reports.
    let misplaced = r#"{"rules": [{"kind": "mode-only", "id": "m", "path": "x", "mode": "600"}]}"#;
    let deadline = Instant::now() + Duration::from_secs(20);
    let mut seen: Option<Event> = None;
    while seen.is_none() && Instant::now() < deadline {
        std::fs::write(&config, misplaced).expect("rewrite the user config");
        seen = watch.try_wait_for("config-invalid", Duration::from_millis(1500));
    }
    let event = seen.expect("changing the per-user config was never noticed");
    assert!(event.message.contains("belongs in the repository config, not the per-user config"), "{}", event.message);
    stop(server);
    assert_eq!(watch.finish(), 0);
}

/// What:
///  A fake server on `socket`: it answers the handshake, then reads one request and hangs up after `events` events.
///  `UnixListener` is a socket that accepts connections on a file path (TS: `net.createServer().listen(path)`).
/// Why:
///  A server that dies in the middle of a request cannot be produced by the real one on demand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fakeServer(socket: string, events: number): Promise<void>
/// ```
fn fake_server(socket: PathBuf, events: usize, answer_request: bool) -> JoinHandle<()> {
    let listener = UnixListener::bind(&socket).expect("bind the fake server");
    return std::thread::spawn(move || {
        let Ok((stream, _)) = listener.accept() else { return };
        let mut reader = BufReader::new(stream.try_clone().expect("clone"));
        let mut writer = stream;
        let mut line = String::new();
        reader.read_line(&mut line).expect("the handshake request");
        let reply = result_line(&Value::from(1), object(&[("ok", Value::Bool(true)), ("pid", Value::from(0))]));
        writer.write_all(reply.as_bytes()).expect("the handshake reply");
        if !answer_request {
            return;
        }
        line.clear();
        reader.read_line(&mut line).expect("the run request");
        for index in 0..events {
            let payload = Event::new("progress", "fake-progress", &format!("event {index}")).to_json(0);
            writer.write_all(notification_line("event", payload).as_bytes()).expect("an event");
        }
    });
}

#[test]
fn a_server_that_hangs_up_mid_request_is_reported_as_disconnected_and_never_replayed() {
    let runtime = short_runtime_dir();
    let dir = scratch_dir();
    let socket = runtime.path().join("fake.sock");
    let server = fake_server(socket.clone(), 1, true);
    let launch = ServerLaunch { program: PathBuf::from("/bin/false"), args: Vec::new() };
    let options = ClientOptions { root: dir.path().to_path_buf(), socket, launch, deadline: Duration::from_secs(5) };
    let mut connection = connect(&options).expect("the fake server answers the handshake");
    let mut sink = CollectingSink::default();
    let code = connection.request("run", object(&[("mode", Value::from("write"))]), &mut sink);
    server.join().expect("the fake server ends");
    assert_eq!(code, 1);
    let codes: Vec<&str> = sink.events.iter().map(|event| return event.code.as_str()).collect();
    assert_eq!(codes, vec!["fake-progress", "server-disconnected"], "events seen before the hang-up are kept: {codes:?}");
    let last = sink.events.last().expect("the disconnect event");
    assert_eq!(last.event_type, "error");
    assert!(last.message.contains("whether it wrote any file is unknown"), "{}", last.message);
    assert!(last.hint.as_deref().is_some_and(|hint| return hint.contains("file-enforcer check")));
}

#[test]
fn a_server_that_hangs_up_right_after_the_handshake_is_also_reported() {
    let runtime = short_runtime_dir();
    let dir = scratch_dir();
    let socket = runtime.path().join("fake.sock");
    let server = fake_server(socket.clone(), 0, false);
    let launch = ServerLaunch { program: PathBuf::from("/bin/false"), args: Vec::new() };
    let options = ClientOptions { root: dir.path().to_path_buf(), socket, launch, deadline: Duration::from_secs(5) };
    let mut connection = connect(&options).expect("the fake server answers the handshake");
    server.join().expect("the fake server ends and closes its end");
    let mut sink = CollectingSink::default();
    let code = connection.request("run", object(&[("mode", Value::from("write"))]), &mut sink);
    assert_eq!(code, 1);
    assert_eq!(sink.events.len(), 1, "{:?}", sink.events);
    assert_eq!(sink.events[0].code, "server-disconnected");
}
