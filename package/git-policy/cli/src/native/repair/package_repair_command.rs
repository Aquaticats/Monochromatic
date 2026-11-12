//! What:
//!  Starting one package-manager program for a worktree repair:
//! finding it on a search path,
//! running it in a directory with
//!  its output captured and a quiet window,
//! and saying how it ended.
//! Why:
//!  The repairs of a new worktree (`package_repair.rs`) start tools the repository uses (pnpm,
//! Cargo,
//! mpm-shadow).
//! Each one
//!  must end in a plain verdict the add can word for the person:
//! the program is missing,
//! it failed with a reason,
//! it showed
//!  no sign of work for its window,
//! or it worked.
//! A tool that works is never ended for being slow (owner,
//! 2026-11-03:
//!  "Install online during the add";
//! pick 9 of `doc/planning/cli-git-shipped-limits-audit.md`,
//! the coordinator's under the
//!  owner's delegation and open to the owner's veto),
//! with the signs of work the push reads (`push_progress.rs`).
//! Nothing here decides which tool runs or what its output means.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const program = findProgram('pnpm', process.env.PATH);
//! const run = await runTool(program, { arguments: ['install'], directory, set: [] }, quietWindow);
//! ```

/// The patience of a run that ends on a stall.
use crate::bounded_push::Patience;
/// A child owned until it is reaped,
/// and awaited on the runtime through its pidfd.
use crate::child_awaited_through_a_pidfd::{OwnedChild, adopt};
/// The process's runtime:
/// the synchronous face enters it,
/// and the reaper of a late leader runs on it.
use crate::lazy_runtime::{run_on_runtime, spawn_on_runtime};
/// The clock that reads the signs of work of a process tree.
use crate::push_progress::Clock;
/// The bounded start of the tool,
/// and the stop of the group it leads.
use crate::package_repair_start::{ToolStart, start_within, stop_group};
/// The longest quiet gap of each tool,
/// kept for the timing records.
use crate::output_capture::Capture;
use crate::quiet_ledger;
use crate::wait_ceilings::REPAIR_TOOL_NOTICE_AFTER;
use std::ffi::{OsStr, OsString};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// What:
///  One program start:
/// its arguments,
/// the directory it runs in,
/// and the variables added to its environment.
/// Why:
///   The program path is found separately,
/// so a test can name a fixture and a person's search path is read once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolCommand = { arguments: string[]; directory: string; set: [string, string][] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolCommand {
    /// The arguments after the program name.
    pub arguments: Vec<OsString>,
    /// The working directory.
    pub directory: PathBuf,
    /// Variables added to the inherited environment.
    pub set: Vec<(OsString, OsString)>,
}

/// What:
///  How long a started program may stay silent and how long it may run in all.
/// Why:
///   Two bounds travel together:
/// the quiet window ends a program that shows nothing,
/// and the ceiling ends one that keeps showing
///  something (owner,
/// 2026-11-04,
/// "Silence plus a ceiling (Recommended)").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolLimits = { window: number; ceiling: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToolLimits {
    /// How long the program may show no sign of work.
    pub window: Duration,
    /// The longest the program runs,
    /// whatever it shows.
    pub ceiling: Duration,
}

/// What:
///  How a started program ended.
/// Why:
///   The repair words each case differently,
/// so they stay apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolEnd = { kind: 'succeeded' } | { kind: 'failed'; reason: string } | { kind: 'timed-out' } | { kind: 'ceiling'; afterMs: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ToolEnd {
    /// Exit status zero.
    Succeeded,
    /// The program did not start,
    /// or ended with a failure status;
    /// the text names the status and the tail of its output.
    Failed(String),
    /// The program showed no sign of work for its window,
    /// and was stopped.
    TimedOut,
    /// The program ran as long as the ceiling allows,
    /// whatever it showed,
    /// and was stopped with its whole process group.
    CeilingReached(Duration),
}

/// What:
///  The end of one run and how long it took.
/// Why:
///   The time is reported to the person and measured by the docs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolRun = { end: ToolEnd; millis: number; stdout: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolRun {
    /// How the program ended.
    pub end: ToolEnd,
    /// Wall time from start to end,
    /// in milliseconds.
    pub millis: u128,
    /// What the program wrote to its standard output;
    /// empty when it did not run to its end.
    pub stdout: Vec<u8>,
    /// What was left out of the output or could not be read,
    /// in words for the person.
    /// Output with a note is not the whole output:
    ///  a parser of `stdout` must not take it for a complete result.
    pub notes: Vec<String>,
}

/// The most bytes of a program's output kept for its failure text.
///  Exempt from a citation under `LQM` (class:
/// display cap):
/// it limits how much text a message,
/// log field or record
///  names,
/// and no command result or duration depends on it.
/// Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
const OUTPUT_TAIL_BYTES: usize = 600;

/// How often a running program is looked at.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// What:
///  The first executable file named `name` in a search path,
///  named by its absolute path.
/// Why:
///   The program is the one the person's environment resolves;
/// a missing one is a verdict,
/// not a failure to start.
/// An entry without a leading slash names a directory of the directory this lookup happens in,
/// and every repair starts its tool in the worktree,
/// so the file is named where it was found rather than with a path the child would read from its own directory.
/// A directory of this process that cannot be read names no file to start,
/// which is the same answer as a program the search path does not hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findProgram(name: string, searchPath?: string): string | undefined;
/// ```
pub fn find_program(name: &str, search_path: Option<&OsStr>) -> Option<PathBuf> {
    use std::os::unix::fs::PermissionsExt;
    let search: &OsStr = search_path?;
    for directory in std::env::split_paths(search) {
        // An empty entry means the current directory in a shell; a repair never runs a program found there.
        if directory.as_os_str().is_empty() {
            continue;
        }
        let candidate: PathBuf = directory.join(name);
        let runnable: bool = match std::fs::metadata(&candidate) {
            Ok(found) => found.is_file() && found.permissions().mode() & 0o111 != 0,
            Err(_) => false,
        };
        if runnable {
            // A name that is already absolute is kept as the person wrote it; only a relative one needs this directory.
            return std::path::absolute(candidate).ok();
        }
    }
    return None;
}

/// What:
///  The last bytes of what a program printed,
/// as text,
/// with the cut ragged edge dropped.
/// Why:
///   A failure is worded with the end of the output,
/// where tools print their reason;
/// the start is progress noise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tailOf(bytes: Uint8Array): string;
/// ```
fn tail_of(bytes: &[u8]) -> String {
    let cut: usize = bytes.len().saturating_sub(OUTPUT_TAIL_BYTES);
    let text: String = String::from_utf8_lossy(&bytes[cut..]).into_owned();
    // Lines that only say a tool waited for another process say nothing about why it failed.
    let mut informative_lines: Vec<&str> = Vec::new();
    for line in text.lines() {
        if !line.trim_start().starts_with("Blocking waiting for file lock") {
            informative_lines.push(line);
        }
    }
    let informative: String = informative_lines.join("\n");
    let trimmed: &str = informative.trim();
    if cut == 0 {
        return trimmed.to_owned();
    }
    let whole_lines: &str = match trimmed.split_once('\n') {
        Some((_ragged, rest)) => rest,
        None => trimmed,
    };
    return whole_lines.trim().to_owned();
}

/// What:
///  How a finished program's status is worded.
/// Why:
///   A signal has no code,
/// and the person should see which of the two ended the program.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function describeStatus(status: ExitStatus): string;
/// ```
fn describe_status(status: std::process::ExitStatus) -> String {
    use std::os::unix::process::ExitStatusExt;
    return match (status.code(), status.signal()) {
        (Some(code), _) => format!("exit status {code}"),
        (None, Some(signal)) => format!("stopped by signal {signal}"),
        (None, None) => String::from("ended without a status"),
    };
}

/// What:
///  Run a program to its end,
/// until it has shown no sign of work for the window,
/// or until it has run as long as the ceiling.
/// Why:
///   The add must not wait for a tool that does nothing,
/// and must keep the tail of its output for the log.
/// A tool that works is
///  waited for until the ceiling (owner,
/// 2026-11-04,
/// "Silence plus a ceiling (Recommended)";
/// `wait_ceilings.rs`,
///  `REPAIR_TOOL_CEILING`),
/// where it is ended like a silent one.
/// The program leads a process group of its own,
/// so the stop reaches the children it
///  started too.
/// Where the system counts nothing (no `/proc/<pid>/io`) the quiet window never ends the tool,
/// and only the
///  ceiling does.
/// The longest quiet gap of the run is kept in the ledger under the program's name and first argument.
/// The output
///  goes to anonymous files,
/// so the ceiling covers the draining of it too:
/// nothing waits for a stream to close,
/// a helper that holds
///  an inherited stream after the tool has exited holds nothing up,
/// and the stop (the group,
/// while the leader is not yet reaped,
///  so the group number is still the leader's) never has to wait for a helper that left the group.
/// A helper still running when the
///  leader has exited successfully is left alone,
/// because a tool may start a helper on purpose;
/// at the ceiling or the window the
///  whole group is stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runTool(program: string, command: ToolCommand, limits: { windowMs: number; ceilingMs: number }): Promise<ToolRun>;
/// ```
pub fn run_tool(program: &Path, command: &ToolCommand, limits: ToolLimits) -> ToolRun {
    let started: Instant = Instant::now();
    return match run_on_runtime(run_tool_reaping(program, command, limits, &reap_within_stop_grace)) {
        Ok(run) => run,
        Err(refusal) => run_refused(program, started, &refusal),
    };
}

/// What:
///  The run a tool never made,
/// because the runtime this process's waits need could not be entered.
/// Why:
///  A repair never fails the add:
/// a tool that could not be run is a failure finding that names the command to run,
/// which is what
///  `run_tool` answers with the same words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const run: ToolRun = { end: { kind: 'failed', reason: `${program} could not be run: ${refusal}` }, millis, stdout: [], notes: [] };
/// ```
pub fn run_refused(program: &Path, started: Instant, refusal: &crate::lazy_runtime::RuntimeError) -> ToolRun {
    return ToolRun { end: ToolEnd::Failed(format!("{} could not be run: {refusal}", program.display())), millis: started.elapsed().as_millis(), stdout: Vec::new(), notes: Vec::new() };
}

/// What:
///  Run a program to its end with every wait of it awaited on the runtime the caller runs on.
/// Why:
///  `run_tool` enters the runtime,
/// which a caller that is itself a task cannot do
/// (`RuntimeError::Nested`),
/// and the
///  repairs of a new worktree await their tools on the runtime rather than holding a thread for them.
/// The waits,
///  their bounds and the words of every end are `run_tool`'s,
/// because both faces share `run_tool_reaping` and the reaper of a
///  stopped leader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const run = await runToolAwaiting(program, command, limits);
/// ```
pub async fn run_tool_awaiting(program: &Path, command: &ToolCommand, limits: ToolLimits) -> ToolRun {
    return run_tool_reaping(program, command, limits, &reap_within_stop_grace).await;
}

/// What:
///  Reap a stopped leader within the push stop grace;
/// whether it was reaped.
/// Why:
///   `run_tool` stops a leader the usual way,
/// and a named function stands where a closure that ignored the captured
///  output stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reapWithinStopGrace = (leader: Child, _captured: Capture) => reapWithin(leader, STOP_GRACE);
/// ```
async fn reap_within_stop_grace(leader: Child, _captured: &Capture) -> bool {
    return reap_within(leader, crate::bounded_push::STOP_GRACE).await;
}

/// The run of a tool that never started because the files for its output could not be made.
fn no_place_for_output(program: &Path, started: Instant, error: &std::io::Error) -> ToolRun {
    return ToolRun { end: ToolEnd::Failed(format!("{} could not be started: no place for its output ({error})", program.display())), millis: started.elapsed().as_millis(), stdout: Vec::new(), notes: Vec::new() };
}

/// `run_tool` as a task of the runtime,
/// with the reaper of a stopped leader given:
/// a control sees that the stop path reaps through it.
///  The wait for the tool is awaited on its pidfd,
/// and the looks at the clock and the ceiling are made at `POLL_INTERVAL`.
///  The start itself is bounded by `REPAIR_TOOL_START_CEILING`;
/// `run_tool_reaping_started_within` takes that bound from its caller.
pub async fn run_tool_reaping<Reap: AsyncFn(Child, &Capture) -> bool>(program: &Path, command: &ToolCommand, limits: ToolLimits, reap: &Reap) -> ToolRun {
    return run_tool_reaping_started_within(program, command, limits, crate::wait_ceilings::REPAIR_TOOL_START_CEILING, reap).await;
}

/// What:
///  `run_tool_reaping` with the bound on the start given,
/// which a control reaches in milliseconds.
/// Why:
///   The bound is one a person chooses through no setting and a control cannot wait for,
/// so the run that ships gives it a
///  named constant and a control names its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runToolReapingStartedWithin(program, command, limits, startMs, reap): Promise<ToolRun>;
/// ```
pub async fn run_tool_reaping_started_within<Reap: AsyncFn(Child, &Capture) -> bool>(program: &Path, command: &ToolCommand, limits: ToolLimits, start: Duration, reap: &Reap) -> ToolRun {
    use std::os::unix::process::CommandExt;
    let started: Instant = Instant::now();
    let mut starting: Command = Command::new(program);
    // The output goes to anonymous files and not to pipes: a helper that keeps an inherited stream open after the tool has exited, or
    // that left the process group while holding it, then holds nothing up, because no thread waits for a stream to close.
    let captured: Capture = match Capture::new() {
        Ok(files) => match files.stdio() {
            Ok((out, err)) => {
                starting.stdout(out).stderr(err);
                files
            }
            Err(error) => return no_place_for_output(program, started, &error),
        },
        Err(error) => return no_place_for_output(program, started, &error),
    };
    starting.args(&command.arguments).current_dir(&command.directory).stdin(Stdio::null()).process_group(0);
    // A control holds the child before its own exec, which is the only way a test can make a start take time.
    #[cfg(test)]
    crate::package_repair_start::hold_the_start_before_its_exec(&mut starting);
    for (name, value) in &command.set {
        starting.env(name, value);
    }
    let mut child: OwnedChild = match start_within(start, starting).await {
        ToolStart::Running(running) => adopt(running),
        ToolStart::Refused(error) => return ToolRun { end: ToolEnd::Failed(format!("{} could not start: {error}", program.display())), millis: started.elapsed().as_millis(), stdout: Vec::new(), notes: Vec::new() },
        ToolStart::NoAnswer => {
            // Nothing reads what the tool writes: the run says it did not start here, and the child it finally starts is stopped.
            captured.discard();
            return ToolRun { end: ToolEnd::Failed(format!("{} could not start: it gave no answer within {start:?}, so nothing of it ran", program.display())), millis: started.elapsed().as_millis(), stdout: Vec::new(), notes: Vec::new() };
        }
    };
    let mut clock: Clock = Clock::start(child.id(), Patience::Progress { stall: limits.window }).without_wall_fallback();
    let mut at_ceiling: bool = false;
    let mut wait_failure: Option<std::io::Error> = None;
    // Whether the person has been told what this tool waits for, which is said once per run and never for a tool that ends quickly.
    let mut noticed: bool = false;
    let status: Option<std::process::ExitStatus> = loop {
        match wait_status(&mut child).await {
            Ok(Some(done)) => break Some(done),
            // The ceiling ends the program whatever it shows (owner, 2026-11-04, "Silence plus a ceiling (Recommended)").
            Ok(None) if started.elapsed() >= limits.ceiling => {
                at_ceiling = true;
                break None;
            }
            Ok(None) if clock.check().is_some() => break None,
            Ok(None) => {}
            // A wait that fails is a stop like the others: the group is stopped, the pages are released and the leader is reaped below.
            Err(error) => {
                wait_failure = Some(error);
                break None;
            }
        }
        // What: A tool that is still running after the notice bound says what it is,
        //       where it runs and what ends it.
        // Why:  An add whose `pnpm` never ended printed its last line at two seconds and said nothing for the next 613 s, though the
        //       quiet window and the ceiling above it were both running (`wait_ceilings.rs`, `REPAIR_TOOL_NOTICE_AFTER`).
        if !noticed && started.elapsed() >= REPAIR_TOOL_NOTICE_AFTER {
            noticed = true;
            say_what_it_waits_for(program, command, limits);
        }
    };
    // A clock that reads nothing (no `/proc`) has no quiet gap to keep: its figure would be the elapsed time.
    if clock.measures() {
        quiet_ledger::note(tool_step(program, command).as_str(), clock.longest_quiet(), status.is_none() && !at_ceiling && wait_failure.is_none());
    }
    let Some(finished) = status else {
        stop_group(child.id());
        // The leader is reaped so it leaves no zombie, within a budget: a process in uninterruptible I/O ends only when the I/O does,
        // and a wait with no bound would keep the add for as long as it lasts. Nothing waits for the output, which is in files.
        // What the tool wrote is released first: nobody reads it, and a descendant that keeps the files open would keep the pages.
        captured.discard();
        // A leader that the wait has collected (it ended while the wait failed) has nothing left to reap.
        if let Some(leader) = child.into_std_when_unreaped() {
            let _reaped_in_time: bool = reap(leader, &captured).await;
        }
        let end: ToolEnd = match wait_failure {
            Some(error) => ToolEnd::Failed(format!("{} could not be waited for: {error}", program.display())),
            None if at_ceiling => ToolEnd::CeilingReached(limits.ceiling),
            None => ToolEnd::TimedOut,
        };
        return ToolRun { end, millis: started.elapsed().as_millis(), stdout: Vec::new(), notes: Vec::new() };
    };
    let shown: crate::output_capture::Captured = captured.snapshot(crate::output_capture::STEP_OUTPUT_LIMIT);
    // Nothing reads the files again, and a helper that still holds them would keep what they hold in memory.
    captured.discard();
    let (stdout, errors): (Vec<u8>, Vec<u8>) = (shown.stdout, shown.stderr);
    let millis: u128 = started.elapsed().as_millis();
    if finished.success() {
        return ToolRun { end: ToolEnd::Succeeded, millis, stdout, notes: shown.notes };
    }
    let combined: Vec<u8> = [stdout.as_slice(), errors.as_slice()].concat();
    let text: String = tail_of(combined.as_slice());
    // What was left out, or could not be read, is part of what the person is told: the text shows the tail only.
    let disclosed: String = if shown.notes.is_empty() { String::new() } else { format!(" ({})", shown.notes.join("; ")) };
    return ToolRun { end: ToolEnd::Failed(format!("{}: {text}{disclosed}", describe_status(finished))), millis, stdout: Vec::new(), notes: shown.notes };
}

/// What:
///  The line that says a repair tool is still running,
/// what it runs,
/// where,
/// and what ends it.
/// Why:
///   The two bounds that end a tool (the quiet window and the ceiling) are the only thing that ends it,
/// and a person who
///  sees nothing for ten minutes cannot tell a tool that works from one the wrapper lost;
/// the tool,
/// its
///  arguments and the worktree it runs in are what tells them apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sayWhatItWaitsFor(pnpm, { arguments: ['install', '--offline'], directory: worktree }, { window: 600, ceiling: 1080 })
/// ```
fn say_what_it_waits_for(program: &Path, command: &ToolCommand, limits: ToolLimits) {
    let words: String = what_it_waits_for(program, command, limits);
    #[cfg(test)]
    NOTICES_SAID.lock().expect("notices").push(words.clone());
    // Terminal output: this line is the person's, not a log record, and there is nowhere to report a closed standard error.
    let _written = writeln!(std::io::stderr(), "{words}");
}

/// What:
///  The words of that line,
/// which a control reads without touching the process's standard error.
/// Why:
///   A repair's waits are announced on standard error,
/// which a control of a test binary that runs its tests side by side cannot
///  take for itself;
/// the words are built here and printed by the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = whatItWaitsFor(pnpm, command, limits);
/// ```
fn what_it_waits_for(program: &Path, command: &ToolCommand, limits: ToolLimits) -> String {
    let arguments: Vec<String> = command.arguments.iter().map(argument_text).collect();
    return format!(
        "monochromatic-git-policy-cli: {} {} is still running in {} for the repair of the new worktree; it is stopped when it shows no sign of work for {} seconds, or after {} minutes at the latest.",
        program.display(),
        arguments.join(" "),
        command.directory.display(),
        limits.window.as_secs(),
        limits.ceiling.as_secs() / 60
    );
}

/// What:
///  One argument of a tool's command line,
/// as the text a person reads.
/// Why:
///  The parameters of a repair are what tells two runs of the same tool apart
/// (`--offline`,
/// `--frozen-lockfile`),
/// and the notice is the only place a person sees them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// argumentText(OsString('--offline')) // '--offline'
/// ```
fn argument_text(word: &OsString) -> String {
    return word.to_string_lossy().into_owned();
}

/// What:
///  What the waiting notices said,
/// so a control reads them without taking the process's standard error.
/// Why:
///   A tool's wait runs as a task of the process's runtime,
/// on a worker of its own,
/// so a record of one thread would miss it;
/// a control of a test process that runs its tests side by side reads its own line by the
///  fixture path in it.
#[cfg(test)]
pub(crate) fn notices_said() -> Vec<String> {
    return NOTICES_SAID.lock().expect("notices").clone();
}

/// The notices the controls read.
#[cfg(test)]
static NOTICES_SAID: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// The child's end if it comes within `POLL_INTERVAL`,
/// awaited on its pidfd,
/// and in a control an error once the file the seam
///  names exists.
async fn wait_status(child: &mut OwnedChild) -> std::io::Result<Option<std::process::ExitStatus>> {
    #[cfg(test)]
    if WAIT_FAILS_ONCE_THIS_FILE_EXISTS.with(|seam| { return seam.borrow().as_ref().is_some_and(|path| { return path.exists(); }); }) {
        return Err(std::io::Error::other("the wait failed"));
    }
    return match tokio::time::timeout(POLL_INTERVAL, child.wait()).await {
        Ok(ended) => ended.map(Some),
        Err(_not_yet) => Ok(None),
    };
}

#[cfg(test)]
thread_local! {
    /// A control makes the wait for the tool fail once the file it names exists,
    /// which the tool makes after it has written.
    pub(crate) static WAIT_FAILS_ONCE_THIS_FILE_EXISTS: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

/// What:
///  Reap `child` within `budget`;
/// when it has not ended by then,
/// hand it to a task that reaps it whenever it does.
/// Why:
///   The stop is a signal,
/// and a process whose kernel call cannot be interrupted ends only when the call does.
/// The add returns
///  without it,
/// and the task keeps the leader from becoming a zombie for good.
/// True when the leader ended within the budget.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ended = await reapWithin(child, budgetMs); // false: a background task reaps it later
/// ```
async fn reap_within(child: Child, budget: Duration) -> bool {
    let mut owned: OwnedChild = adopt(child);
    if let Ok(Ok(_status)) = tokio::time::timeout(budget, owned.wait()).await {
        return true;
    }
    // A runtime with no worker leaves the leader to the system's init when this process ends; the add goes on either way.
    if let Some(late) = owned.into_std_when_unreaped() {
        let _handed = spawn_on_runtime(async move {
            let mut reaper: OwnedChild = adopt(late);
            let _reaped = reaper.wait().await;
        });
    }
    return false;
}

/// What:
///  The name a tool's quiet gap is kept under:
/// the program's file name and its first argument,
/// as in `pnpm install`.
/// Why:
///   Two uses of one program (`cargo metadata`,
/// `cargo clean`) have different gaps,
/// and the arguments after the first are
///  paths that would make a name for every worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [basename(program), args[0]].filter(Boolean).join(' ')
/// ```
fn tool_step(program: &Path, command: &ToolCommand) -> String {
    let name: String = match program.file_name() {
        Some(file) => file.to_string_lossy().into_owned(),
        None => String::new(),
    };
    return match command.arguments.first() {
        Some(first) => format!("{name} {}", first.to_string_lossy()),
        None => name,
    };
}

/// Command controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_command_tests.rs"]
mod tests;
