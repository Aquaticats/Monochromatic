//! Servers that the project's toolchain provides.
//!
//! When the opened folder sits in a worktree that carries the toolchain's
//! `bin-paths` entry point at or above it,
//! the tools that entry point lists are where the language servers are looked up
//! (before the application's `PATH`),
//! and their
//! directories are on the `PATH` the confined server gets,
//! so a Node launcher finds its `node`.
//! A folder without one keeps the plain `PATH` lookup.
//!
//! The lookup is asked once per registry build,
//! never per server,
//! and only for what it resolves:
//! `node package/dev-script/file-enforcer/src/toolchain/bin-paths.ts`,
//! the published bin-paths interface
//! (`doc/planning/mpm-toolchain-migration.md`).
//! It runs in the servers' sandbox
//! (`command.rs`),
//! with a time
//! limit,
//! on the language thread.
//! It never runs a located tool
//! (that would execute project code to answer a question about paths)
//! and never installs or repairs anything.

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

/// The toolchain's published lookup entry point,
///  relative to a worktree root.
///  Pinned by `doc/planning/mpm-toolchain-migration.md`,
///  which names the command consumers run.
pub const ENTRY_POINT: &str = "package/dev-script/file-enforcer/src/toolchain/bin-paths.ts";

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
///  the node program run in the sandbox.
struct ConfinedRunner<'a> {
    /// The node program found on the machine.
    program: &'a Path,
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
        return command::run_confined(self.program, invocation, self.request, limit, BUBBLEWRAP);
    }
}

/// What:
///  The entry point of the worktree that contains a project root.
/// Why:
///  The entry point is published at one path in a toolchain worktree,
///  and a folder opened below the worktree root is served by the same worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EntryPoint = { worktreeRoot: string; script: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryPoint {
    /// The worktree root the entry point belongs to.
    pub worktree_root: PathBuf,
    /// The entry point itself,
    ///  an absolute path.
    pub script: PathBuf,
}

/// What:
///  `ENTRY_POINT` at `root` or at the closest directory above it.
///  `Option<EntryPoint>` is that worktree and script,
///  or nothing when no ancestor carries the entry point.
/// Why:
///  A folder without the entry point has no toolchain this lookup can ask,
///  and the walk stops at the file system root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entryPoint = (root: string) => ...
/// ```
pub fn entry_point(root: &Path) -> Option<EntryPoint> {
    for worktree in root.ancestors() {
        let script = worktree.join(ENTRY_POINT);
        if script.is_file() {
            return Some(EntryPoint {
                worktree_root: worktree.to_path_buf(),
                script,
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
///  find node,
/// run the entry point in the sandbox,
/// and read what it says.
/// Why:
///  A missing node program is not a failure:
///  the folder is then not a toolchain project as far as the application can tell.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const resolveConfined: ToolchainPolicy = request => resolveWith(request, findNode(), runConfined);
/// ```
pub fn resolve_confined(request: &ToolchainRequest) -> ToolchainState {
    let Some(program) = command::find_node() else {
        tracing::debug!(
            "no node program in the toolchain root or on PATH; servers are looked up on PATH"
        );
        return ToolchainState::Unused("no node program was found".to_string());
    };
    let runner = ConfinedRunner {
        program: &program,
        request,
    };
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
                "`node {ENTRY_POINT}` ended with {} and printed no message",
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
///  the entry point at or above the project root is what makes the folder a
///  toolchain project,
///  and its answer is the tool directories.
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
        return ToolchainState::Unused("the project has no toolchain entry point".to_string());
    };
    let started = Instant::now();
    // The entry point is named by its real path,
    //  so a project opened through a link runs the same file the entry point sees.
    let script = match std::fs::canonicalize(&entry.script) {
        Ok(real) => real,
        Err(_) => entry.script.clone(),
    };
    let invocation = command::Invocation {
        arguments: vec![script.to_string_lossy().into_owned()],
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
                message: format!("`node {ENTRY_POINT}` {reason}"),
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
