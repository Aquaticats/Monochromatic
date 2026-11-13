//! Controls of the boundary issue 6293 decided:
//! which session the runtime directory a command uses belongs to,
//! and what that answer does to where the service's durable state lives.

use super::*;
use crate::service_host_of_a_worktree::state_dir_of_the_command;
use std::path::PathBuf;

/// What:
///  A bus socket inside the runtime directory proves this session,
///  a socket outside it proves another session's,
///  and everything else proves nothing.
/// Why:
///   The three answers decide two things each (who may be asked for a scope, and where durable records live),
/// so a proof must never widen:
/// an abstract address names no directory,
/// and reading it as "ours" asked a foreign manager,
/// while reading it as "another session's" moved a real session's retry plans into a directory a logout clears
/// (the review of issue 6293, finding 6).
#[test]
fn the_session_is_proven_only_by_a_bus_socket_the_address_names() {
    let ours: &Path = Path::new("/run/user/1000");
    assert_eq!(session_of(Some(ours), ours, Some("unix:path=/run/user/1000/bus")), Session::ThisSession, "the session's own bus");
    assert_eq!(
        session_of(Some(ours), ours, Some("unix:path=/run/user/1001/bus")),
        Session::AnotherSession,
        "a run given a runtime directory of its own while the account's bus stays where it was"
    );
    assert_eq!(
        session_of(Some(ours), ours, Some("unix:path=/run/user/1000/x/bus")),
        Session::AnotherSession,
        "a bus below our directory is not our session's socket"
    );
    assert_eq!(
        session_of(Some(ours), ours, Some("unix:abstract=/tmp/dbus-abc")),
        Session::Unproven,
        "an abstract address has no directory to prove either session"
    );
    assert_eq!(session_of(Some(ours), ours, Some("tcp:host=127.0.0.1,port=1")), Session::Unproven, "a bus that is no Unix socket proves nothing");
    assert_eq!(session_of(Some(ours), ours, None), Session::Unproven, "no address names no session");
    assert_eq!(session_of(Some(ours), ours, Some("")), Session::Unproven, "an empty address names no session");
    // A list is read in order, and the first entry that names a socket decides.
    assert_eq!(
        session_of(Some(ours), ours, Some("unix:abstract=/tmp/x;unix:path=/run/user/1000/bus")),
        Session::ThisSession,
        "an abstract entry proves nothing and the path after it decides"
    );
    assert_eq!(
        session_of(Some(ours), ours, Some("unix:abstract=/tmp/x;unix:path=/run/user/1001/bus")),
        Session::AnotherSession,
        "the path after an abstract entry names another session"
    );
    // The address that asks for the bus in the runtime directory itself is this session's.
    assert_eq!(session_of(Some(ours), ours, Some("unix:runtime=yes")), Session::ThisSession, "a bus asked for by runtime directory");
}

/// What:
///  A runtime directory the environment did not name, or named and did not use, proves nothing.
/// Why:
///  A folder this program fell back to (`~/.run`) is no session at all,
///  and only the directory in use can be compared with the bus's socket.
///  A directory the environment named itself is the run's own as soon as the bus in it is another session's,
///  even a persistent one under the home: a logout clears nothing there.
#[test]
fn a_runtime_directory_the_environment_did_not_name_proves_no_session() {
    let ours: &Path = Path::new("/run/user/1000");
    let fallback: &Path = Path::new("/home/a/.run");
    assert_eq!(session_of(None, ours, Some("unix:path=/run/user/1000/bus")), Session::Unproven, "a folder the program fell back to");
    assert_eq!(
        session_of(Some(fallback), fallback, Some("unix:path=/run/user/1000/bus")),
        Session::AnotherSession,
        "a directory the environment named itself is the run's own when the bus in it is another session's, \
         and nothing is lost there: that directory outlives a logout"
    );
    assert_eq!(
        session_of(Some(ours), fallback, Some("unix:path=/run/user/1000/bus")),
        Session::Unproven,
        "a named directory that is not the one in use proves nothing"
    );
    assert!(
        !is_the_session_of(Some(ours), ours, Some("unix:abstract=/tmp/dbus-abc")),
        "a manager is never asked for an unproven bus"
    );
    assert!(is_the_session_of(Some(ours), ours, Some("unix:path=/run/user/1000/bus")), "and it is asked for a proven one");
}

/// What:
///  The account's state directory serves a proven session and every unproven bus,
///  and only a bus proven another session's moves the service's durable records into the runtime directory.
/// Why:
///  A run given a runtime directory of its own must leave nothing under the account's real state directory,
/// a session whose own bus lives in its runtime directory keeps exactly the paths earlier releases used,
///  and an unproven bus must lose nothing:
/// its journal and retry plans stay where a logout does not clear them, and the earlier records there are read
///  (the review of issue 6293, finding 6).
#[test]
fn the_state_directory_moves_only_for_a_bus_proven_another_sessions() {
    let ours: &Path = Path::new("/run/user/1000");
    let account: &Path = Path::new("/home/a/.local/state");
    let named: &Path = Path::new("/named");
    let home: &Path = Path::new("/home/a");
    assert_eq!(
        state_dir_of_the_command(Some(ours), ours, Some("unix:path=/run/user/1000/bus"), None, Some(home)),
        Some(account.to_path_buf()),
        "the session's own run keeps the account's state directory"
    );
    assert_eq!(
        state_dir_of_the_command(Some(ours), ours, Some("unix:path=/run/user/1001/bus"), None, Some(home)),
        Some(ours.to_path_buf()),
        "a runtime directory of the run's own holds its state, whatever the account's state directory is"
    );
    assert_eq!(
        state_dir_of_the_command(Some(ours), ours, Some("unix:abstract=/tmp/dbus-repro-6293"), None, Some(home)),
        Some(account.to_path_buf()),
        "an abstract bus proves nothing, so a person's own session keeps the account's state directory"
    );
    assert_eq!(
        state_dir_of_the_command(Some(ours), ours, Some("unix:abstract=/tmp/dbus-repro-6293"), Some(named), None),
        Some(named.to_path_buf()),
        "an unproven bus follows XDG_STATE_HOME as a session's own run does"
    );
    assert_eq!(
        state_dir_of_the_command(Some(ours), ours, None, Some(named), Some(home)),
        Some(named.to_path_buf()),
        "a run that inherited no bus keeps its state where a logout does not clear it"
    );
    assert_eq!(
        state_dir_of_the_command(None, Path::new("/home/a/.run"), None, Some(named), None),
        Some(PathBuf::from("/named")),
        "a folder this program fell back to names no session, so the account's state directory is used"
    );
    assert_eq!(
        state_dir_of_the_command(Some(ours), ours, Some("unix:abstract=/tmp/dbus-abc"), None, None),
        None,
        "an unproven bus with neither XDG_STATE_HOME nor HOME names no state directory at all"
    );
}
