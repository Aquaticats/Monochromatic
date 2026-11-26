//! Controls for the notes of a commit whose standard error nobody drains (issue 6421):
//!  the command returns and the commit lands,
//!  and the lines it could not hand over are given up rather than waited for.
//! Why:
//!  A write to a pipe whose reader has stopped draining waits in the kernel until it resumes,
//!  and `std::io::stderr()` is one handle behind one lock for the whole process,
//!  so a note written on the commit's own thread can hold the commit for as long as the reader
//!  does not read.
//!  The owner's rule is "nothing should ever wait forever or indefinitely".
//! Every control runs the built executable on a disposable repository with the host's Git,
//!  with the read end of a full pipe kept open and never read.
//! The pipe is filled to its whole capacity, so not one byte of any line fits:
//!  a line the wrapper writes before the fix waits there and the commit never returns.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pipe = fullPipeNobodyDrains(); const run = spawn(wrapper, commit, { stderr: pipe.write });
//! expect(await waitBounded(run)).toBe(0); expect(pendingBytes(pipe)).toBe(pipe.capacity);
//! ```

use super::commit::{
    remove_settled,
    wrapped_commit,
};
use super::support::{
    Fixture,
    close_on_exec_pipe,
    fixture,
    git,
    repository,
    spawn_owned,
};
use std::ffi::OsStr;
use std::os::fd::{
    AsRawFd,
    FromRawFd,
    OwnedFd,
    RawFd,
};
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Child,
    Command,
    ExitStatus,
    Stdio,
};
use std::time::Duration;

/// How long this control waits for the commit before it fails, in seconds.
/// Why:
///  The commit's own line ends it at `COMMIT_BOUND_SECONDS` (120 s),
///  so this bound is that one plus the grace in which a `SIGKILL`ed command is reaped;
///  a control that reached it fails with what it waited for rather than hanging the run.
const WAIT_SECONDS: u64 = 150;

/// How long a command this control ended may take to be reaped.
const REAPING: Duration = Duration::from_secs(5);

/// The block one fill of a pipe writes, in bytes:
///  the granularity of a pipe page,
///  so a short write is the pipe's room running out and nothing else.
const FILL_BLOCK: usize = 4096;

/// What:
///  A pipe full of bytes whose read end this control keeps open and never reads.
/// Why:
///  The commit's standard error is the write end of this pipe,
///  so a writer waits there for a reader that never comes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PipeNobodyDrains = { readEnd: number; capacity: number };
/// ```
struct PipeNobodyDrains {
    /// The read end,
    /// kept open so the write end stays a pipe with a reader that never reads,
    /// and read from by this control never.
    read_end: OwnedFd,
    /// How many bytes the pipe holds when it is full.
    capacity: usize,
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
///  and the commit's own write needs the blocking pipe this call puts back.
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

/// The pipe's own capacity, in bytes.
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
fn pending_bytes(descriptor: RawFd) -> usize {
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
///  A pipe filled to its whole capacity, with the write end handed back for the command's standard
///  error and the read end kept open and never read.
/// Why:
///  A pipe full to its capacity refuses every later byte,
///  so the wrapper's first note waits for a reader that never comes,
///  which is the shape the issue recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const aFullPipe = () => { const [read, write] = pipe(); setNonBlocking(write); while (write(write, block) === block.length) {} setBlocking(write); return [{ readEnd: read, capacity: pipeCapacity(write) }, write]; };
/// ```
fn a_full_pipe() -> (PipeNobodyDrains, OwnedFd) {
    let ends: [libc::c_int; 2] = close_on_exec_pipe();
    // SAFETY: `pipe2` made both descriptors here, and neither number is used again.
    let (read_end, write_end): (OwnedFd, OwnedFd) =
        unsafe { (OwnedFd::from_raw_fd(ends[0]), OwnedFd::from_raw_fd(ends[1])) };
    let capacity: usize = pipe_capacity(write_end.as_raw_fd());
    // What: The fill asks for room instead of waiting for it.
    // Why:  A blocking write to a full pipe waits for a reader, and this control is that wait's opposite.
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
///  This control cannot use the shared bounded wait:
///  that one reads a command's output through two pipes it made,
///  and this control hands the command a standard error of its own instead.
///  The group is the one `spawn_owned` made for this child, so the signal names only what this
///  control started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const waitBounded = (child, within, what) => { const ended = waitInAThread(child); try { return ended.recv(within) } catch { kill(-child.pid, SIGKILL); throw new Error(`${what} waited ${within}`) } };
/// ```
fn wait_bounded(
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
                "{what} did not return within {within:?}: a note of the commit waited for a reader that never read its standard error"
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
///  The text of one Git query in a repository,
///  run with real Git directly.
/// Why:
///  A control reads what landed from the repository itself, never from the wrapper's own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gitText = (fixture, repo, args) => exec(git, ['-C', repo, ...args]).stdout.trim();
/// ```
fn git_text(
    fixture: &Fixture,
    repository: &Path,
    arguments: &[&str],
) -> String {
    let output: std::process::Output = git(fixture, repository, arguments);
    return String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
}

/// What:
///  A repository whose staged edit the pre-forward policies correct,
///  so the commit's own path writes a line to standard error.
/// Why:
///   Every note of the commit path must be given up when the reader is gone,
///  and a control with no note would prove only that a silent command is not held.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const aRepositoryWithAnEditThePoliciesCorrect = (fixture) => { ... write('a.txt', 'one\ntwo'); git add -- a.txt }
/// ```
fn a_repository_with_an_edit_the_policies_correct(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"one\n").expect("the tracked file");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(
        fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=base"],
    );
    // The edit has no final newline, which the `final-newline` policy corrects and says so on standard
    //  error once the commit has landed.
    std::fs::write(repo.join("a.txt"), b"one\ntwo").expect("an edit without the final newline");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    return repo;
}

/// What:
///  A commit whose standard error is a full pipe nobody drains returns,
///  and the commit it was asked for is on the branch.
/// Why:
///   The notes of the commit path are lines a person can live without,
///  and the exit status already carries the result,
///  so no reader that stopped reading may hold the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('lands', () => { const pipe = fullPipe(); expect(waitBounded(spawn(commit, { stderr: pipe.write }))).toBe(0); expect(pendingBytes(pipe)).toBe(pipe.capacity) })
/// ```
#[test]
fn a_commit_lands_while_nobody_drains_its_full_standard_error() {
    let fixture: Fixture = fixture("commit-stalled-standard-error");
    let repo: PathBuf = a_repository_with_an_edit_the_policies_correct(&fixture);
    let (pipe, write_end): (PipeNobodyDrains, OwnedFd) = a_full_pipe();
    let mut command: Command = wrapped_commit(
        &fixture,
        repo.as_path(),
        &["commit", "--message=stalled", "a.txt"],
    );
    // What: The command's standard error is the full pipe, and its other two streams go nowhere.
    // Why:  A captured pipe of this control's own would be drained by the wait, which is the reader
    //       this control removes.
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(write_end));
    let child: Child = spawn_owned(&mut command).expect("the commit starts");
    let status: ExitStatus = wait_bounded(child, Duration::from_secs(WAIT_SECONDS), "the commit");
    assert_eq!(
        status.code(),
        Some(0),
        "the commit returned without landing: {status:?}"
    );
    assert_eq!(
        git_text(&fixture, repo.as_path(), &["log", "-1", "--format=%s"]).trim(),
        "stalled",
        "the commit landed"
    );
    assert_eq!(
        pending_bytes(pipe.read_end.as_raw_fd()),
        pipe.capacity,
        "the pipe was read by nobody, so every byte this control put in it is still there"
    );
    remove_settled(&fixture);
}
