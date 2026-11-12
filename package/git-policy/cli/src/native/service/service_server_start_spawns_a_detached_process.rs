//! What:
//!  Starts the commit service's program as a process that outlives the command,
//! the terminal and the shell that started it:
//! a private session,
//! no standard input,
//! its output in a file.
//! Why:
//!   A command that exits must leave the service running,
//! and a terminal that closes or a shell that is killed must not take it along.
//! `setsid()` gives one new session and process group,
//! so the service has no controlling terminal,
//! no `SIGHUP` from one,
//! and can be signalled as a group when it has to be replaced
//! (Bazel's `daemonize` does the same: `setsid` then `execv`,
//! `src/main/tools/daemonize.cc`).
//! Two processes are forked:
//! the middle one only forks the service and exits,
//! so the command waits for a known child and reaps it,
//! and the service itself is nobody's child that anyone waits for.
//! Nothing is allocated between a fork and the `execv`:
//! the program's name,
//! its arguments and the two descriptors are made whole before the first fork,
//! because the wrapper has threads and a fork of a threaded process may only call
//! async-signal-safe functions until it execs.
//! The service inherits the command's environment:
//! the service's own children get the deliberate overlay
//! (`child_environment.rs`),
//! and the wrapper passes what the service reads in its messages.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! spawnDetached({ program, args, log }); // fork, fork, setsid, dup2, execv
//! ```

use std::ffi::{CString, OsStr, OsString};
use std::os::fd::OwnedFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

/// How long the middle process is waited for before the start is called done anyway.
/// It forks and exits, which is a few system calls;
/// a bound keeps a stopped machine from holding a commit at a wait nobody watches.
pub const MIDDLE_PROCESS_WAIT: std::time::Duration = std::time::Duration::from_secs(2);

/// What a detached start needs.
#[derive(Clone, Debug)]
pub struct Detached {
    /// The program to run.
    pub program: PathBuf,
    /// Its arguments, without the program's own name.
    pub arguments: Vec<OsString>,
    /// The file its standard output and standard error are appended to.
    pub log: PathBuf,
}

/// What:
///  Starts `detached` and returns once the middle process is gone.
/// Why:
///   The caller waits for a greeting on the socket;
/// this function only makes the process exist,
/// so it never reads or writes the service's own files.
/// A refusal to open the log or the null device is the caller's to report:
/// that is a start that cannot happen,
/// unlike a service that starts and then exits,
/// which leaves its reason in the log.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawnDetached(detached) // throws only when the fds cannot be opened
/// ```
pub fn spawn(detached: &Detached) -> std::io::Result<Option<i32>> {
    let nothing: OwnedFd = std::fs::OpenOptions::new().read(true).open("/dev/null")?.into();
    let log: OwnedFd = std::fs::OpenOptions::new().create(true).append(true).mode(0o600).open(&detached.log)?.into();
    let program: CString = CString::new(detached.program.as_os_str().as_bytes()).map_err(|_nul| std::io::Error::other("the service program's path holds a zero byte"))?;
    let mut arguments: Vec<CString> = Vec::with_capacity(detached.arguments.len());
    for argument in &detached.arguments {
        arguments.push(CString::new(argument.as_os_str().as_bytes()).map_err(|_nul| std::io::Error::other("an argument of the service holds a zero byte"))?);
    }
    let mut pointers: Vec<*const libc::c_char> = Vec::with_capacity(arguments.len() + 2);
    pointers.push(program.as_ptr());
    for argument in &arguments {
        pointers.push(argument.as_ptr());
    }
    pointers.push(std::ptr::null());
    let nothing_fd: i32 = std::os::fd::AsRawFd::as_raw_fd(&nothing);
    let log_fd: i32 = std::os::fd::AsRawFd::as_raw_fd(&log);
    return spawn_twice(program_ptr(program.as_ptr()), pointers, nothing_fd, log_fd);
}

/// The program pointer as an integer, because a pointer is not `Send` and the fork's
/// child reads it after the split.
fn program_ptr(program: *const libc::c_char) -> usize {
    return program as usize;
}

/// What:
///  The two forks and the middle process's wait,
/// which is the part that must not allocate.
/// Why:
///   Keeping the unsafe part in one small function keeps every later call in the child
/// provably async-signal-safe.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const middle = fork(); if (middle === 0) { const server = fork(); if (server === 0) become(program, argv, nullFd, logFd); _exit(0); } waitpid(middle);
/// ```
fn spawn_twice(program: usize, pointers: Vec<*const libc::c_char>, nothing_fd: i32, log_fd: i32) -> std::io::Result<Option<i32>> {
    // The number of the server, written by the middle process so the caller can name it: a pipe's two ends, closed on exec.
    let mut ends: [libc::c_int; 2] = [0, 0];
    // SAFETY: `pipe2` writes two descriptors it creates, and the array has room for both.
    let made: libc::c_int = unsafe { libc::pipe2(ends.as_mut_ptr(), libc::O_CLOEXEC) };
    if made != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let (reading, writing): (libc::c_int, libc::c_int) = (ends[0], ends[1]);
    // SAFETY: nothing between this fork and `_exit` or `execv` allocates, takes a lock or touches state the parent owns.
    let middle: libc::pid_t = unsafe { libc::fork() };
    if middle < 0 {
        return Err(std::io::Error::last_os_error());
    }
    if middle == 0 {
        // SAFETY: as above; the middle process only forks, writes one number, exits, or becomes the server.
        unsafe {
            libc::close(reading);
            let server: libc::pid_t = libc::fork();
            if server < 0 {
                libc::close(writing);
                libc::_exit(1);
            }
            if server == 0 {
                become_the_server(program, pointers.as_ptr(), nothing_fd, log_fd);
            }
            let mut text: [u8; 24] = [0; 24];
            let digits: &[u8] = decimal_of(server, &mut text);
            let _written: libc::ssize_t = libc::write(writing, digits.as_ptr().cast(), digits.len());
            libc::close(writing);
            libc::_exit(0);
        }
    }
    // SAFETY: the write end belongs to the child from here on.
    unsafe { libc::close(writing) };
    let waited: std::io::Result<()> = wait_for_the_middle_process(middle);
    let pid: Option<i32> = read_the_number(reading);
    // SAFETY: the read end belongs to this process and is closed once.
    unsafe { libc::close(reading) };
    waited?;
    return Ok(pid);
}

/// The decimal text of a process number,
/// written into `buffer` from its end,
/// which allocates nothing.
fn decimal_of(number: libc::pid_t, buffer: &mut [u8; 24]) -> &[u8] {
    let mut at: usize = buffer.len();
    let mut left: u32 = u32::try_from(number).unwrap_or(0);
    if left == 0 {
        at -= 1;
        buffer[at] = b'0';
    }
    while left > 0 {
        at -= 1;
        buffer[at] = b'0' + u8::try_from(left % 10).unwrap_or(0);
        left /= 10;
    }
    return &buffer[at..];
}

/// What:
///  The number the middle process wrote,
///  read bounded so a middle process that died before writing costs no wait.
/// Why:
///   The caller names the server in its lines and a control reads its session from the number;
/// a read that found nothing is no number and never a failure of the start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readTheNumber(fd) // poll 2 s, read at most 24 bytes, parse
/// ```
fn read_the_number(reading: libc::c_int) -> Option<i32> {
    let deadline: std::time::Instant = std::time::Instant::now() + MIDDLE_PROCESS_WAIT;
    loop {
        let mut waiting: libc::pollfd = libc::pollfd { fd: reading, events: libc::POLLIN, revents: 0 };
        // SAFETY: one descriptor this process owns is asked about, with its own structure.
        let ready: libc::c_int = unsafe { libc::poll(&raw mut waiting, 1, 0) };
        if ready > 0 {
            let mut text: [u8; 24] = [0; 24];
            // SAFETY: the buffer has room for the bytes the child writes.
            let read: libc::ssize_t = unsafe { libc::read(reading, text.as_mut_ptr().cast(), text.len()) };
            if read <= 0 {
                return None;
            }
            let text: &str = std::str::from_utf8(&text[..usize::try_from(read).unwrap_or(0)]).ok()?;
            return text.trim().parse().ok();
        }
        if std::time::Instant::now() >= deadline {
            return None;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

/// What:
///  Becomes the server:
/// a session of its own,
/// `SIGHUP` ignored,
/// standard input from the null device,
/// standard output and error to the log,
/// then the program.
/// Why:
///   Every call here is async-signal-safe;
/// a failure is the child's death with status 127 and the caller's waiting for a greeting
/// names the log.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// setsid(); signal(SIGHUP, SIG_IGN); dup2(nullFd, 0); dup2(logFd, 1); dup2(logFd, 2); execv(program, argv); _exit(127);
/// ```
fn become_the_server(program: usize, pointers: *const *const libc::c_char, nothing_fd: i32, log_fd: i32) -> ! {
    // SAFETY: before the `execv`, only async-signal-safe calls run, and every argument was made whole before the fork.
    unsafe {
        let _session: libc::pid_t = libc::setsid();
        libc::signal(libc::SIGHUP, libc::SIG_IGN);
        libc::dup2(nothing_fd, libc::STDIN_FILENO);
        libc::dup2(log_fd, libc::STDOUT_FILENO);
        libc::dup2(log_fd, libc::STDERR_FILENO);
        if nothing_fd > libc::STDERR_FILENO {
            libc::close(nothing_fd);
        }
        if log_fd > libc::STDERR_FILENO {
            libc::close(log_fd);
        }
        libc::execv(program as *const libc::c_char, pointers);
        libc::_exit(127);
    }
}

/// What:
///  Waits for the middle process,
/// bounded,
/// and gives what it said.
/// Why:
///   Reaping it while the command is alive leaves no zombie in the command;
/// the bound is a stopped machine's,
/// never an ordinary wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const deadline = now + MIDDLE_PROCESS_WAIT; while (now < deadline) { if (waitpid(middle, WNOHANG).done) return; sleep(1ms); }
/// ```
fn wait_for_the_middle_process(middle: libc::pid_t) -> std::io::Result<()> {
    let deadline: std::time::Instant = std::time::Instant::now() + MIDDLE_PROCESS_WAIT;
    let mut status: libc::c_int = 0;
    loop {
        // SAFETY: the only child awaited is the middle process, by its own number.
        let waited: libc::pid_t = unsafe { libc::waitpid(middle, &raw mut status, libc::WNOHANG) };
        if waited == middle {
            return Ok(());
        }
        if waited < 0 {
            let error: std::io::Error = std::io::Error::last_os_error();
            // A signal that interrupted the wait is asked again; anything else is the caller's to know about.
            if error.kind() != std::io::ErrorKind::Interrupted {
                return Err(error);
            }
        }
        if std::time::Instant::now() >= deadline {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

/// The program's path as the text an argument list keeps.
pub fn argument(path: &Path) -> OsString {
    return path.as_os_str().to_os_string();
}

/// One text as an argument.
pub fn text_argument(text: &str) -> OsString {
    return OsStr::new(text).to_os_string();
}

/// Controls of the detached start stay out of the release executable.
#[cfg(test)]
#[path = "service_server_start_spawns_a_detached_process_tests.rs"]
mod tests;
