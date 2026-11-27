//! What:
//! Reads of the mounted procfs:
//! opening `/proc` once,
//! showing that it numbers processes as this process's own namespace does,
//! and reading a file below it without leaving it.
//! Why:
//! `control_group_process_identity.rs` tells which process a process id names;
//! the procfs opening and reading were split out of it to keep each file under the code-line limit.
//! The observation that a pidfd's process has a given number and the read of that process's stat file
//! must come from one procfs,
//! so both go through one descriptor.
//! The items are re-exported from `control_group_process_identity`,
//! so their paths there are unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // control-group-procfs-reading.ts
//! function openProcRoot(): ProcRoot                                        // throws the reason
//! function readInProc(root: ProcRoot, relative: string): string            // throws
//! function numberInProcfs(root: ProcRoot, pid: number, pidfd: File): number // throws the reason
//! ```

/// The magic number of procfs,
/// which a file served by the kernel must carry.
use crate::control_group_machine_identity::PROC_MAGIC;
/// Error text built only on failure,
/// from a `format_args!` value.
use crate::lazy_error_text::{
    ExplainedError,
    OrText,
};

/// Imports `std::fs` for this file.
use std::fs;

/// Imports `std::io` for this file.
use std::io;

/// Imports `std::io::Read` for this file.
use std::io::Read;

/// Imports `std::os::fd::OwnedFd` for this file.
use std::os::fd::OwnedFd;

/// Imports `rustix::fs::{AtFlags, StatxFlags, statx}` for this file.
use rustix::fs::{
    AtFlags,
    StatxFlags,
    statx,
};
/// The procfs,
/// opened once:
/// every numeric lookup of one question goes through this descriptor.
pub(crate) struct ProcRoot {
    /// The root directory of the procfs mount,
    /// read here and in `control_group_process_identity`,
    /// which therefore sees this field.
    pub(crate) directory: OwnedFd,
    /// This process's own number in that procfs,
    /// read here and in `control_group_process_identity`,
    /// which therefore sees this field.
    pub(crate) own: i32,
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
    let directory: OwnedFd = rustix::fs::open(
        "/proc",
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::DIRECTORY | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .explained(format_args!("/proc cannot be opened"))?;
    let found =
        rustix::fs::fstatfs(&directory).explained(format_args!("/proc cannot be examined"))?;
    if u64::try_from(found.f_type).unwrap_or(0) != PROC_MAGIC {
        return Err("/proc is not a procfs".to_string());
    }
    let attributes = statx(&directory, "", AtFlags::EMPTY_PATH, StatxFlags::TYPE)
        .explained(format_args!("/proc cannot be examined"))?;
    if !attributes
        .stx_attributes_mask
        .contains(rustix::fs::StatxAttributes::MOUNT_ROOT)
        || !attributes
            .stx_attributes
            .contains(rustix::fs::StatxAttributes::MOUNT_ROOT)
    {
        return Err("/proc is not the root of a procfs mount".to_string());
    }
    let own: i32 = rustix::process::getpid().as_raw_nonzero().get();
    let link = rustix::fs::readlinkat(&directory, "self", Vec::new())
        .explained(format_args!("the procfs does not name this process"))?;
    let named: i32 = link
        .to_str()
        .ok()
        .and_then(parse_pid_text)
        .or_text(format_args!(
            "the procfs names this process in a form that is not a number"
        ))?;
    if named != own {
        return Err(format!(
            "the mounted procfs numbers this process {named} and this process is {own}, so it serves another process id namespace than this process is in"
        ));
    }
    let root: ProcRoot = ProcRoot {
        directory,
        own,
    };
    // Equal numbers do not prove the same namespace: an ancestor's procfs can number this process, by coincidence, as the process
    // numbers itself. The `NSpid` line lists the process's id in every namespace from the one the procfs belongs to down to its own,
    // so a procfs of the process's own namespace lists exactly one (certification, third pass).
    match read_in_proc(&root, &format!("{own}/status")) {
        Ok(status) => nspid_shows_own_namespace(&status)?,
        Err(error) => {
            return Err(format!(
                "the status of this process cannot be read to check its namespace ({error})"
            ));
        }
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
                return Err(format!(
                    "the mounted procfs lists this process in {entries} process id namespaces, so it belongs to an ancestor namespace"
                ));
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
pub(crate) fn read_in_proc(
    root: &ProcRoot,
    relative: &str,
) -> io::Result<String> {
    let opened: OwnedFd = rustix::fs::openat2(
        &root.directory,
        relative,
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
        rustix::fs::ResolveFlags::BENEATH
            | rustix::fs::ResolveFlags::NO_XDEV
            | rustix::fs::ResolveFlags::NO_SYMLINKS,
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
pub(crate) fn number_in_procfs(
    root: &ProcRoot,
    pid: i32,
    pidfd: &OwnedFd,
) -> Result<i32, String> {
    /// The raw descriptor number of the process handle,
    /// which names its entry in `/proc/self/fdinfo`.
    use std::os::fd::AsRawFd;
    let info: String = read_in_proc(root, &format!("{}/fdinfo/{}", root.own, pidfd.as_raw_fd()))
        .explained(format_args!(
            "the pidfd of process {pid} cannot be examined through procfs"
        ))?;
    let line: &str = info.lines().find_map(pid_line_value).or_text(format_args!(
        "the pidfd of process {pid} has no Pid line in procfs"
    ))?;
    return line.trim().parse::<i32>().explained(format_args!(
        "the Pid line of the pidfd of process {pid} is not a number"
    ));
}
