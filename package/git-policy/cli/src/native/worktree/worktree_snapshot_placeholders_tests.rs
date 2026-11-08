//! Controls for the stand-in removal of the snapshot rung:
//! which folders are listed,
//! what is a stand-in,
//! and what is left
//!  alone,
//! for a folder the snapshot does not carry,
//! a folder outside the source and a folder reached through a link.

use super::*;
use crate::fs_beneath::open_root;
use crate::test_support::{fixture, remove};

/// A held folder of a fixture that holds an ordinary empty directory,
/// a file,
/// a directory with a file in it and a link,
///  below a held handle of the same fixture root.
fn folder_of_ordinary_entries(root: &Path) -> (File, File) {
    std::fs::create_dir_all(root.join("folder/empty")).expect("an empty directory");
    std::fs::write(root.join("folder/plain"), b"a file").expect("a file");
    std::fs::create_dir_all(root.join("folder/full")).expect("a full directory");
    std::fs::write(root.join("folder/full/inner"), b"someone's data").expect("a file in it");
    std::fs::create_dir_all(root.join("elsewhere/inner")).expect("a directory outside the folder");
    std::os::unix::fs::symlink(root.join("elsewhere"), root.join("folder/link")).expect("a link");
    return (open_root(root).expect("the fixture"), open_root(root.join("folder").as_path()).expect("the folder"));
}

/// The parent of each registered worktree and of the destination is named once,
/// and a worktree outside the source names no
///  folder of the snapshot.
#[test]
fn the_folders_that_hold_worktrees_are_each_named_once_and_only_below_the_source() {
    let source_root: &Path = Path::new("/repo");
    let nested: Vec<PathBuf> = vec![PathBuf::from("/repo/.claude/worktrees/first"), PathBuf::from("/repo/.claude/worktrees/second"), PathBuf::from("/elsewhere/worktree")];
    let destination: &Path = Path::new("/repo/.claude/worktrees/topic");
    assert_eq!(worktree_folders(nested.as_slice(), destination, source_root), vec![PathBuf::from("/repo/.claude/worktrees")]);
    let none: Vec<PathBuf> = Vec::new();
    assert_eq!(worktree_folders(none.as_slice(), Path::new("/repo-parent/topic"), source_root), Vec::<PathBuf>::new(), "a destination beside the source is in no snapshot of it");
}

/// No ordinary entry is a stand-in:
/// an empty directory,
/// a file,
/// a directory that holds a file and a link all stay where they
///  are.
#[test]
fn no_ordinary_entry_is_a_stand_in() {
    let root: PathBuf = fixture("snapshot-placeholders-ordinary");
    let (held, folder) = folder_of_ordinary_entries(root.as_path());
    for name in ["empty", "plain", "full", "link", "missing"] {
        assert!(!is_stand_in(&folder, OsStr::new(name)), "{name} is no stand-in");
    }
    assert_eq!(remove_stand_ins_below(&held, root.as_path(), root.join("folder").as_path()), Ok(0));
    assert_eq!(std::fs::read(root.join("folder/full/inner")).expect("the data"), b"someone's data", "nothing that holds something is removed");
    remove(root.as_path());
}

/// A folder the snapshot does not carry and a folder outside the source are nothing to do.
#[test]
fn a_folder_the_snapshot_does_not_carry_and_a_folder_outside_the_source_are_left_alone() {
    let root: PathBuf = fixture("snapshot-placeholders-absent");
    let (held, _folder) = folder_of_ordinary_entries(root.as_path());
    assert_eq!(remove_stand_ins_below(&held, root.as_path(), root.join("missing/deep").as_path()), Ok(0));
    assert_eq!(remove_stand_ins_below(&held, root.join("elsewhere").as_path(), root.join("folder").as_path()), Ok(0));
    assert!(root.join("folder/empty").exists());
    remove(root.as_path());
}

/// A folder that is reached through a link is an error,
/// so nothing is listed or removed in the tree the link points at.
#[test]
fn a_folder_reached_through_a_link_is_refused() {
    let root: PathBuf = fixture("snapshot-placeholders-link");
    let (held, _folder) = folder_of_ordinary_entries(root.as_path());
    let error: String = remove_stand_ins_below(&held, root.as_path(), root.join("folder/link/inner").as_path()).expect_err("refused");
    assert!(error.contains("link"), "{error}");
    assert!(root.join("elsewhere/inner").exists(), "the tree the link points at is untouched");
    remove(root.as_path());
}
