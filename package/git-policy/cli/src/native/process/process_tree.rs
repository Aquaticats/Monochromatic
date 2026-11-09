//! What:
//!  The processes below one process:
//! found before it is stopped,
//! and signalled afterwards,
//! each only while it is still
//!  the process that was found.
//! Why:
//!  Stopping a process does not stop what it started.
//! A `git push` that is sent `SIGTERM` or `SIGKILL` leaves its
//!  transport helper (`git-remote-http`,
//! `ssh`) and the hooks it ran (`git-lfs`) alive,
//! orphaned to the init process,
//!  holding their connection until their own timeout ends them (measured:
//! a helper stayed 60 seconds,
//! a `git-lfs`
//!  that never exits stayed for good).
//! The tree has to be read before the root dies,
//! because a child's link to its
//!  parent is gone once the parent has exited.
//! A process number can be reused,
//! so each process is identified by the
//!  start time the kernel recorded for it,
//! and a number whose start time changed is not signalled.
//!  The list comes from `/proc`,
//! so only Linux finds any;
//! elsewhere the tree is empty and only the root is stopped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const tree = await descendantsOf(child.pid); child.kill('SIGTERM'); for (const process of tree) process.signalIfSame('SIGTERM');
//! ```

/// Time spans of a process's processor use.
use std::time::Duration;

/// One process of a tree and the start time that proves it is still that process.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeMember {
    /// The process number.
    pub pid: i32,
    /// The start time the kernel recorded,
    /// in clock ticks since boot (`/proc/<pid>/stat`,
    /// field 22).
    pub started: u64,
}

/// What:
///  The pid,
/// parent and start time of the process a `stat` file describes,
/// and whether it is a zombie.
/// Why:
///   The command name sits in parentheses and may hold spaces and parentheses,
/// so the fields are read after the last
///  closing parenthesis.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [state, ppid, ...rest] = text.slice(text.lastIndexOf(')') + 2).split(' ');
/// ```
#[cfg(target_os = "linux")]
fn parse_stat(text: &str) -> Option<(i32, u64, bool)> {
    let after_name: &str = text.get(text.rfind(')')? + 2..)?;
    let fields: Vec<&str> = after_name.split(' ').collect();
    // After the name: state (index 0), parent (1), ..., start time (19).
    let state: &str = fields.first()?;
    let parent: i32 = fields.get(1)?.parse().ok()?;
    let started: u64 = fields.get(19)?.parse().ok()?;
    return Some((parent, started, state == "Z"));
}

/// What:
///  Every living process below `root`,
/// children before grandchildren,
/// with their start times,
/// or nothing when `/proc` could not be read at all.
/// Why:
///   Read once,
/// before the root is signalled,
/// from one pass over `/proc`.
///  A caller that reads an empty tree as "the step ended" must be able to tell a tree with nothing in it
/// from a look that failed (a descriptor or a memory limit, a `/proc` that is not there):
/// the first is a fact about the step and the second a fact about the look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function descendantsOfRead(root: number): TreeMember[] | undefined;
/// ```
#[cfg(target_os = "linux")]
pub fn descendants_of_read(root: i32) -> Option<Vec<TreeMember>> {
    use std::collections::HashMap;
    let listing = std::fs::read_dir("/proc").ok()?;
    // Parent number to the children found under it.
    let mut children: HashMap<i32, Vec<TreeMember>> = HashMap::new();
    // How many entries name a process, and how many of those could be read.
    let mut numbered: usize = 0;
    let mut read: usize = 0;
    for entry in listing.flatten() {
        let file_name: std::ffi::OsString = entry.file_name();
        let Some(name) = file_name.to_str() else {
            continue;
        };
        let Ok(pid) = name.parse::<i32>() else {
            continue;
        };
        numbered = numbered.saturating_add(1);
        let Ok(text) = std::fs::read_to_string(entry.path().join("stat")) else {
            continue;
        };
        read = read.saturating_add(1);
        if let Some((parent, started, zombie)) = parse_stat(text.as_str())
            && !zombie
        {
            children.entry(parent).or_default().push(TreeMember { pid, started });
        }
    }
    // A listing that numbers processes and lets none of them be read is a look that failed, never a tree with nothing in it.
    if numbered > 0 && read == 0 {
        return None;
    }
    let mut found: Vec<TreeMember> = Vec::new();
    let mut frontier: Vec<i32> = vec![root];
    while let Some(parent) = frontier.pop() {
        for member in children.remove(&parent).unwrap_or_default() {
            frontier.push(member.pid);
            found.push(member);
        }
    }
    return Some(found);
}

/// What:
///  Every living process below `root`,
/// children before grandchildren,
/// with their start times.
/// Why:
///   Most callers stop what they find and have nothing to decide about a look that failed,
/// so they read the tree alone (`descendants_of_read`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function descendantsOf(root: number): Promise<TreeMember[]>;
/// ```
#[cfg(target_os = "linux")]
pub fn descendants_of(root: i32) -> Vec<TreeMember> {
    return descendants_of_read(root).unwrap_or_default();
}

/// Only Linux lists processes;
/// elsewhere no descendant is found and only the root is stopped.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn descendants_of_read(_root: i32) -> Option<Vec<TreeMember>> {
    return Some(Vec::new());
}

/// Only Linux lists processes;
/// elsewhere no descendant is found and only the root is stopped.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn descendants_of(_root: i32) -> Vec<TreeMember> {
    return Vec::new();
}

/// What:
///  The process numbers below `root`,
/// read from the kernel's per-thread child lists.
/// Why:
///   A notice that looks many times must not pay a pass over `/proc` for each look.
/// Measured on 2026-11-09 on this host (1,210 entries in `/proc`;
/// the load average was 44 on 6 threads when the runs ended and 91 an hour before them):
/// one pass reading every `/proc/<pid>/stat` took 17.6 ms at the median and 19.7 ms at most over 30 looks,
/// and one walk of the child lists 19.6 microseconds at the median and 26.3 at most over 30 walks,
/// about 900 times cheaper.
///  The kill still reads all of `/proc` (`descendants_of`),
/// because that read names the start time that proves a process is still the one that was found.
/// Nothing is returned when the kernel has no child lists (`CONFIG_PROC_CHILDREN` off or a `/proc` that is not there),
/// so the caller can read `/proc` instead.
///  A child of any thread of `root` is below it:
/// the list of the thread that started it is the one that names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const numbers = descendantNumbersFromThreadLists(root); // undefined without the kernel's lists
/// ```
#[cfg(target_os = "linux")]
pub fn descendant_numbers_from_thread_lists(root: i32) -> Option<Vec<i32>> {
    let mut frontier: Vec<i32> = child_numbers_of(root)?;
    let mut below: Vec<i32> = Vec::new();
    while let Some(pid) = frontier.pop() {
        below.push(pid);
        frontier.extend(child_numbers_of(pid).unwrap_or_default());
    }
    return Some(below);
}

/// The process numbers of the children `pid` started,
/// from the list of every one of its threads,
/// and nothing when it has no child list at all.
#[cfg(target_os = "linux")]
fn child_numbers_of(pid: i32) -> Option<Vec<i32>> {
    let tasks = std::fs::read_dir(format!("/proc/{pid}/task")).ok()?;
    let mut found: Vec<i32> = Vec::new();
    let mut read_one: bool = false;
    for task in tasks.flatten() {
        let Ok(text) = std::fs::read_to_string(task.path().join("children")) else {
            continue;
        };
        read_one = true;
        for word in text.split_ascii_whitespace() {
            if let Ok(member) = word.parse::<i32>() {
                found.push(member);
            }
        }
    }
    if !read_one {
        return None;
    }
    return Some(found);
}

/// Only Linux lists child processes;
/// elsewhere nothing is found and the caller reads `/proc`.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn descendant_numbers_from_thread_lists(_root: i32) -> Option<Vec<i32>> {
    return None;
}

/// What:
///  What one process is using:
/// its name,
/// the state letter the kernel holds it in,
/// and the processor time it has used.
/// Why:
///   A notice that reports what a step was doing needs all three from one read of one small file,
///  and a process in `D` (uninterruptible sleep,
/// held by a stalled disk or a processor it is not given)
/// is a different answer from one that is computing.
/// Limit:
/// the name is the kernel's `comm`,
/// which is cut to 15 characters.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessUse {
    /// The kernel's name for the process.
    pub name: String,
    /// The start time the kernel recorded for it,
    /// in clock ticks since boot (`/proc/<pid>/stat`,
    /// field 22),
    /// which proves at the next read that the number still names it.
    pub started: u64,
    /// The state letter (`R`,
    /// `S`,
    /// `D`,
    /// `Z`,
    /// ...).
    pub state: char,
    /// The processor time the process has used.
    pub cpu: Duration,
}

/// The name,
/// state and processor time of one process,
/// read from one `/proc/<pid>/stat`,
/// or nothing when it has ended.
#[cfg(target_os = "linux")]
pub fn use_of(pid: i32) -> Option<ProcessUse> {
    let text: String = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    return parse_use(text.as_str());
}

/// The clock ticks the kernel counts processor time in.
#[cfg(target_os = "linux")]
fn ticks_per_second() -> u64 {
    // SAFETY: `sysconf` reads a constant of the system; it takes no pointers and writes no memory.
    let ticks: i64 = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    return u64::try_from(ticks).unwrap_or(100);
}

/// What:
///  The name,
/// the state and the processor time of one `/proc/<pid>/stat` line.
/// Why:
///   The command name sits in parentheses and may hold spaces and parentheses,
/// so the fields are read after the last closing parenthesis.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [state, ..., utime, stime] = text.slice(text.lastIndexOf(')') + 2).split(' ');
/// ```
#[cfg(target_os = "linux")]
fn parse_use(text: &str) -> Option<ProcessUse> {
    let opened: usize = text.find('(')?;
    let closed: usize = text.rfind(')')?;
    let name: String = text.get(opened + 1..closed)?.to_string();
    let fields: Vec<&str> = text.get(closed + 2..)?.split(' ').collect();
    let state: char = fields.first()?.chars().next()?;
    // After the name: state (0), parent (1), ..., utime (11), stime (12), ..., start time (19).
    let utime: u64 = fields.get(11)?.parse().ok()?;
    let stime: u64 = fields.get(12)?.parse().ok()?;
    let started: u64 = fields.get(19)?.parse().ok()?;
    let ticks: u64 = utime.saturating_add(stime);
    let cpu: Duration = Duration::from_millis(ticks.saturating_mul(1_000) / ticks_per_second());
    return Some(ProcessUse { name, started, state, cpu });
}

/// Only Linux reads a process's state and processor time;
/// elsewhere nothing is known.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn use_of(_pid: i32) -> Option<ProcessUse> {
    return None;
}

/// What:
///  The root of a tree as a member:
/// its pid and the start time that proves later that it is still that process.
/// Why:
///   `descendants_of` lists only what is below a process,
/// and a stop must wait for the process itself too.
///  A process that has ended,
/// or is a zombie,
/// is no member.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function memberOf(pid: number): Promise<TreeMember | undefined>;
/// ```
#[cfg(target_os = "linux")]
pub fn member_of(pid: i32) -> Option<TreeMember> {
    let text: String = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let (_parent, started, zombie) = parse_stat(text.as_str())?;
    if zombie {
        return None;
    }
    return Some(TreeMember { pid, started });
}

/// Only Linux reads a process's start time;
/// elsewhere no member is found.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn member_of(_pid: i32) -> Option<TreeMember> {
    return None;
}

/// What:
///  What a read of one process's record came to.
/// Why:
///   `member_of` answers nothing both for a process that ended and for a record that could not be read,
/// and a caller that stops what it finds must tell them apart:
/// the first is a fact about the process and the second a fact about the read,
/// so only the first may be read as "the step ended".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MemberRead = { kind: 'found'; member: TreeMember } | { kind: 'gone' } | { kind: 'unreadable' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MemberRead {
    /// The process, still there and not a zombie.
    Found(TreeMember),
    /// No process has that number,
    /// or the one that has it is a zombie.
    Gone,
    /// The record could not be read at all
    /// (a descriptor, memory or permission limit).
    Unreadable,
}

/// The member `pid` names,
/// or which of the two ways it is not one.
#[cfg(target_os = "linux")]
pub fn member_read(pid: i32) -> MemberRead {
    let text: String = match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
        Ok(read) => read,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return MemberRead::Gone,
        Err(_) => return MemberRead::Unreadable,
    };
    let Some((_parent, started, zombie)) = parse_stat(text.as_str()) else {
        return MemberRead::Unreadable;
    };
    if zombie {
        return MemberRead::Gone;
    }
    return MemberRead::Found(TreeMember { pid, started });
}

/// Only Linux reads a process's record;
/// elsewhere no member is found.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn member_read(_pid: i32) -> MemberRead {
    return MemberRead::Gone;
}

/// What:
///  The start time the kernel recorded for `pid` and whether it holds it as a zombie,
///  or nothing when the number names no process
///  (or no readable one).
/// Why:
///   Every answer to "is this still the process that was found" comes from here,
/// and a signal's identity check and its send must read the same fact,
/// so a control that puts a number's reuse between two reads has one place to stand in
/// (`reused_after_the_next_read`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identityOf = (pid: number): { started: number; zombie: boolean } | undefined;
/// ```
#[cfg(target_os = "linux")]
fn identity_of(pid: i32) -> Option<(u64, bool)> {
    #[cfg(test)]
    if let Some(claimed) = claim_a_reuse_for(pid) {
        return Some((claimed, false));
    }
    let text: String = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let (_parent, started, zombie) = parse_stat(text.as_str())?;
    return Some((started, zombie));
}

// What:
//  The reuse a control has put behind the next identity read of one number:
// that number and the start time the read answers with.
// Why:
//   A process number cannot be reused on demand,
// so the interval between reading a process's identity and signalling it is reachable only by injection.
//  A control of this process's own thread:
// the notice's own thread reads identities too.
#[cfg(all(test, target_os = "linux"))]
thread_local! {
    /// The armed reuse,
    /// taken by the first identity read of the number it names.
    static REUSED_AFTER_THE_NEXT_READ: std::cell::Cell<Option<(i32, u64)>> = const { std::cell::Cell::new(None) };
}

/// Put a reuse of `pid` behind its next identity read:
/// the read answers `started`,
/// and every read after it finds whatever the number names then.
#[cfg(all(test, target_os = "linux"))]
pub(crate) fn reused_after_the_next_read(pid: i32, started: u64) {
    REUSED_AFTER_THE_NEXT_READ.with(|slot| { slot.set(Some((pid, started))); });
}

/// The control's answer for `pid`, when one is armed for that number.
#[cfg(all(test, target_os = "linux"))]
fn claim_a_reuse_for(pid: i32) -> Option<u64> {
    let armed: Option<(i32, u64)> = REUSED_AFTER_THE_NEXT_READ.with(std::cell::Cell::get);
    let (armed_pid, started) = armed?;
    if armed_pid != pid {
        return None;
    }
    REUSED_AFTER_THE_NEXT_READ.with(|slot| { slot.set(None); });
    return Some(started);
}

/// Whether the process is still the one that was found:
/// it exists,
/// is not a zombie,
/// and has the recorded start time.
#[cfg(target_os = "linux")]
pub fn still_running(member: &TreeMember) -> bool {
    return identity_of(member.pid) == Some((member.started, false));
}

/// What:
///  Whether the process is still the one that was found and is in uninterruptible sleep (state `D` in `/proc/<pid>/stat`).
/// Why:
///   A process in that state,
/// held by a stalled disk call or a processor it is not given,
/// cannot act on a signal until the call
///  returns,
/// and a `SIGKILL` then only cuts what follows it:
/// Git's handler removes a lock's pid file and then the lock,
/// and a kill
///  between the two leaves a lock nothing proves stale.
/// A stop therefore waits while a member is in this state,
/// up to its ceiling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// member.stillRunning() && readState(member.pid) === 'D'
/// ```
#[cfg(target_os = "linux")]
pub fn in_uninterruptible_sleep(member: &TreeMember) -> bool {
    return state_letter_of(member) == Some('D');
}

/// Only Linux reads a process's state;
/// elsewhere nothing is known to be in uninterruptible sleep.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn in_uninterruptible_sleep(_member: &TreeMember) -> bool {
    return false;
}

/// The state letter of the process (`R`,
/// `S`,
/// `D`,
/// `T`,
/// `t`,
/// ...) while it is still the one that was found,
/// and nothing otherwise.
#[cfg(target_os = "linux")]
pub fn state_letter_of(member: &TreeMember) -> Option<char> {
    let text: String = std::fs::read_to_string(format!("/proc/{}/stat", member.pid)).ok()?;
    let (_parent, started, zombie) = parse_stat(text.as_str())?;
    if started != member.started || zombie {
        return None;
    }
    return text.get(text.rfind(')')? + 2..)?.chars().next();
}

/// Nothing is found elsewhere,
/// so nothing is still running.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn still_running(_member: &TreeMember) -> bool {
    return false;
}

/// What:
///  What a signal came to.
/// Why:
///   A stop is recorded only when a signal was really sent,
/// so a member the kernel refused to signal is a different answer from one it took.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Delivered = 'sent' | 'gone';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Delivered {
    /// The kernel took the signal for the process that was found.
    Sent,
    /// The process is gone,
    /// or the number names another one now,
    /// so nothing was sent.
    Gone,
}

/// What:
///  One member of a tree, held with the descriptor its signal travels through:
/// the process that was found,
/// and never one that takes its number later.
/// Why:
///   Reading a process's start time and sending its signal are two operations,
/// and a number can pass to another process between them,
/// which would put the signal into a command of the person's that has nothing to do with the stop.
/// A pidfd is made while the number is proven to name the found process,
/// and `pidfd_send_signal` then reaches the process that descriptor names or fails,
/// so no interval is left for a reuse to fall into.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pinned = PinnedMember.open(found); pinned?.signal('SIGKILL');
/// ```
#[cfg(target_os = "linux")]
pub struct PinnedMember {
    /// The number and start time the tree read.
    member: TreeMember,
    /// The descriptor the kernel made for that process,
    /// where it makes pidfds.
    descriptor: Option<std::os::fd::OwnedFd>,
}

/// The descriptor the kernel makes for `pid`,
/// or nothing where it has no pidfds or refuses one at a limit.
#[cfg(target_os = "linux")]
fn pidfd_of(pid: i32) -> Option<std::os::fd::OwnedFd> {
    let process: rustix::process::Pid = rustix::process::Pid::from_raw(pid)?;
    return rustix::process::pidfd_open(process, rustix::process::PidfdFlags::empty()).ok();
}

#[cfg(target_os = "linux")]
impl PinnedMember {
    /// What:
    ///  Pin `member`: its start time is read, the descriptor is opened, and its start time is read again.
    /// Nothing when the number names another process or none.
    /// Why:
    ///   Both reads must answer the recorded start time for the descriptor to be that process's:
    /// a number that came back would have to be handed to the process that held it before.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const pinned = PinnedMember.open(found);
    /// ```
    pub fn open(member: &TreeMember) -> Option<Self> {
        if identity_of(member.pid) != Some((member.started, false)) {
            return None;
        }
        let descriptor: Option<std::os::fd::OwnedFd> = pidfd_of(member.pid);
        // The number named the member before and after the descriptor was made, so the descriptor is that process's.
        if identity_of(member.pid) != Some((member.started, false)) {
            return None;
        }
        return Some(Self { member: member.clone(), descriptor });
    }

    /// Send `signal` to the pinned process,
    /// and to no other.
    pub fn signal(&self, signal: i32) -> Delivered {
        if let Some(descriptor) = &self.descriptor
            && let Some(named) = rustix::process::Signal::from_named_raw(signal)
        {
            return match rustix::process::pidfd_send_signal(descriptor, named) {
                Ok(()) => Delivered::Sent,
                // The kernel took the descriptor but the process it names has ended: nothing of the person's was reached.
                Err(_) => Delivered::Gone,
            };
        }
        // Without a descriptor the start time is read again right before the signal,
        // so the interval that remains is the one a descriptor closes.
        if !still_running(&self.member) {
            return Delivered::Gone;
        }
        // SAFETY: `kill` takes two integers and touches no memory of this process.
        let result: libc::c_int = unsafe { libc::kill(self.member.pid, signal) };
        return if result == 0 { Delivered::Sent } else { Delivered::Gone };
    }
}

/// What:
///  Send `signal` to each member that is still the process that was found;
/// the others are left alone.
/// Answers what each signal came to,
/// in the order of the members.
/// Why:
///   A number that was reused by an unrelated process must never be signalled,
/// and the signal travels through a descriptor opened while the number was proven to name the member.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const member of tree) member.signal('SIGKILL');
/// ```
#[cfg(target_os = "linux")]
pub fn signal_members(members: &[TreeMember], signal: i32) -> Vec<Delivered> {
    let mut reported: Vec<Delivered> = Vec::with_capacity(members.len());
    for member in members {
        let delivered: Delivered = match PinnedMember::open(member) {
            Some(pinned) => pinned.signal(signal),
            None => Delivered::Gone,
        };
        reported.push(delivered);
    }
    return reported;
}

/// The same where the kernel has no pidfd:
/// the start time is read again right before each signal,
/// and the interval that leaves is the one a descriptor closes.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(all(unix, not(target_os = "linux")))]
pub fn signal_members(members: &[TreeMember], signal: i32) -> Vec<Delivered> {
    let mut reported: Vec<Delivered> = Vec::with_capacity(members.len());
    for member in members {
        if !still_running(member) {
            reported.push(Delivered::Gone);
            continue;
        }
        // SAFETY: `kill` takes no pointers; a failure is reported because the process may have ended since the check.
        let result: libc::c_int = unsafe { libc::kill(member.pid, signal) };
        reported.push(if result == 0 { Delivered::Sent } else { Delivered::Gone });
    }
    return reported;
}

/// Platforms without signals stop only the root.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub fn signal_members(_members: &[TreeMember], _signal: i32) -> Vec<Delivered> {
    return Vec::new();
}

/// Ask each member that is still the process that was found to end,
/// with `SIGTERM`,
/// answering what each signal came to.
#[cfg(unix)]
pub fn terminate_members(members: &[TreeMember]) -> Vec<Delivered> {
    return signal_members(members, libc::SIGTERM);
}

/// Platforms without signals stop only the root.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub fn terminate_members(_members: &[TreeMember]) -> Vec<Delivered> {
    return Vec::new();
}

/// End each member that is still the process that was found,
/// with `SIGKILL`,
/// answering what each signal came to.
#[cfg(unix)]
pub fn kill_members(members: &[TreeMember]) -> Vec<Delivered> {
    return signal_members(members, libc::SIGKILL);
}

/// Platforms without signals stop only the root.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub fn kill_members(_members: &[TreeMember]) -> Vec<Delivered> {
    return Vec::new();
}

/// Process-tree controls stay out of the release executable.
#[cfg(test)]
#[path = "process_tree_tests.rs"]
mod tests;

/// Controls for platforms without `/proc`,
/// whose branches the Linux build does not compile.
#[cfg(all(test, not(target_os = "linux")))]
#[path = "process_tree_other_tests.rs"]
mod other_tests;
