//! Controls for the attribute copy:
//! what a refused permission or time change means,
//! a named pipe,
//! and a link.

use super::*;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// A change refused for lack of ownership is counted and tolerated;
/// one refused for another reason is an error that
///  names what was being set;
/// a change that worked counts nothing.
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

/// A named pipe takes its exact permission bits and time through its descriptor,
/// and a link put where the pipe was
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

/// The owner is skipped only when both the user and the group agree;
/// each of the other three combinations needs the change.
#[test]
fn the_owner_is_left_alone_only_when_user_and_group_both_agree() {
    let (user, group): (u32, u32) = (1000, 1001);
    assert!(same_owner((user, group), (user, group)));
    assert!(!same_owner((user, group), (user + 1, group)), "another user");
    assert!(!same_owner((user, group), (user, group + 1)), "another group");
    assert!(!same_owner((user, group), (user + 1, group + 1)), "another user and group");
}

/// Whether the tests run as root,
/// which can give any owner and so cannot make a refused change.
fn running_as_root() -> bool {
    // SAFETY: `geteuid` takes no arguments and cannot fail.
    return unsafe { libc::geteuid() } == 0;
}

/// An owner this user cannot give is counted as lost,
/// once,
/// and the copy keeps its owner;
/// a copy that already has the
///  original's owner and group is left alone and counts nothing.
#[test]
fn an_owner_that_cannot_be_given_is_counted_and_an_equal_one_is_not_touched() {
    if running_as_root() {
        return;
    }
    let parent: PathBuf = fixture("attrs-owner-refused");
    std::fs::write(parent.join("copy"), b"x").expect("copy");
    let copy: File = File::open(parent.join("copy")).expect("open");
    let current: Metadata = copy.metadata().expect("metadata");
    let foreign: Metadata = std::fs::metadata("/").expect("the root directory is owned by root");
    assert_ne!(foreign.uid(), current.uid());
    let mut losses: AttributeLosses = AttributeLosses::default();
    set_owner(Handle::Descriptor(&copy), &current, &foreign, &mut losses);
    assert_eq!(losses.owner, 1);
    assert_eq!(copy.metadata().expect("metadata").uid(), current.uid());
    set_owner(Handle::Descriptor(&copy), &current, &current, &mut losses);
    assert_eq!(losses.owner, 1, "an equal owner is not an attempt");
    remove(parent.as_path());
}

/// A group the user belongs to can be given:
/// the copy ends in the original's group and nothing is counted lost.
/// Where the
///  user belongs to no other group the control reports that it did not run.
#[test]
fn a_group_the_user_belongs_to_is_given_and_not_counted() {
    if running_as_root() {
        return;
    }
    let parent: PathBuf = fixture("attrs-owner-group");
    std::fs::write(parent.join("original"), b"x").expect("original");
    std::fs::write(parent.join("copy"), b"x").expect("copy");
    let copy: File = File::open(parent.join("copy")).expect("open");
    let current: Metadata = copy.metadata().expect("metadata");
    // SAFETY: `getgroups` with a zero size only counts the groups; with a buffer it fills at most that many.
    let count: libc::c_int = unsafe { libc::getgroups(0, std::ptr::null_mut()) };
    let mut groups: Vec<libc::gid_t> = vec![0; usize::try_from(count).unwrap_or(0)];
    // SAFETY: `groups` has room for `count` entries.
    let filled: libc::c_int = unsafe { libc::getgroups(count, groups.as_mut_ptr()) };
    groups.truncate(usize::try_from(filled).unwrap_or(0));
    let Some(other) = groups.into_iter().find(|group: &libc::gid_t| { return *group != current.gid(); }) else {
        eprintln!("OWNER CONTROL NOT RUN: a_group_the_user_belongs_to_is_given_and_not_counted (the user belongs to no second group)");
        remove(parent.as_path());
        return;
    };
    let name = std::ffi::CString::new(parent.join("original").as_os_str().as_encoded_bytes()).expect("path");
    // SAFETY: `name` is NUL-terminated; a user id of -1 leaves the owner as it is.
    let set: libc::c_int = unsafe { libc::chown(name.as_ptr(), libc::uid_t::MAX, other) };
    assert_eq!(set, 0, "give the original a group the user belongs to");
    let original: Metadata = std::fs::metadata(parent.join("original")).expect("original metadata");
    let mut losses: AttributeLosses = AttributeLosses::default();
    set_owner(Handle::Descriptor(&copy), &current, &original, &mut losses);
    assert_eq!(losses.owner, 0);
    assert_eq!(copy.metadata().expect("metadata").gid(), other);
    remove(parent.as_path());
}

/// Set one user attribute on a path;
/// false where the filesystem keeps none.
fn set_user_attribute(path: &Path, name: &str, value: &[u8]) -> bool {
    let path_text = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("path");
    let name_text = std::ffi::CString::new(name).expect("name");
    // SAFETY: both names are NUL-terminated and `value` holds `value.len()` bytes; all outlive the call.
    return unsafe { libc::lsetxattr(path_text.as_ptr(), name_text.as_ptr(), value.as_ptr().cast(), value.len(), 0) } == 0;
}

/// Two paths carry the same attributes when they carry none,
/// or the same names with the same values,
/// and differ when a
///  name or a value differs.
#[test]
fn carried_attributes_are_equal_only_when_names_and_values_agree() {
    let parent: PathBuf = fixture("attrs-equal");
    for name in ["one", "two"] {
        std::fs::write(parent.join(name), b"x").expect("file");
    }
    let (one, two): (PathBuf, PathBuf) = (parent.join("one"), parent.join("two"));
    assert!(carried_attributes_equal(one.as_path(), two.as_path()), "no attributes on either");
    if !set_user_attribute(one.as_path(), "user.cli-git-test", b"value") {
        eprintln!("ATTRIBUTE CONTROL NOT RUN: carried_attributes_are_equal_only_when_names_and_values_agree (no user attributes on {})", parent.display());
        assert!(!carried_attributes_equal(one.as_path(), parent.join("absent").as_path()));
        remove(parent.as_path());
        return;
    }
    assert!(!carried_attributes_equal(one.as_path(), two.as_path()), "an attribute only one carries");
    assert!(set_user_attribute(two.as_path(), "user.cli-git-test", b"other"));
    assert!(!carried_attributes_equal(one.as_path(), two.as_path()), "the same name with another value");
    assert!(set_user_attribute(two.as_path(), "user.cli-git-test", b"value"));
    assert!(carried_attributes_equal(one.as_path(), two.as_path()), "the same name and value");
    assert!(!carried_attributes_equal(one.as_path(), parent.join("absent").as_path()), "a path that is not there");
    remove(parent.as_path());
}

/// A set the system refused loses nothing when the copy already holds that value,
/// which is what a mount forcing one security context reports:
/// the value it carries and the value it refuses to take are the same bytes.
#[test]
fn a_value_the_copy_already_holds_is_not_a_loss() {
    let parent: PathBuf = fixture("attrs-already-carried");
    let path: PathBuf = parent.join("copy");
    std::fs::write(path.as_path(), b"x").expect("file");
    let name: std::ffi::CString = std::ffi::CString::new("user.cli-git-test").expect("name");
    let handle: File = std::fs::File::open(path.as_path()).expect("copy");
    assert!(!already_carries(Handle::Descriptor(&handle), name.as_c_str(), b"value"), "nothing is carried before it is set");
    if !set_user_attribute(path.as_path(), "user.cli-git-test", b"value") {
        eprintln!("ATTRIBUTE CONTROL NOT RUN: a_value_the_copy_already_holds_is_not_a_loss (no user attributes on {})", parent.display());
        remove(parent.as_path());
        return;
    }
    assert!(already_carries(Handle::Descriptor(&handle), name.as_c_str(), b"value"), "the value it holds is the one a refused set would have given");
    assert!(!already_carries(Handle::Descriptor(&handle), name.as_c_str(), b"other"), "another value is not the one it holds");
    let absent: std::ffi::CString = std::ffi::CString::new("user.cli-git-absent").expect("name");
    assert!(!already_carries(Handle::Descriptor(&handle), absent.as_c_str(), b"value"), "a name it does not carry holds no value");
    remove(parent.as_path());
}
