//! What:
//!  Whether any process on this machine is working in a folder,
//!  named by its process id,
//  and what the read of its working folder showed.
//! Why:
//!   A worktree whose branch has landed is removed only when nothing is working inside it,
//!  and a person's own `git worktree remove` is refused while something is:
//!  a process whose working folder is in the worktree is a person's shell,
//!  an agent's session,
//!  or a build that would fail the moment the folder went.
//!  This is a read of each process's working folder (`/proc/<pid>/cwd`),
//!  of the file it runs (`/proc/<pid>/exe`),
//!  and of the files it holds open (`/proc/<pid>/fd`),
//!  never of its command line or its environment:
//!  a command line holds arguments a person would not want copied into a message,
//!  and this program's messages are read, logged and sent to whatever started it.
//!  The program's name comes from `/proc/<pid>/comm`,
//!  which the kernel writes from the name of the running image and nothing else.
//!  A process whose working folder is elsewhere can still be working in a worktree:
//!  an editor,
//! a pager,
//! a build started with `-C` from another directory,
//!  or any program that opened a file there and went on,
//!  holds a descriptor into it,
//!  and removing the worktree pulls the files out from under it
//!  (a qwen4-max-preview audit of 0.6.1-alpha.10,
//! `574063773d`,
//! finding 2,
//! reproduced with a `tail -f` of a file in the worktree whose working folder was the fixture root).
//!  A program that runs from the worktree is working in it too:
//!  a build writes its output there and reads it back,
//!  and deleting the image under a running program ends that program.
//!  A descriptor into the worktree's own Git directory does not count:
//!  the wrapper's commit service holds its marker there for as long as it lives
//!  (measured on this host:
//! `.../<worktree>/.git/index.lock.cli-git-marker-<pid>...`),
//!  and counting it would keep every landed worktree whose service is up,
//!  which is most of them.
//!  The read costs one listing,
//!  one link read per process and one per descriptor:
//!  measured on this host with 1 567 processes,
//!  five rounds took 137.7,
//! 113.4,
//! 108.0,
//! 97.7 and 95.5 ms,
//!  so it is paid once per worktree a removal considers and nowhere else.
//!  Measured again on 2026-11-13 with the executable's own refusal,
//!  the same removal under the escape as the control,
//!  four rounds: 47,
//! 52,
//! 54 and 57 ms more for the refusal that reads the table,
//!  which is what a person waits for the words that name their process,
//!  on a host holding 1,052 processes at the time.
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
//!  The scan carries a bound (`PROCESS_SCAN_CEILING`),
//!  and a scan that reaches it says so instead of answering:
//!  nothing a bound cut short proves the worktree empty,
//!  so every caller refuses or keeps the worktree rather than guess.
//!  The descriptor record of another process is never read
//!  (`/proc/<pid>/fdinfo/<n>`),
//!  because the kernel builds one whole record before the reader gets a byte and an IDE's file watcher
//!  makes that hundreds of megabytes and minutes
//!  (issue 6287);
//!  the links this scan reads are links, and the ones it may not read are skipped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const working = processWorkingInside(worktree, gitDir, '/proc', age); // { kind: 'nobody' }
//! ```

use crate::process_use::PROCESS_SCAN_CEILING;
use std::path::Path;
use std::time::{Duration, Instant};

/// What a read of a process's working folder showed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkingFolder {
    /// It is the worktree or a folder below it.
    Inside,
    /// Its working folder is elsewhere,
    /// and it holds a file open inside the worktree.
    OpenFile,
    /// Its working folder is elsewhere,
    /// and the program it runs is inside the worktree.
    Executable,
    /// It could not be read,
    /// and the process is this account's,
    /// so nothing proves it is outside the worktree.
    Unreadable,
}

/// One process that keeps a worktree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkingProcess {
    /// Its process id.
    pub pid: u32,
    /// The name of the program it runs,
    /// as the kernel records it,
    /// or nothing when that record could not be read.
    pub program: Option<String>,
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
    ///  The program's name says which of a person's programs to stop,
    ///  and it is the kernel's own record of the running image, never a command line.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// working.why() // 'process 4242 (`sleep`) is working in it'
    /// ```
    pub fn why(&self) -> String {
        if self.folder == WorkingFolder::Inside && self.pid == std::process::id() {
            return String::from("this command is working in it, because it was started in that folder");
        }
        let named: String = match &self.program {
            Some(program) => format!("process {} (`{program}`)", self.pid),
            None => format!("process {}", self.pid),
        };
        return match self.folder {
            WorkingFolder::Inside => format!("{named} is working in it"),
            WorkingFolder::OpenFile => format!("{named} has a file open in it, though its working folder is elsewhere"),
            WorkingFolder::Executable => format!("{named} runs a program from it, though its working folder is elsewhere"),
            WorkingFolder::Unreadable => format!(
                "{named} belongs to this account, hides its working folder, and nothing shows it started before this worktree was made, so nothing proves it is outside it"
            ),
        };
    }
}

/// What:
///  What a scan of the process table found:
///  nobody,
///  one process that keeps the worktree,
///  or the reason the scan could not finish.
/// Why:
///   Three answers need three endings.
///  A scan that reached its bound saw only part of the table,
///  so it must never read as "nobody", and a caller that removed on it would take the folder an unreached
///  process is working in.
///  The reason says what the bound was and how far the scan got, so the reader knows why it refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProcessWorking = { kind: 'nobody' } | { kind: 'someone'; process: WorkingProcess } | { kind: 'unfinished'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessWorking {
    /// Nothing on this machine is working in the folder.
    Nobody,
    /// One process is.
    Someone(WorkingProcess),
    /// The scan did not finish within its bound,
    /// so nothing proves the folder is empty.
    Unfinished(String),
}

impl ProcessWorking {
    /// What:
    ///  The reason the folder is kept,
    ///  or nothing when it is empty.
    /// Why:
    ///   Every caller words the same refusal for every ending,
    ///  so the ending and its words are answered together.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// working.reason() // 'process 4242 is working in it' | undefined
    /// ```
    pub fn reason(&self) -> Option<String> {
        return match self {
            ProcessWorking::Nobody => None,
            ProcessWorking::Someone(working) => Some(working.why()),
            ProcessWorking::Unfinished(why) => Some(why.clone()),
        };
    }
}

/// What:
///  What one process table entry contributed to the scan.
/// Why:
///   A process that keeps the worktree ends the scan,
///  and a process that did not contribute leaves the scan as it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Entry = { kind: 'nobody' } | { kind: 'someone'; process: WorkingProcess };
/// ```
enum Entry {
    /// This process is not working in the folder.
    Nobody,
    /// This process is.
    Someone(WorkingProcess),
}

/// What:
///  The first process that keeps the worktree,
///  or nothing when the scan read every process and none did.
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
///  The scan carries `PROCESS_SCAN_CEILING`, and a walk that reached it says so rather than answering.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// processWorkingInside(root, procRoot, worktreeAge) // ProcessWorking
/// ```
pub fn process_working_inside(root: &Path, git_dir: &Path, proc_root: &Path, worktree_age: Option<Duration>) -> ProcessWorking {
    return process_working_inside_until(root, git_dir, proc_root, worktree_age, Instant::now() + PROCESS_SCAN_CEILING);
}

/// What:
///  `process_working_inside` with the moment the scan ends given by its caller.
/// Why:
///   A control writes a process table by hand and reads it in a moment,
///  and a table of two entries proves nothing about a bound.
///  The moment is checked before each entry,
///  so everything one process costs is bounded by the checks inside its own reads,
///  and the table as a whole is what carries the bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// processWorkingInsideUntil(root, procRoot, worktreeAge, deadline)
/// ```
pub fn process_working_inside_until(root: &Path, git_dir: &Path, proc_root: &Path, worktree_age: Option<Duration>, deadline: Instant) -> ProcessWorking {
    let listing: std::fs::ReadDir = match std::fs::read_dir(proc_root) {
        Ok(listed) => listed,
        Err(error) => return ProcessWorking::Unfinished(format!("the process table {proc_root:?} cannot be read ({error}), so nothing proves no process works there")),
    };
    let uptime: Option<Duration> = uptime_of(proc_root);
    let mut reached: usize = 0;
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let Ok(pid) = name.parse::<u32>() else {
            continue;
        };
        if Instant::now() >= deadline {
            return ProcessWorking::Unfinished(stopped_at_the_bound(reached, proc_root));
        }
        reached += 1;
        match one_process(entry.path().as_path(), pid, root, git_dir, proc_root, uptime, worktree_age) {
            Entry::Nobody => {}
            Entry::Someone(working) => return ProcessWorking::Someone(working),
        }
    }
    return ProcessWorking::Nobody;
}

/// What:
///  What one process entry contributes:
///  its working folder,
///  a file it holds open inside the worktree,
///  the program it runs,
///  or a hidden folder this account cannot read.
/// Why:
///   The three answers are read in the order that costs least and says most:
///  the working folder names the process as one working *in* the folder,
///  the descriptor and the executable say it is working there from elsewhere,
///  and a folder this account may not read is judged by its start time alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// oneProcess(entry, pid, root, gitDir, procRoot, uptime, worktreeAge) // Entry
/// ```
fn one_process(entry: &Path, pid: u32, root: &Path, git_dir: &Path, proc_root: &Path, uptime: Option<Duration>, worktree_age: Option<Duration>) -> Entry {
    match std::fs::read_link(entry.join("cwd")) {
        Ok(working) if working.starts_with(root) => return Entry::Someone(WorkingProcess { pid, program: program_of(entry), folder: WorkingFolder::Inside }),
        Ok(_elsewhere) => {
            // A folder outside the worktree can still hold a file open in it, and the program it runs can be there, and
            //  either one is a process working there.
            if holds_a_file_inside(entry, root, git_dir) {
                return Entry::Someone(WorkingProcess { pid, program: program_of(entry), folder: WorkingFolder::OpenFile });
            }
            if runs_a_program_inside(entry, root) {
                return Entry::Someone(WorkingProcess { pid, program: program_of(entry), folder: WorkingFolder::Executable });
            }
            return Entry::Nobody;
        }
        // The process went while it was being read, so it cannot be working in the worktree any more.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Entry::Nobody,
        Err(_refused) => {
            if !owned_by_this_account(entry) {
                return Entry::Nobody;
            }
            if started_before(started_ago(proc_root, pid, uptime), worktree_age) {
                return Entry::Nobody;
            }
            return Entry::Someone(WorkingProcess { pid, program: program_of(entry), folder: WorkingFolder::Unreadable });
        }
    }
}

/// What:
///  Why a scan of the process table ended at its moment rather than at the end of the table.
/// Why:
///   The processes past the stop were not looked at,
///  so the scan proves nothing about a folder they may be working in,
///  and a caller that would remove the folder refuses instead of guessing.
///  The reason names the bound, so a person reading it learns that the wait was this program's and not the disk's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stoppedAtTheBound(reached) // 'the scan of the process table stopped ...'
/// ```
fn stopped_at_the_bound(reached: usize, proc_root: &Path) -> String {
    return format!(
        "the scan of the process table {proc_root:?} stopped at its bound of {} seconds after {reached} processes, so a process it did not reach may be working there",
        PROCESS_SCAN_CEILING.as_secs()
    );
}

/// What:
///  The name of the program a process runs,
///  or nothing when the kernel's record of it cannot be read.
/// Why:
///   The name is what a person needs to know which of their programs to stop,
///  and the kernel writes it from the name of the running image,
///  never from a command line or an environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// programOf('/proc/4242') // 'sleep' | undefined
/// ```
fn program_of(entry: &Path) -> Option<String> {
    let text: String = std::fs::read_to_string(entry.join("comm")).ok()?;
    let name: &str = text.trim();
    return (!name.is_empty()).then(|| return name.to_owned());
}

/// What:
///  Whether the program the process runs is the worktree or a file below it.
/// Why:
///  A build that writes its output into the worktree and runs from it keeps reading what a removal would delete,
///  and deleting the image of a running program ends that program.
///  A link the kernel refuses to read says nothing,
///  and the process's own working folder has already answered when it could.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runsAProgramInside('/proc/4242', worktree) // boolean
/// ```
fn runs_a_program_inside(entry: &Path, worktree: &Path) -> bool {
    return match std::fs::read_link(entry.join("exe")) {
        Ok(running) => running.starts_with(worktree),
        Err(_unreadable) => false,
    };
}

/// What:
///  Whether the process at `entry` holds a file open inside the worktree,
///  its own Git directory excepted.
/// Why:
///   A descriptor is the kernel's own record of what a program is using,
///  so a file a program writes or watches there is found however that program was started.
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
        let named: &Path = without_deleted_suffix(target.as_path());
        if !named.starts_with(worktree) || named.starts_with(git_dir) {
            continue;
        }
        return true;
    }
    return false;
}

/// What:
///  The path a descriptor names,
///  without the text the kernel appends to a target whose file is already unlinked.
/// Why:
///  A program reading a file the worktree no longer holds still holds its descriptor,
///  and the file's directory is what a removal takes.
///  A target that is not text is used as it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withoutDeletedSuffix = (target) => target.replace(/ \(deleted\)$/, '');
/// ```
fn without_deleted_suffix(target: &Path) -> &Path {
    let Some(text) = target.to_str() else {
        return target;
    };
    return match text.strip_suffix(DELETED_SUFFIX) {
        Some(stripped) => Path::new(stripped),
        None => target,
    };
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
