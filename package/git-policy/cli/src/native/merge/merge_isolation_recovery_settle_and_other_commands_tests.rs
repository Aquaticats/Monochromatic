//! What:
//!  The settle as two commands and as the awaited route meet it:
//!  the awaited settle answering what the synchronous one answers,
//!  a branch that could not be put back and is never reported as put back,
//!  a merge landed in another branch's worktree,
//!  and one another command landed before the write.
//! Why:
//!  Two commands work on one worktree at once,
//!  so a settle must say what it did and never claim a move it did not make.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(await settleAwaiting(...)).toEqual(settle(...)); expect(notices.join('')).not.toContain('put back');
//! ```

use super::super::*;
use super::fixtures::{
    World,
    dead_owner,
    deferred_record_of,
    place,
    record_of,
    world,
};
use crate::merge_isolation_journal::parse_merge_intent;
use crate::test_support::{
    git,
    git_context,
    git_text,
    remove,
};
use std::path::PathBuf;

use crate::merge_isolation_recovery_records::recover_interrupted_merges;
/// What:
///  The awaited settle decides what the synchronous one decides,
/// and the synchronous one is refused from inside the runtime.
/// Why:
///   A task of the runtime is the caller the awaited face exists for,
/// so the two faces must agree on the action,
///  the notices,
/// the checkout and the record,
/// and the refusal of the synchronous face is the reason the awaited one is not optional.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const before = settleFailedMerge(place, record, intent); const after = await runOnRuntime(() => settleFailedMergeAwaiting(place, record, intent)); expect(after).toEqual(before);
/// ```
#[test]
fn the_awaited_settle_answers_what_the_synchronous_one_answers() {
    // One world per face, built the same way, with Git having written part of the merge.
    let synchronous: World = world("merge-recovery-two-faces-synchronous");
    std::fs::write(synchronous.repo.join("f.txt"), b"f.txt feature\n").expect("the file Git wrote");
    std::fs::remove_file(synchronous.repo.join("gone.txt")).expect("the file Git removed");
    let synchronous_record: PathBuf = record_of(
        &synchronous,
        dead_owner("97979797-0000-4000-8000-000000000097"),
        Some("refs/heads/main"),
    );
    let synchronous_intent: MergeIntent = parse_merge_intent(
        std::fs::read(synchronous_record.as_path())
            .expect("read")
            .as_slice(),
    )
    .expect("intent");
    let mut synchronous_said: Vec<u8> = Vec::new();
    let synchronous_action: MergeRecoveryAction = settle_failed_merge(
        &place(&synchronous),
        synchronous_record.as_path(),
        &synchronous_intent,
        &mut synchronous_said,
    )
    .expect("settled");
    let synchronous_head: String = git_text(synchronous.repo.as_path(), &["rev-parse", "main"]);
    let synchronous_status: String =
        git_text(synchronous.repo.as_path(), &["status", "--porcelain=v1"]);

    let awaited: World = world("merge-recovery-two-faces-awaited");
    std::fs::write(awaited.repo.join("f.txt"), b"f.txt feature\n").expect("the file Git wrote");
    std::fs::remove_file(awaited.repo.join("gone.txt")).expect("the file Git removed");
    let awaited_record: PathBuf = record_of(
        &awaited,
        dead_owner("97979797-0000-4000-8000-000000000098"),
        Some("refs/heads/main"),
    );
    let awaited_intent: MergeIntent = parse_merge_intent(
        std::fs::read(awaited_record.as_path())
            .expect("read")
            .as_slice(),
    )
    .expect("intent");
    let mut awaited_said: Vec<u8> = Vec::new();
    let awaited_action: MergeRecoveryAction = crate::lazy_runtime::run_on_runtime(async {
        return settle_failed_merge_awaiting(
            &place(&awaited),
            awaited_record.as_path(),
            &awaited_intent,
            &mut awaited_said,
        )
        .await;
    })
    .expect("the runtime runs the settle")
    .expect("settled");

    assert_eq!(
        awaited_action, synchronous_action,
        "the two faces decide the same"
    );
    assert_eq!(
        awaited_action,
        MergeRecoveryAction::Completed,
        "the merge the files show is completed"
    );
    assert_eq!(
        String::from_utf8(awaited_said).expect("UTF-8"),
        String::from_utf8(synchronous_said).expect("UTF-8"),
        "the two faces say the same"
    );
    assert_eq!(
        git_text(synchronous.repo.as_path(), &["rev-parse", "main"]),
        synchronous_intent.new_oid,
        "the synchronous face moves the branch to the merge"
    );
    assert_eq!(
        git_text(awaited.repo.as_path(), &["rev-parse", "main"]),
        awaited_intent.new_oid,
        "the awaited face moves the branch to its own merge"
    );
    // Each fixture names its own merge, and the two need not be the same commit.
    let _ = synchronous_head;
    assert_eq!(
        git_text(awaited.repo.as_path(), &["status", "--porcelain=v1"]),
        synchronous_status,
        "the same checkout"
    );
    assert!(
        !awaited_record.exists(),
        "the record goes once the merge landed"
    );
    remove(synchronous.parent.as_path());
    remove(awaited.parent.as_path());

    // The positive control: the synchronous face registers its step's pipes by entering the runtime,
    //  which a task cannot do, so a task that reached it would settle nothing.
    let refused_world: World = world("merge-recovery-two-faces-refused");
    let refused_record: PathBuf = record_of(
        &refused_world,
        dead_owner("97979797-0000-4000-8000-000000000099"),
        Some("refs/heads/main"),
    );
    let refused_intent: MergeIntent = parse_merge_intent(
        std::fs::read(refused_record.as_path())
            .expect("read")
            .as_slice(),
    )
    .expect("intent");
    let refused: MergeRefusal = crate::lazy_runtime::run_on_runtime(async {
        let mut said: Vec<u8> = Vec::new();
        return settle_failed_merge(
            &place(&refused_world),
            refused_record.as_path(),
            &refused_intent,
            &mut said,
        );
    })
    .expect("the runtime runs the call")
    .expect_err("a task cannot settle through the synchronous face");
    assert!(
        refused.error.0.contains("was asked for from inside itself"),
        "the refusal names what it is: {}",
        refused.error.0
    );
    assert!(refused_record.exists(), "nothing of the record was touched");
    remove(refused_world.parent.as_path());
}

/// A recovery place whose real Git is `shim`.
fn place_with_git(
    world: &World,
    shim: PathBuf,
) -> RecoveryPlace {
    let mut context: crate::transaction_git::GitContext = git_context();
    context.real_git = shim;
    return RecoveryPlace {
        git: context,
        cwd: world.repo.clone(),
        common_dir: world.git_dir.clone(),
        git_dir: world.git_dir.clone(),
    };
}

/// What:
///  A branch move that succeeded but could not be put back is reported as that,
///  and never as a branch put back where it was.
/// Why:
///  The undo's failure only reached the debug log,
/// and the refusal that followed said the branch was restored,
///  so a person read a successful restoration while `main` still named the merge,
/// and the next command removed the record
/// (`SPEC.md`,
/// "Every wait has a bound",
/// and the owner's "never discard user data";
/// a cross-vendor review's second finding).
/// The shim leaves a concurrent holder's own ref lock behind after the branch move,
/// which no other test can place in the instant before the undo.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = deferredRecord(); hooks.set('before-the-branch-write', () => git(['symbolic-ref', 'HEAD', 'refs/heads/other'])); const refused = await recover({ realGit: lockingShim }).catch(e => e); expect(revParse('main')).toBe(newOid); expect(refused.message).not.toContain('was put back where it was');
/// ```
#[cfg(unix)]
#[test]
fn a_branch_that_could_not_be_put_back_is_never_reported_as_put_back() {
    let world: World = world("merge-recovery-undo-fails");
    let record: PathBuf = deferred_record_of(
        &world,
        "43434343-0000-4000-8000-000000000043",
        "refs/cli-git/kept/43",
    );
    git(world.repo.as_path(), &["branch", "--quiet", "other"]);
    let lock: PathBuf = world.git_dir.join("refs/heads/main.lock");
    let shim: PathBuf = world.parent.join("locking-git");
    crate::test_support::executable(
        shim.as_path(),
        format!(
            "#!/bin/sh\nif [ \"$1\" = \"update-ref\" ] && [ \"$2\" = \"-m\" ]; then\n  {real} \"$@\"\n  status=$?\n  if [ \"$status\" -eq 0 ]; then\n    : > '{lock}'\n  fi\n  exit \"$status\"\nfi\nexec {real} \"$@\"\n",
            real = crate::test_support::REAL_GIT,
            lock = lock.display()
        )
        .as_bytes(),
    );
    let place: RecoveryPlace = place_with_git(&world, shim);
    let repository: PathBuf = world.repo.clone();
    crate::service_fault::hooks::set(
        "before-the-branch-write",
        std::rc::Rc::new(move || {
            git(
                repository.as_path(),
                &["symbolic-ref", "HEAD", "refs/heads/other"],
            );
        }),
    );
    let mut said: Vec<u8> = Vec::new();
    let attempted = recover_interrupted_merges(&place, world.git_dir.as_path(), &mut said);
    crate::service_fault::hooks::clear();
    let refused = attempted.expect_err("a branch that could not be put back stops the command");
    assert!(
        !refused.0.contains("The branch was put back where it was"),
        "it never claims the branch was put back: {}",
        refused.0
    );
    assert!(
        refused.0.contains("refs/heads/main"),
        "the branch that still names the merge is named: {}",
        refused.0
    );
    assert!(
        refused.0.contains("still names the merge"),
        "it says where the branch is: {}",
        refused.0
    );
    assert!(
        refused.0.contains("refs/heads/other"),
        "the branch the worktree switched to is named: {}",
        refused.0
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        world.new_oid,
        "the branch still names the merge"
    );
    assert!(lock.is_file(), "the other command still holds the ref");
    assert!(record.exists(), "the record is kept");
    remove(world.parent.as_path());
}

/// What:
///  A merge that landed in a checkout another branch now owns keeps its record and is reported,
///  instead of losing the record that names the merge's installed entries and files.
/// Why:
///  The branch resolved to the merge commit took the record and the side ref away before asking which branch
/// owned the checkout,
///  so a worktree switched after the install lost the only record of what the merge left staged in it
/// (`SPEC.md`,
/// "Every wait has a bound",
/// and the owner's "never discard user data";
/// a cross-vendor review's second finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readTree(repo, old, merged, {updateWorktree: true}); updateRef('main', merged, old); git(['symbolic-ref', 'HEAD', 'refs/heads/other']); const record = recordOf(deadOwner); await expect(recover(...)).rejects; expect(exists(record)).toBe(true);
/// ```
#[test]
fn a_landed_merge_in_another_branchs_worktree_keeps_its_record() {
    let world: World = world("merge-recovery-landed-elsewhere");
    // The merge landed and its index entries and files are in the checkout, which another command then switched.
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
    git(world.repo.as_path(), &["branch", "--quiet", "other"]);
    git(
        world.repo.as_path(),
        &[
            "update-ref",
            "refs/heads/main",
            world.new_oid.as_str(),
            world.old_oid.as_str(),
        ],
    );
    git(
        world.repo.as_path(),
        &["symbolic-ref", "HEAD", "refs/heads/other"],
    );
    let record: PathBuf = record_of(
        &world,
        dead_owner("45454545-0000-4000-8000-000000000045"),
        Some("refs/heads/main"),
    );
    let mut said: Vec<u8> = Vec::new();
    let refused = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said)
        .expect_err("a landed merge in another branch's checkout stops the command");
    assert!(
        refused.0.contains("refs/heads/other"),
        "the branch the worktree has is named: {}",
        refused.0
    );
    assert!(
        refused.0.contains("refs/heads/main"),
        "the branch that names the merge is named: {}",
        refused.0
    );
    assert!(
        refused.0.contains("index entries and files"),
        "it says what stays in this worktree: {}",
        refused.0
    );
    assert!(
        record.exists(),
        "the record that names the installed merge is kept"
    );
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        world.new_oid,
        "the branch still names the merge"
    );
    remove(world.parent.as_path());
}

/// What:
///  A merge another command landed between this command's read of the branch and its write is not a failure,
///  and the record is not said to be kept after that command removed it.
/// Why:
///  Nothing claims a dead record,
/// so two commands recover the same one,
///  and the second command's compare-and-swap fails on a branch that already names the merge:
/// it reported a partly installed merge and a kept record that was gone,
///  and whether a command failed turned on the schedule
/// (a cross-vendor review's third finding).
/// The seam is the instant before this command's branch write,
/// which no other test can reach.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = deferredRecord(); hooks.set('before-the-branch-write', () => { git(['update-ref', 'main', new, old]); rm(record) }); const outcomes = await recover(...); expect(outcomes[0].action).toBe('completed');
/// ```
#[test]
fn a_merge_another_command_landed_before_the_write_is_not_a_failure() {
    let world: World = world("merge-recovery-landed-in-the-window");
    let record: PathBuf = deferred_record_of(
        &world,
        "46464646-0000-4000-8000-000000000046",
        "refs/cli-git/kept/46",
    );
    let repository: PathBuf = world.repo.clone();
    let removed: PathBuf = record.clone();
    let old_oid: String = world.old_oid.clone();
    let new_oid: String = world.new_oid.clone();
    crate::service_fault::hooks::set(
        "before-the-branch-write",
        std::rc::Rc::new(move || {
            git(
                repository.as_path(),
                &[
                    "update-ref",
                    "refs/heads/main",
                    new_oid.as_str(),
                    old_oid.as_str(),
                ],
            );
            std::fs::remove_file(removed.as_path()).expect("the other command removes its record");
        }),
    );
    let mut said: Vec<u8> = Vec::new();
    let attempted = recover_interrupted_merges(&place(&world), world.git_dir.as_path(), &mut said);
    crate::service_fault::hooks::clear();
    let outcomes =
        attempted.expect("a merge that landed must not fail the command that finds it landed");
    assert_eq!(outcomes[0].action, MergeRecoveryAction::Completed);
    assert_eq!(
        git_text(world.repo.as_path(), &["rev-parse", "main"]),
        world.new_oid,
        "the branch names the merge"
    );
    let text: String = String::from_utf8(said).expect("the notices are UTF-8");
    assert!(
        !text.contains("was kept"),
        "nothing claims a record that is gone: {text}"
    );
    remove(world.parent.as_path());
}
