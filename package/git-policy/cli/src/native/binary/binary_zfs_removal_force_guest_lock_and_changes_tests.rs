//! What:
//!  The arms of the forced removal whose subject is what a person left on the
//!  slot:
//!  a lock on the registration,
//!  and the files a worktree at the slot holds.
//! Why:
//!  The owner answered that two forces delete a person's lock file and then run
//!  the removal,
//!  and that one force or none refuses it naming `git worktree unlock <slot>`
//!  (`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
//!  "Two forces at a person's lock").
//!  Git itself writes the lock,
//!  in the file and the mode the lock design measured,
//!  so the arm lets Git write it rather than writing the bytes itself.
//!  The files at the slot are Git's own gradation,
//!  added by the building round with the owner's veto open:
//!  without force the removal refuses what Git's own removal refuses and names
//!  `--force`,
//!  and one force passes it.
//!  These arms test that gradation as it was built and say so;
//!  they are not a decision of the project's.
//!  They run where `CLI_GIT_ZFS_REMOVAL_FORCE_GUEST` names the guest,
//!  printing `SKIP` everywhere else.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(arm.wrapperGit(['worktree', 'remove', '--force', '--force', slot]).status).toBe(0);
//! ```

use super::zfs_removal_force_guest_fixture::{
    Arm,
    SLOT,
    THE_LOCKS_FILE,
    in_the_guest,
    shown,
};
use super::zfs_sweep_guest::command_ok;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

/// The words a person's lock holds in these arms,
/// which is the reason `git worktree lock` was given.
const THE_LOCK_REASON: &str = "held by a person";

/// The file one arm writes at the slot,
/// which is the tracked file the fixture's checkout committed.
const THE_TRACKED_FILE: &str = "a.txt";

/// What:
///  Arm 5:
///  a person's lock on a real registration.
/// Why:
///  The lock design fixes three facts a person's own machine then holds:
///  the file is `worktrees/<id>/locked`,
///  Git writes the words plus one newline,
///  and P1 measured the mode `0644`.
///  The arm reads all three,
///  then the three counts of force the owner's answer names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(said).toContain(`git worktree unlock ${SLOT}`);
/// ```
#[test]
fn zfs_removal_force_guest_a_persons_lock_arm() {
    if !in_the_guest() {
        return;
    }
    let arm: Arm = Arm::make(8);
    arm.attach_at_the_slot();
    let lock_path: PathBuf = arm.place.registration.join(THE_LOCKS_FILE);
    command_ok(
        "git",
        &[
            "-C",
            shown(arm.checkout.as_path()),
            "worktree",
            "lock",
            "--reason",
            THE_LOCK_REASON,
            shown(arm.place.slot.as_path()),
        ],
    );
    let facts: std::fs::Metadata = std::fs::metadata(lock_path.as_path())
        .expect("the lock Git wrote stands under the registration");
    assert!(
        facts.is_file(),
        "the lock is the file Git writes, at {}",
        lock_path.display()
    );
    assert_eq!(
        facts.mode() & 0o7777,
        0o644,
        "the mode the lock design measured for Git's own lock"
    );
    let words: String =
        std::fs::read_to_string(lock_path.as_path()).expect("the lock's words read");
    assert_eq!(
        words,
        format!("{THE_LOCK_REASON}\n"),
        "Git writes the reason and one newline"
    );
    println!(
        "REMOVAL-FORCE arm=5 lock=git-wrote-it at={} mode=0644 words={THE_LOCK_REASON}",
        lock_path.display()
    );
    for (count, words) in [("none", Vec::<&str>::new()), ("one", vec!["--force"])] {
        let mut arguments: Vec<&str> = vec!["worktree", "remove"];
        arguments.extend(words);
        arguments.push(shown(arm.place.slot.as_path()));
        let answer: std::process::Output = arm.wrapper_git(arguments.as_slice());
        let said: String = Arm::said(&answer);
        println!("REMOVAL-FORCE arm=5 count={count} said={said}");
        assert_eq!(
            answer.status.code(),
            Some(1),
            "with {count} force a person's lock refuses the removal: {said}"
        );
        assert!(
            said.contains(
                "holds a lock this add cannot attribute to itself (its words are `held by a person`)"
            ),
            "the refusal reads the lock's own words: {said}"
        );
        assert!(
            said.contains(&format!("git worktree unlock {SLOT}")),
            "the refusal names the command that unlocks it: {said}"
        );
        assert!(
            said.contains(&format!("git worktree remove --force --force {SLOT}")),
            "the refusal names the two forces that pass it: {said}"
        );
        assert_eq!(
            std::fs::read_to_string(lock_path.as_path()).expect("the lock still reads"),
            format!("{THE_LOCK_REASON}\n"),
            "a refusal left the person's lock as Git wrote it"
        );
        assert!(arm.exists(arm.clone.as_str()), "the clone still stands");
        assert_eq!(arm.plan_entries(), 0, "no plan was written");
    }
    let sweeper: std::process::Child = arm.start_the_sweeper();
    let two: std::process::Output = arm.wrapper_git(&[
        "worktree",
        "remove",
        "--force",
        "--force",
        shown(arm.place.slot.as_path()),
    ]);
    arm.stop_the_sweeper(sweeper);
    let said: String = Arm::said(&two);
    println!("REMOVAL-FORCE arm=5 two-forces said={said}");
    assert_eq!(
        two.status.code(),
        Some(0),
        "two forces delete the lock and remove the worktree: {said}"
    );
    assert!(
        said.contains(&format!(
            "two forces were given, so this removal deleted the lock at {}",
            lock_path.display()
        )),
        "the report names the lock it deleted: {said}"
    );
    assert!(
        !lock_path.exists(),
        "the lock file is gone from the registration"
    );
    assert!(
        !arm.exists(arm.clone.as_str()),
        "the clone the lock guarded is gone"
    );
    assert!(
        !arm.place.registration.exists(),
        "the registration went with the removal"
    );
    assert!(
        !arm.place.slot.exists(),
        "the slot folder went with the removal"
    );
    println!("REMOVAL-FORCE arm=5 two-forces=lock-deleted-and-worktree-removed");
    arm.tear_down();
}

/// What:
///  Arm 6's modified-file half:
///  a tracked file at the slot holds what its index does not.
/// Why:
///  Git's own removal refuses a worktree holding modified or untracked files
///  while the command carries no force,
///  and the wrapper answers the slot before Git runs,
///  so without this reading it would destroy files Git would have kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(said).toContain('holds modified or untracked files');
/// ```
#[test]
fn zfs_removal_force_guest_a_modified_file_arm() {
    if !in_the_guest() {
        return;
    }
    let arm: Arm = Arm::make(9);
    arm.attach_at_the_slot();
    std::fs::write(
        arm.place.slot.join(THE_TRACKED_FILE),
        "the arm changed a line\n",
    )
    .expect("the slot's tracked file is changed");
    let refused: std::process::Output =
        arm.wrapper_git(&["worktree", "remove", shown(arm.place.slot.as_path())]);
    let said: String = Arm::said(&refused);
    println!("REMOVAL-FORCE arm=6a modified said={said}");
    assert_eq!(
        refused.status.code(),
        Some(1),
        "a worktree holding a modified file refuses the removal without force: {said}"
    );
    assert!(
        said.contains(&format!(
            "the worktree at the slot `{SLOT}` holds modified or untracked files"
        )),
        "the refusal is Git's own reading: {said}"
    );
    assert!(
        said.contains(&format!("git worktree remove --force {SLOT}")),
        "the refusal names the one force that passes it: {said}"
    );
    assert!(arm.exists(arm.clone.as_str()), "the clone still stands");
    assert_eq!(arm.plan_entries(), 0, "no plan was written");
    let sweeper: std::process::Child = arm.start_the_sweeper();
    let one: std::process::Output = arm.wrapper_git(&[
        "worktree",
        "remove",
        "--force",
        shown(arm.place.slot.as_path()),
    ]);
    arm.stop_the_sweeper(sweeper);
    let removed: String = Arm::said(&one);
    println!("REMOVAL-FORCE arm=6a one-force said={removed}");
    assert_eq!(
        one.status.code(),
        Some(0),
        "one force removes a worktree holding a modified file: {removed}"
    );
    assert!(
        !arm.exists(arm.clone.as_str()),
        "the clone and the file it held are gone"
    );
    println!("REMOVAL-FORCE arm=6a modified=refused-without-force removed-with-one");
    arm.tear_down();
}

/// What:
///  Arm 6's untracked-file half:
///  a file at the slot belongs to no index entry.
/// Why:
///  The untracked scan is the second half of Git's own cleanliness proof,
///  and it is the half a modification to a tracked file cannot prove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(said).toContain('holds modified or untracked files');
/// ```
#[test]
fn zfs_removal_force_guest_an_untracked_file_arm() {
    if !in_the_guest() {
        return;
    }
    let arm: Arm = Arm::make(10);
    arm.attach_at_the_slot();
    std::fs::write(arm.place.slot.join("untracked.txt"), "nobody's file\n")
        .expect("the slot holds an untracked file");
    let refused: std::process::Output =
        arm.wrapper_git(&["worktree", "remove", shown(arm.place.slot.as_path())]);
    let said: String = Arm::said(&refused);
    println!("REMOVAL-FORCE arm=6b untracked said={said}");
    assert_eq!(
        refused.status.code(),
        Some(1),
        "a worktree holding an untracked file refuses the removal without force: {said}"
    );
    assert!(
        said.contains(&format!(
            "the worktree at the slot `{SLOT}` holds modified or untracked files"
        )),
        "the refusal is Git's own reading: {said}"
    );
    assert!(arm.exists(arm.clone.as_str()), "the clone still stands");
    let sweeper: std::process::Child = arm.start_the_sweeper();
    let one: std::process::Output = arm.wrapper_git(&[
        "worktree",
        "remove",
        "--force",
        shown(arm.place.slot.as_path()),
    ]);
    arm.stop_the_sweeper(sweeper);
    let removed: String = Arm::said(&one);
    println!("REMOVAL-FORCE arm=6b one-force said={removed}");
    assert_eq!(
        one.status.code(),
        Some(0),
        "one force removes a worktree holding an untracked file: {removed}"
    );
    assert!(
        !arm.exists(arm.clone.as_str()),
        "the clone and the untracked file are gone"
    );
    println!("REMOVAL-FORCE arm=6b untracked=refused-without-force removed-with-one");
    arm.tear_down();
}

/// What:
///  Arm 6's staged-change half:
///  the index at the slot differs from `HEAD` and the files match the index.
/// Why:
///  Git's own cleanliness proof is `update-index --refresh` and `ls-files
///  --others`,
///  neither of which reads `HEAD`,
///  so a staged change alone is not "modified or untracked" to Git either,
///  and the arm records what this route does with it rather than assuming the
///  three kinds of change behave alike
///  (`builtin/worktree.c` at Git `v2.56.1`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git(['-C', slot, 'add', THE_TRACKED_FILE]);
/// expect(arm.wrapperGit(['worktree', 'remove', slot]).status).toBe(0);
/// ```
#[test]
fn zfs_removal_force_guest_a_staged_change_arm() {
    if !in_the_guest() {
        return;
    }
    let arm: Arm = Arm::make(11);
    arm.attach_at_the_slot();
    std::fs::write(
        arm.place.slot.join(THE_TRACKED_FILE),
        "the arm staged this line\n",
    )
    .expect("the slot's tracked file is changed");
    command_ok(
        "git",
        &[
            "-C",
            shown(arm.place.slot.as_path()),
            "add",
            THE_TRACKED_FILE,
        ],
    );
    let staged: std::process::Output = std::process::Command::new("git")
        .args([
            "-C",
            shown(arm.place.slot.as_path()),
            "status",
            "--porcelain",
        ])
        .output()
        .expect("Git's own status of the slot reads");
    let listed: String = String::from_utf8_lossy(staged.stdout.as_slice()).to_string();
    println!("REMOVAL-FORCE arm=6c git-status={}", listed.trim());
    assert!(
        listed.contains("A ") || listed.contains("M "),
        "the slot's only change is staged: {listed}"
    );
    let sweeper: std::process::Child = arm.start_the_sweeper();
    let none: std::process::Output =
        arm.wrapper_git(&["worktree", "remove", shown(arm.place.slot.as_path())]);
    arm.stop_the_sweeper(sweeper);
    let said: String = Arm::said(&none);
    println!("REMOVAL-FORCE arm=6c staged-no-force said={said}");
    assert_eq!(
        none.status.code(),
        Some(0),
        "a staged change alone is neither modified nor untracked, so the removal runs as Git's own would: {said}"
    );
    assert!(
        !arm.exists(arm.clone.as_str()),
        "the clone went with the removal"
    );
    println!("REMOVAL-FORCE arm=6c staged=ahead-of-git-s-own-rule-and-removed");
    arm.tear_down();
}
