//! Controls for the least traffic a push's connection must carry,
//! on real sockets and real processes,
//! through the push's own clock.
//! Why:
//!  A server that trickles a byte now and then used to be waited for for ever:
//! every byte reset both silence windows
//!  (the audit's F14,
//! and its F3 for the keepalive a server sends while its hook runs).
//! These controls pin the four cases the rule
//!  must tell apart,
//! each with a real connection:
//!       a trickle below the floor is silence,
//! and the connected window ends it;
//!       a transfer that meets the floor is work,
//! however slow it is,
//! and is not ended;
//!       and a window that carries no byte at all is left to the connected window exactly as before,
//!       so a
//!  server's silent check is not cut sooner.
//!  What processor time and a task in disk wait are to a push is not this rule's business:
//! they were work before it
//!  and are work still,
//! which `push_progress`'s own controls hold (a `dd` of 64 MB with `conv=fsync` is a
//!  tree in disk wait and is not stalled).
//!  The floor is a control's,
//! small enough to reach in seconds;
//! the shipped floor and the window are in
//!  `push_minimum_rate.rs` and `service/wait_ceilings.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const clock = Clock.start(holder.pid, { stall, })..withConnectedWindow(window).withMinimumRate(floor);
//! expect(drive(clock, window + margin)).toMatchObject({ window: 'connected' });
//! ```
#![cfg(unix)]

use crate::bounded_push::{Patience, StopCause, StopWindow};
use crate::push_progress::Clock;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// How long the holder may show nothing while its connection is open,
/// before the connected window ends it.
const CONNECTED_WINDOW: Duration = Duration::from_secs(2);

/// The window a run with no connection may show nothing for,
/// which must be at least one sampling interval (`SAMPLE_INTERVAL`,
/// 500 ms) or the stall rule fires between two looks and never sees the connection at all.
const STALL_WINDOW: Duration = Duration::from_secs(1);

/// The least traffic a control's connection must carry in one window:
/// 4096 bytes in the two second window,
/// about 2 KB/s.
/// Above the trickle of a byte every tenth of a second the controls send (20 bytes in the window),
/// and below the 5 KB/s transfer the control that means an honest push to pass moves.
const FLOOR: u64 = 4096;

/// The pause between two bytes of the trickling holder.
const TRICKLE_PAUSE: Duration = Duration::from_millis(100);

/// How long a control waits for something that must happen,
/// so a broken clock fails it and never holds it.
const PATIENCE: Duration = Duration::from_secs(20);

/// Pause between two looks of a control,
/// shorter than every limit it uses.
const LOOK: Duration = Duration::from_millis(50);

/// A holder process the control started,
/// ended when the control ends whatever the outcome.
struct Held(Child);

impl Drop for Held {
    /// End the holder and everything it started.
    fn drop(&mut self) {
        // The holder is its own process group (`hold`), so the signal reaches the shell's children as well; one left running
        //  would write into the fixture after the control ended. The group is named by the negative of its first number,
        //  and the call is the kernel's, so no program on `PATH` can change what it means.
        let group: i32 = -i32::try_from(self.0.id()).unwrap_or(i32::MAX);
        // SAFETY: `kill` with a signal number only sends that signal; it reads and writes no memory of this process.
        let _ended = unsafe { libc::kill(group, libc::SIGKILL) };
        let _reaped = self.0.wait();
    }
}

/// A loopback listener and its port.
fn listening() -> (TcpListener, u16) {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:0").expect("a loopback listener");
    let port: u16 = listener.local_addr().expect("its address").port();
    return (listener, port);
}

/// Run a shell script as the holder:
/// the process that holds the connection.
fn hold(script: &str) -> Held {
    let child: Child = Command::new("sh")
        .args(["-c", script])
        // A group of its own, so the control ends the shell and everything the shell started.
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("start the holder");
    return Held(child);
}

/// The connection the holder made,
/// taken from the listener.
fn accepted(listener: &TcpListener) -> TcpStream {
    listener.set_nonblocking(true).expect("a non-blocking listener");
    let began: Instant = Instant::now();
    loop {
        if let Ok((stream, _)) = listener.accept() {
            stream.set_nonblocking(true).expect("a non-blocking connection");
            return stream;
        }
        assert!(began.elapsed() < PATIENCE, "the holder never connected");
        std::thread::sleep(LOOK);
    }
}

/// Drain whatever the holder sent,
/// so a holder that writes more than the socket's buffers hold never blocks.
fn drain(stream: &mut TcpStream) {
    let mut sink: [u8; 65536] = [0; 65536];
    while matches!(stream.read(&mut sink), Ok(count) if count > 0) {}
}

/// What:
///  What a stop cause is,
///  in a control's own words,
///  since `StopCause` carries no `Debug`.
/// Why:
///   A failing control must say whether the clock stopped the run at all,
/// and by what,
/// without the release code
///  growing a derive it has no other use for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cause === undefined ? 'no stop' : `${cause.window} after ${cause.limit}s`
/// ```
fn described(cause: Option<StopCause>) -> String {
    return match cause {
        None => String::from("no stop"),
        Some(StopCause::Deadline) => String::from("the fixed deadline"),
        Some(StopCause::Flood) => String::from("the output cap"),
        Some(StopCause::ServiceStopped) => String::from("the service's own stop"),
        Some(StopCause::Stalled(report)) => {
            format!("a stop by {:?} after {} seconds, carrying {} bytes", report.window, report.limit.as_secs(), report.carried)
        }
    };
}

/// A clock with the controls' windows and a floor.
fn clock(pid: u32, floor: u64) -> Clock {
    return Clock::start(pid, Patience::Progress { stall: STALL_WINDOW })
        .with_connected_window(CONNECTED_WINDOW)
        .with_minimum_rate(floor);
}

/// What:
///  Ask the clock until it stops the holder or `span` passes,
/// draining the connection as it goes.
/// Why:
///   A holder that writes into a socket nobody reads fills the socket's buffers and stops writing,
/// which is a holder that has stopped trickling,
///  not a clock that failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// while (span not passed) { drain(stream); if (cause = clock.check()) return cause; await sleep(LOOK); }
/// ```
fn drive(clock: &mut Clock, stream: &mut TcpStream, span: Duration) -> Option<StopCause> {
    let began: Instant = Instant::now();
    while began.elapsed() < span {
        drain(stream);
        if let Some(cause) = clock.check() {
            return Some(cause);
        }
        std::thread::sleep(LOOK);
    }
    return None;
}

/// A trickle below the floor is not progress:
/// the connected window ends the run,
/// and the report says what the connection carried,
/// because a byte moved and the words must not say nothing did.
#[test]
fn a_trickle_below_the_floor_is_ended_at_the_connected_window() {
    let (listener, port): (TcpListener, u16) = listening();
    let hold_script: String = format!(
        "exec 3<>/dev/tcp/127.0.0.1/{port}; while :; do printf x >&3; sleep {}; done",
        TRICKLE_PAUSE.as_secs_f64()
    );
    let holder: Held = hold(hold_script.as_str());
    let mut stream: TcpStream = accepted(&listener);
    let mut watched: Clock = clock(holder.0.id(), FLOOR);
    let began: Instant = Instant::now();
    let cause: Option<StopCause> = drive(&mut watched, &mut stream, PATIENCE);
    let Some(StopCause::Stalled(report)) = cause else { panic!("the trickle was not ended: {}", described(cause)) };
    assert_eq!(report.window, StopWindow::Connected, "the connected window ended it");
    assert_eq!(report.limit, CONNECTED_WINDOW);
    assert!(began.elapsed() >= CONNECTED_WINDOW, "not before a whole window: {:?}", began.elapsed());
    assert!(began.elapsed() < CONNECTED_WINDOW + PATIENCE, "and not long after it: {:?}", began.elapsed());
    assert!(report.carried > 0, "the bytes the connection carried are named: {}", report.carried);
    assert_eq!(report.minimum, FLOOR);
}

/// The same trickle where the transport's bytes count by themselves (no floor,
/// which is what every run but a push has) is
/// progress:
/// the run is not ended,
/// which is the behaviour the minimum rate answers for the push.
#[test]
fn the_same_trickle_without_a_floor_is_progress() {
    let (listener, port): (TcpListener, u16) = listening();
    let hold_script: String = format!(
        "exec 3<>/dev/tcp/127.0.0.1/{port}; while :; do printf x >&3; sleep {}; done",
        TRICKLE_PAUSE.as_secs_f64()
    );
    let holder: Held = hold(hold_script.as_str());
    let mut stream: TcpStream = accepted(&listener);
    let mut watched: Clock = clock(holder.0.id(), 0);
    let cause: Option<StopCause> = drive(&mut watched, &mut stream, CONNECTED_WINDOW * 2);
    assert!(cause.is_none(), "a byte of the transport is progress where no floor is set: {}", described(cause));
}

/// What:
///  A transfer that meets the floor is work,
///  however slowly it moves,
///  which is the case a person on a slow line meets.
/// Why the burst comes after a pause:
///   A socket carries no progress on the look that first sees it,
/// because that look's counters stand for the side's own handshake and request
///  (`Meter::observe_sockets`),
/// so a control that wants the transfer itself counted lets the clock see the connection first.
///  The pause is also the point:
/// a connection that showed nothing for a while and then moves above the floor is working,
/// and a
///  rule that judged it by the silence before would end it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const clock = clockOf(holder, FLOOR); expect(drive(clock, 2 * window)).toBeUndefined();
/// ```
#[test]
fn a_transfer_that_meets_the_floor_is_work_however_slow() {
    let (listener, port): (TcpListener, u16) = listening();
    // A pause longer than the stall window, then 1024 bytes every fifth of a second: about 5 KB/s, which is above the
    //  floor's 4096 bytes in the control's 2 second window, and far below any line a person would call fast.
    let holder: Held = hold(&format!(
        "exec 3<>/dev/tcp/127.0.0.1/{port}; sleep 1; while :; do head -c 1024 /dev/zero >&3; sleep 0.2; done"
    ));
    let mut stream: TcpStream = accepted(&listener);
    let mut watched: Clock = clock(holder.0.id(), FLOOR);
    let cause: Option<StopCause> = drive(&mut watched, &mut stream, CONNECTED_WINDOW * 2);
    assert!(cause.is_none(), "a transfer above the floor is work: {}", described(cause));
}

/// A window that carries no byte at all is left to the connected window,
/// which is the rule that bounds a server's silent check,
/// so the floor never ends such a phase sooner than silence alone does.
#[test]
fn a_window_with_no_bytes_is_bounded_by_the_connected_window_and_no_sooner() {
    let (listener, port): (TcpListener, u16) = listening();
    // The holder takes the connection and says nothing at all.
    let holder: Held = hold(&format!("exec 3<>/dev/tcp/127.0.0.1/{port}; while :; do sleep 0.1; done"));
    let mut stream: TcpStream = accepted(&listener);
    let floor: u64 = 1;
    let mut watched: Clock = clock(holder.0.id(), floor);
    let began: Instant = Instant::now();
    let cause: Option<StopCause> = drive(&mut watched, &mut stream, PATIENCE);
    let Some(StopCause::Stalled(report)) = cause else { panic!("a silent connection is not ended: {}", described(cause)) };
    assert_eq!(report.window, StopWindow::Connected);
    assert!(began.elapsed() >= CONNECTED_WINDOW, "the whole window, as before this rule: {:?}", began.elapsed());
    assert_eq!(report.carried, 0, "nothing was carried");
}
