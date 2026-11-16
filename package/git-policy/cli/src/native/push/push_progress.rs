//! What:
//!  The progress of a push:
//! whether the push and everything it started are at work,
//! sampled at a small interval,
//! and the
//!  clock that stops it on a stall (no sign of work for the stall window,
//! and no open connection to a remote).
//! Why:
//!  A fixed deadline stops a slow push that is still working and lets a stuck push take its whole deadline.
//! A process shows
//!  that it works in three ways,
//! each read from `/proc`,
//! and only a tree that shows none of them is stalled:
//!       bytes moved:
//! `rchar` plus `wchar` of `/proc/<pid>/io`,
//! summed over the tree;
//!  for the push,
//! its fetch and its upload those bytes are traffic,
//! so they count only once they add up to the least
//!  traffic a connection must carry (`push_minimum_rate.rs`),
//! while every other run counts each byte as work;
//!       processor time:
//! `utime` plus `stime` of `/proc/<pid>/stat` growing;
//!       a task able to run or waiting for the disk:
//! state `R` or `D` in `/proc/<pid>/task/<tid>/stat`.
//!  Measured on 2026-11-03,
//! which decided the three:
//!       a 30 MB push to a local bare remote moved 94 MB of `rchar` and 156 MB of `wchar` over its four processes while
//!       `read_bytes` was 64 KB and `write_bytes` came late and in bursts,
//! so the block-layer counters see neither pipes nor
//!       sockets and lag behind the transfer;
//!       `rchar` and `wchar` count `read` and `write` calls,
//! not `send` and `recv` (102 bytes for 100000 bytes through
//!       `send`/`recv`,
//! 100102 through `read`),
//! so a stretch that only curl performs shows through the pipe that feeds it;
//! a server's keepalive for a remote that is a path arrives the same way,
//! five bytes at a
//!  time into the push's own standard input;
//!       a `dd` of 64 MB with `conv=fsync` was in state `D` in six of seven looks and used 0 processor ticks (a tick is 10 ms),
//!       so a wait for the disk shows in neither bytes nor processor time,
//! only in the state;
//!       a parent in `vfork` is in state `D` with 0 ticks,
//! the way the controls hold a tree in disk wait exactly;
//!       a process reading a silent socket and a `sleep` are in state `S` with a constant tick count,
//! so a wait on a server
//!       that says nothing shows nothing in the process;
//! only the connection's state shows it.
//!  Measured on 2026-11-11,
//! which decided what a tree with a connection open may count as work:
//! the helper that speaks HTTP for a push,
//! `git remote-http` and `git-remote-http`,
//! was charged one processor tick every three seconds while it polled a server that had taken the
//!  connection and never answered,
//! and moved not one byte.
//! A clock that counts those ticks as work never reaches its connected window:
//! the service's push to such a server ran for 290 to 330 seconds with a 150 second window
//! (the audit's F8),
//! and a tree of one holder that only ticks never ended at all.
//!  So while a connection is open,
//! neither the processor time nor the bytes of the process that holds it are work:
//! that process polls the socket and prints its own account of waiting.
//! Measured the same day,
//! which decided the bytes too:
//! `git-lfs` writes fourteen bytes of its upload meter to its standard error about every ten seconds
//! while it waits,
//! and each of those lines reset the window,
//! so an upload that stayed on one connection was not ended in more than 260 seconds.
//! What that process really sends is on the connection,
//! whose carried bytes are counted here,
//! and the work of every process that holds no connection is still work:
//! the pack a push is making,
//! the delta search,
//! the listing of what to send.
//!  A socket carries no progress on the look that first sees it,
//! because its counters then stand for this side's handshake and request,
//! never for an answer from the remote.
//!  What each costs:
//! bytes miss work that moves none;
//! processor time makes a process that spins or polls look alive,
//! and its
//!  resolution is a tick;
//! state `D` makes a process wedged for ever in the kernel (a dead network file system) look alive,
//! and
//!  state `R` one that is only waiting for a processor;
//! no total time bounds them (owner decision,
//! see below).
//! None of them sees
//!  a server that checks the push before it answers:
//! that tree is in state `S`,
//! indistinguishable from a dead peer by its
//!  processes alone.
//! What tells them apart is the connection:
//! a process of the tree that holds a TCP socket in state
//!  `ESTABLISHED` (read from the kernel's socket diagnostics,
//! `tcp_sockets.rs`) has a remote that has not closed it,
//! and a tree
//!  with such a socket is waited for while it works.
//! The owner's answer "Wait while connected (Recommended)"
//!  (2026-11-03) was replaced the same day by "Bound it,
//! keep retrying":
//!  a connection that stays open with nothing moving for `CONNECTED_PUSH_QUIET_WINDOW` ends the run as a stall,
//! and the
//!  service's retry plan keeps the push and tries it again.
//! The rest is pick 5 of `doc/planning/cli-git-shipped-limits-audit.md`,
//! which the coordinator made
//!  under the owner's delegation ("Pick options as if I had picked them",
//! 2026-11-03) and the owner may veto:
//! no ceiling
//!  for a run that shows progress,
//! no low-speed abort.
//!  What the code cannot observe:
//! a peer that vanished without a reset leaves the socket `ESTABLISHED` until TCP gives up
//!  (retransmission for unacknowledged data,
//! keepalive probes for an idle connection),
//! which the owner accepted.
//!  Where `/proc/<pid>/io` does not exist (no `/proc`,
//! or a kernel without task I/O accounting) nothing is measured and the
//!  stall window is a fixed deadline,
//! as before.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const clock = new Clock(child.pid, { stallMs }); while (!exited) { const stop = clock.check(); if (stop) return stop; }
//! ```

/// What a look at the push's tree found (`push_progress_meter.rs`),
/// and the two readings of a `/proc` line the run-progress controls share.
pub use crate::push_progress_meter::{Meter, SAMPLE_INTERVAL, WaitReport};
pub(crate) use crate::push_progress_meter::{is_working, parse_activity};
/// What the clock reads of the system itself.
use crate::push_progress_meter::system_measures;
/// Whether a control holds a limit for an ordering it needs first.
use crate::push_limit_held_for_an_ordering::limit_has_passed;
/// The patience a clock keeps,
/// and what a stop reports.
use crate::bounded_push::{StopCause, StopReport, StopWindow};
use crate::push_deadline_and_patience_settings::Patience;
/// The window a connected push may stay silent for.
use crate::wait_ceilings::CONNECTED_PUSH_QUIET_WINDOW;
/// Time spans and the monotonic clock.
use std::time::{Duration, Instant};

/// What:
///  The clock of one bounded child:
/// it says when the child has stalled or passed its fixed deadline.
/// Why:
///   The waiting loop asks it every few milliseconds;
/// it reads the counters only every `SAMPLE_INTERVAL`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Clock { check(): StopCause | undefined }
/// ```
pub struct Clock {
    /// The process whose tree is measured.
    root: i32,
    /// What the child may do:
    /// a fixed time,
    /// or a stall window that an open connection suspends.
    patience: Patience,
    /// When the child started.
    started: Instant,
    /// When the tree last moved bytes (or the start,
    /// before it first does).
    last_progress: Instant,
    /// When the counters are next read.
    next_sample: Instant,
    /// The counters,
    /// or nothing where the system does not count bytes and the stall window is a fixed deadline.
    meter: Option<Meter>,
    /// Since when the tree has shown no sign of work while its connection was open;
    /// nothing while it did or was not.
    waiting_since: Option<Instant>,
    /// The wait,
    /// once the stall window has run out with the connection open.
    waited: Option<WaitReport>,
    /// Whether the wait has been handed to the caller (`take_wait_note`),
    /// which says so once.
    wait_noted: bool,
    /// The longest stretch between two samples that showed a sign of work,
    /// as far as the samples saw (the resolution is
    ///  `SAMPLE_INTERVAL`);
    /// a connected wait is not a quiet stretch.
    longest_quiet: Duration,
    /// Whether the first look,
    /// which is of the root alone,
    /// has been taken.
    looked: bool,
    /// Whether a system that counts nothing falls back to the stall window as a fixed deadline;
    /// a run that must never end by
    ///  wall time clears it and then goes unjudged where nothing is counted.
    wall_fallback: bool,
    /// How long a tree with an open connection may show no sign of work before it is stopped (`wait_ceilings.rs`).
    connected_window: Duration,
    /// The least traffic the connection may carry in one connected window and still count as progress (`push_minimum_rate.rs`):
    /// zero while bytes of the transport are progress by themselves,
    /// which every run but a push and its upload is.
    minimum_rate: u64,
    /// The bytes the connection has carried since the run last counted as working,
    /// which is what the minimum rate is judged on;
    /// nothing while no connection is open,
    /// because a stretch without one is judged by silence alone.
    carried: u64,
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
            looked: false,
            meter: if measured { Some(Meter::new()) } else { None },
            waiting_since: None,
            waited: None,
            wait_noted: false,
            longest_quiet: Duration::ZERO,
            wall_fallback: true,
            connected_window: CONNECTED_PUSH_QUIET_WINDOW,
            minimum_rate: 0,
            carried: 0,
        };
    }

    /// The same clock for a run whose transport bytes count as progress only once they add up to a floor within a window
    /// (`push_minimum_rate.rs`).
    pub fn with_minimum_rate(mut self, bytes: u64) -> Self {
        self.minimum_rate = bytes;
        return self;
    }

    /// The same clock for a run that must never end by wall time:
    /// where nothing is counted it is never judged stalled.
    pub fn without_wall_fallback(mut self) -> Self {
        self.wall_fallback = false;
        return self;
    }

    /// The same clock with another window for a tree whose connection is open and silent.
    pub fn with_connected_window(mut self, window: Duration) -> Self {
        self.connected_window = window;
        return self;
    }

    /// Whether this clock reads the tree's signs of work at all (Linux with `/proc/<pid>/io`),
    /// so a stall means quiet time.
    pub fn measures(&self) -> bool {
        return self.meter.is_some();
    }

    /// The longest stretch with no sign of work so far,
    /// the one still running included.
    /// A step that ended by itself gives the
    ///  figure of a healthy step,
    /// which is what an interval must be set above.
    pub fn longest_quiet(&self) -> Duration {
        return self.longest_quiet.max(Instant::now().saturating_duration_since(self.last_progress));
    }

    /// The same clock as on a system that counts no bytes,
    /// for the control of that fallback.
    #[cfg(test)]
    pub fn unmeasured(mut self) -> Self {
        self.meter = None;
        return self;
    }

    /// The wait the clock has let pass instead of a stop,
    /// once the stall window ran out with the connection open.
    pub fn wait_report(&self) -> Option<WaitReport> {
        return self.waited;
    }

    /// The wait,
    /// the first time it is asked for after the wait began,
    /// so the caller says it once.
    pub fn take_wait_note(&mut self) -> Option<WaitReport> {
        if self.wait_noted {
            return None;
        }
        let report: WaitReport = self.waited?;
        self.wait_noted = true;
        return Some(report);
    }

    /// The report of a stop now,
    /// for the window that was hit and the limit it carries.
    /// The bytes are those the connection carried in the stretch that was judged,
    /// which is what a stop under the minimum rate is read against,
    /// and the floor it was judged on.
    fn report(&self, window: StopWindow, limit: Duration, now: Instant) -> StopReport {
        return StopReport {
            window,
            limit,
            moved: self.meter.as_ref().map_or(0, Meter::moved),
            elapsed: now.duration_since(self.started),
            carried: self.carried,
            minimum: self.minimum_rate,
        };
    }

    /// Whether the child must be stopped now,
    /// and why.
    pub fn check(&mut self) -> Option<StopCause> {
        let now: Instant = Instant::now();
        let elapsed: Duration = now.duration_since(self.started);
        let stall: Duration = match self.patience {
            Patience::Fixed(limit) => return limit_has_passed(elapsed, limit).then_some(StopCause::Deadline),
            Patience::Progress { stall } => stall,
        };
        let Some(meter) = self.meter.as_mut() else {
            // Nothing is measured here, so the stall window is the whole deadline, unless the run must never end by wall time.
            return (self.wall_fallback && limit_has_passed(elapsed, stall)).then_some(StopCause::Deadline);
        };
        if now >= self.next_sample {
            let worked: bool = if self.looked { meter.sample(self.root) } else { meter.sample_root(self.root) };
            let carried: u64 = meter.carried_last_sample();
            let other: bool = meter.other_work_last_sample();
            let connected: bool = meter.connected();
            self.looked = true;
            // What the transport carried is judged over the window that bounds silence in the state the run is in
            //  (`push_minimum_rate.rs`): the connected window while a connection is open, and the stall window while none is,
            //  because the tally is cleared by every sign of progress and the window ends a run that shows none. The tally is
            //  not cleared when the connection closes: what the pipe carried for a remote that is a path is the same traffic,
            //  and only the state around the look changes.
            self.carried = self.carried.saturating_add(carried);
            let met: bool = self.minimum_rate == 0 || self.carried >= self.minimum_rate;
            let progressed: bool = worked && (other || met);
            if progressed {
                self.longest_quiet = self.longest_quiet.max(now.saturating_duration_since(self.last_progress));
                self.last_progress = now;
                self.waiting_since = None;
                self.carried = 0;
            } else if connected {
                // The connection to the remote is open: the remote has not answered yet, which shows nothing here. The silence of
                //  a wait is not counted against the window, so a connection that closes gets a whole window.
                let since: Instant = *self.waiting_since.get_or_insert(now);
                self.last_progress = now;
                if self.waited.is_none() && now.duration_since(since) >= stall {
                    self.waited = Some(WaitReport { silent: now.duration_since(since), elapsed, window: self.connected_window });
                }
                // The owner, 2026-11-03: "Bound it, keep retrying". A connection that stays open and silent for the connected
                //  window ends the push as a stall; the service's retry plan keeps the push and tries it again.
                if now.duration_since(since) >= self.connected_window {
                    return Some(StopCause::Stalled(self.report(StopWindow::Connected, self.connected_window, now)));
                }
            } else {
                self.waiting_since = None;
            }
            self.next_sample = now + SAMPLE_INTERVAL;
        }
        if now.duration_since(self.last_progress) >= stall {
            return Some(StopCause::Stalled(self.report(StopWindow::Stall, stall, now)));
        }
        return None;
    }
}

/// Progress behavior is controlled in the test module.
#[cfg(test)]
#[path = "push_progress_tests.rs"]
pub(crate) mod tests;
