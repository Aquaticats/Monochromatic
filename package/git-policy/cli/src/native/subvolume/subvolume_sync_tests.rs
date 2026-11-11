//! Controls for the tree pass:
//! a first copy of every entry kind,
//! the repeat that changes nothing,
//! the deletion pass,
//! the
//!  update-only pass,
//! a change of entry type,
//! hard links,
//! the files and directories this user cannot read,
//! the entries
//!  that are not recreated,
//! the excluded path,
//! and the stop at a simulated kill.
//! They run on the stand-in platform
//!  (clones refused,
//! so the byte-copy path runs);
//! the clone path is the btrfs control's.

use super::*;
use crate::subvolume_anchor::Anchor;
use std::fs::File;
use crate::subvolume_sync_attrs::make_fifo;
use crate::subvolume_test_support::{Hooked, PlainPlatform};
use crate::test_support::{fixture, remove};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{PermissionsExt, symlink};

/// Whether the tests run as root,
/// which can read every file and so cannot make an unreadable one.
fn running_as_root() -> bool {
    // SAFETY: `geteuid` takes no arguments and cannot fail.
    return unsafe { libc::geteuid() } == 0;
}

/// The default options:
/// copy only.
const COPY: SyncOptions<'static> = SyncOptions { delete: false, update_only: false, excluded: &[], created_since: None };

/// Set both times of a path to whole seconds after the epoch.
fn set_mtime(path: &Path, seconds: i64) {
    let times: [libc::timespec; 2] = [libc::timespec { tv_sec: seconds, tv_nsec: 123 }, libc::timespec { tv_sec: seconds, tv_nsec: 456_789 }];
    let name = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("path text");
    // SAFETY: `name` is NUL-terminated and `times` holds two values; both outlive the call.
    let result: libc::c_int = unsafe { libc::utimensat(libc::AT_FDCWD, name.as_ptr(), times.as_ptr(), libc::AT_SYMLINK_NOFOLLOW) };
    assert_eq!(result, 0, "set the time of {}", path.display());
}

/// A source tree with one of each kind of entry,
/// and an empty destination beside it.
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

/// A first pass copies every kind with its bytes,
/// mode,
/// symbolic link and modification time to the nanosecond,
/// and the
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

/// A changed file (new size) is replaced;
/// one whose permission bits differ is replaced too.
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
///  directions:
/// it is neither copied from the source nor deleted from the destination.
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
    let stats: SyncStats = run(source.as_path(), destination.as_path(), SyncOptions { delete: true, update_only: false, excluded: &excluded, created_since: None });
    assert_eq!(stats.deleted, 2);
    assert!(!destination.join("extra.txt").exists() && !destination.join("extra-dir").exists());
    assert!(destination.join("kept.lock").exists() && !destination.join("skipped.lock").exists());
    let kept: SyncStats = run(source.as_path(), destination.as_path(), COPY);
    assert_eq!(kept.deleted, 0, "a copy pass without deletion deletes nothing");
    remove(parent.as_path());
}

/// What:
///  Another command's stage container is left out of a pass at any depth,
///  and nothing of it is copied.
/// Why:
///   A stage container holds the whole ignored state another worktree's copy is filling,
/// credentials
///  included,
/// and nothing names it in the copy;
/// a conversion that carried it into the new subvolume would leave that
///  state in a worktree on another branch for good.
#[test]
fn another_commands_stage_container_is_left_out_of_a_pass() {
    let (parent, source, destination) = trees("sync-stage-container");
    let container: PathBuf = source.join(".claude/worktrees/.cli-git-worktree-copy-8f14e45fceea167a5a36dedd4bea2543/payload");
    std::fs::create_dir_all(&container).expect("the stage container");
    std::fs::write(container.join("credentials"), b"token\n").expect("its payload");
    run(source.as_path(), destination.as_path(), COPY);
    assert!(!destination.join(".claude").exists(), "nothing of the stage container is copied");
    let proved: SyncStats = run(source.as_path(), destination.as_path(), SyncOptions { delete: true, update_only: false, excluded: &[], created_since: None });
    assert_eq!((proved.files_copied, proved.deleted), (0, 0), "the proving pass finds nothing to do");
    remove(parent.as_path());
}

/// Without the deletion switch extras stay;
/// with the update-only switch they stay even when deletion is also asked.
#[test]
fn extras_stay_without_deletion_and_with_update_only() {
    let (parent, source, destination) = trees("sync-extras");
    run(source.as_path(), destination.as_path(), COPY);
    std::fs::write(destination.join("extra.txt"), b"x").expect("extra file");
    run(source.as_path(), destination.as_path(), COPY);
    assert!(destination.join("extra.txt").exists());
    let stats: SyncStats = run(source.as_path(), destination.as_path(), SyncOptions { delete: true, update_only: true, excluded: &[], created_since: None });
    assert_eq!(stats.deleted, 0);
    assert!(destination.join("extra.txt").exists());
    remove(parent.as_path());
}

/// The update-only pass keeps a destination file that is newer,
/// replaces one that is older,
/// and adds what is missing.
#[test]
fn the_update_only_pass_lets_the_newer_file_win() {
    let (parent, source, destination) = trees("sync-update");
    run(source.as_path(), destination.as_path(), COPY);
    std::fs::write(destination.join("file.txt"), b"newer in the destination\n").expect("destination edit");
    set_mtime(destination.join("file.txt").as_path(), 1_500_000_000);
    std::fs::write(source.join("tool.sh"), b"#!/bin/sh\necho newer\n").expect("source edit");
    set_mtime(source.join("tool.sh").as_path(), 2_000_000_000);
    std::fs::write(source.join("added"), b"new file").expect("added");
    let stats: SyncStats = run(source.as_path(), destination.as_path(), SyncOptions { delete: false, update_only: true, excluded: &[], created_since: None });
    assert_eq!(std::fs::read(destination.join("file.txt")).expect("kept"), b"newer in the destination\n");
    assert_eq!(std::fs::read(destination.join("tool.sh")).expect("replaced"), b"#!/bin/sh\necho newer\n");
    assert_eq!(std::fs::read(destination.join("added")).expect("added"), b"new file");
    assert_eq!(stats.files_copied, 2);
    remove(parent.as_path());
}

/// An entry that changed type is replaced:
/// a directory where the destination has a file,
/// a file where it has a
///  directory,
/// a link where it has a file,
/// and a link whose target changed.
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

/// A named pipe replaces a file of the same name,
/// and a pipe already there is left alone.
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

/// A pipe that cannot be made reports the operating system's words;
/// a path with an interior NUL is refused as input.
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

/// Two names of one file become two names of one file in the copy;
/// a copy whose link was broken is linked again.
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

/// A destination directory that lacks owner write permission is made writable for the pass,
/// and its final permission
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
    // Both copies stay closed, so the fixture goes closed:
    // the teardown gives this user the permission back before it removes the tree (`test_support::settle_fixture`).
    remove(parent.as_path());
}

/// A file this user may not read is left out,
/// listed by its relative path,
/// and does not stop the pass.
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

/// A directory this user cannot list stops the pass,
/// naming it.
#[test]
fn an_unreadable_directory_stops_the_pass_naming_it() {
    if running_as_root() {
        return;
    }
    let (parent, source, destination) = trees("sync-unreadable-dir");
    std::fs::create_dir(source.join("dir/closed")).expect("directory");
    std::fs::set_permissions(source.join("dir/closed"), PermissionsExt::from_mode(0o000)).expect("closed");
    let error: MigrateError = sync_tree(&PlainPlatform::new(), source.as_path(), destination.as_path(), COPY, &Interrupt::never()).expect_err("stopped");
    // The directory stays closed, and the fixture goes closed too:
    // the teardown gives this user the permission back before it removes the tree (`test_support::settle_fixture`).
    assert!(matches!(&error, MigrateError::UnreadableDirectory { path, .. } if path.ends_with("dir/closed")), "{error}");
    remove(parent.as_path());
}

/// A socket is not recreated and is listed;
/// a source that does not exist is an error naming it.
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

/// The simulated kill stops the pass at the chosen file,
/// and a repeat completes the tree.
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

/// Extended attributes follow the file where the filesystem has them;
/// a filesystem without them says so loudly.
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
    // Only the security label may be refused, and at most once per entry: a mount that forces one context (`context=` on a
    // container's overlay) refuses it on every entry, the root of the tree included.
    let entries_with_a_label: u64 = stats.entries + 1;
    assert!(stats.losses.extended_attributes <= entries_with_a_label, "at most one security label per entry may be refused: {:?}", stats.losses);
    remove(parent.as_path());
}

/// The update-only options.
const UPDATE: SyncOptions<'static> = SyncOptions { delete: false, update_only: true, excluded: &[], created_since: None };

/// Replace a destination file the way an editor does:
/// write a sibling and rename it over the name.
fn replace_atomically(path: &Path, contents: &[u8]) {
    let sibling: PathBuf = path.with_extension("editor-tmp");
    std::fs::write(sibling.as_path(), contents).expect("sibling");
    std::fs::rename(sibling.as_path(), path).expect("rename over");
}

/// A pass killed after the new output was created and before its content was written leaves nothing at the destination
///  name that a later update-only pass could take for a newer file.
#[test]
fn a_killed_update_only_copy_is_not_mistaken_for_a_newer_file() {
    let (parent, source, destination) = trees("sync-partial");
    run(source.as_path(), destination.as_path(), COPY);
    std::fs::write(source.join("file.txt"), b"a longer replacement\n").expect("source edit");
    set_mtime(source.join("file.txt").as_path(), 1_000_000_500);
    let stopped = sync_tree(&PlainPlatform::new(), source.as_path(), destination.as_path(), UPDATE, &Interrupt::at("copy-partial", 0));
    assert_eq!(stopped.expect_err("killed"), MigrateError::Interrupted(String::from("copy-partial")));
    run(source.as_path(), destination.as_path(), UPDATE);
    assert_eq!(std::fs::read(destination.join("file.txt")).expect("copy"), b"a longer replacement\n");
    let leftovers: Vec<String> = std::fs::read_dir(destination.as_path()).expect("listing").flatten().map(|item| { return item.file_name().to_string_lossy().into_owned(); }).collect();
    assert!(leftovers.iter().all(|name: &String| { return !name.contains("partial"); }), "no temporary file is left: {leftovers:?}");
    remove(parent.as_path());
}

/// A destination directory that took the place of a source file holds newer work:
/// the update-only pass keeps it and
///  names it,
/// instead of deleting it recursively.
#[test]
fn an_update_only_pass_never_removes_a_destination_directory_or_a_newer_entry_of_another_type() {
    let (parent, source, destination) = trees("sync-conflict");
    run(source.as_path(), destination.as_path(), COPY);
    std::fs::remove_file(destination.join("file.txt")).expect("remove");
    std::fs::create_dir(destination.join("file.txt")).expect("directory in the file's place");
    std::fs::write(destination.join("file.txt/work.txt"), b"untracked work").expect("work");
    std::fs::remove_file(destination.join("link")).expect("remove link");
    std::fs::write(destination.join("link"), b"a newer regular file").expect("file in the link's place");
    let stats: SyncStats = run(source.as_path(), destination.as_path(), UPDATE);
    assert_eq!(std::fs::read(destination.join("file.txt/work.txt")).expect("kept"), b"untracked work");
    assert_eq!(std::fs::read(destination.join("link")).expect("kept"), b"a newer regular file");
    assert_eq!(stats.conflicts, vec![PathBuf::from("file.txt"), PathBuf::from("link")]);
    remove(parent.as_path());
}

/// An older destination symlink or file of another type is still replaced by the update-only pass,
/// so a real change in
///  the source wins.
#[test]
fn an_update_only_pass_replaces_an_older_entry_of_another_type() {
    let (parent, source, destination) = trees("sync-older-type");
    run(source.as_path(), destination.as_path(), COPY);
    std::fs::remove_file(source.join("link")).expect("remove");
    symlink("empty", source.join("link")).expect("retarget");
    set_mtime(source.join("link").as_path(), 2_000_000_000);
    run(source.as_path(), destination.as_path(), UPDATE);
    assert_eq!(std::fs::read_link(destination.join("link")).expect("link"), PathBuf::from("empty"));
    remove(parent.as_path());
}

/// A destination name that was replaced with newer content while its sibling hard link stayed as copied must not pass its
///  content on to the sibling,
/// whichever of the two the pass meets first.
#[test]
fn an_update_only_pass_does_not_spread_newer_content_through_a_hard_link() {
    for newer in ["dir/a", "dir/b"] {
        let (parent, source, destination) = trees(format!("sync-link-spread-{}", newer.replace('/', "-")).as_str());
        std::fs::write(source.join("dir/a"), b"old").expect("a");
        std::fs::hard_link(source.join("dir/a"), source.join("dir/b")).expect("b");
        run(source.as_path(), destination.as_path(), COPY);
        replace_atomically(destination.join(newer).as_path(), b"new, from an editor");
        run(source.as_path(), destination.as_path(), UPDATE);
        let (other, other_content): (&str, &[u8]) = if newer == "dir/a" { ("dir/b", b"old") } else { ("dir/a", b"old") };
        assert_eq!(std::fs::read(destination.join(newer)).expect("newer"), b"new, from an editor", "{newer}");
        assert_eq!(std::fs::read(destination.join(other)).expect("sibling"), other_content, "{newer}: the sibling keeps its own content");
        remove(parent.as_path());
    }
}

/// A platform that appends to `path` (and moves its time on) at the clone calls the predicate picks,
/// as a writer would
///  between the moment the pass looked at the file and the moment it copies it.
fn racing(path: PathBuf, picks: fn(u32) -> bool) -> Hooked {
    let on_clone = move |call: u32| {
        if picks(call) {
            use std::io::Write;
            let mut writer: File = std::fs::OpenOptions::new().append(true).open(path.as_path()).expect("append");
            writer.write_all(format!("more {call}\n").as_bytes()).expect("write");
            set_mtime(path.as_path(), 1_100_000_000 + i64::from(call));
        }
    };
    return Hooked { inner: PlainPlatform::new(), on_clone: Box::new(on_clone), calls: std::cell::Cell::new(0), links_fail: false };
}

/// A source file that changes while it is being copied is copied again,
/// and the copy carries the size and time of the
///  content it holds,
/// not those of the directory listing.
#[test]
fn a_source_that_changes_during_the_copy_is_copied_again() {
    let (parent, source, destination) = trees("sync-racing");
    let platform: Hooked = racing(source.join("file.txt"), |call: u32| { return call == 0; });
    sync_tree(&platform, source.as_path(), destination.as_path(), COPY, &Interrupt::never()).expect("pass");
    let (left, right) = (std::fs::metadata(source.join("file.txt")).expect("source"), std::fs::metadata(destination.join("file.txt")).expect("copy"));
    assert_eq!(std::fs::read(destination.join("file.txt")).expect("copy"), std::fs::read(source.join("file.txt")).expect("source"));
    assert_eq!((left.len(), left.mtime(), left.mtime_nsec()), (right.len(), right.mtime(), right.mtime_nsec()));
    remove(parent.as_path());
}

/// A source that never stops changing is an error naming the file,
/// not a torn copy recorded as finished.
#[test]
fn a_source_that_never_settles_is_an_error() {
    let (parent, source, destination) = trees("sync-racing-always");
    let platform: Hooked = racing(source.join("file.txt"), |_call: u32| { return true; });
    let error: MigrateError = sync_tree(&platform, source.as_path(), destination.as_path(), COPY, &Interrupt::never()).expect_err("never settles");
    assert!(error.to_string().contains("file.txt") && error.to_string().contains("while it was being copied"), "{error}");
    remove(parent.as_path());
}

/// Two names of one file whose destinations cannot be linked (different subvolumes) become two copies.
#[test]
fn hard_links_that_cannot_be_made_become_separate_copies() {
    let (parent, source, destination) = trees("sync-link-exdev");
    std::fs::hard_link(source.join("file.txt"), source.join("dir/second-name")).expect("hard link");
    let platform: Hooked = Hooked { inner: PlainPlatform::new(), on_clone: Box::new(|_call: u32| {}), calls: std::cell::Cell::new(0), links_fail: true };
    let stats: SyncStats = sync_tree(&platform, source.as_path(), destination.as_path(), COPY, &Interrupt::never()).expect("pass");
    assert_eq!(stats.hard_linked, 0);
    let (first, second) = (std::fs::metadata(destination.join("file.txt")).expect("first"), std::fs::metadata(destination.join("dir/second-name")).expect("second"));
    assert_ne!(first.ino(), second.ino());
    assert_eq!(std::fs::read(destination.join("dir/second-name")).expect("copy"), b"contents\n");
    remove(parent.as_path());
}

/// Directories of the destination that are replaced by links to other directories while the pass runs must not redirect
///  the pass:
/// the files outside the tree keep their content and times,
/// and the replaced directories are listed.
#[test]
fn a_destination_directory_replaced_by_a_link_during_the_pass_redirects_nothing() {
    let parent: PathBuf = fixture("sync-swapped");
    let (source, destination, outside) = (parent.join("source"), parent.join("destination"), parent.join("outside"));
    for directory in ["p", "q"] {
        std::fs::create_dir_all(source.join(directory)).expect("source directory");
        std::fs::create_dir_all(outside.join(directory)).expect("outside directory");
    }
    std::fs::create_dir(destination.as_path()).expect("destination");
    std::fs::write(source.join("p/f"), b"first").expect("f");
    std::fs::write(source.join("q/g"), b"second").expect("g");
    run(source.as_path(), destination.as_path(), COPY);
    for file in ["p/f", "q/g"] {
        std::fs::write(source.join(file), b"a newer source").expect("edit");
        set_mtime(source.join(file).as_path(), 2_000_000_000);
        std::fs::write(outside.join(file), b"outside, must stay").expect("outside file");
        set_mtime(outside.join(file).as_path(), 1_000_000_000);
    }
    let swap_destination: PathBuf = destination.clone();
    let swap_outside: PathBuf = outside.clone();
    let swap = move |call: u32| {
        if call == 0 {
            for directory in ["p", "q"] {
                std::fs::rename(swap_destination.join(directory), swap_destination.join(format!("{directory}.moved"))).expect("move aside");
                symlink(swap_outside.join(directory), swap_destination.join(directory)).expect("link in its place");
            }
        }
    };
    let platform: Hooked = Hooked { inner: PlainPlatform::new(), on_clone: Box::new(swap), calls: std::cell::Cell::new(0), links_fail: false };
    let stats: SyncStats = sync_tree(&platform, source.as_path(), destination.as_path(), UPDATE, &Interrupt::never()).expect("pass");
    for file in ["p/f", "q/g"] {
        let metadata: std::fs::Metadata = std::fs::metadata(outside.join(file)).expect("outside file");
        assert_eq!(std::fs::read(outside.join(file)).expect("content"), b"outside, must stay", "{file}");
        assert_eq!(metadata.mtime(), 1_000_000_000, "{file}: the time of a file outside the tree is not touched");
    }
    assert_eq!(stats.conflicts, vec![PathBuf::from("p"), PathBuf::from("q")]);
    remove(parent.as_path());
}

/// A temporary entry left by a killed pass is removed by the next one,
/// and a source entry of the same name is copied.
#[test]
fn a_leftover_temporary_entry_is_removed_and_its_name_in_the_source_is_respected() {
    let (parent, source, destination) = trees("sync-leftover");
    std::fs::write(destination.join(PARTIAL_NAME), b"half a file").expect("leftover");
    run(source.as_path(), destination.as_path(), UPDATE);
    assert!(!destination.join(PARTIAL_NAME).exists());
    std::fs::write(source.join(PARTIAL_NAME), b"a real file with that name").expect("source file");
    std::fs::write(source.join("zz-after"), b"x").expect("another file");
    run(source.as_path(), destination.as_path(), COPY);
    assert_eq!(std::fs::read(destination.join(PARTIAL_NAME)).expect("copied"), b"a real file with that name");
    assert_eq!(std::fs::read(destination.join("zz-after")).expect("copied"), b"x");
    remove(parent.as_path());
}

/// Set a user extended attribute;
/// false (and a loud line) where the filesystem refuses.
fn set_user_attribute(path: &Path, name: &str, value: &[u8]) -> bool {
    let (path_text, name_text) = (std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("path"), std::ffi::CString::new(name).expect("name"));
    // SAFETY: both strings are NUL-terminated and the value is readable for its length.
    let set: libc::c_int = unsafe { libc::lsetxattr(path_text.as_ptr(), name_text.as_ptr(), value.as_ptr().cast(), value.len(), 0) };
    if set != 0 {
        eprintln!("XATTR CONTROL NOT RUN: this filesystem refuses extended attributes ({})", std::io::Error::last_os_error());
    }
    return set == 0;
}

/// The value of an extended attribute,
/// or nothing when it is absent.
fn read_attribute(path: &Path, name: &str) -> Option<Vec<u8>> {
    let (path_text, name_text) = (std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("path"), std::ffi::CString::new(name).expect("name"));
    let mut value: Vec<u8> = vec![0; 512];
    // SAFETY: the buffer is writable for its length and the strings are NUL-terminated.
    let read: isize = unsafe { libc::lgetxattr(path_text.as_ptr(), name_text.as_ptr(), value.as_mut_ptr().cast(), value.len()) };
    if read < 0 {
        return None;
    }
    value.truncate(usize::try_from(read).expect("length"));
    return Some(value);
}

/// Remove an extended attribute from a path.
fn remove_attribute(path: &Path, name: &str) {
    let (path_text, name_text) = (std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("path"), std::ffi::CString::new(name).expect("name"));
    // SAFETY: both strings are NUL-terminated.
    let removed: libc::c_int = unsafe { libc::lremovexattr(path_text.as_ptr(), name_text.as_ptr()) };
    assert_eq!(removed, 0, "remove {name}");
}

/// The bytes of a POSIX ACL (`system.posix_acl_access`) that gives a named user `named_user` the permissions `named_bits`,
///  the owner `owner_bits`,
/// the group and the mask `group_bits` and the others nothing.
fn acl_with_named_user(named_user: u32, named_bits: u16, owner_bits: u16, group_bits: u16) -> Vec<u8> {
    let mut bytes: Vec<u8> = 2u32.to_le_bytes().to_vec();
    for (tag, bits, id) in [(1u16, owner_bits, u32::MAX), (2, named_bits, named_user), (4, group_bits, u32::MAX), (0x10, group_bits, u32::MAX), (0x20, 0, u32::MAX)] {
        bytes.extend_from_slice(&tag.to_le_bytes());
        bytes.extend_from_slice(&bits.to_le_bytes());
        bytes.extend_from_slice(&id.to_le_bytes());
    }
    return bytes;
}

/// A change of a user attribute or of a named ACL entry alone (same size,
/// time and mode) reaches the copy on the next
///  pass,
/// and an attribute the source lost is removed from the copy.
#[test]
fn a_changed_or_removed_extended_attribute_or_acl_reaches_the_copy() {
    let (parent, source, destination) = trees("sync-xattr-delta");
    let file: PathBuf = source.join("file.txt");
    if !set_user_attribute(file.as_path(), "user.cli-git-test", b"one") || !set_user_attribute(file.as_path(), "user.cli-git-removed", b"gone soon") {
        remove(parent.as_path());
        return;
    }
    run(source.as_path(), destination.as_path(), COPY);
    assert_eq!(read_attribute(destination.join("file.txt").as_path(), "user.cli-git-test"), Some(b"one".to_vec()));
    assert!(set_user_attribute(file.as_path(), "user.cli-git-test", b"two"));
    remove_attribute(file.as_path(), "user.cli-git-removed");
    set_mtime(file.as_path(), 1_000_000_000);
    run(source.as_path(), destination.as_path(), COPY);
    assert_eq!(read_attribute(destination.join("file.txt").as_path(), "user.cli-git-test"), Some(b"two".to_vec()), "a changed attribute is copied");
    assert_eq!(read_attribute(destination.join("file.txt").as_path(), "user.cli-git-removed"), None, "a removed attribute is removed");
    // A named ACL entry, then a different named entry with the same mask.
    if !set_user_attribute(file.as_path(), "system.posix_acl_access", acl_with_named_user(4242, 4, 6, 4).as_slice()) {
        remove(parent.as_path());
        return;
    }
    run(source.as_path(), destination.as_path(), COPY);
    assert_eq!(read_attribute(destination.join("file.txt").as_path(), "system.posix_acl_access"), read_attribute(file.as_path(), "system.posix_acl_access"));
    assert!(set_user_attribute(file.as_path(), "system.posix_acl_access", acl_with_named_user(4343, 4, 6, 4).as_slice()));
    set_mtime(file.as_path(), 1_000_000_000);
    run(source.as_path(), destination.as_path(), COPY);
    assert_eq!(read_attribute(destination.join("file.txt").as_path(), "system.posix_acl_access"), read_attribute(file.as_path(), "system.posix_acl_access"), "a changed named entry is copied");
    remove(parent.as_path());
}

/// An ACL removed from a source directory is removed from the existing copy of the directory.
#[test]
fn an_acl_removed_from_a_directory_is_removed_from_its_copy() {
    let (parent, source, destination) = trees("sync-acl-directory");
    // A directory is searched, not written: owner rwx, named user r-x, group and mask r-x, others none.
    if !set_user_attribute(source.join("dir").as_path(), "system.posix_acl_access", acl_with_named_user(4242, 5, 7, 5).as_slice()) {
        remove(parent.as_path());
        return;
    }
    run(source.as_path(), destination.as_path(), COPY);
    assert!(read_attribute(destination.join("dir").as_path(), "system.posix_acl_access").is_some(), "the ACL was copied");
    remove_attribute(source.join("dir").as_path(), "system.posix_acl_access");
    run(source.as_path(), destination.as_path(), COPY);
    assert_eq!(read_attribute(destination.join("dir").as_path(), "system.posix_acl_access"), None, "the named entry no longer grants access");
    remove(parent.as_path());
}

/// A copy made inside a directory whose group is another than the caller's primary group gets the original's group.
#[test]
fn the_group_of_a_copy_follows_the_original_not_the_directory_it_was_made_in() {
    // SAFETY: `getgid` takes no arguments and cannot fail.
    let primary: u32 = unsafe { libc::getgid() };
    let mut groups: [libc::gid_t; 64] = [0; 64];
    // SAFETY: the buffer holds 64 entries and the call is told so.
    let count: libc::c_int = unsafe { libc::getgroups(64, groups.as_mut_ptr()) };
    let Some(other) = groups[..usize::try_from(count).unwrap_or(0)].iter().copied().find(|group: &u32| { return *group != primary; }) else {
        eprintln!("GROUP CONTROL NOT RUN: this user has no group besides its primary group");
        return;
    };
    let (parent, source, destination) = trees("sync-group");
    let name = std::ffi::CString::new(destination.as_os_str().as_encoded_bytes()).expect("path");
    // SAFETY: `name` is NUL-terminated; -1 leaves the owner alone.
    let changed: libc::c_int = unsafe { libc::chown(name.as_ptr(), u32::MAX, other) };
    assert_eq!(changed, 0, "give the destination the other group");
    std::fs::set_permissions(destination.as_path(), PermissionsExt::from_mode(0o2755)).expect("setgid");
    run(source.as_path(), destination.as_path(), COPY);
    for entry in ["file.txt", "dir", "dir/deeper/leaf"] {
        assert_eq!(std::fs::metadata(destination.join(entry)).expect("copy").gid(), primary, "{entry}");
    }
    remove(parent.as_path());
}

/// A pass over `source` and `destination` that the controls below drive one step at a time.
fn pass_over<'a>(platform: &'a PlainPlatform, source: &'a Path, destination: &'a Path, interrupt: &'a Interrupt) -> Pass<'a> {
    return Pass { platform, source, destination, options: COPY, interrupt, stats: SyncStats::default(), links: HashMap::new() };
}

/// The destination directory of a relative path,
/// held open with the temporary name a pass uses.
fn directory_of(destination: &Path) -> Directory {
    let anchor: Anchor = Anchor::open_below(destination, Path::new("")).expect("open the destination");
    return Directory { anchor, temporary: OsString::from(PARTIAL_NAME) };
}

/// A read that found its entry gone is counted and skipped,
/// a read that failed otherwise is returned,
/// and one that worked
///  counts nothing;
/// the metadata of a listed entry that was removed after the listing is the vanished kind.
#[test]
fn a_vanished_entry_is_counted_and_any_other_failure_is_returned() {
    let mut vanished: u64 = 0;
    assert_eq!(vanished_is_counted(Ok(5u32), &mut vanished).expect("worked"), Some(5));
    assert_eq!(vanished, 0);
    assert_eq!(vanished_is_counted::<u32>(Err(std::io::Error::from(std::io::ErrorKind::NotFound)), &mut vanished).expect("tolerated"), None);
    assert_eq!(vanished, 1);
    for kind in [std::io::ErrorKind::PermissionDenied, std::io::ErrorKind::InvalidData] {
        let error: std::io::Error = vanished_is_counted::<u32>(Err(std::io::Error::from(kind)), &mut vanished).expect_err("returned");
        assert_eq!(error.kind(), kind);
    }
    assert_eq!(vanished, 1, "a failure that is not a vanished entry is not counted");
    let (parent, source, _destination) = trees("sync-vanished-entry");
    let entry: std::fs::DirEntry = std::fs::read_dir(source.as_path()).expect("listing").flatten().find(|item: &std::fs::DirEntry| { return item.file_name() == "file.txt"; }).expect("the file is listed");
    std::fs::remove_file(source.join("file.txt")).expect("remove after the listing");
    assert!(vanished_is_counted(entry.metadata(), &mut vanished).expect("tolerated").is_none());
    assert_eq!(vanished, 2);
    remove(parent.as_path());
}

/// A directory that is gone when its turn comes is counted and adds no work;
/// one that cannot be listed for another reason
///  stops the pass and names the directory.
#[test]
fn a_directory_that_vanished_is_counted_and_one_that_cannot_be_listed_is_an_error() {
    let (parent, source, destination) = trees("sync-vanished-directory");
    let (platform, interrupt): (PlainPlatform, Interrupt) = (PlainPlatform::new(), Interrupt::never());
    let mut pass: Pass<'_> = pass_over(&platform, source.as_path(), destination.as_path(), &interrupt);
    let mut stack: Vec<Work> = Vec::new();
    pass.visit(Path::new("gone"), &mut stack).expect("a vanished directory is tolerated");
    assert_eq!((pass.stats.vanished, stack.len()), (1, 0));
    let error: MigrateError = pass.visit(Path::new("file.txt"), &mut stack).expect_err("a file cannot be listed");
    assert!(matches!(&error, MigrateError::UnreadableDirectory { path, .. } if path.ends_with("file.txt")), "{error}");
    assert_eq!(pass.stats.vanished, 1);
    std::fs::create_dir(destination.join("dir")).expect("the destination directory the parent's visit makes");
    pass.visit(Path::new("dir"), &mut stack).expect("positive control: a directory that is there");
    assert!(!stack.is_empty(), "its subdirectory is queued");
    assert_eq!(pass.stats.vanished, 1);
    remove(parent.as_path());
}

/// A file that cannot be opened is told apart by the reason:
/// gone is counted as vanished,
/// unreadable is listed,
/// and any
///  other failure stops the pass naming the file.
#[test]
fn an_unopenable_source_file_is_vanished_unreadable_or_an_error_by_its_reason() {
    let (parent, source, destination) = trees("sync-open-reasons");
    let (platform, interrupt): (PlainPlatform, Interrupt) = (PlainPlatform::new(), Interrupt::never());
    let directory: Directory = directory_of(destination.as_path());
    let listing: Metadata = std::fs::symlink_metadata(source.join("file.txt")).expect("listing");
    let mut pass: Pass<'_> = pass_over(&platform, source.as_path(), destination.as_path(), &interrupt);
    pass.copy_file(&directory, Path::new("absent"), &listing, None).expect("a vanished file is tolerated");
    assert_eq!((pass.stats.vanished, pass.stats.unreadable.len(), pass.stats.files_copied), (1, 0, 0));
    let error: MigrateError = pass.copy_file(&directory, Path::new("file.txt/inside"), &listing, None).expect_err("a path through a file");
    assert!(error.to_string().contains("open"), "{error}");
    assert_eq!((pass.stats.vanished, pass.stats.unreadable.len()), (1, 0), "neither list takes another reason");
    if !running_as_root() {
        std::fs::write(source.join("secret"), b"s").expect("secret");
        std::fs::set_permissions(source.join("secret"), PermissionsExt::from_mode(0o000)).expect("closed");
        pass.copy_file(&directory, Path::new("secret"), &listing, None).expect("an unreadable file is tolerated");
        assert_eq!((pass.stats.vanished, pass.stats.unreadable.clone()), (1, vec![PathBuf::from("secret")]));
        std::fs::set_permissions(source.join("secret"), PermissionsExt::from_mode(0o600)).expect("open again");
    }
    pass.copy_file(&directory, Path::new("file.txt"), &listing, None).expect("positive control: a readable file is copied");
    assert_eq!(pass.stats.files_copied, 1);
    remove(parent.as_path());
}

/// A file with one name leaves no record of a link behind,
/// and a file with several names leaves exactly one;
/// a stale
///  record (an inode number the source reused) never makes a single-name file a link of an earlier copy.
#[test]
fn only_a_file_with_several_names_is_recorded_and_linked() {
    let (parent, source, destination) = trees("sync-link-records");
    std::fs::write(source.join("first-name"), b"shared bytes").expect("file");
    std::fs::hard_link(source.join("first-name"), source.join("second-name")).expect("second name");
    run(source.as_path(), destination.as_path(), COPY);
    let (platform, interrupt): (PlainPlatform, Interrupt) = (PlainPlatform::new(), Interrupt::never());
    let directory: Directory = directory_of(destination.as_path());
    let metadata = |relative: &str| -> Metadata { return std::fs::symlink_metadata(source.join(relative)).expect("source metadata"); };
    let existing = |relative: &str| -> Option<Metadata> { return Some(std::fs::symlink_metadata(destination.join(relative)).expect("destination metadata")); };
    // Unchanged files: the single name records nothing, the two names record one copy.
    let mut pass: Pass<'_> = pass_over(&platform, source.as_path(), destination.as_path(), &interrupt);
    pass.copy_file(&directory, Path::new("file.txt"), &metadata("file.txt"), existing("file.txt")).expect("unchanged");
    assert_eq!((pass.stats.unchanged, pass.links.len()), (1, 0), "a single-name file records no link");
    pass.copy_file(&directory, Path::new("first-name"), &metadata("first-name"), existing("first-name")).expect("unchanged");
    assert_eq!((pass.stats.unchanged, pass.links.len()), (2, 1), "a two-name file records its copy");
    // A stale record: the source reused an inode number for a single-name file that is not yet in the destination.
    std::fs::write(destination.join("earlier"), b"another file entirely").expect("earlier copy");
    std::fs::write(source.join("new-single"), b"new bytes").expect("new single");
    std::fs::write(source.join("new-pair"), b"pair bytes").expect("new pair");
    std::fs::hard_link(source.join("new-pair"), source.join("new-pair-two")).expect("second name");
    let mut stale: Pass<'_> = pass_over(&platform, source.as_path(), destination.as_path(), &interrupt);
    for name in ["new-single", "new-pair"] {
        let listing: Metadata = metadata(name);
        stale.links.insert((listing.dev(), listing.ino()), vec![PathBuf::from("earlier")]);
        stale.copy_file(&directory, Path::new(name), &listing, None).expect("copy");
    }
    assert_eq!(std::fs::read(destination.join("new-single")).expect("single"), b"new bytes", "not a link of the earlier copy");
    assert_ne!(std::fs::metadata(destination.join("new-single")).expect("single").ino(), std::fs::metadata(destination.join("earlier")).expect("earlier").ino());
    assert_eq!(std::fs::metadata(destination.join("new-pair")).expect("pair").ino(), std::fs::metadata(destination.join("earlier")).expect("earlier").ino(), "positive control: a two-name file is linked to the recorded copy");
    assert_eq!((stale.stats.hard_linked, stale.stats.files_copied), (1, 1));
    remove(parent.as_path());
}
