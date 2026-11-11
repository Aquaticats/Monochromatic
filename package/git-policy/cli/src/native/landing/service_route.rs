//! What:
//!  How a commit reaches the resident service:
//! already listening,
//! or set up first by the wrapper.
//! Why:
//!  There is no switch and no install state for the person to manage (`doc/planning/cli-git-resident-service.md`,
//!  "Provisioning").
//! The route follows the machine:
//! a user session bus and the wrapper's own executable (which is the service
//!  program too) mean the wrapper can set the service up itself,
//! and keeps it set up (the socket is the manager's and outlives every upgrade,
//! so
//!  its presence never says the installed units name this release's program);
//! a socket without them is used as it is;
//!  anything else has no route,
//! and the commit stops with the plain message of its platform,
//! because the service is the only
//!  writer of a commit.
//! The decision reads the file system only and starts no process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const route = chooseRoute({ runtimeDir, busAddress, configHome, homeDir, ownExecutable, realGit }); // undefined: no route
//! ```

use crate::repo_read_layout::read_layout_in;
/// A pathname from the bytes Git printed for it,
///  and one answer without its terminating line feed.
use crate::hooks_path_values::{path_answer, without_the_terminator};
use crate::service_instance::{SOCKET_DIRECTORY, socket_path, try_instance_of};
use crate::service_provision::{ProvisionHost, RETIREMENT_WAIT};
use crate::service_server::Bind;
use crate::systemd_dbus::session_bus_path;
use crate::transaction_git::{GitContext, GitRequest, run_git};
use std::path::{Path, PathBuf};

static OWN_EXECUTABLE: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/// Remembers where the running wrapper is,
/// once,
/// so the unit of the service can name it (the service is a mode of the same
///  executable).
///  Identification only:
/// nothing is started from this path (`doc/planning/cli-git-no-self-invocation.md`);
/// the path is
///  read to name this executable in a unit file that the operating system's manager runs,
/// as the program of its service mode.
pub fn remember_own_executable(path: &Path) {
    let _first = OWN_EXECUTABLE.set(path.to_path_buf());
}

/// The wrapper's own path,
/// when the process remembered it.
pub fn remembered_own_executable() -> Option<&'static Path> {
    return OWN_EXECUTABLE.get().map(PathBuf::as_path);
}

/// What the route decision reads.
#[derive(Clone, Copy, Debug)]
pub struct RouteInputs<'a> {
    /// `XDG_RUNTIME_DIR`.
    pub runtime_dir: Option<&'a Path>,
    /// `DBUS_SESSION_BUS_ADDRESS`.
    pub bus_address: Option<&'a str>,
    /// `XDG_CONFIG_HOME`.
    pub config_home: Option<&'a Path>,
    /// `HOME`.
    pub home: Option<&'a Path>,
    /// The running wrapper's own executable.
    pub own_executable: Option<&'a Path>,
    /// The real Git the service will run.
    pub real_git: &'a Path,
    /// The worktree's own Git directory,
    /// which names the service instance the commit goes to.
    pub git_dir: &'a Path,
}

/// How a commit reaches the service:
/// through the service at `address`,
/// after provisioning when `provision` is present.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Route {
    /// Where the service listens.
    pub address: Bind,
    /// What to set up,
    /// or to check is set up,
    /// first;
    /// absent only when a socket exists and the wrapper cannot set up
    ///  a service itself (no session bus,
    /// or no executable of its own to name).
    pub provision: Option<ProvisionHost>,
}

/// The directory a user's own unit files live in.
pub fn user_unit_dir(config_home: Option<&Path>, home: Option<&Path>) -> Option<PathBuf> {
    // The configuration home when one is set, else `.config` under the home directory.
    if let Some(directory) = config_home {
        return Some(directory.join("systemd").join("user"));
    }
    let directory: &Path = home?;
    return Some(directory.join(".config").join("systemd").join("user"));
}

/// What:
///  Decides the route.
/// Why:
///   Provisioning needs the session bus socket and the wrapper's own executable,
/// and it is always handed back when both are there,
///  even when the socket exists:
/// the socket outlives an upgrade,
/// and the units it activates may name a program the upgrade
///  removed.
/// Whether anything must be written is decided where the units are read (`provision`),
/// so this decision stays
///  free of reads.
/// A socket that exists without the means to provision is used as it is,
/// whoever made it;
/// with neither
///  the socket nor the means there is no route.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// chooseRoute(inputs) // undefined when nothing can reach or start a service
/// ```
pub fn choose_route(inputs: &RouteInputs<'_>) -> Option<Route> {
    let runtime_dir = inputs.runtime_dir?;
    // A Git directory that cannot be resolved (a link to a drive that is not mounted) names no instance: no socket path or
    // unit instance is made for it.
    let instance: String = try_instance_of(inputs.git_dir).ok()?;
    let socket: PathBuf = socket_path(runtime_dir, instance.as_str());
    let socket_exists: bool = socket.exists();
    // A bus socket that is not there is no bus to provision through.
    let bus: PathBuf = match session_bus_path(inputs.bus_address, Some(runtime_dir)) {
        Some(path) if path.exists() => path,
        _ => return if socket_exists { Some(Route { address: Bind::Path(socket), provision: None }) } else { None },
    };
    // The service is a mode of the wrapper's own executable (`service_main.rs`), so the unit names that file; a path that
    //  is not a file (a deleted executable) is no program for a unit to name.
    let service_binary: PathBuf = match inputs.own_executable {
        Some(executable) if executable.is_file() => executable.to_path_buf(),
        _ => return if socket_exists { Some(Route { address: Bind::Path(socket), provision: None }) } else { None },
    };
    let host: ProvisionHost = ProvisionHost {
        runtime_dir: runtime_dir.to_path_buf(),
        user_unit_dir: user_unit_dir(inputs.config_home, inputs.home),
        bus,
        service_binary,
        git: inputs.real_git.to_path_buf(),
        instance,
        git_dir: inputs.git_dir.to_path_buf(),
        retirement_wait: RETIREMENT_WAIT,
    };
    return Some(Route { address: Bind::Path(socket), provision: Some(host) });
}

/// What `git monochromatic-git-policy-cli service status` prints and exits with.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusReport {
    /// Exit status:
    /// 0 when the service answered.
    pub code: i32,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
}

/// What:
///  The real path of the Git directory of the worktree a command runs in,
/// read from the file system when the layout is a plain
///  one and from Git otherwise.
/// Why:
///   The Git directory names the service instance,
/// so the route cannot be chosen without it;
/// the preparation of the commit
///  asks the same question later and shares the answer's cost when the file system can give it.
///  A command line that names its repository selects the instance of that one:
///  the reader answers for the directory it is given and cannot honour a Git directory beside the work tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// worktreeGitDir(context, cwd) // undefined outside a repository
/// ```
pub fn worktree_git_dir(context: &GitContext, cwd: &Path) -> Option<PathBuf> {
    let selection: Vec<std::ffi::OsString> = crate::repository_selection::from_global_options(cwd, context.global_prefix.as_slice());
    let named: bool = !selection.is_empty();
    if !named
        && let Some(layout) = read_layout_in(cwd, context.overlay.as_slice())
    {
        return Some(layout.git_dir);
    }
    let mut request: GitRequest = GitRequest::new(cwd, &["rev-parse", "--path-format=absolute", "--absolute-git-dir"]);
    if named {
        request.without_prefix = true;
        request.repository_selection = selection;
    }
    let output = run_git(context, &request).ok()?;
    if !output.succeeded() {
        return None;
    }
    // The answer is a pathname, so it is read as the bytes Git printed it as, with only Git's one terminating line
    //  feed removed: a spelling of it named a directory that does not exist when the name was not UTF-8, and trimming
    //  whitespace after the terminator removed a last byte that belongs to the name (a directory named `metadata `
    //  or one whose name ends in a line feed became `metadata`), so the route resolved another path and the command
    //  stopped instead of reaching the service of the repository it selected.
    let printed: &[u8] = without_the_terminator(output.stdout.as_slice());
    return if printed.is_empty() { None } else { Some(path_answer(printed)) };
}

/// What:
///  The path an environment variable holds,
/// nothing when it is unset or empty.
/// Why:
///  An empty runtime,
/// configuration or home directory names no place,
/// so the route treats it as unset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nonEmptyPathVariable = (name: string) => process.env[name] ? process.env[name] : undefined;
/// ```
pub(crate) fn non_empty_path_variable(name: &str) -> Option<PathBuf> {
    return match std::env::var_os(name) {
        Some(value) if !value.is_empty() => Some(PathBuf::from(value)),
        _ => None,
    };
}

/// What:
///  The route to the service of a worktree from this process's environment.
/// Why:
///   The commit and the commands that hand a push over after they moved a branch reach the service the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// routeFromEnvironment(realGit, gitDir) // undefined: no way to reach a service
/// ```
pub fn route_from_environment(real_git: &Path, git_dir: &Path) -> Option<Route> {
    // The read of this process's environment is the read of the invocation's: `main` hands the entry `std::env::vars_os()` as
    //  that environment, and nothing in the crate changes the wrapper's own environment, so a command started with another
    //  `HOME` reads that home here. No service chooses a route (a service start goes to `service_main` before any command's
    //  route is picked), so this read never answers with a manager's home while serving.
    let runtime: Option<PathBuf> = non_empty_path_variable("XDG_RUNTIME_DIR");
    let config_home: Option<PathBuf> = non_empty_path_variable("XDG_CONFIG_HOME");
    let home: Option<PathBuf> = non_empty_path_variable("HOME");
    let bus_address: Option<String> = std::env::var("DBUS_SESSION_BUS_ADDRESS").ok();
    return choose_route(&RouteInputs {
        runtime_dir: runtime.as_deref(),
        bus_address: bus_address.as_deref(),
        config_home: config_home.as_deref(),
        home: home.as_deref(),
        own_executable: remembered_own_executable(),
        real_git,
        git_dir,
    });
}

/// What:
///  Whether something listens on the socket:
/// a connection to it succeeds.
/// Why:
///  A service is running when its socket takes a connection.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const serviceAnswers = async (socket: string) => { try { await connect(socket); return true; } catch { return false; } };
/// ```
fn service_answers(socket: &Path) -> bool {
    return std::os::unix::net::UnixStream::connect(socket).is_ok();
}

/// What:
///  Whether any socket file in a directory takes a connection.
/// Why:
///  Without a worktree to ask about,
/// any commit service of the user counts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readdir(directory)).some((name) => name.endsWith('.sock') && serviceAnswers(join(directory, name)))
/// ```
fn any_socket_answers(directory: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return false;
    };
    for entry in entries.flatten() {
        let path: PathBuf = entry.path();
        let is_socket_file: bool = match path.extension() {
            Some(extension) => extension == "sock",
            None => false,
        };
        if is_socket_file && service_answers(path.as_path()) {
            return true;
        }
    }
    return false;
}

/// What:
///  Says whether the commit service of a worktree answers on its socket.
/// Why:
///   A diagnostic only:
/// there is nothing to install or start by hand.
/// Connecting is what a commit does,
/// so a service that the
///  manager would start on demand is started by this check too.
/// Outside a repository there is no worktree to ask for,
/// and the
///  answer counts the instances that answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// serviceStatus(runtimeDir, gitDir)
/// ```
pub fn service_status(runtime_dir: Option<&Path>, git_dir: Option<&Path>) -> StatusReport {
    let (answered, what): (bool, &str) = match (runtime_dir, git_dir) {
        (Some(runtime), Some(git)) => match try_instance_of(git) {
            Ok(instance) => (service_answers(socket_path(runtime, instance.as_str()).as_path()), "The commit service of this worktree"),
            Err(unreachable) => {
                return StatusReport {
                    code: 1,
                    stdout: String::new(),
                    stderr: format!(
                        "The commit service of this worktree cannot be asked: {unreachable}. Nothing was connected to or started; \
                         if the repository is on a drive that is not mounted, mount it and run this again.\n"
                    ),
                };
            }
        },
        (Some(runtime), None) => {
            let any: bool = any_socket_answers(runtime.join(SOCKET_DIRECTORY).as_path());
            (any, "A commit service")
        }
        (None, _) => (false, "The commit service"),
    };
    return if answered {
        StatusReport { code: 0, stdout: format!("{what} is running and answers.\n"), stderr: String::new() }
    } else {
        StatusReport {
            code: 1,
            stdout: String::new(),
            stderr: format!(
                "{what} does not answer here. It sets itself up the first time a commit needs it, so there is nothing to \
                 install; if a commit fails, its message says what to do.\n"
            ),
        }
    };
}

/// Route controls stay out of the release executable.
#[cfg(test)]
#[path = "service_route_tests.rs"]
mod tests;
