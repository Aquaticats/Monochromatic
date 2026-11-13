//! What:
//!  Which session the runtime directory a command uses belongs to,
//! as far as the environment proves it.
//! Why:
//!  The answer decides two things,
//! and they must never disagree
//! (issue 6293;
//! the owner's decision of 2026-11-13,
//! "Private runtime dir is the boundary"):
//!  a service manager is asked for a scope only for the session whose bus lives in the runtime directory,
//!  and the commit service keeps its durable records (its journal and the plans of the pushes it retries)
//!  under the account's state directory only for a session.
//! The bus's address is the proof:
//! `/run/user/1000/bus` is the bus of the session whose runtime directory is `/run/user/1000`,
//! so a bus outside the runtime directory is provably another session's,
//! and that is the boundary:
//! the wrapper does not use it,
//! it asks no manager for a scope,
//! and the run's records live in the runtime directory it was given.
//! A bus inside the runtime directory is provably this session's,
//! and a folder this program fell back to (`~/.run`,
//! for an account that named none) is no session at all.
//!
//! Everything else is [`Session::Unproven`]:
//! an abstract address (`unix:abstract=…`,
//! the shape `dbus-launch` and sessions without systemd-logind give),
//! a bus that is not a Unix socket,
//! and no address at all.
//! An abstract address names no directory,
//! so it can only be proven the session's own by asking the bus who owns the user manager's name on it,
//! and this wrapper speaks no bus of its own:
//! it reaches a manager only through the manager's own program
//! (`service_server_start_through_the_manager.rs`).
//! So nothing is proven,
//! and an unproven session loses nothing:
//! its durable records stay where a logout does not clear them,
//! under the account's state directory,
//! and earlier records there are the ones its service reads
//! (`service_host_of_a_worktree::state_dir_of_the_command`).
//! No manager is asked for an unproven session either,
//! because asking one would reach whichever manager owns that bus.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! sessionOf(env.XDG_RUNTIME_DIR && pathOf(env.XDG_RUNTIME_DIR), runtimeDir, env.DBUS_SESSION_BUS_ADDRESS)
//! // 'this-session' | 'another-session' | 'unproven'
//! ```

use std::path::Path;

/// Which session the runtime directory in use belongs to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Session = 'this-session' | 'another-session' | 'unproven';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Session {
    /// The inherited bus's socket lives in the runtime directory,
    /// so this is the session whose bus it is.
    ThisSession,
    /// The inherited bus's socket lives outside the runtime directory,
    /// so the runtime directory is this command's own and the bus is another session's.
    AnotherSession,
    /// Nothing proves either way:
    /// an abstract address,
    /// an address that is not a Unix socket,
    /// no address at all,
    /// or no runtime directory the environment named.
    Unproven,
}

/// What:
///  The session the runtime directory belongs to,
///  from the runtime directory in use and the bus address the environment gave.
/// Why:
///   The first entry of the address that names a Unix socket is the fact,
/// so its directory decides;
/// `unix:runtime=yes` names the bus in the runtime directory itself,
/// which is the same session's.
/// Everything else proves nothing,
/// and an unproven answer is never read as "another session's":
/// guessing that moved a real session's retry plans into a directory a logout clears.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sessionOf = (named, runtimeDir, bus) => {
///   if (named !== runtimeDir) return 'unproven';
///   if (bus === undefined) return 'unproven';
///   for (const one of bus.split(';')) {
///     if (one === 'unix:runtime=yes') return 'this-session';
///     if (!one.startsWith('unix:path=')) continue;
///     return path.dirname(one.slice('unix:path='.length)) === runtimeDir ? 'this-session' : 'another-session';
///   }
///   return 'unproven';
/// };
/// ```
pub fn session_of(named_runtime_directory: Option<&Path>, runtime_dir: &Path, bus_address: Option<&str>) -> Session {
    let Some(named) = named_runtime_directory else { return Session::Unproven };
    if named != runtime_dir {
        return Session::Unproven;
    }
    let Some(spelled) = bus_address else { return Session::Unproven };
    for one in spelled.split(';') {
        if one == IN_THE_RUNTIME_DIRECTORY {
            return Session::ThisSession;
        }
        let Some(path) = one.strip_prefix(PATH_ENTRY) else { continue };
        let Some(directory) = Path::new(path).parent() else { return Session::Unproven };
        return if directory == runtime_dir { Session::ThisSession } else { Session::AnotherSession };
    }
    return Session::Unproven;
}

/// What:
///  Whether the session bus this command inherited is the one inside `runtime_dir`.
/// Why:
///   The manager is asked only on this proof,
/// so the answer never widens to an address that was not shown to be this session's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isTheSessionOf(named, runtimeDir, bus) // sessionOf(...) === 'this-session'
/// ```
pub fn is_the_session_of(named_runtime_directory: Option<&Path>, runtime_dir: &Path, bus_address: Option<&str>) -> bool {
    return session_of(named_runtime_directory, runtime_dir, bus_address) == Session::ThisSession;
}

/// What:
///  Whether the runtime directory in use is provably one this command made rather than a session's own.
/// Why:
///   Only a proven other session's bus makes the runtime directory the command's own,
/// and only then may its durable records live somewhere a logout clears.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isAnotherSession(named, runtimeDir, bus) // sessionOf(...) === 'another-session'
/// ```
pub fn is_another_session(named_runtime_directory: Option<&Path>, runtime_dir: &Path, bus_address: Option<&str>) -> bool {
    return session_of(named_runtime_directory, runtime_dir, bus_address) == Session::AnotherSession;
}

/// The entry of a bus address that asks for the bus in the runtime directory itself.
const IN_THE_RUNTIME_DIRECTORY: &str = "unix:runtime=yes";

/// The prefix of a bus address entry that names a socket file.
const PATH_ENTRY: &str = "unix:path=";

/// Controls of the boundary stay out of the release executable.
#[cfg(test)]
#[path = "service_session_of_the_runtime_directory_tests.rs"]
mod tests;
