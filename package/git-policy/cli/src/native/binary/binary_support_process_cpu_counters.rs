//! What:
//!  The clock ticks one process used,
//! as the kernel counts them,
//! summed over every thread of it.
//! Why:
//!   `/proc/<pid>/stat` is the main thread's alone,
//! and a commit service does its looking on a worker of its runtime:
//! a reading of the main thread alone showed no processor time and no switch for a service that looked every five seconds
//! (`binary_a_record_nothing_announced_is_found_tests.rs`).
//! A process starts and ends threads while it serves
//! (the build of its runtime,
//! the bounded wait of one connection),
//! so a thread a listing names can be gone before its own file is read,
//! and a read that failed over that thread would name a service that is still there as one that ended
//! (the gate of 2026-11-15T06:00-05:00,
//! `a_record_nothing_announced_is_found::an_idle_service_runs_no_work`,
//! whose first reading met the thread that builds the service's runtime as it went).
//! So the sum counts the threads each moment has,
//! and the one answer that means "no counters" is the process itself being gone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const used = ticksOfThreads(pid, taskNumbers(pid)); // both read now, and a thread that went spends nothing more
//! ```

/// What:
///  The clock ticks one process used,
///  over every thread of it,
///  or nothing when the process is gone.
/// Why:
///   The process's own file is the one thing that has to be readable:
/// it is there while the process runs and gone once it ended,
///  while a thread's own file can be gone while the process that had it serves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const used = ticksOf(readFile(`/proc/${pid}/stat`)) ? ticksOfThreads(pid, taskNumbers(pid)) : undefined;
/// ```
pub fn cpu_ticks(pid: u32) -> Option<u64> {
    let _process: String = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let threads: Vec<u32> = thread_numbers(pid).ok()?;
    return Some(ticks_of_threads(pid, threads.as_slice()));
}

/// What:
///  The numbers of every thread of `pid` now,
///  or the reason the process's own directory could not be read.
/// Why:
///   A caller that lists and then reads one file per thread is what makes the listing and the reading two moments,
/// so the listing is a value of its own a control can name.
/// A name in the directory that is not a number is no thread and is passed over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const numbers = readdir(`/proc/${pid}/task`).filter((name) => /^[0-9]+$/.test(name)).map(Number);
/// ```
pub fn thread_numbers(pid: u32) -> std::io::Result<Vec<u32>> {
    let mut numbers: Vec<u32> = Vec::new();
    for listed in std::fs::read_dir(format!("/proc/{pid}/task"))? {
        let Ok(entry) = listed else { continue };
        if let Some(number) = entry.file_name().to_str().and_then(|name| { return name.parse::<u32>().ok(); }) {
            numbers.push(number);
        }
    }
    return Ok(numbers);
}

/// What:
///  The clock ticks the threads `numbers` name that are still there when their own file is read.
/// Why:
///   A thread that went between the listing and its own read spent nothing more,
/// and the process that had it is still there:
/// the sum is of the threads this moment has, and no absence among them is a reason to answer nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const used = numbers.map((n) => threadTicks(pid, n) ?? 0).reduce((a, b) => a + b, 0);
/// ```
pub fn ticks_of_threads(pid: u32, numbers: &[u32]) -> u64 {
    let mut used: u64 = 0;
    for number in numbers {
        if let Some(ticks) = thread_ticks(pid, *number) {
            used += ticks;
        }
    }
    return used;
}

/// What:
///  The clock ticks one thread used,
///  or nothing when its own file cannot be read,
///  which is the answer for a thread that ended.
/// Why:
///   The field after the program's name,
/// which may hold spaces and brackets,
///  is the state,
/// and the user and system times are the twelfth and thirteenth of those that follow,
/// the same fields `/proc/<pid>/stat` gives for a process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const f = readFile(`/proc/${pid}/task/${tid}/stat`).split(')')[1].trim().split(/\s+/); return Number(f[11]) + Number(f[12]);
/// ```
pub fn thread_ticks(pid: u32, thread: u32) -> Option<u64> {
    let stat: String = std::fs::read_to_string(format!("/proc/{pid}/task/{thread}/stat")).ok()?;
    let after_name: &str = stat.rsplit_once(')')?.1;
    let fields: Vec<&str> = after_name.split_whitespace().collect();
    let user: u64 = fields.get(11)?.parse().ok()?;
    let system: u64 = fields.get(12)?.parse().ok()?;
    return Some(user + system);
}

/// What:
///  The clock ticks in one second,
///  which turns a difference of two readings into a duration.
/// Why:
///   The kernel counts in its own unit and says which,
/// so no number here may be assumed;
/// a system that will not say gets the hundred the kernel has used on every Linux build this package runs on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const perSecond = sysconf(_SC_CLK_TCK) || 100;
/// ```
pub fn ticks_per_second() -> f64 {
    // SAFETY: `sysconf` reads a constant of the system and touches no memory of this process.
    let per_second: i64 = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    return if per_second > 0 { per_second as f64 } else { 100.0 };
}
