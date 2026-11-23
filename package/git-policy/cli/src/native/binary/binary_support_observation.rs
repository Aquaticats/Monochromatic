//! What:
//!  Running a wrapped command with a time bound and reading what it did:
//! its exit status,
//! its output without the wait notices and hints,
//! and the settling of the service work it started.
//! Why:
//!  The reading of a run needs no fixture beyond its directory,
//! so it lives in a file of its own that the helpers that start commands import.
//!  The bound is this wait's own promise and this wait keeps it:
//! `binary_support_observation_bound.rs` reads the bound the command's own line states,
//! ends the whole process group at it,
//! reaps the command,
//! reads the two pipes under one stated grace,
//! and says who ended the command;
//! this file is the faces a control calls.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const observed = observeRaw(command); // { code, stdout, stderr, signal }
//! ```

use std::ffi::{OsStr, OsString};
use std::os::unix::process::CommandExt as _;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use crate::support_observation_bound::{Answer, LineBound, line_bound_of, spelled_command, wait_under_bound};
use crate::support_program_names_and_time_bound::TIME_BOUND_SECONDS;
use git_policy_cli::work_item::{CLAIMED, PENDING, RECOVERING};

/// The signal `timeout --signal=KILL` re-raises on itself when the bound it was given was hit:
///  it kills
/// the command,
///  then re-raises the same signal on itself.
/// Why it is here:
///  the helpers that run a command under `/usr/bin/timeout` and read its status back need the mark
///  (`native_git_output`,
///  and `binary_resolution_tests` for a forwarding loop);
///  the shared wait never decides an ending from it alone,
///  because a command that kills itself with `SIGKILL` shows the same signal,
///  so the wait reads the status beside the bound the command's own line states,
///  and only a death at or after that bound is the bound's (`observe_with`).
pub const TIMED_OUT_SIGNAL: i32 = 9;

/// What one finished process showed its caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observed {
    /// Exit code,
    ///  absent when a signal ended the process.
    pub code: Option<i32>,
    /// Exact standard output bytes.
    pub stdout: Vec<u8>,
    /// Exact standard error bytes.
    pub stderr: Vec<u8>,
}

/// The bytes without their additive `"hint"` fields.
///  Most binary tests pin the incumbent's event shape,
/// which has no hint;
///  the hint text is checked by tests that use `observe_raw`.
pub fn without_hint_fields(bytes: &[u8]) -> Vec<u8> {
    let marker: &[u8] = b",\"hint\":\"";
    let mut kept: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index: usize = 0;
    while index < bytes.len() {
        if bytes[index..].starts_with(marker) {
            index += marker.len();
            while index < bytes.len() && bytes[index] != b'"' {
                index += if bytes[index] == b'\\' { 2 } else { 1 };
            }
            index += 1;
        } else {
            kept.push(bytes[index]);
            index += 1;
        }
    }
    return kept;
}

/// How every wait notice of the wrapper begins (`wait_progress.rs`,
/// `service_index_lock.rs`).
pub const WAIT_NOTICE_PREFIX: &[u8] = b"monochromatic-git-policy-cli: still waiting for ";

/// How the commit service's shorter notice begins,
/// the one a client whose own commit may be on the branch already reads (`service_busy.rs`).
/// The notice goes on with the command the service is inside,
/// after a comma,
/// so the prefix ends where the fixed words do.
pub const SERVICE_INSIDE_WRITE_NOTICE_PREFIX: &[u8] = b"monochromatic-git-policy-cli: this commit is inside a write of the commit service";

/// How the commit service's longer warning begins,
/// the one a client queued behind that write reads after minutes (`service_busy.rs`).
pub const SERVICE_STUCK_WRITE_NOTICE_PREFIX: &[u8] = b"monochromatic-git-policy-cli: the commit service has been inside a write ";

/// What:
///  Whether a line is one of the notices a wait of the wrapper prints.
/// Why:
///   A control that means the wrapper's words and not the host's speed drops the same lines the three
///  notices are written with.
pub fn is_a_wait_notice(line: &[u8]) -> bool {
    for prefix in [WAIT_NOTICE_PREFIX, SERVICE_INSIDE_WRITE_NOTICE_PREFIX, SERVICE_STUCK_WRITE_NOTICE_PREFIX] {
        if line.starts_with(prefix) {
            return true;
        }
    }
    return false;
}

/// What:
///  The bytes without the lines that are notices about a wait.
/// Why:
///  A step that outlasts its notice grace (5 seconds for a query,
/// 10 for a bulk step) prints a notice on standard
///  error however healthy it is,
/// and a loaded host stretches any step past that.
/// The commit service tells a client whose own commit is in its branch write that it is inside it,
/// which the owner
///  asked for (2026-11-04,
/// "Short extra wait":
/// the commit may be on the branch already,
/// so the client is told when the write starts),
/// and a beat falls inside a write a loaded host stretched past a second even when the commit lands,
/// so a control that means the words drops that notice;
/// it is a sign of the host's speed and not of the command's.
/// The longer warning of the same write,
/// which a client queued behind it reads after minutes,
/// goes too.
/// A control that compares the
///  wrapper's diagnostics with native Git's words means the words,
/// not that the host was fast,
/// so it compares what
///  is left.
/// Only a line that is a wait notice goes;
/// every other line stays,
/// and a control that
///  asserts a notice reads the raw bytes.
pub fn without_wait_notices(bytes: &[u8]) -> Vec<u8> {
    return bytes
        .split_inclusive(|byte| { return *byte == b'\n'; })
        .filter(|line| { return !is_a_wait_notice(line); })
        .flatten()
        .copied()
        .collect();
}

/// What:
///  Run a command to completion under the bound its own line states and observe it without its hint fields,
///  feeding `input` to its standard input.
/// Why:
///  The bound,
///  the ending and the pipes are kept by `observe_with`,
///  once for this face and the two below it.
///
/// # Panics
///  As `observe_raw`.
pub fn observe(command: &mut Command, input: &[u8]) -> Observed {
    let raw: Observed = observe_raw(command, input);
    return Observed {
        code: raw.code,
        stdout: without_hint_fields(raw.stdout.as_slice()),
        stderr: without_hint_fields(raw.stderr.as_slice()),
    };
}

/// What a command run under the shared test hang guard (`TIME_BOUND_SECONDS`) asks the wrapper to do,
/// read the way the
///  wrapper reads it.
pub enum SharedBoundMisuse {
    /// A commit the wrapper lands through its transaction.
    CommitTransaction,
    /// A `--help` that makes Git render a manual page through `man` and the groff programs.
    ManualPage,
}

/// What:
///  Whether `command` runs under the shared bound (`timeout --signal=KILL 5 <program> <arguments>`) with
///  arguments that need a longer bound or a stub instead;
/// absent for any other command.
/// Why:
///  The shared hang guard is for single Git commands,
/// and is not a limit on how long the wrapper may take.
///  A landed commit starts 40 Git processes and makes 36
///  sync calls (`binary_budget_tests.rs`),
/// against one process for direct Git,
/// and a manual page starts 9 more
///  (`man`,
/// `preconv`,
/// `tbl`,
/// `nroff`,
/// `col`,
/// `groff`,
/// `troff`,
/// `grotty`),
/// so under host load either one can pass
///  the bound while the policy it is there to test works.
/// This check does not depend on timing,
/// so a misuse fails
///  on every host,
/// loaded or quiet,
/// instead of once a day on a busy one.
pub fn shared_bound_misuse(command: &Command) -> Option<SharedBoundMisuse> {
    let arguments: Vec<OsString> = command.get_args().map(OsStr::to_os_string).collect();
    let [signal, bound, program, wrapped @ ..] = arguments.as_slice() else {
        return None;
    };
    if signal != "--signal=KILL" || bound != TIME_BOUND_SECONDS {
        return None;
    }
    let stripped = git_policy_cli::wrapper_invocation::strip_wrapper_controls(wrapped);
    if stripped.layout.outcome != git_policy_cli::global_arguments::GlobalOutcome::Command {
        return None;
    }
    let region: &[OsString] = git_policy_cli::wrapper_invocation::command_region(&stripped);
    let word: &[u8] = git_policy_cli::wrapper_invocation::command_word(&stripped);
    if word == b"commit"
        && git_policy_cli::command_commit::parse_commit_region(region, &[]).is_ok_and(|facts| { return !facts.dry_run; })
    {
        return Some(SharedBoundMisuse::CommitTransaction);
    }
    let has_manual_stub: bool = Path::new(program)
        .parent()
        .is_some_and(|directory| { return directory.join("man").exists(); });
    let asks_for_help: bool = region
        .iter()
        .take_while(|token| { return *token != "--"; })
        .any(|token| { return token == "--help"; });
    if word != b"monochromatic-git-policy-cli" && word != b"cli-git" && asks_for_help && !has_manual_stub {
        return Some(SharedBoundMisuse::ManualPage);
    }
    return None;
}

/// What:
///  Waits for a spawned child with its input and its output moving from the first moment,
/// claims the supervisor's kill of it
///  if there is one,
/// and only then runs `cleanup`.
/// Why:
///   The input is written by a thread of its own,
/// so a child that fills its output before it reads all its input is not
///  deadlocked by the test,
/// and both pipes are drained on threads:
/// a command that writes much is not idle.
///  The kill is claimed before the cleanup because the cleanup can take longer than the grace in which an unclaimed kill
///  aborts the process.
///  This child is not one this module started and leads no process group of its own,
///  so `LineBound::UNSTATED` ends the child alone;
///  the supervisor's controls use this wait for a child they spawn themselves.
///
/// # Panics
///  With the supervisor's message when it ended the command.
///  When a process that outlived the child still holds one of its pipes after `PIPE_GRACE`.
pub fn supervised_wait(child: Child, input: &[u8], cleanup: impl FnOnce()) -> Output {
    let answer: Answer = wait_under_bound(child, None, LineBound::UNSTATED, input);
    // The supervisor ended it:
    // a loop or a deadlock,
    // with the reason and the tasks of the tree in the message.
    if let Some(message) = answer.supervisor_message {
        panic!("{message}");
    }
    cleanup();
    answer.refuse_a_second_ending("the command this control started", None);
    return Output { status: answer.status, stdout: answer.stdout, stderr: answer.stderr };
}

/// Run a command to completion with every byte of its output kept,
///  feeding `input` to its standard input.
/// # Panics
///  When the command is a commit transaction or a manual page under the shared bound;
///  see `shared_bound_misuse`.
///  When the supervisor ended the command,
///  with the supervisor's message.
///  When this wait ended the command at the bound,
///  or a process that outlived the command still holds one of its
///  pipes;
///  see `Answer::refuse_a_second_ending`.
pub fn observe_raw(command: &mut Command, input: &[u8]) -> Observed {
    return observe_with(command, input, true);
}

/// What:
///  Run a command to completion and observe it without waiting for the after-commit work the commit service runs.
/// Why:
///  Every observed command waits for that work (`settle_service_work`),
/// because most controls read what the push or the upkeep did after the command returned.
/// A control about what the command itself said,
/// or about the service's first attempt at a step whose record the service tries again at the end of a claim's lease,
/// would wait minutes for tries it does not mean to read,
/// and a lease is 120 seconds (`CLAIM_LEASE`).
///
/// # Panics
///  As `observe_raw`.
pub fn observe_without_waiting_for_the_after_commit_work(command: &mut Command, input: &[u8]) -> Observed {
    return observe_with(command, input, false);
}

/// What:
///  The body both faces above share:
///  one run of a command,
///  and the wait for the after-commit work given or refused.
/// Why:
///  The faces differ only in that wait and in the descriptor they hand,
/// so the bound,
/// the misuse check and the reading live once.
///  The command leads a process group of its own,
///  which is the group the bound ends;
///  `timeout` makes such a group for itself a moment after it starts (`setpgid` in its own process),
///  and this wait makes it before the command runs,
///  so the group it ends is one it made.
///  The line's own `timeout` and this wait keep one value and the same start
///  (`remaining_of`):
///  this wait's deadline is the earlier of the two,
///  and a command its own `timeout` ends at that bound is still the bound's doing (`Ending::AtTheBound`).
///  The bounds the line states and the wait keeps go to the wait as one value (`line_bound_of`),
///  so an exit this wait reads before that deadline names the same ending
///  (`ending_of_an_exit_read_first`).
///
/// # Panics
///  When the command is a commit transaction or a manual page under the shared bound;
///  see `shared_bound_misuse`.
///  When the supervisor ended the command,
///  with the supervisor's message.
///  When this wait ended the command at the bound,
///  or a pipe the command's output came through is still held;
///  see `Answer::refuse_a_second_ending`.
fn observe_with(command: &mut Command, input: &[u8], waiting_for_the_after_commit_work: bool) -> Observed {
    match shared_bound_misuse(command) {
        Some(SharedBoundMisuse::CommitTransaction) => panic!(
            "a commit under the shared bound: the work of a landed commit happens in the commit service, outside the command's \
             process tree, so the supervisor would see a tree that waits and end it; run it with `run_commit` (the {}-second \
             wall-clock bound of binary_commit_tests.rs)",
            super::commit::COMMIT_BOUND_SECONDS
        ),
        Some(SharedBoundMisuse::ManualPage) => panic!(
            "a `--help` under the shared bound renders a manual page through man and groff (9 process starts) that is Git's \
             work and the host's `man`; put an executable `man` stub in the fixture's bin directory first (`stub_manual_viewer`)"
        ),
        None => {}
    }
    let spelled: String = spelled_command(command);
    let directory: Option<PathBuf> = command.get_current_dir().map(Path::to_path_buf);
    command.process_group(0);
    let child: Child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("process starts");
    let child_id: u32 = child.id();
    let group: libc::pid_t = libc::pid_t::try_from(child_id).expect("a process ID fits the pid type");
    let answer: Answer = wait_under_bound(child, Some(group), line_bound_of(command), input);
    // The supervisor ended it: a loop or a deadlock, and its message names the reason and the tasks of the tree.
    //  The wait claimed that message before it reaped the command,
    //  and the cleanup runs after it here, because the cleanup can take longer than the grace in which an
    //  unclaimed kill aborts the process.
    if let Some(message) = answer.supervisor_message {
        panic!("{message}");
    }
    if waiting_for_the_after_commit_work && let Some(at) = directory.as_deref() {
        settle_service_work(at);
    }
    answer.refuse_a_second_ending(spelled.as_str(), directory.as_deref());
    return Observed { code: answer.status.code(), stdout: answer.stdout, stderr: answer.stderr };
}

/// How long a control waits for the after-commit work (the push,
/// the upkeep) the commit service runs once the commit returned.
pub const SERVICE_WORK_PATIENCE: std::time::Duration = std::time::Duration::from_secs(60);

/// The pause between two samples of the work queue the wait reads.
const SAMPLE_PAUSE: std::time::Duration = std::time::Duration::from_millis(20);

/// What:
///  The folders a record of this queue is in flight in,
///  in the order a record moves through them.
/// Why:
///   `take_entry` (`work_item.rs`)
/// claims a record in three steps:
/// a rename from `pending` into `recovering`,
/// a stamp there,
/// and a rename into `claimed`,
/// and a claim settles from `claimed` into `done` or `failed`.
/// A sample that reads these in the order a record moves through them cannot miss a record moving forward
/// during that sample,
/// because a forward move lands it in a folder the sample has not read yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const inFlight = ['pending', 'recovering', 'claimed'] as const;
/// ```
pub const A_RECORD_IN_FLIGHT_FOLDERS: [&str; 3] = [PENDING, RECOVERING, CLAIMED];

/// What:
///  A step a control runs before each folder of one sample is read,
///  and no release run installs one.
/// Why:
///  The window this wait closes is one record moving between two folders while a sample reads them,
/// and only a step between the reads can move the record there,
/// so the control of the wait installs one (`binary_support_observation_tests.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BeforeAFolderRead = (folder: string) => void
/// ```
type BeforeAFolderRead = fn(folder: &str);

thread_local! {
    /// The step the running control installed,
    /// and nothing in an ordinary run.
    static BEFORE_A_FOLDER_READ: std::cell::Cell<Option<BeforeAFolderRead>> = const { std::cell::Cell::new(None) };
}

/// What:
///  Install the step a sample runs before it reads each folder,
///  or take the step away with `None`.
/// Why:
///  A control that means to move a record between two folders of one sample has to move it between two reads,
///  and this is the only point between them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// beforeEachFolderRead(step);
/// ```
pub fn run_before_each_folder_read(step: Option<BeforeAFolderRead>) {
    BEFORE_A_FOLDER_READ.with(|installed| { installed.set(step); });
}

/// What:
///  Whether a record of the queue is in one of the folders it is in flight in,
///  read in the order a record moves through them.
/// Why:
///   See `A_RECORD_IN_FLIGHT_FOLDERS`,
/// and `settle_service_work` for why a second such sample is asked for as well.
/// A folder that cannot be read holds nothing,
/// as before.
fn a_record_is_in_flight(work: &Path) -> bool {
    for folder in A_RECORD_IN_FLIGHT_FOLDERS {
        let installed = BEFORE_A_FOLDER_READ.with(std::cell::Cell::get);
        if let Some(step) = installed {
            step(folder);
        }
        if std::fs::read_dir(work.join(folder)).map_or(0, Iterator::count) > 0 {
            return true;
        }
    }
    return false;
}

/// The controls of the wait above:
/// the endings a command can come back with,
/// a pipe a descendant holds,
/// and a
/// command that writes more than a pipe holds.
#[cfg(test)]
#[path = "binary_support_observation_tests.rs"]
mod tests;

/// What:
///  The Git directory of the worktree `directory` is in:
///  the first `.git` on the way up,
///  followed to the directory it names when that `.git` is a file.
/// Why:
///   A linked worktree has no `.git` directory of its own;
/// it holds a file with one `gitdir:` line naming its administrative directory,
///  where its own queue of after-commit work lives.
/// A wait that looked for a directory alone returned at once for such a worktree,
///  so a control that committed in one read the origin before its push landed (issue 6336).
///  The named path is read as Git wrote it:
///  a trailing space belongs to the directory's name
///  (`a_commit_lands_in_a_worktree_whose_git_directory_name_ends_with_a_space`),
///  and a relative path is relative to the directory that holds the file.
fn git_directory_of(directory: &Path) -> Option<PathBuf> {
    for candidate in directory.ancestors() {
        let marker: PathBuf = candidate.join(".git");
        if marker.is_dir() {
            return Some(marker);
        }
        if !marker.is_file() {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(marker.as_path()) else { return None };
        let named: &str = text.lines().next().and_then(|line| { return line.strip_prefix("gitdir: "); })?;
        if named.is_empty() {
            return None;
        }
        let path: PathBuf = PathBuf::from(named);
        return Some(if path.is_absolute() { path } else { candidate.join(path) });
    }
    return None;
}

/// Every state directory of a work directory with the names it holds,
/// for the failure message of a wait that ran out.
pub fn work_listing(work: &Path) -> String {
    return ["pending", "claimed", "recovering", "done", "failed"]
        .iter()
        .map(|state| {
            let mut names: Vec<String> = std::fs::read_dir(work.join(state))
                .map(|listing| { return listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect(); })
                .unwrap_or_default();
            names.sort();
            return format!("{state}={names:?}");
        })
        .collect::<Vec<String>>()
        .join(" ");
}

/// What:
///  Waits until the commit service has settled the after-commit work of the repository around `directory`.
/// Why:
///   A commit that landed through the service returns before its push and its upkeep run,
/// so a control that inspects the
///  origin,
/// the packs or the push's failure after the command must first wait for the records to leave `pending`,
///  `claimed` and `recovering`;
/// it waits only where a work directory exists.
/// The repository is the first `.git` on the way up,
/// which a linked worktree keeps as a file that names its own Git directory
/// (`git_directory_of`),
/// so the wait covers a commit made in any worktree of the repository.
///  This wait is `SERVICE_WORK_PATIENCE`,
///  which is shorter than one claim's lease (`CLAIM_LEASE`,
///  120 seconds):
/// a record whose handler answered with a retry keeps its claim until that lease ends,
/// so a control that waits here cannot see such a record settle.
/// A control that means to read one attempt of such a record reads that instead
/// (`observe_without_waiting_for_the_after_commit_work`).
///  The folders are read in the order a record moves through them,
/// and an empty reading must be confirmed by a second one,
/// because a record being claimed moves between two of them while a sample reads them
/// (issue 6345,
///  and `A_RECORD_IN_FLIGHT_FOLDERS` for the moves the code makes).
pub fn settle_service_work(directory: &Path) {
    let Some(git_dir) = git_directory_of(directory) else {
        return;
    };
    let work: PathBuf = git_dir.join(git_policy_cli::work_item::WORK_DIRECTORY);
    let began: std::time::Instant = std::time::Instant::now();
    // The flow order alone closes the moves that go forward, and the second sample the ones that go back;
    // the third closes a recovery whose two renames straddle two samples.
    // Every move a record makes between these folders, from the code:
    // `take_entry` (`work_item.rs`) moves `pending` into `recovering` and then into `claimed`,
    // and moves a record it could not publish from `recovering` back into `pending`,
    // takes a `claimed` record of a later release back into `pending`, and settles one in `failed`;
    // `quarantine_pending` moves `pending` into `failed`;
    // `settle_named` moves a claim into `done` or `failed`, and `renew_claim` renames a claim inside `claimed`;
    // `recover` (`work_item_recovery.rs`) moves a `claimed` or `recovering` record into `failed` or, under a new name,
    // into `recovering`, and from there into `pending` or `failed`;
    // `service_handoff` moves a held or a pending record into `done`,
    // `promote` moves a held record into `pending`, and `requeue_after_giving_up` moves a failed one into `pending`.
    // Arrivals into `pending` from outside are written by commands (`enqueue_all`), never by a service round,
    // and a record in `held` is not waited for.
    // What three samples do not close, measured by the second integration pass over schedules of one record's moves
    // placed among the wait's reads (`keep/probes/wait-every-schedule.txt` of its folder):
    // they closed all 3350 schedules of up to three moves, and all but 4 of 18365 of up to four.
    // So a chain of four or more moves that a thread's stall stretches across every sample of this wait is still missed,
    // and so is a record renamed inside one folder while that folder is read.
    while a_record_is_in_flight(work.as_path()) || a_record_is_in_flight(work.as_path()) || a_record_is_in_flight(work.as_path()) {
        assert!(began.elapsed() < SERVICE_WORK_PATIENCE, "the service never settled the after-commit work in {}: {}", work.display(), work_listing(work.as_path()));
        std::thread::sleep(SAMPLE_PAUSE);
    }
}
