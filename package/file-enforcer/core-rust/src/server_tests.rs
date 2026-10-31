//! What:
//!  Tests for the server,
//! the handshake,
//! stale-socket recovery and the runtime directory.
//! Why:
//!  The server's lifecycle contracts (handshake,
//! drain on mismatch,
//! idle exit,
//! root-gone exit,
//! verified stale
//!  recovery,
//! owner-only runtime directory,
//! socket path length) each need a test that can fail.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('server', () => { it('exits when idle', async () => { ... }); });
//! ```

use crate::client::{ClientOptions, connect};
use crate::config_decode::ConfigScope;
use crate::event::CollectingSink;
use crate::json_build::object;
use crate::launch::{ServerLaunch, StartOutcome, start_server_if_needed};
use crate::registry::RuleRegistry;
use crate::rpc_protocol::{PROTOCOL_VERSION, request_line};
use crate::runtime_dir::{MAX_SOCKET_PATH_BYTES, runtime_directory, socket_path};
use crate::server::{ServerOptions, serve};
use crate::test_support::{read_text, scratch_dir, write_files};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// What:
///  A short-lived runtime directory under `${HOME}/temp/agent`,
/// short enough for a socket path.
/// Why:
///  The scratch under `target/` is longer than the 108-byte socket limit,
/// and a test server must never land in the
///  real `$XDG_RUNTIME_DIR`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shortRuntimeDir(): TempDir
/// ```
fn short_runtime_dir() -> TempDir {
    let home = std::env::var("HOME").expect("HOME is set");
    let base = Path::new(&home).join("temp/agent/fe-rust/impl20/rt");
    std::fs::create_dir_all(&base).expect("the runtime base is creatable");
    return TempDir::new_in(&base).expect("a runtime directory is creatable");
}

/// A running in-process server and what it needs to be stopped.
struct TestServer {
    handle: Option<JoinHandle<()>>,
    socket: PathBuf,
    root: PathBuf,
}

impl TestServer {
    /// Start a server thread for `root` with the given idle time and wait for its socket.
    fn start(root: &Path, runtime: &Path, idle: Duration) -> TestServer {
        let socket = runtime.join("test.sock");
        let options = ServerOptions { root: root.to_path_buf(), scope: ConfigScope::Repository, socket: socket.clone(), idle };
        let handle = std::thread::spawn(move || {
            let _ = serve(&options, RuleRegistry::with_defaults());
        });
        let deadline = Instant::now() + Duration::from_secs(5);
        while !socket.exists() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        return TestServer { handle: Some(handle), socket, root: root.to_path_buf() };
    }

    /// Client options that reach this server and never start another.
    fn client(&self) -> ClientOptions {
        let launch = ServerLaunch { program: PathBuf::from("/bin/false"), args: Vec::new() };
        return ClientOptions { root: self.root.clone(), socket: self.socket.clone(), launch, deadline: Duration::from_secs(5) };
    }

    /// Wait up to `limit` for the server thread to finish.
    fn finished_within(&mut self, limit: Duration) -> bool {
        let deadline = Instant::now() + limit;
        while Instant::now() < deadline {
            if self.handle.as_ref().is_some_and(JoinHandle::is_finished) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        return false;
    }
}

impl Drop for TestServer {
    /// Ask the server to stop and join it,
    /// so no test leaves a thread or a socket behind.
    fn drop(&mut self) {
        if let Ok(mut connection) = connect(&self.client()) {
            connection.shutdown();
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// The run parameters for a mode with a config path.
fn run_params(root: &Path, mode: &str) -> Value {
    let config = root.join("file-enforcer.config.jsonc");
    return object(&[("mode", Value::from(mode)), ("configPath", Value::from(config.display().to_string()))]);
}

#[test]
fn a_client_handshakes_runs_write_and_check_and_shuts_the_server_down() {
    let runtime = short_runtime_dir();
    let dir = scratch_dir();
    write_files(dir.path(), &[("file-enforcer.config.jsonc", r#"{"rules": [{"kind": "concat", "id": "c", "output": "out.txt", "parts": ["x"]}]}"#)]);
    let mut server = TestServer::start(dir.path(), runtime.path(), Duration::from_secs(30));
    let mut connection = connect(&server.client()).expect("the handshake succeeds");
    let mut check_sink = CollectingSink::default();
    assert_eq!(connection.request("run", run_params(dir.path(), "check"), &mut check_sink), 1);
    assert!(check_sink.events.iter().any(|event| return event.code == "drift-missing"));
    let mut write_sink = CollectingSink::default();
    assert_eq!(connection.request("run", run_params(dir.path(), "write"), &mut write_sink), 0);
    assert_eq!(read_text(dir.path(), "out.txt"), "x");
    assert!(connection.shutdown().is_some());
    assert!(server.finished_within(Duration::from_secs(5)), "the server exits after shutdown");
    assert!(!server.socket.exists(), "the socket file is removed on exit");
}

#[test]
fn a_handshake_with_another_build_is_refused_and_the_server_drains() {
    let runtime = short_runtime_dir();
    let dir = scratch_dir();
    let mut server = TestServer::start(dir.path(), runtime.path(), Duration::from_secs(30));
    let mut stream = UnixStream::connect(&server.socket).expect("connect");
    let worktree = crate::identity::worktree_identity(dir.path()).expect("identity");
    let pairs = [
        ("protocolVersion", Value::from(PROTOCOL_VERSION)),
        ("buildId", Value::from("some-other-build")),
        ("worktree", worktree),
    ];
    stream.write_all(request_line(1, "handshake", object(&pairs)).as_bytes()).expect("send");
    let mut line = String::new();
    BufReader::new(stream.try_clone().expect("clone")).read_line(&mut line).expect("reply");
    let reply: Value = serde_json::from_str(&line).expect("json");
    assert_eq!(reply["error"]["code"], -32001);
    assert_eq!(reply["error"]["data"]["reason"], "the build identity differs");
    assert!(server.finished_within(Duration::from_secs(5)), "a mismatch makes the server drain and exit");
}

#[test]
fn a_request_before_the_handshake_is_refused() {
    let runtime = short_runtime_dir();
    let dir = scratch_dir();
    let server = TestServer::start(dir.path(), runtime.path(), Duration::from_secs(30));
    let mut stream = UnixStream::connect(&server.socket).expect("connect");
    stream.write_all(request_line(1, "run", Value::Null).as_bytes()).expect("send");
    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line).expect("reply");
    assert!(line.contains("the first request must be the handshake"), "{line}");
}

#[test]
fn an_idle_server_exits_and_removes_its_socket() {
    let runtime = short_runtime_dir();
    let dir = scratch_dir();
    let mut server = TestServer::start(dir.path(), runtime.path(), Duration::from_millis(300));
    assert!(server.finished_within(Duration::from_secs(5)));
    assert!(!server.socket.exists());
}

#[test]
fn a_server_with_an_open_connection_is_not_idle() {
    let runtime = short_runtime_dir();
    let dir = scratch_dir();
    let mut server = TestServer::start(dir.path(), runtime.path(), Duration::from_millis(200));
    let _connection = connect(&server.client()).expect("connect");
    assert!(!server.finished_within(Duration::from_millis(900)), "an open connection keeps the server alive");
}

#[test]
fn a_server_exits_when_its_worktree_root_disappears() {
    let runtime = short_runtime_dir();
    let dir = scratch_dir();
    let root = dir.path().join("tree");
    std::fs::create_dir_all(&root).expect("root");
    let mut server = TestServer::start(&root, runtime.path(), Duration::from_secs(60));
    std::fs::remove_dir_all(&root).expect("remove the root");
    assert!(server.finished_within(Duration::from_secs(5)), "the server must not outlive its worktree");
}

#[test]
fn a_refused_socket_file_is_verified_stale_and_removed_before_a_start_is_attempted() {
    let runtime = short_runtime_dir();
    let socket = runtime.path().join("stale.sock");
    drop(UnixListener::bind(&socket).expect("bind"));
    assert!(socket.exists(), "closing a listener leaves the socket file behind");
    let launch = ServerLaunch { program: PathBuf::from("/bin/true"), args: Vec::new() };
    let outcome = start_server_if_needed(&socket, &launch, Instant::now() + Duration::from_secs(2));
    assert!(outcome.is_err(), "/bin/true never listens, so the start fails");
    assert!(!socket.exists(), "the stale socket file was removed after the refused connection proved it dead");
}

#[test]
fn a_live_server_is_not_replaced_by_a_start_attempt() {
    let runtime = short_runtime_dir();
    let dir = scratch_dir();
    let server = TestServer::start(dir.path(), runtime.path(), Duration::from_secs(30));
    let launch = ServerLaunch { program: PathBuf::from("/bin/false"), args: Vec::new() };
    let outcome = start_server_if_needed(&server.socket, &launch, Instant::now() + Duration::from_secs(2));
    assert_eq!(outcome, Ok(StartOutcome::AlreadyRunning));
    assert!(server.socket.exists());
}

#[test]
fn a_socket_path_over_the_kernel_limit_is_rejected_with_a_named_diagnostic() {
    let long_directory = PathBuf::from(format!("/{}", "a".repeat(MAX_SOCKET_PATH_BYTES)));
    let error = socket_path(&long_directory, "0123456789abcdef").expect_err("too long");
    assert_eq!(error.code, "socket-path-too-long");
    let runtime = short_runtime_dir();
    assert!(socket_path(runtime.path(), "0123456789abcdef").is_ok(), "a short path is accepted");
}

#[test]
fn an_unset_or_unusable_runtime_directory_falls_back_to_a_verified_owner_only_one() {
    let base = scratch_dir();
    let unusable = scratch_dir();
    std::fs::set_permissions(unusable.path(), std::fs::Permissions::from_mode(0o755)).expect("chmod");
    let from_unset = runtime_directory(None, base.path()).expect("fallback when unset");
    assert!(from_unset.starts_with(base.path()));
    let from_unusable = runtime_directory(Some(unusable.path().to_path_buf()), base.path()).expect("fallback when unusable");
    assert_eq!(from_unusable, from_unset, "an unusable XDG directory is skipped");
    let mode = std::fs::metadata(&from_unset).expect("exists").permissions().mode() & 0o777;
    assert_eq!(mode, 0o700);
    std::fs::set_permissions(unusable.path(), std::fs::Permissions::from_mode(0o700)).expect("chmod");
    let from_good = runtime_directory(Some(unusable.path().to_path_buf()), base.path()).expect("xdg");
    assert_eq!(from_good, unusable.path().join("file-enforcer"), "a usable XDG directory is used");
}
