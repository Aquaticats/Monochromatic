//! What:
//!  The controls of the pass's one look at the plan folder:
//!  the absent folder,
//!  the folder's names in one order,
//!  a path that is no folder,
//!  and the empty folder's removal.
//! Why:
//!  The healthy state costs one `stat` and no listing,
//!  and a listing that dropped or reordered a name would make the pass read another order on every machine.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(TheLivePlansFolder.look(folder)).toEqual({ kind: 'no-folder' });
//! ```

use crate::testing::test_support::fixture;
use crate::worktree_clone_removal_finish_pass::the_plans_folder::{FolderLook, TheLivePlansFolder, ThePlansFolder, remove_the_folder_when_empty};
use std::path::PathBuf;

#[test]
fn a_folder_that_is_not_there_is_an_absence_and_not_an_error() {
    let root: PathBuf = fixture("finish-pass-folder-absent");
    let answer: FolderLook = TheLivePlansFolder.look(root.join("clone-destruction-plans").as_path());
    assert_eq!(answer, FolderLook::NoFolder, "nothing at the folder's path is the healthy state");
}

#[test]
fn a_folder_that_stands_answers_its_names_in_one_order() {
    let root: PathBuf = fixture("finish-pass-folder-names");
    let folder: PathBuf = root.join("clone-destruction-plans");
    std::fs::create_dir(folder.as_path()).expect("the folder");
    std::fs::write(folder.join("b").as_path(), b"b").expect("one entry");
    std::fs::write(folder.join("a").as_path(), b"a").expect("another entry");
    let answer: FolderLook = TheLivePlansFolder.look(folder.as_path());
    let FolderLook::Entries(names) = answer else { panic!("a folder that stands answers its names, and this look answered {answer:?}") };
    let spelled: Vec<String> = names.iter().map(|name| return name.to_string_lossy().into_owned()).collect();
    assert_eq!(spelled, vec![String::from("a"), String::from("b")], "the names are sorted, so every machine reads one order");
}

#[test]
fn a_path_that_is_no_folder_answers_what_stands_there() {
    let root: PathBuf = fixture("finish-pass-folder-not-a-folder");
    let folder: PathBuf = root.join("clone-destruction-plans");
    std::fs::write(folder.as_path(), b"a file\n").expect("a file at the folder's path");
    let answer: FolderLook = TheLivePlansFolder.look(folder.as_path());
    let FolderLook::NotAFolder { said } = answer else { panic!("a file at the folder's path is no folder, and this look answered {answer:?}") };
    assert!(said.contains("not a folder"), "the answer says what stands there: {said}");
}

#[test]
fn an_empty_folder_goes_and_one_holding_a_record_stays() {
    let root: PathBuf = fixture("finish-pass-folder-remove");
    let empty: PathBuf = root.join("empty");
    std::fs::create_dir(empty.as_path()).expect("the empty folder");
    assert!(remove_the_folder_when_empty(empty.as_path()), "an empty folder goes");
    assert!(!empty.exists(), "and it is not there afterwards");
    let held: PathBuf = root.join("held");
    std::fs::create_dir(held.as_path()).expect("the folder holding a record");
    std::fs::write(held.join("7828944690444299989").as_path(), b"a plan\n").expect("a record in the folder");
    assert!(!remove_the_folder_when_empty(held.as_path()), "a folder that still holds a record is never removed");
    assert!(held.join("7828944690444299989").exists(), "the record stands");
    assert!(!remove_the_folder_when_empty(root.join("absent").as_path()), "a folder that is not there is not removed twice");
}
