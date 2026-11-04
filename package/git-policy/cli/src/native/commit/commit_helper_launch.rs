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
}

#[cfg(feature = "test-seams")]
static FAULT: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// Makes the next launches of this process (and of the children it forks) fail at `fault`; `None` clears it.
#[cfg(feature = "test-seams")]
pub fn inject_launch_fault(fault: Option<LaunchFault>) {
    let code: u8 = match fault { None => 0, Some(LaunchFault::SecondFork) => 1, Some(LaunchFault::Streams) => 2, Some(LaunchFault::Descriptors) => 3, Some(LaunchFault::FirstChildHangs) => 4 };
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

/// What the helper's grandchild does before its entry: points its streams at `/dev/null`, closes every descriptor but `keep` and `report`,
/// and writes the one report byte (the failed step, or `READY`); false when a setup step failed (it then ends).
fn helper_setup(report: libc::c_int, keep: &[libc::c_int]) -> bool {
    // `keep` and `report` are numbered above the standard streams (checked before the fork), so the redirection below cannot overwrite them.
    let step: u8 = if injected(2) || !null_the_standard_streams() {
        STREAMS_FAILED
    } else {
        let mut kept: Vec<libc::c_int> = keep.to_vec();
        kept.push(report);
        if injected(3) || close_descriptors_except(3, &kept).is_err() { DESCRIPTORS_FAILED } else { READY }
    };
    // SAFETY: `step` is one live byte; `write` and `close` take plain integers. A launcher that stopped waiting closed its end; the
    // write then fails and the helper goes on or ends by its step.
    unsafe {
        libc::write(report, (&raw const step).cast::<libc::c_void>(), 1);
        libc::close(report);
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
/// or 2 was closed does not give it to a file the helper must keep.
pub fn move_above_streams(descriptor: libc::c_int) -> io::Result<libc::c_int> {
    if descriptor > 2 {
        return Ok(descriptor);
    }
    // SAFETY: `fcntl` with `F_DUPFD_CLOEXEC` takes a descriptor and an integer argument and touches no memory.
    let moved: libc::c_int = unsafe { libc::fcntl(descriptor, libc::F_DUPFD_CLOEXEC, 3) };
    if moved < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: closing the descriptor that was copied; the caller owns it.
    unsafe { libc::close(descriptor) };
    return Ok(moved);
}

/// Waits, until `deadline`, for the launcher's handover byte on `go` (or its end of input, which means the launcher is gone and so is its
/// copy of what it held).
fn await_handover(go: libc::c_int, deadline: Instant) {
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
        if count >= 0 || io::Error::last_os_error().kind() != io::ErrorKind::Interrupted {
            break;
        }
    }
    // SAFETY: the handover end is this process's own.
    unsafe { libc::close(go) };
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
    return launch_detached_holding((), move |()| { return entry(); });
}

/// What:
///  As `launch_detached`, and `holding` (a lock, say) goes to the helper: the helper keeps its descriptors and nothing else, the launcher
///  closes its own copy as soon as the helper reported ready, and only then sends the handover byte the helper waits for before it runs
///  `entry`. A failed launch drops `holding` before it returns.
/// Why:
///  An `flock` belongs to the open file description, which a fork shares: a launcher that keeps its copy after the helper finished and
///  released its own would still hold the lock, and a client that found it held would start nothing. With the handover the helper cannot
///  release before the launcher's copy is gone, so the launcher never holds a lock the helper believes it released.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function launchDetachedHolding<H>(holding: H, entry: (holding: H) => number): void; // throws LaunchError
/// ```
pub fn launch_detached_holding<H: Holding, F: FnOnce(H) -> i32>(holding: H, entry: F) -> Result<(), LaunchError> {
    require_single_thread()?;
    let kept: Vec<libc::c_int> = holding.descriptors();
    if let Some(low) = kept.iter().copied().find(|number| { return *number <= 2; }) {
        return Err(LaunchError::LowDescriptor(low));
    }
    let mut report: [libc::c_int; 2] = [-1, -1];
    let mut handover: [libc::c_int; 2] = [-1, -1];
    // SAFETY: both arrays are live arrays of two integers the calls fill.
    let made: bool = unsafe { libc::pipe(report.as_mut_ptr()) == 0 && libc::pipe(handover.as_mut_ptr()) == 0 };
    if !made {
        let error: io::Error = io::Error::last_os_error();
        // SAFETY: closing descriptors that this call made or left at -1.
        unsafe {
            for number in report.into_iter().chain(handover) {
                if number >= 0 {
                    libc::close(number);
                }
            }
        }
        return Err(LaunchError::Refused(error));
    }
    let moved: io::Result<[libc::c_int; 4]> = (|| {
        return Ok([move_above_streams(report[0])?, move_above_streams(report[1])?, move_above_streams(handover[0])?, move_above_streams(handover[1])?]);
    })();
    let [read_end, write_end, go_read, go_write] = match moved {
        Ok(numbers) => numbers,
        Err(error) => return Err(LaunchError::Refused(error)),
    };
    // SAFETY: the child branches below call only `close`, `setsid`, `fork`, `_exit` and, in the grandchild, the helper functions of this
    // module; this process has one thread, so no lock a vanished thread held is copied.
    let first: libc::pid_t = unsafe { libc::fork() };
    if first < 0 {
        let error: io::Error = io::Error::last_os_error();
        // SAFETY: all four descriptors are this call's own.
        unsafe {
            libc::close(read_end);
            libc::close(write_end);
            libc::close(go_read);
            libc::close(go_write);
        }
        return Err(LaunchError::Refused(error));
    }
    if first == 0 {
        // SAFETY: as above; every path of this branch ends in `_exit` or becomes the helper, which ends in `_exit`.
        unsafe {
            libc::close(read_end);
            libc::close(go_write);
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
        let mut with_handover: Vec<libc::c_int> = kept.clone();
        with_handover.push(go_read);
        if !helper_setup(write_end, &with_handover) {
            // SAFETY: `_exit` takes an integer and never returns.
            unsafe { libc::_exit(3) };
        }
        await_handover(go_read, Instant::now() + LAUNCH_CONFIRM_LIMIT);
        let code: i32 = catch_unwind(AssertUnwindSafe(move || { return entry(holding); })).unwrap_or(PANICKED_EXIT);
        // SAFETY: the helper ends here without unwinding into the launcher's stack.
        unsafe { libc::_exit(code) };
    }
    FORKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    // SAFETY: the launcher's copies of the helper's ends are closed so that end of input means every writer is gone.
    unsafe {
        libc::close(write_end);
        libc::close(go_read);
    }
    let deadline: Instant = Instant::now() + if injected(4) { Duration::from_millis(200) } else { LAUNCH_CONFIRM_LIMIT };
    let report = read_report(read_end, deadline);
    super::commit_identity::fault_point("launch:after-report");
    // SAFETY: the read end is this call's own.
    unsafe { libc::close(read_end) };
    // The launcher's copy of what it held goes first; only then does the helper hear that it may start (and so may finish and release).
    drop(holding);
    let ready: bool = matches!(report, Ok((Some(READY), _)));
    if ready {
        let go: u8 = 1;
        // SAFETY: `go` is one live byte; a helper that is gone makes the write fail, which is harmless.
        unsafe { libc::write(go_write, (&raw const go).cast::<libc::c_void>(), 1) };
    }
    // SAFETY: the handover write end is this call's own; closing it also tells a waiting helper that the launcher is gone.
    unsafe { libc::close(go_write) };
    let (byte, timed_out) = report?;
    let mut status: Option<libc::c_int> = reap_first_child(first, deadline)?;
    if status.is_none() {
        // The intermediate child did not leave in time and still holds every inherited description: it is killed and reaped here, so that
        // a launch that reports success leaves no child of the launcher behind.
        let mut waited: libc::c_int = 0;
        // SAFETY: `kill` and `waitpid` take plain integers and a live local; SIGKILL cannot be caught, so the wait ends.
        unsafe {
            libc::kill(first, libc::SIGKILL);
            libc::waitpid(first, &raw mut waited, 0);
        }
        status = None;
    }
    return match (byte, timed_out) {
        (Some(READY), _) => Ok(()),
        (Some(step), _) => Err(LaunchError::SetupFailed(step)),
        (None, true) => Err(LaunchError::NotConfirmed),
        (None, false) => match status {
            None => Err(LaunchError::NotConfirmed),
            Some(code) if libc::WIFEXITED(code) && libc::WEXITSTATUS(code) != 0 => Err(LaunchError::FirstChildFailed(libc::WEXITSTATUS(code))),
            Some(code) if libc::WIFSIGNALED(code) => Err(LaunchError::FirstChildFailed(128 + libc::WTERMSIG(code))),
            Some(_) => Err(LaunchError::LostBeforeReady),
        },
    };
}
