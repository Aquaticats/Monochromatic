//! Controls for the folder a worktree folder is kept in when the name it came from is taken:
//! which entries are read as kept,
//! what a kept name may be,
//! and that only the entry this command moved is taken out of the trash.

use super::*;
use crate::fs_beneath::open_root;
use crate::test_support::{fixture, remove};
use crate::worktree_trash::make_trash;
use std::ffi::OsStr;
use std::fs::File;

/// A stage whose trash holds one folder of this command's,
/// with what the move needs to take it out again.
/// Why:
///   The move names the trash's own entry and compares its identity,
/// so a control needs a real trash and a real entry.
fn trash_with_a_folder(root: &Path) -> (File, Trash, ObjectId) {
    std::fs::create_dir(root.join("folder")).expect("a folder to set aside");
    std::fs::write(root.join("folder/keep"), b"someone's data").expect("a file in it");
    let held: File = open_root(root).expect("the stage");
    let trash: Trash = make_trash(&held).expect("the trash");
    let moved: ObjectId = trash.set_aside(&held, OsStr::new("folder"), "d0", Path::new("folder")).expect("the folder set aside");
    return (held, trash, moved);
}

/// The entry this command set aside is taken out of the trash under a kept name,
/// its bytes go with it,
/// the trash entry the manifest would name is free,
/// and the folder is one the cleanup reads as kept.
#[test]
fn the_entry_this_command_moved_is_kept_beside_the_trash() {
    let root: PathBuf = fixture("kept-folders-move");
    let (_held, trash, moved) = trash_with_a_folder(root.as_path());
    let kept: Option<PathBuf> = keep_the_folder(&trash, "d0", moved, Path::new("folder")).expect("the folder is kept");
    let kept_path: PathBuf = kept.expect("a kept folder is answered");
    assert_eq!(std::fs::read(root.join(kept_path.as_path()).join("keep")).expect("the person's file"), b"someone's data", "the bytes go with the folder");
    assert!(std::fs::symlink_metadata(root.join(TRASH_PARENT).join(trash.name()).join("d0")).is_err(), "the trash entry name a manifest would name is free");
    assert_eq!(kept_folders(root.as_path()), vec![root.join(kept_path)], "and the cleanup reads exactly that folder as kept");
    remove(root.as_path());
}

/// An entry that is another object than the one this command set aside is left to whoever put it there.
#[test]
fn an_entry_that_is_not_the_one_set_aside_is_left_where_it_is() {
    let root: PathBuf = fixture("kept-folders-other-object");
    let (_held, trash, moved) = trash_with_a_folder(root.as_path());
    let entry: PathBuf = root.join(TRASH_PARENT).join(trash.name()).join("d0");
    std::fs::remove_dir_all(entry.as_path()).expect("the entry goes");
    std::fs::create_dir(entry.as_path()).expect("another directory at its name");
    let kept: Option<PathBuf> = keep_the_folder(&trash, "d0", moved, Path::new("folder")).expect("nothing is kept");
    assert_eq!(kept, None, "an entry this command did not move is never named as kept");
    assert!(entry.is_dir(), "and it stays where it is");
    remove(root.as_path());
}

/// Nothing is kept when the entry is not in the trash any more.
#[test]
fn an_entry_that_is_gone_leaves_nothing_to_keep() {
    let root: PathBuf = fixture("kept-folders-gone");
    let (_held, trash, moved) = trash_with_a_folder(root.as_path());
    std::fs::remove_dir_all(root.join(TRASH_PARENT).join(trash.name()).join("d0")).expect("the entry goes");
    let kept: Option<PathBuf> = keep_the_folder(&trash, "d0", moved, Path::new("folder")).expect("nothing is kept");
    assert_eq!(kept, None, "a folder that is gone leaves nothing of this command's to keep");
    remove(root.as_path());
}

/// A stage whose trash parent does not exist holds no kept folder.
#[test]
fn a_stage_without_a_trash_parent_holds_no_kept_folder() {
    let root: PathBuf = fixture("kept-folders-absent");
    assert_eq!(kept_folders(root.as_path()), Vec::<PathBuf>::new(), "an absent trash parent is nothing to name");
    remove(root.as_path());
}

/// The scan names the folders this command kept and no trash,
/// entry or person's folder.
#[test]
fn the_scan_names_only_the_folders_this_command_kept() {
    let root: PathBuf = fixture("kept-folders-scan");
    let parent: PathBuf = root.join(TRASH_PARENT);
    std::fs::create_dir_all(parent.as_path()).expect("the trash parent");
    std::fs::create_dir(parent.join(format!("{KEPT_PREFIX}one"))).expect("a kept folder");
    std::fs::create_dir(parent.join("d0")).expect("an entry of a trash");
    std::fs::create_dir(parent.join("0123456789abcdef0123456789abcdef01234567")).expect("a trash");
    std::fs::create_dir(parent.join("a person's folder")).expect("a folder a person made");
    let kept: Vec<PathBuf> = kept_folders(root.as_path());
    assert_eq!(kept.len(), 1, "only the prefixed folder is read as kept: {kept:?}");
    assert!(kept[0].ends_with(format!("{KEPT_PREFIX}one")), "{:?}", kept[0]);
    remove(root.as_path());
}

/// A kept name says which command made it,
/// holds no byte a name cannot,
/// and never repeats.
#[test]
fn a_kept_name_says_which_command_made_it_and_never_repeats() {
    let first: String = kept_name().expect("a name");
    let second: String = kept_name().expect("a name");
    assert!(first.starts_with(KEPT_PREFIX), "{first}");
    let digits: &str = &first[KEPT_PREFIX.len()..];
    assert!(digits.chars().all(|character: char| { return character.is_ascii_hexdigit() && !character.is_ascii_uppercase(); }), "{first}");
    assert_ne!(first, second, "two commands that keep a folder at once never name one folder twice");
}
