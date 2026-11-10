//! Controls for the empty stage a conversion created and did not record:
//! the state a kill between its creation and its journal line leaves,
//! and what is never taken for it.

use super::*;
use crate::fs_beneath::ObjectId;
use crate::test_support::remove;
use crate::worktree_convert_journal::JOURNAL_NAME;

/// A journal at the phase that creates the stage,
/// with no identity recorded,
/// as the kill leaves it.
fn creating_journal() -> Journal {
    return Journal {
        phase: Phase::Creating,
        token: String::from("a"),
        path: b"/x".to_vec(),
        old_identity: ObjectId { device: 1, inode: 2, birth: Some((3, 4)) },
        git_file: b"g".to_vec(),
        new_identity: None,
        synced_ms: None,
    };
}

/// A directory a conversion would name,
/// with the converting name free.
fn worktree(name: &str) -> PathBuf {
    let root: PathBuf = crate::test_support::fixture(name);
    return root;
}

/// An empty object at the converting name of a conversion that recorded no identity is that conversion's own.
#[test]
fn an_empty_stage_this_conversion_created_is_its_own() {
    let root: PathBuf = worktree("unrecorded-stage-empty");
    let candidate: PathBuf = root.join("wt");
    std::fs::create_dir_all(candidate.as_path()).expect("worktree");
    std::fs::create_dir_all(converting_path(candidate.as_path()).as_path()).expect("empty stage");
    assert!(owns_unrecorded_stage(&creating_journal(), candidate.as_path()).expect("answered"));
    assert!(stage_is_this_conversions(&creating_journal(), candidate.as_path()).expect("answered"));
    remove(root.as_path());
}

/// An entry at the converting name is somebody's copy,
/// so it is never taken for the stage a conversion made.
#[test]
fn a_stage_holding_anything_is_not_adopted() {
    let root: PathBuf = worktree("unrecorded-stage-held");
    let candidate: PathBuf = root.join("wt");
    std::fs::create_dir_all(candidate.as_path()).expect("worktree");
    let converting: PathBuf = converting_path(candidate.as_path());
    std::fs::create_dir_all(converting.as_path()).expect("stage");
    std::fs::write(converting.join("theirs"), b"theirs").expect("their file");
    assert!(!owns_unrecorded_stage(&creating_journal(), candidate.as_path()).expect("answered"));
    remove(root.as_path());
}

/// A journal past the phase that creates the stage owns nothing by this rule,
/// and neither does one whose recorded identity is not the object at the name.
#[test]
fn another_phase_or_another_object_is_not_adopted() {
    let root: PathBuf = worktree("unrecorded-stage-phase");
    let candidate: PathBuf = root.join("wt");
    std::fs::create_dir_all(candidate.as_path()).expect("worktree");
    std::fs::create_dir_all(converting_path(candidate.as_path()).as_path()).expect("empty stage");
    let synced: Journal = Journal { phase: Phase::Synced, ..creating_journal() };
    assert!(!owns_unrecorded_stage(&synced, candidate.as_path()).expect("answered"));
    // A journal that recorded an identity owns the stage with that identity and not the empty one the name holds.
    let recorded: Journal = Journal {
        phase: Phase::Creating,
        new_identity: Some(ObjectId { device: 1, inode: 99, birth: Some((3, 4)) }),
        ..creating_journal()
    };
    assert!(!owns_unrecorded_stage(&recorded, candidate.as_path()).expect("answered"));
    remove(root.as_path());
}

/// Nothing at the converting name means there is no leftover to adopt,
/// and the conversion creates its own.
#[test]
fn a_free_name_owns_nothing() {
    let root: PathBuf = worktree("unrecorded-stage-free");
    let candidate: PathBuf = root.join("wt");
    std::fs::create_dir_all(candidate.as_path()).expect("worktree");
    assert!(!owns_unrecorded_stage(&creating_journal(), candidate.as_path()).expect("answered"));
    assert!(!stage_is_this_conversions(&creating_journal(), candidate.as_path()).expect("answered"));
    assert!(!candidate.join(".git").exists(), "the fixture holds no journal of its own");
    let _ = JOURNAL_NAME;
    remove(root.as_path());
}
