//! What:
//!  The clock ticks one process used,
//! as the kernel counts them,
//! read from the process's own file and never summed over a listing of its threads.
//! Why:
//!   The first round of this branch summed one reading per thread,
//! on the comment that `/proc/<pid>/stat` is the main thread's alone and that a service does its looking on a worker of
//! its runtime,
//!  so a reading of the main thread alone showed no processor time for a service that looked every five seconds
//! (`binary_a_record_nothing_announced_is_found_tests.rs`).
//! Measured on this machine, the process's own file reports the whole thread group and keeps what a thread spent after the
//! thread ends:
//! 4 ticks before a worker thread's work, 123 while it worked, and 123 again after the worker ended,
//!  while the sum over the threads fell back to 3.
//! So the sum was never what made the look's work visible, and it is the one reading that loses a thread that ends:
//! a service starts and ends threads while it serves,
//!  and a difference of two sums then goes down and overflows
//!  (the review of 2026-11-15, its fourth and fifth findings: 8 ticks before and 5 after in its reproduction).
//! Liveness is asked of the process's state and not of a file being readable,
//! because the files of a killed child stay readable until something reaps it.
//!  The listing of the threads stays for what only it can say,
//! which is what one thread spent (`thread_ticks`) and what the threads of one moment sum to (`ticks_of_threads`),
//! and a control shows there what a sum cannot promise.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const used = ticksOfProcess(readFile(`/proc/${pid}/stat`)); // undefined for a zombie or a process that is gone
//! ```

/// What:
///  The clock ticks one process used,
///  or nothing when the process does not live.
/// Why:
///   The process's own file reports every thread of it,
/// and it keeps the ticks of a thread that ended,
/// so this number never goes down while the process lives
///  and a difference of two readings is a duration.
/// A process that ended is answered nothing even while its file is still there:
/// a zombie is a process that nothing has reaped yet, not one that lives,
///  and a liveness check that accepted it would accept a service that had died
///  (the review of 2026-11-15, its fifth finding: `cpu_ticks` answered `Some(0)` for a zombie).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stat = readFile(`/proc/${pid}/stat`); return stat && lives(stateOf(stat)) ? userOf(stat) + systemOf(stat) : undefined;
/// ```
pub fn cpu_ticks(pid: u32) -> Option<u64> {
    let stat: String = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // The program's name may hold spaces and brackets, and the state is the first field after it.
    let after_name: &str = stat.rsplit_once(')')?.1;
    let fields: Vec<&str> = after_name.split_whitespace().collect();
    if !lives(*fields.first()?) {
        return None;
    }
    let user: u64 = fields.get(11)?.parse().ok()?;
    let system: u64 = fields.get(12)?.parse().ok()?;
    return Some(user + system);
}

/// What:
///  Whether a state letter of `/proc/<pid>/stat` names a process that lives.
/// Why:
///   `R`, `S`, `D`, `I`, `T`, `t`, `W` and `P` are states of a process that is running, waiting or stopped,
/// and `Z` is a process that ended and waits to be reaped;
/// `X` and `x` are a process that is dead.
///  The letter is read from the same file as the times, so liveness and the reading are one moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lives = (state) => !['Z', 'X', 'x'].includes(state);
/// ```
fn lives(state: &str) -> bool {
    return !matches!(state, "Z" | "X" | "x");
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
///  This is what one moment's threads sum to, and it is not a number a difference may be taken on:
/// a thread that ends takes its ticks out of every later sum,
/// which is why `cpu_ticks` reads the process's own file instead
/// (the review of 2026-11-15, its fourth finding).
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
