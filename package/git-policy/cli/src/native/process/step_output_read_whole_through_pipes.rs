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
use crate::process_use::{ProcessUser, UseScan, UseTarget};
use std::os::fd::{AsFd as _, OwnedFd};
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::time::{Duration, Instant};
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

/// How long the read of a step's output may go on after the step itself has ended.
///  A pipe holds at most 64 KiB (`pipe-max-size`'s default),
/// which this process reads in one go,
/// so a second is thousands of times what draining a pipe nothing more is written to costs.
///  A program the step started that still holds the write end a second after the step ended is holding it,
///  not writing the step's output,
/// and the wait for the step must not last as long as that program lives.
/// Labelled unmeasured as a quiet window is (`SPEC.md`,
/// "The quiet windows are labelled unmeasured"):
/// no log holds how long a surviving writer takes to close a step's streams.
pub const OUTPUT_DRAIN_AFTER_EXIT: Duration = Duration::from_secs(1);

/// What:
///  The sentence for a step whose output a program it started still held open after the step ended.
/// Why:
///  The step is over and its output is what was read;
/// the person has to know that the step is not what holds the command,
///  and which program does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${waitingFor} ended, but a program it started still holds its output open (${holders}); ...`
/// ```
fn output_held_open_text(waiting_for: &str, holders: &[ProcessUser]) -> String {
    let named: String = holder_named(holders);
    return format!(
        "{waiting_for} ended, but {named} still holds the step's output open, so the output this command read may be incomplete and \
         the wait for it ended after {} seconds of it. Check with `ps` what that program is and whether it is still needed, end it if \
         it is stuck, and run the command again; the step's own end is what the repository state shows.",
        OUTPUT_DRAIN_AFTER_EXIT.as_secs()
    );
}

/// What:
///  The sentence for a step that ended with its standard error cut short:
/// its exit status and its standard output were read whole,
///  and a program it started still holds its standard error open.
/// Why:
///  What the step printed to standard output is its answer,
/// so the answer is returned and the person is told that the standard error
///  read here stops where the program still holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${waitingFor} ended and its exit status and standard output were read whole, but ${named} still holds its standard error open ...`
/// ```
fn error_cut_short_text(waiting_for: &str, holders: &[ProcessUser]) -> String {
    let named: String = holder_named(holders);
    return format!(
        "{waiting_for} ended and its exit status and standard output were read whole, but {named} still holds its standard error open, so \
         the standard error read here stops {} seconds after the step ended. Check with `ps` what that program is and whether it is \
         still needed; the step's own end is what the repository state shows.",
        OUTPUT_DRAIN_AFTER_EXIT.as_secs()
    );
}

/// What:
///  The program that still holds a stream of the step,
/// named as the person knows it.
/// Why:
///  Both sentences about a held-open stream name the same program,
/// and a number alone tells the person nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holders.length === 0 ? 'a program it started, which no longer shows in the process table' : `process ${holders[0].pid} (${holders[0].command})`;
/// ```
fn holder_named(holders: &[ProcessUser]) -> String {
    return match holders.first() {
        Some(holder) => match holder.command.as_ref() {
            Some(command) => format!("process {} ({command})", holder.pid),
            None => format!("process {}", holder.pid),
        },
        None => String::from("a program it started, which no longer shows in the process table"),
    };
}

/// What:
///  The error of a read whose stream a program the step started still held open,
/// with the bytes read before the drain ended and the programs that held it.
/// Why:
///   The caller decides what the loss costs:
/// a step whose standard output was cut short has no answer,
///  while one whose standard error was cut short still has its exit status and its standard output,
/// so the bytes
///  taken before the drain ended have to reach that decision.
#[derive(Debug)]
struct OutputHeldOpen {
    /// The sentence that names the step,
    /// the program and what to do.
    waiting_for: String,
    /// The programs that held the stream open when the drain ended.
    holders: Vec<ProcessUser>,
    /// The bytes read of the stream before the drain ended.
    read: Vec<u8>,
}

impl std::fmt::Display for OutputHeldOpen {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(output_held_open_text(self.waiting_for.as_str(), self.holders.as_slice()).as_str());
    }
}

impl std::error::Error for OutputHeldOpen {}

/// Whether `error` is the error of a step whose output a program it started still held open.
pub fn is_output_held_open(error: &std::io::Error) -> bool {
    return error.get_ref().is_some_and(is_the_held_open_error);
}

/// Whether `inner` is this module's held-open error,
/// for `is_output_held_open`.
fn is_the_held_open_error(inner: &(dyn std::error::Error + Send + Sync + 'static)) -> bool {
    return inner.is::<OutputHeldOpen>();
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
    pub fn read_until_the_child_ends(self, child: Child, waiting_for: &str, ceiling: Option<CarriedCeiling>) -> std::io::Result<Output> {
        // Both wait in an `Option`: whichever of the two paths runs takes them, and a refused runtime leaves them here.
        let mut waiting: Option<Child> = Some(child);
        let mut unread: Option<StepPipes> = Some(self);
        let finished = run_on_runtime(async {
            return match (unread.take(), waiting.take()) {
                (Some(open), Some(started)) => read_until_the_child_ends_awaiting(open, started, waiting_for, ceiling).await,
                _ => Err(std::io::Error::other("the child or its pipes were handed over twice")),
            };
        });        return match finished {
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
    pub async fn read_until_the_child_ends_awaiting(self, child: Child, waiting_for: &str, ceiling: Option<CarriedCeiling>) -> std::io::Result<Output> {
        return read_until_the_child_ends_awaiting(self, child, waiting_for, ceiling).await;
    }
}

/// What:
///  Reads both pipes of a started child to their end while its exit is awaited,
/// on the runtime the caller already runs on,
/// with an end of its own.
/// Why:
///   The one body both faces of this wait share,
/// so a synchronous caller and a task cannot report differently;
/// the child's input is closed first,
///  as `wait_with_output` closes it.
/// A pipe ends at end of file,
/// which needs every write end closed,
/// so a program the step
///  started keeps the read from ending though the step has;
/// each read therefore ends
///  `OUTPUT_DRAIN_AFTER_EXIT` after the step has ended,
/// and the wait says which program held it.
///  `ceiling` is the step's own ceiling when nothing else ends the step (the watch had no task):
/// the wait then ends the
///  child itself,
/// so a step is never waited for past the bound the watch would have given it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [stdout, stderr, status] = await withCeiling(Promise.all([readAll(out), readAll(err), child.exited]), ceiling);
/// ```
async fn read_until_the_child_ends_awaiting(pipes: StepPipes, child: Child, waiting_for: &str, ceiling: Option<CarriedCeiling>) -> std::io::Result<Output> {
    let StepPipes { standard_output, standard_error } = pipes;
    // `mut` lets the wait take the standard input away before the join.
    let mut input_closed: Child = child;
    drop(input_closed.stdin.take());
    // The read ends of the pipes, which are the same open file as their write ends: a program that holds one holds the
    //  output open, and the pipe's identity is what finds it. Each stream is found by its own pipe: a program that holds only
    //  the standard error open is not holding the standard output, and naming the wrong pipe named no holder at all
    //  (the third review's finding 7).
    let held_output: Option<(u64, u64)> = pipe_identity(standard_output.as_fd());
    let held_error: Option<(u64, u64)> = pipe_identity(standard_error.as_fd());
    let mut owned = adopt(input_closed);
    // The moment the child ended, told to both reads so each can end its drain of whatever a program it started still writes.
    let (ended, ends) = tokio::sync::watch::channel::<Option<Instant>>(None);
    let reading = async {
        return tokio::join!(
            read_until_the_step_has_ended(standard_output, ends.clone(), waiting_for, held_output),
            read_until_the_step_has_ended(standard_error, ends, waiting_for, held_error),
        );
    };
    let waiting = async {
        // The ceiling bounds this wait for the child, and the reads run beside it: a ceiling around both reported a step that
        //  finished inside its ceiling as ended at it, because a program the step started still held one stream open (the third
        //  review's finding 5).
        let status: std::io::Result<ExitStatus> = match ceiling {
            Some(carried) => match tokio::time::timeout(carried.limit, owned.wait()).await {
                Ok(status) => status,
                Err(_elapsed) => {
                    // Nothing ended this step before its ceiling, so this wait carries it: whatever the watch could not do, a step
                    //  never outlives the ceiling the wrapper gives it. `StepBound::EndAt` is a whole process tree, so the
                    //  programs the step started end with it and can neither hold its streams nor write after it was reported
                    //  ended (the fifth review's finding 2).
                    ended.send_replace(Some(Instant::now()));
                    // Whether the whole tree could be ended is part of the ending: a step the kernel would not let go is left
                    //  running, and the sentence the caller prints must not say it ended (the fifth review's finding 6).
                    let left_running: bool = owned.end_with_the_processes_it_started();
                    Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        crate::child_waited_for_within_a_ceiling::AtTheCeiling {
                            left_running,
                            sentence: Some(waiting_carried_the_ceiling(waiting_for, carried, left_running)),
                        },
                    ))
                }
            },
            None => owned.wait().await,
        };
        // The moment the child ended, told to both reads so each can end its drain of whatever a program it started still writes.
        ended.send_replace(Some(Instant::now()));
        return status;
    };
    // Both are awaited together: the reads must drain while the child runs, or a step that writes more than a pipe holds waits
    //  for a reader for good.
    let (read, waited): ((std::io::Result<Vec<u8>>, std::io::Result<Vec<u8>>), std::io::Result<ExitStatus>) = tokio::join!(reading, waiting);
    let status: ExitStatus = waited?;
    let (read_output, read_error) = read;
    // The step's answer is its exit status and its standard output, so neither is thrown away because a program the step started
    //  still holds its standard error: what was read of that stream is returned and the person is told where it stops.
    let stdout: Vec<u8> = read_output?;
    let stderr: Vec<u8> = match read_error {
        Ok(bytes) => bytes,
        Err(error) => match cut_short(&error) {
            Some((bytes, sentence)) => {
                crate::diagnostic_log::warn("stepOutput", sentence.as_str());
                bytes
            }
            None => return Err(error),
        },
    };
    return Ok(Output { status, stdout, stderr });
}

/// What:
///  The bytes read before the drain ended of a stream a program the step started still held open,
/// and the sentence that
///  names that program.
/// Why:
///   Only that one loss may be forgiven;
/// a read that failed for any other reason is still the error of the wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cutShort(error: Error): { bytes: Buffer; sentence: string } | undefined;
/// ```
fn cut_short(error: &std::io::Error) -> Option<(Vec<u8>, String)> {
    let held: &OutputHeldOpen = error.get_ref()?.downcast_ref::<OutputHeldOpen>()?;
    return Some((held.read.clone(), error_cut_short_text(held.waiting_for.as_str(), held.holders.as_slice())));
}

/// What:
///  Reads a pipe to its end,
/// or gives up `OUTPUT_DRAIN_AFTER_EXIT` after the step ended and says what still holds it.
/// Why:
///   A pipe that a program the step started holds open never reaches end of file,
/// so without this bound the
///  command waits for as long as that program lives,
/// past the step's own ceiling
/// (`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
/// finding 2).
///  What was read before the drain is returned on the error's way out through the sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytes = []; for (;;) { const r = await Promise.race([pipe.read(), drainEnded]); if (drainEnded) throw heldOpen(holders(pipe)); if (r === EOF) return bytes; bytes.push(...); }
/// ```
async fn read_until_the_step_has_ended(mut pipe: Receiver, mut ends: tokio::sync::watch::Receiver<Option<Instant>>, waiting_for: &str, held: Option<(u64, u64)>) -> std::io::Result<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();
    let drain = async {
        let ended_at: Instant = wait_for_the_end(&mut ends).await;
        tokio::time::sleep_until(tokio::time::Instant::from_std(ended_at + OUTPUT_DRAIN_AFTER_EXIT)).await;
    };
    // End of file wins over the drain: a read that answers is the whole answer,
    // and the drain would report the output as held open
    //  though every writer had closed,
    // which rejects the answer of a step that finished inside its ceiling
    //  (the fifth review's finding 5).
    // The read is asked first,
    // so a read whose readiness the reactor already knows settles the poll at which both are ready;
    // a read it has not heard about yet is answered by the sweep below.
    tokio::select! {
        biased;
        read = pipe.read_to_end(&mut bytes) => {
            read?;
            return Ok(bytes);
        }
        () = drain => {}
    }
    // A read that is waiting on its own reactor answers nothing yet even where the last writer closed at that very moment,
    // so the stream is asked for what it holds now: a pipe whose writers have all closed answers end of file here, and a
    //  writer that is still holding it answers nothing and leaves it held open.
    if reached_its_end(&mut pipe, &mut bytes) {
        return Ok(bytes);
    }
    let holders: Vec<ProcessUser> = held.map(holding_processes).unwrap_or_default();
    return Err(std::io::Error::other(OutputHeldOpen {
        waiting_for: waiting_for.to_owned(),
        holders,
        read: bytes,
    }));
}

/// What:
///  Reads what `pipe` holds right now into `bytes` and answers whether that was its end of file.
/// Why:
///  A read that is waiting on the reactor has not heard about a close yet,
/// so the drain can end a moment after the last writer
///  closed while the read still answers nothing;
/// the writes that closed are the step's whole answer,
/// and a step that finished inside its
///  ceiling must not be reported as one whose output a program holds open.
///  The descriptor is asked itself, and not the reactor:
/// every operation of the runtime answers a read of a stream it has not
///  yet heard about with nothing,
/// and the kernel's own answer is what decides here.
/// The receiver put the descriptor in
///  non-blocking mode when it was made (`Receiver::from_owned_fd`),
/// so a writer that holds the stream open answers nothing
///  rather than waiting.
///  One pipe holds at most `PIPE_CAPACITY` bytes,
/// so that much is the whole of what a stream that has closed can still
///  answer,
/// and a writer that supplies more is writing,
/// not closing:
/// the stream stays one the drain ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reachedItsEnd(descriptor: number, bytes: Uint8Array): boolean; // read until 0, EAGAIN, or the pipe's capacity
/// ```
fn reached_its_end(pipe: &mut Receiver, bytes: &mut Vec<u8>) -> bool {
    let mut chunk: Vec<u8> = vec![0; PIPE_CAPACITY];
    let mut swept: usize = 0;
    while swept < PIPE_CAPACITY {
        let room: usize = PIPE_CAPACITY.saturating_sub(swept);
        let into: &mut [u8] = chunk.get_mut(..room).unwrap_or_default();
        match rustix::io::read(pipe.as_fd(), into) {
            // Every writer has closed: what was read is the whole of the stream.
            Ok(0) => return true,
            // A writer still holds the write end, or wrote nothing this moment: the stream is one the drain ended.
            Err(_nothing_to_read) => return false,
            Ok(read) => {
                bytes.extend_from_slice(chunk.get(..read).unwrap_or_default());
                swept = swept.saturating_add(read);
            }
        }
    }
    return false;
}

/// The most bytes one pipe holds,
/// `pipe-max-size`'s default,
/// so a stream that has closed answers all of what it still holds within this.
const PIPE_CAPACITY: usize = 64 * 1024;

/// Awaits the moment the step ended,
/// which the wait for the child tells;
/// now when the wait is gone without telling it.
async fn wait_for_the_end(ends: &mut tokio::sync::watch::Receiver<Option<Instant>>) -> Instant {
    loop {
        if let Some(at) = *ends.borrow_and_update() {
            return at;
        }
        if ends.changed().await.is_err() {
            return Instant::now();
        }
    }
}

/// What:
///  The mount and inode of the open file `descriptor` names.
/// Why:
///   The two together name the pipe,
/// which is how a process holding its other end is found
/// (`process_use.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { stx_mnt_id, stx_ino } = statx(fd, '', AT_EMPTY_PATH, STATX_MNT_ID);
/// ```
#[cfg(target_os = "linux")]
pub(crate) fn pipe_identity(descriptor: std::os::fd::BorrowedFd<'_>) -> Option<(u64, u64)> {
    let found = rustix::fs::statx(descriptor, "", rustix::fs::AtFlags::EMPTY_PATH, rustix::fs::StatxFlags::MNT_ID).ok()?;
    return Some((found.stx_mnt_id, found.stx_ino));
}

/// Only Linux names a pipe,
/// so nothing that holds it is found elsewhere.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
fn pipe_identity(_descriptor: std::os::fd::BorrowedFd<'_>) -> Option<(u64, u64)> {
    return None;
}

/// The processes that have the pipe with this identity open,
/// this one aside.
fn holding_processes((mount, inode): (u64, u64)) -> Vec<ProcessUser> {
    let scan: UseScan = crate::process_use::processes_using(&UseTarget::File { mount, inode });
    return scan.users;
}

/// What:
///  The step's own ceiling,
/// carried by the wait for its output,
/// and why no watch ended the step before it.
/// Why:
///   A watch announces a wait and ends the child at its ceiling,
/// but the thread it needs can be refused,
///  and a
/// watch that runs is not allowed to end every step it waits for.
/// Whatever it could not do,
/// the wait for the output keeps the
///  ceiling,
/// so no step outlives it
/// (the third review's finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CarriedCeiling = { limit: Duration; why: 'no-watcher' | 'the-watch-could-not-end-it' };
/// ```
pub struct CarriedCeiling {
    /// The longest the step may run.
    pub limit: Duration,
    /// Why the wait,
    /// and not a watch,
    /// ends the step at that ceiling.
    pub why: CeilingWhy,
}

/// Why the wait for a step's output carries the step's own ceiling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CeilingWhy {
    /// No watch could run at all:
    /// the system refused the thread it needs.
    NoWatcher,
    /// A watch runs,
    /// and it could not end the step:
    /// nothing proved which process to stop,
    /// so this wait is what ends it at the ceiling.
    TheWatchCouldNotEndIt,
    /// Nothing watches this read and nothing else bounds it:
    /// it is a read-only query,
    /// and its ceiling is the longest one of those waits
    /// (`src/native/push/manual_push_range.rs`).
    TheQueryIsBounded,
}

/// What:
///  The ceiling a wait carries,
/// and why it is the wait that ends what it waits for.
/// Why:
///   A step with a ceiling may have no watch (a refused thread),
/// a watch that could not end it,
/// or no watch of
///  its own at all (a read-only query,
/// whose ceiling is the longest one of those waits).
/// Why:
///   The reason is the watch's to tell:
/// a watch that could not run at all,
/// or one that runs and could not end this step.
/// `None` in,
///  `None` out:
/// a step with no ceiling is waited for as before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// carriedFor(ceiling, aWatchRuns)
/// ```
pub fn carried_for(ceiling: Option<Duration>, a_watch_runs: bool) -> Option<CarriedCeiling> {
    let why: CeilingWhy = if a_watch_runs { CeilingWhy::TheWatchCouldNotEndIt } else { CeilingWhy::NoWatcher };
    return Some(carried_by(ceiling?, why));
}

/// The ceiling `limit` carried for the reason `why`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// carriedBy(limit, why)
/// ```
pub fn carried_by(limit: Duration, why: CeilingWhy) -> CarriedCeiling {
    return CarriedCeiling { limit, why };
}

/// What:
///  The sentence for a wait that carried the step's own ceiling because no watch could.
/// Why:
///   The watch is what ends a step,
/// and the system refused the thread that would have run it;
/// this wait then gives the step the
///  ceiling the watch would have given it and says why,
/// so nothing waits for a step for ever without a word
///  (`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
/// finding 8).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${waitingFor} ran for ${limit} seconds and was ${leftRunning ? 'left running' : 'ended'}; no watch could announce it: ${why}`
/// ```
fn waiting_carried_the_ceiling(waiting_for: &str, carried: CarriedCeiling, left_running: bool) -> String {
    let why: &str = match carried.why {
        CeilingWhy::NoWatcher => {
            "Nothing could watch it: the system refused a thread for the watcher. Hint: check the process limit (`ulimit -u`) \
             and the control group's `pids.max`, free some, and run the command again."
        }
        CeilingWhy::TheWatchCouldNotEndIt => {
            "Its watch could not end it: nothing proved which process to stop. Hint: check with `ps` what the step waits for, \
             and run the command again."
        }
        CeilingWhy::TheQueryIsBounded => {
            "Such a query changes nothing of the repository. Hint: check with `ps` what that program waits for, and run the \
             command again."
        }
    };
    let ending: &str = if left_running {
        "and could not be ended, so it was left running, and the system collects it when its call returns"
    } else {
        "and was ended"
    };
    return format!(
        "{waiting_for} ran for {} seconds, the longest the wrapper lets one step run, {ending}. {why}",
        carried.limit.as_secs()
    );
}

/// Controls for the read of a step's output stay out of the release executable.
#[cfg(test)]
#[path = "step_output_read_whole_through_pipes_tests.rs"]
mod tests;
