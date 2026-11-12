//! Controls of the paths one command derives for a worktree's service.

use super::*;

/// The address sits in the socket folder and the journal in the state folder,
/// which is where the unit's `StateDirectory=` used to put it.
#[test]
fn the_paths_are_the_ones_the_unit_used_to_name() {
    let host = ServiceHost {
        runtime_dir: PathBuf::from("/run/user/1000"),
        state_dir: PathBuf::from("/home/a/.local/state"),
        service_binary: PathBuf::from("/bin/git"),
        git: PathBuf::from("/usr/bin/git"),
        instance: String::from("3f9a"),
        git_dir: PathBuf::from("/repo/.git"),
    };
    assert_eq!(host.socket(), PathBuf::from(format!("/run/user/1000/{SOCKET_DIRECTORY}/3f9a.sock")));
    assert_eq!(host.journal(), PathBuf::from(format!("/home/a/.local/state/{}/service/3f9a", SERVICE_UNIT.current)));
    assert_eq!(host.server().instance, "3f9a");
    assert_eq!(host.server().program, PathBuf::from("/bin/git"));
}

/// The state folder is the environment's, and a home without one names `.local/state`.
#[test]
fn the_state_folder_comes_from_the_environment() {
    assert_eq!(state_dir_of(Some(Path::new("/s")), Some(Path::new("/h"))), Some(PathBuf::from("/s")));
    assert_eq!(state_dir_of(None, Some(Path::new("/h"))), Some(PathBuf::from("/h/.local/state")));
    assert_eq!(state_dir_of(None, None), None);
}
