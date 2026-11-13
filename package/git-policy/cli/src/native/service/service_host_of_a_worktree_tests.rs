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

/// What:
///  A named runtime folder is used as it is,
///  and an account that names none gets a folder of its own under its home.
/// Why:
///   A container and a service's own child set no `XDG_RUNTIME_DIR`, and a commit must land there.
#[test]
fn a_command_with_no_runtime_directory_gets_one_under_its_home() {
    assert_eq!(runtime_directory(Some(Path::new("/run/user/1000")), Some(Path::new("/home/a"))), Some(PathBuf::from("/run/user/1000")));
    assert_eq!(runtime_directory(None, Some(Path::new("/home/a"))), Some(PathBuf::from("/home/a/.run")));
    assert_eq!(runtime_directory(Some(Path::new("")), Some(Path::new("/home/a"))), Some(PathBuf::from("/home/a/.run")), "an empty value names nothing");
    // Neither a named folder nor a home: nowhere of its own, said plainly rather than guessed.
    assert_eq!(runtime_directory(None, None), None, "an account with nowhere of its own names no folder");
}

/// The socket of the fallback fits one socket address,
/// which is what lets a commit land in a container.
#[test]
fn the_fallback_folder_leaves_room_for_a_socket_name() {
    let folder: PathBuf = runtime_directory(None, Some(Path::new("/home/a"))).expect("a folder");
    let instance: String = "0123456789abcdef0123456789abcdef".to_string();
    let socket: PathBuf = socket_path(folder.as_path(), instance.as_str());
    assert!(socket.as_os_str().len() < crate::service_instance::SOCKET_ADDRESS_BYTES, "{}", socket.display());
}
