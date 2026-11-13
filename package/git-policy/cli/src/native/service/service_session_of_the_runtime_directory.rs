//! What:
//!  Whether the runtime directory a command uses is the session its inherited bus belongs to.
//! Why:
//!  One boundary decides two things, and they must never disagree
//! (issue 6293; the owner's decision of 2026-11-13, "Private runtime dir is the boundary"):
//!  a service manager is asked for a scope only for the session whose bus lives in the runtime directory,
//!  and the commit service keeps its journal and its retry plans under the account's state directory only for that
//!  session.
//! A runtime directory of a command's own is a boundary of its own:
//! the wrapper does not use a bus that lives elsewhere,
//! so nothing of such a run reaches the account's manager,
//! and nothing of it is written under the account's real state directory
//! (`service_host_of_a_worktree::state_dir_of_the_command`).
//! The bus's address is the fact:
//! `/run/user/1000/bus` is the bus of the session whose runtime directory is `/run/user/1000`,
//! and a folder this program fell back to (`~/.run`, for an account that named none) is no session at all.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! isTheSessionOf(env.XDG_RUNTIME_DIR && pathOf(env.XDG_RUNTIME_DIR), runtimeDir, env.DBUS_SESSION_BUS_ADDRESS)
//! ```

use std::path::Path;

/// What:
///  Whether the session bus this command inherited is the one inside `runtime_dir`.
/// Why:
///   The address is the fact,
/// and its socket's directory is the runtime directory its session uses:
/// `/run/user/1000/bus` is the bus of the session whose runtime directory is `/run/user/1000`.
/// An abstract address has no directory, so it cannot be proven ours;
/// a list of addresses is read in order and the first that names a path decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// busAddresses(env.DBUS_SESSION_BUS_ADDRESS).some((a) => path.dirname(a.path) === runtimeDir)
/// ```
pub fn bus_is_ours(runtime_dir: &Path, address: Option<&str>) -> bool {
    let Some(spelled) = address else { return false };
    for one in spelled.split(';') {
        let Some(path) = one.strip_prefix("unix:path=") else { continue };
        let socket: &Path = Path::new(path);
        let Some(directory) = socket.parent() else { continue };
        return directory == runtime_dir;
    }
    return false;
}

/// What:
///  Whether the runtime directory the environment named is the session its inherited bus belongs to.
/// Why:
///   A manager is asked only for such a session, and only such a session keeps its service state under the account's
///  state directory:
///   `systemd-run --user` finds its manager by the session bus, so a bus of another directory is another session's
///  manager,
/// and asking it would put this run's service in the account's manager and its cgroup, which is what a private runtime
///  directory must never cause (issue 6293).
///   A folder the wrapper fell back to names no session, and a runtime directory whose bus lives elsewhere is a
///  directory of the command's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isTheSessionOf(named, runtimeDir, bus) // named === runtimeDir && busIsOurs(runtimeDir, bus)
/// ```
pub fn is_the_session_of(named_runtime_directory: Option<&Path>, runtime_dir: &Path, bus_address: Option<&str>) -> bool {
    let Some(named) = named_runtime_directory else { return false };
    return named == runtime_dir && bus_is_ours(runtime_dir, bus_address);
}

/// Controls of the boundary stay out of the release executable.
#[cfg(test)]
#[path = "service_session_of_the_runtime_directory_tests.rs"]
mod tests;
