//! What:
//!  The progress of a push: whether the push and everything it started are at work, sampled at a small interval, and the
//!  clock that stops it on a stall (no sign of work for the stall window) or at a ceiling (running that long in all).
//! Why:
//!  A fixed deadline stops a slow push that is still working and lets a stuck push take its whole deadline. A process shows
//!  that it works in three ways, each read from `/proc`, and only a tree that shows none of them is stalled:
//!       bytes moved: `rchar` plus `wchar` of `/proc/<pid>/io`, summed over the tree;
//!       processor time: `utime` plus `stime` of `/proc/<pid>/stat` growing;
//!       a task able to run or waiting for the disk: state `R` or `D` in `/proc/<pid>/task/<tid>/stat`.
//!  Measured on 2026-11-03, which decided the three:
//!       a 30 MB push to a local bare remote moved 94 MB of `rchar` and 156 MB of `wchar` over its four processes while
//!       `read_bytes` was 64 KB and `write_bytes` came late and in bursts, so the block-layer counters see neither pipes nor
//!       sockets and lag behind the transfer;
//!       `rchar` and `wchar` count `read` and `write` calls, not `send` and `recv` (102 bytes for 100000 bytes through
//!       `send`/`recv`, 100102 through `read`), so a stretch that only curl performs shows through the pipe that feeds it;
//!       a `dd` of 64 MB with `conv=fsync` was in state `D` in six of seven looks and used 0 processor ticks (a tick is 10 ms),
//!       so a wait for the disk shows in neither bytes nor processor time, only in the state;
//!       a parent in `vfork` is in state `D` with 0 ticks, the way the controls hold a tree in disk wait exactly;
//!       a process reading a silent socket and a `sleep` are in state `S` with a constant tick count, so a wait on a server
//!       that says nothing is a stall.
//!  What each costs: bytes miss work that moves none; processor time makes a process that spins or polls look alive, and its
//!  resolution is a tick; state `D` makes a process wedged for ever in the kernel (a dead network file system) look alive, and
//!  state `R` one that is only waiting for a processor; the ceiling bounds all three. None of them sees a server that checks the
//!  push before it answers: that tree is in state `S`, indistinguishable from a dead peer, so a check longer than the stall
//!  window is stopped, and raising the window is the remedy, which the stop message names.
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

/// What:
///  The state letter and the processor ticks (`utime` plus `stime`) of a `/proc/<pid>/stat` line.
/// Why:
///   The command name sits in parentheses and may hold spaces and parentheses, so the fields are read after the last
///  closing parenthesis; the state is the first of them and the two times are the twelfth and thirteenth.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [state, , , , , , , , , , , utime, stime] = text.slice(text.lastIndexOf(')') + 2).split(' ');
/// ```
fn parse_activity(stat: &str) -> Option<(char, u64)> {
    let after_name: &str = stat.get(stat.rfind(')')? + 2..)?;
    let fields: Vec<&str> = after_name.split(' ').collect();
    let state: char = fields.first()?.chars().next()?;
    let user: u64 = fields.get(11)?.parse().ok()?;
    let system: u64 = fields.get(12)?.parse().ok()?;
    return Some((state, user.saturating_add(system)));
}

/// Whether a state letter is one of a task that is at work: able to run (`R`) or waiting for the disk (`D`).
fn is_working(state: char) -> bool {
    return state == 'R' || state == 'D';
}

/// What:
///  A process's processor ticks, and whether any of its tasks is able to run or waiting for the disk.
/// Why:
///   A thread that waits for the disk (an `fsync`) is in state `D` while the process's own state may be `S`, so every task
///  is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { cpu: parse(`/proc/${pid}/stat`).utime + stime, working: tasks(pid).some((task) => 'RD'.includes(task.state)) }
/// ```
#[cfg(target_os = "linux")]
fn activity_of(pid: i32) -> Option<(u64, bool)> {
    let text: String = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let (state, ticks): (char, u64) = parse_activity(text.as_str())?;
    let mut working: bool = is_working(state);
    if !working && let Ok(tasks) = std::fs::read_dir(format!("/proc/{pid}/task")) {
        for task in tasks.flatten() {
            if let Ok(line) = std::fs::read_to_string(task.path().join("stat"))
                && let Some((task_state, _)) = parse_activity(line.as_str())
                && is_working(task_state)
            {
                working = true;
                break;
            }
        }
    }
    return Some((ticks, working));
}

/// Only Linux reads what a process is doing; elsewhere nothing is.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
fn activity_of(_pid: i32) -> Option<(u64, bool)> {
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
///  The work of the tree under a root so far, and whether a sample saw any sign of work since the one before.
/// Why:
///   A process that ends takes its counters with it, so each process is remembered by its number and start time, and only
///  what it moved or used since it was last seen is added; a task able to run or waiting for the disk is a sign of work at
///  the sample that finds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Meter { moved = 0; sample(rootPid): boolean }
/// ```
#[derive(Debug, Default)]
pub struct Meter {
    /// What each process of the tree had moved and used when it was last seen: bytes, then processor ticks.
    seen: HashMap<(i32, u64), (u64, u64)>,
    /// Every byte the tree has moved since the first sample.
    moved: u64,
}

/// What a push that has sent its pack and waits for the server's checks has shown, once it has waited for the stall window.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaitReport {
    /// How long the tree has shown no sign of work, with its connection open.
    pub silent: Duration,
    /// How long the push has run.
    pub elapsed: Duration,
}

impl Meter {
    /// A meter that has seen nothing.
    pub fn new() -> Self {
        return Self::default();
    }

    /// Every byte the tree has moved so far.
    pub fn moved(&self) -> u64 {
        return self.moved;
    }

    /// Every byte the tree's connected sockets have sent (acknowledged) or received since the first sample.
    // RED SCAFFOLD: counts nothing yet.
    pub fn socket_bytes(&self) -> u64 {
        return 0;
    }

    /// Whether the last sample found the pack sent and the connection to the remote open.
    // RED SCAFFOLD: never waits yet.
    pub fn waiting_on_server(&self) -> bool {
        return false;
    }

    /// Take one look at one process: true when it moved bytes or used processor time since the last look, or is at work now.
    fn observe(&mut self, key: (i32, u64), bytes: u64, ticks: u64, working: bool) -> bool {
        let (bytes_before, ticks_before): (u64, u64) = self.seen.insert(key, (bytes, ticks)).unwrap_or((0, 0));
        let moved_now: u64 = bytes.saturating_sub(bytes_before);
        self.moved = self.moved.saturating_add(moved_now);
        return moved_now > 0 || ticks > ticks_before || working;
    }

    /// Read the counters and the states of the root and everything below it; true when any of them shows work since the last
    ///  sample. A root that cannot be read (it ended) shows nothing.
    pub fn sample(&mut self, root: i32) -> bool {
        let mut members: Vec<TreeMember> = member_of(root).into_iter().collect();
        members.extend(descendants_of(root));
        let mut progressed: bool = false;
        for member in members {
            let bytes: Option<u64> = bytes_moved_by(member.pid);
            let activity: Option<(u64, bool)> = activity_of(member.pid);
            if bytes.is_none() && activity.is_none() {
                continue;
            }
            let (ticks, working): (u64, bool) = activity.unwrap_or((0, false));
            if self.observe((member.pid, member.started), bytes.unwrap_or(0), ticks, working) {
                progressed = true;
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
        let measured: bool = matches!(patience, Patience::Progress { .. } | Patience::Push { .. }) && system_measures();
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

    /// The wait the clock has let pass instead of a stop, once the stall window ran out with the pack sent and the connection open.
    // RED SCAFFOLD: never waits yet.
    pub fn wait_report(&self) -> Option<WaitReport> {
        return None;
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
            Patience::Progress { stall, ceiling } | Patience::Push { stall, ceiling } => (stall, ceiling),
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
