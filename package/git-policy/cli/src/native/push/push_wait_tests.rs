//! Controls for the push clock's wait on a slow remote, on probe processes that hold a real TCP connection to a listener the
//!  test controls, some with a process below them under the name of a Git subcommand (`send-pack`, `pack-objects`).
//! Why:
//!  A push whose remote has not answered shows nothing to `/proc/<pid>/io`, the processor or the process state. The clock must
//!  keep waiting while the connection to the remote is open, whatever phase the push is in and however long it takes, count the
//!  bytes of the socket as progress, and stop as before when the connection is closed and when there is no connection at all.
//!  Each case is a real process and a real connection.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const clock = new Clock(probe.pid, { kind: 'progress', stallMs }); expect(drive(clock, 3 * stallMs)).toBeUndefined();
//! ```
#![cfg(unix)]

use crate::bounded_push::{Patience, StopCause};
use crate::push_progress::{Clock, Meter, SAMPLE_INTERVAL};
use std::net::{TcpListener, TcpStream};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// The name the controls give the probe test in order to run it as a process of its own.
const PROBE_NAME: &str = "push::push_wait_tests::tcp_probe";

/// The variable that holds the port the probe connects to; without it the probe does nothing.
const PORT_VARIABLE: &str = "CLI_GIT_TCP_PROBE_PORT";

/// The variable that says what the probe does after it connected: `connected` (hold the connection and say nothing),
///  `trickle` (send one byte at a time), or `idle` (hold nothing, only exist).
const MODE_VARIABLE: &str = "CLI_GIT_TCP_PROBE_MODE";

/// The variable that names a Git subcommand the probe starts a process under, as `git <subcommand>` reads in a process list.
const CHILD_VARIABLE: &str = "CLI_GIT_TCP_PROBE_CHILD";

/// The stall window of the controls: several sampling intervals, short enough to wait for.
const STALL_WINDOW: Duration = Duration::from_millis(1500);

/// How many stall windows a control lets pass while the clock must not stop.
const WINDOWS_WAITED: u32 = 3;

/// The pause between two bytes of the trickling probe, much shorter than the stall window.
const TRICKLE_PAUSE: Duration = Duration::from_millis(200);

/// How many bytes the trickling probe sends: long enough to outlast every control that uses it.
const TRICKLE_SENDS: u32 = 100;

/// Pause between two looks of a control, shorter than every limit it uses.
const LOOK: Duration = Duration::from_millis(50);

/// How long a control waits for something that must happen, so a broken clock fails it and never holds it.
const PATIENCE: Duration = Duration::from_secs(20);

/// How long the probe sleeps when it only holds what it holds: far longer than any control, ended by the control.
const HOLD: Duration = Duration::from_secs(600);

/// A probe process the control started, which `test_support::spawn_owned` ends with the test thread whatever the outcome.
struct Owned(Child);

impl Owned {
    /// The process number of the probe.
    fn pid(&self) -> u32 {
        return self.0.id();
    }
}

/// Ask the kernel to end this process when the process that started it ends (`prctl(2)`, `PR_SET_PDEATHSIG`).
fn die_with_the_parent() {
    // SAFETY: `prctl` with `PR_SET_PDEATHSIG` only sets a signal for this process to receive; it reads and writes no memory.
    unsafe {
        libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL);
    }
}

/// The process the controls start when they need a tree that holds a TCP connection: not a test of its own, it does nothing
///  unless the variable names a port. It connects (`std` sends and receives with `send` and `recv`, which `rchar` and `wchar`
///  do not count), starts the process it is asked to start under a Git subcommand's name, and then holds or trickles.
#[test]
fn tcp_probe() {
    let Some(port) = std::env::var(PORT_VARIABLE).ok().and_then(|text| { return text.parse::<u16>().ok(); }) else {
        return;
    };
    die_with_the_parent();
    let mode: String = std::env::var(MODE_VARIABLE).unwrap_or_default();
    if mode == "idle" {
        // A process that only exists, under the name of a Git subcommand; it holds nothing.
        std::thread::sleep(HOLD);
        return;
    }
    let mut stream: TcpStream = TcpStream::connect(("127.0.0.1", port)).expect("the control's listener");
    let mut started: Vec<Child> = Vec::new();
    if let Ok(subcommand) = std::env::var(CHILD_VARIABLE) {
        started.push(spawn_named_child(subcommand.as_str()));
    }
    if mode == "trickle" {
        for _ in 0..TRICKLE_SENDS {
            std::io::Write::write_all(&mut stream, &[0]).expect("send one byte");
            std::thread::sleep(TRICKLE_PAUSE);
        }
    }
    std::thread::sleep(HOLD);
    drop(stream);
    drop(started);
}

/// A process that reads in a process list as `git <subcommand> ...`: this test binary, asked to only exist.
fn spawn_named_child(subcommand: &str) -> Child {
    let test_binary: std::path::PathBuf = crate::test_support::this_test_binary().expect("this test binary");
    return Command::new(test_binary)
        .arg0("git")
        .arg(subcommand)
        .args(["--exact", PROBE_NAME, "--nocapture", "--test-threads", "1"])
        .env(PORT_VARIABLE, "0")
        .env(MODE_VARIABLE, "idle")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("start the named child");
}

/// Start the probe against a listener's port, in a mode, with a child under a Git subcommand's name or none.
fn start_probe(port: u16, mode: &str, child: Option<&str>) -> Owned {
    let test_binary: std::path::PathBuf = crate::test_support::this_test_binary().expect("this test binary");
    let mut command: Command = Command::new(test_binary);
    command
        .args(["--exact", PROBE_NAME, "--nocapture", "--test-threads", "1"])
        .env(PORT_VARIABLE, port.to_string())
        .env(MODE_VARIABLE, mode)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(subcommand) = child {
        command.env(CHILD_VARIABLE, subcommand);
    }
    return Owned(crate::test_support::spawn_owned(&mut command).expect("start the probe"));
}

/// The connection the probe made, taken from the listener.
fn accepted(listener: &TcpListener) -> TcpStream {
    listener.set_nonblocking(true).expect("a non-blocking listener");
    let began: Instant = Instant::now();
    loop {
        if let Ok((stream, _)) = listener.accept() {
            return stream;
        }
        assert!(began.elapsed() < PATIENCE, "the probe never connected");
        std::thread::sleep(LOOK);
    }
}

/// A loopback listener and its port.
fn listening() -> (TcpListener, u16) {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:0").expect("a loopback listener");
    let port: u16 = listener.local_addr().expect("its address").port();
    return (listener, port);
}

/// A push clock with the window of the controls.
fn push_clock(pid: u32) -> Clock {
    return Clock::start(pid, Patience::Progress { stall: STALL_WINDOW });
}

/// Ask the clock until it stops the process or `span` passes.
fn drive(clock: &mut Clock, span: Duration) -> Option<StopCause> {
    let began: Instant = Instant::now();
    while began.elapsed() < span {
        if let Some(cause) = clock.check() {
            return Some(cause);
        }
        std::thread::sleep(LOOK);
    }
    return None;
}

/// A tree that is stopped only by the stall window stops at it, with the window in the report.
fn assert_stalled_at_the_window(cause: Option<StopCause>, began: Instant) {
    let Some(StopCause::Stalled(report)) = cause else { panic!("a stall, not another limit or none") };
    assert_eq!(report.limit, STALL_WINDOW);
    assert!(began.elapsed() >= STALL_WINDOW, "{:?}", began.elapsed());
}

/// The bytes of a connected socket are progress when nothing else moves: a probe that only `send`s a byte at a time shows no
///  `rchar`, no `wchar` and no processor time worth a tick, and the meter must still count what its socket carried.
#[test]
fn the_bytes_of_a_socket_are_progress_when_nothing_else_moves() {
    let (listener, port): (TcpListener, u16) = listening();
    let probe: Owned = start_probe(port, "trickle", None);
    let _held: TcpStream = accepted(&listener);
    let mut meter: Meter = Meter::new();
    let pid: i32 = i32::try_from(probe.pid()).expect("pid");
    let began: Instant = Instant::now();
    while meter.socket_bytes() == 0 {
        assert!(began.elapsed() < PATIENCE, "the bytes the probe sent on its socket never counted");
        std::thread::sleep(SAMPLE_INTERVAL);
        let _progress: bool = meter.sample(pid);
    }
    assert!(meter.socket_bytes() > 0);
}

/// A push that holds its connection (a `send-pack` below it) is waiting for the remote: the clock does not stop it, however many
///  windows pass, and it reports the wait.
#[test]
fn a_push_that_sent_its_pack_and_holds_its_connection_is_waiting_not_stalled() {
    let (listener, port): (TcpListener, u16) = listening();
    let probe: Owned = start_probe(port, "connected", Some("send-pack"));
    let _held: TcpStream = accepted(&listener);
    let mut clock: Clock = push_clock(probe.pid());
    let stopped: Option<StopCause> = drive(&mut clock, STALL_WINDOW * WINDOWS_WAITED);
    assert!(stopped.is_none(), "the connection is open: the push waits");
    let waited = clock.wait_report().expect("the wait is reported once the window has run out");
    assert!(waited.silent >= STALL_WINDOW, "{waited:?}");
}

/// The connection closing ends the wait: the stall window applies again from that moment, and the push is stopped as before.
#[test]
fn a_wait_ends_when_the_connection_closes_and_the_stall_window_runs_again() {
    let (listener, port): (TcpListener, u16) = listening();
    let probe: Owned = start_probe(port, "connected", Some("send-pack"));
    let held: TcpStream = accepted(&listener);
    let mut clock: Clock = push_clock(probe.pid());
    assert!(drive(&mut clock, STALL_WINDOW * WINDOWS_WAITED).is_none(), "waiting while the connection is open");
    let closed: Instant = Instant::now();
    drop(held);
    let cause: Option<StopCause> = drive(&mut clock, PATIENCE);
    let Some(StopCause::Stalled(_)) = cause else { panic!("a stall once the connection is closed") };
    assert!(closed.elapsed() + SAMPLE_INTERVAL >= STALL_WINDOW, "a full window after the close, not before: {:?}", closed.elapsed());
}

/// A connection that has not yet seen a pack sent (no `send-pack`, no `pack-objects`: a server that has not answered the first
///  request) is an open connection all the same: the push waits for the remote and is not stopped for time.
#[test]
fn a_connected_push_is_waited_for_before_any_pack_was_sent() {
    let (listener, port): (TcpListener, u16) = listening();
    let probe: Owned = start_probe(port, "connected", None);
    let _held: TcpStream = accepted(&listener);
    let mut clock: Clock = push_clock(probe.pid());
    let stopped: Option<StopCause> = drive(&mut clock, STALL_WINDOW * WINDOWS_WAITED);
    assert!(stopped.is_none(), "the connection is open: the push waits for the remote");
}

/// A pack still being made (a `pack-objects` below the push) with the connection open is waited for as well: what the code can
///  observe is the open connection, not the phase of the push.
#[test]
fn a_connected_push_is_waited_for_while_its_pack_is_made() {
    let (listener, port): (TcpListener, u16) = listening();
    let probe: Owned = start_probe(port, "connected", Some("pack-objects"));
    let _held: TcpStream = accepted(&listener);
    let mut clock: Clock = push_clock(probe.pid());
    let stopped: Option<StopCause> = drive(&mut clock, STALL_WINDOW * WINDOWS_WAITED);
    assert!(stopped.is_none(), "the connection is open: the push waits");
}

/// A connected push that has waited far past the stall window is not stopped for time: there is no total bound.
#[test]
fn a_connected_push_is_not_stopped_for_time_however_long_it_waits() {
    let (listener, port): (TcpListener, u16) = listening();
    let probe: Owned = start_probe(port, "connected", Some("send-pack"));
    let _held: TcpStream = accepted(&listener);
    let mut clock: Clock = push_clock(probe.pid());
    let stopped: Option<StopCause> = drive(&mut clock, STALL_WINDOW * WINDOWS_WAITED * 2);
    assert!(stopped.is_none(), "an open connection is waited for however long it takes");
}

/// A tree with no connection at all (a push to a local path, whose hook is a child) is a stall, as before.
#[test]
fn a_tree_with_no_connection_is_a_stall() {
    let owned: Owned = Owned(crate::test_support::spawn_owned(Command::new("sleep").arg("600").stdin(Stdio::null())).expect("sleep"));
    let mut clock: Clock = push_clock(owned.pid());
    let began: Instant = Instant::now();
    let cause: Option<StopCause> = drive(&mut clock, PATIENCE);
    assert_stalled_at_the_window(cause, began);
}
