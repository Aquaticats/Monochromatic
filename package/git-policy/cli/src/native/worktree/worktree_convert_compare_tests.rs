//! Controls for the read-only comparison of the old tree with the installed one.

use super::*;
use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{fixture, remove};

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
