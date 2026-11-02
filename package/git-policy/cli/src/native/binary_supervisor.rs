//! What:
//!  The hang guard of the binary tests: a thread that watches the process tree of each wrapped command the test started
//!  and ends it when it makes no progress or burns too much processor time, instead of ending every command after five
//!  seconds of wall time.
//! Why:
//!  A wall-clock bound cannot tell a command that waits on the disk, or is starved of the processor by other processes,
//!  from one that loops or has deadlocked. On this host other sessions' write-back stalls a single `fsync` of a trivial
//!  command for several seconds (4.74 s measured, `doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md`),
//!  and a test that fails for that is flaky, not a finding about the wrapper. What a hang looks like instead is a tree that
//!  does nothing: every task asleep on a lock, a futex, a pipe or a socket, none runnable and none waiting for the disk,
//!  with its processor time, its read and write counters and its set of processes all standing still for a window. A loop
//!  looks like processor time that keeps growing past a budget no real command comes near. Both end the command with a
//!  message that names it, the reason and the tasks of the tree; a wall-clock backstop of minutes ends anything else.
//!  The rule never ends a command the old five-second wall bound would have let finish: an idle window of five seconds
//!  needs five seconds of wall time first.
//!  One monitor thread per test process finds the shared-bound commands among the test's own children (a `timeout` with
//!  the shared backstop), so every way a test runs a command is watched, whether it goes through `observe_raw` or starts
//!  the command itself. A command the monitor ended and no test claimed within `UNCLAIMED_GRACE` fails the test by
//!  aborting the process with the same message, so a kill is never silent.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const judge = new Judge(limits);
//! setInterval(() => { const reason = judge.observe(elapsed(), sampleTree(pid)); if (reason) { kill(-pid, 'SIGKILL'); fail(reason); } }, 100);
//! ```

use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::sync::{Mutex, Once};
use std::time::{Duration, Instant};

/// The wall-clock backstop of a shared-bound command as the text `timeout` takes: ten minutes, which only a command
///  that is both not idle and not burning the processor ever reaches.
pub const SHARED_BACKSTOP_SECONDS: &str = "600";

/// How long a process tree may show nothing at all before it counts as hung.
pub const NO_PROGRESS_WINDOW: Duration = Duration::from_secs(5);

/// How much processor time a tree may use in all. Real commands of these tests use well under a second of it.
pub const CPU_BUDGET: Duration = Duration::from_secs(10);

/// How often the monitor looks at the commands.
pub const POLL: Duration = Duration::from_millis(100);

/// How long a kill may stay unclaimed by the test that ran the command before the process aborts with its message.
pub const UNCLAIMED_GRACE: Duration = Duration::from_secs(3);

/// The limits of one watched command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// How long nothing may move.
    pub no_progress: Duration,
    /// The processor time of the tree that ends it.
    pub cpu_budget: Duration,
    /// The wall time that ends it whatever else is true.
    pub backstop: Duration,
}

/// The limits of the shared bound.
pub const SHARED: Limits = Limits { no_progress: NO_PROGRESS_WINDOW, cpu_budget: CPU_BUDGET, backstop: Duration::from_secs(600) };

/// What one look at a process tree showed.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Sample {
    /// Processor time of the tree in clock ticks: user and system time of every live process plus that of the children
    ///  each has waited for.
    pub cpu_ticks: u64,
    /// The read and write counters of every live process added together: characters, calls and bytes moved.
    pub activity: u64,
    /// The processes of the tree, sorted: a new or a gone process is a sign of life.
    pub processes: Vec<i32>,
    /// Whether some task is runnable (state `R`), which is a process starved of the processor, not a hung one.
    pub runnable: bool,
    /// Whether some task waits for the disk (state `D`), which is a slow volume, not a hung process.
    pub disk_wait: bool,
}

/// Why a command was ended.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reason {
    /// Nothing moved for the window.
    NoProgress(Duration),
    /// The tree used its processor budget.
    CpuBudget(Duration),
    /// The wall-clock backstop passed.
    Backstop(Duration),
}

impl std::fmt::Display for Reason {
    /// The reason in words, with the figure that decided it.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Reason::NoProgress(idle) => write!(
                formatter,
                "no progress for {:.1} s: no task was runnable or waiting for the disk, and processor time, read and write counters and the set of processes stood still (a deadlock or a wait on something that never happens)",
                idle.as_secs_f64()
            ),
            Reason::CpuBudget(used) => write!(formatter, "it used {:.1} s of processor time (a loop)", used.as_secs_f64()),
            Reason::Backstop(elapsed) => write!(formatter, "it was still running after the {:.0} s wall-clock backstop", elapsed.as_secs_f64()),
        };
    }
}

/// The judgement of one watched command: it remembers what the last look showed and since when nothing moved.
#[derive(Clone, Debug)]
pub struct Judge {
    /// The limits applied.
    limits: Limits,
    /// The time of the last look at which anything moved or a task was runnable or waiting for the disk.
    last_change: Duration,
    /// What the previous look showed, to compare with.
    previous: Option<(u64, u64, Vec<i32>)>,
    /// The most processor time any look showed: a process that is waited for moves its time from itself to its parent, so
    ///  the figure is not monotonic by itself.
    cpu_peak_ticks: u64,
    /// Processor time of one clock tick, in nanoseconds.
    tick: Duration,
}

impl Judge {
    /// A judge with nothing seen yet; `tick` is the length of a clock tick.
    pub fn new(limits: Limits, tick: Duration) -> Judge {
        return Judge { limits, last_change: Duration::ZERO, previous: None, cpu_peak_ticks: 0, tick };
    }

    /// Judge one look taken `elapsed` after the command started; `Some` ends the command.
    ///  The processor budget and the backstop are checked first, because they hold whatever the tree is doing.
    pub fn observe(&mut self, elapsed: Duration, sample: &Sample) -> Option<Reason> {
        self.cpu_peak_ticks = self.cpu_peak_ticks.max(sample.cpu_ticks);
        let used: Duration = self.tick.saturating_mul(u32::try_from(self.cpu_peak_ticks).unwrap_or(u32::MAX));
        if used >= self.limits.cpu_budget {
            return Some(Reason::CpuBudget(used));
        }
        if elapsed >= self.limits.backstop {
            return Some(Reason::Backstop(elapsed));
        }
        let moved: bool = self
            .previous
            .as_ref()
            .is_none_or(|(cpu, activity, processes)| { return *cpu != sample.cpu_ticks || *activity != sample.activity || *processes != sample.processes; });
        if moved || sample.runnable || sample.disk_wait {
            self.last_change = elapsed;
        }
        self.previous = Some((sample.cpu_ticks, sample.activity, sample.processes.clone()));
        let idle: Duration = elapsed.saturating_sub(self.last_change);
        if idle >= self.limits.no_progress {
            return Some(Reason::NoProgress(idle));
        }
        return None;
    }
}

/// The fields of `/proc/<pid>/stat` the monitor uses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Stat {
    /// The state letter: `R` running, `S` sleeping, `D` waiting for the disk, `T` or `t` stopped, `Z` zombie.
    pub state: char,
    /// User plus system time of the process and of the children it has waited for, in clock ticks.
    pub cpu_ticks: u64,
    /// When the process started, in clock ticks after boot: with the process ID it names one process for good.
    pub started: u64,
}

/// Parse the text of `/proc/<pid>/stat`. The command name is in parentheses and may hold spaces and parentheses, so the
///  fields are counted from the last closing one.
pub fn parse_stat(text: &str) -> Option<Stat> {
    let after: &str = text.get(text.rfind(')')? + 1..)?;
    let fields: Vec<&str> = after.split_whitespace().collect();
    let number = |index: usize| -> Option<u64> { return fields.get(index)?.parse::<u64>().ok(); };
    // After the name: state, ppid, pgrp, session, tty, tpgid, flags, minflt, cminflt, majflt, cmajflt, utime, stime,
    // cutime, cstime, priority, nice, threads, itrealvalue, starttime.
    let cpu_ticks: u64 = number(11)?.saturating_add(number(12)?).saturating_add(number(13)?).saturating_add(number(14)?);
    return Some(Stat { state: fields.first()?.chars().next()?, cpu_ticks, started: number(19)? });
}

/// The counters of `/proc/<pid>/io` added together: `rchar`, `wchar`, `syscr`, `syscw`, `read_bytes` and `write_bytes`.
pub fn parse_io(text: &str) -> u64 {
    return text
        .lines()
        .filter_map(|line| { return line.split_once(": "); })
        .filter(|(name, _)| { return matches!(*name, "rchar" | "wchar" | "syscr" | "syscw" | "read_bytes" | "write_bytes"); })
        .filter_map(|(_, value)| { return value.trim().parse::<u64>().ok(); })
        .fold(0u64, |total, value| { return total.saturating_add(value); });
}

/// The children of a process, from the `children` file of each of its tasks; nothing when the process is gone.
///  `None` when the kernel keeps no such file, which makes the caller scan the process table instead.
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

/// The parent of a process, from its stat; the field after the state.
fn parent_of(pid: i32) -> Option<i32> {
    let text: String = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let after: &str = text.get(text.rfind(')')? + 1..)?;
    return after.split_whitespace().nth(1)?.parse::<i32>().ok();
}

/// The children of a process by scanning the process table, for a kernel without `children` files.
fn children_by_scan(pid: i32) -> Vec<i32> {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    return entries
        .flatten()
        .filter_map(|entry| { return entry.file_name().to_str()?.parse::<i32>().ok(); })
        .filter(|candidate| { return parent_of(*candidate) == Some(pid); })
        .collect();
}

/// The children of a process.
pub fn children(pid: i32) -> Vec<i32> {
    return children_of(pid).unwrap_or_else(|| { return children_by_scan(pid); });
}

/// A process and everything below it, in discovery order.
pub fn tree_of(root: i32) -> Vec<i32> {
    let mut found: Vec<i32> = vec![root];
    let mut index: usize = 0;
    while index < found.len() {
        for child in children(found[index]) {
            if !found.contains(&child) {
                found.push(child);
            }
        }
        index += 1;
    }
    return found;
}

/// The length of a clock tick.
pub fn tick() -> Duration {
    // SAFETY: `sysconf` takes an integer and touches no memory of this process.
    let per_second: libc::c_long = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    return Duration::from_nanos(1_000_000_000 / u64::try_from(per_second).unwrap_or(100).max(1));
}

/// Look at the tree below `root`. `None` when the root is gone.
pub fn sample_tree(root: i32) -> Option<Sample> {
    let mut sample: Sample = Sample::default();
    let mut root_alive: bool = false;
    for pid in tree_of(root) {
        let Some(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok().and_then(|text| { return parse_stat(text.as_str()); }) else {
            continue;
        };
        root_alive |= pid == root;
        if stat.state == 'Z' {
            sample.cpu_ticks = sample.cpu_ticks.saturating_add(stat.cpu_ticks);
            continue;
        }
        sample.cpu_ticks = sample.cpu_ticks.saturating_add(stat.cpu_ticks);
        sample.runnable |= stat.state == 'R';
        sample.disk_wait |= stat.state == 'D';
        sample.activity = sample.activity.saturating_add(std::fs::read_to_string(format!("/proc/{pid}/io")).map_or(0, |text| { return parse_io(text.as_str()); }));
        sample.processes.push(pid);
    }
    sample.processes.sort_unstable();
    return if root_alive { Some(sample) } else { None };
}

/// The tasks of a tree in words, for the message of a kill: process ID, name, state and the kernel function it sleeps in.
pub fn describe(root: i32) -> String {
    let mut lines: Vec<String> = Vec::new();
    for pid in tree_of(root) {
        let name: String = std::fs::read_to_string(format!("/proc/{pid}/comm")).map_or_else(|_| { return String::from("?"); }, |text| { return text.trim().to_string(); });
        let state: String = std::fs::read_to_string(format!("/proc/{pid}/stat"))
            .ok()
            .and_then(|text| { return parse_stat(text.as_str()); })
            .map_or_else(|| { return String::from("gone"); }, |stat| { return stat.state.to_string(); });
        let waiting: String = std::fs::read_to_string(format!("/proc/{pid}/wchan")).map_or_else(|_| { return String::from("?"); }, |text| { return text.trim().to_string(); });
        let command_line: String = std::fs::read(format!("/proc/{pid}/cmdline"))
            .map(|bytes| { return String::from_utf8_lossy(bytes.as_slice()).replace('\0', " "); })
            .unwrap_or_default();
        lines.push(format!("  pid {pid} ({name}) state {state} waiting in {waiting}: {}", command_line.trim().chars().take(160).collect::<String>()));
    }
    return lines.join("\n");
}

/// A command the monitor ended.
#[derive(Clone, Debug)]
struct Kill {
    /// The message.
    message: String,
    /// When it was ended.
    at: Instant,
    /// Whether a test has taken it.
    claimed: bool,
}

/// One command the monitor watches.
struct Watched {
    /// The judge with its limits.
    judge: Judge,
    /// When the monitor first saw the command.
    began: Instant,
    /// When the process started, to tell a reused process ID from the same process.
    started: u64,
}

/// What the monitor shares with the tests.
#[derive(Default)]
struct Registry {
    /// The commands watched, by process ID.
    watched: HashMap<i32, Watched>,
    /// Commands started with their own limits (the controls of the monitor itself), by process ID.
    explicit: HashMap<i32, Limits>,
    /// The commands ended, by process ID.
    kills: HashMap<i32, Kill>,
    /// How many looks found a task of the command waiting for the disk, by process ID, kept after the command ends.
    disk_wait_looks: HashMap<i32, u32>,
}

/// The registry of this process.
static REGISTRY: Mutex<Option<Registry>> = Mutex::new(None);

/// Runs the registry's closure under its lock.
fn with_registry<T>(action: impl FnOnce(&mut Registry) -> T) -> T {
    let mut guard = REGISTRY.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    return action(guard.get_or_insert_with(Registry::default));
}

/// Whether the command line of a process is the shared bound: `timeout --signal=KILL <backstop> <program> ...`.
pub fn is_shared_command_line(arguments: &[&str]) -> bool {
    return arguments.len() > 3
        && arguments[0].rsplit('/').next() == Some("timeout")
        && arguments[1] == "--signal=KILL"
        && arguments[2] == SHARED_BACKSTOP_SECONDS;
}

/// Whether a command about to be started is the shared bound.
pub fn is_shared(command: &Command) -> bool {
    let program: String = command.get_program().to_string_lossy().into_owned();
    let arguments: Vec<String> = command.get_args().map(|argument| { return argument.to_string_lossy().into_owned(); }).collect();
    let mut line: Vec<&str> = vec![program.as_str()];
    line.extend(arguments.iter().map(String::as_str));
    return is_shared_command_line(line.as_slice());
}

/// Ask the monitor to watch the child `pid` with `limits`, however the test runs it (the controls of the monitor itself).
pub fn watch(pid: u32, limits: Limits) {
    let watched: i32 = i32::try_from(pid).expect("a process ID fits");
    ensure_monitor();
    with_registry(|registry| { registry.explicit.insert(watched, limits); });
}

/// Take the message of the kill of the command `pid`, if the monitor ended it; the test that ran the command reports it.
pub fn take_kill(pid: u32) -> Option<String> {
    let ended: i32 = i32::try_from(pid).ok()?;
    return with_registry(|registry| {
        let kill: &mut Kill = registry.kills.get_mut(&ended)?;
        kill.claimed = true;
        return Some(kill.message.clone());
    });
}

/// How many looks of the monitor found a task of the command `pid` waiting for the disk: a control that wants a slow
///  volume to have been in the way reads it after the command ended.
pub fn disk_wait_looks(pid: u32) -> u32 {
    let Ok(watched) = i32::try_from(pid) else { return 0 };
    return with_registry(|registry| { return registry.disk_wait_looks.get(&watched).copied().unwrap_or(0); });
}

/// Start the monitor thread of this process, once.
pub fn ensure_monitor() {
    static STARTED: Once = Once::new();
    STARTED.call_once(|| {
        std::thread::Builder::new()
            .name(String::from("binary-supervisor"))
            .spawn(monitor)
            .expect("the supervisor thread starts");
    });
}

/// The command line of a process as words.
fn command_line_of(pid: i32) -> Vec<String> {
    return std::fs::read(format!("/proc/{pid}/cmdline"))
        .map(|bytes| { return bytes.split(|byte| { return *byte == 0; }).filter(|word| { return !word.is_empty(); }).map(|word| { return String::from_utf8_lossy(word).into_owned(); }).collect(); })
        .unwrap_or_default();
}

/// The working directory of a process.
fn directory_of(pid: i32) -> String {
    return std::fs::read_link(format!("/proc/{pid}/cwd")).map_or_else(|_| { return String::from("?"); }, |path| { return path.display().to_string(); });
}

/// End the tree of `pid`: its group first (a `timeout` leads one), then every task found in the tree, in case one left it.
pub fn end_tree(pid: i32, tree: &[i32]) {
    // SAFETY: `kill` takes two integers and touches no memory of this process.
    unsafe { libc::kill(-pid, libc::SIGKILL) };
    for member in tree {
        // SAFETY: as above.
        unsafe { libc::kill(*member, libc::SIGKILL) };
    }
}

/// The body of the monitor thread: forever, look at the test's own children.
fn monitor() {
    let own: i32 = i32::try_from(std::process::id()).expect("a process ID fits");
    let tick: Duration = tick();
    let mut not_shared: HashSet<(i32, u64)> = HashSet::new();
    loop {
        std::thread::sleep(POLL);
        let present: Vec<i32> = children(own);
        discover(present.as_slice(), &mut not_shared, tick);
        let now: Instant = Instant::now();
        let ids: Vec<i32> = with_registry(|registry| { return registry.watched.keys().copied().collect(); });
        for pid in ids {
            judge_one(pid, now);
        }
        with_registry(|registry| {
            registry.watched.retain(|pid, _| { return present.contains(pid); });
            registry.explicit.retain(|pid, _| { return present.contains(pid); });
            // A kill that a test has taken is forgotten once its process is gone, so a reused process ID starts afresh.
            registry.kills.retain(|pid, kill| { return !kill.claimed || present.contains(pid); });
        });
        abort_on_unclaimed(now);
    }
}

/// Start watching the children that are shared-bound commands or were registered with limits.
fn discover(present: &[i32], not_shared: &mut HashSet<(i32, u64)>, tick: Duration) {
    for pid in present.iter().copied() {
        let Some(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok().and_then(|text| { return parse_stat(text.as_str()); }) else {
            continue;
        };
        if not_shared.contains(&(pid, stat.started)) || with_registry(|registry| { return registry.watched.get(&pid).is_some_and(|watched| { return watched.started == stat.started; }); }) {
            continue;
        }
        if with_registry(|registry| { return registry.kills.contains_key(&pid); }) {
            continue;
        }
        let explicit: Option<Limits> = with_registry(|registry| { return registry.explicit.get(&pid).copied(); });
        let line: Vec<String> = command_line_of(pid);
        let words: Vec<&str> = line.iter().map(String::as_str).collect();
        let found: Option<Limits> = explicit.or_else(|| { return is_shared_command_line(words.as_slice()).then_some(SHARED); });
        match found {
            Some(limits) => with_registry(|registry| {
                registry.watched.insert(pid, Watched { judge: Judge::new(limits, tick), began: Instant::now(), started: stat.started });
            }),
            None => {
                not_shared.insert((pid, stat.started));
            }
        }
    }
    not_shared.retain(|(pid, _)| { return present.contains(pid); });
}

/// Look at one watched command and end it if the judge says so.
fn judge_one(pid: i32, now: Instant) {
    let Some(sample) = sample_tree(pid) else { return };
    if sample.disk_wait {
        with_registry(|registry| { *registry.disk_wait_looks.entry(pid).or_default() += 1; });
    }
    let verdict: Option<(Reason, Instant)> = with_registry(|registry| {
        let watched: &mut Watched = registry.watched.get_mut(&pid)?;
        let began: Instant = watched.began;
        return watched.judge.observe(now.saturating_duration_since(began), &sample).map(|reason| { return (reason, began); });
    });
    let Some((reason, _began)) = verdict else { return };
    let tree: Vec<i32> = tree_of(pid);
    let line: String = command_line_of(pid).join(" ");
    let message: String = format!(
        "the harness supervisor ended the command `{line}` in {}: {reason}\nthe tasks of its tree when it was ended:\n{}",
        directory_of(pid),
        describe(pid)
    );
    end_tree(pid, tree.as_slice());
    eprintln!("{message}");
    with_registry(|registry| {
        registry.watched.remove(&pid);
        registry.kills.insert(pid, Kill { message, at: Instant::now(), claimed: false });
    });
}

/// The message of a kill nobody has claimed within `grace` of `now`, if there is one.
pub fn unclaimed_kill(now: Instant, grace: Duration) -> Option<String> {
    return with_registry(|registry| {
        return registry.kills.values().find(|kill| { return !kill.claimed && now.saturating_duration_since(kill.at) >= grace; }).map(|kill| { return kill.message.clone(); });
    });
}

/// Record a kill as the monitor would, for the control that reads what happens to one nobody claims.
pub fn plant_kill(pid: i32, message: &str, at: Instant) {
    with_registry(|registry| { registry.kills.insert(pid, Kill { message: message.to_string(), at, claimed: false }); });
}

/// A kill nobody claimed within the grace ends the process, so a command ended by the supervisor always fails its test.
fn abort_on_unclaimed(now: Instant) {
    if let Some(message) = unclaimed_kill(now, UNCLAIMED_GRACE) {
        eprintln!("a command the supervisor ended was never reported by its test, so the process aborts:\n{message}");
        std::process::abort();
    }
}
