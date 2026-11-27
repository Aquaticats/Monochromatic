//! What:
//!  One look at the run:
//! the tallies of every task of this process and of the processes it started,
//! the reads of `/proc` that fill them,
//! and the walk over the processes below this one.
//! Why:
//!   The judge compares one whole reading of the run with the one before it,
//! and the reads a look costs are the reader's own work,
//! so the tally counts them and the next look takes exactly them off.
//! The tasks' own figures say a task grew,
//! where the totals can fall while a task that stayed moved.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // look(readerThread): { bytes, ticks, working, members, tasks } | undefined
//! ```

/// The look the judge compares,
/// and the figures of one task it holds,
/// with the figure that names a task.
use super::look_figures::{
    Look,
    TaskFigures,
    task_number,
};
/// Every reader of this process,
/// whose own readings are not the run's work.
use super::readers::{
    OWN_READINGS,
    Reader,
    the_readers,
};
/// The reading of a `stat` line,
/// the states that count as work.
use crate::push_progress::{
    is_working,
    parse_activity,
};
/// Every process this one started,
/// so their work counts too.
use std::collections::HashSet;

/// What:
///  The text of a `/proc` file,
///  with every byte that is not UTF-8 replaced,
/// and the bytes the read delivered.
/// Why:
///   A read counts in this reader's own counters even where its text cannot be decoded
/// (a task name holding a byte that is not UTF-8)
///  and even where it fails part way,
/// so a length taken from a decode or from a read that reports only whether it succeeded
///  would leave some of the reader's own readings in the figures judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytes = readFileLossy(path); return [new TextDecoder().decode(bytes), bytes.length];
/// ```
fn read_a_tasks_file(path: &std::path::Path) -> (String, u64) {
    let mut bytes: Vec<u8> = Vec::new();
    if let Ok(mut file) = std::fs::File::open(path) {
        // A read that fails part way still delivered what it put here, and that is the caller's own reading too.
        let _read = std::io::Read::read_to_end(&mut file, &mut bytes);
    }
    let delivered: u64 = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    return (
        String::from_utf8_lossy(bytes.as_slice()).into_owned(),
        delivered,
    );
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
/// const tally = { bytes: 0, ticks: 0, working: false, members: 0, tasks: [], readable: false };
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
    /// What each task counted had moved.
    tasks: Vec<TaskFigures>,
    /// Whether any task's counters could be read.
    readable: bool,
}

impl Tally {
    /// What:
    ///  Add one task's figures to this look,
    ///  and to the totals that say what the run moved.
    /// Why:
    ///   A total and the tasks it was added from must never disagree,
    /// because the judge reads the tasks and the report reads the totals,
    /// so one place adds both.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// tally.noteTask({ task, bytes, ticks }); // totals += figures; members += 1; tasks.push(figures)
    /// ```
    fn note_task(
        &mut self,
        figures: TaskFigures,
    ) {
        self.members = self.members.saturating_add(1);
        self.bytes = self.bytes.saturating_add(figures.bytes);
        self.ticks = self.ticks.saturating_add(figures.ticks);
        self.tasks.push(figures);
    }

    /// What:
    ///  Add what the looking thread has moved that no reading of its own accounts for,
    ///  and count every reading this look takes of it.
    /// Why:
    ///   A reader's own work is the run's work:
    /// a predicate that makes a retry or a round does real disk work on the waiting thread,
    /// and its
    ///  `D` state is the evidence `run_progress` treats as work.
    /// Its readings of `/proc` are not,
    ///  and the kernel charges every one of them to the thread that read it.
    /// `own_readings` is the bytes its previous looks read,
    /// so the difference is the work it did of its own,
    /// and this look's
    ///  readings join that figure for the next one,
    /// counted or not:
    /// a read that delivered bytes whose text cannot be decoded is the reader's own reading too.
    /// The `io` file is read first,
    /// because the counter it yields cannot include the reading that yields it.
    /// Only the disk wait counts for its state,
    ///  never state `R`:
    /// the reader is running because it is taking this very look.
    fn add_the_readers_own_counters(
        &mut self,
        number: i32,
        reader: i32,
        task: &std::path::Path,
        own_readings: u64,
    ) {
        let (io, io_bytes) = read_a_tasks_file(&task.join("io"));
        self.read_bytes = self.read_bytes.saturating_add(io_bytes);
        // Its processor time is this look's too, so the run is judged on what it moved that was not a reading.
        let moved: u64 = bytes_in(io.as_str()).saturating_sub(own_readings);
        let (stat, stat_bytes) = read_a_tasks_file(&task.join("stat"));
        self.read_bytes = self.read_bytes.saturating_add(stat_bytes);
        let Some((state, _ticks)) = parse_activity(stat.as_str()) else {
            return;
        };
        self.readable = true;
        self.working = self.working || state == 'D';
        self.note_task(TaskFigures {
            task: task_number(number, reader),
            bytes: moved,
            ticks: 0,
        });
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
    fn add_a_readers_disk_wait(
        &mut self,
        task: &std::path::Path,
    ) {
        let (stat, stat_bytes) = read_a_tasks_file(&task.join("stat"));
        self.read_bytes = self.read_bytes.saturating_add(stat_bytes);
        let Some((state, _ticks)) = parse_activity(stat.as_str()) else {
            return;
        };
        self.readable = true;
        self.working = self.working || state == 'D';
    }

    /// Add the counters of task `task` of process `number`,
    /// whose directory under `/proc` is `folder`,
    /// and leave the tally alone where they cannot be read.
    fn add_task(
        &mut self,
        number: i32,
        task: i32,
        folder: &std::path::Path,
    ) {
        let (stat, stat_bytes) = read_a_tasks_file(&folder.join("stat"));
        self.read_bytes = self.read_bytes.saturating_add(stat_bytes);
        let Some((state, ticks)) = parse_activity(stat.as_str()) else {
            return;
        };
        self.readable = true;
        self.working = self.working || is_working(state);
        // A kernel without task I/O accounting has no `io` file; its tasks are still seen to run and to wait.
        let (io, io_bytes) = read_a_tasks_file(&folder.join("io"));
        self.read_bytes = self.read_bytes.saturating_add(io_bytes);
        let bytes: u64 = bytes_in(io.as_str());
        self.note_task(TaskFigures {
            task: task_number(number, task),
            bytes,
            ticks,
        });
    }

    /// Add every task of the process `number`,
    /// leaving out the threads `leaving_out` names,
    /// which are the watchers whose own readings are not the run's work.
    fn add_process(
        &mut self,
        number: i32,
        leaving_out: &[i32],
    ) {
        let Ok(tasks) = std::fs::read_dir(format!("/proc/{number}/task")) else {
            return;
        };
        for task in tasks.flatten() {
            let name: std::ffi::OsString = task.file_name();
            let Some(tid) = name.to_str().and_then(|text| {
                return text.parse::<i32>().ok();
            }) else {
                continue;
            };
            if leaving_out.contains(&tid) {
                continue;
            }
            self.add_task(number, tid, task.path().as_path());
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
    let Ok(own) = i32::try_from(std::process::id()) else {
        return Vec::new();
    };
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
fn children_of(
    parent: i32,
    read: &mut u64,
) -> Vec<i32> {
    let mut children: Vec<i32> = Vec::new();
    let Ok(tasks) = std::fs::read_dir(format!("/proc/{parent}/task")) else {
        return children;
    };
    for task in tasks.flatten() {
        let (text, bytes_read) = read_a_tasks_file(&task.path().join("children"));
        *read = read.saturating_add(bytes_read);
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
///   The caller's own `io` is read before anything else this look reads.
/// Every file a look reads is taken off the next look,
///  so only the counters as they were before this look read
///  anything hold the reader's work with none of its readings:
/// reading its `stat` first would leave that file's
///  length in the figures judged,
/// and a reader's stat file changes length as its own counters gain digits,
/// which made a
///  sleeping run look like it kept moving under load.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tally = { bytes: 0, ticks: 0, working: false, members: 0, readable: false };
/// const watched = [...readingThreads(), reader];
/// addTheReadersOwnCounters(readersOwnIoFirst()); addProcess(process.pid, watched); for (const child of processesBelow()) addProcess(child, []);
/// return tally.readable ? tally : undefined;
/// ```
pub(super) fn look(reader: i32) -> Option<Look> {
    let mut tally: Tally = Tally::default();
    let own: i32 = i32::try_from(std::process::id()).unwrap_or(i32::MAX);
    let own_task: String = format!("/proc/{own}/task/{reader}");
    let readers: Vec<Reader> = the_readers();
    let mut leaving_out: Vec<i32> = readers
        .iter()
        .map(|each: &Reader| {
            return each.thread;
        })
        .collect();
    if !leaving_out.contains(&reader) {
        leaving_out.push(reader);
    }
    // What the looking thread moved that no reading of its own accounts for is the run's work, and the reading that
    //  learns this is itself taken off the next look (`Tally::add_the_readers_own_counters`).
    let own_readings: u64 = OWN_READINGS.with(std::cell::Cell::get);
    tally.add_the_readers_own_counters(
        own,
        reader,
        std::path::Path::new(own_task.as_str()),
        own_readings,
    );
    // Another reader's disk wait is work whatever reader took the look, and nothing else of it is counted: its bytes
    //  would be its own readings, which only its own looks can take off.
    for each in &readers {
        if each.thread == reader {
            continue;
        }
        tally.add_a_readers_disk_wait(std::path::Path::new(
            format!("/proc/{own}/task/{}", each.thread).as_str(),
        ));
    }
    tally.add_process(own, leaving_out.as_slice());
    let mut read_by_the_walk: u64 = 0;
    for child in processes_below(&mut read_by_the_walk) {
        tally.add_process(child, &[]);
    }
    tally.read_bytes = tally.read_bytes.saturating_add(read_by_the_walk);
    // Every byte this look read was read by this thread, so its next look takes exactly these off. The booking comes
    //  before the look is judged readable, so a look that found no task's counters to read still leaves nothing of its
    //  own readings behind.
    OWN_READINGS.with(|readings: &std::cell::Cell<u64>| {
        readings.set(own_readings.saturating_add(tally.read_bytes));
    });
    if !tally.readable {
        return None;
    }
    return Some(Look {
        bytes: tally.bytes,
        ticks: tally.ticks,
        working: tally.working,
        members: tally.members,
        tasks: tally.tasks,
    });
}
