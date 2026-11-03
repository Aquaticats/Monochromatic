//! What:
//!  The progress of a push: the bytes read and written by the push and everything it started, sampled at a small interval,
//!  and the clock that stops it on a stall (no bytes for the stall window) or at a ceiling (running that long in all).
//! Why:
//!  A fixed deadline stops a slow push that is still moving data and lets a stuck push take its whole deadline. The
//!  counters are `rchar` and `wchar` of `/proc/<pid>/io`, summed over the push's process tree, chosen by measurement on
//!  2026-11-03: a 30 MB push to a local bare remote moved 94 MB of `rchar` and 156 MB of `wchar` over its four processes while
//!  `read_bytes` was 64 KB and `write_bytes` came in delayed bursts from the receiving side's disk writes, so the block-layer
//!  counters see neither pipes nor sockets and lag behind the transfer. `rchar` and `wchar` count `read` and `write` calls,
//!  which is how Git moves a pack between its processes (pipes), how `ssh` and Go programs (`git-lfs`) move it to a socket,
//!  and how `git-remote-https` receives it from `git push`. They do not count `send` and `recv` (measured: 102 bytes of `rchar`
//!  for 100000 bytes through `send`/`recv`, 100102 through `read`), so a curl-only stretch is seen through the pipe that feeds
//!  or drains it, which backs up as soon as the socket stops moving.
//!  Where `/proc/<pid>/io` does not exist (no `/proc`, or a kernel without task I/O accounting) nothing is measured and the
//!  stall window is a fixed deadline, as before.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const clock = new Clock(child.pid, { stallMs, ceilingMs }); while (!exited) { const stop = clock.check(); if (stop) return stop; }
//! ```

/// The patience a clock keeps, and what a stop reports.
use super::bounded_push::{Patience, StopCause, StopReport};
/// The processes below the push, and the identity of each.
use super::process_tree::{TreeMember, descendants_of, member_of};
/// What each process of the tree was last seen to have moved.
use std::collections::HashMap;
/// Time spans and the monotonic clock.
use std::time::{Duration, Instant};

/// How often the counters of the tree are read.
pub const SAMPLE_INTERVAL: Duration = Duration::from_millis(500);

/// What:
///  The bytes a process has read and written, from its `/proc/<pid>/io`, or nothing where they cannot be read.
/// Why:
///   `rchar` and `wchar` are the counters that see pipes and sockets read and written with `read` and `write`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const io = Object.fromEntries(readFileSync(`/proc/${pid}/io`, 'utf8').split('\n').map((line) => line.split(': ')));
/// return Number(io.rchar) + Number(io.wchar);
/// ```
#[cfg(target_os = "linux")]
fn bytes_moved_by(pid: i32) -> Option<u64> {
    let text: String = std::fs::read_to_string(format!("/proc/{pid}/io")).ok()?;
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

/// Only Linux counts a process's bytes; elsewhere nothing is measured.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
fn bytes_moved_by(_pid: i32) -> Option<u64> {
    return None;
}

/// Whether this system counts the bytes of a process at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// existsSync('/proc/self/io')
/// ```
fn system_measures() -> bool {
    return bytes_moved_by(i32::try_from(std::process::id()).unwrap_or(1)).is_some();
}

/// What:
///  The bytes the tree under a root has moved so far, and whether a sample saw more than the one before.
/// Why:
///   A process that ends takes its counters with it, so the total of the live processes can fall; each process is
///  remembered by its number and start time, and only what it moved since it was last seen is added.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Meter { moved = 0; sample(rootPid): boolean | undefined }
/// ```
#[derive(Debug, Default)]
pub struct Meter {
    /// What each process of the tree had moved when it was last seen.
    seen: HashMap<(i32, u64), u64>,
    /// Everything the tree has moved since the first sample.
    moved: u64,
}

impl Meter {
    /// A meter that has seen nothing.
    pub fn new() -> Self {
        return Self::default();
    }

    /// Everything the tree has moved so far.
    pub fn moved(&self) -> u64 {
        return self.moved;
    }

    /// Read the counters of the root and everything below it; true when any of them moved bytes since the last sample.
    ///  A root that cannot be read (it ended) shows nothing.
    pub fn sample(&mut self, root: i32) -> bool {
        let mut members: Vec<TreeMember> = member_of(root).into_iter().collect();
        members.extend(descendants_of(root));
        let mut progressed: bool = false;
        for member in members {
            let Some(total) = bytes_moved_by(member.pid) else {
                continue;
            };
            let before: u64 = self.seen.insert((member.pid, member.started), total).unwrap_or(0);
            let delta: u64 = total.saturating_sub(before);
            if delta > 0 {
                progressed = true;
                self.moved = self.moved.saturating_add(delta);
            }
        }
        return progressed;
    }
}

/// What:
///  The clock of one bounded child: it says when the child has stalled, run past its ceiling or passed its fixed deadline.
/// Why:
///   The waiting loop asks it every few milliseconds; it reads the counters only every `SAMPLE_INTERVAL`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Clock { check(): StopCause | undefined }
/// ```
pub struct Clock {
    /// The process whose tree is measured.
    root: i32,
    /// What the child may do: a fixed time, or a stall window under a ceiling.
    patience: Patience,
    /// When the child started.
    started: Instant,
    /// When the tree last moved bytes (or the start, before it first does).
    last_progress: Instant,
    /// When the counters are next read.
    next_sample: Instant,
    /// The counters, or nothing where the system does not count bytes and the stall window is a fixed deadline.
    meter: Option<Meter>,
}

impl Clock {
    /// Start the clock of a child that has just been started.
    pub fn start(pid: u32, patience: Patience) -> Self {
        let now: Instant = Instant::now();
        let measured: bool = matches!(patience, Patience::Progress { .. }) && system_measures();
        return Self {
            root: i32::try_from(pid).unwrap_or(i32::MAX),
            patience,
            started: now,
            last_progress: now,
            next_sample: now,
            meter: if measured { Some(Meter::new()) } else { None },
        };
    }

    /// The same clock as on a system that counts no bytes, for the control of that fallback.
    #[cfg(test)]
    pub fn unmeasured(mut self) -> Self {
        self.meter = None;
        return self;
    }

    /// The report of a stop now, for the limit that was hit.
    fn report(&self, limit: Duration, now: Instant) -> StopReport {
        return StopReport { limit, moved: self.meter.as_ref().map_or(0, Meter::moved), elapsed: now.duration_since(self.started) };
    }

    /// Whether the child must be stopped now, and why.
    pub fn check(&mut self) -> Option<StopCause> {
        let now: Instant = Instant::now();
        let elapsed: Duration = now.duration_since(self.started);
        let (stall, ceiling): (Duration, Duration) = match self.patience {
            Patience::Fixed(limit) => return (elapsed >= limit).then_some(StopCause::Deadline),
            Patience::Progress { stall, ceiling } => (stall, ceiling),
        };
        let Some(meter) = self.meter.as_mut() else {
            // Nothing is measured here, so the stall window is the whole deadline.
            return (elapsed >= stall).then_some(StopCause::Deadline);
        };
        if now >= self.next_sample {
            if meter.sample(self.root) {
                self.last_progress = now;
            }
            self.next_sample = now + SAMPLE_INTERVAL;
        }
        if elapsed >= ceiling {
            return Some(StopCause::Ceiling(self.report(ceiling, now)));
        }
        if now.duration_since(self.last_progress) >= stall {
            return Some(StopCause::Stalled(self.report(stall, now)));
        }
        return None;
    }
}

/// Progress behavior is controlled in the test module.
#[cfg(test)]
#[path = "push_progress_tests.rs"]
pub(crate) mod tests;
