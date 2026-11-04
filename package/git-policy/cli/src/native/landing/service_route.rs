//! What:
//!  How a commit reaches the resident service: already listening, or set up first by the wrapper.
//! Why:
//!  There is no switch and no install state for the person to manage (`doc/planning/cli-git-resident-service.md`,
//!  "Provisioning"). The route follows the machine: a user session bus and the wrapper's own executable (which is the service
//!  program too) mean the wrapper can set the service up itself, and keeps it set up (the socket is the manager's and outlives every upgrade, so
//!  its presence never says the installed units name this release's program); a socket without them is used as it is;
//!  anything else has no route, and the commit stops with the plain message of its platform, because the service is the only
//!  writer of a commit. The decision reads the file system only and starts no process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const route = chooseRoute({ runtimeDir, busAddress, configHome, homeDir, ownExecutable, realGit }); // undefined: no route
//! ```

use crate::repo_read_layout::read_layout_in;
use crate::service_instance::{SOCKET_DIRECTORY, instance_of, socket_for};
use crate::service_provision::{ProvisionHost, RETIREMENT_WAIT};
use crate::service_server::Bind;
use crate::systemd_dbus::session_bus_path;
use crate::transaction_git::{GitContext, GitRequest, run_git};
use std::path::{Path, PathBuf};

static OWN_EXECUTABLE: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/// Remembers where the running wrapper is, once, so the unit of the service can name it (the service is a mode of the same
///  executable).
///  Identification only: nothing is started from this path (`doc/planning/cli-git-no-self-invocation.md`); the path is
///  read to name this executable in a unit file that the operating system's manager runs, as the program of its service mode.
pub fn remember_own_executable(path: &Path) {
    let _first = OWN_EXECUTABLE.set(path.to_path_buf());
}

/// The wrapper's own path, when the process remembered it.
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
    /// The worktree's own Git directory, which names the service instance the commit goes to.
    pub git_dir: &'a Path,
}

/// How a commit reaches the service: through the service at `address`, after provisioning when `provision` is present.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Route {
    /// Where the service listens.
    pub address: Bind,
    /// What to set up, or to check is set up, first; absent only when a socket exists and the wrapper cannot set up
    ///  a service itself (no session bus, or no executable of its own to name).
    pub provision: Option<ProvisionHost>,
}

/// The directory a user's own unit files live in.
pub fn user_unit_dir(config_home: Option<&Path>, home: Option<&Path>) -> Option<PathBuf> {
    return config_home
        .map(|directory| { return directory.join("systemd").join("user"); })
        .or_else(|| { return home.map(|directory| { return directory.join(".config").join("systemd").join("user"); }); });
}

/// What:
///  Decides the route.
/// Why:
///   Provisioning needs the session bus socket and the wrapper's own executable, and it is always handed back when both are there,
///  even when the socket exists: the socket outlives an upgrade, and the units it activates may name a program the upgrade
///  removed. Whether anything must be written is decided where the units are read (`provision`), so this decision stays
///  free of reads. A socket that exists without the means to provision is used as it is, whoever made it; with neither
///  the socket nor the means there is no route.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// chooseRoute(inputs) // undefined when nothing can reach or start a service
/// ```
pub fn choose_route(inputs: &RouteInputs<'_>) -> Option<Route> {
    let runtime_dir = inputs.runtime_dir?;
    let socket: PathBuf = socket_for(runtime_dir, inputs.git_dir);
    let socket_exists: bool = socket.exists();
    let Some(bus) = session_bus_path(inputs.bus_address, Some(runtime_dir)).filter(|path| { return path.exists(); }) else {
        return if socket_exists { Some(Route { address: Bind::Path(socket), provision: None }) } else { None };
    };
    // The service is a mode of the wrapper's own executable (`service_main.rs`), so the unit names that file; a path that
    //  is not a file (a deleted executable) is no program for a unit to name.
    let Some(service_binary) = inputs.own_executable.map(Path::to_path_buf).filter(|path| { return path.is_file(); }) else {
        return if socket_exists { Some(Route { address: Bind::Path(socket), provision: None }) } else { None };
    };
    let host: ProvisionHost = ProvisionHost {
        runtime_dir: runtime_dir.to_path_buf(),
        user_unit_dir: user_unit_dir(inputs.config_home, inputs.home),
        bus,
        service_binary,
        git: inputs.real_git.to_path_buf(),
        instance: instance_of(inputs.git_dir),
        retirement_wait: RETIREMENT_WAIT,
    };
    return Some(Route { address: Bind::Path(socket), provision: Some(host) });
}

/// What `git cli-git service status` prints and exits with.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusReport {
    /// Exit status: 0 when the service answered.
    pub code: i32,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
}

/// What:
///  The real path of the Git directory of the worktree a command runs in, read from the file system when the layout is a plain
///  one and from Git otherwise.
/// Why:
///   The Git directory names the service instance, so the route cannot be chosen without it; the preparation of the commit
///  asks the same question later and shares the answer's cost when the file system can give it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// worktreeGitDir(context, cwd) // undefined outside a repository
/// ```
pub fn worktree_git_dir(context: &GitContext, cwd: &Path) -> Option<PathBuf> {
    if let Some(layout) = read_layout_in(cwd, context.overlay.as_slice()) {
        return Some(layout.git_dir);
    }
    let output = run_git(context, &GitRequest::new(cwd, &["rev-parse", "--path-format=absolute", "--absolute-git-dir"])).ok()?;
    if !output.succeeded() {
        return None;
    }
    let text: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string();
    return if text.is_empty() { None } else { Some(PathBuf::from(text)) };
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
    let path_variable = |name: &str| -> Option<PathBuf> { return std::env::var_os(name).filter(|value| { return !value.is_empty(); }).map(PathBuf::from); };
    let runtime: Option<PathBuf> = path_variable("XDG_RUNTIME_DIR");
    let config_home: Option<PathBuf> = path_variable("XDG_CONFIG_HOME");
    let home: Option<PathBuf> = path_variable("HOME");
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
///  Says whether the commit service of a worktree answers on its socket.
/// Why:
///   A diagnostic only: there is nothing to install or start by hand. Connecting is what a commit does, so a service that the
///  manager would start on demand is started by this check too. Outside a repository there is no worktree to ask for, and the
///  answer counts the instances that answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// serviceStatus(runtimeDir, gitDir)
/// ```
pub fn service_status(runtime_dir: Option<&Path>, git_dir: Option<&Path>) -> StatusReport {
    let answers = |socket: &Path| -> bool { return std::os::unix::net::UnixStream::connect(socket).is_ok(); };
    let (answered, what): (bool, &str) = match (runtime_dir, git_dir) {
        (Some(runtime), Some(git)) => (answers(socket_for(runtime, git).as_path()), "The commit service of this worktree"),
        (Some(runtime), None) => {
            let any: bool = std::fs::read_dir(runtime.join(SOCKET_DIRECTORY)).is_ok_and(|entries| {
                return entries.flatten().any(|entry| { return entry.path().extension().is_some_and(|extension| { return extension == "sock"; }) && answers(entry.path().as_path()); });
            });
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
