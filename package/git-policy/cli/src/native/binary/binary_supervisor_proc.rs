//! What:
//!  The readers of `/proc` for the hang guard of the binary tests:
//! the fields of a process's `stat`,
//! the counters of its `io`,
//!  the states of every one of its tasks,
//! the processes below it,
//! and one look at a whole tree.
//! Why:
//!  The supervisor judges a tree by what the kernel says about it.
//! A thread-group leader asleep on a futex says nothing about a
//!  worker that waits for the disk,
//! so the states are read from every task;
//! a child that ran and was waited for between two looks
//!  is no longer in the tree,
//! so its work is read from the counters its parent keeps for the children it waited for.
//! A process
//!  number alone names nothing for good,
//! so every process a look includes is checked to be the child of the one it was found
//!  under,
//! and its start time is part of what the look returns.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sample = lookAt(rootPid, rootStarted); // { kind: 'seen', cpuTicks, activity, childFaults, processes, runnable, diskWait, held }
//! ```

use std::time::Duration;

/// The fields of `/proc/<pid>/stat` the monitor uses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Stat {
    /// The state letter:
    /// `R` running,
    /// `S` sleeping,
    /// `D` waiting for the disk,
    /// `T` or `t` stopped,
    /// `Z` zombie.
    pub state: char,
    /// The parent's process ID.
    pub parent: i32,
    /// User plus system time of the process and of the children it has waited for,
    /// in clock ticks.
    pub cpu_ticks: u64,
    /// Page faults of the children the process has waited for:
    /// a child that ran and was reaped between two looks leaves its
    ///  faults here,
    /// so its work is seen though the child is not.
    pub child_faults: u64,
    /// When the process started,
    /// in clock ticks after boot:
    /// with the process ID it names one process for good.
    pub started: u64,
}

/// Parse the text of `/proc/<pid>/stat`.
/// The command name is in parentheses and may hold spaces and parentheses,
/// so the
///  fields are counted from the last closing one.
pub fn parse_stat(text: &str) -> Option<Stat> {
    let after: &str = text.get(text.rfind(')')? + 1..)?;
    let fields: Vec<&str> = after.split_whitespace().collect();
    let number = |index: usize| -> Option<u64> { return fields.get(index)?.parse::<u64>().ok(); };
    // After the name:
    // state,
    // ppid,
    // pgrp,
    // session,
    // tty,
    // tpgid,
    // flags,
    // minflt,
    // cminflt,
    // majflt,
    // cmajflt,
    // utime,
    // stime,
    // cutime,
    // cstime,
    // priority,
    // nice,
    // threads,
    // itrealvalue,
    // starttime.
    let cpu_ticks: u64 = number(11)?.saturating_add(number(12)?).saturating_add(number(13)?).saturating_add(number(14)?);
    let child_faults: u64 = number(8)?.saturating_add(number(10)?);
    return Some(Stat {
        state: fields.first()?.chars().next()?,
        parent: i32::try_from(number(1)?).ok()?,
        cpu_ticks,
        child_faults,
        started: number(19)?,
    });
}

/// The bytes counters of `/proc/<pid>/io` added together:
/// `rchar`,
/// `wchar`,
/// `read_bytes` and `write_bytes`.
///  The counts of calls (`syscr`,
/// `syscw`) are left out on purpose:
/// a loop that polls an empty descriptor makes calls for ever
///  and moves nothing,
/// so a call is no sign that anything happened.
pub fn parse_io(text: &str) -> u64 {
    return text
        .lines()
        .filter_map(|line| { return line.split_once(": "); })
        .filter(|(name, _)| { return matches!(*name, "rchar" | "wchar" | "read_bytes" | "write_bytes"); })
        .filter_map(|(_, value)| { return value.trim().parse::<u64>().ok(); })
        .fold(0u64, |total, value| { return total.saturating_add(value); });
}

/// The `stat` of a process now;
/// nothing when it is gone.
pub fn stat_of(pid: i32) -> Option<Stat> {
    return std::fs::read_to_string(format!("/proc/{pid}/stat")).ok().and_then(|text| { return parse_stat(text.as_str()); });
}

/// The state letters of every task of a process,
/// the leader's first;
/// nothing when the process is gone.
pub fn task_states(pid: i32) -> Vec<char> {
    let Ok(tasks) = std::fs::read_dir(format!("/proc/{pid}/task")) else {
        return Vec::new();
    };
    let mut found: Vec<(i32, char)> = tasks
        .flatten()
        .filter_map(|task| {
            let id: i32 = task.file_name().to_str()?.parse::<i32>().ok()?;
            let text: String = std::fs::read_to_string(task.path().join("stat")).ok()?;
            return Some((id, parse_stat(text.as_str())?.state));
        })
        .collect();
    found.sort_unstable_by_key(|(id, _)| { return *id != pid; });
    return found.into_iter().map(|(_, state)| { return state; }).collect();
}

/// The children of a process,
/// from the `children` file of each of its tasks;
/// nothing when the process is gone.
///  `None` when the kernel keeps no such file,
/// which makes the caller scan the process table instead.
fn children_of(pid: i32) -> Option<Vec<i32>> {
    let mut found: Vec<i32> = Vec::new();
    let Ok(tasks) = std::fs::read_dir(format!("/proc/{pid}/task")) else {
        return Some(found);
    };
    for task in tasks.flatten() {
        match std::fs::read_to_string(task.path().join("children")) {
            Ok(text) => found.extend(text.split_whitespace().filter_map(|word| { return word.parse::<i32>().ok(); })),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && !task.path().exists() => {}
            Err(_) => return None,
        }
    }
    return Some(found);
}

/// The children of a process by scanning the process table,
/// for a kernel without `children` files.
fn children_by_scan(pid: i32) -> Vec<i32> {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    return entries
        .flatten()
        .filter_map(|entry| { return entry.file_name().to_str()?.parse::<i32>().ok(); })
        .filter(|candidate| { return stat_of(*candidate).is_some_and(|stat| { return stat.parent == pid; }); })
        .collect();
}

/// The children of a process.
pub fn children(pid: i32) -> Vec<i32> {
    return children_of(pid).unwrap_or_else(|| { return children_by_scan(pid); });
}

/// A process of a tree,
/// with the start time that names it for good.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Member {
    /// The process ID.
    pub pid: i32,
    /// When it started,
    /// in clock ticks after boot.
    pub started: u64,
}

/// A process and everything below it,
/// in discovery order.
/// A process found under a parent counts only if its own `stat` names
///  that parent,
/// so a number that was reused after the parent's list was read is not taken for a child.
pub fn tree_of(root: i32) -> Vec<Member> {
    let Some(first) = stat_of(root) else {
        return Vec::new();
    };
    let mut found: Vec<Member> = vec![Member { pid: root, started: first.started }];
    let mut index: usize = 0;
    while index < found.len() {
        let parent: i32 = found[index].pid;
        for child in children(parent) {
            if found.iter().any(|member| { return member.pid == child; }) {
                continue;
            }
            if let Some(stat) = stat_of(child)
                && stat.parent == parent
            {
                found.push(Member { pid: child, started: stat.started });
            }
        }
        index += 1;
    }
    return found;
}

/// The length of a clock tick.
pub fn tick() -> Duration {
    // SAFETY:
    // `sysconf` takes an integer and touches no memory of this process.
    let per_second: libc::c_long = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    return Duration::from_nanos(1_000_000_000 / u64::try_from(per_second).unwrap_or(100).max(1));
}

/// What one look at a process tree showed.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Sample {
    /// Processor time of the tree in clock ticks:
    /// user and system time of every live process plus that of the children
    ///  each has waited for.
    pub cpu_ticks: u64,
    /// The bytes read and written by every live process added together.
    pub activity: u64,
    /// The page faults of the children each live process has waited for,
    /// added together:
    /// the work of children that came and
    ///  went between two looks.
    pub child_faults: u64,
    /// The processes of the tree,
    /// sorted,
    /// each with its start time:
    /// a new or a gone process is a sign of life.
    pub processes: Vec<(i32, u64)>,
    /// Whether some task is runnable (state `R`),
    /// which is a process starved of the processor,
    /// not a hung one.
    pub runnable: bool,
    /// Whether some task waits for the disk (state `D`),
    /// which is a slow volume,
    /// not a hung process.
    pub disk_wait: bool,
    /// Whether some task sleeps in the kernel's read of a Unix socket,
    /// which is this tree waiting for an answer from a program outside it.
    /// The wrapper waits this way for the commit service's answer,
    /// and that answer's latency is the service's own disk work.
    pub answer_wait: bool,
    /// Whether some task is stopped (state `T`) or traced (state `t`):
    /// something outside the tree holds it,
    /// a signal a test
    ///  sent or a debugger,
    /// and a tree held from outside is not idle by its own doing.
    pub held: bool,
}

/// Fold the states of the tasks of one process into a sample.
pub fn note_states(sample: &mut Sample, states: &[char]) {
    for state in states {
        sample.runnable |= *state == 'R';
        sample.disk_wait |= *state == 'D';
        sample.held |= matches!(*state, 'T' | 't');
    }
}

/// What one look at a tree found.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Look {
    /// The root is gone,
    /// or is another process now.
    Gone,
    /// The root has exited and waits to be reaped:
    /// the command is finished,
    /// whatever its pipes still hold.
    Exited,
    /// The tree is alive.
    Seen(Sample),
}

/// Look at the tree below `root`,
/// which started at `started`.
pub fn look_at(root: i32, started: u64) -> Look {
    match stat_of(root) {
        None => return Look::Gone,
        Some(stat) if stat.started != started => return Look::Gone,
        Some(stat) if stat.state == 'Z' => return Look::Exited,
        Some(_) => {}
    }
    let mut sample: Sample = Sample::default();
    for member in tree_of(root) {
        let Some(stat) = stat_of(member.pid).filter(|stat| { return stat.started == member.started; }) else {
            continue;
        };
        sample.cpu_ticks = sample.cpu_ticks.saturating_add(stat.cpu_ticks);
        sample.child_faults = sample.child_faults.saturating_add(stat.child_faults);
        if stat.state == 'Z' {
            continue;
        }
        note_states(&mut sample, task_states(member.pid).as_slice());
        sample.answer_wait |= waits_for_an_answer(member.pid);
        sample.activity = sample.activity.saturating_add(std::fs::read_to_string(format!("/proc/{}/io", member.pid)).map_or(0, |text| { return parse_io(text.as_str()); }));
        sample.processes.push((member.pid, member.started));
    }
    sample.processes.sort_unstable();
    return Look::Seen(sample);
}

/// The kernel functions a process sleeps in while it reads a Unix socket an answer is expected on.
///  Linux's `af_unix.c` waits in the first for a stream socket with no data yet.
const ANSWER_WAIT_CHANNELS: [&str; 2] = ["unix_stream_read_generic", "unix_stream_data_wait"];

/// The kernel function a process sleeps in,
/// or `?` when the kernel does not say.
pub fn wait_channel(pid: i32) -> String {
    return std::fs::read_to_string(format!("/proc/{pid}/wchan")).map_or_else(|_| { return String::from("?"); }, |text| { return text.trim().to_string(); });
}

/// Whether a process sleeps in the read of a Unix socket.
pub fn waits_for_an_answer(pid: i32) -> bool {
    let channel: String = wait_channel(pid);
    return ANSWER_WAIT_CHANNELS.contains(&channel.as_str());
}

/// The tasks of a tree in words,
/// for the message of a kill:
/// process ID,
/// name,
/// the state of the process and of each of its tasks,
///  and the kernel function it sleeps in.
pub fn describe(root: i32) -> String {
    let mut lines: Vec<String> = Vec::new();
    for member in tree_of(root) {
        let pid: i32 = member.pid;
        let name: String = std::fs::read_to_string(format!("/proc/{pid}/comm")).map_or_else(|_| { return String::from("?"); }, |text| { return text.trim().to_string(); });
        let states: Vec<char> = task_states(pid);
        let state: String = states.first().map_or_else(|| { return String::from("gone"); }, ToString::to_string);
        let tasks: String = if states.len() > 1 { format!(" (tasks {})", states.iter().collect::<String>()) } else { String::new() };
        let waiting: String = wait_channel(pid);
        let command_line: String = std::fs::read(format!("/proc/{pid}/cmdline"))
            .map(|bytes| { return String::from_utf8_lossy(bytes.as_slice()).replace('\0', " "); })
            .unwrap_or_default();
        lines.push(format!("  pid {pid} ({name}) state {state}{tasks} waiting in {waiting}: {}", command_line.trim().chars().take(160).collect::<String>()));
    }
    return lines.join("\n");
}

/// The command line of a process as words.
pub fn command_line_of(pid: i32) -> Vec<String> {
    return std::fs::read(format!("/proc/{pid}/cmdline"))
        .map(|bytes| { return bytes.split(|byte| { return *byte == 0; }).filter(|word| { return !word.is_empty(); }).map(|word| { return String::from_utf8_lossy(word).into_owned(); }).collect(); })
        .unwrap_or_default();
}

/// The working directory of a process.
pub fn directory_of(pid: i32) -> String {
    return std::fs::read_link(format!("/proc/{pid}/cwd")).map_or_else(|_| { return String::from("?"); }, |path| { return path.display().to_string(); });
}
