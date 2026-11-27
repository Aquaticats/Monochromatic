//! What:
//!  Run one real-Git query under a bound of this helper's own,
//!  and answer the status the wait returned,
//!  how the query ended,
//!  the bytes its pipes had given by the grace after that ending,
//!  and what a descendant of it still holds.
//! Why:
//!  A control that ran Git directly would hold the suite for ever against a Git that never answers;
//!  a bound asked of another program (`/usr/bin/timeout`) can only be read back from an exit status,
//!  which an ordinary 124 or 137 or a signal's death does not tell apart from the bound;
//!  and a pipe or an input a descendant holds must turn no answer into a panic.
//!  So this file owns the wait,
//! the clock,
//! the signal at the bound,
//! the reap and the grace after it,
//!  and every other part here is one of the things that wait stands on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = runUnderBound('git', ['--version'], 60_000); // { output, ending, heldOpen, inputHeld }
//! ```

/// The reader of one pipe,
/// the kernel's own answer about that pipe,
/// and the wait for the reader's word.
use super::pipes_read_beside_the_query::{
    draining,
    has_ended_within,
    the_reader_handed_over_what_it_read,
};
/// What one bounded query answered,
/// and what ended it.
use super::query_answer::QueryAnswer;
/// How often the wait looks whether the query ended,
/// how long each pipe is given to reach its end,
/// and how long a killed query may take to be reaped.
use super::query_bounds::{
    LOOK_EVERY,
    PIPE_GRACE,
    REAPING,
};
/// What ended a query,
/// as the last status the wait returned showed it.
use super::query_ending::QueryEnding;
/// The seam a control places an ending in,
/// between the last look at the query and the bound's signal.
use super::query_ending::the_ending_seam;
/// `OsString` is owned operating-system text, which this query's arguments and environment are.
/// Its sibling `&str` is a borrowed view that must be UTF-8, and `OsString` need not be.
use std::ffi::OsString;
/// `Write` fills the standard input of a query,
/// on a thread of its own.
use std::io::Write;
/// `process_group` gives the query a process group of its own,
/// which the bound ends as a whole.
use std::os::unix::process::CommandExt as _;
/// `signal` reads which signal ended the query,
/// which is how the ending is known.
use std::os::unix::process::ExitStatusExt as _;
/// `Path` is a borrowed filesystem path of raw OS bytes.
use std::path::Path;
/// The command builder,
/// the child's pipes and the finished child every fixture Git query is.
use std::process::{
    Command,
    Output,
    Stdio,
};

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
