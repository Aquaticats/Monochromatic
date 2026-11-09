//! What:
//! The unit tests of the standard-input classification.
//! Why:
//! The lost-input rule turns on a pipe exactly,
//! so every kind the file system can report needs a case:
//! a regular file,
//! a pipe,
//! a socket,
//! and the things that are none of them (a directory and `/dev/null`).

use super::StdinKind;
use crate::test_scratch::scratch;
use std::fs;
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::Command;

/// What:
/// `fn file_type_of(path: &Path) -> std::fs::FileType` declares a helper that asks the file system what a path is,
/// the same way the shadow asks about `/proc/self/fd/0`.
/// Why:
/// Every case below differs in the file type and in nothing else.
fn file_type_of(path: &Path) -> fs::FileType {
    return fs::metadata(path).expect("stat the fixture").file_type();
}

/// What:
/// `fn make_pipe(path: &Path)` declares a helper that creates a named pipe (FIFO) at `path`.
/// Why:
/// A pipe is the one kind that cannot be made from the standard library without `unsafe`,
/// and this crate keeps its single `unsafe` call in `priority.rs`.
/// `mkfifo` is coreutils,
/// present on every host this crate supports.
fn make_pipe(path: &Path) {
    let status = Command::new("mkfifo").arg(path).status().expect("run mkfifo");
    assert!(status.success(), "mkfifo could not create {}", path.display());
}

/// A pipe on standard input is a FIFO,
/// and only a FIFO is a pipe.
#[test]
fn a_named_pipe_is_a_pipe() {
    let directory = scratch();
    let pipe: PathBuf = directory.path().join("pipe");
    make_pipe(&pipe);
    assert_eq!(StdinKind::of(&file_type_of(&pipe)), StdinKind::Pipe);
}

/// A redirected file reads as a file,
/// never as a pipe:
/// `rg PATTERN < file` is deliberate.
#[test]
fn a_regular_file_is_a_file_and_not_a_pipe() {
    let directory = scratch();
    let file: PathBuf = directory.path().join("input.txt");
    fs::write(&file, "line\n").expect("write the fixture");
    let kind = StdinKind::of(&file_type_of(&file));
    assert_eq!(kind, StdinKind::File);
    assert!(!kind.is_pipe());
}

/// A socket reads as a socket,
/// never as a pipe.
#[test]
fn a_socket_is_a_socket_and_not_a_pipe() {
    let directory = scratch();
    // A socket address is limited to a short path, and this repository's worktree paths are longer (measured 2026-11-08:
    // `bind` refused the real path with "path must be shorter than SUN_LEN"). Opening the directory and binding through
    // `/proc/self/fd/<descriptor>` keeps the address short while the socket itself is still created in the scratch directory.
    let directory_handle = fs::File::open(directory.path()).expect("open the scratch directory");
    let address: PathBuf = PathBuf::from(format!("/proc/self/fd/{}", directory_handle.as_raw_fd())).join("socket");
    let listener = UnixListener::bind(&address).expect("bind the fixture socket");
    let kind = StdinKind::of(&file_type_of(&directory.path().join("socket")));
    assert_eq!(kind, StdinKind::Socket);
    assert!(!kind.is_pipe());
    drop(listener);
}

/// `/dev/null` is a character device,
/// which is what an unpiped command sees,
/// so it is no input at all.
#[test]
fn the_null_device_is_no_input() {
    let kind = StdinKind::of(&file_type_of(Path::new("/dev/null")));
    assert_eq!(kind, StdinKind::None);
    assert!(!kind.reads_input());
    assert!(!kind.is_pipe());
}

/// A directory is none of the four,
/// and must not be mistaken for a pipe.
#[test]
fn a_directory_is_no_input() {
    let directory = scratch();
    assert_eq!(StdinKind::of(&file_type_of(directory.path())), StdinKind::None);
}

/// Every kind that gives a tool something to read reports so,
/// and only `None` does not.
#[test]
fn only_none_leaves_a_tool_nothing_to_read() {
    assert!(StdinKind::File.reads_input());
    assert!(StdinKind::Pipe.reads_input());
    assert!(StdinKind::Socket.reads_input());
    assert!(!StdinKind::None.reads_input());
    // `is_pipe` is the stricter question the lost-input rule asks.
    assert!(StdinKind::Pipe.is_pipe());
    assert!(!StdinKind::File.is_pipe());
    assert!(!StdinKind::Socket.is_pipe());
    assert!(!StdinKind::None.is_pipe());
}
