//! What:
//!  What one look at the push's tree finds:
//!  the bytes each process has moved and the processor time it has used,
//!  the connected sockets of the run with the bytes they carried,
//!  and the work of the tree summed so far.
//! Why:
//!  `push_progress.rs` owns the clock that decides when a push is stopped;
//! the counting outgrew that file
//!  (`rust/max-lines`),
//! and it is the half that reads `/proc`:
//! `rchar` plus `wchar` of `/proc/<pid>/io`,
//! `utime` plus
//!  `stime` of `/proc/<pid>/stat`,
//! and the state letter of every task of every process of the tree.
//!  Measured on 2026-11-03,
//! which decided the three signs:
//! a 30 MB push to a local bare remote moved 94 MB of `rchar` and 156 MB of `wchar` over its four
//!  processes while `read_bytes` was 64 KB,
//! `rchar` and `wchar` count `read` and `write` calls and not
//!  `send` and `recv`,
//! and a `dd` of 64 MB with `conv=fsync` sat in state `D` through six of seven looks with no
//!  processor tick.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class Meter { moved = 0; sample(rootPid): boolean }
//! ```

/// The processes below the push,
/// and the identity of each.
use crate::process_tree::{TreeMember, descendants_of, member_of};
/// What one look at one process found,
/// and the identities a look at a tree's sockets works with.
pub use crate::push_progress_signs::{Signs, holds_no_socket, inodes_of, start_time_of};
/// The connected sockets of the machine with their byte counts,
/// and the sockets a process holds.
use crate::tcp_sockets::{ESTABLISHED, established_sockets};
/// What each process of the tree was last seen to have moved.
use std::collections::HashMap;
/// Time spans.
use std::time::Duration;

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
pub(crate) fn bytes_moved_by(pid: i32) -> Option<u64> {
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
pub(crate) fn bytes_moved_by(_pid: i32) -> Option<u64> {
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
pub(crate) fn activity_of(pid: i32) -> Option<(u64, bool)> {
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
pub(crate) fn activity_of(_pid: i32) -> Option<(u64, bool)> {
    return None;
}

/// Whether this system counts the bytes of a process at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// existsSync('/proc/self/io')
/// ```
pub(crate) fn system_measures() -> bool {
    return bytes_moved_by(i32::try_from(std::process::id()).unwrap_or(1)).is_some();
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
    /// What:
    ///  The bytes of the transport the sample just taken carried:
    ///  the connected sockets' own counters in both directions,
    ///  and what every process that holds no connection read or wrote,
    ///  which is the transport itself when the remote is a path.
    /// Why:
    ///   The least traffic a push's connection must carry is judged on this sum (`push_minimum_rate.rs`),
    /// and the two
    ///  belong together:
    /// a keepalive packet arrives on a socket or through a pipe,
    /// and neither is progress of a
    ///  transfer.
    carried_last: u64,
    /// Whether the sample just taken found a sign of work that is not traffic:
    /// processor time of a process that holds no connection,
    /// or a task of it able to run or waiting for the disk.
    other_last: bool,
    /// Whether the last sample found a process of the tree holding a connected socket.
    connected: bool,
    /// Every process of the tree that has been seen to hold a connected socket,
    /// by its number and start time:
    /// the helpers that speak to the remote,
    /// whose own bytes and processor time are not work.
    pub(crate) holders: Vec<(i32, u64)>,
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

    /// What the sample just taken found the transport carried (`push_minimum_rate.rs`):
    /// the connected sockets' own counters in both directions,
    /// and what every process that holds no connection read or wrote.
    pub fn carried_last_sample(&self) -> u64 {
        return self.carried_last;
    }

    /// Whether the sample just taken found work that is not traffic:
    /// processor time of a process that holds no connection,
    /// or a task of it able to run or waiting for the disk.
    pub fn other_work_last_sample(&self) -> bool {
        return self.other_last;
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
    /// Why the processes that held one are remembered for the life of the clock:
    ///   A look that cannot read the kernel's socket list answers nothing about who holds what,
    /// and a holder whose own meter counted again on such a look would reset the window:
    /// under load those looks fail now and then,
    /// and the window would then never be reached.
    /// Each holder is remembered by its number and start time,
    /// so a number the kernel reuses is not taken for the process that held a connection before.
    pub(crate) fn observe_sockets(&mut self, members: &[TreeMember]) -> bool {
        let held: Vec<(i32, Vec<u64>)> = members.iter().map(inodes_of).collect();
        self.connected = false;
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
                    let holder: (i32, u64) = (*pid, start_time_of(members, *pid));
                    if !self.holders.contains(&holder) {
                        self.holders.push(holder);
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
            self.carried_last = self.carried_last.saturating_add(carried);
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
    pub(crate) fn observe(&mut self, key: (i32, u64), bytes: u64, ticks: u64, working: bool) -> Signs {
        let (bytes_before, ticks_before): (u64, u64) = self.seen.insert(key, (bytes, ticks)).unwrap_or((0, 0));
        let moved_now: u64 = bytes.saturating_sub(bytes_before);
        self.moved = self.moved.saturating_add(moved_now);
        return Signs { moved: moved_now > 0, moved_bytes: moved_now, busy: ticks > ticks_before || working };
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
        // The markers of this look are its own: a sample that says nothing answers nothing about the last one.
        self.carried_last = 0;
        self.other_last = false;
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
            // A process that holds no connection is the transport itself when the remote is a path (the push writes the pack
            //  into `receive-pack`'s pipe and reads the sideband back), so what it moves is traffic, judged by the floor.
            if self.holders.contains(&(member.pid, member.started)) {
                continue;
            }
            self.carried_last = self.carried_last.saturating_add(signs.moved_bytes);
            if signs.busy {
                self.other_last = true;
            }
            if signs.moved || signs.busy {
                progressed = true;
            }
        }
        return progressed;
    }
}
