//! Controls of the modes the socket folder and the socket file carry.

use super::*;
use std::os::unix::fs::PermissionsExt;

fn folder(name: &str) -> std::path::PathBuf {
    let root: std::path::PathBuf = crate::test_support::fixture_root().join(format!("socket-modes-{name}"));
    std::fs::create_dir_all(&root).expect("fixture folder");
    return root;
}

/// The folder is made with 0700 and repaired when it is not.
#[test]
fn the_folder_is_enterable_by_this_account_alone() {
    let root = folder("folder");
    let socket_folder = path(root.as_path());
    std::fs::create_dir_all(socket_folder.as_path()).expect("the folder");
    std::fs::set_permissions(socket_folder.as_path(), std::fs::Permissions::from_mode(0o755)).expect("a wide mode");
    ensure_private(socket_folder.as_path()).expect("the folder is made private");
    let mode = std::fs::metadata(socket_folder.as_path()).expect("the folder is there").permissions().mode() & 0o777;
    assert_eq!(mode, 0o700, "only this account may enter the folder the sockets are in");
}

/// The socket file is 0600, whichever umask made it.
#[test]
fn the_socket_is_openable_by_this_account_alone() {
    let root = folder("socket");
    let listener = std::os::unix::net::UnixListener::bind(root.join("a.sock")).expect("a socket");
    std::fs::set_permissions(root.join("a.sock"), std::fs::Permissions::from_mode(0o755)).expect("a wide mode");
    keep_only_its_owner(root.join("a.sock").as_path()).expect("the socket is made private");
    let mode = std::fs::metadata(root.join("a.sock")).expect("the socket is there").permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "only this account may open the address");
    drop(listener);
}
