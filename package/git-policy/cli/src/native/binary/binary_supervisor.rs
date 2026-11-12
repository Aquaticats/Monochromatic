//! What:
//!  The hang guard of the binary tests:
//! a thread that watches the process tree of each wrapped command the test started
//!  and ends it when it makes no progress or burns too much processor time,
//! instead of ending every command after five
//!  seconds of wall time.
//! Why:
//!  A wall-clock bound cannot tell a command that waits on the disk,
//! or is starved of the processor by other processes,
//!  from one that loops or has deadlocked.
//! On this host other sessions' write-back stalls a single `fsync` of a trivial
//!  command for several seconds (4.74 s measured,
//! `doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md`),
//!  and a test that fails for that is flaky,
//! not a finding about the wrapper.
//! What a hang looks like instead is a tree that
//!  does nothing:
//! every task asleep on a lock,
//! a futex,
//! a pipe or a socket,
//! none runnable and none waiting for the disk,
//!  with its processor time,
//! the bytes it moved,
//! the work of the children it waited for and its set of processes all standing
//!  still for a window.
//! A tree waiting in the kernel's read of a Unix socket is not that:
//! it waits for an answer from a
//!  program outside it,
//! the wrapper for the commit service's,
//! and that answer's latency is the service's own disk work,
//!  so no look of this tree can see it as progress or as a hang of its own
//!  (`doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md`,
//! "A merge that waited for the commit service's answer").
//! A loop looks like processor time that keeps growing past a budget no real command comes near.
//! Both end
//!  the command with a message that names it,
//! the reason and the tasks of the tree;
//! a wall-clock backstop of minutes ends
//!  anything else.
//!  The idle window never ends a command the old five-second wall bound would have let finish:
//! it needs five seconds of wall
//!  time first.
//! The processor budget is a second,
//! independent limit and is not a function of wall time:
//! a tree that runs in
//!  parallel can use ten processor seconds in less than five wall seconds,
//! which the old bound would have let finish,
//! and the
//!  budget ends it.
//! No real command of these tests comes near it (they use well under a second),
//! and a control that needs more
//!  says so with limits of its own.
//!  One monitor thread per test process finds the shared-bound commands among the test's own children (a `timeout` with
//!  the shared backstop),
//! so every way a test runs a command is watched,
//! whether it goes through `observe_raw` or starts
//!  the command itself.
//! A command the monitor ended is recorded before the first signal goes out;
//! the test that ran the command
//!  claims the record right after it waited,
//! and a record nobody claims fails the test:
//! the process aborts three seconds after
//!  the kill,
//! and a process that exits first exits with status 101 from an exit handler,
//! so a kill is never silent.
//! Limits:
//!  - Signals reach only pinned processes (`binary_supervisor_pin.rs`):
//! a pidfd where the kernel has one,
//! and the start time read
//!    again before each signal where it has not.
//!  - A process that double-forks and starts a session leaves the tree it was found in and is neither watched nor ended by the
//!    guard.
//! The commit service and other background programs of the wrapper outlive their commands on purpose,
//! so ending
//!    everything that was ever the command's descendant would break the tests;
//! a hang in such a process is ended by the per-test
//!    limit of the test runner.
//!  - The idle rule cannot tell an endless wait from a finite wait on something outside the tree.
//! A test that holds a command
//!    for its own reasons says so with `hold_idle_judgment`;
//! a task that is stopped or traced is held from outside and is not
//!    judged idle either;
//! a pipe that fills is drained from the spawn by `observe_raw`,
//! so a command that writes much is not idle.
//!  - A loop that polls without moving bytes,
//! forking or burning processor time stays below every sign the guard reads.
//! No
//!    predicate over `/proc` decides it,
//! so it is ended by the wall-clock backstop or the per-test limit,
//! whichever comes first.
//!    Process churn is treated as a sign of life.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const judge = new Judge(limits);
//! setInterval(() => { const reason = judge.observe(elapsed(), lookAt(pid)); if (reason) { record(reason); endTree(pin); fail(reason); } }, 100);
//! ```

use super::supervisor_pin::{Pin, end_tree};
use super::supervisor_proc::{Look, Sample, children, command_line_of, describe, directory_of, look_at, stat_of, tick};
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Mutex, Once};
use std::time::{Duration, Instant};

/// The wall-clock backstop of a shared-bound command as the text `timeout` takes:
/// ten minutes,
/// which only a command
///  that is both not idle and not burning the processor ever reaches.
pub const SHARED_BACKSTOP_SECONDS: &str = "600";

/// How long a process tree may show nothing at all before it counts as hung.
pub const NO_PROGRESS_WINDOW: Duration = Duration::from_secs(5);

/// How much processor time a tree may use in all.
/// Real commands of these tests use well under a second of it.
pub const CPU_BUDGET: Duration = Duration::from_secs(10);

/// How often the monitor looks at the commands.
pub const POLL: Duration = Duration::from_millis(100);

/// How long a kill may stay unclaimed by the test that ran the command before the process aborts with its message.
pub const UNCLAIMED_GRACE: Duration = Duration::from_secs(3);

/// The status a process exits with when a kill nobody claimed is found at its exit.
pub const UNCLAIMED_EXIT_STATUS: i32 = 101;

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
    /// The reason in words,
    /// with the figure that decided it.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Reason::NoProgress(idle) => write!(
                formatter,
                "no progress for {:.1} s: no task was runnable, waiting for the disk or held from outside, and processor time, bytes moved, the work of waited-for children and the set of processes stood still (a deadlock or a wait on something that never happens)",
                idle.as_secs_f64()
            ),
            Reason::CpuBudget(used) => write!(formatter, "it used {:.1} s of processor time (a loop)", used.as_secs_f64()),
            Reason::Backstop(elapsed) => write!(formatter, "it was still running after the {:.0} s wall-clock backstop", elapsed.as_secs_f64()),
        };
    }
}

/// What the previous look showed,
/// to compare with.
type Previous = (u64, u64, u64, Vec<(i32, u64)>);

/// The judgement of one watched command:
/// it remembers what the last look showed and since when nothing moved.
#[derive(Clone, Debug)]
pub struct Judge {
    /// The limits applied.
    limits: Limits,
    /// The time of the last look at which anything moved or a task was runnable,
    /// waiting for the disk or held.
    last_change: Duration,
    /// What the previous look showed,
    /// to compare with.
    previous: Option<Previous>,
    /// The most processor time any look showed:
    /// a process that is waited for moves its time from itself to its parent,
    /// so
    ///  the figure is not monotonic by itself.
    cpu_peak_ticks: u64,
    /// Processor time of one clock tick,
    /// in nanoseconds.
    tick: Duration,
}

impl Judge {
    /// A judge with nothing seen yet;
    /// `tick` is the length of a clock tick.
    pub fn new(limits: Limits, tick: Duration) -> Judge {
        return Judge { limits, last_change: Duration::ZERO, previous: None, cpu_peak_ticks: 0, tick };
    }

    /// Judge one look taken `elapsed` after the command started;
    /// `Some` ends the command.
    /// `held_by_test` says the test holds
    ///  the command on purpose (`hold_idle_judgment`),
    /// which suspends the idle rule and nothing else.
    ///  The processor budget and the backstop are checked first,
    /// because they hold whatever the tree is doing.
    pub fn observe(&mut self, elapsed: Duration, sample: &Sample, held_by_test: bool) -> Option<Reason> {
        self.cpu_peak_ticks = self.cpu_peak_ticks.max(sample.cpu_ticks);
        let used: Duration = self.tick.saturating_mul(u32::try_from(self.cpu_peak_ticks).unwrap_or(u32::MAX));
        if used >= self.limits.cpu_budget {
            return Some(Reason::CpuBudget(used));
        }
        if elapsed >= self.limits.backstop {
            return Some(Reason::Backstop(elapsed));
        }
        let moved: bool = self.previous.as_ref().is_none_or(|(cpu, activity, faults, processes)| {
            return *cpu != sample.cpu_ticks || *activity != sample.activity || *faults != sample.child_faults || *processes != sample.processes;
        });
        if moved || sample.runnable || sample.disk_wait || sample.answer_wait || sample.held || held_by_test {
            self.last_change = elapsed;
        }
        self.previous = Some((sample.cpu_ticks, sample.activity, sample.child_faults, sample.processes.clone()));
        let idle: Duration = elapsed.saturating_sub(self.last_change);
        if idle >= self.limits.no_progress {
            return Some(Reason::NoProgress(idle));
        }
        return None;
    }
}

/// A command the monitor ended.
#[derive(Clone, Debug)]
struct Kill {
    /// The process ID of the command.
    pid: i32,
    /// The message.
    message: String,
    /// When it was recorded,
    /// which is before the first signal.
    at: Instant,
}

/// One command the monitor watches.
struct Watched {
    /// The judge with its limits.
    judge: Judge,
    /// When the command was first seen.
    began: Instant,
    /// The command's process,
    /// pinned.
    pin: Pin,
}

/// What the monitor shares with the tests.
#[derive(Default)]
struct Registry {
    /// The commands watched,
    /// by process ID.
    watched: HashMap<i32, Watched>,
    /// The commands ended and not yet claimed.
    kills: Vec<Kill>,
    /// How many tests hold the idle judgment of a command,
    /// by process ID.
    holds: HashMap<i32, u32>,
    /// How many looks found a task of the command waiting for the disk,
    /// by process ID,
    /// kept after the command ends.
    disk_wait_looks: HashMap<i32, u32>,
    /// How many looks the monitor took of the command,
    /// by process ID,
    /// kept after the command ends.
    looks: HashMap<i32, u32>,
}

/// The registry of this process.
static REGISTRY: Mutex<Option<Registry>> = Mutex::new(None);

/// Runs the registry's closure under its lock.
fn with_registry<T>(action: impl FnOnce(&mut Registry) -> T) -> T {
    let mut guard = REGISTRY.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    return action(guard.get_or_insert_with(Registry::default));
}

/// Whether the command line of a process is the shared bound:
/// `timeout --signal=KILL <backstop> <program> ...`.
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

/// Ask the monitor to watch the child `pid` with `limits`,
/// however the test runs it (the controls of the monitor itself).
///  The child is pinned now,
/// while the test still holds it unreaped,
/// and entered into the registry directly,
/// so no look of the
///  monitor that came before can have written it off.
pub fn watch(pid: u32, limits: Limits) {
    let watched: i32 = i32::try_from(pid).expect("a process ID fits");
    ensure_monitor();
    let started: u64 = stat_of(watched).expect("the child to watch is running").started;
    let pin: Pin = Pin::open(watched, started).expect("the child to watch can be pinned");
    with_registry(|registry| { registry.watched.insert(watched, Watched { judge: Judge::new(limits, tick()), began: Instant::now(), pin }); });
}

/// Whether the monitor still watches the command `pid`.
pub fn is_watching(pid: u32) -> bool {
    let Ok(watched) = i32::try_from(pid) else { return false };
    return with_registry(|registry| { return registry.watched.contains_key(&watched); });
}

/// Take the message of the kill of the command `pid`,
/// if the monitor ended it;
/// the test that ran the command reports it.
///  The record is published before the first signal of the kill,
/// so a command the monitor ended has its record by the time
///  the test sees the command end.
pub fn take_kill(pid: u32) -> Option<String> {
    let ended: i32 = i32::try_from(pid).ok()?;
    return with_registry(|registry| {
        let position: usize = registry.kills.iter().rposition(|kill| { return kill.pid == ended; })?;
        return Some(registry.kills.remove(position).message);
    });
}

/// How many looks of the monitor found a task of the command `pid` waiting for the disk:
/// a control that wants a slow
///  volume to have been in the way reads it after the command ended.
pub fn disk_wait_looks(pid: u32) -> u32 {
    let Ok(watched) = i32::try_from(pid) else { return 0 };
    return with_registry(|registry| { return registry.disk_wait_looks.get(&watched).copied().unwrap_or(0); });
}

/// How many looks the monitor has taken of the command `pid`:
/// a control that waits for the monitor to have judged a command
///  a number of times counts them instead of the clock.
pub fn looks_of(pid: u32) -> u32 {
    let Ok(watched) = i32::try_from(pid) else { return 0 };
    return with_registry(|registry| { return registry.looks.get(&watched).copied().unwrap_or(0); });
}

/// A test's hold on the idle judgment of a command,
/// released when it is dropped.
pub struct IdleHold {
    /// The command's process ID.
    pid: i32,
}

impl Drop for IdleHold {
    fn drop(&mut self) {
        with_registry(|registry| {
            if let Some(count) = registry.holds.get_mut(&self.pid) {
                *count -= 1;
                if *count == 0 {
                    registry.holds.remove(&self.pid);
                }
            }
        });
    }
}

/// Suspend the idle rule for the command `pid` while the returned hold lives:
/// for a wait the test itself controls (it will
///  drain the pipe,
/// release the lock or send the signal once its own work is done).
/// The processor budget and the backstop
///  keep judging.
/// A test that holds a command and never lets it go ends at the backstop.
pub fn hold_idle_judgment(pid: u32) -> IdleHold {
    let held: i32 = i32::try_from(pid).expect("a process ID fits");
    with_registry(|registry| { *registry.holds.entry(held).or_default() += 1; });
    return IdleHold { pid: held };
}

/// The exit handler:
/// a kill nobody claimed ends the process with a failing status,
/// whatever the test body concluded.
extern "C" fn fail_unclaimed_at_exit() {
    if let Some(message) = unclaimed_kill(Instant::now(), Duration::ZERO) {
        eprintln!("a command the supervisor ended was never reported by its test, so the process exits with a failure:\n{message}");
        // SAFETY:
        // `_exit` ends the process at once and touches no memory of it;
        // an exit handler must not call `exit` again.
        unsafe { libc::_exit(UNCLAIMED_EXIT_STATUS) };
    }
}

/// Start the monitor thread of this process,
/// once,
/// and register the exit handler.
pub fn ensure_monitor() {
    static STARTED: Once = Once::new();
    STARTED.call_once(|| {
        std::thread::Builder::new()
            .name(String::from("binary-supervisor"))
            .spawn(monitor)
            .expect("the supervisor thread starts");
        // SAFETY:
        // the handler is an `extern "C"` function with no arguments,
        // as `atexit` requires.
        unsafe { libc::atexit(fail_unclaimed_at_exit) };
    });
}

/// The body of the monitor thread:
/// forever,
/// look at the test's own children.
fn monitor() {
    let own: i32 = i32::try_from(std::process::id()).expect("a process ID fits");
    let tick: Duration = tick();
    loop {
        std::thread::sleep(POLL);
        discover(own, tick);
        let now: Instant = Instant::now();
        let ids: Vec<i32> = with_registry(|registry| { return registry.watched.keys().copied().collect(); });
        for pid in ids {
            judge_one(pid, now);
        }
        abort_on_unclaimed(now);
    }
}

/// Start watching the children that are shared-bound commands.
/// Nothing is remembered about a child that is not one:
/// a child
///  between `fork` and `exec` still has the test's own command line,
/// and `exec` keeps its process ID and start time,
/// so a
///  verdict about it is read again at every look.
fn discover(own: i32, tick: Duration) {
    for pid in children(own) {
        let Some(stat) = stat_of(pid) else { continue };
        if stat.parent != own || with_registry(|registry| { return registry.watched.get(&pid).is_some_and(|watched| { return watched.pin.started() == stat.started; }); }) {
            continue;
        }
        let line: Vec<String> = command_line_of(pid);
        let words: Vec<&str> = line.iter().map(String::as_str).collect();
        if !is_shared_command_line(words.as_slice()) {
            continue;
        }
        let Some(pin) = Pin::open(pid, stat.started) else { continue };
        with_registry(|registry| {
            registry.watched.insert(pid, Watched { judge: Judge::new(SHARED, tick), began: Instant::now(), pin });
        });
    }
}

/// Look at one watched command and end it if the judge says so.
fn judge_one(pid: i32, now: Instant) {
    let Some(started) = with_registry(|registry| { return registry.watched.get(&pid).map(|watched| { return watched.pin.started(); }); }) else { return };
    let sample: Sample = match look_at(pid, started) {
        // The command is over;
        // the entry goes,
        // and a kill that was recorded for it stays for its test.
        Look::Gone | Look::Exited => {
            with_registry(|registry| {
                if registry.watched.get(&pid).is_some_and(|watched| { return watched.pin.started() == started; }) {
                    registry.watched.remove(&pid);
                }
            });
            return;
        }
        Look::Seen(seen) => seen,
    };
    let verdict: Option<Reason> = with_registry(|registry| {
        *registry.looks.entry(pid).or_default() += 1;
        if sample.disk_wait {
            *registry.disk_wait_looks.entry(pid).or_default() += 1;
        }
        let held: bool = registry.holds.contains_key(&pid);
        let watched: &mut Watched = registry.watched.get_mut(&pid)?;
        let began: Instant = watched.began;
        return watched.judge.observe(now.saturating_duration_since(began), &sample, held);
    });
    let Some(reason) = verdict else { return };
    let line: String = command_line_of(pid).join(" ");
    let message: String = format!(
        "the harness supervisor ended the command `{line}` in {}: {reason}\nthe tasks of its tree when it was ended:\n{}",
        directory_of(pid),
        describe(pid)
    );
    let Some(watched) = with_registry(|registry| { return registry.watched.remove(&pid); }) else { return };
    end_command(&watched.pin, message.as_str(), &end_tree);
    eprintln!("{message}");
}

/// Record the kill of the command `pin` names,
/// and only then end its tree with `end`:
/// the test that waits for the command
///  sees it end after the first signal,
/// so the record has to be there before.
pub fn end_command(pin: &Pin, message: &str, end: &dyn Fn(&Pin)) {
    with_registry(|registry| { registry.kills.push(Kill { pid: pin.pid(), message: message.to_string(), at: Instant::now() }); });
    end(pin);
    // The test has the whole grace to claim the record from the moment the command is gone: an ending that a starved host
    //  makes slow must not use the grace up before the test has had a chance to run (a gate saw the process abort that way).
    with_registry(|registry| {
        if let Some(kill) = registry.kills.iter_mut().rev().find(|kill| { return kill.pid == pin.pid(); }) {
            kill.at = Instant::now();
        }
    });
}

/// Whether a kill of the command `pid` is recorded and not claimed.
pub fn kill_recorded(pid: i32) -> bool {
    return with_registry(|registry| { return registry.kills.iter().any(|kill| { return kill.pid == pid; }); });
}

/// The message of a kill nobody has claimed within `grace` of `now`,
/// if there is one.
pub fn unclaimed_kill(now: Instant, grace: Duration) -> Option<String> {
    return with_registry(|registry| {
        return registry.kills.iter().find(|kill| { return now.saturating_duration_since(kill.at) >= grace; }).map(|kill| { return kill.message.clone(); });
    });
}

/// Record a kill as the monitor would,
/// for the control that reads what happens to one nobody claims.
pub fn plant_kill(pid: i32, message: &str, at: Instant) {
    with_registry(|registry| { registry.kills.push(Kill { pid, message: message.to_string(), at }); });
}

/// A kill nobody claimed within the grace ends the process,
/// so a command ended by the supervisor always fails its test.
fn abort_on_unclaimed(now: Instant) {
    if let Some(message) = unclaimed_kill(now, UNCLAIMED_GRACE) {
        eprintln!("a command the supervisor ended was never reported by its test, so the process aborts:\n{message}");
        std::process::abort();
    }
}
