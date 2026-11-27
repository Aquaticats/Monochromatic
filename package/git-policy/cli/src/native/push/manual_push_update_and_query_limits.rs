//! What:
//!  What a push would update:
//!  the public update of one pushed reference,
//!  the limits on how many queries of a push probe run side by side,
//!  and the batch query whose answer a range and its published tips are read with.
//! Why:
//!  The probe in `manual_push_probe.rs`,
//!  the range in `manual_push_range.rs`
//!  and the lifecycle that reads the scopes all need these;
//!  none of them uses the probe,
//!  so they sit in a file of their own and `manual_push_probe.rs` re-exports them.
//!  The query moved here too,
//!  because a query that reads a push's own objects is what a push that would be sent is read with,
//!  and the range names it from here rather than the other way round.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type PushUpdate = { remoteName: string; remoteOid?: string; to: string; ... };
//! const MOST_CONCURRENT_REMOTE_QUERIES = 4;
//! const bytes = await runGitBytes(git, ['rev-list', tip, '--not', ...published], undefined);
//! ```

/// The Git runner the batch query starts,
/// the cause a failure names,
/// and the start that rides out a short shortage of processes.
use crate::auto_push_single_flight::{
    PushGit,
    git_command,
};
use crate::manual_push_cause::ManualPushCause;
use crate::start_retry::RetryStart;
/// Owned OS strings for Git arguments and the stdin of batch queries.
use std::ffi::OsString;
/// The child a query starts,
/// handed to the thread that reads its answers.
use std::process::Child;
use std::process::Stdio;
/// How long a range query may run,
/// and the clock the thread that writes its list measures that against.
use std::time::{
    Duration,
    Instant,
};

/// What:
///  One confirmed push update,
/// the public fact the policies read.
/// Why:
///   An absent value is `None`,
/// which replaces the incumbent's `ABSENT_GIT_VALUE` sentinel.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PushUpdate = { localOid: string | typeof ABSENT_GIT_VALUE; remoteOid: string | typeof ABSENT_GIT_VALUE; remoteName; remoteRef };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PushUpdate {
    /// Local object name;
    /// `None` for a deletion.
    pub local_oid: Option<String>,
    /// Destination object name confirmed by `ls-remote`;
    /// `None` when the ref is absent.
    pub remote_oid: Option<String>,
    /// Destination remote name.
    pub remote_name: String,
    /// Destination ref.
    pub remote_ref: String,
}

/// The most per-destination queries that open a connection to a remote and run at once (`git ls-remote` of each destination):
///  the bound `SPEC.md` gives,
/// four.
/// The remote side (how many connections a server tolerates) was not measured,
/// so the value is
///  the one the SPEC already gave;
/// it is the coordinator's pick under the owner's delegation,
/// open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Picked for these follow-ups",
/// concurrent per-update queries,
/// option B).
pub const MOST_CONCURRENT_REMOTE_QUERIES: usize = 4;

/// The most per-update queries of the local repository that run at once (`git cat-file` and `git rev-list` of each update).
/// Measured
///  on 16 branch tips of this repository,
/// medians of three runs in alternating orders,
/// other sessions running:
/// `git ls-tree -r`
///  took 218,
/// 114,
/// 80,
/// 50 and 46 ms at 1,
/// 2,
/// 4,
/// 8 and 16 at once,
/// and `git rev-list` of a whole branch 845,
/// 413,
/// 231,
/// 190 and 194
///  ms,
/// so eight is where the gain stops.
/// The coordinator's pick under the owner's delegation,
/// open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Picked for these follow-ups",
/// concurrent per-update queries,
/// option B).
pub const MOST_CONCURRENT_LOCAL_QUERIES: usize = LOCAL_QUERY_GAIN_STOPS_AT;

/// The number of queries at once at which the measured gain of local queries stopped (see `MOST_CONCURRENT_LOCAL_QUERIES`).
pub const LOCAL_QUERY_GAIN_STOPS_AT: usize = 8;

/// The bytes one query holds at its peak when it lists the whole history of this repository:
/// the largest resident size of
///  `git rev-list --all` in three runs (29,164 KiB,
/// 29,032 KiB and 28,848 KiB,
/// so 29,863,936 bytes) plus the 1,413,721 bytes of
///  its output,
/// which the wrapper reads whole (34,481 commits on 2026-11-03).
/// `git ls-tree -r` of the main tree peaked at 26,080
///  KiB with 2,459,743 bytes of output.
pub const MEASURED_WHOLE_HISTORY_QUERY: u64 = 29_863_936 + 1_413_721;

/// The most bytes one write of the request list hands the kernel,
/// the pipe buffer POSIX promises is written without waiting for a reader.
const PIPE_BUFFER: usize = 4096;

/// What:
///  Why the range could not be computed.
/// Why:
///   Every variant stops the lifecycle with a failure event;
/// a guessed range would hide commits from the scan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ManualPushProbeError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManualPushRangeError {
    /// Diagnostic naming the failed query.
    pub message: String,
    /// What the person can do about it,
    /// which the hint is chosen by.
    pub cause: ManualPushCause,
}

impl std::fmt::Display for ManualPushRangeError {
    /// Writes the diagnostic text.
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        return formatter.write_str(&self.message);
    }
}

impl std::error::Error for ManualPushRangeError {}

/// Build a range error from text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new ManualPushProbeError(message)
/// ```
pub(crate) fn range_error(
    cause: ManualPushCause,
    message: String,
) -> ManualPushRangeError {
    return ManualPushRangeError {
        message,
        cause,
    };
}

/// The longest one of these read-only queries runs before it is ended:
/// the ceiling of a quick Git step of the wrapper
/// (`src/native/service/wait_ceilings.rs`),
/// because each of them reads the object store or the refs and answers at once.
/// A query that is slower than that is reported rather than waited for longer.
const RANGE_QUERY_CEILING: Duration = crate::wait_ceilings::PLUMBING_STEP_CEILING;

/// What:
///  Run a Git query with stdin text and return its exact stdout bytes.
/// Why:
///   Batch queries (`cat-file --batch-check`,
/// `rev-list --stdin`) read their requests from stdin,
///  and any Git failure is an error,
/// never an empty answer.
///  The request list goes in from a thread of its own while the answers are read on another,
/// because Git reads
///  and answers in one loop:
/// a list larger than a pipe was written before anything was read,
/// so a full reply pipe
///  stopped Git reading the list and the write waited for it for good
/// (`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
/// finding 9).
///  The wait carries `RANGE_QUERY_CEILING`,
/// so a Git that never answers is reported instead of waited for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const status = await withDeadline(runWithInputFromAnotherThread(command, input), ceiling);
/// ```
pub fn run_git_bytes(
    git: &PushGit,
    arguments: &[OsString],
    input: Option<&str>,
) -> Result<Vec<u8>, ManualPushRangeError> {
    return run_git_bytes_starting_with(git, arguments, input);
}

/// What:
///  The body of `run_git_bytes`:
/// the request list on a thread of its own,
/// and the answers read,
/// and the query ended at its ceiling,
/// by the thread that reads them.
/// Why:
///  Git reads and answers in one loop,
/// so neither side can wait for the other;
/// and the thread that reads the answers owns the
///  child while it waits,
/// so the ceiling's kill reaches that child and never a number another thread
///  could have reaped after it ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runGitBytesWithinItsThreads(git, args, input): Buffer;
/// ```
fn run_git_bytes_starting_with(
    git: &PushGit,
    arguments: &[OsString],
    input: Option<&str>,
) -> Result<Vec<u8>, ManualPushRangeError> {
    let mut command = git_command(git, arguments);
    // The two pipes of the query's answers are made and registered with the runtime before the start, so a refusal starts
    //  nothing, and both streams are read by the runtime in one place that holds the cap-free read and the post-exit drain.
    // This is the reader `process/step_output_read_whole_through_pipes.rs` already uses for a step's streams
    // (the fourth review's findings about reader threads, their unbounded queues and their blocked reads).
    let pipes: crate::step_output_read_whole_through_pipes::StepPipes =
        match crate::step_output_read_whole_through_pipes::attach_pipes(&mut command) {
            Ok(opened) => opened,
            Err(error) => {
                return Err(range_error(
                    ManualPushCause::GitNotStarted,
                    format!("git could not be given its answers: {error}"),
                ));
            }
        };
    // The request list goes in through a pipe of this process's own, so its write end is this process's and a refused thread
    //  start costs the query its start rather than leaving a child nothing waits for.
    let (reader, writer) = match std::io::pipe() {
        Ok(pair) => pair,
        Err(error) => {
            return Err(range_error(
                ManualPushCause::GitNotStarted,
                format!("git could not be given its input: {error}"),
            ));
        }
    };
    // The input is written on this thread while the runtime reads both replies: neither can hold the other up, which is what a
    //  request list larger than a pipe did before. What became of that write is told back, because a list that was cut short
    //  leaves a partial answer that only its own report can name (the fourth review's finding 7).
    let (wrote, watch_the_write) = std::sync::mpsc::channel::<bool>();
    let writing: Option<std::thread::JoinHandle<()>> = match input {
        Some(requests) => {
            let text: String = requests.to_owned();
            let mut writing_end = writer;
            let started = std::thread::Builder::new()
                .name(String::from("cli-git-range-input"))
                .spawn(move || {
                    // A Git that exits before reading the whole list surfaces through its status, and one that never reads it has
                    //  its own ceiling: the write gives up when that passes rather than waiting for the reader's life.
                    let wrote_all: bool =
                        write_within(&mut writing_end, text.as_bytes(), RANGE_QUERY_CEILING);
                    let _reported = wrote.send(wrote_all);
                });
            match started {
                Ok(written) => {
                    command.stdin(Stdio::from(reader));
                    Some(written)
                }
                Err(error) => {
                    return Err(range_error(
                        ManualPushCause::GitNotStarted,
                        format!(
                            "git was not started, because no thread could be given its request list: {error}. Hint: check the process limit \
                     (`ulimit -u`) and the control group's `pids.max`, free some, and run the command again"
                        ),
                    ));
                }
            }
        }
        None => {
            drop(writer);
            drop(reader);
            command.stdin(Stdio::null());
            None
        }
    };
    let spawned = command.spawn_retrying();
    // The write ends of the answers are this process's until the command lets them go, and a pipe whose write end this process
    //  holds never reaches its end: the readers are given them back whether or not the start worked.
    crate::step_output_read_whole_through_pipes::release_write_ends(&mut command);
    let child: Child = match spawned {
        Ok(running) => running,
        Err(error) => {
            return Err(range_error(
                ManualPushCause::GitNotStarted,
                format!("git could not be started: {error}"),
            ));
        }
    };
    // The runtime reads both streams and waits for the child, and this wait carries the ceiling: one place holds the read, the
    //  post-exit drain and the end of the wait, so no reader thread and no queue stands between the query and its answer.
    let shown: String = words_of(arguments);
    let carried: crate::step_output_read_whole_through_pipes::CarriedCeiling =
        crate::step_output_read_whole_through_pipes::carried_by(
            RANGE_QUERY_CEILING,
            crate::step_output_read_whole_through_pipes::CeilingWhy::TheQueryIsBounded,
        );
    let answered: std::process::Output = match pipes.read_until_the_child_ends(
        child,
        shown.as_str(),
        Some(carried),
    ) {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::TimedOut => {
            // A query the kernel would not release is reported as left running, not as ended (the fourth review's finding 9).
            let left_running: bool =
                crate::child_waited_for_within_a_ceiling::left_running_when_the_ceiling_was_reached(
                    &error,
                );
            let ending: &str = if left_running {
                "and could not be ended, so it was left running"
            } else {
                "and was ended"
            };
            return Err(range_error(
                ManualPushCause::GitSilent,
                format!(
                    "git {shown} did not answer within {} seconds, the longest one of these queries waits, {ending}. Hint: check with \
                 `ps` what that Git waits for, and run the command again.",
                    RANGE_QUERY_CEILING.as_secs()
                ),
            ));
        }
        Err(error) => {
            return Err(range_error(
                ManualPushCause::GitSilent,
                format!("git did not finish: {error}"),
            ));
        }
    };
    // What: the thread that writes the request list is not joined, and what became of its write is read here.
    // Why:  its write ends when the query's input closes, which the query's own end does: the next write fails with `EPIPE` and
    //       the thread ends. A program the query started that holds the read end open past that keeps this thread alive until it
    //       ends or this process exits, and nothing of the command waits for it.
    let _input_thread_left_to_end = writing;
    // Nothing waits for that report: a writer still in its write has not put the whole list in, and the answer at hand covers
    //  only the part of it that went (the fourth review's finding 7).
    let wrote_all: bool = watch_the_write.try_recv().unwrap_or(false);
    if input.is_some() && !wrote_all {
        // Git answers what it read, and a list that was cut short leaves answers to part of it: that is not this query's answer
        //  (the fourth review's finding 7).
        return Err(range_error(
            ManualPushCause::GitSilent,
            format!(
                "git {shown} answered, but the request list this command wrote was cut short at its {} second deadline, so the \
             answers cover only the part of the list that went in. Hint: check with `ps` what that Git waits for, and run the \
             command again.",
                RANGE_QUERY_CEILING.as_secs()
            ),
        ));
    }
    if !answered.status.success() {
        let mut words: Vec<String> = Vec::new();
        for argument in arguments {
            words.push(argument.to_string_lossy().into_owned());
        }
        return Err(range_error(
            ManualPushCause::ContentMissing,
            format!(
                "git {} failed: {}",
                words.join(" "),
                String::from_utf8_lossy(&answered.stderr).trim_end()
            ),
        ));
    }
    return Ok(answered.stdout);
}

/// What:
///  Writes every byte of `text` to `writer`,
/// giving up once `within` has passed,
/// and says whether all of it went.
/// Why:
///  A `write_all` of a list larger than a pipe waits for the reader,
/// and a program the query started can hold the read
///  end of that input without ever reading it,
/// so one blocked call outlived the query by as long as that program lived.
///  Waiting for the pipe to take more,
/// with the query's own ceiling as the deadline,
/// ends the thread with the query.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeWithin(writer: Writable, text: Buffer, within: number): boolean;
/// ```
pub(crate) fn write_within(
    writer: &mut std::io::PipeWriter,
    text: &[u8],
    within: Duration,
) -> bool {
    use std::io::Write as _;
    let began: Instant = Instant::now();
    let mut written: usize = 0;
    while written < text.len() {
        let left: Duration = match within.checked_sub(began.elapsed()) {
            Some(left) if !left.is_zero() => left,
            _ => return false,
        };
        let mut waiting: libc::pollfd = libc::pollfd {
            fd: std::os::fd::AsRawFd::as_raw_fd(writer),
            events: libc::POLLOUT,
            revents: 0,
        };
        let millis: libc::c_int =
            libc::c_int::try_from(left.as_millis()).unwrap_or(libc::c_int::MAX);
        // SAFETY: `waiting` is one valid `pollfd` for the call, which reads and writes only that.
        let ready: libc::c_int = unsafe { libc::poll(&raw mut waiting, 1, millis) };
        if ready < 0 {
            return false;
        }
        // A pipe that took nothing within the deadline is the end of this write: writing again would wait in the kernel with no
        //  end of its own, which is what this deadline exists to prevent.
        if ready == 0 {
            return false;
        }
        // One pipe buffer at a time: a larger write in blocking mode waits in the kernel until all of it is taken, which is
        //  the call this deadline exists to bound.
        let room: usize = (text.len() - written).min(PIPE_BUFFER);
        match writer.write(&text[written..written + room]) {
            Ok(0) => return false,
            Ok(took) => written += took,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            // A query that ended before reading the whole list is the status its caller reports, not a failure of the write.
            Err(_ended) => return false,
        }
    }
    return true;
}

/// One argument as the words a message shows it with.
fn word_of(argument: &OsString) -> String {
    return argument.to_string_lossy().into_owned();
}

/// The words of an argument list,
/// for a message that names the command.
fn words_of(arguments: &[OsString]) -> String {
    return arguments
        .iter()
        .map(word_of)
        .collect::<Vec<String>>()
        .join(" ");
}

/// Split Git line output into non-empty lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.split('\n').filter((line) => line.length > 0)
/// ```
pub(crate) fn lines_of(bytes: &[u8]) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for line in String::from_utf8_lossy(bytes).split('\n') {
        if !line.is_empty() {
            lines.push(line.to_owned());
        }
    }
    return lines;
}
