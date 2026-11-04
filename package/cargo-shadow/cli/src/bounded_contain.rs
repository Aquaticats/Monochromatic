//! What:
//! Containment of everything a bounded run starts,
//! found by the process tree instead of the process group.
//! Why:
//! Round 6 (fifth review 5a F1,
//! 5c N2):
//! a descendant that calls `setsid` or `setpgid` leaves the group and may close its pipes,
//! so neither the group nor the pipes show it.
//! While the caller is a child subreaper,
//! every orphan of the run is adopted by the caller,
//! and every descendant that is
//! still alive has the caller as an ancestor,
//! so a scan of `/proc` finds all of them.
//! The limit:
//! after the caller itself died nothing adopts them,
//! so a recovery can only look at the recorded process group.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // bounded-contain.ts (Linux only)
//! export class Containment { static begin(): Containment; stopAll(direct: number | undefined, bound: number): boolean }
//! ```

/// Imports `std::collections::{HashMap, HashSet}` for this file.
use std::collections::{HashMap, HashSet};

/// Imports `std::fs` for this file.
use std::fs;

/// Imports `std::io` for this file.
use std::io;

/// Imports `std::sync::{Mutex, MutexGuard, PoisonError}` for this file.
use std::sync::{Mutex, MutexGuard, PoisonError};

/// Imports `std::thread` for this file.
use std::thread;

/// Imports `std::time::{Duration, Instant}` for this file.
use std::time::{Duration, Instant};

/// Imports `rustix::process::{Pid, Signal, WaitOptions, getpid, kill_process, set_child_subreaper, waitpid}` for this file.
use rustix::process::{Pid, Signal, WaitOptions, getpid, kill_process, set_child_subreaper, waitpid};

/// How often the process tree is scanned while it is being stopped.
/// Delegated pick of round 6,
/// open to veto:
/// the interval of the group polling it replaces (10 ms),
/// which round 5 measured
/// against the time a killed process takes to disappear (milliseconds).
const SCAN_INTERVAL: Duration = Duration::from_millis(10);

/// Whoever is stopping descendants is the only one in this process:
/// adoption cannot tell the orphans of two runs apart.
static ONE_RUN: Mutex<()> = Mutex::new(());

/// One process as `/proc` shows it.
struct Process {
    /// The process id.
    pid: i32,
    /// The parent's process id.
    parent: i32,
    /// Whether it has ended and only waits to be reaped.
    zombie: bool,
}

/// The containment of one run.
/// Holding it makes this process a child subreaper and the only run in the process.
pub(crate) struct Containment {
    /// Keeps other runs of this process out for the life of this one.
    _one_run: MutexGuard<'static, ()>,
    /// This process.
    me: i32,
    /// The children this process already had,
    /// which are not part of the run.
    preexisting: HashSet<i32>,
}

/// What:
/// `fn parse_stat(text: &str) -> Option<(i32, bool)>` reads the parent and the zombie state from `/proc/<pid>/stat`.
/// Why:
/// The command name may hold spaces and parentheses,
/// so the fields are counted from the last closing parenthesis.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseStat(text: string): [number, boolean] | undefined
/// ```
fn parse_stat(text: &str) -> Option<(i32, bool)> {
    let after: &str = text.get(text.rfind(')')? + 1..)?;
    let mut fields = after.split_whitespace();
    let state: &str = fields.next()?;
    let parent: i32 = fields.next()?.parse().ok()?;
    return Some((parent, state == "Z" || state == "X"));
}

/// What:
/// `fn processes() -> io::Result<Vec<Process>>` lists every process in `/proc`.
/// Why:
/// A process that exits during the scan is skipped;
/// any other failure is an error,
/// never "no processes".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function processes(): Process[]  // throws when /proc cannot be read
/// ```
fn processes() -> io::Result<Vec<Process>> {
    let mut found: Vec<Process> = Vec::new();
    for entry in fs::read_dir("/proc")? {
        let name: String = entry?.file_name().to_string_lossy().to_string();
        let Ok(pid) = name.parse::<i32>() else {
            continue;
        };
        let text: String = match fs::read_to_string(format!("/proc/{pid}/stat")) {
            Ok(found_text) => found_text,
            // A process that ends during the scan reads as missing (ENOENT) or as gone (ESRCH).
            Err(error) if error.kind() == io::ErrorKind::NotFound || error.raw_os_error() == Some(3) => continue,
            Err(error) => return Err(error),
        };
        let Some((parent, zombie)) = parse_stat(&text) else {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("/proc/{pid}/stat cannot be read as a process record")));
        };
        found.push(Process { pid, parent, zombie });
    }
    return Ok(found);
}

/// The methods of the type above.
impl Containment {
    /// What:
    /// `pub(crate) fn begin() -> io::Result<Containment>` makes this process a child subreaper and remembers its current children.
    /// Why:
    /// Without the attribute the orphans of a run go to init and cannot be found.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static begin(): Containment  // throws when the process cannot become a subreaper
    /// ```
    pub(crate) fn begin() -> io::Result<Containment> {
        let guard: MutexGuard<'static, ()> = ONE_RUN.lock().unwrap_or_else(PoisonError::into_inner);
        let me: Pid = getpid();
        set_child_subreaper(Some(me)).map_err(io::Error::from)?;
        let preexisting: HashSet<i32> = processes()?.into_iter().filter(|process| { return process.parent == me.as_raw_nonzero().get(); }).map(|process| { return process.pid; }).collect();
        return Ok(Containment { _one_run: guard, me: me.as_raw_nonzero().get(), preexisting });
    }

    /// What:
    /// `fn tree(&self) -> io::Result<Vec<Process>>` is every process that descends from a child this run started or adopted.
    /// Why:
    /// Roots are the children of this process that it did not have before the run;
    /// the rest follows by parent links.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// tree(): Process[]
    /// ```
    fn tree(&self) -> io::Result<Vec<Process>> {
        let all: Vec<Process> = processes()?;
        let mut children: HashMap<i32, Vec<i32>> = HashMap::new();
        for process in &all {
            children.entry(process.parent).or_default().push(process.pid);
        }
        let mut wanted: HashSet<i32> = HashSet::new();
        let mut pending: Vec<i32> = children.get(&self.me).into_iter().flatten().copied().filter(|pid| { return !self.preexisting.contains(pid); }).collect();
        while let Some(pid) = pending.pop() {
            if wanted.insert(pid) {
                pending.extend(children.get(&pid).into_iter().flatten().copied());
            }
        }
        return Ok(all.into_iter().filter(|process| { return wanted.contains(&process.pid); }).collect());
    }

    /// What:
    /// `pub(crate) fn stop_all(&self, direct: Option<i32>, bound: Duration) -> io::Result<bool>` kills every live process of the run,
    /// reaps the adopted ones,
    /// and returns `true` once a scan finds no live process left.
    /// Why:
    /// The direct child is reaped by its `Child` handle,
    /// so it is killed here but never waited for here.
    /// Killed processes cannot start others,
    /// so repeating until a scan is clean ends the race with forks.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stopAll(direct: number | undefined, bound: number): boolean
    /// ```
    pub(crate) fn stop_all(&self, direct: Option<i32>, bound: Duration) -> io::Result<bool> {
        let deadline: Instant = Instant::now() + bound;
        loop {
            let tree: Vec<Process> = self.tree()?;
            let mut alive: bool = false;
            for process in &tree {
                if !process.zombie {
                    alive = true;
                    if let Some(pid) = Pid::from_raw(process.pid) {
                        let _ = kill_process(pid, Signal::KILL);
                    }
                }
            }
            for process in &tree {
                if process.parent == self.me && Some(process.pid) != direct && let Some(pid) = Pid::from_raw(process.pid) {
                    let _ = waitpid(Some(pid), WaitOptions::NOHANG);
                }
            }
            if !alive {
                return Ok(true);
            }
            if Instant::now() >= deadline {
                return Ok(false);
            }
            thread::sleep(SCAN_INTERVAL);
        }
    }
}

/// How the containment ends.
impl Drop for Containment {
    /// What:
    /// `fn drop(&mut self)` stops this process from adopting orphans.
    /// Why:
    /// The attribute survives `exec`,
    /// and the shim may replace itself with the real command after the run.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose](): void
    /// ```
    fn drop(&mut self) {
        let _ = set_child_subreaper(None);
    }
}
