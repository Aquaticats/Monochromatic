//! What:
//!  The launch of a detached helper that is a function of this program and not a start of it: two forks, a new session, standard
//!  streams on `/dev/null` and every other descriptor closed with `close_range`. The first child leaves at once; the helper is the
//!  grandchild, whose parent is then the reaper of orphans.
//! Why:
//!  The one-shot flusher (`doc/planning/cli-git-commits-never-fail.md`, rule V, "Receipt responsibility") must survive its launcher,
//!  must hold no lane lock, client lock, gate or pipe description it inherited (a `flock` belongs to the open file description, which a
//!  fork shares, so an inherited copy would keep a lock the launcher released), and must never be a second start of the wrapper's
//!  executable: the owner's rule is "We should never invoke ourselves recursively." There is no `exec` and no command line here.
//!  Where a service manager exists the same entry point is started as a transient unit instead; that path is not this module's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // detach(entry: () => number): void; // fork twice, setsid, null the streams, close every other descriptor, run entry, exit with its code
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
}

#[cfg(feature = "test-seams")]
static FAULT: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// Makes the next launches of this process (and of the children it forks) fail at `fault`; `None` clears it.
#[cfg(feature = "test-seams")]
pub fn inject_launch_fault(fault: Option<LaunchFault>) {
    let code: u8 = match fault { None => 0, Some(LaunchFault::SecondFork) => 1, Some(LaunchFault::Streams) => 2, Some(LaunchFault::Descriptors) => 3 };
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
        };
    }
}

impl std::error::Error for LaunchError {}

/// What:
///  Closes every descriptor numbered `first` or above in this process, with the `close_range` system call; where that call is absent
///  (`ENOSYS`, before Linux 5.9) or filtered (`EPERM`), it lists `/proc/self/fd` and closes each one.
/// Why:
///  A forked helper shares every open file description of its launcher.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function closeDescriptorsFrom(first: number): void;
/// ```
pub fn close_descriptors_from(first: u32) -> io::Result<()> {
    // SAFETY: `close_range` takes three plain integers and has no memory preconditions.
    let status: libc::c_long = unsafe { libc::syscall(libc::SYS_close_range, first as libc::c_uint, libc::c_uint::MAX, 0 as libc::c_uint) };
    if status == 0 {
        return Ok(());
    }
    let error: io::Error = io::Error::last_os_error();
    if !matches!(error.raw_os_error(), Some(libc::ENOSYS | libc::EPERM)) {
        return Err(error);
    }
    let mut open: Vec<libc::c_int> = Vec::new();
    for entry in std::fs::read_dir("/proc/self/fd")? {
        let name = entry?.file_name();
        if let Some(number) = name.to_str().and_then(|text| { return text.parse::<libc::c_int>().ok(); })
            && number >= 0
            && number as u32 >= first
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

/// Waits, with a bound, for the first child to exit; false when it did not within the limit.
fn first_child_left(child: libc::pid_t) -> Result<bool, LaunchError> {
    let deadline: Instant = Instant::now() + LAUNCH_CONFIRM_LIMIT;
    loop {
        let mut status: libc::c_int = 0;
        // SAFETY: `status` is a live local the call writes through.
        let reaped: libc::pid_t = unsafe { libc::waitpid(child, &raw mut status, libc::WNOHANG) };
        if reaped == child {
            return Ok(true);
        }
        if reaped < 0 {
            return Err(LaunchError::Wait(io::Error::last_os_error()));
        }
        if Instant::now() >= deadline {
            return Ok(false);
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// What:
///  Starts `entry` as a detached helper and returns when the first child has left: the helper runs `entry` in a new session with its
///  standard streams on `/dev/null` and no other descriptor, and exits with the code `entry` returns (`PANICKED_EXIT` if it panics).
///  It runs no `exec`: the helper is a copy of this process's memory, not a start of its executable and not a parse of its command line.
/// Why:
///  See the module documentation. Call it before this process has other threads that hold locks the helper might need; the helper
///  allocates, so it relies on the C library's fork handling of its allocator.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function launchDetached(entry: () => number): void; // throws LaunchError
/// ```
pub fn launch_detached<F: FnOnce() -> i32>(entry: F) -> Result<(), LaunchError> {
    // SAFETY: the child branches below call only `setsid`, `fork`, `_exit` and, in the grandchild, the helper functions of this module.
    let first: libc::pid_t = unsafe { libc::fork() };
    if first < 0 {
        return Err(LaunchError::Refused(io::Error::last_os_error()));
    }
    if first == 0 {
        // SAFETY: as above; every path of this branch ends in `_exit` or becomes the helper, which ends in `_exit`.
        unsafe {
            if libc::setsid() < 0 {
                libc::_exit(1);
            }
            let second: libc::pid_t = if injected(1) { -1 } else { libc::fork() };
            if second < 0 {
                libc::_exit(2);
            }
            if second > 0 {
                libc::_exit(0);
            }
        }
        if injected(2) || !null_the_standard_streams() || injected(3) || close_descriptors_from(3).is_err() {
            // SAFETY: `_exit` takes an integer and never returns.
            unsafe { libc::_exit(3) };
        }
        let code: i32 = catch_unwind(AssertUnwindSafe(entry)).unwrap_or(PANICKED_EXIT);
        // SAFETY: the helper ends here without unwinding into the launcher's stack.
        unsafe { libc::_exit(code) };
    }
    FORKED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    return if first_child_left(first)? { Ok(()) } else { Err(LaunchError::NotConfirmed) };
}
