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
use crate::support_observation_bound::{Answer, UNSTATED_BOUND, spelled_command, stated_bound, wait_under_bound};
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
///  the shared wait no longer reads it,
///  because a command that kills itself with `SIGKILL` shows the same signal,
///  and only this wait's own act can say that the bound ended a command (`observe_with`).
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
///  so `UNSTATED_BOUND` ends the child alone;
///  the supervisor's controls use this wait for a child they spawn themselves.
///
/// # Panics
///  With the supervisor's message when it ended the command.
///  When a process that outlived the child still holds one of its pipes after `PIPE_GRACE`.
pub fn supervised_wait(child: Child, input: &[u8], cleanup: impl FnOnce()) -> Output {
    let answer: Answer = wait_under_bound(child, None, UNSTATED_BOUND, input);
    // The supervisor ended it:
    // a loop or a deadlock,
    // with the reason and the tasks of the tree in the message.
    if let Some(message) = answer.supervisor_message {
        panic!("{message}");
    }
    cleanup();
    answer.refuse_a_second_ending(UNSTATED_BOUND, "the command this control started", None);
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
    let bound: std::time::Duration = stated_bound(command);
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
    let answer: Answer = wait_under_bound(child, Some(group), bound, input);
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
    answer.refuse_a_second_ending(bound, spelled.as_str(), directory.as_deref());
    return Observed { code: answer.status.code(), stdout: answer.stdout, stderr: answer.stderr };
}

/// How long a control waits for the after-commit work (the push,
/// the upkeep) the commit service runs once the commit returned.
pub const SERVICE_WORK_PATIENCE: std::time::Duration = std::time::Duration::from_secs(60);

/// The controls of the wait above:
/// the endings a command can come back with,
/// a pipe a descendant holds,
/// and a
/// command that writes more than a pipe holds.
#[cfg(test)]
#[path = "binary_support_observation_tests.rs"]
mod tests;

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

/// How long a wait for the after-commit work pauses before it looks again:
///  the pause between its two looks,
/// and the pause between one look and the next while work is still queued.
/// Why this value:
///  it is the pause this wait already took between its looks,
/// so the two looks are a poll apart,
/// and a record the service puts back into `pending` (or a record that arrives)
/// after the first look is there for the second.
const SETTLE_PAUSE: std::time::Duration = std::time::Duration::from_millis(20);

/// The states a work directory holds a record in until the work it names is settled,
/// in the order an item moves through them when the service claims it
/// (`work_item_state_directories_and_claim_names.rs`,
/// `take_entry`).
/// Why this order:
///  a claim moves a record `pending` to `recovering` to `claimed` by two renames with a stamp between them,
/// so a look lists them in that order:
/// a record that left `pending` before it was listed is in `recovering` when that is listed,
/// and it cannot have reached `claimed` unseen,
/// because that listing comes last.
const OPEN_STATES: [&str; 3] = [PENDING, RECOVERING, CLAIMED];

/// The same three states in the order an expired claim returns a record to `pending`
/// (`landing/work_item_recovery.rs`,
/// `requeue_with`).
/// Why this order is read as well:
///  a record that only moves one way is missed by a look that lists the states the other way,
/// so a look reads both:
/// the order a claim carries a record through (`OPEN_STATES`) sees a claim,
/// and this order sees the return an expired claim makes,
/// whichever gap between two listings the renames of either fall in.
/// A record that is in one of the three folders when a look ends was in one of the listings of one of the two orders,
/// unless it changes state three times inside that single look,
/// which the two looks a pause apart then have to miss as well.
const RETURNING_STATES: [&str; 3] = [CLAIMED, RECOVERING, PENDING];

/// The orders one look lists the three states in,
/// the order a claim moves a record and the order an expired claim returns it.
const ORDERS_A_LOOK_READS: [[&str; 3]; 2] = [OPEN_STATES, RETURNING_STATES];

/// How many state listings one look takes,
/// for a control that moves a record at a gap between two of them.
pub const LISTINGS_IN_A_LOOK: usize = ORDERS_A_LOOK_READS.len() * OPEN_STATES.len();

/// What:
///  How many records one state directory of a work directory holds,
///  and why a directory that could not be listed is not a directory that holds none.
/// Why:
///   A record is in exactly one of the states a look reads until its work is settled,
/// and a claim moves it between two of them,
/// so one listing is one instant and three listings are three:
/// a listing the wait could not make is its answer to report and never a record's absence.
/// A wait that read one as "no record there" answered "settled" without evidence,
/// while the commit service kept the record and ran the tool after the control had already read its result
///  (`doc/troubleshooting/native-binary-settle-wait-reads-an-unread-state-as-settled.md`).
///  A state directory that is absent holds nothing (`recovering` is made by the first claim),
/// while any other refusal is reported with the directory named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recordsIn = (work, state) => { try { return readdir(join(work, state)).length } catch (error) { return error.code === 'ENOENT' ? 0 : raise(error) } };
/// ```
pub fn records_in(work: &Path, state: &str) -> Result<usize, String> {
    let directory: PathBuf = work.join(state);
    return match std::fs::read_dir(directory.as_path()) {
        Ok(listing) => Ok(listing.count()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(error) => Err(format!(
            "{} could not be listed ({error}), so whether this repository's after-commit work settled is not known",
            directory.display()
        )),
    };
}

/// What:
///  The records one look finds in a work directory,
///  a listing it could not make named in a failure.
/// Why:
///   A record is in one of the three states at every instant,
/// and either of the two motions the queue makes carries it from one to another between two listings,
/// so a look lists the states in both orders (`ORDERS_A_LOOK_READS`):
/// whichever way the record moves,
/// the order that lists the state it left before the state it reached sees it,
/// and an answer of "settled" cannot turn on how fast the machine carried it.
/// One pass in the order a claim moves a record already could not miss a claim,
/// which is what the failing sighting of 2026-11-14 was,
/// but an expired claim's return runs the other way and one pass can miss it
/// (review of 2026-11-16,
/// `one_recovery_missed_by_two_consecutive_empty_readings.rs`).
///  A queue nothing can list is reported with the work it would have held
///  (`work_listing`) instead of ending the wait as settled work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const betweenTheOrders = (work, recordsIn) => ORDERS_A_LOOK_READS.flat().reduce((total, state) => total + recordsIn(work, state), 0);
/// ```
fn records_between_the_orders(work: &Path, records_in: &dyn Fn(&Path, &str) -> Result<usize, String>) -> usize {
    let mut total: usize = 0;
    for order in ORDERS_A_LOOK_READS {
        for state in order {
            match records_in(work, state) {
                Ok(count) => total += count,
                Err(unread) => panic!("{unread}: {}", work_listing(work)),
            }
        }
    }
    return total;
}

/// What:
///  The queue folders of the repository around `directory`,
/// and nothing when no Git directory above it holds one.
/// Why:
///   A repository can hold both data-folder spellings at once,
/// while the folder is folded or after a process was killed during a move,
/// and the wrapper's own reader of those folders says that reading the first one alone hides the other
/// (`foundation/kept_names_in_the_git_directories.rs`,
/// `relative_paths_for_reading`).
/// A record open under the earlier name was never listed while this wait resolved one folder
/// (review of 2026-11-16,
/// `current_folder_selected_while_earlier_queue_is_open.rs`).
fn work_directories(directory: &Path) -> Option<Vec<PathBuf>> {
    let git_dir: PathBuf = directory.ancestors().map(|candidate| { return candidate.join(".git"); }).find(|candidate| { return candidate.is_dir(); })?;
    let relative: Vec<String> = git_policy_cli::kept_names_in_the_git_directories::relative_paths_for_reading(git_dir.as_path(), git_policy_cli::work_item::WORK_DIRECTORY);
    return Some(relative.iter().map(|path: &String| { return git_dir.join(path); }).collect());
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
///  This wait is `SERVICE_WORK_PATIENCE`,
///  which is shorter than one claim's lease (`CLAIM_LEASE`,
///  120 seconds):
/// a record whose handler answered with a retry keeps its claim until that lease ends,
/// so a control that waits here cannot see such a record settle.
/// A control that means to read one attempt of such a record reads that instead
/// (`observe_without_waiting_for_the_after_commit_work`),
/// and a control that means to read what one step of the work did waits for that step's own record
/// (`wait_for_pnpm_runs`),
/// because no reading of a queue says what a service has not reached yet.
///  Every queue the repository's Git directory can hold is read,
/// in both orders a record moves through its states (`work_directories`,
/// `records_between_the_orders`).
pub fn settle_service_work(directory: &Path) {
    return settle_service_work_list_with(directory, &records_in);
}

/// What:
///  `settle_service_work` with one state directory's listing given,
/// so a control moves a record between two of a look's listings
/// and proves what a look that found nothing is worth.
/// Why:
///   A listing is where the answer comes from,
/// and no control can make a claim straddle a listing on a real filesystem:
/// the two renames of `pending` to `recovering` to `claimed` sit microseconds apart.
/// `LISTINGS_IN_A_LOOK` says how many listings one look takes.
///  Work is settled only when two looks in a row find none,
/// `SETTLE_PAUSE` apart,
/// because a record an expired claim returns to `pending` can pass between two of the listings of one look,
/// and because the second look is taken at a different moment:
/// a record the service put back into `pending` after the first look is there for the second.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// settleServiceWorkListWith(dir, recordsIn); // a look that finds nothing twice, a pause apart, is settled work
/// ```
pub fn settle_service_work_list_with(directory: &Path, records_in: &dyn Fn(&Path, &str) -> Result<usize, String>) {
    let Some(work_paths) = work_directories(directory) else {
        return;
    };
    return settle_while_records_are_open(work_paths.as_slice(), &|work: &Path| { return records_between_the_orders(work, records_in); });
}

/// What:
///  The look loop of the wait:
///  one look at every queue of the repository,
///  and the wait over when two looks in a row found nothing.
/// Why:
///  The two faces above differ only in what one look reads,
/// so the pause,
/// the patience and the ending live once.
fn settle_while_records_are_open(work_paths: &[PathBuf], one_look: &dyn Fn(&Path) -> usize) {
    let began: std::time::Instant = std::time::Instant::now();
    let mut a_look_found_nothing: bool = false;
    loop {
        let open: bool = work_paths.iter().any(|work: &PathBuf| { return one_look(work.as_path()) > 0; });
        if open {
            a_look_found_nothing = false;
        } else if a_look_found_nothing {
            return;
        } else {
            a_look_found_nothing = true;
        }
        assert!(began.elapsed() < SERVICE_WORK_PATIENCE, "the service never settled the after-commit work in {}: {}", work_paths.iter().map(|work: &PathBuf| { return work.display().to_string(); }).collect::<Vec<String>>().join(", "), work_paths.iter().map(|work: &PathBuf| { return work_listing(work.as_path()); }).collect::<Vec<String>>().join(" "));
        std::thread::sleep(SETTLE_PAUSE);
    }
}
