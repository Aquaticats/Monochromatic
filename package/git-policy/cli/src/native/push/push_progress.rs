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

/// The patience a clock keeps,
/// and what a stop reports.
use crate::bounded_push::{StopCause, StopReport, StopWindow};
/// Whether a control holds a limit for an ordering it needs first.
use crate::push_limit_held_for_an_ordering::limit_has_passed;
use crate::push_deadline_and_patience_settings::Patience;
/// The processes below the push,
/// and the identity of each.
use crate::process_tree::{TreeMember, descendants_of, member_of};
/// The connected sockets of the machine with their byte counts,
/// and the sockets a process holds.
use crate::wait_ceilings::CONNECTED_PUSH_QUIET_WINDOW;
use crate::tcp_sockets::{ESTABLISHED, established_sockets, socket_inodes_of};
/// What each process of the tree was last seen to have moved.
use std::collections::HashMap;
/// Time spans and the monotonic clock.
use std::time::{Duration, Instant};

/// How often the counters of the tree are read.
pub const SAMPLE_INTERVAL: Duration = Duration::from_millis(500);

/// What:
///  The bytes a process has read and written,
/// from its `/proc/<pid>/io`,
/// or nothing where they cannot be read.
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

/// Only Linux counts a process's bytes;
/// elsewhere nothing is measured.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
fn bytes_moved_by(_pid: i32) -> Option<u64> {
    return None;
}

/// What:
///  The state letter and the processor ticks (`utime` plus `stime`) of a `/proc/<pid>/stat` line.
/// Why:
///   The command name sits in parentheses and may hold spaces and parentheses,
/// so the fields are read after the last
///  closing parenthesis;
/// the state is the first of them and the two times are the twelfth and thirteenth.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [state, , , , , , , , , , , utime, stime] = text.slice(text.lastIndexOf(')') + 2).split(' ');
/// ```
pub(crate) fn parse_activity(stat: &str) -> Option<(char, u64)> {
    let after_name: &str = stat.get(stat.rfind(')')? + 2..)?;
    let fields: Vec<&str> = after_name.split(' ').collect();
    let state: char = fields.first()?.chars().next()?;
    let user: u64 = fields.get(11)?.parse().ok()?;
    let system: u64 = fields.get(12)?.parse().ok()?;
    return Some((state, user.saturating_add(system)));
}

/// Whether a state letter is one of a task that is at work:
/// able to run (`R`) or waiting for the disk (`D`).
pub(crate) fn is_working(state: char) -> bool {
    return state == 'R' || state == 'D';
}

/// What:
///  A process's processor ticks,
/// and whether any of its tasks is able to run or waiting for the disk.
/// Why:
///   A thread that waits for the disk (an `fsync`) is in state `D` while the process's own state may be `S`,
/// so every task
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

/// Only Linux reads what a process is doing;
/// elsewhere nothing is.
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

/// What one look at one process found:
/// the bytes it moved since the look before,
/// and whether it used processor time or has a task at work.
/// Why:
///   The signs are kept apart because a process that holds the connection to the remote may be polling it,
/// and a poll is not work the remote sees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Signs = { moved: boolean; busy: boolean };
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Signs {
    /// Bytes read or written since the look before.
    pub moved: bool,
    /// Processor time used since the look before,
    /// or a task able to run or waiting for the disk now.
    pub busy: bool,
}

/// The sockets one process of the tree holds,
/// with its number.
fn inodes_of(member: &TreeMember) -> (i32, Vec<u64>) {
    return (member.pid, socket_inodes_of(member.pid));
}

/// Whether a process of the tree holds no socket at all.
fn holds_no_socket(held: &(i32, Vec<u64>)) -> bool {
    return held.1.is_empty();
}

/// What:
///  The work of the tree under a root so far,
/// and whether a sample saw any sign of work since the one before.
/// Why:
///   A process that ends takes its counters with it,
/// so each process is remembered by its number and start time,
/// and only
///  what it moved or used since it was last seen is added;
/// a task able to run or waiting for the disk is a sign of work at
///  the sample that finds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Meter { moved = 0; sample(rootPid): boolean }
/// ```
#[derive(Debug, Default)]
pub struct Meter {
    /// What each process of the tree had moved and used when it was last seen:
    /// bytes,
    /// then processor ticks.
    seen: HashMap<(i32, u64), (u64, u64)>,
    /// Every byte the tree has moved since the first sample.
    moved: u64,
    /// The bytes each connected socket of the tree had sent (acknowledged) and received when it was last seen,
    /// by inode.
    sockets: HashMap<u64, u64>,
    /// Every byte the tree's connected sockets have carried since the first sample.
    socket_moved: u64,
    /// Whether the last sample found a process of the tree holding a connected socket.
    connected: bool,
    /// The processes of the tree that held a connected socket at the last sample:
    /// the helpers that speak to the remote.
    holders: Vec<i32>,
    /// How many times a sample listed the processes of the whole machine to find the tree;
    /// a stand-in for the cost of a
    ///  sample,
    /// which grows with the number of processes the machine runs.
    scans: u64,
}

/// What a push that waits for its remote with the connection open has shown,
/// once it has been silent for the stall window.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaitReport {
    /// How long the tree has shown no sign of work,
    /// with its connection open.
    pub silent: Duration,
    /// How long the push has run.
    pub elapsed: Duration,
    /// The connected window the run will be stopped by if it stays silent,
    /// so the line a person reads names the limit that will end it.
    pub window: Duration,
}

impl Meter {
    /// A meter that has seen nothing.
    pub fn new() -> Self {
        return Self::default();
    }

    /// Every byte the tree has moved so far,
    /// on its pipes and files and on its connected sockets.
    pub fn moved(&self) -> u64 {
        return self.moved.saturating_add(self.socket_moved);
    }

    /// How many samples listed the machine's processes to find the tree.
    pub fn scans(&self) -> u64 {
        return self.scans;
    }

    /// Every byte the tree's connected sockets have sent (acknowledged) or received since the first sample.
    pub fn socket_bytes(&self) -> u64 {
        return self.socket_moved;
    }

    /// Whether the last sample found the connection to a remote open:
    /// some process of the tree holds a socket in state
    ///  `ESTABLISHED`.
    /// This is the whole evidence of "connected";
    /// the phase of the push (making the pack,
    /// sent,
    /// checked)
    ///  is not consulted,
    /// because a connection that is open is a remote that has not answered yet,
    /// whatever the phase.
    ///  Where the kernel's socket list cannot be read,
    /// nothing says the connection is open,
    /// and the stall rule applies.
    pub fn connected(&self) -> bool {
        return self.connected;
    }

    /// Take one look at the connected sockets of the tree:
    /// true when a socket it has seen before carried bytes since the last look,
    /// and records the processes that hold them.
    /// Why a socket seen for the first time is kept apart:
    ///   Its counters stand for the handshake and the request this side wrote,
    /// not for an answer from the remote,
    /// so a client that goes on reconnecting would reset the wait for its remote for good.
    fn observe_sockets(&mut self, members: &[TreeMember]) -> bool {
        let held: Vec<(i32, Vec<u64>)> = members.iter().map(inodes_of).collect();
        self.connected = false;
        self.holders.clear();
        if held.iter().all(holds_no_socket) {
            return false;
        }
        let Some(connected) = established_sockets() else {
            return false;
        };
        let mut progressed: bool = false;
        for socket in connected {
            if socket.state != ESTABLISHED {
                continue;
            }
            let mut carried_by_a_holder: bool = false;
            for (pid, inodes) in &held {
                if inodes.contains(&socket.inode) {
                    carried_by_a_holder = true;
                    if !self.holders.contains(pid) {
                        self.holders.push(*pid);
                    }
                }
            }
            if !carried_by_a_holder {
                continue;
            }
            self.connected = true;
            let total: u64 = socket.bytes_acked.saturating_add(socket.bytes_received);
            let seen_before: Option<u64> = self.sockets.insert(socket.inode, total);
            let carried: u64 = match seen_before {
                Some(before) => total.saturating_sub(before),
                None => 0,
            };
            self.socket_moved = self.socket_moved.saturating_add(carried);
            if carried > 0 {
                progressed = true;
            }
        }
        return progressed;
    }

    /// What:
    ///  Take one look at one process,
    /// and say which of its signs changed:
    /// bytes moved,
    /// or processor time used or a task at work.
    /// Why:
    ///   The two are kept apart because they say different things about a process that holds the connection:
    /// its processor time is a poll of the socket,
    /// and its bytes are its own account of waiting (`Meter::look`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { moved: bytes > before, busy: ticks > ticksBefore || working }
    /// ```
    fn observe(&mut self, key: (i32, u64), bytes: u64, ticks: u64, working: bool) -> Signs {
        let (bytes_before, ticks_before): (u64, u64) = self.seen.insert(key, (bytes, ticks)).unwrap_or((0, 0));
        let moved_now: u64 = bytes.saturating_sub(bytes_before);
        self.moved = self.moved.saturating_add(moved_now);
        return Signs { moved: moved_now > 0, busy: ticks > ticks_before || working };
    }

    /// Read the counters and the states of the root and everything below it;
    /// true when any of them shows work since the last
    ///  sample.
    /// A root that cannot be read (it ended) shows nothing.
    pub fn sample(&mut self, root: i32) -> bool {
        return self.look(root, true);
    }

    /// The first look at a child that has just been started:
    /// the root alone.
    ///  Nothing is below a process that has just been started,
    /// and a process it starts later is found by the next sample and
    ///  counted at the first sight,
    /// so the listing of every process the machine runs (a status file for each,
    /// 7 ms for 841
    ///  processes) would find nothing and be paid by every Git the wrapper starts,
    /// however briefly it lives.
    pub fn sample_root(&mut self, root: i32) -> bool {
        return self.look(root, false);
    }

    /// What:
    ///  One look at the root,
    /// and at everything below it when `whole_tree` asks for it.
    /// Why the sockets are looked at first:
    ///   The holders of the connections are known from this look then,
    /// and a holder's own processor time is what a helper that polls its silent socket spends,
    /// never work the remote sees.
    /// Every byte the tree moves is work,
    /// and so is the processor time of every process that does not hold a connection:
    /// the pack a push is making,
    /// a delta search,
    /// the listing of what to send.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// progressed = observeSockets(members); for (const member of members) { const s = observe(member); if (s.moved || (s.busy && !holders.includes(member.pid))) progressed = true; }
    /// ```
    fn look(&mut self, root: i32, whole_tree: bool) -> bool {
        let mut members: Vec<TreeMember> = member_of(root).into_iter().collect();
        if whole_tree {
            self.scans += 1;
            members.extend(descendants_of(root));
        }
        // The connected sockets count too: `libcurl` moves the bytes of an HTTP push with `send` and `recv`, which `rchar` and
        //  `wchar` do not see.
        let mut progressed: bool = self.observe_sockets(members.as_slice());
        for member in &members {
            let bytes: Option<u64> = bytes_moved_by(member.pid);
            let activity: Option<(u64, bool)> = activity_of(member.pid);
            if bytes.is_none() && activity.is_none() {
                continue;
            }
            let (ticks, working): (u64, bool) = activity.unwrap_or((0, false));
            let signs: Signs = self.observe((member.pid, member.started), bytes.unwrap_or(0), ticks, working);
            // A process that holds the connection reports its own wait while it waits, and what it writes about itself is no
            //  work the remote sees; everything it really sends shows on the socket's own counters, which are counted above.
            if (signs.moved || signs.busy) && !self.holders.contains(&member.pid) {
                progressed = true;
            }
        }
        return progressed;
    }
}

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
        };
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
    fn report(&self, window: StopWindow, limit: Duration, now: Instant) -> StopReport {
        return StopReport {
            window,
            limit,
            moved: self.meter.as_ref().map_or(0, Meter::moved),
            elapsed: now.duration_since(self.started),
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
            let progressed: bool = if self.looked { meter.sample(self.root) } else { meter.sample_root(self.root) };
            self.looked = true;
            if progressed {
                self.longest_quiet = self.longest_quiet.max(now.saturating_duration_since(self.last_progress));
                self.last_progress = now;
                self.waiting_since = None;
            } else if meter.connected() {
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
