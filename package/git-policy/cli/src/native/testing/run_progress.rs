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
/// Writing the report on standard error,
/// whose failure is ignored so the ending always runs.
use std::io::Write as _;
/// Time spans and the monotonic clock.
use std::time::{Duration, Instant};

/// What:
///  The threads that read this run,
/// as the kernel numbers them:
/// every watcher,
/// and every thread inside a
///  control's wait.
/// Why:
///   A reader reads the counters of every task,
/// and those reads are its own work,
/// not the run's;
/// a second reader
///  (a control arms a watcher beside the fixture's,
/// or waits beside another control's wait)
/// would otherwise count the other's
///  reads at every look,
/// so two of them would keep each other,
/// and the run they read,
/// alive for ever.
static READING_THREADS: std::sync::Mutex<Vec<Reader>> = std::sync::Mutex::new(Vec::new());

/// What:
///  A thread that reads this run,
///  and the bytes its own readings of `/proc` have added so far.
/// Why:
///   A reader's work is the run's work and its readings are not,
/// and the counters are monotonic,
/// so the two are only told
///  apart by what each look added to its own reader's counters:
/// that figure is kept here and subtracted from the reader's counters at every
///  look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reader = { thread: number; ownReads: number };
/// ```
#[derive(Clone, Copy)]
struct Reader {
    /// The thread the kernel numbers for the reader.
    thread: i32,
    /// The bytes this reader's own looks have added to its counters.
    own_reads: u64,
}

/// What:
///  A thread registered as a reader of this run,
/// for as long as the value lives.
/// Why:
///   The reads of a waiter happen between its looks as well as inside them,
/// and a look taken meanwhile must
///  leave them out too,
/// so the registration lasts the whole wait rather than one reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using reading = Reading.begin(); // unregistered when the block ends
/// ```
pub(crate) struct Reading {
    /// The thread the kernel numbers for the reader.
    thread: i32,
}

impl Reading {
    /// What:
    ///  Register the calling thread as a reader of this run,
    ///  or nothing where this system numbers no thread,
    /// in which case no look can read the run
    ///  either.
    /// Why:
    ///   Every reader registers,
    /// so no look taken by any of them counts another's readings as the run's work.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const reading = Reading.begin();
    /// ```
    pub(crate) fn begin() -> Option<Reading> {
        let thread: i32 = current_thread_id()?;
        let Ok(mut reading) = READING_THREADS.lock() else { return None };
        reading.push(Reader { thread, own_reads: 0 });
        return Some(Reading { thread });
    }
}

/// What:
///  Note what the look this reader took added to its own counters,
///  so the next look counts the reader's work and not its readings.
/// Why:
///   Each look reads one file per task,
/// and those bytes are in the reader's own counters before the next look;
/// subtracting them is what lets the
///  reader's own work (a predicate that retries, a round it runs) still count.
fn note_a_reader_s_own_reads(reader: i32, added: u64) {
    let Ok(mut reading) = READING_THREADS.lock() else { return };
    for each in reading.iter_mut() {
        if each.thread == reader {
            each.own_reads = each.own_reads.saturating_add(added);
            return;
        }
    }
}

/// A reader that ends,
/// however its thread ends,
/// is no longer left out.
impl Drop for Reading {
    fn drop(&mut self) {
        let Ok(mut reading) = READING_THREADS.lock() else { return };
        // The last registration of this thread, because nested reads of one thread end in the order they began.
        if let Some(at) = reading.iter().rposition(|each: &Reader| { return each.thread == self.thread; }) {
            let _gone: Reader = reading.remove(at);
        }
    }
}

/// The threads a look leaves out,
/// which is every reader of this process.
pub(crate) fn reading_threads() -> Vec<i32> {
    let Ok(reading) = READING_THREADS.lock() else { return Vec::new() };
    return reading.iter().map(|each: &Reader| { return each.thread; }).collect();
}

/// The readers of this process,
/// with the bytes their own looks have added to their counters.
fn the_readers() -> Vec<Reader> {
    let Ok(reading) = READING_THREADS.lock() else { return Vec::new() };
    return reading.clone();
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
    /// Every byte this look itself read,
    /// which is the caller's own work and never the run's.
    read_bytes: u64,
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
    /// What:
    ///  Add the counters a reader had before this look,
    ///  so the readings the look itself makes are not the run's work.
    /// Why:
    ///   A reader's own work is the run's work:
    /// a predicate that makes a retry or a round does real disk work on the waiting thread,
    /// and its
    ///  `D` state is the evidence `run_progress` treats as work.
    /// Its readings of `/proc` are not.
    /// Reading its counters before the walk
    ///  separates the two,
    /// so the walk may count that thread's work and still leave out the bytes the walk adds to it.
    /// Only the disk wait counts as its work here,
    ///  never state `R`:
    /// the reader is running because it is taking this very look.
    fn add_a_readers_own_work(&mut self, task: &std::path::Path, own_reads: u64) -> u64 {
        let Ok(stat) = std::fs::read_to_string(task.join("stat")) else { return 0 };
        self.read_bytes = self.read_bytes.saturating_add(stat.len() as u64);
        let Some((state, _ticks)) = parse_activity(stat.as_str()) else { return 0 };
        self.readable = true;
        // The reader runs because it is taking this very look, so only its disk wait is its work.
        self.working = self.working || state == 'D';
        let io: String = std::fs::read_to_string(task.join("io")).unwrap_or_default();
        self.read_bytes = self.read_bytes.saturating_add(io.len() as u64);
        // Its processor time is this look's too, so the run is judged on the bytes the reader moved, its own readings
        //  taken off.
        let work: u64 = bytes_in(io.as_str()).saturating_sub(own_reads);
        self.bytes = self.bytes.saturating_add(work);
        return work;
    }

    /// What:
    ///  Add what another reader is waiting for the disk with,
    ///  and nothing else of it.
    /// Why:
    ///   Another reader's counters carry the reads its own looks make,
    /// and only that reader knows what those were worth,
    ///  so its
    /// bytes are left to its own looks and its disk wait is counted here,
    /// which is work whatever reader took the look.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// tally.addAReadersDiskWait(`/proc/${pid}/task/${tid}`);
    /// ```
    fn add_a_readers_disk_wait(&mut self, task: &std::path::Path) {
        let Ok(stat) = std::fs::read_to_string(task.join("stat")) else { return };
        self.read_bytes = self.read_bytes.saturating_add(stat.len() as u64);
        let Some((state, _ticks)) = parse_activity(stat.as_str()) else { return };
        self.readable = true;
        self.working = self.working || state == 'D';
    }

    /// Add the counters of the task whose directory under `/proc` is `task`,
    /// and leave the tally alone where they cannot be read.
    fn add_task(&mut self, task: &std::path::Path) {
        let Ok(stat) = std::fs::read_to_string(task.join("stat")) else { return };
        self.read_bytes = self.read_bytes.saturating_add(stat.len() as u64);
        let Some((state, ticks)) = parse_activity(stat.as_str()) else { return };
        self.readable = true;
        self.members = self.members.saturating_add(1);
        self.ticks = self.ticks.saturating_add(ticks);
        self.working = self.working || is_working(state);
        // A kernel without task I/O accounting has no `io` file; its tasks are still seen to run and to wait.
        let io: Option<String> = std::fs::read_to_string(task.join("io")).ok();
        self.read_bytes = self.read_bytes.saturating_add(io.as_ref().map_or(0, |text: &String| { return text.len() as u64; }));
        let bytes: u64 = io.map_or(0, |text: String| { return bytes_in(text.as_str()); });
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
fn processes_below(read: &mut u64) -> Vec<i32> {
    let Ok(own) = i32::try_from(std::process::id()) else { return Vec::new() };
    let mut found: Vec<i32> = Vec::new();
    let mut seen: HashSet<i32> = HashSet::from([own]);
    let mut frontier: Vec<i32> = vec![own];
    while let Some(parent) = frontier.pop() {
        for child in children_of(parent, read) {
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
fn children_of(parent: i32, read: &mut u64) -> Vec<i32> {
    let mut children: Vec<i32> = Vec::new();
    let Ok(tasks) = std::fs::read_dir(format!("/proc/{parent}/task")) else { return children };
    for task in tasks.flatten() {
        let Ok(text) = std::fs::read_to_string(task.path().join("children")) else { continue };
        *read = read.saturating_add(text.len() as u64);
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
/// leaving out the counters of `reader` and of every other reader,
/// or nothing where this system counts no
///  task's work.
/// Why:
///   The readers' own reads would otherwise be a sign of work at every look,
/// so a run that had stopped
///  would never be judged quiet,
/// and one reader would keep another alive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tally = { bytes: 0, ticks: 0, working: false, members: 0, readable: false };
/// const watched = [...readingThreads(), reader];
/// addProcess(process.pid, watched); for (const child of processesBelow()) addProcess(child, []);
/// return tally.readable ? tally : undefined;
/// ```
fn look(reader: i32) -> Option<Look> {
    let mut tally: Tally = Tally::default();
    let own: i32 = i32::try_from(std::process::id()).unwrap_or(i32::MAX);
    let readers: Vec<Reader> = the_readers();
    let mut leaving_out: Vec<i32> = readers.iter().map(|each: &Reader| { return each.thread; }).collect();
    if !leaving_out.contains(&reader) {
        leaving_out.push(reader);
    }
    // Every thread whose readings are not the run's work is read first, with its own readings taken off: its own work
    //  still counts, and the bytes this walk adds to its counters do not.
    for each in &readers {
        let task: String = format!("/proc/{own}/task/{}", each.thread);
        if each.thread == reader {
            tally.add_a_readers_own_work(std::path::Path::new(task.as_str()), each.own_reads);
        } else {
            tally.add_a_readers_disk_wait(std::path::Path::new(task.as_str()));
        }
    }
    if !readers.iter().any(|each: &Reader| { return each.thread == reader; }) {
        tally.add_a_readers_own_work(std::path::Path::new(format!("/proc/{own}/task/{reader}").as_str()), 0);
    }
    tally.add_process(own, leaving_out.as_slice());
    let mut read_by_the_walk: u64 = 0;
    for child in processes_below(&mut read_by_the_walk) {
        tally.add_process(child, &[]);
    }
    tally.read_bytes = tally.read_bytes.saturating_add(read_by_the_walk);
    if !tally.readable {
        return None;
    }
    // Every byte this look read was read by the caller, so it is in the caller's own counters before the next look: the
    //  next look takes exactly these off, and the caller's own work still counts.
    note_a_reader_s_own_reads(reader, tally.read_bytes);
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
///  One look at this run now,
/// taken by the thread that calls it,
/// for a control that ends a wait when the run stops working.
/// Why:
///   The controls judge a wait by the same evidence the watchdog does:
/// a run that keeps moving is left alone,
/// however long its event takes,
/// and only a run that shows nothing is over.
/// The calling thread's own readings are left out,
/// as every other reader's are,
/// so a control that looks for its event is not itself the work it waits for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lookNow(): Look | undefined { looksTaken.set(looksTaken.get() + 1); return look(currentThreadId()) }
/// ```
pub(crate) fn look_now() -> Option<Look> {
    LOOKS_TAKEN.with(|taken: &std::cell::Cell<u64>| { taken.set(taken.get().saturating_add(1)); });
    return current_thread_id().and_then(look);
}

std::thread_local! {
    /// The looks `look_now` has taken on this thread.
    static LOOKS_TAKEN: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// What:
///  How many looks the calling thread has taken at this run.
/// Why:
///   A control that says a wait with nothing to wait for does not look at the run cannot measure how long the call took:
/// a thread
///  the scheduler holds just before the call makes a wait that returned at once look slow,
/// and a thread it holds just
///  after a sleep makes one that slept look quick.
/// The count is the event itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function looksTakenByThisThread(): number;
/// ```
pub(crate) fn looks_taken_by_this_thread() -> u64 {
    return LOOKS_TAKEN.with(std::cell::Cell::get);
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
    // The reader's own readings are not the run's work, and neither is another reader's (`look`).
    let _reading: Option<Reading> = Reading::begin();
    loop {
        std::thread::sleep(SAMPLE_INTERVAL);
        if let Some(ending) = judge.observe(started.elapsed(), look(reader)) {
            end(ending);
        }
    }
}

/// What:
///  Say why the run ended and end the process,
/// whatever became of the report.
/// Why:
///   The line is what the person sees in the test log,
/// so it names the window that passed and the signs that were
///  read,
/// and there is no other way to end a stuck run from its own thread.
/// A report that cannot be written (a full disk,
/// a closed pipe) must not be a precondition for ending:
/// a
///  panicking macro would kill only this thread,
/// and the test it watches would stay blocked without a watcher.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function end(ending) { try { process.stderr.write(`the unit-test run ${ending}: a test is stuck\n`) } catch {} process.abort(); }
/// ```
fn end(ending: Ending) -> ! {
    // The error is ignored on purpose: `eprintln!` panics when the write fails, and only this thread would die.
    let mut stderr = std::io::stderr().lock();
    let _written = writeln!(stderr, "the unit-test run {ending}: a test is stuck");
    std::process::abort();
}

/// Controls for the judgment,
/// the look and the ending.
#[path = "run_progress_tests.rs"]
mod tests;
