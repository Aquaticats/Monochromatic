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
    if !matches!(same_place(named, runtime_dir), Some(true)) {
        return Session::Unproven;
    }
    let Some(spelled) = bus_address else { return Session::Unproven };
    for one in spelled.split(';') {
        if one == IN_THE_RUNTIME_DIRECTORY {
            return Session::ThisSession;
        }
        let Some(path) = one.strip_prefix(PATH_ENTRY) else { continue };
        let Some(directory) = Path::new(path).parent() else { return Session::Unproven };
        return match same_place(directory, runtime_dir) {
            Some(true) => Session::ThisSession,
            Some(false) => Session::AnotherSession,
            // No answer in time is no proof: "not proven" never moves a real session's records.
            None => Session::Unproven,
        };
    }
    return Session::Unproven;
}

/// What:
///  Whether two spellings name one place:
///  the same text,
///  or two paths the file system resolves to one.
/// Why:
///  The runtime directory is the boundary this release draws,
/// and a boundary must not turn on how a path is spelled:
/// a bus address written with a `..` part or through a link named another session,
/// so a real session's journals, scratch and retry plans were moved into a directory a logout clears
/// (the sixth review of issue 6293, finding 5).
/// A path the file system refuses to resolve is another place:
/// a bus socket that is not there is not in this runtime directory,
/// and the specification calls a bus socket outside it another session's.
/// A resolution that gave no answer **in time** is not proof of either,
/// and only that case answers "not proven",
/// which keeps the account's state directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// samePlace(a, b) // a === b || (realpath(a) === realpath(b))
/// ```
pub(crate) fn same_place(left: &Path, right: &Path) -> Option<bool> {
    if left == right {
        return Some(true);
    }
    return same_place_within(left, right, PLACE_LOOK_CEILING);
}

/// What:
///  Whether two spellings name one place, resolved within `ceiling`,
///  or nothing when the file system gave no answer in time.
/// Why:
///   A path resolution waits in the kernel, and a mount that stopped answering waits with no end;
/// the owner's rule is that nothing waits forever, and no blocking call may hold a worker
/// (`work_awaited_within_a_ceiling.rs`, `doc/planning/git-policy-cli-full-tokio.md`).
/// A resolve that gave no answer is not evidence that the two paths differ,
/// so the caller gets nothing and answers "not proven", which keeps the account's state directory:
/// the safe side, because the other side moves durable records into storage a logout clears
/// (the sixth review of issue 6293, finding 5).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// samePlaceWithin(a, b, ceiling) // true | false | undefined within the ceiling
/// ```
fn same_place_within(left: &Path, right: &Path, ceiling: std::time::Duration) -> Option<bool> {
    let here: std::path::PathBuf = left.to_path_buf();
    let there: std::path::PathBuf = right.to_path_buf();
    let resolved = crate::work_awaited_within_a_ceiling::work_within_reporting(ceiling, move || {
        run_the_controls_step_before_the_resolution();
        return (std::fs::canonicalize(here), std::fs::canonicalize(there));
    });
    return match resolved {
        crate::work_awaited_within_a_ceiling::TheAnswerOfAThreadOfWork::Ran((Ok(at_the_first), Ok(at_the_second))) => Some(at_the_first == at_the_second),
        // What: A path the file system refuses to resolve is another place.
        // Why:  A bus socket whose folder cannot be resolved (a link that loops, a folder that is gone) is not in this
        // runtime directory: the specification calls a bus socket outside it another session's, and a socket that is not
        // there is the plainest case of that. A resolution that gave no answer **in time** is the other case and is handled
        // above: it is no proof either way, and "not proven" keeps the account's state directory
        // (the eighth review of issue 6293, findings 5 and 6).
        crate::work_awaited_within_a_ceiling::TheAnswerOfAThreadOfWork::Ran(_) => Some(false),
        crate::work_awaited_within_a_ceiling::TheAnswerOfAThreadOfWork::NoThreadToStart
        | crate::work_awaited_within_a_ceiling::TheAnswerOfAThreadOfWork::NoAnswer => None,
    };
}

// What:
//  A control's step before the two paths are resolved, so a resolution that outlasts the ceiling is produced without a stalled
//  mount.
// Why:
//   The rule this change answers is about a resolution that gives no answer in time, and no control on this host can stall a
//   file system call.
// The resolution runs on the thread of the bounded job, so the control's step is shared by the threads of this process.
#[cfg(test)]
static BEFORE_THE_RESOLUTION: std::sync::Mutex<Option<std::time::Duration>> = std::sync::Mutex::new(None);

/// What:
///  Makes the next resolution take `slowly`, so a control can produce one that outlasts its ceiling.
/// Why:
///   It is the one way to hold the file-system call the bound exists for,
/// and it runs on the thread of the bounded job, which is left behind exactly as a stuck call is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// aControlTakesTheResolutionSlowly(2000)
/// ```
#[cfg(test)]
pub(crate) fn a_control_takes_the_resolution_slowly(how_long: std::time::Duration) {
    let mut slot = match BEFORE_THE_RESOLUTION.lock() {
        Ok(slot) => slot,
        Err(poisoned) => poisoned.into_inner(),
    };
    *slot = Some(how_long);
}

/// Runs a control's slow resolution once, if one was set.
fn run_the_controls_step_before_the_resolution() {
    #[cfg(test)]
    {
        let how_long: Option<std::time::Duration> = match BEFORE_THE_RESOLUTION.lock() {
            Ok(mut slot) => slot.take(),
            Err(poisoned) => poisoned.into_inner().take(),
        };
        if let Some(pause) = how_long {
            std::thread::sleep(pause);
        }
    }
}

/// The longest the session's own boundary waits for two spellings of a path to be resolved.
/// Why:
///   It is a look at the file system on a command's own path,
/// and it buys the answer this boundary needs:
/// a quarter of a second is `WARM_REQUEST_CONNECT_CEILING`, the ceiling the package gives a look made before a request,
/// and a resolve that outlasts it is one the file system is not answering.
pub const PLACE_LOOK_CEILING: std::time::Duration = crate::wait_ceilings::WARM_REQUEST_CONNECT_CEILING;

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
