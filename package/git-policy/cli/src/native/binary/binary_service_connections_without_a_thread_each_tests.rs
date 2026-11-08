//! What:
//!  The connections the commit service holds open,
//! counted against the threads of the service process.
//! Why:
//!  The service runs on tokio (`doc/planning/git-policy-cli-full-tokio.md`,
//! step T10):
//! a connection is a task,
//! so many clients that wait on the service cost memory and no thread each,
//! where a thread per connection spent one of the process's limited threads on every client and failed the connection when the
//! system refused one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const before = threadsOf(service); const held = await connectMany(32); expect(threadsOf(service) - before).toBeLessThan(16);
//! ```

use super::support::{Fixture, fixture};
use git_policy_cli::service_protocol::{PROTOCOL_VERSION, ServiceMessage, decode_service};
use std::io::{BufRead, BufReader};
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixStream};
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};

/// The longest a control waits for anything:
/// far beyond what a healthy run needs,
/// so only a hang reaches it.
const PATIENCE: Duration = Duration::from_secs(60);

/// How many clients the control keeps waiting on the service at once.
const CONNECTIONS: usize = 32;

/// A service process that is stopped when the control ends,
/// whether it passed or panicked.
struct Held(Child);

impl Drop for Held {
    fn drop(&mut self) {
        let _killed = self.0.kill();
        let _waited = self.0.wait();
    }
}

/// Starts a service on the abstract address `name` and waits until it listens.
fn start(fixture: &Fixture, name: &str) -> Held {
    let child: Child = super::support::service_command()
        .args(["--journal"])
        .arg(fixture.root.join("journal"))
        .args(["--abstract", name])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("the service starts");
    let began: Instant = Instant::now();
    while connect(name).is_err() {
        assert!(began.elapsed() < PATIENCE, "the service never listened");
        std::thread::sleep(Duration::from_millis(10));
    }
    return Held(child);
}

/// A connection to the service.
fn connect(name: &str) -> std::io::Result<UnixStream> {
    let socket: SocketAddr = SocketAddr::from_abstract_name(name.as_bytes())?;
    return UnixStream::connect_addr(&socket);
}

/// The number of threads the process `pid` has,
/// read from its status.
fn threads_of(pid: u32) -> usize {
    let status: String = std::fs::read_to_string(format!("/proc/{pid}/status")).expect("the status of the service");
    let line: &str = status.lines().find(|line| return line.starts_with("Threads:")).expect("a Threads line");
    return line.trim_start_matches("Threads:").trim().parse().expect("a thread count");
}

/// Reads the greeting a connection is given first.
fn greeting(stream: &UnixStream) -> ServiceMessage {
    let mut line: String = String::new();
    BufReader::new(stream).read_line(&mut line).expect("the service greets");
    return decode_service(line.trim_end_matches('\n')).expect("a service message");
}

/// Many connections that wait on the service,
/// each greeted,
/// add far fewer threads than connections:
/// a connection is a task and not a thread.
#[test]
fn connections_held_open_cost_the_service_no_thread_each() {
    let fixture: Fixture = fixture("service-connections-tasks");
    let name: String = format!("service-connections-tasks-{}", std::process::id());
    let service: Held = start(&fixture, name.as_str());
    // One connection first, so the threads the service starts for any first connection (a runtime's workers) are counted already.
    let first: UnixStream = connect(name.as_str()).expect("the first connection");
    assert!(matches!(greeting(&first), ServiceMessage::Welcome { version, .. } if version == PROTOCOL_VERSION));
    let before: usize = threads_of(service.0.id());
    let mut held: Vec<UnixStream> = Vec::new();
    for _ in 0..CONNECTIONS {
        let stream: UnixStream = connect(name.as_str()).expect("a connection");
        assert!(matches!(greeting(&stream), ServiceMessage::Welcome { .. }), "every connection is greeted while the others wait");
        held.push(stream);
    }
    let growth: usize = threads_of(service.0.id()) - before;
    assert!(growth < CONNECTIONS / 2, "{CONNECTIONS} waiting connections added {growth} threads: a connection must not hold a thread");
    drop(held);
    drop(first);
    super::commit::remove_settled(&fixture);
}
