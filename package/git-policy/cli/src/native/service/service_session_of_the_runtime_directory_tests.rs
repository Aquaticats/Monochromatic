//! Controls of the boundary issue 6293 decided:
//! whether the runtime directory a command uses is the session its inherited bus belongs to,
//! and what that answer does to where the service's state lives.

use super::*;
use std::path::PathBuf;
use crate::service_host_of_a_worktree::state_dir_of_the_command;

/// What:
///  The bus belongs to the runtime directory only when its socket's directory is that directory.
/// Why:
///  `systemd-run --user` finds the manager by this bus,
/// so a bus elsewhere is another session's manager,
/// and asking it is
///  what issue 6293 forbids.
#[test]
fn the_bus_is_ours_only_when_its_socket_sits_in_our_runtime_directory() {
    let ours: &Path = Path::new("/run/user/1000");
    assert!(bus_is_ours(ours, Some("unix:path=/run/user/1000/bus")));
    assert!(!bus_is_ours(ours, Some("unix:path=/run/user/1001/bus")), "a bus of another account");
    assert!(!bus_is_ours(ours, Some("unix:path=/run/user/1000/x/bus")), "a bus below our directory is not our session's socket");
    assert!(!bus_is_ours(ours, Some("unix:abstract=/tmp/dbus-abc")), "an abstract address has no directory to check");
    assert!(!bus_is_ours(ours, None), "no address names no bus");
    // A list is read in order, and the first path decides.
    assert!(bus_is_ours(ours, Some("unix:abstract=/tmp/x;unix:path=/run/user/1000/bus")));
}

/// What:
///  A command belongs to the session only when the environment named the runtime directory its bus lives in.
/// Why:
///  The same answer is what lets a manager be asked and what keeps the service's state under the account's state directory,
///  so a run given a runtime directory of its own answers no to both (issue 6293).
#[test]
fn a_command_belongs_to_the_session_only_when_its_own_bus_lives_in_the_runtime_directory_it_named() {
    let ours: &Path = Path::new("/run/user/1000");
    assert!(is_the_session_of(Some(ours), ours, Some("unix:path=/run/user/1000/bus")), "the session's own bus, named by it");
    assert!(
        !is_the_session_of(Some(ours), ours, Some("unix:path=/run/user/1001/bus")),
        "a run given a runtime directory of its own while the account's bus stays where it was"
    );
    assert!(!is_the_session_of(Some(ours), ours, None), "no inherited bus names no session");
    assert!(!is_the_session_of(None, ours, Some("unix:path=/run/user/1000/bus")), "a folder this program fell back to is no session");
    let fallback: &Path = Path::new("/home/a/.run");
    assert!(!is_the_session_of(Some(fallback), fallback, Some("unix:path=/run/user/1000/bus")), "a folder of the program's own is no session");
    assert!(!is_the_session_of(Some(ours), fallback, Some("unix:path=/run/user/1000/bus")), "a named directory that is not the one used names no session");
}

/// What:
///  The account's state directory serves the session's own run,
/// and the runtime directory serves every other one.
/// Why:
///  A run given a runtime directory of its own must leave nothing under the account's real state directory,
/// and a session
///  whose own bus lives in its runtime directory keeps exactly the paths earlier releases used.
#[test]
fn the_state_directory_is_the_accounts_only_for_the_sessions_own_run() {
    let ours: &Path = Path::new("/run/user/1000");
    let account: &Path = Path::new("/home/a/.local/state");
    assert_eq!(
        state_dir_of_the_command(Some(ours), ours, Some("unix:path=/run/user/1000/bus"), None, Some(Path::new("/home/a"))),
        Some(account.to_path_buf()),
        "the session's own run keeps the account's state directory"
    );
    assert_eq!(
        state_dir_of_the_command(Some(ours), ours, Some("unix:path=/run/user/1001/bus"), None, Some(Path::new("/home/a"))),
        Some(ours.to_path_buf()),
        "a runtime directory of the run's own holds its state, whatever the account's state directory is"
    );
    assert_eq!(
        state_dir_of_the_command(Some(ours), ours, None, Some(Path::new("/named")), Some(Path::new("/home/a"))),
        Some(ours.to_path_buf()),
        "a run that inherited no bus but named its runtime directory keeps its state there"
    );
    assert_eq!(
        state_dir_of_the_command(None, Path::new("/home/a/.run"), None, Some(Path::new("/named")), None),
        Some(PathBuf::from("/named")),
        "a folder this program fell back to names no session, so the account's state directory is used"
    );
    assert_eq!(
        state_dir_of_the_command(Some(ours), ours, None, None, None),
        Some(ours.to_path_buf()),
        "a private run needs no home to know where its state goes"
    );
    assert_eq!(
        state_dir_of_the_command(Some(ours), ours, Some("unix:path=/run/user/1000/bus"), None, None),
        None,
        "the session's own run with neither XDG_STATE_HOME nor HOME names no state directory at all"
    );
}
