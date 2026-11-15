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
//!  A descriptor into the worktree's own Git directory does not count:
//!  the wrapper's commit service holds its marker there for as long as it lives
//!  (measured on this host:
//! `.../<worktree>/.git/index.lock.cli-git-marker-<pid>...`),
//!  and counting it would keep every landed worktree whose service is up,
//!  which is most of them.
//!  The descriptor read costs one listing and one link read per descriptor of every process:
//!  measured on this host with 1 567 processes,
//!  five rounds took 137.7,
//! 113.4,
//! 108.0,
//! 97.7 and 95.5 ms,
//!  so it is paid once per worktree a merge considers removing and nowhere else.
//!  A kernel gives no reverse lookup from a directory to the processes holding it open,
//!  so a scan is what the truth costs.
//!  A process that ended while it was being read is skipped:
//!  it cannot be working there any more.
//!  A process of this account whose working folder cannot be read keeps the worktree
//!  when nothing shows it started before the worktree was made,
//!  because a program can hide its working folder from every other reader
//!  (`PR_SET_DUMPABLE` 0 makes `/proc/<pid>/cwd` unreadable to the account that owns the process),
//!  and skipping it removed a worktree under a live process,
//!  which a cross-vendor review of the landed sweep reproduced
//!  with the same process keeping its worktree while the link was readable.
//!  The kernel answers nothing else about such a process,
//!  and the processes that hide their folders on a desktop are the services that start at login
//!  (measured here:
//! `systemd --user`,
//! `(sd-pam)` and four more),
//!  so an answer that kept every worktree a hidden process might be in would remove none.
//!  What decides instead is the process's start:
//!  a process of this account whose working folder cannot be read is taken to be working inside the worktree
//!  when it started after the worktree was made,
//!  when its age and the worktree's own are equal,
//!  when its start cannot be read,
//!  or when the age of the worktree cannot be read,
//!  and taken to be outside when it started before the worktree was made,
//!  and the reason names the process when the answer is that it may be inside.
//!  That is a guess at which processes change into a worktree,
//!  not a proof:
//!  a process that started before the worktree was made can change into it afterwards,
//!  and one that does is the miss this rule leaves.
//!  Whether that miss is accepted is an open question for the owner
//!  (`doc/handover/cli-git-rust-implementation.md`,
//! `Questions kept for the owner's return`,
//!  the item added 2026-11-15T03:06:17-05:00).
//!  A refused read of the files a process holds open is never none either:
//!  a program can keep its working folder readable and stop every other reader of its descriptor table
//!  (measured on this host with a password manager that runs from login:
//! its working folder read as the account's home folder,
//! and its descriptor table refused with `EACCES`),
//!  so a file it holds in a worktree would be invisible.
//!  The same ages decide that answer as decide a working folder that cannot be read:
//!  the process is taken to be working inside when it is younger than the worktree,
//!  when its age and the worktree's own are equal,
//!  when its age cannot be read,
//!  or when the age of the worktree cannot be read,
//!  and taken to be outside when it is older.
//!  The miss is larger there than it is for a working folder,
//!  because a program can open a file in a worktree long after it started:
//!  an older program that opens one after the worktree was made is taken to be outside all the same.
//!  The rule is as it is because without it every walk of this host stops at that password manager,
//!  which keeps every worktree and removes none,
//!  and whether its miss is accepted is the same open question for the owner as the folder's.
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

/// What the reads of one process showed:
/// where it works,
/// and what it holds open.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkingFolder {
    /// It is the worktree or a folder below it.
    Inside,
    /// Its working folder is elsewhere,
    /// and it holds a file open inside the worktree.
    OpenFile,
    /// Its working folder could not be read,
    /// and the process is this account's,
    /// so nothing proves it is outside the worktree.
    Unreadable,
    /// Its working folder is elsewhere,
    /// and the files it holds open could not be read,
    /// and the process is this account's,
    /// so nothing proves it holds none inside the worktree.
    OpenFilesUnreadable,
}

/// One process that keeps a worktree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkingProcess {
    /// Its process id.
    pub pid: u32,
    /// What the reads of this process showed.
    pub folder: WorkingFolder,
}

impl WorkingProcess {
    /// What:
    ///  The words that say why this process keeps the worktree.
    /// Why:
    ///   A person reads the reason and decides,
    ///  so it names the process and says whether its folder or the files it holds could not be read at all,
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
            WorkingFolder::OpenFilesUnreadable => format!(
                "process {} belongs to this account, works elsewhere, and the files it holds open could not be read, and nothing shows it started before this worktree was made, so nothing proves it holds none in it",
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
///  which is what a process this account could not see into is judged by:
///  one older than the folder is taken to be outside it,
///  though one can change into it or open a file in it after the folder was made,
///  which is the miss the module doc names.
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
                match open_files_in(entry.path().as_path(), root, git_dir) {
                    OpenFiles::Inside => return Some(WorkingProcess { pid, folder: WorkingFolder::OpenFile }),
                    OpenFiles::Outside => {}
                    // A descriptor table this account may not read hides what the process holds, so the age of the
                    // process decides it as it decides a hidden working folder: one younger than the worktree keeps
                    // it, as do equal ages and an age that cannot be read, one older is taken to hold nothing there,
                    // and an older one that opened a file there after the worktree was made is the miss the module
                    // doc names.
                    OpenFiles::Unreadable => {
                        if could_be_working_inside(entry.path().as_path(), proc_root, pid, uptime, worktree_age) {
                            return Some(WorkingProcess { pid, folder: WorkingFolder::OpenFilesUnreadable });
                        }
                    }
                }
            }
            // The process went while it was being read, so it cannot be working in the worktree any more.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_refused) => {
                if could_be_working_inside(entry.path().as_path(), proc_root, pid, uptime, worktree_age) {
                    return Some(WorkingProcess { pid, folder: WorkingFolder::Unreadable });
                }
            }
        }
    }
    return None;
}

/// What a read of one process's open files showed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OpenFiles {
    /// A descriptor names a file of the worktree.
    Inside,
    /// Every descriptor was read and none names one.
    Outside,
    /// The descriptor table,
    /// or one descriptor's target,
    /// could not be read,
    /// so nothing proves what it holds.
    Unreadable,
}

/// What:
///  Whether a process this account could not see into may still be working in the worktree.
/// Why:
///   Two reads can be refused for the same reason,
///  a working folder and a descriptor table,
///  and both are judged by the same two facts:
///  another account's process is not this account's to wait for,
///  and a process that started before the worktree was made is taken to be outside it,
///  though one can change into it or open a file in it afterwards,
///  which is the miss the module doc names.
///  An age that cannot be read excuses nothing,
///  so the worktree stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// couldBeWorkingInside(entry, procRoot, pid, uptime, worktreeAge) // boolean
/// ```
fn could_be_working_inside(entry: &Path, proc_root: &Path, pid: u32, uptime: Option<Duration>, worktree_age: Option<Duration>) -> bool {
    if !owned_by_this_account(entry) {
        return false;
    }
    return !started_before(started_ago(proc_root, pid, uptime), worktree_age);
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
///  A process that ended while it was being read is not inside:
///  the kernel closed its descriptors with it,
///  and a descriptor that went between the listing and its link is no descriptor any more.
///  A table or a link that could not be read for another reason says nothing,
///  and none is never the answer to a read that did not happen:
///  a program can keep its working folder readable and stop every other reader of its descriptors
///  (measured on this host:
/// a password manager that runs from login),
///  and a refused read answering none takes the worktree out from under a file such a program holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// openFilesIn(entry, worktree, gitDir) // 'inside' | 'outside' | 'unreadable'
/// ```
fn open_files_in(entry: &Path, worktree: &Path, git_dir: &Path) -> OpenFiles {
    let descriptors: std::fs::ReadDir = match std::fs::read_dir(entry.join("fd")) {
        Ok(found) => found,
        // The process went while it was being read, and its descriptors went with it.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return OpenFiles::Outside,
        Err(_refused) => return OpenFiles::Unreadable,
    };
    for listed in descriptors {
        let descriptor: std::fs::DirEntry = match listed {
            Ok(one) => one,
            Err(_unread) => return OpenFiles::Unreadable,
        };
        let target: std::path::PathBuf = match std::fs::read_link(descriptor.path()) {
            Ok(found) => found,
            // The descriptor closed between the listing and the read, so it holds no file now.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_refused) => return OpenFiles::Unreadable,
        };
        // The kernel appends this to a path whose file is already deleted, and the file itself is what a removal takes.
        let named: &Path = without_deleted_suffix(target.as_path());
        if named.starts_with(worktree) && !named.starts_with(git_dir) {
            return OpenFiles::Inside;
        }
    }
    return OpenFiles::Outside;
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
///  Whether a process that started this long ago is taken to be outside a folder this young.
/// Why:
///   A working folder is a directory the process changed into,
///  and a process that started before the worktree was made can change into it afterwards,
///  so this answers by a guess at which processes do that and not by a proof:
///  what it assumes is that a process works in the folder it started in.
///  A process younger than the worktree may be inside,
///  as may one whose age cannot be read,
///  and one that started before and came in anyway is the miss the module doc names.
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
