//! Servers that the project's toolchain provides.
//!
//! When the opened folder sits in a worktree that carries a `toolchain.plan.jsonc` at or above it,
//! the tools that the worktree's `mpm-shadow` lists are where the language servers are looked up
//! (before the application's `PATH`),
//! and their
//! directories are on the `PATH` the confined server gets,
//! so a Node launcher finds its `node`.
//! A folder without one keeps the plain `PATH` lookup.
//!
//! The lookup is asked once per registry build,
//! never per server,
//! and only for what it resolves:
//! `mpm-shadow bin --json`,
//! run by the explicit path `.cache/toolchain-bin/mpm-shadow` of the worktree,
//! the published bin interface
//! (`package/mpm-shadow/cli/README.md`).
//! It runs in the servers' sandbox
//! (`command.rs`),
//! with a time
//! limit,
//! on the language thread.
//! It never runs a located tool
//! (that would execute project code to answer a question about paths)
//! and never installs or repairs anything.
//! A worktree whose plan exists but whose `mpm-shadow` link does not (nothing installed yet)
//! is a failed lookup with a notice that names `mise run toolchain:install`.

/// The lookup's command line,
///  the sandbox,
///  and the time-limited run.
pub mod command;
/// What the lookup prints,
///  read as text.
pub mod parse;
/// A run that cannot outlast its time limit.
mod run;
/// Program lookup and the `PATH` a server gets.
pub mod search;
/// The outcome of asking the toolchain about a project.
pub mod state;

/// What the lookup and the reader need from the outcome.
pub use state::{Failed, Provided, ToolchainState};

/// The sandbox the run uses.
use crate::language::confine::BUBBLEWRAP;
/// What:
///  `Path` and `PathBuf` are borrowed and owned filesystem paths;
///  `Duration` and `Instant` are a time span and a clock reading.
/// Why:
///  The lookup has one time limit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Path = string;
/// ```
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

/// The file that makes a directory a toolchain worktree,
///  relative to a worktree root.
pub const PLAN_FILE: &str = "toolchain.plan.jsonc";

/// The program that answers the lookup,
///  relative to a worktree root:
///  the exposure link of the plan's `mpm-shadow`,
///  named explicitly because no `PATH` of the application holds it.
pub const PROGRAM: &str = ".cache/toolchain-bin/mpm-shadow";

/// The lookup's arguments after the program name:
///  the IDE launcher document of `mpm-shadow`.
pub const ARGUMENTS: [&str; 2] = ["bin", "--json"];

/// The lookup as a person types it,
///  for messages.
pub const COMMAND_LINE: &str = "mpm-shadow bin --json";

/// Longest time the lookup run may take.
///  Chosen,
///  not measured:
///  the run it replaces took 25 to 600 ms in the measurements of
///  `doc/troubleshooting/mise-servers-for-the-confined-ide.md`.
pub const TIME_LIMIT: Duration = Duration::from_secs(10);

/// Shortest time between two resolutions of the same project.
///  Chosen,
///  not measured:
///  a failed or incomplete resolution is tried again when a file is displayed,
///  but not at every display.
pub const RETRY_AFTER: Duration = Duration::from_secs(2);

/// What:
///  What the policy needs to know about the project.
/// Why:
///  The sandbox binds the project and the private state like it does for a server.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolchainRequest = { projectRoot: string; projectSpellings: string[]; stateRoot?: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolchainRequest {
    /// Canonical project root.
    pub project_root: PathBuf,
    /// Other spellings of the project root that servers are given.
    pub project_spellings: Vec<PathBuf>,
    /// Private application state directory,
    ///  when the application has one.
    pub state_root: Option<PathBuf>,
}

/// What:
///  The function that resolves a project's toolchain,
///  held by the setup.
/// Why:
///  Like the launch policy,
///  one explicit function decides,
///  and the unconfined setup of tests never runs the lookup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolchainPolicy = (request: ToolchainRequest) => ToolchainState;
/// ```
pub type ToolchainPolicy = fn(&ToolchainRequest) -> ToolchainState;

/// What:
///  How one run is made.
///  A type that implements this trait makes the runs of the sandboxed lookup,
///  or answers them in a test without node.
/// Why:
///  The decision logic is the same for both,
///  and a named type carries what each needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Runner { run(invocation: Invocation, limit: number): Completed | RunFailure }
/// ```
pub trait Runner {
    /// Make one run within `limit`.
    fn run(
        &self,
        invocation: &command::Invocation,
        limit: Duration,
    ) -> Result<command::Completed, command::RunFailure>;
}

/// The runner of the production policy:
///  the worktree's `mpm-shadow` run in the sandbox.
struct ConfinedRunner<'a> {
    /// The project whose toolchain is asked about.
    request: &'a ToolchainRequest,
}

/// Each run is confined by bubblewrap.
impl Runner for ConfinedRunner<'_> {
    /// Run `invocation` in the sandbox.
    fn run(
        &self,
        invocation: &command::Invocation,
        limit: Duration,
    ) -> Result<command::Completed, command::RunFailure> {
        return command::run_confined(invocation, self.request, limit, BUBBLEWRAP);
    }
}

/// What:
///  The toolchain program of the worktree that contains a project root.
/// Why:
///  The plan file marks one directory as a toolchain worktree,
///  and a folder opened below the worktree root is served by the same worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EntryPoint = { worktreeRoot: string; program: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryPoint {
    /// The worktree root the plan file belongs to.
    pub worktree_root: PathBuf,
    /// The worktree's `mpm-shadow` link,
    ///  an absolute path that may not exist yet.
    pub program: PathBuf,
}

/// What:
///  The toolchain worktree at `root` or at the closest directory above it.
///  `Option<EntryPoint>` is that worktree and its program path,
///  or nothing when no ancestor carries a `PLAN_FILE`.
/// Why:
///  A folder without a plan has no toolchain this lookup can ask,
///  and the walk stops at the file system root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entryPoint = (root: string) => ...
/// ```
pub fn entry_point(root: &Path) -> Option<EntryPoint> {
    for worktree in root.ancestors() {
        if worktree.join(PLAN_FILE).is_file() {
            return Some(EntryPoint {
                worktree_root: worktree.to_path_buf(),
                program: worktree.join(PROGRAM),
            });
        }
    }
    return None;
}

/// What:
///  The policy of the unconfined setup:
///  the lookup is never run.
/// Why:
///  Tests and guard controls on disposable projects must not depend on the machine's toolchain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const resolveDisabled: ToolchainPolicy = () => ({ kind: 'disabled' });
/// ```
pub fn resolve_disabled(_request: &ToolchainRequest) -> ToolchainState {
    return ToolchainState::Disabled;
}

/// What:
///  The production policy:
///  run the worktree's `mpm-shadow` in the sandbox,
/// and read what it says.
/// Why:
///  The program is found by the worktree's own path,
///  so no program of the machine is looked up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const resolveConfined: ToolchainPolicy = request => resolveWith(request, runConfined);
/// ```
pub fn resolve_confined(request: &ToolchainRequest) -> ToolchainState {
    let runner = ConfinedRunner { request };
    return resolve_with(request, &runner);
}

/// What:
///  A message for a run that did not succeed.
/// Why:
///  An empty standard error still needs words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function failure(completed: Completed): Failed
/// ```
fn failure(completed: &command::Completed) -> Failed {
    let message = parse::message(&completed.stderr);
    if message.is_empty() {
        return Failed {
            message: format!(
                "`{COMMAND_LINE}` ended with {} and printed no message",
                completed.status
            ),
        };
    }
    return Failed { message };
}

/// What:
///  Ask the toolchain about a project with `runner`.
///  `&dyn Runner` is a borrowed value that makes one run.
/// Why:
///  The decision logic is the same for the sandboxed run and a test double:
///  the plan file at or above the project root is what makes the folder a
///  toolchain project,
///  and the program's answer is the tool directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveWith(request: ToolchainRequest, runner: Runner): ToolchainState
/// ```
pub fn resolve_with(request: &ToolchainRequest, runner: &dyn Runner) -> ToolchainState {
    let Some(entry) = entry_point(&request.project_root) else {
        tracing::debug!(
            root = %request.project_root.display(),
            "no toolchain entry point at or above the project; servers are looked up on PATH"
        );
        return ToolchainState::Unused("the project has no toolchain plan".to_string());
    };
    let started = Instant::now();
    // A plan without the program link means nothing is installed in this worktree yet;
    //  a dangling link counts as missing.
    if !entry.program.is_file() {
        tracing::warn!(
            program = %entry.program.display(),
            "the worktree has a toolchain plan but no mpm-shadow link; the toolchain is not installed"
        );
        return ToolchainState::Failed(Failed {
            message: format!(
                "`{COMMAND_LINE}` cannot run: {} does not exist, so the toolchain is not installed in this worktree yet",
                entry.program.display()
            ),
        });
    }
    // The program is named by the worktree's own link path,
    //  never by the file behind it:
    //  the launcher that the link reaches learns the tree from the path it was started under.
    let invocation = command::Invocation {
        program: entry.program.clone(),
        arguments: ARGUMENTS
            .iter()
            .map(|argument| return (*argument).to_string())
            .collect(),
        directory: request.project_root.clone(),
    };
    let done = match runner.run(&invocation, TIME_LIMIT) {
        Ok(done) => done,
        Err(command::RunFailure::Refused(reason)) => {
            tracing::warn!(%reason, "the toolchain lookup is not run: the sandbox is not available");
            return ToolchainState::Unused(reason);
        }
        Err(command::RunFailure::Failed(reason)) => {
            tracing::warn!(%reason, "the toolchain lookup could not serve the project");
            return ToolchainState::Failed(Failed {
                message: format!("`{COMMAND_LINE}` {reason}"),
            });
        }
    };
    if !done.success {
        let failed = failure(&done);
        tracing::warn!(status = %done.status, message = %failed.message, "the toolchain lookup could not serve the project");
        return ToolchainState::Failed(failed);
    }
    let directories = match parse::bin_paths(&done.stdout) {
        Ok(directories) => directories,
        Err(reason) => {
            tracing::warn!(%reason, "the toolchain lookup printed an answer that cannot be read");
            return ToolchainState::Failed(Failed { message: reason });
        }
    };
    let warnings = parse::message(&done.stderr);
    tracing::info!(
        directories = directories.len(),
        elapsed_ms = started.elapsed().as_millis(),
        "the toolchain provides tool directories for the project"
    );
    return ToolchainState::Provided(Provided {
        directories,
        warnings,
    });
}

/// What:
///  Whether the state should be resolved again now.
///  `age` is the time since the last resolution;
///  `server_missing` says a server of the language being displayed was not found.
/// Why:
///  A failure may have been fixed (the user provisioned the toolchain),
///  and a missing server may have been installed.
///  A project the toolchain does not serve stays as it is until the application restarts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const due = (state: ToolchainState, ageMs: number, serverMissing: boolean) => ...
/// ```
pub fn due(state: &ToolchainState, age: Duration, server_missing: bool) -> bool {
    return match state {
        ToolchainState::Failed(_) => age >= RETRY_AFTER,
        ToolchainState::Provided(_) => server_missing && age >= RETRY_AFTER,
        ToolchainState::Disabled | ToolchainState::Unused(_) => false,
    };
}

/// Resolution against a double that answers like the entry point,
///  the notice text,
///  and the retry rule.
#[cfg(test)]
#[path = "toolchain_tests.rs"]
mod tests;
