//! What:
//!  Running a wrapped command with a time bound and reading what it did:
//! its exit status,
//! its output without the wait notices and hints,
//! and the settling of the service work it started.
//! Why:
//!  The reading of a run needs no fixture beyond its directory,
//! so it lives in a file of its own that the helpers that start commands import.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const observed = observeRaw(command); // { code, stdout, stderr, signal }
//! ```

use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use crate::support_program_names_and_time_bound::TIME_BOUND_SECONDS;
use git_policy_cli::work_item::{CLAIMED, PENDING, RECOVERING};

/// The signal that ends `timeout --signal=KILL` itself when the bound was hit:
///  it kills
/// the command,
///  then re-raises the same signal on itself.
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

/// Run a command to completion and observe it without its hint fields,
///  feeding `input` to its standard input.
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
/// and `wait_with_output` drains both pipes from then on:
/// a command that writes much is not idle.
///  The kill is claimed before the cleanup because the cleanup can take longer than the grace in which an unclaimed kill
///  aborts the process.
///
/// # Panics
///  With the supervisor's message when it ended the command.
pub fn supervised_wait(mut child: Child, input: &[u8], cleanup: impl FnOnce()) -> Output {
    let child_id: u32 = child.id();
    let mut stdin = child.stdin.take().expect("piped stdin");
    let owned: Vec<u8> = input.to_vec();
    // A child that exits without reading its input closes the pipe;
    // that is not a test failure.
    let writer = std::thread::spawn(move || { let _ = stdin.write_all(owned.as_slice()); });
    let output: Output = child.wait_with_output().expect("process finishes");
    let _joined = writer.join();
    // The supervisor ended it:
    // a loop or a deadlock,
    // with the reason and the tasks of the tree in the message.
    if let Some(message) = super::supervisor::take_kill(child_id) {
        panic!("{message}");
    }
    cleanup();
    return output;
}

/// Run a command to completion with every byte of its output kept,
///  feeding `input` to its standard input.
/// # Panics
///  When the command is a commit transaction or a manual page under the shared bound;
///  see `shared_bound_misuse`.
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
///  one run of a command, and the wait for the after-commit work given or refused.
/// Why:
///  The two faces differ only in that wait,
/// so the bound,
/// the misuse check and the reading live once.
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
    let child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("process starts");
    let output: Output = supervised_wait(child, input, || {
        if !waiting_for_the_after_commit_work {
            return;
        }
        if let Some(directory) = command.get_current_dir() {
            settle_service_work(directory);
        }
    });
    assert_ne!(
        output.status.signal(),
        Some(TIMED_OUT_SIGNAL),
        // Built only on failure: a bound hit under host load must name the command that stalled.
        "the command hit its wall-clock bound: {:?} in {:?}",
        command.get_args().collect::<Vec<&OsStr>>(),
        command.get_current_dir()
    );
    return Observed {
        code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    };
}

/// How long a control waits for the after-commit work (the push,
/// the upkeep) the commit service runs once the commit returned.
pub const SERVICE_WORK_PATIENCE: std::time::Duration = std::time::Duration::from_secs(60);

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

/// How long a wait for the after-commit work pauses before it reads the queue again:
///  the pause between its two readings,
/// and the pause between one look and the next while work is still queued.
/// Why this value:
///  it is the pause this wait already took between its looks,
/// so the two readings are a poll apart,
/// and a record the service puts back into `pending` (or a record that arrives)
/// after the first reading is there for the second.
const SETTLE_PAUSE: std::time::Duration = std::time::Duration::from_millis(20);

/// The states a work directory holds a record in until the work it names is settled,
/// in the order an item moves through them when the service claims it
/// (`work_item_state_directories_and_claim_names.rs`, `take_entry`).
/// Why this order:
///  a claim moves a record `pending` to `recovering` to `claimed` by two renames with a stamp between them,
/// so the listings are taken in that order:
/// a record that left `pending` before it was listed is in `recovering` when that is listed,
/// and it cannot have reached `claimed` unseen,
/// because that listing comes last.
/// A record that leaves `claimed` for `done` or `failed` has settled,
/// so a pass over these three states does not miss work the service holds for a claim.
const OPEN_STATES: [&str; 3] = [PENDING, RECOVERING, CLAIMED];

/// What:
///  The number of records a work directory holds in the states a settling wait means,
///  and the state directory a listing could not read.
/// Why:
///   A record is in exactly one of those states until its work is settled,
/// and a claim moves it between two of them between two listings,
/// so one listing is one instant and three listings are three:
/// the states are read in the order an item moves through them (`OPEN_STATES`),
/// so a record the wait is asked about while a claim carries it is seen in one pass,
/// whichever gap between two listings the claim's two renames fall in.
/// A settled answer can therefore not turn on how fast the machine carried a claim,
/// which is what the failing sighting did:
/// one control read the queue as settled and the service ran the relink it held three milliseconds later
/// (`package_repair::quiet_runs_the_repair_and_says_nothing_about_success`, 2026-11-14).
/// A record an expired claim is returning to `pending` runs the other way (`requeue_with`),
/// and a pass misses it only where one of its two renames falls between two of the listings,
///  which the second reading of `settle_service_work_read_with` makes a coincidence twice over.
///  A listing the wait could not make is its answer to report and never a record's absence:
/// a wait that read one as "no record there" answered "settled" without evidence,
/// while the commit service kept the record and ran the tool after the control had already read its result
///  (`doc/troubleshooting/native-binary-settle-wait-reads-an-unread-state-as-settled.md`).
///  A state directory that is absent holds nothing (`recovering` is made by the first claim),
/// while any other refusal is reported with the directory named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const openRecords = (workDir) => OPEN_STATES.reduce((total, state) => total + countOrThrow(join(workDir, state)), 0);
/// ```
fn open_records(work: &Path) -> Result<usize, String> {
    let mut total: usize = 0;
    for state in OPEN_STATES {
        let directory: PathBuf = work.join(state);
        match std::fs::read_dir(directory.as_path()) {
            Ok(listing) => total += listing.count(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "{} could not be listed ({error}), so whether this repository's after-commit work settled is not known",
                    directory.display()
                ));
            }
        }
    }
    return Ok(total);
}

/// What:
///  How many records of a work directory mean work that has not settled,
/// in the states the wait lists and by the wrapper's own reading of them,
/// and a listing it could not make named in a failure.
/// Why:
///   This is the reading `settle_service_work` asks for every look,
/// so a queue nothing can list is reported with the work it would have held
///  (`work_listing`) instead of ending the wait as settled work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outstanding = (work) => openRecords(work) ?? panic(`${unread}: ${listing(work)}`);
/// ```
fn work_outstanding(work: &Path) -> usize {
    return match open_records(work) {
        Ok(total) => total,
        Err(unread) => panic!("{unread}: {}", work_listing(work)),
    };
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
///  This wait is `SERVICE_WORK_PATIENCE`, which is shorter than one claim's lease (`CLAIM_LEASE`, 120 seconds):
/// a record whose handler answered with a retry keeps its claim until that lease ends,
/// so a control that waits here cannot see such a record settle.
/// A control that means to read one attempt of such a record reads that instead
/// (`observe_without_waiting_for_the_after_commit_work`),
/// and a control that means to read what one step of the work did waits for that step's own record
/// (`wait_for_pnpm_runs`),
/// because no reading of a queue says what a service has not reached yet.
///  The queue is read under the data folder the wrapper's own reading of a Git directory names
/// (`path_for_reading`),
/// because a repository whose folder keeps the earlier release's name holds its queue there.
pub fn settle_service_work(directory: &Path) {
    return settle_service_work_read_with(directory, &work_outstanding);
}

/// What:
///  `settle_service_work` with the reading of the queue given,
/// so a control proves what one empty reading is worth.
/// Why:
///   The reading is where the answer comes from,
/// and no control can make a claim straddle a listing on a real filesystem:
/// the two renames of `pending` to `recovering` to `claimed` sit microseconds apart.
/// Work is settled only when two readings in a row find none,
/// `SETTLE_PAUSE` apart,
/// because a record an expired claim returns to `pending` can pass between two of a pass's listings,
/// and because the second reading is taken at a different moment:
/// a record the service put back into `pending` after the first reading is there for the second.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// settleServiceWorkReadWith(dir, (work) => outstanding(work)); // empty twice, a pause apart, is settled
/// ```
pub fn settle_service_work_read_with(directory: &Path, outstanding: &dyn Fn(&Path) -> usize) {
    let Some(git_dir) = directory.ancestors().map(|candidate| { return candidate.join(".git"); }).find(|candidate| { return candidate.is_dir(); }) else {
        return;
    };
    let work: PathBuf = git_policy_cli::kept_names_in_the_git_directories::path_for_reading(git_dir.as_path(), git_policy_cli::work_item::WORK_DIRECTORY);
    let began: std::time::Instant = std::time::Instant::now();
    let mut a_reading_found_nothing: bool = false;
    loop {
        if outstanding(work.as_path()) > 0 {
            a_reading_found_nothing = false;
        } else if a_reading_found_nothing {
            return;
        } else {
            a_reading_found_nothing = true;
        }
        assert!(began.elapsed() < SERVICE_WORK_PATIENCE, "the service never settled the after-commit work in {}: {}", work.display(), work_listing(work.as_path()));
        std::thread::sleep(SETTLE_PAUSE);
    }
}
