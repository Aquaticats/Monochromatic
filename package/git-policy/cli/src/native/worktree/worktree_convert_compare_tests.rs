//! Controls for the read-only comparison of the old tree with the installed one.

use super::*;
use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::PermissionsExt;

/// Two trees with the same file,
/// link and directory,
/// the second one made by copying the first with its times.
fn twin_trees(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let old: PathBuf = root.join("old");
    let installed: PathBuf = root.join("installed");
    for tree in [&old, &installed] {
        std::fs::create_dir_all(tree.join("dir")).expect("directory");
        std::fs::write(tree.join("dir/file"), b"same\n").expect("file");
        std::os::unix::fs::symlink("dir/file", tree.join("link")).expect("link");
    }
    let mtime = std::fs::metadata(old.join("dir/file")).expect("metadata").modified().expect("mtime");
    std::fs::OpenOptions::new().write(true).open(installed.join("dir/file")).expect("open").set_modified(mtime).expect("same time");
    return (root, old, installed);
}

/// Trees that hold the same entries have no difference,
/// and the installed tree may hold more.
#[test]
fn equal_trees_have_no_difference_and_more_in_the_installed_tree_is_fine() {
    let (root, old, installed) = twin_trees("compare-equal");
    assert_eq!(first_difference(old.as_path(), installed.as_path()).expect("compared"), None);
    std::fs::write(installed.join("added-since"), b"new\n").expect("added");
    assert_eq!(first_difference(old.as_path(), installed.as_path()).expect("compared"), None);
    remove(root.as_path());
}

/// A missing entry,
/// another kind of entry,
/// another size or time,
/// and another link target are each named,
/// and nothing is changed.
#[test]
fn each_kind_of_difference_is_named_and_nothing_is_changed() {
    let (root, old, installed) = twin_trees("compare-differences");
    std::fs::write(installed.join("dir/file"), b"another length\n").expect("rewrite");
    let size = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(size.contains("dir/file") && size.contains("differs"), "{size}");
    assert_eq!(std::fs::read(old.join("dir/file")).expect("old"), b"same\n", "the old tree is unchanged");
    assert_eq!(std::fs::read(installed.join("dir/file")).expect("installed"), b"another length\n", "the installed tree is unchanged");
    std::fs::remove_file(installed.join("dir/file")).expect("remove");
    let missing = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(missing.contains("dir/file") && missing.contains("missing"), "{missing}");
    std::fs::create_dir(installed.join("dir/file")).expect("a directory in its place");
    let kind = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(kind.contains("not the same kind"), "{kind}");
    std::fs::remove_dir(installed.join("dir/file")).expect("remove");
    std::fs::write(installed.join("dir/file"), b"same\n").expect("restore");
    std::fs::remove_file(installed.join("link")).expect("remove link");
    std::os::unix::fs::symlink("elsewhere", installed.join("link")).expect("another target");
    let target = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(target.contains("link") && target.contains("points elsewhere"), "{target}");
    remove(root.as_path());
}

/// What:
///  An edit of the same length whose modification time was put back is a difference,
/// because the contents are what a person changed and the old tree is the copy that holds them.
/// Why:
///   Size and modification time alone cannot see such an edit;
/// the contents of a file are compared where the
///  comparison can decide the deletion of it.
#[test]
fn a_same_size_edit_with_the_time_put_back_is_a_difference() {
    let (root, old, installed) = twin_trees("compare-same-size");
    let file: PathBuf = old.join("dir/file");
    let before: std::fs::Metadata = std::fs::metadata(&file).expect("metadata");
    let bytes: Vec<u8> = std::fs::read(&file).expect("bytes");
    let mut edited: Vec<u8> = bytes.clone();
    edited[0] = b'D';
    std::fs::write(&file, &edited).expect("an edit of the same length");
    std::fs::OpenOptions::new().write(true).open(&file).expect("open the edited file").set_times(std::fs::FileTimes::new().set_modified(before.modified().expect("modified"))).expect("put the time back");
    assert_eq!(std::fs::metadata(&file).expect("metadata").len(), before.len(), "the length is unchanged");
    let difference: String = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(difference.contains("dir/file") && difference.contains("contents"), "{difference}");
    remove(root.as_path());
}

/// What:
///  A mode change of the old root directory itself is a difference.
/// Why:
///   The old root's own metadata supplied only the device number,
/// so a `chmod` of the old tree's top
///  directory was never compared and the tree that held it was deleted.
#[test]
fn a_mode_change_of_the_old_root_is_a_difference() {
    let (root, old, installed) = twin_trees("compare-root-mode");
    std::fs::set_permissions(old.as_path(), std::fs::Permissions::from_mode(0o700)).expect("a narrowed old root");
    let difference: String = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(difference.contains("the old tree itself") && difference.contains("permission bits"), "{difference}");
    remove(root.as_path());
}

/// What:
///  A mode change of a named pipe is a difference.
/// Why:
///   A FIFO enters none of the directory,
/// link and regular-file branches,
/// so its mode,
/// owner,
/// group
///  and carried attributes were never compared and its distinct permissions were lost with the old tree.
#[test]
fn a_mode_change_of_a_named_pipe_is_a_difference() {
    let (root, old, installed) = twin_trees("compare-fifo-mode");
    crate::subvolume_sync_attrs::make_fifo(old.join("queue").as_path(), 0o600).expect("the old queue");
    crate::subvolume_sync_attrs::make_fifo(installed.join("queue").as_path(), 0o644).expect("the installed queue");
    let difference: String = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(difference.contains("queue") && difference.contains("permission bits"), "{difference}");
    remove(root.as_path());
}

/// What:
///  Give `path` the user attribute `user.cli-git-control`,
///  and answer whether the filesystem took it.
/// Why:
///   A filesystem or a mount without user attributes cannot carry the difference this control makes,
/// which says so
///  instead of failing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function giveUserAttribute(path: string): boolean;
/// ```
fn give_user_attribute(path: &Path) -> bool {
    let name = std::ffi::CString::new("user.cli-git-control").expect("a name without NUL");
    let text = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("a path without NUL");
    let value: &[u8] = b"only in the old tree";
    // SAFETY: both strings are NUL-terminated and outlive the call, and `value` is readable for its own length.
    let result: libc::c_int = unsafe { libc::lsetxattr(text.as_ptr(), name.as_ptr(), value.as_ptr().cast(), value.len(), 0) };
    return result == 0;
}

/// Whether the tests run as root,
/// which can give a file another owner.
fn running_as_root() -> bool {
    // SAFETY: `geteuid` takes no arguments and cannot fail.
    return unsafe { libc::geteuid() } == 0;
}

/// A change of the permission bits of a file or of a directory,
/// of the owner or the group,
/// and of a carried
///  extended attribute is each a difference,
/// although the size and modification time stay the same.
#[test]
fn a_change_of_the_mode_an_owner_or_an_attribute_is_a_difference() {
    let (root, old, installed) = twin_trees("compare-attributes");
    let file: PathBuf = old.join("dir/file");
    let original: std::fs::Permissions = std::fs::metadata(installed.join("dir/file")).expect("installed").permissions();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).expect("a narrowed file");
    let bits = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(bits.contains("dir/file") && bits.contains("permission bits"), "{bits}");
    std::fs::set_permissions(&file, original).expect("the same bits again");
    assert_eq!(first_difference(old.as_path(), installed.as_path()).expect("compared"), None, "the control can report no difference");
    std::fs::set_permissions(old.join("dir"), std::fs::Permissions::from_mode(0o700)).expect("a narrowed directory");
    let directory = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(directory.contains("dir differs") && directory.contains("permission bits"), "{directory}");
    std::fs::set_permissions(old.join("dir"), std::fs::Permissions::from_mode(0o755)).expect("the same bits again");
    if !give_user_attribute(&file) {
        eprintln!("XATTR CONTROL NOT RUN: a_change_of_the_mode_an_owner_or_an_attribute_is_a_difference (the filesystem took no user attribute)");
        remove(root.as_path());
        return;
    }
    let attribute = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(attribute.contains("dir/file") && attribute.contains("extended attributes"), "{attribute}");
    assert_eq!(std::fs::read(installed.join("dir/file")).expect("installed"), b"same\n", "the installed tree is unchanged");
    remove(root.as_path());
}

/// A file given another owner in the old tree is a difference.
/// Giving one needs root,
/// so the control says it did not run elsewhere.
#[test]
fn an_owner_change_is_a_difference() {
    if !running_as_root() {
        eprintln!("OWNER CONTROL NOT RUN: an_owner_change_is_a_difference (giving a file another owner needs root)");
        return;
    }
    let (root, old, installed) = twin_trees("compare-owner");
    let file: PathBuf = old.join("dir/file");
    let name = std::ffi::CString::new(file.as_os_str().as_encoded_bytes()).expect("a path without NUL");
    // SAFETY: `name` is NUL-terminated and outlives the call.
    let result: libc::c_int = unsafe { libc::lchown(name.as_ptr(), 65534, 65534) };
    assert_eq!(result, 0, "give the old tree's file another owner");
    let found = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(found.contains("dir/file") && found.contains("owner or group"), "{found}");
    remove(root.as_path());
}

/// A subvolume inside the old tree is named as something the command does not delete,
/// so the old tree is kept with the path.
#[test]
fn btrfs_a_nested_subvolume_is_named_as_not_deleted() {
    let Some(base) = btrfs_fixture("compare-nested") else { return };
    let old: PathBuf = base.join("old");
    let installed: PathBuf = base.join("installed");
    std::fs::create_dir_all(&old).expect("old");
    std::fs::create_dir_all(installed.join("inner")).expect("installed");
    subvolume_create_at(old.join("inner").as_path()).expect("nested subvolume");
    let found = first_difference(old.as_path(), installed.as_path()).expect("compared").expect("a difference");
    assert!(found.contains("inner") && found.contains("subvolume or a mount point") && found.contains("does not delete"), "{found}");
    remove(base.as_path());
}
