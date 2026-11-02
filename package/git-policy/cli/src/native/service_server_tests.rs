//! Controls for how the service opens its listening socket.

use super::*;
use std::os::unix::net::UnixStream;

/// A directory of this test's own, short enough for a socket path: a socket path holds about a hundred bytes, which a fixture
///  root the person points the tests at would not leave.
fn short_directory(name: &str) -> std::path::PathBuf {
    let home: std::path::PathBuf = std::path::PathBuf::from(std::env::var_os("HOME").expect("a home directory for the short directory"));
    let directory: std::path::PathBuf = home.join("temp/agent").join(format!("ss-{}-{name}", std::process::id()));
    let _stale = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("short directory");
    return directory;
}

/// Removes a directory made by `short_directory`.
fn remove_short(directory: &std::path::Path) {
    let _removed = std::fs::remove_dir_all(directory);
}

/// A socket file nobody listens on any more, as a service that was killed leaves it.
fn stale_socket(directory: &std::path::Path) -> std::path::PathBuf {
    let path: std::path::PathBuf = directory.join("service.sock");
    let listener: UnixListener = UnixListener::bind(&path).expect("first bind");
    drop(listener);
    assert!(path.exists(), "a dropped listener leaves its socket file behind");
    return path;
}

/// A service run bound to a path takes over a socket file that no service listens on.
#[test]
fn a_path_whose_socket_file_is_stale_is_bound_again() {
    let root = short_directory("stale");
    let path = stale_socket(root.as_path());
    let listener: UnixListener = bind(&Bind::Path(path.clone())).expect("a stale socket file does not stop the bind");
    assert!(UnixStream::connect(&path).is_ok(), "the new listener answers");
    drop(listener);
    remove_short(root.as_path());
}

/// A socket some service still listens on is never taken over: the second bind fails and the first keeps answering.
#[test]
fn a_path_whose_socket_a_service_listens_on_is_not_taken_over() {
    let root = short_directory("live");
    let path = root.join("service.sock");
    let live: UnixListener = bind(&Bind::Path(path.clone())).expect("first bind");
    assert!(bind(&Bind::Path(path.clone())).is_err(), "a live socket stays with its service");
    assert!(UnixStream::connect(&path).is_ok(), "the first service still answers");
    drop(live);
    remove_short(root.as_path());
}

/// A path that is not a socket (a regular file the person owns) is never removed.
#[test]
fn a_path_that_holds_a_regular_file_is_left_alone() {
    let root = short_directory("file");
    let path = root.join("service.sock");
    std::fs::write(&path, b"mine").expect("write");
    assert!(bind(&Bind::Path(path.clone())).is_err(), "a regular file is not a socket to replace");
    assert_eq!(std::fs::read(&path).expect("read"), b"mine", "the file is untouched");
    remove_short(root.as_path());
}
