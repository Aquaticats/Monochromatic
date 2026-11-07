//! What:
//! Which process a pid names and whether the launcher of a scope is alive:
//! reads of `/proc` that tell a process from the one that ran before it.
//! Why:
//! Split out of `control_group.rs`,
//! which holds the containment of a run in a control group,
//! to keep that file under the code-line limit.
//! The items are re-exported from `control_group`,
//! so their paths there are unchanged.

/// The magic number of procfs,
/// which a file served by the kernel must carry.
use crate::control_group_machine_identity::PROC_MAGIC;
/// Error text built only on failure,
/// from a `format_args!` value.
use crate::lazy_error_text::{ExplainedError, OrText};

/// Imports `std::fs` for this file.
use std::fs;

/// Imports `std::io` for this file.
use std::io;

/// Imports `std::io::{Read, Write}` for this file.
use std::io::{Read};

/// Imports `std::os::fd::OwnedFd` for this file.
use std::os::fd::OwnedFd;

/// Imports `std::os::unix::fs::{FileExt, MetadataExt}` for this file.
use std::os::unix::fs::{MetadataExt};








/// Imports `rustix::fs::{AtFlags, CWD, StatxFlags, statx}` for this file.
use rustix::fs::{AtFlags, StatxFlags, statx};

/// Imports `rustix::io::Errno` for this file.
use rustix::io::Errno;

/// Imports `rustix::process::{Pid, PidfdFlags, Signal, pidfd_open, pidfd_send_signal}` for this file.
use rustix::process::{Pid, PidfdFlags, pidfd_open};

/// What `launcher_state` found about the process a launcher record names.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum LauncherState {
    /// No process with that id and start time exists in the namespace of the record:
    /// it can never join the scope.
    Gone,
    /// The process exists (a zombie that is not reaped yet counts,
    /// because only its reaping shows it ended).
    Alive,
    /// The question cannot be answered,
    /// with the reason.
    Unknown(
        /// Why.
        String,
    ),
}

/// What:
/// `pub(crate) fn pid_namespace_id() -> io::Result<u64>` is the inode number of this process's process id namespace.
/// Why:
/// A recorded process id means the same process only inside the namespace it was recorded in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function pidNamespaceId(): number
/// ```
pub(crate) fn pid_namespace_id() -> io::Result<u64> {
    return Ok(fs::metadata("/proc/self/ns/pid")?.ino());
}

/// What `start_of` found about a process id.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum StartOf {
    /// No process has this id in this process's namespace.
    Gone,
    /// The process,
    /// with its start time,
    /// read through a procfs that numbers it with this id.
    Process(
        /// The start time in clock ticks since boot.
        u64,
    ),
    /// The question cannot be answered,
    /// with the reason.
    Unknown(
        /// Why.
        String,
    ),
}

/// What:
/// `pub(crate) fn missing_entry_verdict(number: i64, pid: i32, reason: &str) -> StartOf` reads the number that the procfs gives a pidfd's process
/// after the process's directory could not be read.
/// Why:
/// Only the value the kernel prints for a process that has ended and been reaped (`-1`) shows that the process is gone;
/// the same number as before means the process exists and procfs hides it,
/// and any other number (another positive id,
/// or `0` for a
/// process that this procfs does not show) says nothing about whether it ended (round 13,
/// 12a).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function missingEntryVerdict(number: number, pid: number, reason: string): StartOf
/// ```
pub(crate) fn missing_entry_verdict(number: i64, pid: i32, reason: &str) -> StartOf {
    return match number {
        n if n == i64::from(pid) => StartOf::Unknown(format!("process {pid} exists but /proc/{pid}/stat cannot be read ({reason})")),
        -1 => StartOf::Gone,
        other => StartOf::Unknown(format!("process {pid} is numbered {other} by the procfs after its directory could not be read ({reason}), which does not show that it ended")),
    };
}

/// The procfs,
/// opened once:
/// every numeric lookup of one question goes through this descriptor.
pub(crate) struct ProcRoot {
    /// The root directory of the procfs mount.
    directory: OwnedFd,
    /// This process's own number in that procfs.
    own: i32,
}

/// What:
/// `fn open_proc_root() -> Result<ProcRoot, String>` opens `/proc` once and shows that it is the root of a procfs mount that numbers
/// this process as the caller's own namespace does.
/// Why:
/// The observation that a pidfd's process has a given number and the read of that process's stat file must come from one procfs,
/// so both go through this one descriptor (round 13,
/// 12a);
/// a procfs that numbers the caller differently is another namespace's.
/// This is a guard that costs a few system calls;
/// a program that deliberately changes mounts or `/proc` while a run or a recovery
/// is in progress is outside the contract (assumption a,
/// amended in round 13).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openProcRoot(): ProcRoot  // throws the reason
/// ```
pub(crate) fn open_proc_root() -> Result<ProcRoot, String> {
    let directory: OwnedFd = rustix::fs::open("/proc", rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::DIRECTORY | rustix::fs::OFlags::CLOEXEC, rustix::fs::Mode::empty())
        .explained(format_args!("/proc cannot be opened"))?;
    let found = rustix::fs::fstatfs(&directory).explained(format_args!("/proc cannot be examined"))?;
    if u64::try_from(found.f_type).unwrap_or(0) != PROC_MAGIC {
        return Err("/proc is not a procfs".to_string());
    }
    let attributes = statx(&directory, "", AtFlags::EMPTY_PATH, StatxFlags::TYPE).explained(format_args!("/proc cannot be examined"))?;
    if !attributes.stx_attributes_mask.contains(rustix::fs::StatxAttributes::MOUNT_ROOT) || !attributes.stx_attributes.contains(rustix::fs::StatxAttributes::MOUNT_ROOT) {
        return Err("/proc is not the root of a procfs mount".to_string());
    }
    let own: i32 = rustix::process::getpid().as_raw_nonzero().get();
    let link = rustix::fs::readlinkat(&directory, "self", Vec::new()).explained(format_args!("the procfs does not name this process"))?;
    let named: i32 = link.to_str().ok().and_then(parse_pid_text).or_text(format_args!("the procfs names this process in a form that is not a number"))?;
    if named != own {
        return Err(format!("the mounted procfs numbers this process {named} and this process is {own}, so it serves another process id namespace than this process is in"));
    }
    let root: ProcRoot = ProcRoot { directory, own };
    // Equal numbers do not prove the same namespace: an ancestor's procfs can number this process, by coincidence, as the process
    // numbers itself. The `NSpid` line lists the process's id in every namespace from the one the procfs belongs to down to its own,
    // so a procfs of the process's own namespace lists exactly one (certification, third pass).
    match read_in_proc(&root, &format!("{own}/status")) {
        Ok(status) => nspid_shows_own_namespace(&status)?,
        Err(error) => return Err(format!("the status of this process cannot be read to check its namespace ({error})")),
    }
    return Ok(root);
}

/// What:
/// `fn nspid_shows_own_namespace(status: &str) -> Result<(), String>` accepts a status file whose `NSpid` line has one entry (or that
/// has no such line,
/// a kernel before 4.1) and refuses one that lists the process in more than one namespace.
/// Why:
/// A procfs of an ancestor namespace lists the process in its namespace and in each below it,
/// even when two of the numbers coincide.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nspidShowsOwnNamespace(status: string): void  // throws the reason
/// ```
pub(crate) fn nspid_shows_own_namespace(status: &str) -> Result<(), String> {
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("NSpid:") {
            let entries: usize = rest.split_whitespace().count();
            if entries != 1 {
                return Err(format!("the mounted procfs lists this process in {entries} process id namespaces, so it belongs to an ancestor namespace"));
            }
            return Ok(());
        }
    }
    return Ok(());
}

/// What:
/// `fn read_in_proc(root: &ProcRoot, relative: &str) -> io::Result<String>` reads a file below the procfs root,
/// refusing to leave it,
/// to cross a mount point or to follow a link on the way.
/// Why:
/// `openat2` with `RESOLVE_BENEATH | RESOLVE_NO_XDEV | RESOLVE_NO_SYMLINKS` (Linux 5.6;
/// a kernel without it gives an error,
/// which the
/// callers read as unknown,
/// never as gone) keeps a numeric lookup inside the procfs that `open_proc_root` checked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readInProc(root: ProcRoot, relative: string): string
/// ```
pub(crate) fn read_in_proc(root: &ProcRoot, relative: &str) -> io::Result<String> {
    let opened: OwnedFd = rustix::fs::openat2(
        &root.directory,
        relative,
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
        rustix::fs::ResolveFlags::BENEATH | rustix::fs::ResolveFlags::NO_XDEV | rustix::fs::ResolveFlags::NO_SYMLINKS,
    )
    .map_err(io::Error::from)?;
    let mut text: String = String::new();
    fs::File::from(opened).read_to_string(&mut text)?;
    return Ok(text);
}

/// What:
/// `fn parse_pid_text(text: &str) -> Option<i32>` reads a process id from text.
/// Why:
/// `Option::and_then` takes it by path instead of a closure.
pub(crate) fn parse_pid_text(text: &str) -> Option<i32> {
    return text.parse::<i32>().ok();
}

/// What:
/// `fn pid_line_value(candidate: &str) -> Option<&str>` returns the text after a `Pid:` prefix.
/// Why:
/// `find_map` takes it by path instead of a closure.
pub(crate) fn pid_line_value(candidate: &str) -> Option<&str> {
    return candidate.strip_prefix("Pid:");
}

/// What:
/// `fn number_in_procfs(root: &ProcRoot, pid: i32, pidfd: &OwnedFd) -> Result<i32, String>` reads the number that the mounted procfs
/// gives to the process a pidfd names.
/// Why:
/// Only when it is the id asked for does `/proc/<pid>` name that process;
/// a function replaces the closure that captured the three arguments.
pub(crate) fn number_in_procfs(root: &ProcRoot, pid: i32, pidfd: &OwnedFd) -> Result<i32, String> {
    /// The raw descriptor number of the process handle,
    /// which names its entry in `/proc/self/fdinfo`.
    use std::os::fd::AsRawFd;
    let info: String = read_in_proc(root, &format!("{}/fdinfo/{}", root.own, pidfd.as_raw_fd())).explained(format_args!("the pidfd of process {pid} cannot be examined through procfs"))?;
    let line: &str = info.lines().find_map(pid_line_value).or_text(format_args!("the pidfd of process {pid} has no Pid line in procfs"))?;
    return line.trim().parse::<i32>().explained(format_args!("the Pid line of the pidfd of process {pid} is not a number"));
}

/// What:
/// `fn start_of(pid: i32) -> StartOf` reads the start time of a process id after showing that the mounted procfs numbers processes
/// as this process's own namespace does.
/// Why:
/// `kill`,
/// `pidfd_open` and the other system calls read a process id in the caller's active namespace,
/// but `/proc/<pid>` reads it in
/// the namespace of the procfs mount,
/// and `/proc/self/ns/pid` names only the caller's (round 12,
/// 11a:
/// a child namespace that still
/// uses an ancestor's procfs would make the two name different processes).
/// A pidfd names the process of the first reading,
/// and its `fdinfo` (read through the procfs) gives the number that this procfs
/// has for that same process:
/// only when it is the id asked for does `/proc/<pid>` name that process.
/// Anything else is unknown,
/// never gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startOf(pid: number): { gone } | { start: number } | { unknown: string }
/// ```
pub(crate) fn start_of(pid: i32) -> StartOf {
    let Some(found) = Pid::from_raw(pid) else {
        return StartOf::Unknown(format!("{pid} is not a process id"));
    };
    let pidfd: OwnedFd = match pidfd_open(found, PidfdFlags::empty()) {
        Ok(opened) => opened,
        Err(Errno::SRCH) => return StartOf::Gone,
        Err(error) => return StartOf::Unknown(format!("a pidfd for process {pid} cannot be opened ({error})")),
    };
    let root: ProcRoot = match open_proc_root() {
        Ok(opened) => opened,
        Err(reason) => return StartOf::Unknown(reason),
    };
    match number_in_procfs(&root, pid, &pidfd) {
        Ok(number) if number == pid => {}
        Ok(number) => return StartOf::Unknown(format!("the mounted procfs numbers process {pid} as {number}, so it serves another process id namespace than this process is in")),
        Err(reason) => return StartOf::Unknown(reason),
    }
    let stat_read: io::Result<String> = read_in_proc(&root, &format!("{pid}/stat"));
    let stat_parsed: io::Result<(u64, char)> = match stat_read {
        Ok(text) => parse_stat(pid, &text),
        Err(error) => Err(error),
    };
    return match stat_parsed {
        Ok((start, _)) => StartOf::Process(start),
        // The same process was numbered `pid` a moment ago: if the procfs now has no entry, it ended (its number is then -1), or hides it.
        Err(error) if error.kind() == io::ErrorKind::NotFound => match number_in_procfs(&root, pid, &pidfd) {
            Ok(number) => missing_entry_verdict(i64::from(number), pid, &error.to_string()),
            Err(reason) => StartOf::Unknown(reason),
        },
        Err(error) => StartOf::Unknown(format!("/proc/{pid}/stat cannot be read ({error})")),
    };
}

/// What:
/// `pub(crate) fn procfs_matches_namespace() -> Result<(), String>` says whether the mounted procfs numbers processes as this
/// process's own namespace does,
/// and why not when it does not.
/// Why:
/// An oracle of the tests that reads a numeric entry of `/proc` (a thread,
/// for one) must not run in an environment whose procfs
/// names other processes than the caller's namespace does:
/// it would read "no such entry" for a live process (certification,
/// second
/// pass).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function procfsMatchesNamespace(): void  // throws the reason
/// ```
#[cfg(test)]
pub(crate) fn procfs_matches_namespace() -> Result<(), String> {
    return open_proc_root().map(|_root| {});
}

/// What:
/// `fn names_an_absent_process(error: &io::Error) -> bool` says whether a failed read of a process's file shows that the process is
/// not there:
/// `ENOENT` (the entry is gone) or `ESRCH` (the entry was there when it was opened and the process ended before the read).
/// Why:
/// A read of `/proc/<pid>/stat` of a process that ends under it fails with either;
/// the map says both are read as gone,
/// and the pidfd
/// then confirms it,
/// so neither is taken as a failure of the oracle (fourth certification pass).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namesAnAbsentProcess(error: NodeJS.ErrnoException): boolean { return error.code === 'ENOENT' || error.code === 'ESRCH'; }
/// ```
#[cfg(test)]
pub(crate) fn names_an_absent_process(error: &io::Error) -> bool {
    return error.kind() == io::ErrorKind::NotFound || error.raw_os_error() == Some(Errno::SRCH.raw_os_error());
}

/// What:
/// `pub(crate) fn process_is_running(pid: i32) -> Result<bool, String>` says whether a process with this id is running (not ended,
/// not a zombie that is waiting to be reaped),
/// and fails when it cannot tell.
/// Why:
/// An oracle of the tests that decides "this process is stopped" must not fail open:
/// an inspection that cannot tell (a procfs of
/// another namespace,
/// an unreadable entry) is an error,
/// never "gone" (certification group 1).
/// It reads through the pinned procfs descriptor and a pidfd,
/// as the launcher question does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function processIsRunning(pid: number): boolean  // throws when it cannot tell
/// ```
#[cfg(test)]
pub(crate) fn process_is_running(pid: i32) -> Result<bool, String> {
    /// The raw descriptor number of the process handle,
    /// which names its entry in `/proc/self/fdinfo`.
    use std::os::fd::AsRawFd;
    let found: Pid = Pid::from_raw(pid).ok_or_else(|| { return format!("{pid} is not a process id"); })?;
    let pidfd: OwnedFd = match pidfd_open(found, PidfdFlags::empty()) {
        Ok(opened) => opened,
        Err(Errno::SRCH) => return Ok(false),
        Err(error) => return Err(format!("a pidfd for process {pid} cannot be opened ({error})")),
    };
    let root: ProcRoot = open_proc_root()?;
    let number = || -> Result<i64, String> {
        let info: String = read_in_proc(&root, &format!("{}/fdinfo/{}", root.own, pidfd.as_raw_fd())).map_err(|error| { return format!("the pidfd of process {pid} cannot be examined ({error})"); })?;
        let line: &str = info.lines().find_map(|candidate| { return candidate.strip_prefix("Pid:"); }).ok_or_else(|| { return format!("the pidfd of process {pid} has no Pid line"); })?;
        return line.trim().parse::<i64>().map_err(|error| { return format!("the Pid line of process {pid} is not a number ({error})"); });
    };
    match number()? {
        -1 => return Ok(false),
        other if other == i64::from(pid) => {}
        other => return Err(format!("the mounted procfs numbers process {pid} as {other}, so it serves another process id namespace")),
    }
    return match read_in_proc(&root, &format!("{pid}/stat")).and_then(|text| { return parse_stat(pid, &text); }) {
        Ok((_, state)) => Ok(state != 'Z' && state != 'X'),
        Err(error) if names_an_absent_process(&error) => match number()? {
            -1 => Ok(false),
            _ => Err(format!("process {pid} exists but /proc/{pid}/stat cannot be read ({error})")),
        },
        Err(error) => Err(format!("/proc/{pid}/stat cannot be read ({error})")),
    };
}

/// What:
/// `pub(crate) fn process_start(pid: i32) -> io::Result<u64>` is the start time of a process,
/// in clock ticks since boot
/// (field 22 of `/proc/<pid>/stat`).
/// Why:
/// A process id alone can be reused;
/// the id with its start time cannot name two processes in one boot.
/// It is an error,
/// never a guess,
/// when the procfs does not number processes as this process's namespace does (`start_of`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function processStart(pid: number): number
/// ```
pub(crate) fn process_start(pid: i32) -> io::Result<u64> {
    return match start_of(pid) {
        StartOf::Process(start) => Ok(start),
        StartOf::Gone => Err(io::Error::new(io::ErrorKind::NotFound, format!("process {pid} does not exist"))),
        StartOf::Unknown(reason) => Err(io::Error::other(reason)),
    };
}

/// What:
/// `fn parse_stat(pid: i32, text: &str) -> io::Result<(u64, char)>` reads the start time and the state letter from the text of a stat
/// file.
/// Why:
/// The name field of the stat file may hold spaces and brackets,
/// so the fields are counted after its last closing parenthesis.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseStat(pid: number, text: string): [number, string]
/// ```
pub(crate) fn parse_stat(pid: i32, text: &str) -> io::Result<(u64, char)> {
    let Some((_, after)) = text.rsplit_once(')') else {
        return Err(invalid_stat(pid, text));
    };
    let fields: Vec<&str> = after.split_whitespace().collect();
    // After the name: state is field 3, start time is field 22, so index 0 and 19 of what follows.
    let Some(state) = fields.first().and_then(first_char_of) else {
        return Err(invalid_stat(pid, text));
    };
    let Some(start_text) = fields.get(19) else {
        return Err(invalid_stat(pid, text));
    };
    let Ok(start) = start_text.parse::<u64>() else {
        return Err(invalid_stat(pid, text));
    };
    return Ok((start, state));
}

/// What:
/// `fn first_char_of(field: &&str) -> Option<char>` returns the first character of a field.
/// Why:
/// `Option::and_then` takes it by path instead of a closure.
pub(crate) fn first_char_of(field: &&str) -> Option<char> {
    return field.chars().next();
}

/// What:
/// `fn invalid_stat(pid: i32, text: &str) -> io::Error` is the error for a `stat` text that is not understood.
/// Why:
/// A function replaces the closure that captured the process id and the text.
pub(crate) fn invalid_stat(pid: i32, text: &str) -> io::Error {
    return io::Error::new(io::ErrorKind::InvalidData, format!("/proc/{pid}/stat is not understood: {text:?}"));
}

/// What:
/// `pub(crate) fn launcher_state(pid: i32, start: u64, pid_namespace: u64) -> LauncherState` says whether the recorded launcher can
/// still join its scope.
/// Why:
/// The launcher becomes the program,
/// and a process that is gone cannot be attached to a scope later (round 11).
/// It is gone only when this process is in the process id namespace of the record and no process has that id (a different start
/// time is unknown,
/// not gone),
/// read through a procfs that numbers processes as this namespace does (`start_of`,
/// round 12);
/// a different namespace,
/// another namespace's procfs,
/// a hidden or unreadable process and a state that is not understood are
/// "unknown",
/// never "gone".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function launcherState(pid: number, start: number, pidNamespace: number): 'gone' | 'alive' | { unknown: string }
/// ```
pub(crate) fn launcher_state(pid: i32, start: u64, pid_namespace: u64) -> LauncherState {

    match pid_namespace_id() {
        Ok(found) if found == pid_namespace => {}
        Ok(_) => return LauncherState::Unknown(format!("the launcher {pid} was recorded in another process id namespace than this process is in")),
        Err(error) => return LauncherState::Unknown(format!("the process id namespace of this process cannot be identified ({error})")),
    }
    return match start_of(pid) {
        StartOf::Gone => LauncherState::Gone,
        // The start time in procfs is relative to the boot-time offset of the reader's time namespace, and that the reader is in the
        // writer's time namespace cannot be proven (a namespace number is recycled once the namespace has no process), so a start time
        // that differs from the record does not show a reused id: it is unknown (round 15, 14 section 2).
        // An equal start time keeps the launcher alive, and an id that no process has is gone whatever the basis.
        StartOf::Process(seen) if seen != start => LauncherState::Unknown(format!(
            "process {pid} has start time {seen} and the launcher was recorded with start time {start}; the start time in procfs is relative to the reader's time namespace, which cannot be shown to be the writer's, so it cannot be told whether the launcher is gone or its id was reused"
        )),
        StartOf::Process(_) => LauncherState::Alive,
        StartOf::Unknown(reason) => LauncherState::Unknown(reason),
    };
}
