//! What:
//! The start of a contained run:
//! the check that the program exists before a scope is asked for,
//! the launcher record that lets a recovery tell a killed wrapper from a running one,
//! and the start of the child inside the scope or inside its own process group.
//! Why:
//! A start is the one moment at which the journal must already say what is about to run,
//! because a wrapper killed between the record and the spawn is indistinguishable from one that never started;
//! split out of `bounded_run_containment.rs`,
//! which decides the containment and stops it.
//! The items are re-exported from `bounded_run_containment`,
//! so their paths there are unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // bounded_run_start.ts: programResolves(command), noteLauncher(...), startContained(...)
//! ```

/// What:
/// `use crate::bounded_run::{RunError, ScopeState, append_record, append_record_with, configure_stdio, trace};` imports the error of a
/// bounded run,
/// the state of its journal,
/// the two record writes,
/// the standard-input setup and the trace.
/// Why:
/// The start writes its records before the program runs,
/// sets up the stdio of the launcher it starts and traces the spawn.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RunError, ScopeState, append_record, append_record_with, configure_stdio, trace } from 'crate/bounded_run';
/// ```
use crate::bounded_run::{
    RunError,
    ScopeState,
    append_record,
    append_record_with,
    configure_stdio,
    trace,
};
/// What:
/// `use crate::bounded_run::{START_HOOK, trace_text};` imports the hook a test puts inside a start and the one way a test build turns a trace event into text.
/// Why:
/// Both exist in a test build only,
/// so this import is compiled for tests alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { START_HOOK, trace_text } from 'crate/bounded_run';
/// ```
#[cfg(test)]
use crate::bounded_run::{
    START_HOOK,
    trace_text,
};
/// What:
/// `use crate::bounded_run_containment::{Containment, lose, stop_process_group};` imports the choice of containment,
/// the mark that writers may remain and the stop of a bare process group.
/// Why:
/// The start reads the choice,
/// marks the transaction lost when a child cannot be recorded,
/// and stops a child whose process group could not be announced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Containment, lose, stop_process_group } from 'crate/bounded_run_containment';
/// ```
use crate::bounded_run_containment::{
    Containment,
    lose,
    stop_process_group,
};
/// What:
/// `use crate::control_group::Record;` imports the journal record a scope or a process group writes about itself.
/// Why:
/// The launcher record and the process-group announcement are two of its variants.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Record } from 'crate/control_group';
/// ```
use crate::control_group::Record;
/// What:
/// `use rustix::process::Pid;` imports the operating system's process id type.
/// Why:
/// A process group is named by one,
/// and the weaker path stops the group of the child it started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Node names a process (not a process group) by a number
/// ```
use rustix::process::Pid;
/// What:
/// `use std::io;` imports the standard library's input/output module so this file can name it unqualified.
/// Why:
/// A record that cannot be written is an `io::Error`,
/// and the start reports it as the bounded run's error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // an I/O failure is an ordinary thrown Error in TS
/// ```
use std::io;
/// What:
/// `use std::os::unix::process::CommandExt;` imports the trait that adds `process_group` to a command.
/// Why:
/// The weaker path starts the child as the leader of a new process group,
/// which is the only containment it then has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { spawn } from 'node:child_process';   // `detached: true` is the nearest thing
/// ```
use std::os::unix::process::CommandExt;
/// What:
/// `use std::path::{Path, PathBuf};` imports the borrowed and owned path types.
/// Why:
/// The journal log and the scope directory are lent to the record writes,
/// and the program search builds owned paths from the current directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::{
    Path,
    PathBuf,
};
/// What:
/// `use std::process::{Child, Command};` imports the child handle and the command builder.
/// Why:
/// The start runs a command and answers the child,
/// and the process-group path records the child's own process id as its leader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ChildProcess, spawn } from 'node:child_process';
/// ```
use std::process::{
    Child,
    Command,
};
/// What:
/// `use std::rc::Rc;` imports the reference-counted pointer type (its sibling `Arc` is the one that is safe to share between threads).
/// Why:
/// The journal state is shared inside this one thread only,
/// so the cheaper single-threaded count is enough.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // every object reference in TS is shared and counted by the garbage collector
/// ```
use std::rc::Rc;
/// What:
/// `fn program_resolves(command: &Command) -> bool` says whether the program of the command exists as an executable file,
/// searched on the `PATH` the command will have when its name has no slash.
/// Why:
/// `systemd-run` reports a program it cannot start as an exit status like any other,
/// so a program that does not exist is found before the launcher is started and reported as the I/O error it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function programResolves(command: Command): boolean
/// ```
pub(crate) fn program_resolves(command: &Command) -> bool {
    let program: &std::ffi::OsStr = command.get_program();
    let relative_to: PathBuf = command
        .get_current_dir()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    if program.as_encoded_bytes().contains(&b'/') {
        return crate::control_group::is_executable_file(&relative_to.join(program));
    }
    let mut own_path: Option<std::ffi::OsString> = None;
    for (name, value) in command.get_envs() {
        if name == "PATH" {
            own_path = value.map(std::ffi::OsStr::to_os_string);
            break;
        }
    }
    if own_path.is_none() {
        own_path = std::env::var_os("PATH");
    }
    if let Some(search) = own_path {
        for directory in std::env::split_paths(&search) {
            if crate::control_group::is_executable_file(&relative_to.join(directory).join(program))
            {
                return true;
            }
        }
    }
    return false;
}

/// What:
/// `fn run_start_hook()` runs the hook a test put inside the start of a run,
/// if any.
/// Why:
/// It is the boundary at which a test overlaps other work with a start,
/// or kills a wrapper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runStartHook(): void
/// ```
#[cfg(test)]
pub(crate) fn run_start_hook() {
    if let Some(hook) = START_HOOK.with(|slot| {
        return slot.borrow().clone();
    }) {
        hook();
    }
}

/// The production build has no hook.
#[cfg(not(test))]
pub(crate) fn run_start_hook() {}

/// What:
/// `fn note_launcher(log: &Path, scope: &Path, child: &Child)` appends the launcher record:
/// the process id,
/// the start time and the
/// process id namespace of the process that is about to become the program.
/// Why:
/// A recovery that finds this process gone knows it can never join the scope,
/// so it can read the scope as final without an
/// acknowledgement (round 11).
/// A line that cannot be written is only a missing proof:
/// the journal then waits for an acknowledgement like an older journal,
/// so the
/// run goes on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function noteLauncher(log: string, scope: string, child: ChildProcess): void
/// ```
pub(crate) fn note_launcher(
    log: &Path,
    scope: &Path,
    child: &Child,
) {
    let pid: i32 = i32::try_from(child.id()).unwrap_or(0);
    let noted: io::Result<()> = match crate::control_group::process_start(pid) {
        Ok(start) => record_launcher(log, scope, pid, start),
        Err(error) => Err(error),
    };
    if let Err(error) = noted {
        eprintln!(
            "cargo-shadow: warning: the launcher of {} could not be recorded ({error}); if this wrapper is killed, a recovery will ask for an acknowledgement",
            scope.display()
        );
    }
}

/// What:
/// `fn record_launcher(log: &Path, scope: &Path, pid: i32, start: u64) -> io::Result<()>` appends the launcher record.
/// Why:
/// A function replaces the closure that `note_launcher` passed to `and_then`.
pub(crate) fn record_launcher(
    log: &Path,
    scope: &Path,
    pid: i32,
    start: u64,
) -> io::Result<()> {
    let pid_namespace: u64 = crate::control_group::pid_namespace_id()?;
    return append_record_with(
        log,
        &Record::Launcher {
            pid,
            start,
            pid_namespace,
            path: scope.to_path_buf(),
        },
        false,
    );
}

/// What:
/// `fn scope_record_failed(error: io::Error) -> RunError` is the refusal for a scope record that could not be written.
/// Why:
/// `Result::map_err` takes it by path instead of a closure.
pub(crate) fn scope_record_failed(error: io::Error) -> RunError {
    return RunError::Io(io::Error::new(
        error.kind(),
        format!("{error} (a program whose scope cannot be recorded is not started)"),
    ));
}

/// What:
/// `fn group_announcement_failed(error: io::Error) -> RunError` is the refusal for a process-group announcement that could not be written.
/// Why:
/// `Result::map_err` takes it by path instead of a closure.
pub(crate) fn group_announcement_failed(error: io::Error) -> RunError {
    return RunError::Io(io::Error::new(
        error.kind(),
        format!("{error} (a child whose process group cannot be announced is not started)"),
    ));
}

/// What:
/// `fn start_contained(command, containment, scope, has_input, clean_environment) -> Result<Child, RunError>` records the
/// containment in the journal and starts the child inside it.
/// Why:
/// For a scope the record is forced to disk before the program starts,
/// and a record that cannot be written means the program is not started.
/// For the weaker path the process group is only known once the child exists,
/// so a `pgroup starting` line is forced to disk first and the group's line follows the start;
/// a child whose group cannot be recorded is stopped at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startContained(command: Command, containment: Containment, scope: ScopeState | undefined, hasInput: boolean, cleanEnvironment: boolean): ChildProcess  // throws RunError
/// ```
pub(crate) fn start_contained(
    command: &mut Command,
    containment: &Containment,
    scope: Option<&Rc<ScopeState>>,
    has_input: bool,
    clean_environment: bool,
) -> Result<Child, RunError> {
    match containment {
        Containment::Scope {
            group,
            route,
        } => {
            if !program_resolves(command) {
                return Err(RunError::Io(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("{:?} was not found (os error 2)", command.get_program()),
                )));
            }
            if let Some(state) = scope {
                append_record(&state.log, &group.record()).map_err(scope_record_failed)?;
            }
            run_start_hook();
            let mut launch: Command = group.launcher(route, command, clean_environment);
            configure_stdio(&mut launch, has_input);
            // The text computes a file name, so this statement exists in a test build only.
            #[cfg(test)]
            trace_text(format!(
                "launch scope {}",
                group
                    .directory()
                    .file_name()
                    .map_or_else(String::new, |name| {
                        return name.to_string_lossy().into_owned();
                    })
            ));
            let child: Child = launch.spawn().map_err(RunError::Io)?;
            // The direct child of this process, as the kernel numbered it: a test compares it with what the program says it is.
            trace(format_args!("spawned {}", child.id()));
            crate::bounded_run::run_after_launch_hook(child.id());
            if let Some(state) = scope {
                note_launcher(&state.log, group.directory(), &child);
            }
            return Ok(child);
        }
        Containment::ProcessGroup => {
            if let Some(state) = scope {
                state.process_group_only.set(true);
            }
            command.process_group(0);
            // The child's group is its process id, known only after the spawn: a "starting" line is durable before it, so a
            // wrapper killed in between is seen by a recovery as one that may have a child running.
            if let Some(state) = scope {
                append_record(&state.log, &Record::ProcessGroupStarting)
                    .map_err(group_announcement_failed)?;
            }
            run_start_hook();
            trace(format_args!("launch pgroup"));
            let mut child: Child = command.spawn().map_err(RunError::Io)?;
            trace(format_args!("spawned {}", child.id()));
            let leader: i32 = i32::try_from(child.id()).unwrap_or(0);
            if let Some(state) = scope
                && let Err(error) = append_record(&state.log, &Record::ProcessGroup(leader))
            {
                let stopped: bool = match Pid::from_raw(leader) {
                    Some(group) => stop_process_group(&mut child, group),
                    None => false,
                };
                if !stopped {
                    lose(&Some(Rc::clone(state)));
                }
                return Err(RunError::Io(io::Error::new(
                    error.kind(),
                    format!(
                        "{error} (the child was stopped because its process group could not be recorded)"
                    ),
                )));
            }
            return Ok(child);
        }
    }
}
