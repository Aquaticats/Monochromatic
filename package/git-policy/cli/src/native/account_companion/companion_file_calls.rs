//! What:
//!  The file and mount calls the companion's child makes:
//!  opening and writing the two mapping files,
//!  the scratch tmpfs and the private propagation,
//!  the directories,
//! the link and the rename of the refusal tree,
//!  and the identity of one folder.
//! Why:
//!  Every one of these runs in a process that copied only the thread that
//!  forked it,
//!  so no call here allocates,
//! formats text or takes a library lock
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The account companion",
//! "The child starts").
//!  A refused call answers its own name and the kernel's number,
//!  and every path or word came from the parent.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! unshareUserNamespace(); writeAll(openForWrite(setgroups), 'deny'); mountTmpfs(scratch, options);
//! ```

use super::companion_calls::{CompanionCall, Failure, failed};
use std::ffi::CStr;
use std::os::fd::RawFd;

/// The identity of one filesystem object,
/// as the walk records and the changed-identity control compares it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Identity {
    /// The device the object is on.
    pub device: u64,
    /// Its inode number.
    pub inode: u64,
    /// Its birth time,
    /// where the filesystem reports one.
    pub birth: Option<(i64, u32)>,
}

/// What:
///  End this child at once,
///  never through its parent's frames.
/// Why:
///  The release profile unwinds a panic,
///  and a child that returned would run a second copy of the wrapper.
pub fn exit_now(status: i32) -> ! {
    // SAFETY: `_exit` ends this process and returns nothing.
    unsafe { libc::_exit(status) };
}

/// What:
///  Put this process,
///  or the child it names,
///  in a process group of its own.
/// Why:
///  The deadline kills the group,
///  which is the only way to end a child and everything it started.
///
/// # Errors
/// The call refused.
pub fn set_process_group(pid: libc::pid_t) -> Result<(), Failure> {
    // SAFETY: `setpgid` reads two process identifiers and touches no memory of ours.
    if unsafe { libc::setpgid(pid, pid) } == 0 { Ok(()) } else { Err(failed(CompanionCall::Fork)) }
}

/// What:
///  Have the kernel kill this child when the thread that forked it dies.
/// Why:
///  A wrapper killed outright must take its child with it,
///  so its namespace and the tmpfs inside it go,
///  and the signal follows the forking thread rather than the process.
///
/// # Errors
/// The call refused.
pub fn parent_death_signal() -> Result<(), Failure> {
    // SAFETY: `prctl` reads its four words and writes nothing of ours.
    if unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL, 0, 0, 0) } == 0 { Ok(()) } else { Err(failed(CompanionCall::Fork)) }
}

/// The process that started this one.
pub fn parent_id() -> libc::pid_t {
    // SAFETY: `getppid` takes no argument and reads one number.
    return unsafe { libc::getppid() };
}

/// What:
///  Make one descriptor the copy of another,
///  which is how a child keeps its status pipe below the range it closes.
///
/// # Errors
/// The call refused.
pub fn move_descriptor(from: RawFd, to: RawFd) -> Result<(), Failure> {
    // SAFETY: `dup2` reads two descriptor numbers and touches no memory of ours.
    if unsafe { libc::dup2(from, to) } < 0 {
        return Err(failed(CompanionCall::Pipe));
    }
    if from != to {
        // SAFETY: the source is this process's own descriptor and is closed exactly once.
        let _closed: libc::c_int = unsafe { libc::close(from) };
    }
    return Ok(());
}

/// What:
///  Close every descriptor numbered `first` or higher,
///  and the one number given,
///  which is how a child drops what its parent's other threads held.
///
/// # Errors
/// The call refused.
pub fn close_range(first: u32, last: u32) -> Result<(), Failure> {
    // SAFETY: `close_range` closes descriptors this process holds and touches no memory of ours.
    if unsafe { libc::syscall(libc::SYS_close_range, first, last, 0 as libc::c_uint) } == 0 { Ok(()) } else { Err(failed(CompanionCall::Fork)) }
}

/// What:
///  Open one file for writing,
///  with no name created.
///
/// # Errors
/// The call refused.
pub fn open_for_write(path: &CStr, call: CompanionCall) -> Result<RawFd, Failure> {
    // SAFETY: the path is the caller's and outlives the call; the answer is owned once.
    let opened: libc::c_int = unsafe { libc::open(path.as_ptr(), libc::O_WRONLY | libc::O_CLOEXEC) };
    return if opened < 0 { Err(failed(call)) } else { Ok(opened) };
}

/// What:
///  Write one whole buffer to a descriptor.
/// Why:
///  A pipe or a file takes part of a buffer,
///  and a mapping line that was cut in half is not a line.
///
/// # Errors
/// The call refused.
pub fn write_all(descriptor: RawFd, bytes: &[u8], call: CompanionCall) -> Result<(), Failure> {
    let mut written: usize = 0;
    while written < bytes.len() {
        // SAFETY: the buffer holds the room the call is given, from the offset it is written at.
        let put: libc::ssize_t = unsafe { libc::write(descriptor, bytes.as_ptr().add(written).cast::<libc::c_void>(), bytes.len() - written) };
        if put <= 0 {
            return Err(failed(call));
        }
        written += usize::try_from(put).unwrap_or(bytes.len());
    }
    return Ok(());
}

/// What:
///  Close one descriptor a call this module made opened.
pub fn close(descriptor: RawFd) {
    // SAFETY: the descriptor is this call's own and is not used after this.
    let _closed: libc::c_int = unsafe { libc::close(descriptor) };
}

/// What:
///  `unshare(CLONE_NEWUSER)`:
///  a user namespace of this process's own.
///
/// # Errors
/// The call refused.
pub fn unshare_user_namespace() -> Result<(), Failure> {
    // SAFETY: `unshare` reads the one flag word and touches no memory of ours.
    if unsafe { libc::unshare(libc::CLONE_NEWUSER) } == 0 { Ok(()) } else { Err(failed(CompanionCall::UnshareUserNamespace)) }
}

/// What:
///  `unshare(CLONE_NEWNS)`:
///  a mount namespace of this process's own,
///  so no mount reaches the host's table.
///
/// # Errors
/// The call refused.
pub fn unshare_mount_namespace() -> Result<(), Failure> {
    // SAFETY: `unshare` reads the one flag word and touches no memory of ours.
    if unsafe { libc::unshare(libc::CLONE_NEWNS) } == 0 { Ok(()) } else { Err(failed(CompanionCall::UnshareMountNamespace)) }
}

/// What:
///  Make one directory inside the scratch tmpfs.
///
/// # Errors
/// The call refused.
pub fn make_directory(path: &CStr) -> Result<(), Failure> {
    // SAFETY: the path is the caller's and outlives the call.
    let made: libc::c_int = unsafe { libc::mkdir(path.as_ptr(), 0o700) };
    return if made == 0 { Ok(()) } else { Err(failed(CompanionCall::Mkdir)) };
}

/// What:
///  Make one symbolic link at a name inside a held directory.
///
/// # Errors
/// The call refused.
pub fn make_symlink(target: &CStr, directory: RawFd, name: &CStr) -> Result<(), Failure> {
    // SAFETY: both names are the caller's and outlive the call.
    let made: libc::c_int = unsafe { libc::symlinkat(target.as_ptr(), directory, name.as_ptr()) };
    return if made == 0 { Ok(()) } else { Err(failed(CompanionCall::Symlinkat)) };
}

/// What:
///  Move one name inside a held directory to another.
///
/// # Errors
/// The call refused.
pub fn rename_within(directory: RawFd, from: &CStr, to: &CStr) -> Result<(), Failure> {
    // SAFETY: both names are the caller's and outlive the call, and neither link is followed.
    let moved: libc::c_int = unsafe { libc::renameat(directory, from.as_ptr(), directory, to.as_ptr()) };
    return if moved == 0 { Ok(()) } else { Err(failed(CompanionCall::Renameat)) };
}

/// What:
///  The identity of one open folder,
///  read without following anything.
/// Why:
///  The walk's answer is an identity rather than a name,
///  and the changed-identity control compares two of them.
///
/// # Errors
/// The call refused.
pub fn identity_of(descriptor: RawFd) -> Result<Identity, Failure> {
    // SAFETY: an all-zero `statx` is a valid value for the call to fill.
    let mut found: libc::statx = unsafe { std::mem::zeroed() };
    // SAFETY: the descriptor is the caller's, the empty name names it, and `found` is a live `statx`.
    let answered: libc::c_int = unsafe { libc::statx(descriptor, c"".as_ptr(), libc::AT_EMPTY_PATH, libc::STATX_BASIC_STATS | libc::STATX_BTIME, &mut found) };
    if answered < 0 {
        return Err(failed(CompanionCall::Statx));
    }
    let birth: Option<(i64, u32)> = if found.stx_mask & libc::STATX_BTIME == 0 {
        None
    } else {
        let _ = &found;
        Some((found.stx_btime.tv_sec, found.stx_btime.tv_nsec))
    };
    return Ok(Identity { device: libc::makedev(found.stx_dev_major, found.stx_dev_minor), inode: found.stx_ino, birth });
}
