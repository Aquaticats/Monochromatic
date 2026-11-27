//! What:
//!  The two things a recovery says to a person:
//!  the refusal that names a record another namespace owns and changes nothing,
//!  and the one line that reports a merge it finished,
//!  with nothing said for a record that moved no branch.
//! Why:
//!  A command run only to look at the worktree must not move a branch in silence,
//!  and a record it may not settle must stop it with the record named.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(refused.text).toContain(record); expect(notices).toEqual([line]);
//! ```

use super::super::*;
use super::fixtures::{
    World,
    dead_owner,
    place,
    record_of,
    world,
};
use super::readings::snapshot;
use crate::test_support::{
    git,
    git_text,
    remove,
};
use crate::transaction_owner::TransactionOwner;
use std::path::PathBuf;

use crate::merge_isolation_recovery_records::merge_intents_to_recover;
use crate::merge_isolation_recovery_records::recover_interrupted_merges;
/// A record whose owner is in another PID namespace and older than the patience is neither alive nor dead (owner,
/// 2026-11-04,
/// "Refuse and name it (Recommended)"):
/// recovery stops with an error that names the record,
/// the owner,
/// the namespace,
/// the age and
/// the `rm`,
/// and changes nothing:
/// the record stays,
/// the branch stays,
/// the staged entries and the worktree stay.
#[test]
fn an_old_record_of_another_namespace_is_refused_and_named_and_nothing_changes() {
    let world: World = world("merge-recovery-foreign-expired");
    let mut owner: TransactionOwner = dead_owner("11111111-0000-4000-8000-000000000042");
    owner.owner_pid_namespace = Some(String::from("pid:[4026531999]"));
    owner.owner_pid = 31337;
    let record: PathBuf = record_of(&world, owner, Some("refs/heads/main"));
    std::fs::File::options()
        .write(true)
        .open(record.as_path())
        .expect("open")
        .set_modified(
            std::time::SystemTime::now()
                - crate::pid_namespace::FOREIGN_OWNER_PATIENCE
                - std::time::Duration::from_secs(300),
        )
        .expect("age the record");
    let before = snapshot(&world);
    let head: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let refused =
        recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new())
            .expect_err("an expired foreign record is refused");
    let text: String = refused.0;
    assert!(
        text.contains(record.to_string_lossy().as_ref()),
        "the record file is named: {text}"
    );
    assert!(
        text.contains("31337") && text.contains("pid:[4026531999]"),
        "the owner and its namespace are named: {text}"
    );
    assert!(
        text.contains("minutes old")
            && text.contains("another container")
            && text.contains("rm -- "),
        "age, the live-owner possibility and the rm: {text}"
    );
    assert!(
        text.contains("left running"),
        "the condition covers Git processes the wrapper left running, not only a monochromatic-git-policy-cli command: {text}"
    );
    assert!(record.exists(), "the record is untouched");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), head);
    assert_eq!(snapshot(&world), before);
    assert!(
        merge_intents_to_recover(world.git_dir.as_path()).is_err(),
        "the check that decides whether recovery is needed refuses as well"
    );
    remove(world.parent.as_path());
}

/// Records `[dead A, expired-foreign B]`:
/// recovery refuses before it settles A (sol-1 round 3,
/// P1 C:
/// the refusal said nothing was
/// changed after the first record had been settled).
#[test]
fn an_old_record_of_another_namespace_after_a_dead_one_stops_recovery_before_the_dead_one_is_settled()
 {
    let world: World = world("merge-recovery-foreign-after-dead");
    let dead: PathBuf = record_of(
        &world,
        dead_owner("00000000-0000-4000-8000-000000000001"),
        Some("refs/heads/main"),
    );
    let mut owner: TransactionOwner = dead_owner("ffffffff-0000-4000-8000-000000000002");
    owner.owner_pid_namespace = Some(String::from("pid:[4026531999]"));
    owner.owner_pid = 31337;
    let foreign: PathBuf = record_of(&world, owner, Some("refs/heads/main"));
    std::fs::File::options()
        .write(true)
        .open(foreign.as_path())
        .expect("open")
        .set_modified(
            std::time::SystemTime::now()
                - crate::pid_namespace::FOREIGN_OWNER_PATIENCE
                - std::time::Duration::from_secs(300),
        )
        .expect("age the record");
    assert!(
        dead.file_name() < foreign.file_name(),
        "the dead owner's record is read first"
    );
    let refused =
        recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut Vec::new())
            .expect_err("an expired foreign record is refused");
    assert!(refused.0.contains("31337"), "{}", refused.0);
    assert!(
        dead.exists() && foreign.exists(),
        "nothing was settled before the refusal"
    );
    remove(world.parent.as_path());
}

/// What:
///  A merge that recovery finishes tells the person so,
///  naming the ref and the commit,
///  and a merge it discards tells them nothing.
/// Why:
///  A command run only to look at the worktree must not move the branch onto a merge commit in silence
///  (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 1);
/// the notice is the one record of that change outside the debug log,
/// and it goes to standard error alone,
/// because scripts read the command's standard output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = []; await recoverInterruptedMerges(place, gitDir, notices); expect(notices).toEqual([line]); expect(stdout).toBe('');
/// ```
#[test]
fn a_completed_merge_is_reported_with_the_ref_and_the_commit() {
    let world: World = world("merge-recovery-reported");
    git(
        world.repo.as_path(),
        &[
            "read-tree",
            "-m",
            "-u",
            world.old_oid.as_str(),
            world.new_oid.as_str(),
        ],
    );
    let _record: PathBuf = record_of(
        &world,
        dead_owner("dddddddd-0000-4000-8000-00000000000d"),
        Some("refs/heads/main"),
    );
    let mut notices: Vec<u8> = Vec::new();
    let outcomes =
        recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut notices)
            .expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    let said: String = String::from_utf8(notices).expect("the notices are UTF-8");
    assert_eq!(
        said.lines().count(),
        1,
        "one line for the one merge that changed the branch: {said}"
    );
    assert!(said.contains("refs/heads/main"), "the ref is named: {said}");
    assert!(
        said.contains(world.new_oid.as_str()),
        "the merge commit is named: {said}"
    );
    assert!(
        said.starts_with("monochromatic-git-policy-cli: "),
        "the line is the wrapper's: {said}"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        world.new_oid
    );
    remove(world.parent.as_path());
}

/// A record that is only discarded,
/// a live owner's record and a branch that already names the merge change nothing of the branch,
/// so they say nothing.
#[test]
fn a_merge_that_changed_nothing_is_not_reported() {
    let discarded: World = world("merge-recovery-unreported");
    std::fs::write(
        discarded.repo.join("f.txt"),
        b"an edit made while the merge was dead\n",
    )
    .expect("edit");
    let _record: PathBuf = record_of(
        &discarded,
        dead_owner("eeeeeeee-0000-4000-8000-00000000000e"),
        Some("refs/heads/main"),
    );
    let mut notices: Vec<u8> = Vec::new();
    let outcomes = recover_interrupted_merges(
        &place(&discarded),
        discarded.git_dir.as_path(),
        &mut notices,
    )
    .expect("recovery");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Discarded);
    assert_eq!(
        String::from_utf8_lossy(&notices),
        "",
        "a discarded merge was never in the checkout"
    );
    remove(discarded.parent.as_path());

    let landed: World = world("merge-recovery-unreported-landed");
    git(
        landed.repo.as_path(),
        &["merge", "--ff-only", "--quiet", landed.new_oid.as_str()],
    );
    let _landed_record: PathBuf = record_of(
        &landed,
        dead_owner("ffffffff-0000-4000-8000-00000000000f"),
        Some("refs/heads/main"),
    );
    let mut landed_notices: Vec<u8> = Vec::new();
    let landed_outcomes = recover_interrupted_merges(
        &place(&landed),
        landed.git_dir.as_path(),
        &mut landed_notices,
    )
    .expect("recovery");
    assert_eq!(
        landed_outcomes[0].action,
        MergeRecoveryAction::AlreadyLanded
    );
    assert_eq!(
        String::from_utf8_lossy(&landed_notices),
        "",
        "a branch that already names the merge did not move"
    );
    remove(landed.parent.as_path());
}
