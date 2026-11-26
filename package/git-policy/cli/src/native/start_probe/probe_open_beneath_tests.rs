//! What:
//!  Controls of the control's own `openat2`:
//!  a plain name beneath a held folder opens,
//!  a `..` escape refuses `EXDEV`,
//!  and a symbolic link refuses `ELOOP`.
//! Why:
//!  The escape control asserts the kernel's `EXDEV`,
//!  which the wrapper's walker never reaches because it refuses `..` as a
//!  name first.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(openDirectoryBeneath(held, '../escape').errno).toBe(EXDEV);
//! ```

use super::{
    open_directory_beneath,
    open_path_handle,
};
use crate::testing::test_support::fixture;
use std::os::fd::AsRawFd;
use std::path::PathBuf;

/// A plain folder opens,
/// and its handle opens too.
#[test]
fn a_plain_folder_opens() {
    let root: PathBuf = fixture("start-probe-open-beneath");
    std::fs::create_dir(root.join("slot")).expect("the fixture folder is made");
    let held: std::fs::File = std::fs::File::open(root.as_path()).expect("the fixture root opens");
    let slot = open_directory_beneath(&held, "slot").expect("the folder opens");
    assert!(slot.as_raw_fd() > 0);
    let handle = open_path_handle(root.join("slot").as_path()).expect("the handle opens");
    assert!(handle.as_raw_fd() > 0);
}

/// A `..` escape refuses,
/// with the kernel's `EXDEV`.
#[test]
fn an_escape_refuses_with_exdev() {
    let root: PathBuf = fixture("start-probe-open-escape");
    let held: std::fs::File = std::fs::File::open(root.as_path()).expect("the fixture root opens");
    let refusal = open_directory_beneath(&held, "../escape").expect_err("the escape is refused");
    assert_eq!(refusal.errno, libc::EXDEV);
    assert_eq!(refusal.errno_name(), Some("EXDEV"));
}

/// A symbolic link refuses with `ELOOP`.
#[test]
fn a_link_refuses_with_eloop() {
    let root: PathBuf = fixture("start-probe-open-link");
    std::fs::create_dir(root.join("real")).expect("the fixture folder is made");
    std::os::unix::fs::symlink(root.join("real"), root.join("link")).expect("the link is made");
    let held: std::fs::File = std::fs::File::open(root.as_path()).expect("the fixture root opens");
    let refusal = open_directory_beneath(&held, "link").expect_err("the link is refused");
    assert_eq!(refusal.errno, libc::ELOOP);
}
