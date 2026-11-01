//! Running the toolchain lookup:
//!  the command line that confines it like a server,
//!  and a run that cannot outlast its time limit.
//!
//! The lookup runs the worktree's own `mpm-shadow` link,
//!  and what a worktree links is project state:
//!  it never runs on the bare host.
//!  It runs in the sandbox the servers get,
//!  without network,
//!  with the whole file system and the project read-only,
//!  and with a cleared environment.

/// A bounded run,
/// and what it gives back.
pub use super::run::{Completed, run_bounded};
/// The same sandbox recipe,
///  probe,
///  and private state as a server's.
use crate::language::{
    confine::confine_with,
    launch::{LaunchRequest, prepare},
};
/// What:
///  `Command` describes a child process;
///  `Stdio` says where its streams go.
/// Why:
///  The lookup is started with no input and with both output streams captured.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { spawn } from 'node:child_process';
/// ```
use std::{collections::HashMap, path::PathBuf, process::Command, time::Duration};

/// What:
///  Variables that name places and nothing else,
///  passed on when the application has them:
///  the home directory (the parent of the machine's toolchain root)
///  and the XDG base directories.
/// Why:
///  The lookup's launcher derives the machine's toolchain root from these,
///  and they name places,
///  never credentials;
///  every other variable of the application is cleared,
///  so nothing of it (a `NODE_OPTIONS`,
///  a token) reaches the run.
pub const LOCATION_VARIABLES: &[&str] =
    &["HOME", "XDG_CONFIG_HOME", "XDG_DATA_HOME", "XDG_STATE_HOME"];

/// What:
///  One lookup run:
///  its program, its arguments and the directory it runs in.
/// Why:
///  The project is named only as the working directory and inside the absolute
///  program path,
///  so a directory name that starts with a dash can never be read as an option.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Invocation = { program: string; arguments: string[]; directory: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invocation {
    /// The program to run,
    ///  an absolute path.
    pub program: PathBuf,
    /// Arguments after the program name.
    pub arguments: Vec<String>,
    /// Working directory of the run.
    pub directory: PathBuf,
}

/// Why a run did not produce a result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RunFailure = { kind: 'refused'; reason: string } | { kind: 'failed'; reason: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunFailure {
    /// The sandbox cannot be set up,
    ///  so nothing is run;
    ///  every server is refused for the same reason.
    Refused(
        /// What the confinement reported.
        String,
    ),
    /// The lookup could not be started,
    ///  or did not finish in time.
    Failed(
        /// What was observed.
        String,
    ),
}

/// What:
///  The sandbox request for one lookup run.
///  `locations` holds the location variables the application has,
///  as `(name, value)` pairs.
/// Why:
///  The request carries only what the lookup needs:
///  its program,
///  its own directories,
///  and the project to read.
///  Directories the sandbox replaced with its own (below `/tmp`,
///  `/run`,
///  `/dev`) are bound again read-only,
///  so a toolchain installed there still works.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function request(invocation: Invocation, project: ProjectView, locations: [string, string][]): LaunchRequest
/// ```
pub fn launch_request(
    invocation: &Invocation,
    project: &super::ToolchainRequest,
    locations: &[(String, String)],
) -> LaunchRequest {
    let mut environment: HashMap<String, String> = HashMap::new();
    let mut read_only_binds: Vec<PathBuf> = Vec::new();
    for (name, value) in locations {
        environment.insert(name.clone(), value.clone());
        for part in std::env::split_paths(value) {
            if part.is_absolute() && part.exists() {
                read_only_binds.push(part);
            }
        }
    }
    return LaunchRequest {
        server: "toolchain".to_string(),
        executable: invocation.program.clone(),
        args: invocation.arguments.clone(),
        environment,
        settings: None,
        project_root: project.project_root.clone(),
        project_spellings: project.project_spellings.clone(),
        read_only_binds,
        state_root: project.state_root.clone(),
    };
}

/// What:
///  The location variables of this process.
/// Why:
///  Read once per resolution,
///  so a test can pass its own pairs to `launch_request`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const locations = () => LOCATION_VARIABLES.flatMap(name => (env[name] ? [[name, env[name]]] : []));
/// ```
pub fn locations_from_environment() -> Vec<(String, String)> {
    let mut found = Vec::new();
    for name in LOCATION_VARIABLES {
        if let Ok(value) = std::env::var(name) {
            found.push((name.to_string(), value));
        }
    }
    return found;
}

/// What:
///  Run the lookup in the servers' sandbox.
///  `bubblewrap` is the bubblewrap program.
/// Why:
///  The confinement is the one function every server goes through,
///  so the lookup cannot get a weaker sandbox than a server.
///  A refusal is reported apart from a failed run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runConfined(invocation: Invocation, project: ProjectView, limitMs: number): Completed // throws RunFailure
/// ```
pub fn run_confined(
    invocation: &Invocation,
    project: &super::ToolchainRequest,
    limit: Duration,
    bubblewrap: &str,
) -> Result<Completed, RunFailure> {
    let request = launch_request(invocation, project, &locations_from_environment());
    let launch = confine_with(&request, bubblewrap).map_err(RunFailure::Refused)?;
    prepare(
        &launch,
        &project.project_root,
        project.state_root.as_deref(),
    )
    .map_err(RunFailure::Refused)?;
    let mut command = Command::new(&launch.command);
    command
        .args(&launch.args)
        .env_clear()
        .current_dir(&invocation.directory);
    tracing::debug!(program = %invocation.program.display(), arguments = ?invocation.arguments, directory = %invocation.directory.display(), "running the toolchain lookup in the sandbox");
    return run_bounded(command, limit).map_err(RunFailure::Failed);
}

/// The sandbox request's contents
///  and the program lookup.
#[cfg(test)]
#[path = "command_tests.rs"]
mod tests;
