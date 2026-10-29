//! Unit tests for the owner-only control socket.

use std::{
    os::unix::{
        fs::{FileTypeExt, PermissionsExt},
        net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
    process::Command,
};

use super::{bind_listener, SOCKET_MODE};

/// Environment variable that makes the child test in this file do its work.
const CHILD_FLAG: &str = "NWS_CONTROL_SOCKET_UMASK_CHILD";

/// Longest path a Unix socket address holds,
///  without the terminating zero byte.
const SOCKET_PATH_LIMIT: usize = 107;

/// A disposable directory removed at the end of the test.
struct Scratch {
    /// Directory that holds everything a test creates.
    directory: PathBuf,
}

impl Scratch {
    /// Create an empty directory named after the test.
    fn new(name: &str) -> Scratch {
        let directory = std::env::temp_dir().join(format!("nws-socket-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("creating the scratch directory");
        return Scratch { directory }
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

/// The permission bits of a path,
///  without the file type bits.
fn mode_of(path: &Path) -> u32 {
    return std::fs::metadata(path).expect("reading the metadata of the path").permissions().mode() & 0o777
}

/// A new socket is a socket file only its owner can open.
#[test]
fn the_bound_socket_is_owner_only() {
    let scratch = Scratch::new("owner-only");
    let path = scratch.directory.join("control.sock");
    let _listener = bind_listener(&path).expect("binding the control socket");
    let file_type = std::fs::metadata(&path).expect("reading the socket metadata").file_type();
    assert!(file_type.is_socket(), "{file_type:?}");
    assert_eq!(mode_of(&path), SOCKET_MODE, "mode is {:o}", mode_of(&path));
}

/// The child half of the umask test:
///  runs only when the parent test started this binary under `umask 000`.
#[test]
fn umask_child_binds_under_a_permissive_umask() {
    if std::env::var_os(CHILD_FLAG).is_none() {
        return;
    }
    let scratch = Scratch::new("umask-child");

    // Positive control: a plain bind under this umask is open to everyone,
    // so an owner-only result below comes from bind_listener and not from the environment.
    let plain = scratch.directory.join("plain.sock");
    let _plain_listener = UnixListener::bind(&plain).expect("binding a plain socket");
    assert_eq!(mode_of(&plain), 0o777, "umask 000 must leave a plain bind open; got {:o}", mode_of(&plain));

    let path = scratch.directory.join("control.sock");
    let _listener = bind_listener(&path).expect("binding the control socket under umask 000");
    assert_eq!(mode_of(&path), SOCKET_MODE, "mode is {:o}", mode_of(&path));
}

/// A looser umask does not widen the socket:
///  the same bind run under `umask 000` still yields mode 0600.
#[test]
fn a_looser_umask_does_not_widen_the_socket() {
    let executable = std::env::current_exe().expect("locating this test binary");
    // libtest names a test by its module path without the crate name.
    let (_crate_name, module) = module_path!().split_once("::").expect("a module path below the crate name");
    let output = Command::new("sh")
        .args(["-c", "umask 000 && exec \"$0\" \"$@\""])
        .arg(&executable)
        .arg(format!("{module}::umask_child_binds_under_a_permissive_umask"))
        .args(["--exact", "--nocapture"])
        .env(CHILD_FLAG, "1")
        .output()
        .expect("starting the umask child through sh");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "child failed\nstdout:\n{stdout}\nstderr:\n{stderr}");
    // The child must have run its one test, not matched nothing.
    assert!(stdout.contains("1 passed"), "child ran no test\nstdout:\n{stdout}");
}

/// A leftover file at the path is replaced,
///  and the new socket accepts a connection from its owner.
#[test]
fn a_stale_file_is_replaced_and_the_socket_accepts_connections() {
    let scratch = Scratch::new("stale");
    let path = scratch.directory.join("control.sock");
    std::fs::write(&path, b"left over").expect("writing the stale file");
    let listener = bind_listener(&path).expect("binding over a stale file");
    let _client = UnixStream::connect(&path).expect("connecting to the renamed socket");
    let (_server, _address) = listener.accept().expect("accepting the connection");
}

/// Nothing is left beside the socket:
///  the staging directory is gone after the bind.
#[test]
fn no_staging_directory_remains() {
    let scratch = Scratch::new("no-staging");
    let path = scratch.directory.join("control.sock");
    let _listener = bind_listener(&path).expect("binding the control socket");
    let names: Vec<_> = std::fs::read_dir(&scratch.directory)
        .expect("listing the scratch directory")
        .map(|entry| return entry.expect("reading an entry").file_name())
        .collect();
    assert_eq!(names, vec![std::ffi::OsString::from("control.sock")]);
}

/// A path that just fits a socket address binds,
///  though the staging path beside it would not fit.
#[test]
fn a_path_at_the_address_limit_still_binds() {
    let scratch = Scratch::new("long");
    let prefix_length = scratch.directory.as_os_str().len() + 1;
    let name = "s".repeat(SOCKET_PATH_LIMIT - prefix_length);
    let path = scratch.directory.join(name);
    assert_eq!(path.as_os_str().len(), SOCKET_PATH_LIMIT);
    let listener = bind_listener(&path).expect("binding a path at the address limit");
    assert_eq!(mode_of(&path), SOCKET_MODE);
    let _client = UnixStream::connect(&path).expect("connecting to the long path");
    let (_server, _address) = listener.accept().expect("accepting on the long path");
}

/// A path whose directory is missing fails with a message that names the path.
#[test]
fn a_missing_directory_is_an_error_naming_the_path() {
    let scratch = Scratch::new("missing");
    let path = scratch.directory.join("absent").join("control.sock");
    let error = bind_listener(&path).expect_err("binding below a missing directory");
    let message = format!("{error:#}");
    assert!(message.contains(&path.display().to_string()), "{message}");
    assert_eq!(std::fs::read_dir(&scratch.directory).expect("listing the scratch directory").count(), 0);
}
