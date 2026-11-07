//! What:
//!  Tell whether any process on the machine has a file open for writing, with a read lease taken and given back at once.
//! Why:
//!  The fold of an earlier log (`earlier_log_lines_added_to_current_log.rs`) removes the log it moved aside only when no program
//!  that opened the log before the move can still write a line to it, and the owner's rules are "never discard user data" and
//!  "Use as few locks as possible". A read lease (`fcntl` `F_SETLEASE` with `F_RDLCK`) is granted only while no process holds
//!  the file's inode open for writing, in any pid namespace and for any user, so the grant is a probe: it takes nothing from
//!  anyone and nothing stays held. Scanning `/proc/<pid>/fd` cannot see other users' processes or other namespaces.
//!  A lease break sends a signal whose default action ends the process, so the descriptor's signal is set to `SIGURG` first,
//!  which does nothing by default; a writer that opens the file in the instant of the probe waits for the lease to be given back,
//!  and it is given back before this function returns. The standard library and `rustix` expose no lease call, so this file
//!  calls `libc::fcntl` as `rename_that_never_replaces.rs` calls `libc::renameat2`.
//!  Measured on 2026-11-07 on this host's btrfs and in the test container's overlay file system: the grant is refused while a
//!  writer is open and granted once it closed, and a writer opening during the lease keeps the prober alive.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const check = await writerCheck(path); // 'no-writer' | 'writer-holds'; no Node API, the kernel call is fcntl(F_SETLEASE)
//! ```

use std::path::Path;

/// What the probe found.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WriterCheck {
    /// No process has the file open for writing, and none can open it by a name nobody else knows.
    NoWriter,
    /// Some process has the file open for writing now.
    WriterHolds,
}

/// The `fcntl` command that picks the signal a lease break sends (`asm-generic/fcntl.h`, which `libc` lacks for GNU Linux).
#[cfg(target_os = "linux")]
const F_SETSIG: libc::c_int = 10;

/// Take a read lease on an open file without waiting: `true` when granted, `false` when a process holds the file open for writing.
///
/// # Errors
/// Any other refusal: the file belongs to another user (`EACCES`), leases are off (`/proc/sys/fs/leases-enable`), or the file
/// system has no leases (`EINVAL`).
#[cfg(target_os = "linux")]
fn read_lease_granted(file: &std::fs::File) -> std::io::Result<bool> {
    use std::os::fd::AsRawFd;
    // SAFETY: the descriptor is open for both calls, and neither call reads or writes memory.
    let signal_set: libc::c_int = unsafe { libc::fcntl(file.as_raw_fd(), F_SETSIG, libc::SIGURG) };
    if signal_set != 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: as above.
    let granted: libc::c_int = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETLEASE, libc::F_RDLCK) };
    if granted == 0 {
        return Ok(true);
    }
    let error: std::io::Error = std::io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::EAGAIN) {
        return Ok(false);
    }
    return Err(error);
}

/// Give a lease back.
///
/// # Errors
/// The kernel's refusal, which closing the descriptor would answer anyway.
#[cfg(target_os = "linux")]
fn lease_given_back(file: &std::fs::File) -> std::io::Result<()> {
    use std::os::fd::AsRawFd;
    // SAFETY: the descriptor is open for the call, which reads or writes no memory.
    let released: libc::c_int = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETLEASE, libc::F_UNLCK) };
    return if released == 0 { Ok(()) } else { Err(std::io::Error::last_os_error()) };
}

/// What:
///  Whether any process has the file at `path` open for writing, found by taking a read lease on it and giving it back.
/// Why:
///   See the module's account. `NoWriter` also means none can appear while the file has only a private name, because every
///  open for writing goes through the name or an inherited descriptor, and an open that follows the probe finds the name gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writerCheck(path: string): WriterCheck;
/// ```
///
/// # Errors
/// The file cannot be opened, or the kernel refuses the lease for a reason other than a writer (see `read_lease_granted`).
#[cfg(target_os = "linux")]
pub fn writer_check(path: &Path) -> std::io::Result<WriterCheck> {
    use std::os::unix::fs::OpenOptionsExt;
    let file: std::fs::File = std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(path)?;
    if !read_lease_granted(&file)? {
        return Ok(WriterCheck::WriterHolds);
    }
    lease_given_back(&file)?;
    return Ok(WriterCheck::NoWriter);
}

/// A system without leases cannot tell, so the caller keeps the file.
#[cfg(not(target_os = "linux"))]
pub fn writer_check(_path: &Path) -> std::io::Result<WriterCheck> {
    return Err(std::io::Error::from(std::io::ErrorKind::Unsupported));
}

#[cfg(test)]
#[path = "file_open_for_write_by_any_process_tests.rs"]
mod tests;
