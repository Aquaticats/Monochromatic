//! Controls for the tree pass: a first copy of every entry kind, the repeat that changes nothing, the deletion pass, the
//!  update-only pass, a change of entry type, hard links, the files and directories this user cannot read, the entries
//!  that are not recreated, the excluded path, and the stop at a simulated kill. They run on the stand-in platform
//!  (clones refused, so the byte-copy path runs); the clone path is the btrfs control's.

use super::*;
use crate::subvolume_sync_attrs::make_fifo;
use crate::subvolume_test_support::PlainPlatform;
use crate::test_support::{fixture, remove};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::symlink;

/// Whether the tests run as root, which can read every file and so cannot make an unreadable one.
fn running_as_root() -> bool {
    // SAFETY: `geteuid` takes no arguments and cannot fail.
    return unsafe { libc::geteuid() } == 0;
}

/// The default options: copy only.
const COPY: SyncOptions<'static> = SyncOptions { delete: false, update_only: false, excluded: &[] };

/// Set both times of a path to whole seconds after the epoch.
fn set_mtime(path: &Path, seconds: i64) {
    let times: [libc::timespec; 2] = [libc::timespec { tv_sec: seconds, tv_nsec: 123 }, libc::timespec { tv_sec: seconds, tv_nsec: 456_789 }];
    let name = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("path text");
    // SAFETY: `name` is NUL-terminated and `times` holds two values; both outlive the call.
    let result: libc::c_int = unsafe { libc::utimensat(libc::AT_FDCWD, name.as_ptr(), times.as_ptr(), libc::AT_SYMLINK_NOFOLLOW) };
    assert_eq!(result, 0, "set the time of {}", path.display());
}

/// A source tree with one of each kind of entry, and an empty destination beside it.
fn trees(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let parent: PathBuf = fixture(name);
    let source: PathBuf = parent.join("source");
    let destination: PathBuf = parent.join("destination");
    std::fs::create_dir_all(source.join("dir/deeper")).expect("directories");
    std::fs::create_dir(destination.as_path()).expect("destination");
    std::fs::write(source.join("file.txt"), b"contents\n").expect("file");
    std::fs::write(source.join("dir/deeper/leaf"), vec![7u8; 70_000]).expect("leaf");
    std::fs::write(source.join("empty"), b"").expect("empty");
    std::fs::write(source.join("tool.sh"), b"#!/bin/sh\n").expect("tool");
    std::fs::set_permissions(source.join("tool.sh"), PermissionsExt::from_mode(0o750)).expect("mode");
    symlink("file.txt", source.join("link")).expect("link");
    set_mtime(source.join("file.txt").as_path(), 1_000_000_000);
    return (parent, source, destination);
}

/// Run a pass on the stand-in platform.
fn run(source: &Path, destination: &Path, options: SyncOptions<'_>) -> SyncStats {
    return sync_tree(&PlainPlatform::new(), source, destination, options, &Interrupt::never()).expect("pass");
}

/// A first pass copies every kind with its bytes, mode, symbolic link and modification time to the nanosecond, and the
///  counts say what was done.
#[test]
fn a_first_pass_copies_every_kind_of_entry() {
    let (parent, source, destination) = trees("sync-first");
    make_fifo(source.join("pipe").as_path(), 0o640).expect("fifo");
    let stats: SyncStats = run(source.as_path(), destination.as_path(), COPY);
    assert_eq!(std::fs::read(destination.join("dir/deeper/leaf")).expect("leaf"), vec![7u8; 70_000]);
    assert_eq!(std::fs::read(destination.join("empty")).expect("empty"), Vec::<u8>::new());
    assert_eq!(std::fs::metadata(destination.join("tool.sh")).expect("tool").mode() & 0o7777, 0o750);
    assert_eq!(std::fs::read_link(destination.join("link")).expect("link"), PathBuf::from("file.txt"));
    assert!(std::fs::symlink_metadata(destination.join("pipe")).expect("pipe").file_type().is_fifo());
    let (left, right) = (std::fs::metadata(source.join("file.txt")).expect("source"), std::fs::metadata(destination.join("file.txt")).expect("copy"));
    assert_eq!((left.mtime(), left.mtime_nsec()), (right.mtime(), right.mtime_nsec()));
    let directory_times: (i64, i64) = (std::fs::metadata(source.join("dir")).expect("dir").mtime(), std::fs::metadata(destination.join("dir")).expect("dir").mtime());
    assert_eq!(directory_times.0, directory_times.1, "a directory's own time is applied after its children");
    assert_eq!((stats.files_copied, stats.cloned, stats.symlinks, stats.fifos, stats.directories), (4, 0, 1, 1, 2));
    assert_eq!((stats.unchanged, stats.deleted, stats.hard_linked), (0, 0, 0));
    assert_eq!(stats.entries, 8);
    assert!(stats.unreadable.is_empty() && stats.special_skipped.is_empty());
    remove(parent.as_path());
}

/// A second pass over equal trees copies nothing and counts every non-directory entry as unchanged.
#[test]
fn a_repeated_pass_changes_nothing() {
    let (parent, source, destination) = trees("sync-repeat");
    make_fifo(source.join("pipe").as_path(), 0o640).expect("fifo");
    run(source.as_path(), destination.as_path(), COPY);
    let stats: SyncStats = run(source.as_path(), destination.as_path(), COPY);
    assert_eq!((stats.files_copied, stats.symlinks, stats.fifos), (0, 0, 0));
    assert_eq!(stats.unchanged, 6, "four files, one link, one pipe");
    remove(parent.as_path());
}

/// A changed file (new size) is replaced; one whose permission bits differ is replaced too.
#[test]
fn a_changed_file_is_copied_again() {
    let (parent, source, destination) = trees("sync-changed");
    run(source.as_path(), destination.as_path(), COPY);
    std::fs::write(source.join("file.txt"), b"longer contents\n").expect("rewrite");
    std::fs::set_permissions(source.join("tool.sh"), PermissionsExt::from_mode(0o700)).expect("mode");
    set_mtime(source.join("tool.sh").as_path(), 1_000_000_001);
    let stats: SyncStats = run(source.as_path(), destination.as_path(), COPY);
    assert_eq!(stats.files_copied, 2);
    assert_eq!(std::fs::read(destination.join("file.txt")).expect("copy"), b"longer contents\n");
    assert_eq!(std::fs::metadata(destination.join("tool.sh")).expect("tool").mode() & 0o7777, 0o700);
    remove(parent.as_path());
}

/// The deletion pass removes what the source lacks (files and whole directories) and keeps the excluded path in both
///  directions: it is neither copied from the source nor deleted from the destination.
#[test]
fn the_deletion_pass_removes_extras_and_spares_the_excluded_path() {
    let (parent, source, destination) = trees("sync-delete");
    run(source.as_path(), destination.as_path(), COPY);
    std::fs::write(destination.join("extra.txt"), b"x").expect("extra file");
    std::fs::create_dir_all(destination.join("extra-dir/inner")).expect("extra directory");
    std::fs::write(destination.join("extra-dir/inner/f"), b"x").expect("inner");
    std::fs::write(destination.join("kept.lock"), b"x").expect("excluded in the destination");
    std::fs::write(source.join("skipped.lock"), b"x").expect("excluded in the source");
    let excluded: [PathBuf; 2] = [PathBuf::from("kept.lock"), PathBuf::from("skipped.lock")];
    let stats: SyncStats = run(source.as_path(), destination.as_path(), SyncOptions { delete: true, update_only: false, excluded: &excluded });
    assert_eq!(stats.deleted, 2);
    assert!(!destination.join("extra.txt").exists() && !destination.join("extra-dir").exists());
    assert!(destination.join("kept.lock").exists() && !destination.join("skipped.lock").exists());
    let kept: SyncStats = run(source.as_path(), destination.as_path(), COPY);
    assert_eq!(kept.deleted, 0, "a copy pass without deletion deletes nothing");
    remove(parent.as_path());
}

/// Without the deletion switch extras stay; with the update-only switch they stay even when deletion is also asked.
#[test]
fn extras_stay_without_deletion_and_with_update_only() {
    let (parent, source, destination) = trees("sync-extras");
    run(source.as_path(), destination.as_path(), COPY);
    std::fs::write(destination.join("extra.txt"), b"x").expect("extra file");
    run(source.as_path(), destination.as_path(), COPY);
    assert!(destination.join("extra.txt").exists());
    let stats: SyncStats = run(source.as_path(), destination.as_path(), SyncOptions { delete: true, update_only: true, excluded: &[] });
    assert_eq!(stats.deleted, 0);
    assert!(destination.join("extra.txt").exists());
    remove(parent.as_path());
}

/// The update-only pass keeps a destination file that is newer, replaces one that is older, and adds what is missing.
#[test]
fn the_update_only_pass_lets_the_newer_file_win() {
    let (parent, source, destination) = trees("sync-update");
    run(source.as_path(), destination.as_path(), COPY);
    std::fs::write(destination.join("file.txt"), b"newer in the destination\n").expect("destination edit");
    set_mtime(destination.join("file.txt").as_path(), 1_500_000_000);
    std::fs::write(source.join("tool.sh"), b"#!/bin/sh\necho newer\n").expect("source edit");
    set_mtime(source.join("tool.sh").as_path(), 2_000_000_000);
    std::fs::write(source.join("added"), b"new file").expect("added");
    let stats: SyncStats = run(source.as_path(), destination.as_path(), SyncOptions { delete: false, update_only: true, excluded: &[] });
    assert_eq!(std::fs::read(destination.join("file.txt")).expect("kept"), b"newer in the destination\n");
    assert_eq!(std::fs::read(destination.join("tool.sh")).expect("replaced"), b"#!/bin/sh\necho newer\n");
    assert_eq!(std::fs::read(destination.join("added")).expect("added"), b"new file");
    assert_eq!(stats.files_copied, 2);
    remove(parent.as_path());
}

/// An entry that changed type is replaced: a directory where the destination has a file, a file where it has a
///  directory, a link where it has a file, and a link whose target changed.
#[test]
fn an_entry_that_changed_type_is_replaced() {
    let (parent, source, destination) = trees("sync-types");
    std::fs::write(destination.join("dir"), b"a file where the source has a directory").expect("file for directory");
    std::fs::create_dir(destination.join("file.txt")).expect("directory for file");
    std::fs::write(destination.join("link"), b"a file where the source has a link").expect("file for link");
    run(source.as_path(), destination.as_path(), COPY);
    assert!(destination.join("dir/deeper/leaf").is_file() && destination.join("file.txt").is_file());
    assert_eq!(std::fs::read_link(destination.join("link")).expect("link"), PathBuf::from("file.txt"));
    std::fs::remove_file(source.join("link")).expect("remove");
    symlink("empty", source.join("link")).expect("retargeted link");
    run(source.as_path(), destination.as_path(), COPY);
    assert_eq!(std::fs::read_link(destination.join("link")).expect("link"), PathBuf::from("empty"));
    remove(parent.as_path());
}

/// A named pipe replaces a file of the same name, and a pipe already there is left alone.
#[test]
fn a_named_pipe_replaces_a_file_and_is_then_unchanged() {
    let (parent, source, destination) = trees("sync-fifo");
    make_fifo(source.join("pipe").as_path(), 0o600).expect("fifo");
    std::fs::write(destination.join("pipe"), b"x").expect("file in the way");
    let first: SyncStats = run(source.as_path(), destination.as_path(), COPY);
    assert_eq!(first.fifos, 1);
    let second: SyncStats = run(source.as_path(), destination.as_path(), COPY);
    assert_eq!((second.fifos, second.unchanged), (0, 6));
    remove(parent.as_path());
}

/// A pipe that cannot be made reports the operating system's words; a path with an interior NUL is refused as input.
#[test]
fn a_pipe_that_cannot_be_made_is_an_error() {
    let parent: PathBuf = fixture("sync-fifo-error");
    std::fs::write(parent.join("taken"), b"x").expect("file");
    let taken: std::io::Error = make_fifo(parent.join("taken").as_path(), 0o600).expect_err("exists");
    assert_eq!(taken.raw_os_error(), Some(libc::EEXIST));
    let nul: PathBuf = PathBuf::from(std::ffi::OsStr::from_bytes(b"a\0b"));
    assert!(make_fifo(nul.as_path(), 0o600).is_err());
    remove(parent.as_path());
}

/// Two names of one file become two names of one file in the copy; a copy whose link was broken is linked again.
#[test]
fn hard_links_stay_hard_links_and_a_broken_link_is_restored() {
    let (parent, source, destination) = trees("sync-links");
    std::fs::hard_link(source.join("file.txt"), source.join("dir/second-name")).expect("hard link");
    let first: SyncStats = run(source.as_path(), destination.as_path(), COPY);
    assert_eq!(first.hard_linked, 1);
    let ino = |path: PathBuf| -> u64 { return std::fs::metadata(path).expect("metadata").ino(); };
    assert_eq!(ino(destination.join("file.txt")), ino(destination.join("dir/second-name")));
    let repeat: SyncStats = run(source.as_path(), destination.as_path(), COPY);
    assert_eq!(repeat.files_copied + repeat.hard_linked, 0, "a correct link is left alone");
    // An independent file with the same bytes, mode and time: equal by content, but no longer the same file.
    std::fs::remove_file(destination.join("dir/second-name")).expect("remove");
    std::fs::write(destination.join("dir/second-name"), b"contents\n").expect("independent copy");
    set_mtime(destination.join("dir/second-name").as_path(), 1_000_000_000);
    let mode: u32 = std::fs::metadata(source.join("file.txt")).expect("source").mode() & 0o7777;
    std::fs::set_permissions(destination.join("dir/second-name"), PermissionsExt::from_mode(mode)).expect("mode");
    let relinked: SyncStats = run(source.as_path(), destination.as_path(), COPY);
    assert_eq!(relinked.hard_linked, 1);
    assert_eq!(ino(destination.join("file.txt")), ino(destination.join("dir/second-name")));
    remove(parent.as_path());
}

/// A destination directory that lacks owner write permission is made writable for the pass, and its final permission
///  bits are applied afterwards.
#[test]
fn a_read_only_directory_is_filled_and_ends_read_only() {
    let (parent, source, destination) = trees("sync-read-only");
    std::fs::create_dir(source.join("locked")).expect("directory");
    std::fs::write(source.join("locked/a"), b"a").expect("file");
    std::fs::set_permissions(source.join("locked"), PermissionsExt::from_mode(0o555)).expect("read-only");
    run(source.as_path(), destination.as_path(), COPY);
    std::fs::set_permissions(source.join("locked"), PermissionsExt::from_mode(0o755)).expect("writable for the edit");
    std::fs::write(source.join("locked/b"), b"b").expect("file");
    std::fs::set_permissions(source.join("locked"), PermissionsExt::from_mode(0o555)).expect("read-only again");
    run(source.as_path(), destination.as_path(), COPY);
    assert_eq!(std::fs::read(destination.join("locked/b")).expect("copied into the read-only directory"), b"b");
    assert_eq!(std::fs::metadata(destination.join("locked")).expect("directory").mode() & 0o7777, 0o555);
    std::fs::set_permissions(source.join("locked"), PermissionsExt::from_mode(0o755)).expect("restore");
    std::fs::set_permissions(destination.join("locked"), PermissionsExt::from_mode(0o755)).expect("restore");
    remove(parent.as_path());
}

/// A file this user may not read is left out, listed by its relative path, and does not stop the pass.
#[test]
fn an_unreadable_file_is_listed_and_the_pass_continues() {
    if running_as_root() {
        return;
    }
    let (parent, source, destination) = trees("sync-unreadable-file");
    std::fs::write(source.join("dir/secret"), b"x").expect("secret");
    std::fs::set_permissions(source.join("dir/secret"), PermissionsExt::from_mode(0o000)).expect("unreadable");
    let stats: SyncStats = run(source.as_path(), destination.as_path(), COPY);
    assert_eq!(stats.unreadable, vec![PathBuf::from("dir/secret")]);
    assert!(!destination.join("dir/secret").exists() && destination.join("file.txt").exists());
    remove(parent.as_path());
}

/// A directory this user cannot list stops the pass, naming it.
#[test]
fn an_unreadable_directory_stops_the_pass_naming_it() {
    if running_as_root() {
        return;
    }
    let (parent, source, destination) = trees("sync-unreadable-dir");
    std::fs::create_dir(source.join("dir/closed")).expect("directory");
    std::fs::set_permissions(source.join("dir/closed"), PermissionsExt::from_mode(0o000)).expect("closed");
    let error: MigrateError = sync_tree(&PlainPlatform::new(), source.as_path(), destination.as_path(), COPY, &Interrupt::never()).expect_err("stopped");
    std::fs::set_permissions(source.join("dir/closed"), PermissionsExt::from_mode(0o755)).expect("restore");
    assert!(matches!(&error, MigrateError::UnreadableDirectory { path, .. } if path.ends_with("dir/closed")), "{error}");
    remove(parent.as_path());
}

/// A socket is not recreated and is listed; a source that does not exist is an error naming it.
#[test]
fn a_socket_is_listed_and_a_missing_source_is_an_error() {
    let (parent, source, destination) = trees("sync-special");
    // A socket node made with `mknod`, because a bound socket's path must fit the short `sun_path` limit.
    let node = std::ffi::CString::new(source.join("dir/sock").as_os_str().as_encoded_bytes()).expect("path text");
    // SAFETY: `node` is NUL-terminated and outlives the call; a socket node takes no device number.
    let made: libc::c_int = unsafe { libc::mknod(node.as_ptr(), libc::S_IFSOCK | 0o600, 0) };
    assert_eq!(made, 0, "make the socket node: {}", std::io::Error::last_os_error());
    let stats: SyncStats = run(source.as_path(), destination.as_path(), COPY);
    assert_eq!(stats.special_skipped, vec![PathBuf::from("dir/sock")]);
    assert!(!destination.join("dir/sock").exists());
    let error: MigrateError = sync_tree(&PlainPlatform::new(), parent.join("absent").as_path(), destination.as_path(), COPY, &Interrupt::never()).expect_err("missing");
    assert!(error.to_string().contains("cannot examine") && error.to_string().contains("absent"), "{error}");
    remove(parent.as_path());
}

/// The simulated kill stops the pass at the chosen file, and a repeat completes the tree.
#[test]
fn a_stopped_pass_is_completed_by_the_next_one() {
    let (parent, source, destination) = trees("sync-interrupted");
    let error: MigrateError = sync_tree(&PlainPlatform::new(), source.as_path(), destination.as_path(), COPY, &Interrupt::at("copy-entry", 1)).expect_err("stopped");
    assert_eq!(error, MigrateError::Interrupted(String::from("copy-entry")));
    run(source.as_path(), destination.as_path(), COPY);
    assert_eq!(std::fs::read(destination.join("dir/deeper/leaf")).expect("leaf"), vec![7u8; 70_000]);
    assert_eq!(std::fs::read(destination.join("file.txt")).expect("file"), b"contents\n");
    remove(parent.as_path());
}

/// Extended attributes follow the file where the filesystem has them; a filesystem without them says so loudly.
#[test]
fn user_extended_attributes_are_copied() {
    let (parent, source, destination) = trees("sync-xattr");
    let name = std::ffi::CString::new("user.cli-git-test").expect("name");
    let path = std::ffi::CString::new(source.join("file.txt").as_os_str().as_encoded_bytes()).expect("path");
    // SAFETY: both strings are NUL-terminated and the value is five readable bytes.
    let set: libc::c_int = unsafe { libc::lsetxattr(path.as_ptr(), name.as_ptr(), b"value".as_ptr().cast(), 5, 0) };
    if set != 0 {
        eprintln!("XATTR CONTROL NOT RUN: this filesystem refuses user extended attributes ({})", std::io::Error::last_os_error());
        remove(parent.as_path());
        return;
    }
    let stats: SyncStats = run(source.as_path(), destination.as_path(), COPY);
    let copy = std::ffi::CString::new(destination.join("file.txt").as_os_str().as_encoded_bytes()).expect("path");
    let mut value: [u8; 8] = [0; 8];
    // SAFETY: the buffers are valid for the lengths given and the strings are NUL-terminated.
    let read: isize = unsafe { libc::lgetxattr(copy.as_ptr(), name.as_ptr(), value.as_mut_ptr().cast(), value.len()) };
    assert_eq!(&value[..usize::try_from(read).expect("attribute present")], b"value");
    assert!(stats.losses.extended_attributes <= 1, "at most the security label may be refused: {:?}", stats.losses);
    remove(parent.as_path());
}
