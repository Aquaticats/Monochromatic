//! What:
//!  The ways the controls start real Git in a fixture:
//! one that answers its output,
//! one that requires it to have succeeded,
//! and one that keeps a bound on the query and answers, for the last status the wait returned,
//! whether that status was the signal this helper sends at the bound.
//! Why:
//!  Every control that needs a repository,
//!  a commit or an answer from Git starts it the same way,
//! with no system or global
//!  configuration and a fixed identity,
//!  so nothing of the account's own configuration reaches a fixture.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! spawnSync(REAL_GIT, args, { cwd, env: { PATH: '/usr/bin:/bin', GIT_CONFIG_NOSYSTEM: '1', ... } })
//! ```

/// The one path of the host's Git that the binary tests share.
use super::REAL_GIT;
/// `OsStr`/`OsString` are borrowed/owned operating-system text (siblings `&str`/`String` must be UTF-8).
use std::ffi::{
    OsStr,
    OsString,
};
/// `Read`/`Write` fill the standard input of a query and drain its pipes.
use std::io::{
    Read,
    Write,
};
/// `AsRawFd` is the descriptor number behind an open pipe end,
/// `FromRawFd` is the one way to take ownership of a number the kernel handed back,
/// and `OwnedFd` is that ownership: it closes the number when it goes out of scope.
use std::os::unix::io::{
    AsRawFd,
    FromRawFd,
    OwnedFd,
};
/// `process_group` gives the query a process group of its own, which the bound ends as a whole.
use std::os::unix::process::CommandExt as _;
/// `signal` reads which signal ended the query, which is how the ending is known.
use std::os::unix::process::ExitStatusExt as _;
/// `Path` is a borrowed filesystem path of raw OS bytes.
use std::path::Path;
/// The command builder, the child's pipes and the finished child every fixture Git run is.
use std::process::{
    Command,
    Output,
    Stdio,
};

/// Run real Git in a fixture with no system or global configuration and a fixed identity.
pub(crate) fn git_output<S: AsRef<OsStr>>(
    directory: &Path,
    arguments: &[S],
) -> Output {
    return Command::new(REAL_GIT)
        .current_dir(directory)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .args(arguments)
        .output()
        .expect("native Git fixture command");
}

/// Run real Git in a fixture and require success.
pub(crate) fn git<S: AsRef<OsStr>>(
    directory: &Path,
    arguments: &[S],
) -> Output {
    let output: Output = git_output(directory, arguments);
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        arguments.iter().map(AsRef::as_ref).collect::<Vec<&OsStr>>(),
        String::from_utf8_lossy(&output.stderr)
    );
    return output;
}

/// How long one real-Git query of a control may run before this helper ends it.
/// Why sixty:
///  The same bound the binary-level controls give one fixture Git run
/// (`NATIVE_GIT_BOUND_SECONDS`), and for the same measurement:
/// the longest disk stall one `fsync` of a trivial command showed on this host is 4.74 s
/// (`doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md`),
/// so sixty is more than ten times it.
pub(crate) const FIXTURE_GIT_BOUND: std::time::Duration = std::time::Duration::from_secs(60);

/// How long the answer gives one of the query's pipes to reach its end once the query is gone,
/// before it says that something outside the query still holds that pipe.
/// Why 250 ms:
///  the query's own process is gone when this window opens,
/// and the kernel reports a pipe's end the instant its last writing end closes,
/// so this window is spent only by a holder that is not the query:
/// a descendant that inherited the pipe, or a program that was forked with it open.
///  The window is asked of the pipe itself (`has_ended_within`) and not of a thread of this process,
/// so a host that wakes such a thread late cannot spend it:
/// measured 2026-11-27 on this host at a load average of 86.0 to 142.5,
/// the thread that read a query's standard output handed it over up to 110.4 ms after the pipe's end,
/// 2.3 times inside this window,
/// which is how close that window came to failing for a reason that was never about the query.
pub(crate) const PIPE_GRACE: std::time::Duration = std::time::Duration::from_millis(250);

/// How long the reader of one of the query's pipes may take to hand over what it read,
/// once the kernel says that pipe has no writing end left.
/// Why sixty seconds:
///  this bound only stops a hang:
/// the kernel has already said that every byte the query wrote is in the pipe,
/// so what is left is one thread's wakes, which no honest host holds for a minute.
///  It is 543 times the worst hand-over measured on this host at a load average of 86.0 to 142.5
/// (110.4 ms), and the same figure this crate's run watchdog allows a working run to show nothing at all
/// (`STILL_WORKING_WINDOW`, which rests on a measured 8.0 s of silence).
pub(crate) const REPORTING_THE_END: std::time::Duration = std::time::Duration::from_secs(60);

/// How often the wait looks whether the query ended.
/// Why one millisecond:
///  this helper owns the wait, so no second thread may collect the query while the bound's signal is on its way;
/// the standard library has no timed wait, so the wait looks,
/// and a query that ends in a millisecond costs about a millisecond.
///  The typed-value sweep, which makes thousands of these calls, measured 143 s with a ten-millisecond look and 25 s
/// with a wait in another thread that this replaces.
const LOOK_EVERY: std::time::Duration = std::time::Duration::from_millis(1);

/// How long a query this helper has killed may take to be reaped.
/// Why five seconds:
///  a `SIGKILL`ed child is reaped at once, so this window only covers the kernel and the scheduler;
/// a query that still has not been reaped in it is a defect of this helper, not a slow host.
const REAPING: std::time::Duration = std::time::Duration::from_secs(5);

/// What:
///  The seam a control of the ending uses:
///  it runs between the last look at the query and the signal that ends it.
/// Why:
///  That window is a few instructions wide,
/// so no query can arrange to end in it from outside,
/// and a control about which ending this helper reports needs a way to put an ending there.
///  Nothing of this seam exists in the release build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// beforeTheSignal?.(query); // absent unless a control armed it
/// ```
#[cfg(test)]
fn the_ending_seam(query: libc::pid_t) {
    if let Some(seam) = ENDING_SEAM.lock().expect("the ending seam").take() {
        seam(query);
    }
}

/// The one ending seam a control arms, for the one call it measures.
#[cfg(test)]
static ENDING_SEAM: std::sync::Mutex<Option<Box<dyn FnOnce(libc::pid_t) + Send>>> =
    std::sync::Mutex::new(None);

/// `the_ending_seam` in a build without controls: nothing runs.
#[cfg(not(test))]
fn the_ending_seam(_query: libc::pid_t) {}

/// Arm the ending seam for one call of `real_git_within`.
#[cfg(test)]
pub(crate) fn arm_the_ending_seam(seam: impl FnOnce(libc::pid_t) + Send + 'static) {
    *ENDING_SEAM.lock().expect("the ending seam") = Some(Box::new(seam));
}

/// Disarm the ending seam, answering whether it was still armed.
/// Why:
///  a control that did not reach the seam must not leave it armed for another test.
#[cfg(test)]
pub(crate) fn disarm_the_ending_seam() -> bool {
    return ENDING_SEAM
        .lock()
        .expect("the ending seam")
        .take()
        .is_some();
}

/// What ended one bounded query, as the last status the wait returned showed it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum QueryEnding {
    /// The last status the wait returned was not the signal this helper sends at its bound:
    /// the query ended before that bound ran out, or in the instant after it, and this answer does not say which.
    ///  Its status and output are the query's own, whatever status that was:
    /// a code of 124 or 137 or a signal's death included.
    ///  A signal another process sent reads the same here:
    /// a wait says which signal ended a child and nothing about who sent it.
    Itself,
    /// The bound ran out, and the last status the wait returned was the signal this helper sends at the bound
    /// to the process group the query led.
    ///  A query killed by that same signal from elsewhere in that instant reads the same,
    /// because a wait says which signal ended a child and nothing about who sent it.
    AtTheBound,
}

/// What one bounded query of real Git answered.
#[derive(Debug)]
pub(crate) struct QueryAnswer {
    /// The status the wait returned, and the bytes the two pipes had given by the end of the grace after the ending.
    ///  A descendant that holds a pipe can add bytes in that window, and `held_open` says when one did.
    pub output: Output,
    /// What ended the query, as the last status the wait returned showed it.
    pub ending: QueryEnding,
    /// Whether a pipe of the query was still held when the window for its end passed,
    /// as the kernel answered it,
    /// so a descendant of the query kept that end open and the output may be short of what the query wrote.
    pub held_open: bool,
    /// Whether the input was still being written when the window passed,
    /// so a descendant of the query kept its read end open and the query never read all of it.
    ///  No verdict rests on this figure, so the writer's own late waking can make it say so and mislead nothing.
    pub input_held: bool,
}

/// What:
///  One pipe of a query: the bytes as they arrive,
///  the news that its reader reached the end,
///  and a second reading end of the same pipe for the answer to ask the kernel about.
/// Why:
///  Reading the pipes on this thread would wait for the end of file,
/// which a descendant that inherited the pipe can postpone past every bound;
/// a reader of its own lets the answer keep what has arrived and leave the rest to whoever holds it,
/// and that reader ends when the holder lets go or with the test's process.
///  The second reading end is the kernel's own answer about the same pipe,
/// kept there until someone asks for it,
/// so a verdict about a pipe that something still holds is about the query and not about when this process's
/// own threads ran.
///  A second reading end never keeps a pipe from ending: only a writing end does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { bytes, ended, theKernelsHand } = drain(child.stdout); // ended: a message when the pipe reaches its end
/// ```
fn draining<R: Read + Send + AsRawFd + 'static>(
    mut reader: R
) -> (
    std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
    std::sync::mpsc::Receiver<()>,
    OwnedFd,
) {
    // What:     `libc::dup` asks the kernel for a second descriptor number for the same open pipe end;
    //           the number the reader holds keeps working and the new one has to be closed by its owner.
    //           `reader.as_raw_fd()` is the number behind the handle the thread below takes.
    // Why:      the asking outlives that handle, so it needs a number of its own;
    //           a negative answer means the ask failed and nothing can be reported about this pipe.
    // Gotcha:   a duplicated READING end of a pipe does not delay the pipe's end, though a duplicated writing
    //           end would: the kernel counts writing ends alone when it decides that a pipe has ended.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const forTheKernel: number = duplicateHandle(reader.asRawFd()); // a number this code must close
    // if (forTheKernel < 0) throw new Error("a second reading end of a query's pipe");
    // ```
    let doubled: libc::c_int = unsafe { libc::dup(reader.as_raw_fd()) };
    assert!(
        doubled >= 0,
        "a second reading end of one of a query's pipes: {}",
        std::io::Error::last_os_error()
    );
    // What:     `OwnedFd::from_raw_fd` takes ownership of one descriptor number the kernel answered, so the
    //           number is closed when this value goes out of scope.
    // Why:      every return path of the answer then closes the second reading end, including a panic's.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const theKernelsHand = new OwnedHandle(forTheKernel); // closed on scope exit
    // ```
    let the_kernel_s_hand: OwnedFd = unsafe { OwnedFd::from_raw_fd(doubled) };
    let kept: std::sync::Arc<std::sync::Mutex<Vec<u8>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let (sender, receiver) = std::sync::mpsc::channel::<()>();
    let sink: std::sync::Arc<std::sync::Mutex<Vec<u8>>> = std::sync::Arc::clone(&kept);
    let _reader: std::thread::JoinHandle<()> = std::thread::Builder::new()
        .spawn(move || {
            let mut buffer: [u8; 8192] = [0; 8192];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(read) => sink
                        .lock()
                        .expect("the pipe's bytes")
                        .extend_from_slice(&buffer[..read]),
                    Err(_) => break,
                }
            }
            let _ended = sender.send(());
        })
        .expect("the reader of a query's pipe starts");
    return (kept, receiver, the_kernel_s_hand);
}

/// What:
///  Whether the kernel says one of the query's pipes has reached its end within `bound`:
///  no process is left that holds a writing end of it.
/// Why:
///  A pipe's end is the kernel's news: it arrives when the last writing end closes and stays there until
///  someone asks, so a thread of this process that a loaded host wakes late still finds it instead of missing it.
/// Asking the pipe is what makes this answer about the query and whatever it started,
/// and not about when this process's own threads ran.
///  A window that has run out still asks: a pipe whose end has come answers at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ended = hasEndedWithin(pipe.theKernelsHand, 250); // false while something holds a writing end
/// ```
fn has_ended_within(
    pipe: &OwnedFd,
    bound: std::time::Duration,
) -> bool {
    // What:     `libc::pollfd` is one question about one descriptor: `events` is what to wait for and `revents`
    //           is what the kernel answered.
    // Why:      no event is asked for here, because the bytes are the reader's job;
    //           `POLLHUP` and `POLLERR` are reported whether or not they are asked for.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const question = { fd: pipe.number, asked: 0, answered: 0 };
    // ```
    let mut asked: libc::pollfd = libc::pollfd {
        fd: pipe.as_raw_fd(),
        events: 0,
        revents: 0,
    };
    let began: std::time::Instant = std::time::Instant::now();
    loop {
        // The window left, as the whole milliseconds `libc::poll` takes; a bound past that is asked in full.
        let left: std::time::Duration = bound.saturating_sub(began.elapsed());
        let milliseconds: libc::c_int =
            libc::c_int::try_from(left.as_millis()).unwrap_or(libc::c_int::MAX);
        // SAFETY: `poll` reads the one question it is given, writes its answer into the same entry, and touches
        //  no memory beyond it; `asked.fd` is a descriptor this answer owns through `pipe`.
        let answered: libc::c_int = unsafe { libc::poll(&mut asked, 1, milliseconds) };
        if answered < 0 {
            let error: std::io::Error = std::io::Error::last_os_error();
            // A signal this process received ends the wait early and says nothing about the pipe,
            // so the same question is asked again with the window that is left.
            if error.kind() == std::io::ErrorKind::Interrupted && began.elapsed() < bound {
                continue;
            }
            panic!("one of a query's pipes could not be waited for: {error}");
        }
        // Nothing was answered within the window: something outside the query still holds a writing end.
        if answered == 0 {
            return false;
        }
        // `POLLHUP` is the kernel's news that no writing end is left; `POLLERR` is a pipe it reports as failed.
        return asked.revents & (libc::POLLHUP | libc::POLLERR) != 0;
    }
}

/// What:
///  Wait for the reader of one of the query's pipes to hand over what it read,
///  and answer whether that pipe reached its end at all.
/// Why:
///  Once the kernel says a pipe has no writing end left, every byte the query wrote is in the pipe,
/// so what is left is the reader's own wakes;
/// the window that bounds them is long enough to stop a hang and nothing else,
/// and a reader that misses it is a defect of this helper, said here with what was waited for and how long.
/// A pipe that something outside the query still holds needs no wait: its reader ends when that holder lets go.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ended = await theReaderHandedOverWhatItRead("standard output", "git", pipeEnded, readerWord);
/// ```
fn the_reader_handed_over_what_it_read(
    what: &str,
    program: &Path,
    the_pipe_has_ended: bool,
    the_reader_s_word: std::sync::mpsc::Receiver<()>,
) -> bool {
    if !the_pipe_has_ended {
        return false;
    }
    if the_reader_s_word.recv_timeout(REPORTING_THE_END).is_err() {
        panic!(
            "the query {program:?} reached the end of its {what}, and the thread that read it did not hand over what it read within {REPORTING_THE_END:?}"
        );
    }
    return true;
}

/// What:
///  Run one real-Git query under `bound` and answer the status the wait returned, how the query ended,
///  the bytes its pipes had given by the grace after that ending, and what a descendant of it still holds.
/// Why:
///  A control that ran Git directly would hold the suite for ever against a Git that never answers,
/// and report nothing at all;
///  and a bound asked of another program (`/usr/bin/timeout`) can only be read back from an exit status,
/// which an ordinary 124 or 137 or a signal's death does not tell apart from the bound.
/// So the bound is this helper's own promise and this helper keeps it:
/// it starts the query in a process group of its own,
/// watches the clock itself,
/// and at the bound ends that group and reaps the query.
///  `QueryEnding::AtTheBound` means the bound ran out and the last status the wait returned was that signal;
/// a query killed by the same signal from elsewhere in that instant reads the same,
/// because a wait says which signal ended a child and never who sent it.
/// A query that ended by itself comes back with its own status and output.
///  The answer is what this helper bounds, and a pipe or an input a descendant holds turns no answer into a panic:
/// the write of the input runs on a thread of its own,
/// so a query that keeps its standard input open and reads none of it fills the pipe and is ended at the bound
/// like any other,
/// a write the ending broke is part of that ending rather than a failure,
/// and an input a descendant of the query still holds is said in `input_held`, as a held pipe is in `held_open`.
///  A descendant that keeps a pipe or the input is where that bound stops:
/// the thread reading or writing that pipe is left to end when the descendant lets go or when the test's process ends,
/// so what is bounded is the answer and not every thread that served it.
/// The caller answers each ending with its own sentence,
/// because only it knows which query it made and what the query is for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = runUnderBound('git', ['--version'], 60); // { output, ending, heldOpen }
/// ```
pub(crate) fn real_git_within(
    program: &Path,
    directory: Option<&Path>,
    environment: &[(OsString, OsString)],
    arguments: &[OsString],
    input: Option<&[u8]>,
    bound: std::time::Duration,
) -> QueryAnswer {
    let mut command: Command = Command::new(program);
    command.args(arguments);
    if let Some(at) = directory {
        command.current_dir(at);
    }
    command.env_clear();
    for (name, value) in environment {
        command.env(name, value);
    }
    command.stdin(if input.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    // The query leads a process group of its own, so the bound takes what it started in that group with it.
    command.process_group(0);
    let mut child: std::process::Child = command.spawn().expect("the query starts");
    let group: libc::pid_t = libc::pid_t::try_from(child.id()).expect("a pid fits the pid type");
    let (standard_output, output_reached_its_end, output_s_end) =
        draining(child.stdout.take().expect("piped standard output"));
    let (standard_error, error_reached_its_end, error_s_end) =
        draining(child.stderr.take().expect("piped standard error"));
    // The input is written on a thread of its own, under the same bound as the wait:
    // a query that keeps its standard input open and reads none of it fills the pipe,
    // and a write on this thread would wait there with nothing watching the clock.
    let (input_sender, input_receiver) = std::sync::mpsc::channel::<std::io::Result<()>>();
    let input_writer: Option<std::thread::JoinHandle<()>> = match input {
        Some(bytes) => {
            let mut standard_input: std::process::ChildStdin =
                child.stdin.take().expect("piped standard input");
            let kept: Vec<u8> = bytes.to_vec();
            Some(
                std::thread::Builder::new()
                    .spawn(move || {
                        let written: std::io::Result<()> =
                            standard_input.write_all(kept.as_slice());
                        // Dropping the handle in this thread is what gives the query its end of file.
                        let _sent = input_sender.send(written);
                    })
                    .expect("the writer of a query's input starts"),
            )
        }
        // Without an input the query's standard input is the null device, and nothing is written.
        None => None,
    };
    // This helper owns the wait, so no other thread can collect the query before the bound's signal is sent:
    // the query stays unreaped, and the number it leads its group by stays its own until it is killed or reaped.
    let began: std::time::Instant = std::time::Instant::now();
    let (status, ending): (std::process::ExitStatus, QueryEnding) = loop {
        if let Some(status) = child.try_wait().expect("the query's state") {
            break (status, QueryEnding::Itself);
        }
        if began.elapsed() >= bound {
            // A control about the ending places the query's own exit here, before the signal.
            the_ending_seam(group);
            // SAFETY: `kill` reads the two numbers it is given and touches no memory of ours.
            let _killed = unsafe { libc::kill(-group, libc::SIGKILL) };
            // The killed query is reaped within its own window, so this reap waits with a bound too.
            let reaping: std::time::Instant = std::time::Instant::now() + REAPING;
            let status: std::process::ExitStatus = loop {
                if let Some(status) = child.try_wait().expect("the killed query's state") {
                    break status;
                }
                if std::time::Instant::now() >= reaping {
                    panic!(
                        "the query {program:?} was killed at its bound of {bound:?} and not reaped within {REAPING:?}"
                    );
                }
                std::thread::sleep(LOOK_EVERY);
            };
            // The status the wait returns says how the query ended: the signal this helper sends at its bound,
            // or an ending the query reached first.
            // Which process sent a signal is not in a status, so a query killed by that same signal from elsewhere
            // in this instant reads as the bound's. Whether a signal could be sent says nothing about that,
            // so the kill's answer is not read here.
            break (
                status,
                if status.signal() == Some(libc::SIGKILL) {
                    QueryEnding::AtTheBound
                } else {
                    QueryEnding::Itself
                },
            );
        }
        std::thread::sleep(LOOK_EVERY);
    };
    // One window for the write and the two pipes' ends, and never a wait past it.
    let grace_ends: std::time::Instant = std::time::Instant::now() + PIPE_GRACE;
    // The write is judged with the query's ending:
    // a write the query's death broke is part of that ending and no failure,
    // any other failure of it is said,
    // and a write a descendant of the query still holds is said in the answer, as a held pipe is.
    let mut input_held: bool = false;
    if let Some(writer) = input_writer {
        let written: Option<std::io::Result<()>> = input_receiver
            .recv_timeout(grace_ends.saturating_duration_since(std::time::Instant::now()))
            .ok();
        match written {
            // A descendant of the query kept the read end open and reads none of it: the write is left to that process.
            None => input_held = true,
            Some(Ok(())) => {}
            Some(Err(error)) if error.kind() == std::io::ErrorKind::BrokenPipe => {}
            Some(Err(error)) => {
                panic!("the input of the query {program:?} could not be written: {error}")
            }
        }
        if !input_held {
            let _writer_joined = writer.join();
        }
    }
    // What the pipes gave: the end of each is the kernel's answer, and the bytes are the reader's word,
    // so a reader this host wakes late can neither invent a held pipe nor hide one.
    let output_ended_by_the_kernel: bool = has_ended_within(
        &output_s_end,
        grace_ends.saturating_duration_since(std::time::Instant::now()),
    );
    let error_ended_by_the_kernel: bool = has_ended_within(
        &error_s_end,
        grace_ends.saturating_duration_since(std::time::Instant::now()),
    );
    let output_ended: bool = the_reader_handed_over_what_it_read(
        "standard output",
        program,
        output_ended_by_the_kernel,
        output_reached_its_end,
    );
    let error_ended: bool = the_reader_handed_over_what_it_read(
        "standard error",
        program,
        error_ended_by_the_kernel,
        error_reached_its_end,
    );
    let stdout: Vec<u8> = standard_output
        .lock()
        .expect("the query's standard output")
        .clone();
    let stderr: Vec<u8> = standard_error
        .lock()
        .expect("the query's standard error")
        .clone();
    return QueryAnswer {
        output: Output {
            status,
            stdout,
            stderr,
        },
        ending,
        held_open: !(output_ended && error_ended),
        input_held,
    };
}

/// The controls of the bounded query above.
#[cfg(test)]
#[path = "fixture_git_commands_tests.rs"]
mod tests;
