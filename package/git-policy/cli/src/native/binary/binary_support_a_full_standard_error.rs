//! What:
//!  A standard error nobody drains,
//!  the bounded wait for a command given one,
//!  and the count of bytes still waiting in it.
//! Why:
//!  A write to a pipe whose reader has stopped draining waits in the kernel until it resumes,
//!  so this stream is what shows whether a note of a command is written on the command's own thread
//!  (`foundation/standard_error_off_the_calling_thread.rs`;
//!  issue 6421).
//!  The shared reading of a command's output drains its pipes,
//!  so a control that needs this stream hands the command one of its own and waits for it here.
//!  One harness for every control that proves a note off the command's thread,
//!  because a second copy of the pipe arithmetic would be a second thing to get wrong.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pipe = fullPipeNobodyDrains(); const run = spawn(wrapper, words, { stderr: pipe.write });
//! expect(await waitBounded(run)).toBe(0); expect(pendingBytes(pipe)).toBe(pipe.capacity);
//! ```

use super::support::{
    close_on_exec_pipe,
    spawn_owned,
};
use std::os::fd::{
    AsRawFd,
    FromRawFd,
    OwnedFd,
    RawFd,
};
use std::process::{
    Child,
    ExitStatus,
};
use std::time::Duration;

/// How long a command this harness ended may take to be reaped,
/// in seconds' worth.
/// Why:
///  A command ended at its bound is reaped before the control reports,
///  so the ending it was killed with is not left to the test process's own cleanup.
pub const REAPING: Duration = Duration::from_secs(5);

/// The block one fill of a pipe writes,
/// in bytes.
///  It is the granularity of a pipe page,
///  so a short write is the pipe's room running out and nothing else.
const FILL_BLOCK: usize = 4096;

/// What:
///  A pipe full of bytes whose read end a control keeps open and never reads.
/// Why:
///  A command's standard error is the write end of this pipe,
///  so a writer waits there for a reader that never comes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PipeNobodyDrains = { readEnd: number; capacity: number };
/// ```
pub struct PipeNobodyDrains {
    /// The read end,
    /// kept open so the write end stays a pipe with a reader that never reads,
    /// and read from by a control never.
    pub read_end: OwnedFd,
    /// How many bytes the pipe holds when it is full.
    pub capacity: usize,
}

/// The file status flags of one descriptor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const descriptorFlags = (fd: number): number => fcntl(fd, F_GETFL);
/// ```
fn descriptor_flags(descriptor: RawFd) -> libc::c_int {
    // SAFETY: `F_GETFL` reads the descriptor's flags and writes nothing into this process.
    return unsafe { libc::fcntl(descriptor, libc::F_GETFL) };
}

/// What:
///  Whether a write to the descriptor waits for room or is refused when there is none.
/// Why:
///  The fill needs a write that answers "no room" instead of waiting for a reader,
///  and the command's own write needs the blocking pipe this call puts back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const waitForRoom = (fd: number, waiting: boolean) => fcntl(fd, F_SETFL, waiting ? flags & ~O_NONBLOCK : flags | O_NONBLOCK);
/// ```
fn wait_for_room(
    descriptor: RawFd,
    waiting: bool,
) {
    let flags: libc::c_int = descriptor_flags(descriptor);
    let wanted: libc::c_int = if waiting {
        flags & !libc::O_NONBLOCK
    } else {
        flags | libc::O_NONBLOCK
    };
    // SAFETY: `F_SETFL` sets the flags of an open descriptor of this process.
    assert_eq!(
        unsafe { libc::fcntl(descriptor, libc::F_SETFL, wanted) },
        0,
        "the descriptor's flags"
    );
}

/// How many bytes one write took,
/// and nothing when it was refused for want of room.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const oneWrite = (fd: number, bytes: Uint8Array): number => { const taken = write(fd, bytes); return taken < 0 ? 0 : taken };
/// ```
fn one_write(
    descriptor: RawFd,
    bytes: &[u8],
) -> usize {
    // SAFETY: `bytes` is read for its own length, and the descriptor is open.
    let written: libc::ssize_t =
        unsafe { libc::write(descriptor, bytes.as_ptr().cast(), bytes.len()) };
    return usize::try_from(written).unwrap_or_default();
}

/// The pipe's own capacity,
/// in bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pipeCapacity = (fd: number): number => fcntl(fd, F_GETPIPE_SZ);
/// ```
fn pipe_capacity(descriptor: RawFd) -> usize {
    // SAFETY: `F_GETPIPE_SZ` reads the pipe's capacity and writes nothing into this process.
    let size: libc::c_int = unsafe { libc::fcntl(descriptor, libc::F_GETPIPE_SZ) };
    return usize::try_from(size).unwrap_or_default();
}

/// How many bytes are waiting in a pipe,
/// read without taking any.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pendingBytes = (fd: number): number => ioctl(fd, FIONREAD);
/// ```
pub fn pending_bytes(descriptor: RawFd) -> usize {
    let mut waiting: libc::c_int = 0;
    // SAFETY: `FIONREAD` writes one integer into the buffer it is given, and the descriptor is open.
    assert_eq!(
        unsafe { libc::ioctl(descriptor, libc::FIONREAD, &mut waiting) },
        0,
        "how many bytes the pipe holds"
    );
    return usize::try_from(waiting).unwrap_or_default();
}

/// What:
///  A pipe filled to its whole capacity,
///  with the write end handed back for the command's standard error,
///  and the read end kept open and never read.
/// Why:
///  A pipe full to its capacity refuses every later byte,
///  so a note written on the command's own thread waits for a reader that never comes,
///  which is the shape the issue recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const aFullPipe = () => { const [read, write] = pipe(); setNonBlocking(write); while (write(write, block) === block.length) {} setBlocking(write); return [{ readEnd: read, capacity: pipeCapacity(write) }, write]; };
/// ```
pub fn a_full_pipe() -> (PipeNobodyDrains, OwnedFd) {
    let ends: [libc::c_int; 2] = close_on_exec_pipe();
    // SAFETY: `pipe2` made both descriptors here, and neither number is used again.
    let (read_end, write_end): (OwnedFd, OwnedFd) =
        unsafe { (OwnedFd::from_raw_fd(ends[0]), OwnedFd::from_raw_fd(ends[1])) };
    let capacity: usize = pipe_capacity(write_end.as_raw_fd());
    // What: The fill asks for room instead of waiting for it.
    // Why:  A blocking write to a full pipe waits for a reader, and this harness is that wait's opposite.
    wait_for_room(write_end.as_raw_fd(), false);
    let block: [u8; FILL_BLOCK] = [b'p'; FILL_BLOCK];
    // What: Write whole blocks until the pipe refuses one.
    // Why:  The last accepted block fills whatever room was left, so the pipe is then as full as it can be.
    while one_write(write_end.as_raw_fd(), block.as_slice()) == block.len() {}
    // What: Put the blocking pipe back before the command is given it.
    // Why:  The flags belong to the pipe itself, so the command's standard error would otherwise refuse
    //       a write with a signal rather than wait in the kernel, which is not the shape under test.
    wait_for_room(write_end.as_raw_fd(), true);
    let full: PipeNobodyDrains = PipeNobodyDrains {
        read_end,
        capacity,
    };
    assert_eq!(
        pending_bytes(full.read_end.as_raw_fd()),
        capacity,
        "the pipe starts full"
    );
    // The write end goes to the command; the read end stays here, open and unread.
    return (full, write_end);
}

/// What:
///  Waits for `child` no longer than `within`,
///  ending the process group it leads at that bound,
///  and fails with `what` and what it waited for.
/// Why:
///  The shared bounded wait reads a command's output through two pipes it made,
///  and this harness hands the command a standard error of its own instead,
///  so only the status is read here.
///  The group is the one `spawn_owned` made for this child,
///  so the signal names only what the control started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const waitBounded = (child, within, what) => { const ended = waitInAThread(child); try { return ended.recv(within) } catch { kill(-child.pid, SIGKILL); throw new Error(`${what} waited ${within}`) } };
/// ```
pub fn wait_bounded(
    mut child: Child,
    within: Duration,
    what: &str,
) -> ExitStatus {
    let id: u32 = child.id();
    let group: libc::pid_t = libc::pid_t::try_from(id).expect("a process ID fits the pid type");
    let (told, ended) = std::sync::mpsc::channel::<std::io::Result<ExitStatus>>();
    // What: The wait itself runs on a thread of its own.
    // Why:  A blocking wait for the exit cannot be given a bound, and this control must fail at one.
    let waited: std::thread::JoinHandle<()> = std::thread::Builder::new()
        .name(String::from("stalled-standard-error-wait"))
        .spawn(move || {
            let _sent = told.send(child.wait());
        })
        .expect("the waiter of the command starts");
    let first: std::io::Result<ExitStatus> = match ended.recv_timeout(within) {
        Ok(answer) => answer,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            // SAFETY: the group number is the one `spawn_owned` made for this child, and the signal goes to
            //  that group and nothing else.
            let _signalled = unsafe { libc::kill(-group, libc::SIGKILL) };
            let reaped: std::io::Result<ExitStatus> =
                ended.recv_timeout(REAPING).unwrap_or_else(|_| {
                    panic!("{what} was ended at {within:?} and not reaped within {REAPING:?}")
                });
            let _status: ExitStatus = reaped
                .unwrap_or_else(|error| panic!("the ending of {what} cannot be read: {error}"));
            let _joined = waited.join();
            panic!(
                "{what} did not return within {within:?}: a note of the command waited for a reader that never read its standard error"
            );
        }
        // The waiter sends before it ends, so a disconnect here is a defect of this control.
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            panic!("the waiter of {what} ended without its ending")
        }
    };
    let _joined = waited.join();
    return first.unwrap_or_else(|error| panic!("the ending of {what} cannot be read: {error}"));
}

/// What:
///  Starts `command` with a full pipe nobody drains as its standard error,
///  and answers the command and the pipe that proves nobody read a byte of it.
/// Why:
///  Every control of this harness does the same three steps:
///  null standard input and output,
///  the pipe as standard error,
///  and a child this process owns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const startedOnAFullPipe = (command) => { const [pipe, write] = aFullPipe(); command.stdin('ignore').stdout('ignore').stderr(write); return { child: spawnOwned(command), pipe } };
/// ```
pub fn started_on_a_full_pipe(command: &mut std::process::Command) -> (Child, PipeNobodyDrains) {
    let (pipe, write_end): (PipeNobodyDrains, OwnedFd) = a_full_pipe();
    command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::from(write_end));
    let child: Child = spawn_owned(command).expect("the command starts");
    return (child, pipe);
}

/// What:
///  Starts `command` with a pipe as its standard error whose read end this control holds open and
///  whose reader never appears, because the read end is closed here at once.
/// Why:
///   A person's terminal can go away while a command runs,
///  and every byte the command writes to standard error is then refused with `EPIPE`.
///  The read ends the shared reading makes are drained, so a control that needs a reader that is gone
///  hands the command this pipe instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const startedOnAClosedPipe = (command) => { command.stdin('ignore').stdout('ignore').stderr('pipe'); const child = spawnOwned(command); child.stderr.destroy(); return child };
/// ```
pub fn started_on_a_closed_pipe(command: &mut std::process::Command) -> Child {
    command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped());
    let mut child: Child = spawn_owned(command).expect("the command starts");
    // What: The read end is closed here, in the control's own process.
    // Why:  Every write the command makes to its standard error is then refused, as a program whose reader ended
    //       has it refused, and no byte of it can ever be read.
    drop(child.stderr.take());
    return child;
}
