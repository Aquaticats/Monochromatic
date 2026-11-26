//! Controls for the facts a damaged link is judged from,
//! on a real Git registration.

use super::*;
use crate::test_support::{
    fixture,
    git,
    remove,
};
use std::path::PathBuf;

/// What:
///  One real worktree's registration and pointer read as facts.
/// Why:
///  The registration's `gitdir` and the slot's `.git` are Git's own files,
///  and only real Git writes them the way the report compares.
#[test]
fn a_real_registration_names_its_slot_and_keeps_the_pointer() {
    let root: PathBuf = fixture("link-facts");
    let repository: PathBuf = root.join("repository");
    std::fs::create_dir(repository.as_path()).expect("repository");
    git(repository.as_path(), &["init", "--quiet"]);
    git(
        repository.as_path(),
        &["commit", "--quiet", "--allow-empty", "--message", "base"],
    );
    let slot: PathBuf = root.join("one");
    git(
        repository.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--no-checkout",
            slot.to_str().expect("utf-8"),
            "HEAD",
        ],
    );
    let common: PathBuf = repository.join(".git");
    let registration: PathBuf = common.join("worktrees").join("one");
    let pointer: Vec<u8> = std::fs::read(slot.join(".git")).expect("pointer reads");
    let link: RecordedLink<'_> = RecordedLink {
        slot_path: slot.clone(),
        registration: Some(registration.clone()),
        pointer: Some(pointer.as_slice()),
        clone_dataset: "tank/clones/one",
    };
    let facts: LinkFacts = link_facts_of("one", &link, &CloneMount::Nowhere);
    assert!(facts.registration_stands);
    assert_eq!(facts.registration_names_the_slot, Some(true));
    assert_eq!(facts.pointer, PointerState::Intact);
    assert!(facts.backup);
    assert_eq!(facts.site, PointerSite::Underlay);
    // A pointer the account deleted is the state a repair can put right.
    std::fs::remove_file(slot.join(".git")).expect("pointer removed");
    assert_eq!(
        link_facts_of("one", &link, &CloneMount::Nowhere).pointer,
        PointerState::Missing
    );
    // A registration Git pruned is the state nothing rebuilds.
    std::fs::remove_dir_all(registration.as_path()).expect("registration removed");
    let pruned: LinkFacts = link_facts_of("one", &link, &CloneMount::Nowhere);
    assert!(!pruned.registration_stands);
    assert_eq!(pruned.registration_names_the_slot, None);
    remove(root.as_path());
}
