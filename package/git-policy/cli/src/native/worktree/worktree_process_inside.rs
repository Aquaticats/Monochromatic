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
//!  A table that could not be read is never a table with nothing in it:
//!  a removal rests on a proof that no process works in the worktree,
//!  and a read that did not happen proves nothing,
//!  so the worktree stays and the reason names the table and the operating system's own answer.
//!  There are two such reads,
//!  and both were taken for an answer that nothing works there
//!  (found by reading, then reproduced through the real removal):
//!  the listing of the table's own directory,
//!  which fails when the table is not there,
//!  is not a directory,
//!  or may not be listed,
//!  and a listing that stops answering part way,
//!  whose entries nobody read are processes nobody looked at.
//!  A removal that could not look keeps the worktree and says so in one line,
//!  naming the table,
//!  what the operating system refused,
//!  and the command a person runs when nothing there is theirs
//!  (`worktree_landed_sweep_notice.rs`,
//!  `worktree_kept_notices.rs`);
//!  nobody is asked to prove anything a read of the table did not.
//!  A table read to its end is still no proof when it is not this machine's own view:
//!  one of another PID namespace lists only that namespace's processes,
//!  and one under a mount that hides other accounts' processes lists none of them,
//!  so both answer that the look could not be made
//!  (an independent review of 2026-11-16 removed a landed worktree through the built wrapper
//!  with a shell outside the namespace working in it).
//!  A process's own entry answers `NotFound` for its working folder once its first thread has ended,
//!  while its other threads still work in the folder,
//!  so that answer is settled by the threads the kernel still lists
//!  and only a process whose every thread has ended is taken to have gone.
//!  A descriptor is not everything a program uses:
//!  a file mapped into memory outlives the descriptor it was made from,
//!  so the map table is read as well,
//!  and one of the worktree's files there keeps it.
//!  A bind mount is a second name for one folder that no canonical path resolves,
//!  so the folder is compared by every path this machine's mount table reaches it by,
//!  and not by the text of one path alone.
//!  A `stat` that fails says nothing about which account owns a process:
//!  one that cannot be read is judged like this account's and says so,
//!  rather than being taken for another account's and skipped.
//!  Two readings of a start and of a birth cannot be ordered when they are closer than the granularity
//!  they were read with,
//!  so a process younger than the worktree is never taken as older than it.
//!  Each of these was a way the read answered that nothing works there while a live process worked in it,
//!  and each has a control that failed before the answer was written.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const look = processWorkingInside(worktree, '/proc'); // { working } | 'nothingWorksThere' | { cannotLook: reason }
//! ```

use std::path::{Path, PathBuf};
use std::time::Duration;

/// What the reads of one process showed:
/// where it works,
/// and what it holds open or has mapped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkingFolder {
    /// It is the worktree or a folder below it.
    Inside,
    /// Its working folder is elsewhere,
    /// and it holds a file open inside the worktree.
    OpenFile,
    /// Its working folder is elsewhere,
    /// and it has a file of the worktree mapped into memory.
    MappedFile,
    /// Its working folder could not be read,
    /// and the process is this account's,
    /// so nothing proves it is outside the worktree.
    Unreadable,
    /// Its working folder is elsewhere,
    /// and the files it holds open could not be read,
    /// and the process is this account's,
    /// so nothing proves it holds none inside the worktree.
    OpenFilesUnreadable,
    /// What the process does could not be read,
    /// and nothing proves which account it belongs to,
    /// so nothing proves it is outside the worktree.
    AccountUnknown,
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
            WorkingFolder::MappedFile => format!("process {} has a file of it mapped into memory, though its working folder is elsewhere", self.pid),
            WorkingFolder::Unreadable => format!(
                "process {} belongs to this account, hides its working folder, and nothing shows it started before this worktree was made, so nothing proves it is outside it",
                self.pid
            ),
            WorkingFolder::OpenFilesUnreadable => format!(
                "process {} belongs to this account, works elsewhere, and the files it holds open could not be read, and nothing shows it started before this worktree was made, so nothing proves it holds none in it",
                self.pid
            ),
            WorkingFolder::AccountUnknown => format!(
                "process {} hides what it is doing and nothing proves which account it belongs to, and nothing shows it started before this worktree was made, so nothing proves it is outside it",
                self.pid
            ),
        };
    }
}

/// What the read of a process table answered.
/// Why:
///  A removal that destroys a folder needs a proof that nothing works inside it,
///  and a read that could not be made is not that proof:
///  answering "nothing works there" for a table nobody could read
///  took a worktree out from under a process that was working in it.
///  The three answers are kept apart for that reason,
///  and `CannotLook` carries why in the words a person reads.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessLook {
    /// One process is working in the folder,
    /// and the reason names it.
    Working(WorkingProcess),
    /// Every process the table lists was read,
    /// and none of them is working in the folder.
    NothingWorksThere,
    /// A read the answer rests on could not be made,
    /// so nothing proves no process is working in the folder:
    /// why,
    /// as the sentence a person reads beside the worktree that stays.
    CannotLook(String),
}

/// What:
///  The entry a process file system makes for whatever reads it,
///  which is the one entry a table of another PID namespace numbers differently.
/// Why:
///   A process knows the number its own namespace gave it,
///  and a table of another namespace answers that entry with the number the table's namespace gave it,
///  so the two numbers tell the two views apart without a guess.
const PROCESS_TABLE_SELF: &str = "/proc/self";

/// The start of the mount option that hides other accounts' processes from this account.
const HIDEPID_OPTION: &str = "hidepid=";

/// The values of `hidepid` that hide nothing.
const HIDEPID_OFF: [&str; 2] = ["0", "off"];

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
///  one older than the folder is taken to hold nothing there,
///  though one can change into it or open a file in it after the folder was made,
///  which is the miss the module doc names.
///  A table that cannot be listed,
///  or one whose listing stops answering part way,
///  answers that the look could not be made,
///  never that nothing works there,
///  and so does a table that is another PID namespace's view of the machine,
///  or one whose mount hides other accounts' processes:
///  none of them lists every process the machine has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// processWorkingInside(root, procRoot, worktreeAge) // { working } | 'nothingWorksThere' | { cannotLook: reason }
/// ```
pub fn process_working_inside(root: &Path, git_dir: &Path, proc_root: &Path, worktree_age: Option<Duration>) -> ProcessLook {
    let listing: std::fs::ReadDir = match std::fs::read_dir(proc_root) {
        Ok(found) => found,
        // A table nobody could list hides every process it holds, and a removal that read this refusal as a table with
        // nothing in it took a worktree out from under a process working in it. A table that cannot be listed cannot be
        // asked which view of the machine it is either, so this is the first question.
        Err(refused) => {
            return ProcessLook::CannotLook(format!("the process table {} could not be listed ({refused})", proc_root.display()));
        }
    };
    if let Err(reason) = this_views_the_machine(proc_root) {
        return ProcessLook::CannotLook(reason);
    }
    return look_at_each(
        listing.map(|listed| { return listed.map(|entry: std::fs::DirEntry| { return entry.path(); }); }),
        root,
        git_dir,
        proc_root,
        worktree_age,
    );
}

/// What:
///  Whether the table `proc_root` is this process's own view of every process the machine has,
///  and why not when it is not.
/// Why:
///  A table of another PID namespace lists only that namespace's processes,
///  and it was read to its end and answered that nothing works in a worktree
///  while a shell outside the namespace worked in it
///  (an independent review of 2026-11-16 through the built wrapper).
///  The kernel answers `/proc/self/ns/pid` with the namespace this process runs in,
///  and a namespace that is not the initial one (`stale_ref_lock_judge_environment.rs`
///  `INITIAL_PID_NAMESPACE`) holds a part of the machine's processes and not all of them,
///  whatever process file system is mounted for it.
///  A table that names no namespace for its reader is a control's own table and never a running machine's.
///  A mount that hides other accounts' processes lists none of them either,
///  so a table under one is no proof about the machine
///  (the same question `lock/stale_ref_lock_judge_environment.rs` `proc_hides_no_process` asks).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// thisViewsTheMachine('/proc') // void, or the reason it does not
/// ```
fn this_views_the_machine(proc_root: &Path) -> Result<(), String> {
    let scope: PathBuf = proc_root.join("self").join("ns").join("pid");
    match std::fs::read_link(scope.as_path()) {
        // A table that names no namespace for its reader is a control's own table, never a running machine's.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!("nothing shows which PID namespace the process table {} is read from ({error})", proc_root.display()));
        }
        Ok(named) => {
            let namespace: String = named.to_string_lossy().into_owned();
            if namespace != crate::stale_ref_lock_judge_environment::INITIAL_PID_NAMESPACE {
                return Err(format!(
                    "the process table {} is not this machine's own view of its processes: this process reads it from the PID namespace {namespace}, which is not the machine's own, so a table read to its end lists only that namespace's processes",
                    proc_root.display()
                ));
            }
        }
    }
    let facts: crate::fs_mount_options::MountFacts = match crate::fs_mount_options::read_mount_facts(proc_root) {
        Ok(Some(found)) => found,
        Ok(None) => {
            return Err(format!("no mount of this machine holds the process table {}, so nothing shows what it lists", proc_root.display()));
        }
        Err(reason) => {
            return Err(format!("nothing shows whether the process table {} lists every process: {reason}", proc_root.display()));
        }
    };
    for option in facts.mount_options.iter().chain(facts.super_options.iter()) {
        if let Some(value) = option.strip_prefix(HIDEPID_OPTION)
            && !HIDEPID_OFF.contains(&value)
        {
            return Err(format!(
                "the mount of the process table {} hides other accounts' processes, so a table read to its end names none of them",
                proc_root.display()
            ));
        }
    }
    return Ok(());
}

/// The folder a removal is about,
/// as the paths this machine reaches it by.
struct Inside<'a> {
    /// The worktree as the caller names it.
    root: &'a Path,
    /// The worktree's own Git directory, whose own files never count on their own.
    git_dir: &'a Path,
    /// The other paths this machine reaches the worktree's folder by.
    spellings: Vec<PathBuf>,
    /// The same for the Git directory.
    git_dir_spellings: Vec<PathBuf>,
}

impl<'a> Inside<'a> {
    /// What:
    ///  The folder with every path this machine reaches it by,
    ///  read from the machine's mount table.
    /// Why:
    ///  A bind mount is a second name for one folder that no canonical path resolves,
    ///  so comparing the text of paths alone read a process working there as elsewhere,
    ///  and a removal took the worktree with the files that process had open
    ///  (reproduced by this branch's own control, in a private mount namespace).
    ///  A mount table that cannot be read says nothing about the paths,
    ///  so nothing is taken for outside it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// Inside.of('/x/repo/wt', '/x/repo/.git') // the folder with the paths its mounts add
    /// ```
    fn of(root: &'a Path, git_dir: &'a Path) -> Result<Inside<'a>, String> {
        let table: Vec<crate::fs_mount_options::MountFacts> = mount_table()?;
        return Ok(Inside { root, git_dir, spellings: other_spellings(table.as_slice(), root), git_dir_spellings: other_spellings(table.as_slice(), git_dir) });
    }

    /// Whether `at` is the worktree or a folder below it,
    /// by any path this machine reaches it by.
    fn holds(&self, at: &Path) -> bool {
        return at.starts_with(self.root) || self.spellings.iter().any(|one| { return at.starts_with(one); });
    }

    /// Whether `at` is the Git directory or a folder below it,
    /// by any path this machine reaches it by.
    fn is_the_git_dir(&self, at: &Path) -> bool {
        return at.starts_with(self.git_dir) || self.git_dir_spellings.iter().any(|one| { return at.starts_with(one); });
    }
}

/// What:
///  This machine's mount table,
///  or why it could not be read.
/// Why:
///  The table is the only place the machine says that two paths stand at one device and inode,
///  so a read that could not have it knows no second spelling of any folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mountTable() // MountFacts[], or the reason
/// ```
fn mount_table() -> Result<Vec<crate::fs_mount_options::MountFacts>, String> {
    let text: Vec<u8> = match crate::plain_file::read_plain_file(Path::new(crate::fs_mount_options::MOUNTINFO_PATH), crate::plain_file::MOUNT_TABLE_LIMIT) {
        Ok(found) => found,
        Err(error) => {
            return Err(format!(
                "nothing shows which paths this machine reaches the folder by: {} could not be read ({error})",
                crate::fs_mount_options::MOUNTINFO_PATH
            ));
        }
    };
    return Ok(crate::fs_mount_options::parse_mount_table(text.as_slice()));
}

/// What:
///  The paths this machine reaches `folder` by besides the one the caller named.
/// Why:
///  A mount point whose own folder is `folder`, or a folder above it, is another name for it:
///  the folder's device and inode are the kernel's answer to which folder this is,
///  and every mount point that answers the same is a spelling of the same folder.
///  A spelling is the mount point with the part of `folder` that lies below it appended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// otherSpellings(table, '/x/repo/wt') // ['/alias/wt'] for a bind mount of /x/repo at /alias
/// ```
fn other_spellings(table: &[crate::fs_mount_options::MountFacts], folder: &Path) -> Vec<PathBuf> {
    let mut spellings: Vec<PathBuf> = Vec::new();
    for above in folder.ancestors() {
        let Some(here) = identity_of(above) else {
            continue;
        };
        for mount in table {
            if mount.mount_point == above || identity_of(mount.mount_point.as_path()) != Some(here) {
                continue;
            }
            let Ok(below) = folder.strip_prefix(above) else {
                continue;
            };
            spellings.push(mount.mount_point.join(below));
        }
    }
    return spellings;
}

/// The device and inode of a path,
/// which is the kernel's answer to which folder or file this is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identityOf('/x/repo') // [device, inode] | undefined
/// ```
#[cfg(unix)]
fn identity_of(at: &Path) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt as _;
    let found = std::fs::metadata(at).ok()?;
    return Some((found.dev(), found.ino()));
}

// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
/// No device or inode to read on a platform without Unix files,
/// so no path is proven to be another's spelling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identityOf(at) // undefined
/// ```
#[cfg(not(unix))]
fn identity_of(_at: &Path) -> Option<(u64, u64)> {
    return None;
}

/// What:
///  The answer one listing of a process table gives.
/// Why:
///  The walk is written over the listing rather than over `read_dir` so that a control can name a table that stops
///  answering part way,
///  which no directory of a fixture can make a real listing do:
///  production names the table's own listing,
///  and every item of it is a process,
///  or the read that would have named one failed,
///  and a failed item hides the processes it never named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const listed of listing) { if (listed.error) return cannotLook(listed.error); ... }
/// ```
fn look_at_each<Listed>(listed: Listed, root: &Path, git_dir: &Path, proc_root: &Path, worktree_age: Option<Duration>) -> ProcessLook
where
    Listed: Iterator<Item = std::io::Result<PathBuf>>,
{
    let inside: Inside<'_> = match Inside::of(root, git_dir) {
        Ok(found) => found,
        Err(reason) => return ProcessLook::CannotLook(reason),
    };
    let uptime: Option<Duration> = uptime_of(proc_root);
    for one in listed {
        let entry: PathBuf = match one {
            Ok(found) => found,
            // The listing stopped before its end, so the processes it had not named were never looked at.
            Err(stopped) => {
                return ProcessLook::CannotLook(format!("the process table {} stopped answering before its end ({stopped})", proc_root.display()));
            }
        };
        let name: String = entry.file_name().map(|name| { return name.to_string_lossy().into_owned(); }).unwrap_or_default();
        let Ok(pid) = name.parse::<u32>() else {
            continue;
        };
        match std::fs::read_link(entry.join("cwd")) {
            Ok(working) if inside.holds(working.as_path()) => return ProcessLook::Working(WorkingProcess { pid, folder: WorkingFolder::Inside }),
            Ok(_elsewhere) => {
                // A folder outside the worktree can still hold a file open in it, or have one of its files mapped, and that
                // process is working there.
                match files_in(entry.join("fd").as_path(), &inside) {
                    OpenFiles::Inside => return ProcessLook::Working(WorkingProcess { pid, folder: WorkingFolder::OpenFile }),
                    OpenFiles::Unreadable => {
                        if let Some(folder) = may_work_there(entry.as_path(), proc_root, pid, uptime, worktree_age, WhatWasUnreadable::Files) {
                            return ProcessLook::Working(WorkingProcess { pid, folder });
                        }
                    }
                    OpenFiles::Outside => match mapped_in(entry.join("maps").as_path(), &inside) {
                        OpenFiles::Inside => return ProcessLook::Working(WorkingProcess { pid, folder: WorkingFolder::MappedFile }),
                        OpenFiles::Outside => {}
                        // A map that could not be read hides what the process uses, so the age of the process decides it as
                        // it decides a descriptor table that could not be read.
                        OpenFiles::Unreadable => {
                            if let Some(folder) = may_work_there(entry.as_path(), proc_root, pid, uptime, worktree_age, WhatWasUnreadable::Mapped) {
                                return ProcessLook::Working(WorkingProcess { pid, folder });
                            }
                        }
                    },
                }
            }
            // A process's first thread can end while another works on, and the kernel then answers this entry's working
            // folder with `NotFound`: the process went only when no thread of it is left.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => match threads_of(entry.as_path(), &inside) {
                Threads::Inside(folder) => return ProcessLook::Working(WorkingProcess { pid, folder }),
                Threads::Outside => {}
                Threads::Unreadable => {
                    if let Some(folder) = may_work_there(entry.as_path(), proc_root, pid, uptime, worktree_age, WhatWasUnreadable::Folder) {
                        return ProcessLook::Working(WorkingProcess { pid, folder });
                    }
                }
            },
            Err(_refused) => {
                if let Some(folder) = may_work_there(entry.as_path(), proc_root, pid, uptime, worktree_age, WhatWasUnreadable::Folder) {
                    return ProcessLook::Working(WorkingProcess { pid, folder });
                }
            }
        }
    }
    return ProcessLook::NothingWorksThere;
}

/// What the threads of a process showed,
/// when its own entry's working folder could not be read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Threads {
    /// One thread works in the worktree, or holds a file of it, or has one mapped.
    Inside(WorkingFolder),
    /// Every thread was read and none is working in the worktree.
    Outside,
    /// A thread,
    /// or the list of them,
    /// could not be read,
    /// so nothing proves what they do.
    Unreadable,
}

/// What:
///  What the threads of a process do,
///  for a process whose own entry answered `NotFound` for its working folder.
/// Why:
///  A process whose first thread has ended keeps its entry and its other threads,
///  and `/proc/<pid>/task/<thread>/cwd` still names the folder every one of them works in,
///  while `/proc/<pid>/cwd` and `/proc/<pid>/fd` answer `NotFound` and nothing
///  (measured on this host with a helper that ended its first thread alone).
///  A thread whose own entry answers `NotFound` has ended, and a process whose every thread has ended went.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// threadsOf('/proc/4242', inside) // 'inside' | 'outside' | 'unreadable'
/// ```
fn threads_of(entry: &Path, inside: &Inside<'_>) -> Threads {
    let tasks: std::fs::ReadDir = match std::fs::read_dir(entry.join("task")) {
        Ok(found) => found,
        // The process's own entry no longer names a thread, so it went.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Threads::Outside,
        Err(_refused) => return Threads::Unreadable,
    };
    for listed in tasks {
        let task: PathBuf = match listed {
            Ok(one) => one.path(),
            Err(_unread) => return Threads::Unreadable,
        };
        match std::fs::read_link(task.join("cwd")) {
            Ok(working) if inside.holds(working.as_path()) => return Threads::Inside(WorkingFolder::Inside),
            Ok(_elsewhere) => match files_in(task.join("fd").as_path(), inside) {
                OpenFiles::Inside => return Threads::Inside(WorkingFolder::OpenFile),
                OpenFiles::Unreadable => return Threads::Unreadable,
                OpenFiles::Outside => match mapped_in(task.join("maps").as_path(), inside) {
                    OpenFiles::Inside => return Threads::Inside(WorkingFolder::MappedFile),
                    OpenFiles::Unreadable => return Threads::Unreadable,
                    OpenFiles::Outside => {}
                },
            },
            // This thread has ended, which is what the process's own entry answered for its first one.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_refused) => return Threads::Unreadable,
        }
    }
    return Threads::Outside;
}

/// What a read of the files of one process showed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OpenFiles {
    /// One entry names a file of the worktree.
    Inside,
    /// Every entry was read and none names one.
    Outside,
    /// The list,
    /// or one entry's target,
    /// could not be read,
    /// so nothing proves what it holds.
    Unreadable,
}

/// What:
///  Whether the files one process holds open,
///  named by their directory of links,
///  include one of the worktree,
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
/// filesIn('<entry>/fd', inside) // 'inside' | 'outside' | 'unreadable'
/// ```
fn files_in(descriptors: &Path, inside: &Inside<'_>) -> OpenFiles {
    let listed: std::fs::ReadDir = match std::fs::read_dir(descriptors) {
        Ok(found) => found,
        // The process went while it was being read, and its descriptors went with it.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return OpenFiles::Outside,
        Err(_refused) => return OpenFiles::Unreadable,
    };
    for one in listed {
        let descriptor: std::fs::DirEntry = match one {
            Ok(found) => found,
            Err(_unread) => return OpenFiles::Unreadable,
        };
        let target: PathBuf = match std::fs::read_link(descriptor.path()) {
            Ok(found) => found,
            // The descriptor closed between the listing and the read, so it holds no file now.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_refused) => return OpenFiles::Unreadable,
        };
        // The kernel appends this to a path whose file is already deleted, and the file itself is what a removal takes.
        let named: &Path = without_deleted_suffix(target.as_path());
        if inside.holds(named) && !inside.is_the_git_dir(named) {
            return OpenFiles::Inside;
        }
    }
    return OpenFiles::Outside;
}

/// What:
///  Whether the files one process has mapped into memory,
///  named by its map table,
///  include one of the worktree.
/// Why:
///   A mapping outlives the descriptor it was made from,
///  so a program that maps a file and closes it still reads that file,
///  and a shared mapping still writes it,
///  while a read of the descriptors alone answers that it holds nothing
///  (reproduced by this branch's own control, whose probe maps a file of the worktree and closes it).
///  The kernel names a mapped file in `/proc/<pid>/maps`,
///  which is the only record of the mapping it keeps.
///  A map table that cannot be read says nothing, and an entry that went between the reading of a line and its end
///  is no mapping any more.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mappedIn('<entry>/maps', inside) // 'inside' | 'outside' | 'unreadable'
/// ```
fn mapped_in(maps: &Path, inside: &Inside<'_>) -> OpenFiles {
    use std::io::BufRead as _;
    let file: std::fs::File = match std::fs::File::open(maps) {
        Ok(found) => found,
        // The process went while it was being read, and its mappings went with it.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return OpenFiles::Outside,
        Err(_refused) => return OpenFiles::Unreadable,
    };
    for line in std::io::BufReader::new(file).lines() {
        let text: String = match line {
            Ok(found) => found,
            Err(_unread) => return OpenFiles::Unreadable,
        };
        let Some(named) = mapped_file_of(text.as_str()) else {
            continue;
        };
        let at: &Path = without_deleted_suffix(Path::new(named));
        if inside.holds(at) && !inside.is_the_git_dir(at) {
            return OpenFiles::Inside;
        }
    }
    return OpenFiles::Outside;
}

/// What:
///  The file one line of a map table names,
///  or nothing for a mapping of no file.
/// Why:
///  The kernel writes six fields and then the file's path,
///  pads the path with spaces so paths line up,
///  and a path may itself hold spaces,
///  so only the first five separators end fields.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mappedFileOf('7f-7f r--p 00000000 00:2f 1234    /x/y') // '/x/y'
/// ```
fn mapped_file_of(line: &str) -> Option<&str> {
    let mut fields = line.splitn(6, ' ');
    for _field in 0..5 {
        fields.next()?;
    }
    // The kernel pads the last field with spaces so the paths line up, and a mapping of no file ends after the inode.
    let named: &str = fields.next()?.trim_start_matches(' ');
    return (!named.is_empty()).then_some(named);
}

/// What a read of one process could not say.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WhatWasUnreadable {
    /// Its working folder,
    /// or the working folders of its threads.
    Folder,
    /// The files it holds open.
    Files,
    /// The files it has mapped.
    Mapped,
}

/// What:
///  Why a process this account could not see into may still be working in the worktree,
///  or nothing when it is taken to hold nothing there.
/// Why:
///   Three reads can be refused for the same reason,
///  a working folder,
///  a descriptor table and a map table,
///  and all are judged by the same two facts:
///  another account's process is not this account's to wait for,
///  and a process that started before the worktree was made is taken to hold nothing there,
///  though one can change into it or open a file in it afterwards,
///  which is the miss the module doc names.
///  An account that cannot be read is nobody's proof:
///  a `stat` that failed once answered "another account's" and the process was skipped in silence,
///  so a process whose account is unknown is judged by its start like one of this account's,
///  and its reason says that its account could not be read.
///  An age that cannot be read excuses nothing,
///  so the worktree stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mayWorkThere(entry, procRoot, pid, uptime, worktreeAge, what) // WorkingFolder | undefined
/// ```
fn may_work_there(
    entry: &Path,
    proc_root: &Path,
    pid: u32,
    uptime: Option<Duration>,
    worktree_age: Option<Duration>,
    what: WhatWasUnreadable,
) -> Option<WorkingFolder> {
    let owner: Owner = owner_of(entry);
    if owner == Owner::OtherAccount {
        return None;
    }
    if started_before(started_ago(proc_root, pid, uptime), worktree_age) {
        return None;
    }
    return Some(match (owner, what) {
        (Owner::ThisAccount, WhatWasUnreadable::Folder) => WorkingFolder::Unreadable,
        (Owner::ThisAccount, WhatWasUnreadable::Files | WhatWasUnreadable::Mapped) => WorkingFolder::OpenFilesUnreadable,
        (Owner::Unknown, _) => WorkingFolder::AccountUnknown,
        // Another account's process is taken to hold nothing there, which the answer above has already returned for.
        (Owner::OtherAccount, _) => WorkingFolder::AccountUnknown,
    });
}

/// What a read could tell about the account that owns one process.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Owner {
    /// The account this program runs as,
    /// which is the one whose hidden processes this read protects.
    ThisAccount,
    /// Another account's process,
    /// which `git worktree remove` has never waited for.
    OtherAccount,
    /// Nothing proves which account it belongs to,
    /// so it is nobody's reason to skip.
    Unknown,
}

/// What:
///  The account that owns one process,
///  or that nothing proves it.
/// Why:
///   A folder only this account's processes hide is this account's to protect,
///  and another account's process is not.
///  A `stat` that failed answered "another account's" once,
///  and a process whose record could not be read was then skipped in silence:
///  an entry that went while it was read is nobody's, and one whose own record cannot be read is unknown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ownerOf('/proc/4242') // 'this' | 'other' | 'unknown'
/// ```
#[cfg(unix)]
fn owner_of(entry: &Path) -> Owner {
    use std::os::unix::fs::MetadataExt as _;
    let Ok(theirs) = std::fs::metadata(entry) else {
        // A record that cannot be read as a link either has gone; one that is there and unreadable proves nothing.
        return if std::fs::symlink_metadata(entry).is_err() { Owner::OtherAccount } else { Owner::Unknown };
    };
    let Ok(ours) = std::fs::metadata(Path::new(PROCESS_TABLE_SELF)) else {
        return Owner::Unknown;
    };
    return if theirs.uid() == ours.uid() { Owner::ThisAccount } else { Owner::OtherAccount };
}

// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
/// No account of its own on a platform without Unix users,
/// so a folder that cannot be read is read as another program's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ownerOf(entry) // 'other'
/// ```
#[cfg(not(unix))]
fn owner_of(_entry: &Path) -> Owner {
    return Owner::OtherAccount;
}

/// What:
///  The path a descriptor names or a map line names,
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
///  How far apart two readings of a start and of a birth must be before one is older than the other.
/// Why:
///  `/proc/uptime` counts hundredths of a second and a start counts ticks of ten milliseconds,
///  so `/proc` places a start within twenty milliseconds of when it happened
///  (`process/process_start.rs` `LINUX_RESOLUTION_MS` is the same twenty),
///  and a folder's birth is a time the file system records to the nanosecond.
///  Measured on this host before the allowance,
///  a process started one millisecond after the folder was taken as older in four of a hundred runs,
///  and one started eight milliseconds after it in six of a hundred:
///  the removal then kept the worktree of a process that was younger than it and removed another.
///  Two readings closer than this cannot be ordered, and a pair that cannot be ordered keeps the worktree.
const START_RESOLUTION: Duration = Duration::from_millis(20);

/// What:
///  Whether a process that started this long ago is proven to have started before a folder this young.
/// Why:
///   A working folder is a directory the process changed into,
///  and a process that started before the worktree was made can change into it afterwards,
///  so this answers by a guess at which processes do that and not by a proof:
///  what it assumes is that a process works in the folder it started in.
///  A process younger than the worktree may be inside,
///  as may one whose age cannot be read,
///  as may one that is older by less than the two readings can show,
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
    return aged_process > aged_folder.saturating_add(START_RESOLUTION);
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

/// Process look controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_process_inside_tests.rs"]
mod tests;
