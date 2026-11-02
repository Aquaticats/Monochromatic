//! Controls for the attribute copy: what a refused permission or time change means, a named pipe, and a link.

use super::*;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// A change refused for lack of ownership is counted and tolerated; one refused for another reason is an error that
///  names what was being set; a change that worked counts nothing.
#[test]
fn a_missing_privilege_is_counted_and_any_other_refusal_is_an_error() {
    let mut refused: u64 = 0;
    assert!(settle(Ok(()), "set the times of", &mut refused).is_ok());
    assert_eq!(refused, 0);
    for errno in [libc::EPERM, libc::EACCES] {
        assert!(settle(Err(std::io::Error::from_raw_os_error(errno)), "set the times of", &mut refused).is_ok());
    }
    assert_eq!(refused, 2);
    for errno in [libc::ENOSPC, libc::EROFS, libc::EIO] {
        let failure: AttributeFailure = settle(Err(std::io::Error::from_raw_os_error(errno)), "set the permission bits of", &mut refused).expect_err("refused");
        assert_eq!((failure.action, failure.error.raw_os_error()), ("set the permission bits of", Some(errno)));
    }
    assert_eq!(refused, 2, "an error is not counted as a tolerated loss");
}

/// A named pipe takes its exact permission bits and time through its descriptor, and a link put where the pipe was
///  is refused instead of followed.
#[test]
fn a_named_pipe_is_given_its_attributes_through_its_descriptor() {
    let parent: PathBuf = fixture("attrs-fifo");
    make_fifo(parent.join("original").as_path(), 0o640).expect("original pipe");
    make_fifo(parent.join("copy").as_path(), 0o600).expect("copy pipe");
    let metadata: Metadata = std::fs::symlink_metadata(parent.join("original")).expect("metadata");
    let handle: File = open_fifo(parent.join("copy").as_path()).expect("open without blocking");
    let mut losses: AttributeLosses = AttributeLosses::default();
    preserve_on_descriptor(parent.join("original").as_path(), &handle, &metadata, true, &mut losses).expect("attributes");
    let copied: Metadata = std::fs::symlink_metadata(parent.join("copy")).expect("copy metadata");
    assert_eq!((copied.mode() & 0o7777, copied.mtime(), copied.mtime_nsec()), (metadata.mode() & 0o7777, metadata.mtime(), metadata.mtime_nsec()));
    std::os::unix::fs::symlink(parent.join("original"), parent.join("link")).expect("link");
    assert!(open_fifo(parent.join("link").as_path()).is_err(), "a link is not opened");
    std::fs::set_permissions(parent.join("copy"), std::fs::Permissions::from_mode(0o600)).expect("mode");
    remove(parent.as_path());
}

/// A link takes the original's times without touching its target.
#[test]
fn a_link_is_given_its_times_without_following_it() {
    let parent: PathBuf = fixture("attrs-link");
    std::fs::write(parent.join("target"), b"x").expect("target");
    std::os::unix::fs::symlink("target", parent.join("original")).expect("original link");
    std::os::unix::fs::symlink("target", parent.join("copy")).expect("copy link");
    let name = std::ffi::CString::new(parent.join("original").as_os_str().as_encoded_bytes()).expect("path");
    let times: [libc::timespec; 2] = [libc::timespec { tv_sec: 1_000_000_000, tv_nsec: 5 }, libc::timespec { tv_sec: 1_000_000_001, tv_nsec: 7 }];
    // SAFETY: `name` is NUL-terminated and `times` holds two values, both alive for the call.
    let set: libc::c_int = unsafe { libc::utimensat(libc::AT_FDCWD, name.as_ptr(), times.as_ptr(), libc::AT_SYMLINK_NOFOLLOW) };
    assert_eq!(set, 0);
    let target_before: i64 = std::fs::metadata(parent.join("target")).expect("target").mtime();
    let metadata: Metadata = std::fs::symlink_metadata(parent.join("original")).expect("metadata");
    let mut losses: AttributeLosses = AttributeLosses::default();
    preserve_on_link(parent.join("original").as_path(), parent.join("copy").as_path(), &metadata, &mut losses).expect("attributes");
    let copied: Metadata = std::fs::symlink_metadata(parent.join("copy")).expect("copy");
    assert_eq!((copied.mtime(), copied.mtime_nsec()), (1_000_000_001, 7));
    assert_eq!(std::fs::metadata(parent.join("target")).expect("target").mtime(), target_before);
    remove(parent.as_path());
}
