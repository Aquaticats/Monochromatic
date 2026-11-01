//! What:
//!  Which way a commit goes: through the resident service (already listening, or set up first by the wrapper), or the
//!  way it went before the service existed.
//! Why:
//!  There is no switch and no install state for the person to manage (`doc/planning/cli-git-resident-service.md`,
//!  "Provisioning"). The route follows the machine: a socket that is there means a service to use; a user session bus and
//!  a service program next to the wrapper mean the wrapper can set the service up itself; anything else keeps the earlier
//!  path until that path is retired, after which it is a plain failure. The decision reads the file system only and starts
//!  no process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const route = chooseRoute({ runtimeDir, busAddress, configHome, homeDir, ownExecutable, realGit });
//! ```

use super::service_provision::{ProvisionHost, SERVICE_PROGRAM};
use super::service_server::Bind;
use super::systemd_dbus::session_bus_path;
use std::path::{Path, PathBuf};

/// The socket's place under the user's runtime directory.
pub const SOCKET_RELATIVE: &str = "cli-git/service.sock";

static OWN_EXECUTABLE: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/// Remembers where the running wrapper is, once, so the service program can be found beside it.
///  Identification only: nothing is started from this path (`doc/planning/cli-git-no-self-invocation.md`); the path is
///  read to name a sibling program in a unit file that the operating system's manager runs.
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
}

/// Which way a commit goes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Route {
    /// The earlier path.
    Earlier,
    /// Through the service at `address`, after provisioning when `provision` is present.
    Service {
        /// Where the service listens.
        address: Bind,
        /// What to set up first; absent when the socket already exists.
        provision: Option<ProvisionHost>,
    },
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
///   A socket that exists is used as it is, whoever made it; otherwise provisioning needs the session bus socket and the
///  service program, and without both the earlier path stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// chooseRoute(inputs)
/// ```
pub fn choose_route(inputs: &RouteInputs<'_>) -> Route {
    let Some(runtime_dir) = inputs.runtime_dir else { return Route::Earlier };
    let socket: PathBuf = runtime_dir.join(SOCKET_RELATIVE);
    if socket.exists() {
        return Route::Service { address: Bind::Path(socket), provision: None };
    }
    let Some(bus) = session_bus_path(inputs.bus_address, Some(runtime_dir)).filter(|path| { return path.exists(); }) else { return Route::Earlier };
    let Some(service_binary) = inputs
        .own_executable
        .and_then(Path::parent)
        .map(|directory| { return directory.join(SERVICE_PROGRAM); })
        .filter(|path| { return path.is_file(); })
    else {
        return Route::Earlier;
    };
    let host: ProvisionHost = ProvisionHost {
        runtime_dir: runtime_dir.to_path_buf(),
        user_unit_dir: user_unit_dir(inputs.config_home, inputs.home),
        bus,
        service_binary,
        git: inputs.real_git.to_path_buf(),
    };
    return Route::Service { address: Bind::Path(socket), provision: Some(host) };
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
///  Says whether the commit service answers on its socket.
/// Why:
///   A diagnostic only: there is nothing to install or start by hand. Connecting is what a commit does, so a service
///  that the manager would start on demand is started by this check too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// serviceStatus(runtimeDir)
/// ```
pub fn service_status(runtime_dir: Option<&Path>) -> StatusReport {
    let answered: bool = runtime_dir
        .map(|directory| { return directory.join(SOCKET_RELATIVE); })
        .is_some_and(|socket| { return std::os::unix::net::UnixStream::connect(socket).is_ok(); });
    return if answered {
        StatusReport { code: 0, stdout: String::from("The commit service is running and answers.\n"), stderr: String::new() }
    } else {
        StatusReport {
            code: 1,
            stdout: String::new(),
            stderr: String::from(
                "The commit service does not answer here. It sets itself up the first time a commit needs it, so there is nothing to \
                 install; if a commit fails, its message says what to do.\n",
            ),
        }
    };
}

/// Route controls stay out of the release executable.
#[cfg(test)]
#[path = "service_route_tests.rs"]
mod tests;
