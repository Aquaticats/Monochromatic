//! Controls for the pointer's exchange:
//!  the pointer bytes go to the trash's next entry,
//!  the manifest records the copied administration,
//!  one call swaps them,
//!  a resume finds the swap already done,
//!  and a `.git` the manifest does not name refuses.

use super::{
    exchange_the_git_entry,
    prepare_the_exchange,
};
use crate::test_support::{
    fixture,
    remove,
};
use std::path::PathBuf;

/// A clone root holding the copied administration and the pointer bytes.
fn a_clone_root(name: &str) -> (PathBuf, Vec<u8>) {
    let root: PathBuf = fixture(name);
    std::fs::create_dir(root.join(".git")).expect("the copied administration");
    std::fs::write(root.join(".git").join("HEAD"), b"ref: refs/heads/topic\n")
        .expect("a file inside the copy");
    let pointer: Vec<u8> = format!(
        "gitdir: {}\n",
        root.join(".git").join("worktrees").join("topic").display()
    )
    .into_bytes();
    return (root, pointer);
}

#[test]
fn the_exchange_puts_the_pointer_in_the_slot_and_the_copy_in_the_trash() {
    let (root, pointer) = a_clone_root("exchange-one");
    let prepared = prepare_the_exchange(root.as_path(), pointer.as_slice(), None)
        .expect("the exchange is prepared");
    assert!(!prepared.already_swapped);
    let trash_entry: PathBuf = root
        .join(".claude")
        .join("cli-git-trash")
        .join(prepared.manifest.name.as_str())
        .join(prepared.entry.as_str());
    assert_eq!(
        std::fs::read(trash_entry.as_path()).expect("the pointer file"),
        pointer
    );
    assert_eq!(
        prepared.manifest.entries.len(),
        1,
        "the copied administration is recorded"
    );
    exchange_the_git_entry(root.as_path(), &prepared).expect("the exchange");
    assert_eq!(
        std::fs::read(root.join(".git")).expect("the slot's pointer"),
        pointer,
        "the slot holds the saved pointer"
    );
    assert!(
        trash_entry.is_dir(),
        "the copied administration is in the trash"
    );
    assert!(trash_entry.join("HEAD").is_file(), "the copy is whole");
    remove(root.as_path());
}

#[test]
fn a_resume_finds_the_swap_done_and_does_not_swap_again() {
    let (root, pointer) = a_clone_root("exchange-resume");
    let prepared =
        prepare_the_exchange(root.as_path(), pointer.as_slice(), None).expect("the preparation");
    exchange_the_git_entry(root.as_path(), &prepared).expect("the exchange");
    let carried =
        crate::worktree_clone_add_route::add_fill::the_trash_of_a_filled_clone(root.as_path())
            .expect("the trash reads")
            .expect("one trash stands");
    let again = prepare_the_exchange(root.as_path(), pointer.as_slice(), Some(carried))
        .expect("the resume's preparation");
    assert!(
        again.already_swapped,
        "the resume reads the pointer and the manifest"
    );
    exchange_the_git_entry(root.as_path(), &again).expect("a second exchange does nothing");
    assert_eq!(
        std::fs::read(root.join(".git")).expect("the pointer"),
        pointer
    );
    remove(root.as_path());
}

#[test]
fn a_pointer_the_manifest_does_not_name_refuses() {
    let (root, pointer) = a_clone_root("exchange-unnamed");
    // The slot already holds a pointer file, and no trash names it.
    std::fs::remove_dir_all(root.join(".git")).expect("the copy goes");
    std::fs::write(root.join(".git"), pointer.as_slice()).expect("a pointer no manifest names");
    let refusal: String = prepare_the_exchange(root.as_path(), pointer.as_slice(), None)
        .expect_err("nothing proves whose pointer it is");
    assert!(refusal.contains("does not name"), "{refusal}");
    assert_eq!(
        std::fs::read(root.join(".git")).expect("the pointer"),
        pointer,
        "nothing was changed"
    );
    remove(root.as_path());
}

#[test]
fn a_slot_that_is_not_the_pointer_the_add_saved_refuses() {
    let (root, pointer) = a_clone_root("exchange-foreign");
    let prepared =
        prepare_the_exchange(root.as_path(), pointer.as_slice(), None).expect("the preparation");
    exchange_the_git_entry(root.as_path(), &prepared).expect("the exchange");
    let other: Vec<u8> = b"gitdir: /another/registration\n".to_vec();
    std::fs::write(root.join(".git"), other.as_slice()).expect("another pointer");
    let carried =
        crate::worktree_clone_add_route::add_fill::the_trash_of_a_filled_clone(root.as_path())
            .expect("the trash reads")
            .expect("one trash stands");
    let refusal: String = prepare_the_exchange(root.as_path(), pointer.as_slice(), Some(carried))
        .expect_err("another pointer is refused");
    assert!(
        refusal.contains("not the pointer this add saved"),
        "{refusal}"
    );
    remove(root.as_path());
}
