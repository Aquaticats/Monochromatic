//! What:
//!  Whether one thread of this process is at work,
//! judged with the signs the push meter reads for a child process,
//! and the
//!  clock that says when the thread has shown none of them for a window.
//! Why:
//!  The Markdown linter runs on a thread of this process and the gateway call of the standards review blocks one,
//! so neither
//!  has a child process whose tree `push_progress.rs` could sample.
//! Linux keeps the same counters for each thread under
//!  `/proc/self/task/<tid>/`:
//! the bytes it read and wrote (`io`),
//! and its processor ticks and state (`stat`).
//! The thread being
//!  judged is never the thread that reads these files,
//! so the reading does not count as the thread's work.
//!  Where `/proc/self/task/<tid>/io` or `stat` cannot be read nothing is measured and the thread is never judged quiet.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const clock = new ThreadClock(tid, windowMs); while (!done) { if (clock.quiet()) abandon(); await sleep(SAMPLE_MS); }
//! ```

/// The parsing of a `stat` line and the state letters that count as work.
use crate::push_progress::{SAMPLE_INTERVAL, is_working, parse_activity};
/// The connected sockets of the machine,
/// and those a process holds.
use crate::tcp_sockets::{ESTABLISHED, established_sockets, socket_inodes_of};
/// What each socket had carried when it was last seen.
use std::collections::HashMap;
/// Shared flags between the thread that judges and the one that waits.
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
/// Time spans and the monotonic clock.
use std::time::{Duration, Instant};

/// What:
///  The kernel's number for the calling thread,
/// which names its files under `/proc/self/task/`.
/// Why:
///   `std` does not expose it;
/// the system call takes no arguments and cannot fail.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tid = threadId();
/// ```
#[cfg(target_os = "linux")]
pub fn current_thread_id() -> Option<i32> {
    // SAFETY: `gettid` takes no arguments, touches no memory and cannot fail.
    let raw: libc::c_long = unsafe { libc::syscall(libc::SYS_gettid) };
    return i32::try_from(raw).ok();
}

/// Only Linux names a thread by a number that `/proc` understands;
/// elsewhere nothing is measured.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn current_thread_id() -> Option<i32> {
    return None;
}

/// The bytes a thread has read and written,
/// from its `io` file,
/// or nothing where the file cannot be read.
#[cfg(target_os = "linux")]
fn thread_bytes(tid: i32) -> Option<u64> {
    let text: String = std::fs::read_to_string(format!("/proc/self/task/{tid}/io")).ok()?;
    let mut read: Option<u64> = None;
    let mut written: Option<u64> = None;
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("rchar: ") {
            read = value.trim().parse::<u64>().ok();
        } else if let Some(value) = line.strip_prefix("wchar: ") {
            written = value.trim().parse::<u64>().ok();
        }
    }
    return Some(read?.saturating_add(written?));
}

/// Only Linux counts a thread's bytes;
/// elsewhere nothing is measured.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
fn thread_bytes(_tid: i32) -> Option<u64> {
    return None;
}

/// The processor ticks of a thread and whether it is able to run or waiting for the disk,
/// or nothing where `stat` cannot be read.
#[cfg(target_os = "linux")]
fn thread_activity(tid: i32) -> Option<(u64, bool)> {
    let text: String = std::fs::read_to_string(format!("/proc/self/task/{tid}/stat")).ok()?;
    let (state, ticks): (char, u64) = parse_activity(text.as_str())?;
    return Some((ticks, is_working(state)));
}

/// Only Linux reads what a thread is doing;
/// elsewhere nothing is.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
fn thread_activity(_tid: i32) -> Option<(u64, bool)> {
    return None;
}

/// What:
///  What one thread had done when it was last looked at,
/// and the look that says whether it did more since.
/// Why:
///   Bytes and ticks only grow,
/// so a change since the last look is a sign of work;
/// a thread in a state able to run or waiting
///  for the disk is at work now.
/// The connected sockets of the process count too,
/// when asked:
/// an HTTP client moves its bytes
///  with `send` and `recv`,
/// which the thread's `rchar` and `wchar` do not see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ThreadSigns { tid: number; look(): boolean }
/// ```
#[derive(Debug)]
pub struct ThreadSigns {
    /// The thread's number.
    tid: i32,
    /// The bytes and processor ticks at the last look.
    before: Option<(u64, u64)>,
    /// Whether the process's connected sockets count as the thread's work.
    count_sockets: bool,
    /// The bytes each connected socket had carried at the last look,
    /// by inode.
    sockets: HashMap<u64, u64>,
    /// Whether anything about the thread could be read at the last look.
    measured: bool,
}

impl ThreadSigns {
    /// Signs of the thread `tid`;
    /// `count_sockets` adds the process's connected sockets.
    pub fn new(tid: i32, count_sockets: bool) -> Self {
        return Self { tid, before: None, count_sockets, sockets: HashMap::new(), measured: false };
    }

    /// Whether the last look could read the thread at all,
    /// so a quiet thread is quiet and not unreadable.
    pub fn measured(&self) -> bool {
        return self.measured;
    }

    /// The bytes the process's connected sockets carried since the last look,
    /// or zero where they cannot be listed.
    fn socket_bytes_moved(&mut self) -> u64 {
        // A descriptor list that cannot be read names no socket, so a look that cannot list them counts nothing, as before.
        let held: Vec<u64> = socket_inodes_of(i32::try_from(std::process::id()).unwrap_or(i32::MAX)).unwrap_or_default();
        let Some(connected) = established_sockets() else {
            return 0;
        };
        let mut moved: u64 = 0;
        for socket in connected {
            if socket.state != ESTABLISHED || !held.contains(&socket.inode) {
                continue;
            }
            let total: u64 = socket.bytes_acked.saturating_add(socket.bytes_received);
            let seen: u64 = self.sockets.insert(socket.inode, total).unwrap_or(total);
            moved = moved.saturating_add(total.saturating_sub(seen));
        }
        return moved;
    }

    /// Look at the thread once:
    /// true when it moved bytes or used processor time since the last look,
    /// or is at work now.
    ///  The first look is a sign of work,
    /// because there is nothing to compare with.
    pub fn look(&mut self) -> bool {
        let bytes: Option<u64> = thread_bytes(self.tid);
        let activity: Option<(u64, bool)> = thread_activity(self.tid);
        self.measured = bytes.is_some() || activity.is_some();
        if !self.measured {
            return false;
        }
        // No activity reading means no processor ticks and not at work.
        let (ticks, working): (u64, bool) = activity.unwrap_or_default();
        let now: (u64, u64) = (bytes.unwrap_or(0), ticks);
        // The first look has nothing to compare with, so it counts as a change.
        let changed: bool = match self.before {
            Some((old_bytes, old_ticks)) => now.0 > old_bytes || now.1 > old_ticks,
            None => true,
        };
        self.before = Some(now);
        let carried: bool = self.count_sockets && self.socket_bytes_moved() > 0;
        return changed || working || carried;
    }
}

/// What:
///  The clock of one thread:
/// it looks at the thread's signs at the sampling interval and says when none has shown for a window.
/// Why:
///   The waiting thread asks every few milliseconds;
/// the counters are read only every `SAMPLE_INTERVAL`,
/// as for a child process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ThreadClock { quiet(): boolean; longestQuiet(): number }
/// ```
#[derive(Debug)]
pub struct ThreadClock {
    /// What the thread has done.
    signs: ThreadSigns,
    /// How long the thread may show no sign of work.
    window: Duration,
    /// When a sign of work was last seen (the start,
    /// before the first).
    last_progress: Instant,
    /// When the counters are next read.
    next_sample: Instant,
    /// The longest stretch between two looks that found a sign of work.
    longest_quiet: Duration,
}

impl ThreadClock {
    /// Start the clock of the thread `tid` now;
    /// `count_sockets` adds the process's connected sockets to its signs.
    pub fn start(tid: i32, window: Duration, count_sockets: bool) -> Self {
        let now: Instant = Instant::now();
        return Self { signs: ThreadSigns::new(tid, count_sockets), window, last_progress: now, next_sample: now, longest_quiet: Duration::ZERO };
    }

    /// Read the signs when a sample is due,
    /// and say whether the thread has now been quiet for the whole window.
    ///  A thread that cannot be read is never judged quiet.
    pub fn quiet(&mut self) -> bool {
        let now: Instant = Instant::now();
        if now >= self.next_sample {
            if self.signs.look() {
                self.longest_quiet = self.longest_quiet.max(now.saturating_duration_since(self.last_progress));
                self.last_progress = now;
            }
            self.next_sample = now + SAMPLE_INTERVAL;
        }
        return self.signs.measured() && now.saturating_duration_since(self.last_progress) >= self.window;
    }

    /// The longest stretch with no sign of work so far,
    /// the one still running included.
    pub fn longest_quiet(&self) -> Duration {
        return self.longest_quiet.max(Instant::now().saturating_duration_since(self.last_progress));
    }
}

/// What:
///  The measurement of how quiet the calling thread gets while a call runs,
/// begun before the call and finished after it.
/// Why:
///   A call that ends itself (the gateway call ends on its transport's own idle timeouts) still needs its quiet gap recorded.
///  Nothing here ends the call.
/// The call runs in place between `begin` and `finish`,
/// so it reads the caller's variables
///  without a closure to carry them.
/// A measurement that is never finished leaves its sampling thread running,
/// as a call
///  that panicked between the two always did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const measurement = QuietMeasurement.begin('gateway call');
/// const value = await post(body);
/// measurement.finish();
/// ```
pub struct QuietMeasurement {
    /// The ledger entry the longest gap is noted under.
    step: String,
    /// Tells the sampling thread that the call is over.
    done: Arc<AtomicBool>,
    /// The sampling thread,
    /// or nothing where the thread's number cannot be read or the thread could not start.
    sampler: Option<std::thread::JoinHandle<Duration>>,
}

impl QuietMeasurement {
    /// Start sampling the calling thread now,
    /// to be noted in the ledger as `step` at `finish`.
    pub fn begin(step: &str) -> Self {
        let done: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let Some(tid) = current_thread_id() else {
            return Self { step: step.to_string(), done, sampler: None };
        };
        let finished: Arc<AtomicBool> = Arc::clone(&done);
        let sampler: Option<std::thread::JoinHandle<Duration>> = std::thread::Builder::new()
            .name(String::from("cli-git-quiet-measure"))
            .spawn(move || {
                let mut clock: ThreadClock = ThreadClock::start(tid, Duration::MAX, true);
                while !finished.load(Ordering::Acquire) {
                    // The result is unused: this clock only measures, and its window is never reached.
                    let _quiet = clock.quiet();
                    std::thread::sleep(MEASURE_PAUSE);
                }
                return clock.longest_quiet();
            })
            .ok();
        return Self { step: step.to_string(), done, sampler };
    }

    /// Stop sampling and keep the longest gap in the ledger.
    pub fn finish(self) {
        self.done.store(true, Ordering::Release);
        if let Some(handle) = self.sampler
            && let Ok(gap) = handle.join()
        {
            crate::quiet_ledger::note(self.step.as_str(), gap, false);
        }
    }
}

/// How long the measuring thread sleeps between two looks at its clock.
const MEASURE_PAUSE: Duration = Duration::from_millis(20);

/// The thread's signs are controlled in the test module.
#[cfg(test)]
#[path = "thread_quiet_tests.rs"]
mod tests;
