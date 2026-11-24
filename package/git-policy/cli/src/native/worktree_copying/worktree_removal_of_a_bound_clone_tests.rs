//! Controls for the refusal of a removal of a slot a whole managed clone is
//! bound to:
//! the two states that refuse,
//! and the states that stay Git's.

use super::*;
use crate::testing::fixture_zfs_binding_record::{publish_under, record_naming};
use crate::test_support::{fixture, git, remove};
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_remove_plan::RemovePlan;
use std::ffi::OsString;
use std::path::PathBuf;

/// One fixture slot with a real registration,
/// its record,
/// and the facts a removal carries.
struct Bound {
    /// The fixture root.
    root: PathBuf,
    /// The Git common directory.
    common: PathBuf,
    /// The slot's path.
    slot: PathBuf,
    /// The bytes Git wrote into the slot.
    pointer: Vec<u8>,
    /// The registration's path.
    registration: PathBuf,
}

/// One real registration of one slot.
fn bound_slot(name: &str) -> Bound {
    let root: PathBuf = fixture(name);
    let repository: PathBuf = root.join("repository");
    std::fs::create_dir(repository.as_path()).expect("repository");
    git(repository.as_path(), &["init", "--quiet"]);
    git(repository.as_path(), &["commit", "--quiet", "--allow-empty", "--message", "base"]);
    let slot: PathBuf = root.join("one");
    git(repository.as_path(), &["worktree", "add", "--quiet", "--no-checkout", slot.to_str().expect("utf-8"), "HEAD"]);
    let common: PathBuf = repository.join(".git");
    let pointer: Vec<u8> = std::fs::read(slot.join(".git")).expect("pointer reads");
    let registration: PathBuf = common.join("worktrees").join("one");
    return Bound { root, common, slot, pointer, registration };
}

/// The facts a `git worktree remove --force` of one slot carries.
fn facts_of(bound: &Bound) -> CopyFacts {
    return CopyFacts {
        identity: WorktreeIdentity::MainWorktree { common_dir: bound.common.clone(), git_dir: bound.common.clone(), worktree_root: bound.root.join("repository") },
        creates: false,
        destination: None,
        add_plan: None,
        remove_plan: Some(RemovePlan { target: bound.slot.clone(), operand: OsString::from("one"), force: true }),
        quiet: false,
    };
}

/// What:
///  A removal of a slot whose record proves a whole worktree is refused,
///  and the report names the slot and the clone a Git removal would orphan.
/// Why:
///  Git's own removal deletes the registration and the underlay pointer while
///  the clone dataset keeps the files,
///  which is the loss this refusal prevents in a release with no clone
///  removal.
#[test]
fn a_whole_worktree_refuses_its_removal() {
    let bound: Bound = bound_slot("removal-of-a-bound-clone");
    publish_under(bound.common.as_path(), &record_naming(Phase::Filled, bound.slot.as_path(), bound.registration.as_path(), Some(bound.pointer.clone())), "one");
    match refuse_the_removal_of_a_bound_clone(&facts_of(&bound)) {
        Some(ForwardedRun::Failed { stderr, code }) => {
            assert_eq!(code, REFUSED_EXIT_CODE);
            assert!(stderr.contains("this release has no removal for a clone"), "{stderr}");
            assert!(stderr.contains("state: known unmounted clone"), "{stderr}");
            assert!(stderr.contains("next safe action:"), "{stderr}");
        }
        other => panic!("a whole worktree's removal refuses: {other:?}"),
    }
    assert!(bound.slot.is_dir(), "nothing was removed");
    assert!(bound.registration.is_dir(), "the registration stands");
    remove(bound.root.as_path());
}

/// What:
///  A slot with no record,
///  and a slot whose record says the add is unfinished,
///  are left to Git.
/// Why:
///  The add's own refusal names `git worktree remove <slot>` as its way
///  forward,
///  and a slot no record names is not this route's to refuse.
#[test]
fn every_other_slot_stays_gits() {
    let unrecorded: Bound = bound_slot("removal-of-an-unrecorded-slot");
    assert!(refuse_the_removal_of_a_bound_clone(&facts_of(&unrecorded)).is_none());
    let adding: Bound = bound_slot("removal-of-a-pending-add");
    publish_under(adding.common.as_path(), &record_naming(Phase::Snapshotted, adding.slot.as_path(), adding.registration.as_path(), Some(adding.pointer.clone())), "one");
    assert!(refuse_the_removal_of_a_bound_clone(&facts_of(&adding)).is_none());
    remove(unrecorded.root.as_path());
    remove(adding.root.as_path());
}

/// What:
///  A run of the removal without a plan,
///  which is every other command,
///  answers nothing.
/// Why:
///  The refusal exists for one removal,
///  and a command that names no directory must not read a slot at all.
#[test]
fn a_command_without_a_removal_plan_answers_nothing() {
    let bound: Bound = bound_slot("removal-of-no-plan");
    publish_under(bound.common.as_path(), &record_naming(Phase::Filled, bound.slot.as_path(), bound.registration.as_path(), Some(bound.pointer.clone())), "one");
    let mut facts: CopyFacts = facts_of(&bound);
    facts.remove_plan = None;
    assert!(refuse_the_removal_of_a_bound_clone(&facts).is_none());
    remove(bound.root.as_path());
}
