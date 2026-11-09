//! What:
//!  Whether the unit-test run is moving:
//! every task of this process and of the processes it started,
//! their states,
//! their
//!  processor time and the bytes they moved,
//! and the clock that ends a run which has shown none of them for a window.
//! Why:
//!  A wall-clock limit cannot tell a stuck run from a slow one.
//! The tests that copy a checkout into btrfs subvolumes spend nearly all of their time
//!  waiting in the kernel (measured 2026-11-08 on this host,
//! `doc/troubleshooting/git-policy-cli-unit-test-watchdog-judges-progress.md`):
//! 43 of 47 s of one
//!  of them alone,
//! and 64 of 69 s of another while its family ran,
//! in state `D` inside the btrfs transaction and log
//!  commit waits,
//! so the run passed a 120 s wall clock as soon as the disk
//!  was busy and the failures followed the host's load instead of the code.
//! What a run that is stuck looks like instead is a run that shows nothing:
//! no task able to run,
//!  no task waiting for the disk,
//!  no processor time and no bytes moved,
//!  and no process or task started or ended.
//!  Those are the signs the wrapper's own wait for a child process reads (`push_progress.rs`),
//! so a test run
//!  and a shipped wait are judged alike.
//! A run that keeps moving is left alone,
//! and a run that moves for ever is
//!  ended by the test runner's own per-test limit (`.config/nextest.toml`,
//! `slow-timeout` 60 s times
//!  `terminate-after` 5),
//! so nothing waits without a bound.
//! A look that cannot be read (a task that ended while it was read,
//!  or a process out of file descriptors)
//!  is neither a sign of work nor proof that the run stopped:
//! the clock keeps running through it,
//! so a run
//!  whose counters never come back still ends,
//! and a later look that reads the run counts again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const judge = new Judge(windowMs); setInterval(() => judge.observe(elapsedMs, lookAt(process.pid)), SAMPLE_MS);
//! ```

/// The reading of a `stat` line,
/// the states that count as work,
/// and how often a run is looked at.
use crate::push_progress::{SAMPLE_INTERVAL, is_working, parse_activity};
/// The kernel's number for the calling thread,
/// which the look leaves out so its own reading never counts as the run's work.
use crate::thread_quiet::current_thread_id;
/// Every process this one started,
/// so their work counts too.
use std::collections::HashSet;
/// Time spans and the monotonic clock.
use std::time::{Duration, Instant};

/// What:
///  The threads that watch this run,
/// as the kernel numbers them.
/// Why:
///   A watcher reads the counters of every task,
/// and those reads are its own work,
/// not the run's;
/// a second watcher
///  (a control arms one beside the fixture's)
/// would otherwise count the other's reads at every look,
/// so two of them would
///  keep each other, and the run they watch, alive for ever.
static WATCHING_THREADS: std::sync::Mutex<Vec<i32>> = std::sync::Mutex::new(Vec::new());

/// Add `thread` to the watchers a look leaves out.
fn start_watching(thread: i32) {
    let Ok(mut watching) = WATCHING_THREADS.lock() else { return };
    watching.push(thread);
}

/// The threads a look leaves out,
/// which is every watcher of this process.
fn watching() -> Vec<i32> {
    let Ok(watching) = WATCHING_THREADS.lock() else { return Vec::new() };
    return watching.clone();
}

/// What:
///  What one look at the run found,
/// added up over every task of this process and of the processes it started.
/// Why:
///   Bytes and processor ticks only grow,
/// and a task able to run or waiting for the disk is at work now,
/// so a look
///  says whether anything moved since the one before it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Look = { bytes: number; ticks: number; working: boolean; members: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Look {
    /// Every byte the run's tasks read and wrote,
    /// from their `io` files.
    pub bytes: u64,
    /// Every processor tick the run's tasks used,
    /// from their `stat` files.
    pub ticks: u64,
    /// Whether some task is able to run (`R`) or waiting for the disk (`D`).
    pub working: bool,
    /// How many tasks were seen.
    pub members: u64,
}

/// What:
///  Why one run was ended.
/// Why:
///   A run that showed nothing,
/// a run whose counters stopped being readable,
/// and a system that counts no work at all
///  end for different reasons,
/// and the message has to say which,
/// so the person can tell a stuck test from a system
///  without the counters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Ending =
///     | { kind: 'quiet'; quietMs: number }
///     | { kind: 'unreadable'; unreadMs: number }
///     | { kind: 'unmeasurable'; elapsedMs: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Ending {
    /// Nothing moved for the window.
    Quiet(Duration),
    /// The window passed and no task's counters could be read,
    /// though earlier looks read them.
    Unreadable(Duration),
    /// The window passed and this system counts no work,
    /// so the wall clock is its bound.
    Unmeasurable(Duration),
}

/// The reason in words,
/// with the figure that decided it and the signs that were read.
impl std::fmt::Display for Ending {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Ending::Quiet(quiet) => write!(
                formatter,
                "showed no sign of work for {:.1} s (no task able to run or waiting for the disk, no processor time, no bytes moved, and no process or task started or ended)",
                quiet.as_secs_f64()
            ),
            Ending::Unreadable(unread) => write!(
                formatter,
                "could not read its tasks' counters for {:.1} s (a task that ended while it was read, or a process that ran out of file descriptors), so it is ended rather than watched for ever",
                unread.as_secs_f64()
            ),
            Ending::Unmeasurable(elapsed) => write!(
                formatter,
                "ran {:.1} s and this system counts no work of its tasks, so its wall clock is the bound",
                elapsed.as_secs_f64()
            ),
        };
    }
}

/// What:
///  The tallies one look adds up,
/// and whether it could read any of them.
/// Why:
///   A task that ended while it was read is skipped,
/// so a look is assembled from as many counters as could be read
///  at that moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tally = { bytes: 0, ticks: 0, working: false, members: 0, readable: false };
/// ```
#[derive(Debug, Default)]
struct Tally {
    /// Every byte read and written so far.
    bytes: u64,
    /// Every processor tick used so far.
    ticks: u64,
    /// Whether some task is able to run or waiting for the disk.
    working: bool,
    /// How many tasks were counted.
    members: u64,
    /// Whether any task's counters could be read.
    readable: bool,
}

impl Tally {
    /// Add the counters of the task whose directory under `/proc` is `task`,
    /// and leave the tally alone where they cannot be read.
    fn add_task(&mut self, task: &std::path::Path) {
        let Ok(stat) = std::fs::read_to_string(task.join("stat")) else { return };
        let Some((state, ticks)) = parse_activity(stat.as_str()) else { return };
        self.readable = true;
        self.members = self.members.saturating_add(1);
        self.ticks = self.ticks.saturating_add(ticks);
        self.working = self.working || is_working(state);
        // A kernel without task I/O accounting has no `io` file; its tasks are still seen to run and to wait.
        let bytes: u64 = std::fs::read_to_string(task.join("io")).map_or(0, |text| { return bytes_in(text.as_str()); });
        self.bytes = self.bytes.saturating_add(bytes);
    }

    /// Add every task of the process `number`,
    /// leaving out the threads `leaving_out` names,
    /// which are the watchers whose own readings are not the run's work.
    fn add_process(&mut self, number: i32, leaving_out: &[i32]) {
        let Ok(tasks) = std::fs::read_dir(format!("/proc/{number}/task")) else { return };
        for task in tasks.flatten() {
            let name: std::ffi::OsString = task.file_name();
            let Some(tid) = name.to_str().and_then(|text| { return text.parse::<i32>().ok(); }) else { continue };
            if leaving_out.contains(&tid) {
                continue;
            }
            self.add_task(task.path().as_path());
        }
    }
}

/// What:
///  The bytes a thread has read and written,
/// added up,
/// from the text of its `io` file.
/// Why:
///   `rchar` and `wchar` are the counters that see pipes and files,
/// and they only grow,
/// so their sum says whether a
///  thread moved anything between two looks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const io = Object.fromEntries(text.split('\n').map((line) => line.split(': ')));
/// return Number(io.rchar) + Number(io.wchar);
/// ```
fn bytes_in(text: &str) -> u64 {
    let mut read: u64 = 0;
    let mut written: u64 = 0;
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("rchar: ") {
            read = value.trim().parse().unwrap_or(0);
        } else if let Some(value) = line.strip_prefix("wchar: ") {
            written = value.trim().parse().unwrap_or(0);
        }
    }
    return read.saturating_add(written);
}

/// What:
///  The processes this process started,
/// from the `children` file of every task.
/// Why:
///   A child belongs to the task that forked it,
/// so asking only the main thread would miss the children of every
///  other one,
/// and the work a test's Git commands do is most of what it waits for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const found: number[] = []; const queue = [process.pid];
/// while (queue.length) for (const child of childrenOf(queue.pop())) { found.push(child); queue.push(child) }
/// ```
fn processes_below() -> Vec<i32> {
    let Ok(own) = i32::try_from(std::process::id()) else { return Vec::new() };
    let mut found: Vec<i32> = Vec::new();
    let mut seen: HashSet<i32> = HashSet::from([own]);
    let mut frontier: Vec<i32> = vec![own];
    while let Some(parent) = frontier.pop() {
        for child in children_of(parent) {
            if !seen.insert(child) {
                continue;
            }
            found.push(child);
            frontier.push(child);
        }
    }
    return found;
}

/// What:
///  The processes the tasks of `parent` started,
/// read from each task's own `children` file.
/// Why:
///   The file names the children of one task,
/// which is the thread that forked them,
/// and it is a small read,
///  where
/// listing every process of the machine to build the tree would read one file for each of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readdirSync(`/proc/${parent}/task`).flatMap((tid) => readFileSync(`/proc/${parent}/task/${tid}/children`, 'utf8').trim().split(/\s+/));
/// ```
fn children_of(parent: i32) -> Vec<i32> {
    let mut children: Vec<i32> = Vec::new();
    let Ok(tasks) = std::fs::read_dir(format!("/proc/{parent}/task")) else { return children };
    for task in tasks.flatten() {
        let Ok(text) = std::fs::read_to_string(task.path().join("children")) else { continue };
        for word in text.split_whitespace() {
            if let Ok(child) = word.parse::<i32>() {
                children.push(child);
            }
        }
    }
    return children;
}

/// What:
///  One look at the run,
/// leaving out the counters of `reader` and of every other watcher,
/// or nothing where this system counts no
///  task's work.
/// Why:
///   The watchers' own reads would otherwise be a sign of work at every look,
/// so a run that had stopped
///  would never be judged quiet,
/// and one watcher would keep another alive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tally = { bytes: 0, ticks: 0, working: false, members: 0, readable: false };
/// const watched = [...watching(), reader];
/// addProcess(process.pid, watched); for (const child of processesBelow()) addProcess(child, []);
/// return tally.readable ? tally : undefined;
/// ```
fn look(reader: i32) -> Option<Look> {
    let mut tally: Tally = Tally::default();
    let own: i32 = i32::try_from(std::process::id()).unwrap_or(i32::MAX);
    let mut leaving_out: Vec<i32> = watching();
    leaving_out.push(reader);
    tally.add_process(own, leaving_out.as_slice());
    for child in processes_below() {
        tally.add_process(child, &[]);
    }
    if !tally.readable {
        return None;
    }
    return Some(Look { bytes: tally.bytes, ticks: tally.ticks, working: tally.working, members: tally.members });
}

/// What:
///  The clock of the run:
/// it compares each look with the one before it and says when the run has shown nothing for
///  its window.
/// Why:
///   Nothing is ended here;
/// the caller decides what an ending does,
/// which is what lets the controls drive the
///  judgment with made-up looks and times instead of waiting a window out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Judge { constructor(windowMs) {} observe(elapsedMs, look?: Look): Ending | undefined }
/// ```
#[derive(Debug)]
pub(crate) struct Judge {
    /// How long the run may show nothing before it counts as stuck.
    window: Duration,
    /// When the last look that showed something was taken.
    last_change: Duration,
    /// When the last look that could read this system's counters was taken,
    /// or nothing where no look has.
    last_read: Option<Duration>,
    /// The look before the current one,
    /// to compare with.
    previous: Option<Look>,
}

impl Judge {
    /// A judge for a window,
    /// which has seen nothing yet.
    pub(crate) fn new(window: Duration) -> Judge {
        return Judge { window, last_change: Duration::ZERO, last_read: None, previous: None };
    }

    /// Judge one look taken `elapsed` after the run started;
    /// `Some` is the reason the run ends.
    ///  A look that could not be read is no sign of work and no proof that the run stopped:
    /// the clock keeps
    ///  running through it,
    /// so a run whose counters never come back ends,
    /// and a later look that reads the run counts again.
    pub(crate) fn observe(&mut self, elapsed: Duration, look: Option<Look>) -> Option<Ending> {
        let Some(current): Option<Look> = look else {
            return self.ending_without_a_look(elapsed);
        };
        self.last_read = Some(elapsed);
        let moved: bool = self.previous.is_none_or(|before: Look| {
            return before.bytes != current.bytes || before.ticks != current.ticks || before.members != current.members;
        });
        if moved || current.working {
            self.last_change = elapsed;
        }
        self.previous = Some(current);
        return self.ending_after_the_quiet_stretch(elapsed);
    }

    /// What:
    ///  The ending of a look that read nothing.
    /// Why:
    ///   Nothing was read,
    /// so the look is no sign of work and no proof that the run stopped;
    /// the clock keeps
    ///  running,
    /// and a system whose counters never come back ends as unreadable rather than as quiet,
    /// because
    ///  quiet would claim signs this run never showed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const endingWithoutALook = (elapsedMs) => lastRead === undefined
    ///     ? (elapsedMs >= windowMs ? unmeasurable(elapsedMs) : undefined)
    ///     : (elapsedMs - lastRead >= windowMs ? unreadable(elapsedMs - lastRead) : quiet(elapsedMs));
    /// ```
    fn ending_without_a_look(&self, elapsed: Duration) -> Option<Ending> {
        let Some(last_read): Option<Duration> = self.last_read else {
            // No look has read this system's counters yet, so until one does the wall clock is the bound.
            return if elapsed >= self.window { Some(Ending::Unmeasurable(elapsed)) } else { None };
        };
        let unread: Duration = elapsed.saturating_sub(last_read);
        if unread >= self.window {
            return Some(Ending::Unreadable(unread));
        }
        return self.ending_after_the_quiet_stretch(elapsed);
    }

    /// What:
    ///  The ending of a run that has shown nothing since `last_change`,
    /// or nothing before its window
    ///  has passed.
    /// Why:
    ///   Every ending of a run some look could read goes through this one comparison of the clock,
    /// so a look
    ///  that read nothing is measured on the same clock as one that read a still run.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const quiet = elapsedMs - lastChangeMs; return quiet >= windowMs ? { kind: 'quiet', quietMs: quiet } : undefined;
    /// ```
    fn ending_after_the_quiet_stretch(&self, elapsed: Duration) -> Option<Ending> {
        let quiet: Duration = elapsed.saturating_sub(self.last_change);
        return if quiet >= self.window { Some(Ending::Quiet(quiet)) } else { None };
    }
}

/// What:
///  Start the one thread that ends this process when the run has shown no sign of work for `window`.
/// Why:
///   A stuck test has to end the run itself rather than wait for an outside timeout,
/// which the mutation tool
///  records as a timeout instead of a failed test;
/// the window is what a stuck run costs,
/// where a wall clock would end
///  every run that is merely slow.
/// The thread is started where a process is allowed one,
/// and a process that is given none runs
///  without it (a control of a site that starts threads).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function watch(windowMs) { try { spawnThread(() => run(windowMs)); } catch {} }
/// ```
pub(crate) fn watch(window: Duration) -> Option<std::thread::JoinHandle<()>> {
    return std::thread::Builder::new().name(String::from("unit-test-watchdog")).spawn(move || { run(window) }).ok();
}

/// What:
///  Look at the run until it ends the process.
/// Why:
///   The loop is the whole of the watchdog:
/// one look at each interval,
/// one judgment,
/// and the process ends when
///  the judgment says the run stopped moving.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function run(windowMs) { let started = Date.now(); let judge = new Judge(windowMs); for (;;) { sleep(500); if (judge.observe(Date.now() - started, look(reader))) { abort() } } }
/// ```
fn run(window: Duration) {
    let started: Instant = Instant::now();
    let Some(reader): Option<i32> = current_thread_id() else {
        // Where the kernel numbers no thread, nothing can be read, and the window is a fixed deadline as it was before.
        std::thread::sleep(window);
        end(Ending::Unmeasurable(window));
    };
    let mut judge: Judge = Judge::new(window);
    // The watcher's own readings are not the run's work, and neither are another watcher's (`look`).
    start_watching(reader);
    loop {
        std::thread::sleep(SAMPLE_INTERVAL);
        if let Some(ending) = judge.observe(started.elapsed(), look(reader)) {
            end(ending);
        }
    }
}

/// What:
///  Say why the run ended and end the process.
/// Why:
///   The line is what the person sees in the test log,
/// so it names the window that passed and the signs that were
///  read,
/// and there is no other way to end a stuck run from its own thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function end(ending) { console.error(`the unit-test run ${ending}: a test is stuck`); process.abort(); }
/// ```
fn end(ending: Ending) -> ! {
    eprintln!("the unit-test run {ending}: a test is stuck");
    std::process::abort();
}

/// Controls for the judgment,
/// the look and the ending.
#[path = "run_progress_tests.rs"]
mod tests;
