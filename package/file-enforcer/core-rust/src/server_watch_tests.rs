//! What:
//!  Tests for `watch` (an absent directory that gets created,
//! and the per-user server)
//!  and for a client whose server vanishes in the middle of a request.
//! Why:
//!  Both paths depend on timing and on a peer that misbehaves,
//! so they need a client that can watch in the
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
///  `Sender<Event>` is the sending half of a thread-safe queue (TS:
/// an `EventTarget` another thread can dispatch on).
/// Why:
///  The watch request blocks its thread until the server stops,
/// so the test reads events from another thread.
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
///  A closed receiver means the test is over;
/// dropped events are fine then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ChannelSink { emit(event: Event): void }
/// ```
impl EventSink for ChannelSink {
    /// Send the event;
    /// ignore a closed channel.
    fn emit(&mut self, event: Event) {
        let _ = self.sender.send(event);
    }
}

/// What:
///  A `watch` request running on its own thread,
/// and the events it has produced.
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
    /// The request thread,
    /// joined at the end.
    thread: Option<JoinHandle<()>>,
}

/// What:
///  Start and stop a watch,
/// and wait for events by code.
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

    /// Wait up to `seconds` for an event with `code`,
    /// returning it;
    /// earlier events are skipped.
    fn wait_for(&self, code: &str, seconds: u64) -> Event {
        return match self.try_wait_for(code, Duration::from_secs(seconds)) {
            Ok(event) => event,
            Err(skipped) => panic!("no {code} event within {seconds} seconds; saw {skipped:?}"),
        };
    }

    /// Like `wait_for` but returns the messages of the events skipped on timeout.
    fn try_wait_for(&self, code: &str, limit: Duration) -> Result<Event, Vec<String>> {
        let deadline = Instant::now() + limit;
        let mut skipped: Vec<String> = Vec::new();
        while Instant::now() < deadline {
            if let Ok(event) = self.events.recv_timeout(Duration::from_millis(100)) {
                if event.code == code {
                    return Ok(event);
                }
                skipped.push(format!("{}: {}", event.code, event.message));
            }
        }
        return Err(skipped);
    }

    /// Wait for the request to return after the server was asked to stop,
    /// and join its thread.
    fn finish(mut self) -> i32 {
        let code = self.done.recv_timeout(Duration::from_secs(15)).expect("the watch request returns after shutdown");
        if let Some(thread) = self.thread.take() {
            thread.join().expect("the watch thread ends cleanly");
        }
        return code;
    }
}

/// What:
///  Run `action` again and again until an event with `code` arrives or twenty seconds pass.
///  `impl FnMut(usize)` is any closure that takes the attempt number and may change what it captured
///  (TS:
/// a callback `(attempt: number) => void`).
/// Why:
///  A file-system change made before the watcher has registered the path is lost,
/// and nothing on the event stream
///  says when registration finished,
/// so the test repeats the change instead of sleeping a fixed time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function retryUntil(watch: WatchClient, code: string, action: (attempt: number) => void): boolean
/// ```
fn retry_until(watch: &WatchClient, code: &str, mut action: impl FnMut(usize)) -> bool {
    let deadline = Instant::now() + Duration::from_secs(20);
    let mut attempt = 0;
    while Instant::now() < deadline {
        attempt += 1;
        action(attempt);
        if watch.try_wait_for(code, Duration::from_millis(1500)).is_ok() {
            return true;
        }
    }
    return false;
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
    // The config lives outside the worktree so that the worktree root is watched only because the absent
    // input's nearest existing ancestor is the root; otherwise the config's own directory would hide a bug there.
    let config_home = scratch_dir();
    let config = config_home.path().join("file-enforcer.config.jsonc");
    let text = r#"{"rules": [{"kind": "concat", "id": "c", "output": "out.txt", "parts": [{"file": "later/in.txt"}]}]}"#;
    write_files(config_home.path(), &[("file-enforcer.config.jsonc", text)]);
    let server = TestServer::start(dir.path(), runtime.path(), Duration::from_secs(60));
    let watch = WatchClient::start(&server.client(), &config);
    watch.wait_for("watch-started", 10);
    watch.wait_for("input-missing", 10);
    // The worktree root is watched only after the first run finishes, so a change made right now can be missed;
    // touching an unrelated file until another run happens proves the root is watched before the real step.
    let root_watched = retry_until(&watch, "input-missing", |attempt| {
        std::fs::write(dir.path().join("probe.txt"), format!("probe {attempt}")).expect("touch an unrelated file");
    });
    assert!(root_watched, "the worktree root was never watched: it is the nearest existing ancestor of the absent input");
    // Creating the absent directory is a change to that ancestor, which the watch must notice.
    std::fs::create_dir_all(dir.path().join("later")).expect("create the absent directory");
    watch.wait_for("input-missing", 10);
    // The new directory is watched only after the next loop pass, so a write made too early is missed.
    let written = retry_until(&watch, "file-written", |attempt| {
        std::fs::write(dir.path().join("later/in.txt"), format!("created {attempt}")).expect("write the input");
    });
    assert!(written, "the file created inside the new directory was never noticed");
    assert!(read_text(dir.path(), "out.txt").starts_with("created "), "{}", read_text(dir.path(), "out.txt"));
    stop(server);
    assert_eq!(watch.finish(), 0);
}

/// What:
///  A change made the moment the first run reports its write is noticed without the test repeating it.
/// Why:
///  The directories a run read are watched only after that run ends,
/// so a change in that gap produced no event and was lost until the next unrelated change.
/// The server now runs one more cycle after it arms a new watch,
/// which is a condition the test can wait on,
/// not a delay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// it('notices a change made right after the first run', async () => { ... });
/// ```
#[test]
fn a_change_made_right_after_the_first_run_is_not_lost() {
    let runtime = short_runtime_dir();
    let dir = scratch_dir();
    let text = r#"{"rules": [{"kind": "concat", "id": "c", "output": "out.txt", "parts": [{"file": "in.txt"}]}]}"#;
    write_files(dir.path(), &[("file-enforcer.config.jsonc", text), ("in.txt", "first\n")]);
    let config = dir.path().join("file-enforcer.config.jsonc");
    let server = TestServer::start(dir.path(), runtime.path(), Duration::from_secs(60));
    let watch = WatchClient::start(&server.client(), &config);
    watch.wait_for("watch-started", 10);
    watch.wait_for("file-written", 10);
    std::fs::write(dir.path().join("in.txt"), "second\n").expect("rewrite the input");
    watch.wait_for("file-written", 30);
    assert!(read_text(dir.path(), "out.txt").contains("second"), "{}", read_text(dir.path(), "out.txt"));
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
        seen = watch.try_wait_for("config-invalid", Duration::from_millis(1500)).ok();
    }
    let event = seen.expect("changing the per-user config was never noticed");
    assert!(event.message.contains("belongs in the repository config, not the per-user config"), "{}", event.message);
    stop(server);
    assert_eq!(watch.finish(), 0);
}

/// What:
///  A fake server on `socket`:
/// it answers the handshake,
/// then reads one request and hangs up after `events` events.
///  `UnixListener` is a socket that accepts connections on a file path (TS:
/// `net.createServer().listen(path)`).
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
