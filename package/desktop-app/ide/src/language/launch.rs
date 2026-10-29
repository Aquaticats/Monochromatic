//! The launch seam:
//!  what the application knows about a server,
//!  and what is actually spawned.
//!
//! Every server definition passes through one `LaunchPolicy` before Helix sees it.
//!  The default
//! policy spawns the resolved server program unchanged.
//!  A confining policy returns a wrapper
//! command instead;
//!  if it cannot,
//!  it returns an error and the server is reported as refused.
//! There is no fallback to an unconfined launch.

/// What:
///  `Value` is any JSON value.
/// Why:
///  A server's settings table travels as JSON and a policy may override entries in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Value = unknown;
/// ```
use serde_json::Value;
/// What:
///  `HashMap` is a key-value table;
///  `PermissionsExt` adds Unix permission bits to file
///       metadata;
///  `Path` is a borrowed filesystem path and `PathBuf` its owned sibling.
/// Why:
///  Executables are found by walking `PATH` and checking the execute permission.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { accessSync, constants } from 'node:fs';
/// ```
use std::{
    collections::HashMap,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

/// What:
///  Everything the application knows about one server before launch.
///  `String` and
///       `PathBuf` own their data;
///  `Vec<String>` is a growable list;
///  `Option<...>` is "a value,
///       or nothing".
/// Why:
///  A policy needs the real program (already resolved to an absolute path),
///  the project
///      it must not write to,
///  and the private directory it may write to instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LaunchRequest = { server: string; executable: string; args: string[];
///                        environment: Record<string, string>; settings?: unknown;
///                        projectRoot: string; projectSpellings: string[]; readOnlyBinds: string[]; stateRoot?: string };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct LaunchRequest {
    /// Name of the server definition,
    ///  for example `rust-analyzer`.
    pub server: String,
    /// Absolute path of the server's own program.
    pub executable: PathBuf,
    /// The server's own arguments.
    pub args: Vec<String>,
    /// Environment variables the definition adds for the server.
    pub environment: HashMap<String, String>,
    /// The server's settings table,
    ///  sent as initialization options and configuration.
    pub settings: Option<Value>,
    /// Canonical project root.
    pub project_root: PathBuf,
    /// Other spellings of the project root that servers are given,
    ///  such as the one Helix derives
    /// from `PWD` when the project was reached through a symbolic link.
    pub project_spellings: Vec<PathBuf>,
    /// Directories the server reads but must not write,
    ///  such as the tool directories of mise;
    ///  a confining policy shows them again read-only when the sandbox replaced their location.
    pub read_only_binds: Vec<PathBuf>,
    /// Private application state directory outside the project,
    ///  when the application has one.
    pub state_root: Option<PathBuf>,
}

/// What is actually handed to Helix for one server.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ServerLaunch = { command: string; args: string[]; environment: Record<string, string>;
///                       settings?: unknown; directories: string[]; scratch: string[] };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ServerLaunch {
    /// Program to spawn:
    ///  the server itself,
    ///  or a wrapper that runs it.
    pub command: String,
    /// Arguments of that program;
    ///  a wrapper's recipe comes first,
    ///  then the server and its arguments.
    pub args: Vec<String>,
    /// Environment variables added for the spawned program.
    pub environment: HashMap<String, String>,
    /// Settings table after the policy's overrides.
    pub settings: Option<Value>,
    /// Directories that must exist before every launch;
    ///  created by the worker below the private state
    /// root,
    ///  which lies inside the project only when the home folder is opened.
    pub directories: Vec<PathBuf>,
    /// Directories emptied before every launch,
    ///  such as a private temporary directory.
    pub scratch: Vec<PathBuf>,
}

/// What:
///  `fn(...) -> ...` as a type is a plain function pointer:
///  a value that is a function.
///       `Result<ServerLaunch, String>` is either a launch (`Ok`) or a refusal reason (`Err`).
/// Why:
///  One explicit function decides how every server starts,
///  so write confinement can be
///      added without touching the worker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LaunchPolicy = (request: LaunchRequest) => ServerLaunch; // throws to refuse
/// ```
pub type LaunchPolicy = fn(&LaunchRequest) -> Result<ServerLaunch, String>;

/// What:
///  The default policy:
///  spawn the resolved server program itself,
///  unchanged.
/// Why:
///  It keeps the seam explicit while no confinement is wired in.
///  Servers launched this way
///      can write to the project,
///  so real servers may only be pointed at disposable projects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const launchDirectly: LaunchPolicy = request =>
///   ({ command: request.executable, args: request.args, environment: request.environment,
///      settings: request.settings, directories: [], scratch: [] });
/// ```
pub fn launch_directly(request: &LaunchRequest) -> Result<ServerLaunch, String> {
    // What: `to_str` returns `Option<&str>`: Helix stores commands as text, so a path that is
    //       not valid Unicode cannot be passed on. `ok_or_else` turns "nothing" into an error.
    // Why: Refusing is safer than spawning a lossily converted, and so different, path.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const command: string = request.executable;
    // ```
    let Some(command) = request.executable.to_str() else {
        return Err(format!(
            "the path of {} is not valid Unicode: {}",
            request.server,
            request.executable.display()
        ));
    };
    // `Ok(...)` is the success variant of `Result`; `clone` copies each borrowed part.
    return Ok(ServerLaunch {
        command: command.to_string(),
        args: request.args.clone(),
        environment: request.environment.clone(),
        settings: request.settings.clone(),
        directories: Vec::new(),
        scratch: Vec::new(),
    });
}

/// True for an existing regular file with at least one execute permission bit.
pub(crate) fn is_executable(path: &Path) -> bool {
    // What: `metadata` returns `Result`; `is_ok_and` is true only for success whose value passes
    //       the closure's test. `0o111` is an octal literal: the three execute bits.
    // Why: A directory or a non-executable file of the right name is not a program.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { accessSync(path, constants.X_OK); return statSync(path).isFile(); } catch { return false; }
    // ```
    return match std::fs::metadata(path) {
        Ok(found) => found.is_file() && found.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    };
}

/// What:
///  Find the absolute path of a server program,
///  or nothing.
///  A command containing a path
///       separator is taken relative to the project root;
///  a bare name is searched on `PATH`.
/// Why:
///  A wrapper in front of the server would hide a missing server from Helix's own lookup,
///      and a confining wrapper needs an absolute path after its `--`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveExecutable(command: string, root: string): string | undefined
/// ```
pub fn resolve_executable(command: &str, root: &Path) -> Option<PathBuf> {
    if command.contains('/') {
        // `join` keeps an absolute command as it is and puts a relative one under the root.
        let candidate = root.join(command);
        if is_executable(&candidate) {
            // `Some(...)` is the "value present" variant of `Option`.
            return Some(candidate);
        }
        // `None` is the "nothing" variant.
        return None;
    }
    // The trailing `?` returns `None` when `PATH` is unset.
    let search = std::env::var_os("PATH")?;
    for directory in std::env::split_paths(&search) {
        // A relative entry would depend on the working directory; only absolute ones are trusted.
        if !directory.is_absolute() {
            continue;
        }
        let candidate = directory.join(command);
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }
    return None;
}

/// The scratch directory of one process,
/// so processes on one project never empty each other's.
mod scratch;
/// The private state directory rules and the preparation of a launch's directories.
mod state;
/// Re-exported:
/// the confining policy names its scratch directory through the seam.
pub use scratch::scratch_name;
/// Re-exported:
/// the confining policy and the worker use them through the seam.
pub use state::{check_state_directory, prepare, resolve_existing};

/// Executable lookup,
///  the default policy,
///  and state-directory containment,
///  on disposable directories.
#[cfg(test)]
#[path = "launch_tests.rs"]
mod tests;
