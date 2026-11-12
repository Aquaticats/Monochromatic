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
/// from a look that failed (a descriptor or a memory limit,
/// a `/proc` that is not there):
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
    /// The process,
    /// still there and not a zombie,
    /// and the parent the kernel records for it.
    Found {
        /// The number and start time.
        member: TreeMember,
        /// The parent the kernel records,
        /// which is this process for a child that has not been reaped.
        parent: i32,
    },
    /// No process has that number,
    /// or the one that has it is a zombie.
    Gone,
    /// The record could not be read at all
    /// (a descriptor,
    /// memory or permission limit).
    Unreadable,
}

/// The member `pid` names,
/// or which of the two ways it is not one.
#[cfg(target_os = "linux")]
pub fn member_read(pid: i32) -> MemberRead {
    #[cfg(test)]
    if claim_an_unreadable_for(pid) {
        return MemberRead::Unreadable;
    }
    let text: String = match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
        Ok(read) => read,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return MemberRead::Gone,
        Err(_) => return MemberRead::Unreadable,
    };
    let Some((parent, started, zombie)) = parse_stat(text.as_str()) else {
        return MemberRead::Unreadable;
    };
    if zombie {
        return MemberRead::Gone;
    }
    return MemberRead::Found { member: TreeMember { pid, started }, parent };
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
pub(crate) fn identity_of(pid: i32) -> Option<(u64, bool)> {
    #[cfg(test)]
    if claim_an_unreadable_for(pid) {
        return None;
    }
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
    // The armed reuse,
    // taken by the first identity read of the number it names.
    static REUSED_AFTER_THE_NEXT_READ: std::cell::Cell<Option<(i32, u64)>> = const { std::cell::Cell::new(None) };
}

/// Put a reuse of `pid` behind its next identity read:
/// the read answers `started`,
/// and every read after it finds whatever the number names then.
#[cfg(all(test, target_os = "linux"))]
pub(crate) fn reused_after_the_next_read(pid: i32, started: u64) {
    REUSED_AFTER_THE_NEXT_READ.with(|slot| { slot.set(Some((pid, started))); });
}

// What:
//  A control that makes the next identity read of one number fail,
//  as a record that cannot be read does (`EACCES`, `EIO`, `ENOMEM`).
// Why:
//   A read that fails for one process while `/proc` itself answers cannot be produced on demand,
// so the kernel's answer is injected.
//  A control of this process's own thread:
// the notice's own thread reads identities too.
#[cfg(all(test, target_os = "linux"))]
thread_local! {
    // The number whose read fails,
    // and how many of its reads answer normally first.
    static UNREADABLE_AFTER_THE_NEXT_READ: std::cell::Cell<Option<(i32, usize)>> = const { std::cell::Cell::new(None) };
}

/// Make the read of `pid` that comes after `answered_first` normal reads of it fail.
/// Why:
///   A control that must reach the read inside the pin,
/// past the one the caller makes before it,
/// says how many reads are answered first.
#[cfg(all(test, target_os = "linux"))]
pub(crate) fn unreadable_after_the_next_reads(pid: i32, answered_first: usize) {
    UNREADABLE_AFTER_THE_NEXT_READ.with(|slot| { slot.set(Some((pid, answered_first))); });
}

/// Take the control's refusal for `pid`,
/// when one is armed for it and its normal reads are used up.
#[cfg(all(test, target_os = "linux"))]
fn claim_an_unreadable_for(pid: i32) -> bool {
    let armed: Option<(i32, usize)> = UNREADABLE_AFTER_THE_NEXT_READ.with(std::cell::Cell::get);
    let Some((armed_pid, answered_first)) = armed else {
        return false;
    };
    if armed_pid != pid {
        return false;
    }
    if answered_first > 0 {
        UNREADABLE_AFTER_THE_NEXT_READ.with(|slot| { slot.set(Some((armed_pid, answered_first.saturating_sub(1)))); });
        return false;
    }
    UNREADABLE_AFTER_THE_NEXT_READ.with(|slot| { slot.set(None); });
    return true;
}

/// The control's answer for `pid`,
/// when one is armed for that number.
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
///  Make the child this command starts the reaper of its own descendants,
/// so a helper that detaches and is reparented answers to that child instead of to init.
/// Why:
///  A stopped run ends the tree it can still find,
/// and a transfer helper that calls `setsid` and is reparented escaped that tree and ran on
/// (audit `alpha-12-lfs`,
/// finding "A stopped upload can leave its detached transfer helper running").
/// `PR_SET_CHILD_SUBREAPER` makes a reparented helper a descendant of the child again,
/// where the sweep that ends a stopped run finds it (`stop_push`).
/// The setting survives `execve` and is not inherited by a fork,
/// so it is set between the fork and the exec,
/// and the nearest living ancestor that has it takes the orphan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no equivalent: it is a Linux process attribute, set between fork and exec
/// ```
#[cfg(target_os = "linux")]
pub fn own_the_descendants_it_starts(command: &mut std::process::Command) {
    use std::os::unix::process::CommandExt;
    // SAFETY: `prctl` with `PR_SET_CHILD_SUBREAPER` sets this child's own attribute and touches nothing else, and it is
    //  async-signal-safe, which is what a closure that runs between fork and exec may call.
    unsafe {
        command.pre_exec(|| {
            let refused: libc::c_int = libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0);
            if refused != 0 {
                return Err(std::io::Error::last_os_error());
            }
            return Ok(());
        });
    }
}

/// Only Linux reparents an orphan to a subreaper;
/// elsewhere a command is started as it stands.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn own_the_descendants_it_starts(_command: &mut std::process::Command) {}

/// Process-tree controls stay out of the release executable.
#[cfg(test)]
#[path = "process_tree_tests.rs"]
mod tests;
/// Controls for platforms without `/proc`,
/// whose branches the Linux build does not compile.
#[cfg(all(test, not(target_os = "linux")))]
#[path = "process_tree_other_tests.rs"]
mod other_tests;
