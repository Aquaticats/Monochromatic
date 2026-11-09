//! What:
//!  The output of a child that a caller started itself,
//! read to the end through two pipes on the runtime,
//! with every byte kept,
//!  while the child's end is awaited through its pidfd.
//! Why:
//!  `Command::output` and `Child::wait_with_output` block a thread until the child ends.
//! The watched one-shot steps of the
//!  wrapper (`push/child_watch.rs`) must return all of a step's output,
//! not the tail the captured files keep
//!  (`output_capture.rs`),
//! so they cannot use the captured run;
//! this reads the pipes on the runtime instead (owner,
//! 2026-11-07:
//!  "Full on tokio.
//! Multi-thread.";
//! the coordinator's settlement of the T5 one-shot form).
//! The pipes are made and registered
//!  with the reactor before the child starts,
//! so a refusal (a descriptor limit,
//! a call from inside the runtime) leaves no child.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pipes = await openPipes();                           // before the start: a failure here starts nothing
//! const child = spawn(program, args, { stdio: ['pipe', pipes.stdoutWriter, pipes.stderrWriter] });
//! const [stdout, stderr, status] = await Promise.all([readAll(pipes.stdout), readAll(pipes.stderr), child.exited]);
//! ```

use crate::child_awaited_through_a_pidfd::adopt;
use crate::lazy_runtime::run_on_runtime;
use std::os::fd::OwnedFd;
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use tokio::io::AsyncReadExt as _;
use tokio::net::unix::pipe::Receiver;

/// The read ends of a child's standard output and standard error,
/// registered with the runtime's reactor.
pub struct StepPipes {
    /// What the child writes to its standard output.
    standard_output: Receiver,
    /// What the child writes to its standard error.
    standard_error: Receiver,
}

/// What:
///  Makes the two pipes,
/// registers their read ends with the runtime (building it on the first need) and hands their write ends to
///  `command` as its standard output and standard error.
/// Why:
///  Nothing here starts a child,
/// so a failure is plain and leaves nothing running.
/// A call from inside the runtime is refused,
///  because the later wait enters the runtime and would never end from within it.
/// After the start the caller must call
///  `release_write_ends`,
/// or the readers never see the end of the output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pipes = await attachPipes(command); // throws before any child exists
/// ```
pub fn attach_pipes(command: &mut Command) -> std::io::Result<StepPipes> {
    return match run_on_runtime(attach_pipes_awaiting(command)) {
        Ok(opened) => opened,
        Err(refusal) => Err(std::io::Error::other(refusal.to_string())),
    };
}

/// What:
///  `attach_pipes` as the future a task of the runtime awaits.
/// Why:
///   The read ends must be registered with the reactor of the runtime the wait will run on,
/// and a caller that is already a task has one;
///  entering the runtime from inside it would never end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pipes = await attachPipes(command);
/// ```
pub async fn attach_pipes_awaiting(command: &mut Command) -> std::io::Result<StepPipes> {
    let (output_reader, output_writer) = std::io::pipe()?;
    let (error_reader, error_writer) = std::io::pipe()?;
    let pipes: StepPipes =
        StepPipes { standard_output: Receiver::from_owned_fd(OwnedFd::from(output_reader))?, standard_error: Receiver::from_owned_fd(OwnedFd::from(error_reader))? };
    command.stdout(Stdio::from(output_writer)).stderr(Stdio::from(error_writer));
    return Ok(pipes);
}

/// What:
///  Drops the write ends that `attach_pipes` gave `command`,
/// by replacing its standard output and standard error.
/// Why:
///  The command holds them until it is changed or dropped;
/// while this process holds a write end,
/// the readers wait for an end of
///  output that never comes.
/// Call it after the start,
/// whether or not the start worked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// releaseWriteEnds(command);
/// ```
pub fn release_write_ends(command: &mut Command) {
    command.stdout(Stdio::null()).stderr(Stdio::null());
}

/// Reads `pipe` to its end and returns every byte.
async fn read_whole(mut pipe: Receiver) -> std::io::Result<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();
    pipe.read_to_end(&mut bytes).await?;
    return Ok(bytes);
}

impl StepPipes {
    /// What:
    ///  Reads both pipes to their end and awaits `child`'s exit together,
    /// then returns the status and every byte of both streams,
    ///  like `Child::wait_with_output`,
    /// with no thread blocked on the child.
    /// Why:
    ///  Waiting first and reading after would deadlock a child that fills a pipe;
    /// reading to the end of both while awaiting the
    ///  exit is what `wait_with_output` does,
    /// so a descendant that keeps a pipe open keeps the read open,
    /// as before.
    /// A read error
    ///  ends the child,
    /// because nothing is left to read what it writes.
    /// The child's input is closed first,
    /// as `wait_with_output`
    ///  closes it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const [stdout, stderr, status] = await Promise.all([readAll(out), readAll(err), child.exited]);
    /// ```
    pub fn read_until_the_child_ends(self, child: Child) -> std::io::Result<Output> {
        // Both wait in an `Option`: whichever of the two paths runs takes them, and a refused runtime leaves them here.
        let mut waiting: Option<Child> = Some(child);
        let mut unread: Option<StepPipes> = Some(self);
        let finished = run_on_runtime(async {
            return match (unread.take(), waiting.take()) {
                (Some(open), Some(started)) => read_until_the_child_ends_awaiting(open, started).await,
                _ => Err(std::io::Error::other("the child or its pipes were handed over twice")),
            };
        });
        return match finished {
            Ok(output) => output,
            Err(refusal) => {
                // The future never ran, so the child is still here; it is ended rather than left to run unwatched.
                if let Some(mut unwatched) = waiting.take() {
                    let _ended = unwatched.kill();
                    let _reaped = unwatched.wait();
                }
                Err(std::io::Error::other(refusal.to_string()))
            }
        };
    }

    /// What:
    ///  `read_until_the_child_ends` as the future a task of the runtime awaits,
    /// with no call into the runtime from inside it.
    /// Why:
    ///   A caller that is already a task (the asynchronous Git runner,
    /// `transaction/transaction_git_awaiting.rs`) must not enter the runtime
    ///  from inside it;
    /// this is the same wait those callers make.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const [stdout, stderr, status] = await Promise.all([readAll(out), readAll(err), child.exited]);
    /// ```
    pub async fn read_until_the_child_ends_awaiting(self, child: Child) -> std::io::Result<Output> {
        return read_until_the_child_ends_awaiting(self, child).await;
    }
}

/// What:
///  Reads both pipes of a started child to their end while its exit is awaited,
/// on the runtime the caller already runs on.
/// Why:
///   The one body both faces of this wait share,
/// so a synchronous caller and a task cannot report differently;
/// the child's input is closed first,
///  as `wait_with_output` closes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [stdout, stderr, status] = await Promise.all([readAll(out), readAll(err), child.exited]);
/// ```
async fn read_until_the_child_ends_awaiting(pipes: StepPipes, child: Child) -> std::io::Result<Output> {
    let StepPipes { standard_output, standard_error } = pipes;
    // `mut` lets the wait take the standard input away before the join.
    let mut input_closed: Child = child;
    drop(input_closed.stdin.take());
    let mut owned = adopt(input_closed);
    let (read_output, read_error, waited) = tokio::join!(read_whole(standard_output), read_whole(standard_error), owned.wait());
    let status: ExitStatus = waited?;
    return Ok(Output { status, stdout: read_output?, stderr: read_error? });
}
