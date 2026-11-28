//! What:
//!  The commit service plan step 21's queued arm starts,
//!  the address it listens on,
//!  and the readings of the queue it drains.
//! Why:
//!  The queued repair's whole claim is that the add returns before the tools run
//!  and that the service runs them in the worktree the record names,
//!  so the arm starts the shipped service where the wrapper's own route would
//!  find it
//!  (`landing/service_route.rs`,
//!  `choose_route`) and reads the queue's own states afterwards.
//!  The arm itself is `binary_zfs_step_21_queued_arm.rs`'s.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const service = TheService.start(fixture); await waitUntilSettled(fixture); service.stop();
//! ```

#![cfg(target_os = "linux")]

use super::zfs_step_21_guest_fixture::Fixture;
use super::zfs_step_21_guest_layout::WRAPPER;
use git_policy_cli::service::service_instance::{
    instance_of,
    socket_path,
};
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Child,
    Command,
    Stdio,
};
use std::time::{
    Duration,
    Instant,
};

/// How long the service may take to listen before the arm gives up.
const LISTEN_WAIT: Duration = Duration::from_secs(60);

/// How long the service may take to drain the records an add handed it.
const DRAIN_WAIT: Duration = Duration::from_secs(180);

/// How often the arm looks at the socket and at the queue.
const POLL: Duration = Duration::from_millis(20);

/// The folder the service keeps its journal in.
const JOURNAL: &str = "/root/step21-journal";

/// The line the service writes where the arm can read it.
const SERVICE_LOG: &str = "/root/step21-service.log";

/// The program the service runs as its Git,
/// never the wrapper's own link.
const REAL_GIT: &str = "/usr/bin/git";

/// What:
///  One commit service the arm started,
///  and the socket it listens on.
/// Why:
///  The arm owns the process it started and ends it,
///  so no service outlives the arm that needed it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const service = { child, socket }; service.stop();
/// ```
pub(super) struct TheService {
    /// The process this arm started.
    child: Child,
    /// The socket it listens on.
    socket: PathBuf,
}

impl TheService {
    /// Start one service for the fixture's repository,
    /// at the address the wrapper's own route computes.
    pub(super) fn start(fixture: &Fixture) -> TheService {
        let runtime: PathBuf = fixture.runtime_folder();
        let instance: String = instance_of(fixture.common().as_path());
        let socket: PathBuf = socket_path(runtime.as_path(), instance.as_str());
        if let Some(parent) = socket.parent() {
            std::fs::create_dir_all(parent).expect("the socket's folder");
        }
        let _stale: std::io::Result<()> = std::fs::remove_file(socket.as_path());
        std::fs::create_dir_all(JOURNAL).expect("the journal folder");
        let log: std::fs::File = std::fs::File::create(SERVICE_LOG).expect("the service's log");
        let kept: std::fs::File = log.try_clone().expect("the second handle of the log");
        let mut command_line: Command = Command::new(WRAPPER);
        command_line
            .args([
                "monochromatic-git-policy-cli",
                "service",
                "run",
                "--path",
                socket.to_str().expect("a text path"),
                "--instance",
                instance.as_str(),
                "--journal",
                JOURNAL,
                "--git",
                REAL_GIT,
                "--idle-seconds",
                "600",
            ])
            .current_dir(fixture.root())
            .env_clear()
            .stdin(Stdio::null())
            .stdout(Stdio::from(log))
            .stderr(Stdio::from(kept));
        for (key, value) in fixture.environment(Path::new("/usr/bin")) {
            command_line.env(key, value);
        }
        let child: Child = command_line.spawn().expect("the service starts");
        let service: TheService = TheService {
            child,
            socket,
        };
        service.wait_until_listening();
        return service;
    }

    /// Wait until the socket is bound,
    /// and refuse with the service's own log when it never is.
    fn wait_until_listening(&self) {
        let started: Instant = Instant::now();
        while started.elapsed() < LISTEN_WAIT {
            if self.socket.exists() {
                println!(
                    "STEP21 service listening socket-bytes={} listen-ms={}",
                    self.socket.as_os_str().len(),
                    started.elapsed().as_millis()
                );
                return;
            }
            std::thread::sleep(POLL);
        }
        panic!(
            "the service never listened on {}: {}",
            self.socket.display(),
            the_service_log()
        );
    }

    /// End the service this arm started,
    /// and take its socket away.
    pub(super) fn stop(self) {
        let mut child: Child = self.child;
        let _signalled: std::io::Result<()> = child.kill();
        let _reaped: std::io::Result<std::process::ExitStatus> = child.wait();
        let _gone: std::io::Result<()> = std::fs::remove_file(self.socket.as_path());
    }
}

/// The service's own log,
/// bounded to its last lines.
fn the_service_log() -> String {
    let text: String = std::fs::read_to_string(SERVICE_LOG).unwrap_or_default();
    let lines: Vec<&str> = text.lines().collect();
    let from: usize = lines.len().saturating_sub(20);
    return lines[from..].join("\n");
}

/// The queue's own states for the fixture's repository.
fn the_queue(fixture: &Fixture) -> PathBuf {
    return fixture.home_records().join("work");
}

/// What:
///  Whether one record of each kind the add handed over reached `done`.
/// Why:
///  A record in `done` is the service's own answer that the work ran,
///  and a record still in `pending` or `claimed` is a service that did not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const drained = ['relink-', 'prewarm-'].every((kind) => readdir(done).some((name) => name.startsWith(kind)));
/// ```
fn the_records_settled(fixture: &Fixture) -> bool {
    let done: PathBuf = the_queue(fixture).join("done");
    let names: Vec<String> = std::fs::read_dir(done.as_path())
        .map(|entries| {
            return entries
                .filter_map(|entry| return entry.ok())
                .map(|entry| return entry.file_name().to_string_lossy().to_string())
                .collect::<Vec<String>>();
        })
        .unwrap_or_default();
    return ["relink-", "prewarm-"]
        .iter()
        .all(|kind| return names.iter().any(|name| return name.starts_with(kind)));
}

/// What:
///  The records each state of the queue holds,
///  for a log that says what the service did.
/// Why:
///  A round that fails names the state it found instead of only the wait that
///  ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const states = ['pending', 'claimed', 'done', 'failed'].map((state) => `${state}=${readdirSync(join(queue, state)).length}`);
/// ```
fn the_queue_states(fixture: &Fixture) -> String {
    let mut counted: Vec<String> = Vec::new();
    for state in ["pending", "claimed", "done", "failed"] {
        let folder: PathBuf = the_queue(fixture).join(state);
        let count: usize = std::fs::read_dir(folder.as_path())
            .map(|entries| return entries.count())
            .unwrap_or(0);
        let names: Vec<String> = std::fs::read_dir(folder.as_path())
            .map(|entries| {
                return entries
                    .filter_map(|entry| return entry.ok())
                    .map(|entry| return entry.file_name().to_string_lossy().to_string())
                    .collect::<Vec<String>>();
            })
            .unwrap_or_default();
        counted.push(format!("{state}={count}[{}]", names.join(",")));
    }
    return counted.join(" ");
}

/// Wait until the service has run both records the add handed over.
pub(super) fn wait_until_settled(fixture: &Fixture) {
    let started: Instant = Instant::now();
    while started.elapsed() < DRAIN_WAIT {
        if the_records_settled(fixture) {
            println!(
                "STEP21 queued drained-ms={} states={}",
                started.elapsed().as_millis(),
                the_queue_states(fixture)
            );
            return;
        }
        std::thread::sleep(POLL);
    }
    panic!(
        "the service never settled the records the add handed over: {}; {}",
        the_queue_states(fixture),
        the_service_log()
    );
}
