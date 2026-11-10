//! What:
//!  The device and the inode of the file an open descriptor of another process refers to,
//!  read in a child of this process that the reader stops waiting for at a ceiling.
//! Why:
//!  Reading those attributes through the path of the descriptor's `/proc` link walks that path,
//!  and a stopped file system server holds that walk in uninterruptible sleep for ever
//!  (`doc/troubleshooting/fuse-descriptor-attributes.md`).
//!  The mount's own type decides whether such a server can be behind the file,
//!  and that type is not always something the reader can act on:
//!  a container's root is a kernel `overlay` mount whose layers are the host's directories,
//!  which no process inside the container can name.
//!  Measured on 2026-11-09 in this package's gate container:
//!  the mount's own options name `lowerdir` and `upperdir` under
//!  `/var/home/user/.local/share/containers/storage`,
//!  and every one of those paths is `ENOENT` inside the container,
//!  while the layers themselves are the host's local disk.
//!  Refusing that mount leaves an abandoned `index.lock` that no scan can prove unheld,
//!  so every commit behind it fails at its ceiling forever.
//!  So the file is read by a child,
//! and the answer is waited for only until the ceiling:
//!  a child that has not answered by then is ended,
//!  and one the kernel does not let go is handed to a reaper that collects it whenever it does,
//!  the same bargain `stuck_children_reaped_when_the_kernel_lets_go.rs` makes for a command's child.
//!  The reader's ceiling is `Unread::File` to its caller,
//!  which a removal treats as a holder that may be alive,
//! never as one that is not.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const opened = openedFileInAChild({ process: '/proc/1234', fd: '3' }); // { device, inode }, or a refusal at the ceiling
//! ```

use super::open_file_of_a_descriptor::OpenedFile;
/// `CString` and `CStr` are the names a system call takes.
use std::ffi::{CStr, CString, OsStr};
/// `Path` is a borrowed filesystem path.
use std::path::Path;
/// `Duration` is the ceiling.
use std::time::Duration;

/// How long a child's answer is waited for.
///
/// Measured on 2026-11-09 on this host:
/// one `fork`,
/// one descriptor read and one pipe answer took a mean of
/// 0.5 ms over two hundred runs,
/// and the slowest of those two hundred took 2.1 ms.
/// The ceiling is a hundred and forty times the slowest of them,
/// so a loaded host still answers,
/// and a server that has stopped is abandoned rather than waited on.
pub const CHILD_READ_CEILING: Duration = Duration::from_millis(300);

/// The name of the thread that collects a child whose answer was abandoned.
pub const REAPER_THREAD_NAME: &str = "cli-git-descriptor-reaper";

/// What a child asks of the descriptor's path:
/// the attributes a read of the file behind the link gives.
pub type Ask = fn(&CStr) -> std::io::Result<OpenedFile>;

/// The children whose answer was abandoned and which no reaper has collected yet.
static WAITING_FOR_A_REAPER: std::sync::Mutex<Vec<libc::pid_t>> = std::sync::Mutex::new(Vec::new());

/// What:
///  The device and inode of the file the descriptor `fd` of the process whose table directory is `process`
///  holds open,
/// read in a child and waited for until `CHILD_READ_CEILING`.
/// Why:
///   The one read a mount this process cannot classify gets,
///  so that neither a stopped server nor an unreadable mount type keeps a commit from its answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// openedFileInAChild({ process: '/proc/1234', fd: '3' });
/// ```
pub fn opened_file_of_in_a_child(process: &Path, fd: &OsStr) -> std::io::Result<OpenedFile> {
    return read_in_a_child_with(process, fd, CHILD_READ_CEILING, attributes_through_the_link);
}

/// What:
///  The number a refused descriptor read carries,
///  in the shape the answer's own field holds it.
/// Why:
///   The child writes the refusal as one field of its answer,
///  and a read that failed without a kernel number has none,
///  which the field spells as zero.
fn refusal_number(number: i32) -> Option<u64> {
    return u64::try_from(number).ok();
}

/// What:
///  `opened_file_of_in_a_child` with the ceiling and the child's own work given,
///  so a control can hold a child past its ceiling without a stalled file system.
/// Why:
///   No test on this host can stop a file system server,
///  and the state that matters here is a child that has not answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readInAChildWith({ process, fd, ceiling, ask: sleepForAnHour });
/// ```
pub fn read_in_a_child_with(process: &Path, fd: &OsStr, ceiling: Duration, ask: Ask) -> std::io::Result<OpenedFile> {
    let name: CString = link_name(process, fd)?;
    // The child allocates nothing, so every name and buffer it needs is made here first.
    let mut ends: [libc::c_int; 2] = [0; 2];
    // SAFETY: `ends` holds two descriptors of room, and a refused call leaves it untouched.
    if unsafe { libc::pipe2(ends.as_mut_ptr(), libc::O_CLOEXEC) } < 0 {
        return Err(std::io::Error::last_os_error());
    }
    let read_end: libc::c_int = ends[0];
    let write_end: libc::c_int = ends[1];
    // SAFETY: the process is not asked to run anything between the fork and the child's own end, and the child calls
    //  only `ask`, `write` and `_exit`.
    let child: libc::pid_t = unsafe { libc::fork() };
    if child < 0 {
        let refused: std::io::Error = std::io::Error::last_os_error();
        close(read_end);
        close(write_end);
        return Err(refused);
    }
    if child == 0 {
        close(read_end);
        // A refusal carries the number the kernel gave it, and zero for a read that failed without one.
        let mut answer: [u64; 4] = [1, 0, 0, 0];
        match ask(name.as_c_str()) {
            Ok(opened) => answer = [0, opened.device, opened.inode, 0],
            Err(error) => {
                answer[3] = error.raw_os_error().and_then(refusal_number).unwrap_or(0);
            }
        }
        write_whole(write_end, &answer);
        // SAFETY: the child ends here, without a parent's exit handler or a flushed stream that another thread's lock
        //  could hold.
        unsafe { libc::_exit(0) };
    }
    close(write_end);
    let answer: Option<[u64; 4]> = wait_for_answer(read_end, ceiling);
    close(read_end);
    match answer {
        Some([0, device, inode, _refusal]) => {
            collected(child);
            return Ok(OpenedFile { device, inode });
        }
        Some([_, _, _, refusal]) => {
            collected(child);
            let number: i32 = i32::try_from(refusal).unwrap_or(libc::EIO);
            return Err(std::io::Error::from_raw_os_error(number));
        }
        None => {
            // The child is ended if it can be, and collected whenever the kernel lets go of it.
            kill(child);
            reap_soon(child);
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                format!(
                    "its file is on a mount this process cannot ask in its own name, and the child that asked did not answer within {} ms, so the file is not read",
                    ceiling.as_millis()
                ),
            ));
        }
    }
}

/// What:
///  The attributes of the file the name `name` leads to,
///  read in the child.
/// Why:
///   The mount's own type was not one this process may read,
///  so the walk happens where a server that stopped cannot hold the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// statx({ path: name, flags: AT_STATX_DONT_SYNC, mask: STATX_BASIC_STATS });
/// ```
fn attributes_through_the_link(name: &CStr) -> std::io::Result<OpenedFile> {
    let mut found: libc::statx = unsafe { std::mem::zeroed() };
    // SAFETY: the name outlives the call, `found` is a live `statx`, and the flag only says what may be answered from what
    //  the kernel already holds.
    let answer: libc::c_int = unsafe {
        libc::statx(libc::AT_FDCWD, name.as_ptr(), libc::AT_STATX_DONT_SYNC, libc::STATX_BASIC_STATS, &mut found)
    };
    if answer < 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(OpenedFile { device: libc::makedev(found.stx_dev_major, found.stx_dev_minor), inode: found.stx_ino });
}

/// What:
///  The refusal a descriptor path that holds a zero byte is answered with.
/// Why:
///   The path crosses into C as a C string,
///  whose only zero byte is the terminator,
///  so an interior one names no descriptor,
///  and both the guard and the conversion below answer with these words.
fn a_path_with_a_zero_byte() -> std::io::Error {
    return std::io::Error::new(std::io::ErrorKind::InvalidInput, "the descriptor's path holds a zero byte");
}

/// What:
///  The path of the descriptor's link as a name a system call takes.
/// Why:
///   It is built before the fork,
///  so the child never allocates in a process whose other threads it did not copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.from(`${process}/fd/${fd}\0`)
/// ```
fn link_name(process: &Path, fd: &OsStr) -> std::io::Result<CString> {
    use std::os::unix::ffi::OsStrExt as _;
    let mut name: Vec<u8> = process.as_os_str().as_bytes().to_vec();
    name.extend_from_slice(b"/fd/");
    name.extend_from_slice(fd.as_bytes());
    if name.contains(&0) {
        return Err(a_path_with_a_zero_byte());
    }
    return match CString::new(name) {
        Ok(path) => Ok(path),
        Err(_zero) => Err(a_path_with_a_zero_byte()),
    };
}

/// What:
///  What the child wrote,
/// or nothing when it did not write within the ceiling.
/// Why:
///   A pipe says when the child has answered,
///  so the wait is bounded by the clock even for a child the kernel will not let end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// poll([{ fd, events: POLLIN }], ceilingMs) > 0 ? readWhole(fd) : null;
/// ```
fn wait_for_answer(read_end: libc::c_int, ceiling: Duration) -> Option<[u64; 4]> {
    let milliseconds: i32 = i32::try_from(ceiling.as_millis()).unwrap_or(i32::MAX);
    let mut watched: libc::pollfd = libc::pollfd { fd: read_end, events: libc::POLLIN, revents: 0 };
    // SAFETY: `watched` is one live entry, which is what the count says.
    let ready: libc::c_int = unsafe { libc::poll(&raw mut watched, 1, milliseconds) };
    if ready <= 0 {
        return None;
    }
    let mut answer: [u64; 4] = [1, 0, 0, 0];
    let mut read: usize = 0;
    let mut bytes: [u8; 32] = [0; 32];
    while read < bytes.len() {
        // SAFETY: the buffer holds the room the call is given, from the offset it is read at.
        let taken: libc::ssize_t = unsafe {
            libc::read(read_end, bytes.as_mut_ptr().add(read).cast::<libc::c_void>(), bytes.len() - read)
        };
        if taken <= 0 {
            return None;
        }
        read += usize::try_from(taken).unwrap_or(bytes.len());
    }
    for (index, field) in answer.iter_mut().enumerate() {
        let mut number: [u8; 8] = [0; 8];
        number.copy_from_slice(&bytes[index * 8..(index + 1) * 8]);
        *field = u64::from_ne_bytes(number);
    }
    return Some(answer);
}

/// What:
///  The child's answer written whole.
/// Why:
///   A pipe takes part of a buffer when it is full,
///  and the answer is one word per field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeAll(fd, words);
/// ```
fn write_whole(write_end: libc::c_int, answer: &[u64; 4]) {
    let mut bytes: [u8; 32] = [0; 32];
    for (index, field) in answer.iter().enumerate() {
        bytes[index * 8..(index + 1) * 8].copy_from_slice(&field.to_ne_bytes());
    }
    let mut written: usize = 0;
    while written < bytes.len() {
        // SAFETY: the buffer holds the room the call is given, from the offset it is written at.
        let put: libc::ssize_t = unsafe {
            libc::write(write_end, bytes.as_ptr().add(written).cast::<libc::c_void>(), bytes.len() - written)
        };
        if put <= 0 {
            return;
        }
        written += usize::try_from(put).unwrap_or(bytes.len());
    }
}

/// What:
///  One descriptor closed,
/// and a refusal ignored.
/// Why:
///   The descriptors of the pipe belong to this call,
///  and a close that fails has nothing left to give back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { close(fd); } catch {}
/// ```
fn close(descriptor: libc::c_int) {
    // SAFETY: the descriptor is this call's own and is not used after this.
    let _closed = unsafe { libc::close(descriptor) };
}

/// What:
///  One signal sent to the child whose answer was abandoned,
/// and a refusal ignored.
/// Why:
///   The child is ended if it can be,
///  and one the kernel has not let go of is collected later instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { process.kill(pid, 'SIGKILL'); } catch {}
/// ```
fn kill(pid: libc::pid_t) {
    // SAFETY: the number is this process's own unreaped child, so it cannot name anything else.
    let _sent = unsafe { libc::kill(pid, libc::SIGKILL) };
}

/// What:
///  The child collected when it has already ended,
/// and handed to a reaper when it has not.
/// Why:
///   A child that answered is about to leave,
///  so collecting it here costs no wait,
///  and one that is somehow still there is collected by a thread instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (waitpidPid(pid, WNOHANG) === 0) reapSoon(pid);
/// ```
fn collected(pid: libc::pid_t) {
    let mut status: libc::c_int = 0;
    // SAFETY: the number is this process's own child, and `status` is a live variable of the right type.
    let answered: libc::pid_t = unsafe { libc::waitpid(pid, &raw mut status, libc::WNOHANG) };
    if answered == 0 {
        reap_soon(pid);
    }
}

/// What:
///  A child whose answer was abandoned handed to a thread that collects it whenever the kernel lets go,
/// or kept for the next reaper when no thread can start.
/// Why:
///   The caller must not wait for a child that is asleep in the kernel,
///  and a process nobody waits for stays a zombie while this process lives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reapSoon(pid);
/// ```
fn reap_soon(pid: libc::pid_t) {
    let started = std::thread::Builder::new().name(String::from(REAPER_THREAD_NAME)).spawn(move || {
        collect(pid);
        drain_the_registry();
    });
    if started.is_err() {
        keep(pid);
    }
    return;
}

/// What:
///  One child collected,
/// waiting however long the kernel takes.
/// Why:
///   It is the reaper's whole work,
///  and it runs on a thread that has nothing else to do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// waitpid(pid, 0);
/// ```
fn collect(pid: libc::pid_t) {
    let mut status: libc::c_int = 0;
    // SAFETY: the number is this process's own child, and `status` is a live variable of the right type.
    let _collected = unsafe { libc::waitpid(pid, &raw mut status, 0) };
}

/// What:
///  A child kept for a reaper that the next one drains.
/// Why:
///   A lost child is a zombie for as long as this process lives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// waitingForAReaper.push(pid);
/// ```
fn keep(pid: libc::pid_t) {
    let Ok(mut waiting) = WAITING_FOR_A_REAPER.lock() else {
        return;
    };
    waiting.push(pid);
}

/// What:
///  Every child a refused start left behind,
/// collected by the reaper that is running now.
/// Why:
///   The list covers one push and one take,
///  which is the smallest unit a lock may cover.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const pid of waitingForAReaper.splice(0)) waitpid(pid, 0);
/// ```
fn drain_the_registry() {
    let kept: Vec<libc::pid_t> = match WAITING_FOR_A_REAPER.lock() {
        Ok(mut waiting) => std::mem::take(&mut *waiting),
        Err(_poisoned) => return,
    };
    for pid in kept {
        collect(pid);
    }
}

/// Controls on the bounded child read stay out of the release executable.
#[cfg(test)]
#[path = "open_file_of_a_descriptor_in_a_child_tests.rs"]
mod tests;
