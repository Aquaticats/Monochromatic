//! What:
//!  Whether any process on this machine is working in a folder,
//!  named by its process id,
//!  and what the read of its working folder showed.
//! Why:
//!   A worktree whose branch has landed is removed only when nothing is working inside it:
//!  a process whose working folder is in the worktree is a person's shell,
//!  an agent's session,
//!  or a build that would fail the moment the folder went.
//!  This is a read of each process's working folder (`/proc/<pid>/cwd`)
//!  and of the files it holds open (`/proc/<pid>/fd`),
//!  never of its environment.
//!  A process whose working folder is elsewhere can still be working in a worktree:
//!  an editor, a pager, a build started with `-C` from another directory,
//!  or any program that opened a file there and went on,
//!  holds a descriptor into it,
//!  and removing the worktree pulls the files out from under it
//!  (a qwen4-max-preview audit of 0.6.1-alpha.10,
//! `574063773d`,
//! finding 2,
//! reproduced with a `tail -f` of a file in the worktree whose working folder was the fixture root).
//!  A descriptor into the worktree's own Git directory does not count:
//!  the wrapper's commit service holds its marker there for as long as it lives
//!  (measured on this host:
//! `.../<worktree>/.git/index.lock.cli-git-marker-<pid>...`),
//!  and counting it would keep every landed worktree whose service is up,
//!  which is most of them.
//!  The descriptor read costs one listing and one link read per descriptor of every process:
//!  measured on this host with 1 567 processes,
//!  five rounds took 137.7, 113.4, 108.0, 97.7 and 95.5 ms,
//!  so it is paid once per worktree a merge considers removing and nowhere else.
//!  A kernel gives no reverse lookup from a directory to the processes holding it open,
//!  so a scan is what the truth costs.
//!  A process that ended while it was being read is skipped:
//!  it cannot be working there any more.
//!  A process of this account whose working folder cannot be read keeps the worktree only when it could have been there:
//!  a program can hide its working folder from every other reader
//!  (`PR_SET_DUMPABLE` 0 makes `/proc/<pid>/cwd` unreadable to the account that owns the process),
//!  and skipping it removed a worktree under a live process,
//!  which a cross-vendor review of the landed sweep reproduced
//!  with the same process keeping its worktree while the link was readable.
//!  The kernel answers nothing else about such a process,
//!  and the processes that hide their folders on a desktop are the services that start at login
//!  (measured here:
//! `systemd --user`,
//! `(sd-pam)` and four more),
//!  so keeping every worktree they might be in would remove none.
//!  A process that started before the worktree was made cannot be working inside it,
//!  which is what its start time proves:
//!  a folder can only lead into a directory that already existed.
//!  So a hidden process of this account keeps the worktree only while its age is shorter than the worktree's,
//!  or while either age cannot be read,
//!  and the reason names the process.
//!  A process of another account is skipped:
//!  its folder is unreadable to this account by design,
//!  and `git worktree remove` has never waited for one.
//!  This command's own process is read like every other one:
//!  a wrapper started with a worktree as its working folder
//!  (a program that spawns `git -C <another worktree> merge <branch>` there,
//!  which is what a tool driving this wrapper does)
//!  does not work in the worktree `-C` names,
//!  and skipping it removed the folder the command itself was started in,
//!  which a cross-vendor audit of the landed sweep reproduced:
//!  `/proc/<pid>/cwd` of the wrapper was the landed worktree,
//! and no other process was in it.
//!  The worktree then stays,
//! the notice says this command was started in it,
//!  and a person removes it when nothing there is theirs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const working = processWorkingInside(worktree, '/proc'); // undefined: nothing works there
//! ```

use std::path::Path;
use std::time::Duration;

/// What a read of a process's working folder showed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkingFolder {
    /// It is the worktree or a folder below it.
    Inside,
    /// Its working folder is elsewhere,
    /// and it holds a file open inside the worktree.
    OpenFile,
    /// It could not be read,
    /// and the process is this account's,
    /// so nothing proves it is outside the worktree.
    Unreadable,
}

/// One process that keeps a worktree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkingProcess {
    /// Its process id.
    pub pid: u32,
    /// What the read of its working folder showed.
    pub folder: WorkingFolder,
}

impl WorkingProcess {
    /// What:
    ///  The words that say why this process keeps the worktree.
    /// Why:
    ///   A person reads the reason and decides,
    ///  so it names the process and says whether its folder was seen there or could not be read at all,
    ///  and this command's own process is named as itself:
    ///  "process 4242 is working in it" about the command a person just ran says nothing to them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// working.why() // 'process 4242 is working in it'
    /// ```
    pub fn why(&self) -> String {
        if self.folder == WorkingFolder::Inside && self.pid == std::process::id() {
            return String::from("this command is working in it, because it was started in that folder");
        }
        return match self.folder {
            WorkingFolder::Inside => format!("process {} is working in it", self.pid),
            WorkingFolder::OpenFile => format!("process {} has a file open in it, though its working folder is elsewhere", self.pid),
            WorkingFolder::Unreadable => format!(
                "process {} belongs to this account, hides its working folder, and nothing shows it started before this worktree was made, so nothing proves it is outside it",
                self.pid
            ),
        };
    }
}

/// What:
///  The first process id whose working folder is `root` or below it,
///  or whose working folder this account cannot read and could lead into it.
/// Why:
///   The removal needs only one reason to keep a worktree,
///  and the question is the same for every process,
///  so the walk stops at the first answer.
///  The caller's own process is read like every other one:
///  a command started in a worktree is a process working there,
///  and `-C` names the repository rather than that folder.
///  `worktree_age` is how long `root` has existed,
///  which is what a hidden folder is judged by:
///  a process older than the folder cannot have its working folder inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// processWorkingInside(root, procRoot, worktreeAge) // WorkingProcess | undefined
/// ```
pub fn process_working_inside(root: &Path, git_dir: &Path, proc_root: &Path, worktree_age: Option<Duration>) -> Option<WorkingProcess> {
    let listing = std::fs::read_dir(proc_root).ok()?;
    let uptime: Option<Duration> = uptime_of(proc_root);
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let Ok(pid) = name.parse::<u32>() else {
            continue;
        };
        match std::fs::read_link(entry.path().join("cwd")) {
            Ok(working) if working.starts_with(root) => return Some(WorkingProcess { pid, folder: WorkingFolder::Inside }),
            Ok(_elsewhere) => {
                // A folder outside the worktree can still hold a file open in it, and that process is working there.
                if holds_a_file_inside(entry.path().as_path(), root, git_dir) {
                    return Some(WorkingProcess { pid, folder: WorkingFolder::OpenFile });
                }
            }
            // The process went while it was being read, so it cannot be working in the worktree any more.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_refused) => {
                if !owned_by_this_account(entry.path().as_path()) {
                    continue;
                }
                if started_before(started_ago(proc_root, pid, uptime), worktree_age) {
                    continue;
                }
                return Some(WorkingProcess { pid, folder: WorkingFolder::Unreadable });
            }
        }
    }
    return None;
}

/// What:
///  Whether the process at `entry` holds a file open inside the worktree,
///  its own Git directory excepted.
/// Why:
///   A descriptor is the kernel's own record of what a program is using,
///  so a file written, read or watched there is found however the program was started.
///  The worktree's Git directory is left out because this program's own commit service
///  holds its marker file there for as long as it lives,
///  and a rule that counted it would keep every landed worktree a service runs for.
///  A descriptor directory this account cannot read says nothing,
///  and its process's working folder could not be read either,
///  which the caller has already decided by the process's age.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdsAFileInside(entry, worktree, gitDir) // boolean
/// ```
fn holds_a_file_inside(entry: &Path, worktree: &Path, git_dir: &Path) -> bool {
    let Ok(descriptors) = std::fs::read_dir(entry.join("fd")) else {
        return false;
    };
    for descriptor in descriptors.flatten() {
        let Ok(target) = std::fs::read_link(descriptor.path()) else {
            continue;
        };
        // The kernel appends this to a path whose file is already deleted, and the file itself is what a removal takes.
        let named: &Path = match target.to_str().and_then(|text| { return text.strip_suffix(DELETED_SUFFIX); }) {
            Some(stripped) => Path::new(stripped),
            None => target.as_path(),
        };
        if !named.starts_with(worktree) || named.starts_with(git_dir) {
            continue;
        }
        return true;
    }
    return false;
}

/// The text the kernel appends to the target of a descriptor whose file is already unlinked.
const DELETED_SUFFIX: &str = " (deleted)";

/// What:
///  Whether a process that started this long ago cannot be working in a folder this young.
/// Why:
///   A working folder can only be a directory that existed when the process changed into it,
///  so a process older than the worktree is outside it,
///  and one younger may be inside,
/// as may one whose age cannot be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// startedBefore(processAge, worktreeAge) // boolean
/// ```
fn started_before(process_age: Option<Duration>, worktree_age: Option<Duration>) -> bool {
    // `match` on both ages: only two ages can decide it, and every other pair keeps the worktree.
    let (Some(aged_process), Some(aged_folder)) = (process_age, worktree_age) else {
        return false;
    };
    return aged_process > aged_folder;
}

/// The first field of the process table's `uptime`,
/// which is how long this machine has been up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// uptimeOf(procRoot) // Duration | undefined
/// ```
fn uptime_of(proc_root: &Path) -> Option<Duration> {
    let text: String = std::fs::read_to_string(proc_root.join("uptime")).ok()?;
    let seconds: f64 = text.split_whitespace().next()?.parse::<f64>().ok()?;
    return Duration::try_from_secs_f64(seconds).ok();
}

/// The clock ticks one second holds,
/// which is how `stat` counts a process's start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ticksPerSecond() // number
/// ```
#[cfg(unix)]
fn ticks_per_second() -> f64 {
    // SAFETY: `sysconf` reads one system constant and touches no memory of this process.
    let ticks: libc::c_long = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    return if ticks > 0 { f64::from(i32::try_from(ticks).unwrap_or(100)) } else { 100.0 };
}

/// No clock ticks to read on a platform without Unix processes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ticksPerSecond() // 100
/// ```
#[cfg(not(unix))]
fn ticks_per_second() -> f64 {
    return 100.0;
}

/// What:
///  How long ago one process started,
///  or nothing when the table does not say.
/// Why:
///   The start is the only thing the kernel still answers about a process that hides its working folder,
///  and this is the one reader of it (`/proc/<pid>/stat`,
/// field 22,
/// counted from this machine's boot).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// startedAgo(procRoot, pid, uptime) // Duration | undefined
/// ```
fn started_ago(proc_root: &Path, pid: u32, uptime: Option<Duration>) -> Option<Duration> {
    let up: Duration = uptime?;
    let stat: String = std::fs::read_to_string(proc_root.join(pid.to_string()).join("stat")).ok()?;
    // The second field is the program's name in parentheses and may hold spaces and parentheses, so the fields after it
    // are counted from the last closing parenthesis: the start time is the twentieth of those.
    let after_name: &str = stat.rsplit_once(')')?.1;
    let start_ticks: f64 = after_name.split_whitespace().nth(19)?.parse::<f64>().ok()?;
    let started: Duration = Duration::try_from_secs_f64(start_ticks / ticks_per_second()).ok()?;
    return Some(up.saturating_sub(started));
}

/// What:
///  Whether the process entry belongs to the account this program runs as.
/// Why:
///   A folder only this account's processes hide is this account's to protect,
///  and another account's process is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ownedByThisAccount('/proc/4242') // boolean
/// ```
#[cfg(unix)]
fn owned_by_this_account(entry: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    let Ok(theirs) = std::fs::metadata(entry) else {
        return false;
    };
    let Ok(ours) = std::fs::metadata(Path::new("/proc/self")) else {
        return false;
    };
    return theirs.uid() == ours.uid();
}

// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
/// No account of its own on a platform without Unix users,
/// so a folder that cannot be read is read as another program's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ownedByThisAccount(entry) // false
/// ```
#[cfg(not(unix))]
fn owned_by_this_account(_entry: &Path) -> bool {
    return false;
}

/// Process look controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_process_inside_tests.rs"]
mod tests;
