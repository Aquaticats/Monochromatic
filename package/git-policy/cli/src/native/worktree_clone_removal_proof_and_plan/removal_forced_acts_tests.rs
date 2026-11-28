//! Controls for the acts a removal does only because of a force:
//! the plan file a forced run names for removal,
//! both GUIDs a clone destroyed without a proof is reported by,
//! and the three answers deleting a person's lock makes.

use crate::test_support::{
    fixture,
    remove,
};
use crate::worktree_clone_removal_proof_and_plan::removal_forced_acts::{
    ForcedActs,
    THE_LOCKS_FILE,
    delete_the_persons_lock,
    forced_past_a_plan,
    forced_past_a_proof,
};
use std::path::PathBuf;

/// Nothing forced prints no line and names no file.
#[test]
fn nothing_forced_is_no_line_and_no_file() {
    let nothing: ForcedActs = ForcedActs::nothing();
    assert!(nothing.is_nothing(), "{nothing:?}");
    assert!(nothing.lines.is_empty());
    assert!(nothing.the_unreadable_plan.is_none());
}

/// A plan a forced run went past is named for removal and for the report,
/// in the words the owner's answer gives.
#[test]
fn a_forced_run_past_a_plan_names_the_file_and_the_act() {
    let plan: PathBuf = PathBuf::from(
        "/repo/.git/monochromatic-git-policy-cli/clone-destruction-plans/7828944690444299989",
    );
    let acts: ForcedActs = forced_past_a_plan(plan.as_path());
    assert_eq!(acts.the_unreadable_plan.as_deref(), Some(plan.as_path()));
    assert!(!acts.is_nothing());
    let line: &str = acts.lines.first().expect("one line");
    assert!(line.contains(plan.display().to_string().as_str()), "{line}");
    assert!(
        line.contains("destroys the clone and its snapshots from the pool's own list"),
        "{line}"
    );
}

/// What:
///  A clone destroyed without a proof is reported by the dataset's name and by
///  both GUIDs,
///  because the report is the only trace of it.
#[test]
fn a_clone_destroyed_without_a_proof_is_reported_by_both_guids() {
    let acts: ForcedActs = forced_past_a_proof("tank/wt/one", "7828944690444299989", "12");
    let line: &str = acts.lines.first().expect("one line");
    assert!(line.contains("tank/wt/one"), "{line}");
    assert!(line.contains("`7828944690444299989`"), "{line}");
    assert!(line.contains("`12`"), "{line}");
    assert_eq!(acts.recorded_guid.as_deref(), Some("7828944690444299989"));
    assert_eq!(acts.answered_guid.as_deref(), Some("12"));
}

/// What:
///  Deleting a person's lock removes the file and says what it did,
///  a lock that went between the read and the unlink is the same state,
///  and a lock that cannot be deleted stops the removal.
/// Why:
///  Two forces ask for the deletion,
///  and a file that stays where the removal asked to delete it must not be
///  reported as deleted.
#[test]
fn the_locks_deletion_answers_what_it_did() {
    let root: PathBuf = fixture("forced-lock-deletion");
    let registration: PathBuf = root.join("worktrees/one");
    std::fs::create_dir_all(registration.as_path()).expect("the registration folder");
    let lock: PathBuf = registration.join(THE_LOCKS_FILE);
    std::fs::write(lock.as_path(), b"a person's words\n").expect("the lock writes");
    let deleted: String =
        delete_the_persons_lock(registration.as_path()).expect("the lock is deleted");
    assert!(!lock.exists(), "the lock is gone");
    assert!(
        deleted.contains("deleted the lock")
            && deleted.contains(lock.display().to_string().as_str()),
        "{deleted}"
    );
    let already_gone: String =
        delete_the_persons_lock(registration.as_path()).expect("a missing lock is the same state");
    assert!(
        already_gone.contains("was gone by the time it deleted it"),
        "{already_gone}"
    );
    // A directory cannot be unlinked, which is the one shape a lock's removal refuses on.
    std::fs::create_dir(lock.as_path()).expect("a directory stands where the lock was");
    let refused = delete_the_persons_lock(registration.as_path())
        .expect_err("a lock that cannot go stops the removal");
    assert!(
        refused.message().contains("could not be deleted"),
        "{refused:?}"
    );
    remove(root.as_path());
}
