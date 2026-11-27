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
//!
//! Files in this module,
//! each using only the files listed before it:
//!
//! - `package_repair_command_answers.rs`:
//!   the command one run is asked for,
//!   the two bounds it runs under,
//!   how it ended,
//!   and what it left.
//! - `package_repair_command_finding_the_program.rs`:
//!   the first executable file the person's search path holds under a tool's name.
//! - `package_repair_command_words.rs`:
//!   the words a run is reported in,
//!   and the quiet gap's name.
//! - `package_repair_command_the_childs_end.rs`:
//!   the wait that reads a child's end,
//!   and the reaping of a leader a stop left behind.

/// Import the patience of a run that ends on a stall.
use crate::bounded_push::Patience;
/// Import a child owned until it is reaped,
/// and awaiting it on the runtime through its pidfd.
use crate::child_awaited_through_a_pidfd::{
    OwnedChild,
    adopt,
};
/// Import the process's runtime,
/// which the synchronous face enters.
use crate::lazy_runtime::run_on_runtime;
/// Import the anonymous files a run's output goes to.
use crate::output_capture::Capture;
/// Import the command one run is asked for,
/// the two bounds it runs under,
/// how it ended,
/// and what it left.
pub use crate::package_repair_command_answers::{
    ToolCommand,
    ToolEnd,
    ToolLimits,
    ToolRun,
};
/// Import the lookup of the program a repair starts,
/// which every repair names here.
pub use crate::package_repair_command_finding_the_program::find_program;
/// Import the wait that reads whether a child came within the poll interval,
/// and the reaping of a stopped leader within the push stop grace.
use crate::package_repair_command_the_childs_end::{
    reap_within_stop_grace,
    wait_status,
};
/// Import the words a run is reported in,
/// and the quiet gap's name.
use crate::package_repair_command_words::{
    describe_status,
    failure_text,
    say_what_it_waits_for,
    tool_step,
};
/// Import the bounded start of the tool,
/// and the stop of the group it leads.
use crate::package_repair_start::{
    ToolStart,
    start_within,
    stop_group,
};
/// Import the clock that reads the signs of work of a process tree.
use crate::push_progress::Clock;
/// Import the ledger one tool's longest quiet gap is kept in.
use crate::quiet_ledger;
/// Import the bound after which a tool is announced.
use crate::wait_ceilings::REPAIR_TOOL_NOTICE_AFTER;
/// Import the borrowed filesystem path a run names.
use std::path::Path;
/// Import the child process handle and the standard-library command it is started from.
use std::process::{
    Child,
    Command,
    Stdio,
};
/// Import the spans of time a bound and a measurement are written as.
use std::time::{
    Duration,
    Instant,
};

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
pub fn run_tool(
    program: &Path,
    command: &ToolCommand,
    limits: ToolLimits,
) -> ToolRun {
    let started: Instant = Instant::now();
    return match run_on_runtime(run_tool_reaping(
        program,
        command,
        limits,
        &reap_within_stop_grace,
    )) {
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
pub fn run_refused(
    program: &Path,
    started: Instant,
    refusal: &crate::lazy_runtime::RuntimeError,
) -> ToolRun {
    return ToolRun {
        end: ToolEnd::Failed(format!("{} could not be run: {refusal}", program.display())),
        millis: started.elapsed().as_millis(),
        stdout: Vec::new(),
        notes: Vec::new(),
    };
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
pub async fn run_tool_awaiting(
    program: &Path,
    command: &ToolCommand,
    limits: ToolLimits,
) -> ToolRun {
    return run_tool_reaping(program, command, limits, &reap_within_stop_grace).await;
}

/// The run of a tool that never started because the files for its output could not be made.
fn no_place_for_output(
    program: &Path,
    started: Instant,
    error: &std::io::Error,
) -> ToolRun {
    return ToolRun {
        end: ToolEnd::Failed(format!(
            "{} could not be started: no place for its output ({error})",
            program.display()
        )),
        millis: started.elapsed().as_millis(),
        stdout: Vec::new(),
        notes: Vec::new(),
    };
}

/// `run_tool` as a task of the runtime,
/// with the reaper of a stopped leader given:
/// a control sees that the stop path reaps through it.
///  The wait for the tool is awaited on its pidfd,
/// and the looks at the clock and the ceiling are made at `POLL_INTERVAL`.
///  The start itself is bounded by `REPAIR_TOOL_START_CEILING`;
/// `run_tool_reaping_started_within` takes that bound from its caller.
///  The tool is held before its own `exec` until the run lets it go (`package_repair_start.rs`),
/// so a start the bound ends has run nothing of the tool,
/// and the one answer that cannot promise that says so in its own words.
pub async fn run_tool_reaping<Reap: AsyncFn(Child, &Capture) -> bool>(
    program: &Path,
    command: &ToolCommand,
    limits: ToolLimits,
    reap: &Reap,
) -> ToolRun {
    return run_tool_reaping_started_within(
        program,
        command,
        limits,
        crate::wait_ceilings::REPAIR_TOOL_START_CEILING,
        reap,
    )
    .await;
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
pub async fn run_tool_reaping_started_within<Reap: AsyncFn(Child, &Capture) -> bool>(
    program: &Path,
    command: &ToolCommand,
    limits: ToolLimits,
    start: Duration,
    reap: &Reap,
) -> ToolRun {
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
    starting
        .args(&command.arguments)
        .current_dir(&command.directory)
        .stdin(Stdio::null())
        .process_group(0);
    for (name, value) in &command.set {
        starting.env(name, value);
    }
    let mut child: OwnedChild = match start_within(start, starting).await {
        ToolStart::Running(not_yet_taken) => adopt(not_yet_taken.taken()),
        ToolStart::Refused(error) => {
            return ToolRun {
                end: ToolEnd::Failed(format!("{} could not start: {error}", program.display())),
                millis: started.elapsed().as_millis(),
                stdout: Vec::new(),
                notes: Vec::new(),
            };
        }
        ToolStart::NoAnswer => {
            // Nothing reads what the tool writes: the run says it did not start here, and the gate kept the child from
            // ever running, so the worktree keeps exactly the files Git wrote.
            captured.discard();
            return ToolRun {
                end: ToolEnd::Failed(format!(
                    "{} could not start: it gave no answer within {start:?}, so nothing of it ran",
                    program.display()
                )),
                millis: started.elapsed().as_millis(),
                stdout: Vec::new(),
                notes: Vec::new(),
            };
        }
        ToolStart::LetGoWithoutAnAnswer => {
            // The child was let go and never came up, so it was stopped: nothing reads what it may have written before
            // that, and the finding says so instead of promising that nothing ran.
            captured.discard();
            return ToolRun {
                end: ToolEnd::Failed(format!(
                    "{} could not start: it was let go and its own start gave no answer within {start:?}, so it was stopped, and what it wrote before that stands",
                    program.display()
                )),
                millis: started.elapsed().as_millis(),
                stdout: Vec::new(),
                notes: Vec::new(),
            };
        }
    };
    let mut clock: Clock = Clock::start(
        child.id(),
        Patience::Progress {
            stall: limits.window,
        },
    )
    .without_wall_fallback();
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
        quiet_ledger::note(
            tool_step(program, command).as_str(),
            clock.longest_quiet(),
            status.is_none() && !at_ceiling && wait_failure.is_none(),
        );
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
            Some(error) => ToolEnd::Failed(format!(
                "{} could not be waited for: {error}",
                program.display()
            )),
            None if at_ceiling => ToolEnd::CeilingReached(limits.ceiling),
            None => ToolEnd::TimedOut,
        };
        return ToolRun {
            end,
            millis: started.elapsed().as_millis(),
            stdout: Vec::new(),
            notes: Vec::new(),
        };
    };
    let shown: crate::output_capture::Captured =
        captured.snapshot(crate::output_capture::STEP_OUTPUT_LIMIT);
    // Nothing reads the files again, and a helper that still holds them would keep what they hold in memory.
    captured.discard();
    let (stdout, errors): (Vec<u8>, Vec<u8>) = (shown.stdout, shown.stderr);
    let millis: u128 = started.elapsed().as_millis();
    if finished.success() {
        return ToolRun {
            end: ToolEnd::Succeeded,
            millis,
            stdout,
            notes: shown.notes,
        };
    }
    let combined: Vec<u8> = [stdout.as_slice(), errors.as_slice()].concat();
    let text: String = failure_text(combined.as_slice());
    // What was left out, or could not be read, is part of what the person is told: the text shows the tail only.
    let disclosed: String = if shown.notes.is_empty() {
        String::new()
    } else {
        format!(" ({})", shown.notes.join("; "))
    };
    return ToolRun {
        end: ToolEnd::Failed(format!("{}: {text}{disclosed}", describe_status(finished))),
        millis,
        stdout: Vec::new(),
        notes: shown.notes,
    };
}

// The owned string and path a control builds its fixture from.
#[cfg(test)]
use std::ffi::OsString;
#[cfg(test)]
use std::path::PathBuf;

// The names this module's controls reach through `super::`,
// re-exported so a control sees the whole module.
#[cfg(test)]
pub(super) use crate::package_repair_command_the_childs_end::{
    WAIT_FAILS_ONCE_THIS_FILE_EXISTS,
    reap_within,
};
#[cfg(test)]
pub(super) use crate::package_repair_command_words::{
    OUTPUT_TAIL_BYTES,
    notices_said,
    tail_of,
};

/// Command controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_command_tests.rs"]
mod tests;
