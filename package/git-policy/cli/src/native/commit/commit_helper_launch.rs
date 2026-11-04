//! What:
//!  The launch of a detached helper that is a function of this program and not a start of it: two forks, a new session, standard
//!  streams on `/dev/null` and every other descriptor closed with `close_range`. The first child leaves at once; the helper is the
//!  grandchild, whose parent is then the reaper of orphans. The launch returns success only after the helper itself reported ready on a
//!  pipe (its setup done, its entry about to run); a failure of the first child or of the helper's setup is the launch's error.
//!  A process with more than one thread is refused before the fork.
//! Why:
//!  The one-shot flusher (`doc/planning/cli-git-commits-never-fail.md`, rule V, "Receipt responsibility") must survive its launcher,
//!  must hold no lane lock, client lock, gate or pipe description it inherited (a `flock` belongs to the open file description, which a
//!  fork shares, so an inherited copy would keep a lock the launcher released), and must never be a second start of the wrapper's
//!  executable: the owner's rule is "We should never invoke ourselves recursively." There is no `exec` and no command line here.
//!  Where a service manager exists the same entry point is started as a transient unit instead; that path is not this module's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // detach(entry: () => number): void; // one thread only; fork twice, setsid, null the streams, close every other descriptor, report ready, run entry
//! ```

use std::fmt;
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::{Duration, Instant};

/// The longest the launcher waits for the first child to leave. The first child only calls `setsid`, `fork` and `_exit`; the bound is the
/// owner's "nothing should ever wait forever or indefinitely", not a measured latency.
pub const LAUNCH_CONFIRM_LIMIT: Duration = Duration::from_secs(5);

/// How many helpers this process has forked a first child for (a launch counter, for tests and for the logs of a launcher).
static FORKED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// How many first children this process has forked.
pub fn helpers_forked() -> u64 {
    return FORKED.load(std::sync::atomic::Ordering::SeqCst);
}

/// The step of a launch where a test makes the system fail.
#[cfg(feature = "test-seams")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaunchFault {
    /// The first child's second `fork` fails.
    SecondFork,
    /// The helper's standard streams cannot be pointed at `/dev/null`.
    Streams,
    /// The helper's descriptors cannot be closed.
    Descriptors,
    /// The first child stalls (and is killed by its own alarm) after its second fork, before it exits.
    FirstChildHangs,
    /// The helper waits only a short while for the handover byte.
    ShortHandoverWait,
    /// The first child stalls as with `FirstChildHangs`, and the kill that cleans it up is refused.
    KillRefused,
}

#[cfg(feature = "test-seams")]
static FAULT: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// Makes the next launches of this process (and of the children it forks) fail at `fault`; `None` clears it.
#[cfg(feature = "test-seams")]
pub fn inject_launch_fault(fault: Option<LaunchFault>) {
    let code: u8 = match fault { None => 0, Some(LaunchFault::SecondFork) => 1, Some(LaunchFault::Streams) => 2, Some(LaunchFault::Descriptors) => 3, Some(LaunchFault::FirstChildHangs) => 4, Some(LaunchFault::ShortHandoverWait) => 5, Some(LaunchFault::KillRefused) => 6 };
    FAULT.store(code, std::sync::atomic::Ordering::SeqCst);
}

/// Whether a test asked `fault` to fail; always false without the test seams.
fn injected(fault: u8) -> bool {
    #[cfg(feature = "test-seams")]
    return FAULT.load(std::sync::atomic::Ordering::SeqCst) == fault;
    #[cfg(not(feature = "test-seams"))]
    return { let _ = fault; false };
}

/// The exit code of a helper whose entry point panicked.
pub const PANICKED_EXIT: i32 = 101;

/// Why a helper was not started.
#[derive(Debug)]
pub enum LaunchError {
    /// `fork` was refused (a process-count or memory limit someone else set): the stated outside cause, to be named and not retried.
    Refused(io::Error),
    /// The first child did not leave within `LAUNCH_CONFIRM_LIMIT`; no helper is claimed.
    NotConfirmed,
    /// Waiting for the first child failed.
    Wait(io::Error),
    /// The first child left with a failure code (its `setsid` or its second `fork` failed): no helper exists.
    FirstChildFailed(i32),
    /// The helper could not finish its own setup (streams, descriptors) and reported the step; it exited.
    SetupFailed(u8),
    /// The helper was lost before it reported ready (no report, no failure code).
    LostBeforeReady,
    /// This process has more than one thread, so a fork would copy locks other threads hold; nothing was forked.
    Threaded(usize),
    /// The thread count could not be read, so single-threadedness cannot be shown; nothing was forked.
    ThreadsUnknown(io::Error),
    /// A descriptor the helper must keep is numbered 0, 1 or 2, where the helper's standard-stream setup would overwrite it; nothing was
    /// forked.
    LowDescriptor(libc::c_int),
    /// The helper did not acknowledge the handover (it gave up waiting, or it is gone); it ran no entry, and the caller accounts for it.
    HandoverNotAcknowledged,
    /// The intermediate child could not be killed and reaped within the bound.
    IntermediateNotReaped,
    /// The helper could not take what it prepares (the lock, say) and reported it instead of acknowledging.
    HelperCouldNotPrepare,
}

impl fmt::Display for LaunchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            LaunchError::Refused(error) => write!(formatter, "the system refused to start a helper process (a process or memory limit): {}", error),
            LaunchError::NotConfirmed => write!(formatter, "the helper launch was not confirmed within {:?}", LAUNCH_CONFIRM_LIMIT),
            LaunchError::Wait(error) => write!(formatter, "waiting for the helper launch failed: {}", error),
            LaunchError::FirstChildFailed(code) => write!(formatter, "the helper launch failed before the helper existed (first child exit code {})", code),
            LaunchError::SetupFailed(step) => write!(formatter, "the helper could not finish its setup (step {})", step),
            LaunchError::LostBeforeReady => write!(formatter, "the helper was lost before it reported ready"),
            LaunchError::Threaded(count) => write!(formatter, "a helper was not forked because this process has {} threads and a fork would copy locks they may hold", count),
            LaunchError::ThreadsUnknown(error) => write!(formatter, "a helper was not forked because the thread count could not be read: {}", error),
            LaunchError::HelperCouldNotPrepare => write!(formatter, "the helper could not take what it needs to work and ran nothing"),
            LaunchError::HandoverNotAcknowledged => write!(formatter, "the helper did not acknowledge the handover and ran nothing"),
            LaunchError::IntermediateNotReaped => write!(formatter, "the intermediate child of the launch could not be killed and reaped in time"),
            LaunchError::LowDescriptor(number) => write!(formatter, "a helper was not forked because descriptor {} is among the standard streams the helper redirects", number),
        };
    }
}

impl std::error::Error for LaunchError {}

/// What:
///  Closes every descriptor numbered `first` or above in this process, except those in `keep`, with the `close_range` system call; where
///  that call is absent (`ENOSYS`, before Linux 5.9) or filtered (`EPERM`), it lists `/proc/self/fd` and closes each one.
/// Why:
///  A forked helper shares every open file description of its launcher; the few it must keep (its ready report, the flusher lock it was
///  handed) are named, and nothing else survives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function closeDescriptorsExcept(first: number, keep: number[]): void;
/// ```
pub fn close_descriptors_except(first: u32, keep: &[libc::c_int]) -> io::Result<()> {
    let mut kept: Vec<u32> = keep.iter().filter_map(|number| { return u32::try_from(*number).ok(); }).filter(|number| { return *number >= first; }).collect();
    kept.sort_unstable();
    let mut ranges: Vec<(u32, u32)> = Vec::with_capacity(kept.len() + 1);
    let mut start: u32 = first;
    for number in &kept {
        if *number > start {
            ranges.push((start, *number - 1));
        }
        start = number.saturating_add(1);
    }
    ranges.push((start, u32::MAX));
    let mut fallback: bool = false;
    for (low, high) in &ranges {
        // SAFETY: `close_range` takes three plain integers and has no memory preconditions.
        let status: libc::c_long = unsafe { libc::syscall(libc::SYS_close_range, *low as libc::c_uint, *high as libc::c_uint, 0 as libc::c_uint) };
        if status == 0 {
            continue;
        }
        let error: io::Error = io::Error::last_os_error();
        if !matches!(error.raw_os_error(), Some(libc::ENOSYS | libc::EPERM)) {
            return Err(error);
        }
        fallback = true;
        break;
    }
    if !fallback {
        return Ok(());
    }
    let mut open: Vec<libc::c_int> = Vec::new();
    for entry in std::fs::read_dir("/proc/self/fd")? {
        let name = entry?.file_name();
        if let Some(number) = name.to_str().and_then(|text| { return text.parse::<libc::c_int>().ok(); })
            && number >= 0
            && number as u32 >= first
            && !keep.contains(&number)
        {
            open.push(number);
        }
    }
    for number in open {
        // SAFETY: closing a number that may already be closed (the directory's own descriptor) is harmless; the result is not needed.
        unsafe { libc::close(number) };
    }
    return Ok(());
}

/// Closes every descriptor numbered `first` or above; see `close_descriptors_except`.
pub fn close_descriptors_from(first: u32) -> io::Result<()> {
    return close_descriptors_except(first, &[]);
}

/// Points descriptors 0, 1 and 2 at `/dev/null`; false when that failed.
fn null_the_standard_streams() -> bool {
    // SAFETY: the path is a NUL-terminated literal; `open`, `dup2` and `close` take plain integers.
    unsafe {
        let null: libc::c_int = libc::open(c"/dev/null".as_ptr(), libc::O_RDWR);
        if null < 0 {
            return false;
        }
        for target in 0..=2 {
            if libc::dup2(null, target) < 0 {
                return false;
            }
        }
        if null > 2 {
            libc::close(null);
        }
    }
    return true;
}

/// The report byte of a helper whose setup finished.
const READY: u8 = 0;

/// The report byte of a helper that could not point its streams at `/dev/null`.
const STREAMS_FAILED: u8 = 1;

/// The report byte of a helper that could not close its inherited descriptors.
const DESCRIPTORS_FAILED: u8 = 2;

/// The number of threads of this process, read from `/proc/self/task`; the launch forks only when it is one.
fn require_single_thread() -> Result<(), LaunchError> {
    let mut count: usize = 0;
    for entry in std::fs::read_dir("/proc/self/task").map_err(LaunchError::ThreadsUnknown)? {
        entry.map_err(LaunchError::ThreadsUnknown)?;
        count += 1;
    }
    return if count == 1 { Ok(()) } else { Err(LaunchError::Threaded(count)) };
}

/// Waits, until `deadline`, for the first child to exit; its wait status, or `None` when it did not leave in time.
fn reap_first_child(child: libc::pid_t, deadline: Instant) -> Result<Option<libc::c_int>, LaunchError> {
    loop {
        let mut status: libc::c_int = 0;
        // SAFETY: `status` is a live local the call writes through.
        let reaped: libc::pid_t = unsafe { libc::waitpid(child, &raw mut status, libc::WNOHANG) };
        if reaped == child {
            return Ok(Some(status));
        }
        if reaped < 0 {
            return Err(LaunchError::Wait(io::Error::last_os_error()));
        }
        if Instant::now() >= deadline {
            return Ok(None);
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Reads the helper's ready report from `report`, waiting until `deadline`: the byte, or `None` at end of input (every writer closed
/// without a byte) or at the deadline (`timed_out` says which).
fn read_report(report: libc::c_int, deadline: Instant) -> Result<(Option<u8>, bool), LaunchError> {
    loop {
        let left: Duration = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Ok((None, true));
        }
        let mut waiting = libc::pollfd { fd: report, events: libc::POLLIN, revents: 0 };
        let milliseconds: libc::c_int = libc::c_int::try_from(left.as_millis().max(1)).unwrap_or(libc::c_int::MAX);
        // SAFETY: `waiting` is one live `pollfd`.
        let ready: libc::c_int = unsafe { libc::poll(&raw mut waiting, 1, milliseconds) };
        if ready < 0 {
            let error: io::Error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(LaunchError::Wait(error));
        }
        if ready == 0 {
            continue;
        }
        let mut byte: u8 = 0;
        // SAFETY: `byte` is one live byte the call writes through.
        let count: isize = unsafe { libc::read(report, (&raw mut byte).cast::<libc::c_void>(), 1) };
        if count == 1 {
            return Ok((Some(byte), false));
        }
        if count == 0 {
            return Ok((None, false));
        }
        let error: io::Error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(LaunchError::Wait(error));
        }
    }
}

/// The report byte of a helper that has the handover and runs its entry.
const ACKNOWLEDGED: u8 = 3;

/// The report byte of a helper whose `prepare` failed: it reports it instead of acknowledging.
const PREPARE_FAILED: u8 = 4;

/// How long the launcher waits, after the first child was killed, for it to be reaped (the kill cannot be caught; this only bounds a
/// process stuck in the kernel). A launch is bounded by three of these limits: the report, the intermediate child's exit and the
/// helper's acknowledgment, plus this.
const KILL_REAP_LIMIT: Duration = Duration::from_secs(5);

/// What the helper's grandchild does before its entry: points its streams at `/dev/null`, closes every descriptor but `keep` and `report`,
/// and writes the one report byte (the failed step, or `READY`); false when a setup step failed (it then ends). The report descriptor
/// stays open after `READY`: the acknowledgment of the handover goes through it.
fn helper_setup(report: libc::c_int, keep: &[libc::c_int]) -> bool {
    // `keep` and `report` are numbered above the standard streams (checked before the fork), so the redirection below cannot overwrite them.
    let step: u8 = if injected(2) || !null_the_standard_streams() {
        STREAMS_FAILED
    } else {
        let mut kept: Vec<libc::c_int> = keep.to_vec();
        kept.push(report);
        if injected(3) || close_descriptors_except(3, &kept).is_err() { DESCRIPTORS_FAILED } else { READY }
    };
    // SAFETY: `step` is one live byte; `write` takes plain integers. A launcher that stopped waiting closed its end; the write then fails
    // and the helper ends by its step.
    unsafe { libc::write(report, (&raw const step).cast::<libc::c_void>(), 1) };
    if step != READY {
        // SAFETY: the report end is this process's own.
        unsafe { libc::close(report) };
    }
    return step == READY;
}

/// What a launcher hands to its helper: the descriptors (open file descriptions, such as a lock) that the helper keeps and that the
/// launcher gives up once the helper is ready.
pub trait Holding {
    /// The descriptor numbers, each above the standard streams.
    fn descriptors(&self) -> Vec<libc::c_int>;
}

impl Holding for () {
    fn descriptors(&self) -> Vec<libc::c_int> {
        return Vec::new();
    }
}

/// Moves `descriptor` above the standard streams (a copy numbered 3 or higher, the original closed), so that a process whose descriptor 0, 1
/// or 2 was closed does not give it to a file the helper must keep. The descriptor has an owner throughout: on failure it is closed by
/// being dropped here.
pub fn move_above_streams(descriptor: OwnedFd) -> io::Result<OwnedFd> {
    if descriptor.as_raw_fd() > 2 {
        return Ok(descriptor);
    }
    // SAFETY: `fcntl` with `F_DUPFD_CLOEXEC` takes a descriptor and an integer argument and touches no memory.
    let moved: libc::c_int = unsafe { libc::fcntl(descriptor.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 3) };
    if moved < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `moved` was just returned by `fcntl` and nothing else owns it; the original closes when `descriptor` drops.
    return Ok(unsafe { OwnedFd::from_raw_fd(moved) });
}

/// A pipe whose two ends are owned from the moment they exist and numbered above the standard streams: (read end, write end).
fn make_pipe() -> Result<(OwnedFd, OwnedFd), LaunchError> {
    let mut ends: [libc::c_int; 2] = [-1, -1];
    // SAFETY: `ends` is a live array of two integers the call fills.
    if unsafe { libc::pipe(ends.as_mut_ptr()) } != 0 {
        return Err(LaunchError::Refused(io::Error::last_os_error()));
    }
    // SAFETY: both numbers were just returned by `pipe` and nothing else owns them.
    let (low_read, low_write): (OwnedFd, OwnedFd) = unsafe { (OwnedFd::from_raw_fd(ends[0]), OwnedFd::from_raw_fd(ends[1])) };
    let high_read: OwnedFd = move_above_streams(low_read).map_err(LaunchError::Refused)?;
    let high_write: OwnedFd = move_above_streams(low_write).map_err(LaunchError::Refused)?;
    return Ok((high_read, high_write));
}

/// Waits, until `deadline`, for the launcher's handover byte on `go`; true only when the byte arrived. A timeout, an end of input (the
/// launcher is gone) or a read failure is not a handover, and the helper then ends without running its entry. `go` is closed here.
fn await_handover(go: libc::c_int, deadline: Instant) -> bool {
    let mut granted: bool = false;
    loop {
        let left: Duration = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            break;
        }
        let mut waiting = libc::pollfd { fd: go, events: libc::POLLIN, revents: 0 };
        let milliseconds: libc::c_int = libc::c_int::try_from(left.as_millis().max(1)).unwrap_or(libc::c_int::MAX);
        // SAFETY: `waiting` is one live `pollfd`.
        let ready: libc::c_int = unsafe { libc::poll(&raw mut waiting, 1, milliseconds) };
        if ready < 0 && io::Error::last_os_error().kind() == io::ErrorKind::Interrupted {
            continue;
        }
        if ready <= 0 {
            break;
        }
        let mut byte: u8 = 0;
        // SAFETY: `byte` is one live byte the call writes through.
        let count: isize = unsafe { libc::read(go, (&raw mut byte).cast::<libc::c_void>(), 1) };
        if count < 0 && io::Error::last_os_error().kind() == io::ErrorKind::Interrupted {
            continue;
        }
        granted = count == 1;
        break;
    }
    // SAFETY: the handover end is this process's own.
    unsafe { libc::close(go) };
    return granted;
}

/// Kills the intermediate child and reaps it, within `KILL_REAP_LIMIT`. A refused kill (other than "no such process", which means it
/// already left), a failed wait and a child that does not die in time are errors; an interrupted wait is repeated.
fn kill_and_reap(child: libc::pid_t) -> Result<(), LaunchError> {
    if injected(6) {
        return Err(LaunchError::Wait(io::Error::from_raw_os_error(libc::EPERM)));
    }
    // SAFETY: `kill` takes plain integers.
    if unsafe { libc::kill(child, libc::SIGKILL) } != 0 {
        let error: io::Error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::ESRCH) {
            return Err(LaunchError::Wait(error));
        }
    }
    let deadline: Instant = Instant::now() + KILL_REAP_LIMIT;
    loop {
        let mut status: libc::c_int = 0;
        // SAFETY: `status` is a live local the call writes through.
        let reaped: libc::pid_t = unsafe { libc::waitpid(child, &raw mut status, libc::WNOHANG) };
        if reaped == child {
            return Ok(());
        }
        if reaped < 0 {
            let error: io::Error = io::Error::last_os_error();
            match error.raw_os_error() {
                Some(libc::EINTR) => continue,
                Some(libc::ECHILD) => return Ok(()),
                _ => return Err(LaunchError::Wait(error)),
            }
        }
        if Instant::now() >= deadline {
            return Err(LaunchError::IntermediateNotReaped);
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Waits for the intermediate child to leave until `deadline`; one that does not is killed and reaped. `Some(status)` when it left by
/// itself, `None` when it had to be killed.
fn reap_or_kill(child: libc::pid_t, deadline: Instant) -> Result<Option<libc::c_int>, LaunchError> {
    return match reap_first_child(child, deadline)? {
        Some(status) => Ok(Some(status)),
        None => {
            kill_and_reap(child)?;
            Ok(None)
        }
    };
}

/// What:
///  Starts `entry` as a detached helper and returns when the helper has reported ready: it runs `entry` in a new session with its
///  standard streams on `/dev/null` and no other descriptor, and exits with the code `entry` returns (`PANICKED_EXIT` if it panics).
///  It runs no `exec`: the helper is a copy of this process's memory, not a start of its executable and not a parse of its command line.
/// Why:
///  See the module documentation. A fork copies the memory of the forking thread only, so a lock another thread holds would be held for ever
///  in the copy; the launch therefore refuses a process with another thread (`LaunchError::Threaded`) instead of relying on the caller.
///  The helper allocates, which a single-threaded fork leaves safe.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function launchDetached(entry: () => number): void; // throws LaunchError
/// ```
pub fn launch_detached<F: FnOnce() -> i32>(entry: F) -> Result<(), LaunchError> {
    return launch_detached_holding((), || { return Ok(()); }, move |_| { return entry(); });
}

/// What:
///  As `launch_detached`, and the launcher holds `holding` (a lock, say) until the helper is ready: the intermediate child and the helper
///  close its descriptors at once and never carry it; the launcher reaps the intermediate child (killing it if it does not leave), closes
///  its own copy, and only then sends the handover byte. The helper then runs `prepare` (taking the lock itself, say), acknowledges, and
///  runs `entry` with what `prepare` returned. A `prepare` that fails is reported instead of the acknowledgment (`HelperCouldNotPrepare`),
///  and `entry` still gets the error so that the helper can account for what it could not take. A helper that gets no byte (a wait that expires, an end of input because the launcher died)
///  still runs `prepare` and `entry`, without acknowledging: if the launcher is gone the helper is the executor, and if the launcher
///  is only slow it still holds the lock and `prepare` finds it taken. A launcher that does not get the acknowledgment fails with
///  `HandoverNotAcknowledged`, and its caller accounts for every receipt.
/// Why:
///  An `flock` belongs to the open file description, which a fork shares: any process but the launcher that carries a copy can keep the
///  lock after the one that works has released it, and a client that found it held starts nothing. Here no process but the launcher
///  carries a copy while the lock is held for the launch, so a failure or a death at any step leaves either the launcher's own cleanup or
///  the helper's `prepare` as the one that decides who holds the lock next (review 12 section 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function launchDetachedHolding<H, T>(holding: H, prepare: () => T, entry: (prepared: T) => number): void; // throws LaunchError
/// ```
pub fn launch_detached_holding<H: Holding, T, P: FnOnce() -> io::Result<T>, F: FnOnce(io::Result<T>) -> i32>(holding: H, prepare: P, entry: F) -> Result<(), LaunchError> {
    require_single_thread()?;
    let kept: Vec<libc::c_int> = holding.descriptors();
    if let Some(low) = kept.iter().copied().find(|number| { return *number <= 2; }) {
        return Err(LaunchError::LowDescriptor(low));
    }
    let (report_read, report_write): (OwnedFd, OwnedFd) = make_pipe()?;
    let (go_read, go_write): (OwnedFd, OwnedFd) = make_pipe()?;
    // SAFETY: the child branches below call only `close`, `setsid`, `fork`, `_exit` and, in the grandchild, the helper functions of this
    // module; this process has one thread, so no lock a vanished thread held is copied.
    let first: libc::pid_t = unsafe { libc::fork() };
    if first < 0 {
        return Err(LaunchError::Refused(io::Error::last_os_error()));
    }
    if first == 0 {
        // SAFETY: as above; every path of this branch ends in `_exit` or becomes the helper, which ends in `_exit`.
        unsafe {
            if injected(6) {
                // The intermediate child is delayed before it closes anything: it still holds every inherited description.
                libc::alarm(20);
                loop {
                    libc::pause();
                }
            }
            libc::close(report_read.as_raw_fd());
            libc::close(go_write.as_raw_fd());
            for number in &kept {
                libc::close(*number);
            }
            if libc::setsid() < 0 {
                libc::_exit(1);
            }
            let second: libc::pid_t = if injected(1) { -1 } else { libc::fork() };
            if second < 0 {
                libc::_exit(2);
            }
            if second > 0 {
                if injected(4) {
                    libc::alarm(20);
                    loop {
                        libc::pause();
                    }
                }
                libc::_exit(0);
            }
        }
        if !helper_setup(report_write.as_raw_fd(), &[go_read.as_raw_fd()]) {
            if injected(2) {
                // A helper that failed its setup lingers before it exits (the schedule of review 12 section 3B).
                std::thread::sleep(Duration::from_millis(400));
            }
            // SAFETY: `_exit` takes an integer and never returns.
            unsafe { libc::_exit(3) };
        }
        let wait: Duration = if injected(5) { Duration::from_millis(100) } else { LAUNCH_CONFIRM_LIMIT };
        let granted: bool = await_handover(go_read.as_raw_fd(), Instant::now() + wait);
        let code: i32 = catch_unwind(AssertUnwindSafe(move || {
            let prepared: io::Result<T> = prepare();
            if granted {
                // The result of `prepare` is part of the acknowledgment: a helper that could not take what it needs says so.
                let acknowledged: u8 = if prepared.is_ok() { ACKNOWLEDGED } else { PREPARE_FAILED };
                // SAFETY: `acknowledged` is one live byte; the report end is this process's own.
                unsafe { libc::write(report_write.as_raw_fd(), (&raw const acknowledged).cast::<libc::c_void>(), 1) };
            }
            // SAFETY: the report end is this process's own and is closed here, after the acknowledgment.
            unsafe { libc::close(report_write.as_raw_fd()) };
            return entry(prepared);
        }))
        .unwrap_or(PANICKED_EXIT);
        // SAFETY: the helper ends here without unwinding into the launcher's stack.
        unsafe { libc::_exit(code) };
    }
    FORKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    // The launcher's copies of the helper's ends are closed so that end of input means every writer is gone.
    drop(report_write);
    drop(go_read);
    let deadline: Instant = Instant::now() + if injected(4) || injected(6) { Duration::from_millis(200) } else { LAUNCH_CONFIRM_LIMIT };
    let outcome = read_report(report_read.as_raw_fd(), deadline);
    super::commit_identity::fault_point("launch:after-report");
    let ready: bool = matches!(outcome, Ok((Some(READY), _)));
    // The intermediate child goes first, on every path: reaped when it left, killed and reaped when it did not.
    let status: Result<Option<libc::c_int>, LaunchError> = reap_or_kill(first, deadline);
    if !ready {
        drop(holding);
        drop(go_write);
        let (byte, timed_out) = outcome?;
        let left_by_itself: Option<libc::c_int> = status?;
        return match (byte, timed_out) {
            (Some(step), _) => Err(LaunchError::SetupFailed(step)),
            (None, true) => Err(LaunchError::NotConfirmed),
            (None, false) => match left_by_itself {
                None => Err(LaunchError::NotConfirmed),
                Some(code) if libc::WIFEXITED(code) && libc::WEXITSTATUS(code) != 0 => Err(LaunchError::FirstChildFailed(libc::WEXITSTATUS(code))),
                Some(code) if libc::WIFSIGNALED(code) => Err(LaunchError::FirstChildFailed(128 + libc::WTERMSIG(code))),
                Some(_) => Err(LaunchError::LostBeforeReady),
            },
        };
    }
    if let Err(error) = status {
        // The helper is waiting for a handover that will not come: closing the handover end tells it so, and it runs nothing.
        drop(holding);
        drop(go_write);
        return Err(error);
    }
    // Every copy but the helper's is gone once the launcher's own copy is closed; only then does the helper hear that it may start.
    drop(holding);
    let go: u8 = 1;
    // SAFETY: `go` is one live byte; a helper that is gone makes the write fail, which is the failure reported below.
    let sent: isize = unsafe { libc::write(go_write.as_raw_fd(), (&raw const go).cast::<libc::c_void>(), 1) };
    super::commit_identity::fault_point("launch:after-handover");
    drop(go_write);
    if sent != 1 {
        return Err(LaunchError::HandoverNotAcknowledged);
    }
    let acknowledgment_deadline: Instant = Instant::now() + LAUNCH_CONFIRM_LIMIT;
    return match read_report(report_read.as_raw_fd(), acknowledgment_deadline)? {
        (Some(ACKNOWLEDGED), _) => Ok(()),
        (Some(PREPARE_FAILED), _) => Err(LaunchError::HelperCouldNotPrepare),
        _ => Err(LaunchError::HandoverNotAcknowledged),
    };
}
