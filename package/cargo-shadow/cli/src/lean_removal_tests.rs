//! The tests of the removal primitive:
//! the identity is checked before the move and again before the unlink,
//! and a swapped entry is never deleted.

use std::{fs, path::PathBuf};

use crate::lean_removal::{Item, Refusal, claim_into_trash, delete_claimed, item_of};
use crate::test_scratch::scratch;

#[test]
fn an_item_is_claimed_and_deleted_and_its_bytes_are_reported() {
    let root = scratch();
    let path: PathBuf = root.path().join("unit");
    fs::create_dir_all(path.join("inner")).expect("directory");
    fs::write(path.join("inner").join("file"), vec![1_u8; 8192]).expect("file");
    let item: Item = item_of(&path).expect("identity");
    let claimed: PathBuf = claim_into_trash(&item, &root.path().join("trash"), 1).expect("claim");
    assert!(!path.exists());
    let bytes: u64 = delete_claimed(&claimed, item.identity).expect("delete");
    assert!(bytes >= 8192, "{bytes}");
    assert!(!claimed.exists());
}

#[test]
fn a_path_swapped_for_another_entry_is_not_claimed() {
    let root = scratch();
    let path: PathBuf = root.path().join("unit");
    fs::create_dir_all(&path).expect("directory");
    let item: Item = item_of(&path).expect("identity");
    fs::remove_dir(&path).expect("remove the judged entry");
    fs::create_dir_all(&path).expect("put another entry in its place");
    fs::write(path.join("precious"), b"user data").expect("data");
    let outcome = claim_into_trash(&item, &root.path().join("trash"), 1);
    assert!(matches!(outcome, Err(Refusal::Changed { .. })), "{outcome:?}");
    assert!(path.join("precious").exists(), "the other entry stays where it is");
}

#[test]
fn a_claimed_entry_that_changed_before_the_unlink_is_not_deleted() {
    let root = scratch();
    let path: PathBuf = root.path().join("unit");
    fs::create_dir_all(&path).expect("directory");
    let item: Item = item_of(&path).expect("identity");
    let claimed: PathBuf = claim_into_trash(&item, &root.path().join("trash"), 1).expect("claim");
    fs::remove_dir(&claimed).expect("another owner removes it");
    fs::create_dir_all(&claimed).expect("and a different entry takes the name");
    fs::write(claimed.join("precious"), b"user data").expect("data");
    let outcome = delete_claimed(&claimed, item.identity);
    assert!(matches!(outcome, Err(Refusal::Changed { .. })), "{outcome:?}");
    assert!(claimed.join("precious").exists());
}

#[test]
fn an_item_another_owner_already_removed_is_gone_not_an_error() {
    let root = scratch();
    let path: PathBuf = root.path().join("unit");
    fs::create_dir_all(&path).expect("directory");
    let item: Item = item_of(&path).expect("identity");
    fs::remove_dir(&path).expect("another owner");
    assert!(matches!(claim_into_trash(&item, &root.path().join("trash"), 1), Err(Refusal::Gone(_))));
}

#[test]
fn a_symbolic_link_is_removed_as_a_link_and_its_target_stays() {
    let root = scratch();
    let target: PathBuf = root.path().join("target-dir");
    fs::create_dir_all(&target).expect("target");
    fs::write(target.join("data"), b"keep").expect("data");
    let link: PathBuf = root.path().join("link");
    std::os::unix::fs::symlink(&target, &link).expect("link");
    let item: Item = item_of(&link).expect("identity");
    let claimed: PathBuf = claim_into_trash(&item, &root.path().join("trash"), 1).expect("claim");
    delete_claimed(&claimed, item.identity).expect("delete");
    assert!(target.join("data").exists(), "the link's target is untouched");
}
