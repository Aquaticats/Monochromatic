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
//!  Two of these controls state the clock their transfer is judged by,
//!  and the two whose subject is a stop read the host's clock,
//!  because a window judged by the host's clock measured how busy the host was rather than what the
//!  connection carried:
//! in the gate of 2026-11-27 the sender's own two tenths of a second between two chunks took two
//!  thirds of a second,
//! the window counted the 3072 bytes the sender had really offered of the 10240 it meant to send,
//!  and a transfer that was work was stopped
//! (`doc/troubleshooting/git-policy-cli-minimum-rate-control-measured-the-hosts-scheduling.md`,
//!  issue 6591).
//!  A host that is late can only make the stop those two controls wait for late,
//!  never early,
//! so their own bounds carry the whole of its lateness.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const clock = Clock.start(holder.pid, { stall, })..withConnectedWindow(window).withMinimumRate(floor);
//! expect(drive(clock, window + margin)).toMatchObject({ window: 'connected' });
//! ```
#![cfg(unix)]

use crate::bounded_push::{
    Patience,
    StopCause,
    StopWindow,
};
use crate::push_progress::Clock;
use crate::test_support::spawn_owned;
use std::io::{
    ErrorKind,
    Read,
};
use std::net::{
    TcpListener,
    TcpStream,
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
/// A control that waits for its own transfer's bytes waits this long for all of them,
/// which is a hundred times the two tenths of a second its sender waits between two chunks.
const PATIENCE: Duration = Duration::from_secs(20);

/// Pause between two looks of a control,
/// shorter than every limit it uses.
const LOOK: Duration = Duration::from_millis(50);

/// How many bytes a control reads from the connection at a time,
/// so a holder that sends more between two of the control's looks is taken in one read.
const CARRIED_AT_ONCE: usize = 65536;

/// What:
///  How many slices of its own clock a control divides one connected window into.
/// Why:
///  A control states its own clock one slice at a time,
///  and the slices are what makes the window a window:
/// a clock stated in whole windows would have one look per window and no room to see the bytes a
///  window carried arrive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SLICES_PER_WINDOW = 8
/// ```
const SLICES_PER_WINDOW: u32 = 8;

/// What:
///  How many whole windows a control that means a slow transfer states its clock over.
/// Why:
///  It is the span the two controls that read the host's clock drive for,
///  so the pair still says the same thing about the same stretch of the transfer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WINDOWS_A_TRANSFER_STATES = 2
/// ```
const WINDOWS_A_TRANSFER_STATES: u32 = 2;

/// What:
///  How many slices of its own clock a control that means a slow transfer states.
/// Why:
///  The whole drive in one number,
///  so a control that names it cannot drive for a different stretch than the one its doc names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SLICES_A_TRANSFER_STATES = WINDOWS_A_TRANSFER_STATES * SLICES_PER_WINDOW
/// ```
const SLICES_A_TRANSFER_STATES: u32 = WINDOWS_A_TRANSFER_STATES * SLICES_PER_WINDOW;

/// What:
///  The bytes the connection must carry before a control's own clock is told one slice of the window
///  has passed.
/// Why:
///  The floor's own share of one slice,
///  doubled,
///  so a window the control states carries twice the floor:
/// a slice the control lost to a host that woke it late is then not a window under the floor.
///  Derived from the floor and the slices rather than written as a number,
///  so a control that lowers the floor for its own fixture takes its slices with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BYTES_PER_SLICE = (2 * FLOOR) / SLICES_PER_WINDOW
/// ```
const BYTES_PER_SLICE: u64 = 2 * FLOOR / SLICES_PER_WINDOW as u64;

/// What:
///  The bytes a slice of the window means to a control whose floor is nothing,
///  where every byte of the transport is progress.
/// Why:
///  Such a control states one byte of the window per byte it has seen the connection carry,
///  so its clock runs at the rate its own trickle offers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BYTES_A_TRICKLE_SLICE_MEANS = 1
/// ```
const BYTES_A_TRICKLE_SLICE_MEANS: u64 = 1;

/// What:
///  The part of a connected window one slice of a control's own clock is,
///  the window divided evenly among its slices.
/// Why:
///   A control adds one slice of stated time each time the connection carried the bytes a slice means,
/// and the slices must add back up to the window the release judges,
///  or no window the control states would be the one the rule is about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sliceOfAWindow() { return CONNECTED_WINDOW / SLICES_PER_WINDOW }
/// ```
fn slice_of_a_window() -> Duration {
    // `Duration / u32` splits a length of time into that many equal parts, and the window is a whole number of
    //  milliseconds, so the split leaves nothing out.
    return CONNECTED_WINDOW / SLICES_PER_WINDOW;
}

/// What:
///  A holder process the control started,
///  ended and reaped when the control ends whatever the outcome.
/// Why:
///  `spawn_owned` gives the child a group of its own and hands that group to the test thread's fixtures,
///  so a control that panics cannot leave the holder running;
///  the child's own handle is still the control's to reap,
///  which is what keeps the run from carrying a finished child it never collected.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Held { child; [Symbol.dispose]() { kill(-this.child.pid, 'SIGKILL'); this.child.wait(); } }
/// ```
struct Held(Child);

impl Drop for Held {
    /// End the holder and everything it started.
    fn drop(&mut self) {
        // The holder is its own process group (`spawn_owned`), so the signal reaches the shell's children as well; one left
        //  running would write into the fixture after the control ended. The group is named by the negative of its first
        //  number, and the call is the kernel's, so no program on `PATH` can change what it means.
        let group: i32 = -i32::try_from(self.0.id()).unwrap_or(i32::MAX);
        // `spawn_owned` set the parent-death signal, so the child is gone already where a control panicked past this; the
        //  signal tells a group that had not noticed yet, and its failure is nothing to report.
        //
        // SAFETY: `kill` with a signal number only sends that signal; it reads and writes no memory of this process.
        let _ended = unsafe { libc::kill(group, libc::SIGKILL) };
        // `wait` collects the child this handle started, so no finished child stays for this process to reap later.
        let _reaped = self.0.wait();
    }
}

/// A loopback listener and its port.
fn listening() -> (TcpListener, u16) {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:0").expect("a loopback listener");
    let port: u16 = listener.local_addr().expect("its address").port();
    return (listener, port);
}

/// What:
///  Run a shell script as the holder:
///  the process that holds the connection.
/// Why:
///  The clock measures a tree under one root,
///  and a script of the control's own is a root whose whole tree this process ends.
///  The child is `spawn_owned`'s:
/// it leads a group of its own,
///  which the test thread's fixtures end however the control ends,
///  so no holder outlives the run that started it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hold(script) { return new Held(spawnOwned(['sh', '-c', script], { stdin: 'null', stdout: 'null', stderr: 'null' })) }
/// ```
fn hold(script: &str) -> Held {
    let mut command: Command = Command::new("sh");
    // `args` gives the script as the shell's one argument; the three null devices keep the child off this test's own
    //  streams, so nothing it says is read as the run's output and nothing holds a pipe open after it is gone.
    command
        .args(["-c", script])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // `spawn_owned` starts the child in a group of its own and hands that group to the test thread's fixtures; the
    //  `expect` fails the control, with the operating system's own words, where the child cannot start.
    let child: Child = spawn_owned(&mut command).expect("start the holder");
    return Held(child);
}

/// The connection the holder made,
/// taken from the listener.
fn accepted(listener: &TcpListener) -> TcpStream {
    listener
        .set_nonblocking(true)
        .expect("a non-blocking listener");
    let began: Instant = Instant::now();
    loop {
        if let Ok((stream, _)) = listener.accept() {
            stream
                .set_nonblocking(true)
                .expect("a non-blocking connection");
            return stream;
        }
        assert!(began.elapsed() < PATIENCE, "the holder never connected");
        std::thread::sleep(LOOK);
    }
}

/// Drain whatever the holder sent,
/// so a holder that writes more than the socket's buffers hold never blocks.
fn drain(stream: &mut TcpStream) {
    let mut sink: [u8; CARRIED_AT_ONCE] = [0; CARRIED_AT_ONCE];
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
            format!(
                "a stop by {:?} after {} seconds, carrying {} bytes",
                report.window,
                report.limit.as_secs(),
                report.carried
            )
        }
    };
}

/// A clock with the controls' windows and a floor.
fn clock(
    pid: u32,
    floor: u64,
) -> Clock {
    return Clock::start(
        pid,
        Patience::Progress {
            stall: STALL_WINDOW,
        },
    )
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
///  This is the drive of the controls whose subject is a stop:
/// they want the bytes out of the socket and do not care when they come.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// while (span not passed) { drain(stream); if (cause = clock.check()) return cause; await sleep(LOOK); }
/// ```
fn drive(
    clock: &mut Clock,
    stream: &mut TcpStream,
    span: Duration,
) -> Option<StopCause> {
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

/// What:
///  Read the connection until it has carried `want` bytes,
///  and fail the control with its own words where `began`'s patience runs out first.
/// Why:
///  A control that states its own clock has to know the connection carried a slice's bytes before it
///  can state that slice took a slice's time;
/// waiting for those bytes is the whole of the control's own wait,
///  and a sender that offers none must fail this control rather than hold the run for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let carried = 0; while (carried < want) { const read = stream.read(sink, { mayBlock: false }); carried += read; if (nothingYet) assert(Date.now() - began < PATIENCE); }
/// ```
fn carry_a_slice(
    stream: &mut TcpStream,
    sink: &mut [u8],
    want: u64,
    began: Instant,
) {
    let mut carried: u64 = 0;
    while carried < want {
        match stream.read(sink) {
            // A connection the holder closed carries nothing more, and no patience would change that.
            Ok(0) => panic!(
                "the sender closed the connection after {carried} of the {want} bytes a slice means"
            ),
            // `count` is how many bytes this read took; `u64::try_from` widens it, and the fallback can never be
            //  reached because a read is shorter than the buffer's own length.
            Ok(count) => carried += u64::try_from(count).unwrap_or(u64::MAX),
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                assert!(
                    began.elapsed() < PATIENCE,
                    "the sender offered {carried} of the {want} bytes a slice means within {PATIENCE:?}"
                );
                std::thread::sleep(LOOK);
            }
            Err(error) => panic!("the connection was not read: {error}"),
        }
    }
}

/// What:
///  Drive a control's own clock over `slices` slices of the window,
///  and answer the stop it made,
///  if any.
/// Why:
///   The window a control states is its transfer's own:
///  the control states one slice of the window each time the connection has carried the bytes that
///  slice means,
///  so a host that starved the sender or the control's own thread can never make a window look empty
///  (issue 6591).
///  The bytes are the socket's,
///  read here,
///  so the judgement is still made on traffic the connection really carried and never on a number
///  the control wrote down.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let stated = 0; for (let slice = 0; slice < slices; slice++) { await carry(bytesPerSlice); stated += sliceMs; if (stop = clock.judgedAfter(stated)) return stop; }
/// ```
fn drive_the_clock_the_transfer_states(
    clock: &mut Clock,
    stream: &mut TcpStream,
    slices: u32,
    bytes_per_slice: u64,
) -> Option<StopCause> {
    let began: Instant = Instant::now();
    let mut sink: [u8; CARRIED_AT_ONCE] = [0; CARRIED_AT_ONCE];
    let mut stated: Duration = Duration::ZERO;
    for _ in 0..slices {
        carry_a_slice(stream, &mut sink, bytes_per_slice, began);
        stated += slice_of_a_window();
        // `judged_after` asks the clock what it would have said had `stated` time passed, so the judgement is
        //  the release's own and only the time it is taken at is the control's.
        let stop: Option<StopCause> = clock.judged_after(stated);
        if stop.is_some() {
            return stop;
        }
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
    let Some(StopCause::Stalled(report)) = cause else {
        panic!("the trickle was not ended: {}", described(cause))
    };
    assert_eq!(
        report.window,
        StopWindow::Connected,
        "the connected window ended it"
    );
    assert_eq!(report.limit, CONNECTED_WINDOW);
    assert!(
        began.elapsed() >= CONNECTED_WINDOW,
        "not before a whole window: {:?}",
        began.elapsed()
    );
    assert!(
        began.elapsed() < CONNECTED_WINDOW + PATIENCE,
        "and not long after it: {:?}",
        began.elapsed()
    );
    assert!(
        report.carried > 0,
        "the bytes the connection carried are named: {}",
        report.carried
    );
    assert_eq!(report.minimum, FLOOR);
}

/// The same trickle where the transport's bytes count by themselves (no floor,
/// which is what every run but a push has) is
/// progress:
/// the run is not ended,
/// which is the behaviour the minimum rate answers for the push.
/// Its clock is the transfer's own,
/// as the slow transfer's below is,
/// for the reason that control's doc names.
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
    let cause: Option<StopCause> = drive_the_clock_the_transfer_states(
        &mut watched,
        &mut stream,
        SLICES_A_TRANSFER_STATES,
        BYTES_A_TRICKLE_SLICE_MEANS,
    );
    assert!(
        cause.is_none(),
        "a byte of the transport is progress where no floor is set: {}",
        described(cause)
    );
}

/// What:
///  A transfer that meets the floor is work,
///  however slowly it moves,
///  which is the case a person on a slow line meets.
/// Why the clock is the transfer's own:
///   A window judged by this host's clock measured how busy the host was and not what the connection
///   carried:
/// under the gate of 2026-11-27 the sender's own two tenths of a second between two chunks took two
///   thirds of a second,
/// the window counted the 3072 bytes the sender had really offered of the 10240 it meant to send,
///   and a transfer that was work was stopped (issue 6591).
///  So this control states the rate its sender offers,
///  one slice of the window for every slice's worth of bytes the connection carried,
///  and asks the clock at those windows:
/// the transfer is judged on the traffic the socket carried,
///  never on how soon the host woke a thread.
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
/// const clock = clockOf(holder, FLOOR);
/// expect(driveTheClockTheTransferStates(clock, 2 * window, bytesPerSlice)).toBeUndefined();
/// expect(realElapsed).toBeGreaterThanOrEqual(window);   // slow in the seconds a person counts, too
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
    let began: Instant = Instant::now();
    let cause: Option<StopCause> = drive_the_clock_the_transfer_states(
        &mut watched,
        &mut stream,
        SLICES_A_TRANSFER_STATES,
        BYTES_PER_SLICE,
    );
    assert!(
        cause.is_none(),
        "a transfer above the floor is work: {}",
        described(cause)
    );
    // The clock was the transfer's own, so the control also says the transfer really was slow: the bytes it states
    //  arrived over more seconds than one window holds, which a host that is late can only lengthen.
    assert!(
        began.elapsed() >= CONNECTED_WINDOW,
        "the transfer is slow in the seconds a person counts too: it took {:?}",
        began.elapsed()
    );
}

/// A window that carries no byte at all is left to the connected window,
/// which is the rule that bounds a server's silent check,
/// so the floor never ends such a phase sooner than silence alone does.
#[test]
fn a_window_with_no_bytes_is_bounded_by_the_connected_window_and_no_sooner() {
    let (listener, port): (TcpListener, u16) = listening();
    // The holder takes the connection and says nothing at all.
    let holder: Held = hold(&format!(
        "exec 3<>/dev/tcp/127.0.0.1/{port}; while :; do sleep 0.1; done"
    ));
    let mut stream: TcpStream = accepted(&listener);
    let floor: u64 = 1;
    let mut watched: Clock = clock(holder.0.id(), floor);
    let began: Instant = Instant::now();
    let cause: Option<StopCause> = drive(&mut watched, &mut stream, PATIENCE);
    let Some(StopCause::Stalled(report)) = cause else {
        panic!("a silent connection is not ended: {}", described(cause))
    };
    assert_eq!(report.window, StopWindow::Connected);
    assert!(
        began.elapsed() >= CONNECTED_WINDOW,
        "the whole window, as before this rule: {:?}",
        began.elapsed()
    );
    assert_eq!(report.carried, 0, "nothing was carried");
}
