//! What:
//!  The arms of the forced removal that begin with a destruction plan whose file
//!  cannot be read:
//!  the settling command beside the removal with no force,
//!  with one force,
//!  with two forces,
//!  and the state the building round chose to refuse,
//!  where the plan cannot be read and the clone is gone.
//! Why:
//!  The project decided that such a plan is refused without force,
//!  that one force goes past it and destroys the clone and its snapshots from
//!  the pool's own list,
//!  and that the settling command prints the plan it finds without acting on one
//!  it cannot read
//!  (`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
//!  "A destruction plan that cannot be read" and "One force").
//!  Each sentence promises a person a way through a state their own machine
//!  reached,
//!  so each count of force is its own arm on its own pool:
//!  a count that destroys is proved by the pool's list,
//!  and a count that refuses is proved by the plan's bytes and the clone still
//!  standing.
//!  They run where `CLI_GIT_ZFS_REMOVAL_FORCE_GUEST` names the guest,
//!  printing `SKIP` everywhere else.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const said = arm.wrapperGit(['worktree', 'remove', '--force', slot]);
//! expect(arm.exists(arm.clone)).toBe(false);
//! ```

use super::zfs_removal_force_guest_fixture::{
    Arm,
    OWN_SNAPSHOT_PREFIX,
    SLOT,
    in_the_guest,
    plant_an_unreadable_plan,
    shown,
};
use std::path::PathBuf;

/// What:
///  Arm 4's no-force half:
///  the settling command and the removal both refuse an unreadable plan,
///  and nothing of the pool changes.
/// Why:
///  The refusal's promise is that nothing was removed and Git did not run,
///  so the arm reads the plan's bytes and the pool's lists after each command
///  rather than the command's words alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(said).toContain('settle-worktree-clone-record');
/// ```
#[test]
fn zfs_removal_force_guest_a_plan_that_cannot_be_read_with_no_force_arm() {
    if !in_the_guest() {
        return;
    }
    let arm: Arm = Arm::make(4);
    arm.attach_at_the_slot();
    arm.snapshot(format!("{OWN_SNAPSHOT_PREFIX}a").as_str());
    let plan: PathBuf = plant_an_unreadable_plan(arm.place.common.as_path(), arm.clone_guid);
    let bytes: Vec<u8> = std::fs::read(plan.as_path()).expect("the plan's bytes read");
    let snapshots: Vec<String> = arm.the_snapshots();
    let none: std::process::Output =
        arm.wrapper_git(&["worktree", "remove", shown(arm.place.slot.as_path())]);
    let said: String = Arm::said(&none);
    println!("REMOVAL-FORCE arm=4a no-force said={said}");
    assert_eq!(
        none.status.code(),
        Some(1),
        "a plan that cannot be read refuses the removal with no force: {said}"
    );
    assert!(
        said.contains("nothing proves what the slot"),
        "the refusal says what it could not prove: {said}"
    );
    assert!(
        said.contains(&format!("the destruction plan at {}", plan.display())),
        "the refusal names the plan's file: {said}"
    );
    assert!(
        said.contains(&format!(
            "git monochromatic-git-policy-cli settle-worktree-clone-record {SLOT} --confirm-removal-of {SLOT}"
        )),
        "the refusal names the settling command: {said}"
    );
    assert!(
        said.contains(&format!("git worktree remove --force {SLOT}")),
        "the refusal names the one force that goes past it: {said}"
    );
    assert_eq!(
        std::fs::read(plan.as_path()).expect("the plan's bytes still read"),
        bytes,
        "a refused removal left the plan's bytes as they were"
    );
    assert_eq!(arm.plan_entries(), 1, "the plan still stands alone");
    assert!(arm.exists(arm.clone.as_str()), "the clone still stands");
    assert_eq!(
        arm.the_snapshots(),
        snapshots,
        "no snapshot went with a refused removal"
    );
    let settled: std::process::Output = arm.wrapper_git(&[
        "monochromatic-git-policy-cli",
        "settle-worktree-clone-record",
        SLOT,
        "--confirm-removal-of",
        SLOT,
        "--confirm-clone-guid",
        last_four(arm.clone_guid.file_name().as_str()).as_str(),
    ]);
    let answered: String = Arm::said(&settled);
    println!("REMOVAL-FORCE arm=4a settling said={answered}");
    assert_eq!(
        settled.status.code(),
        Some(1),
        "the settling command refuses a plan it cannot read: {answered}"
    );
    assert!(
        answered.contains(&format!(
            "the destruction plan {} names this clone and cannot be read",
            plan.display()
        )),
        "the settling command names the plan's file: {answered}"
    );
    assert_eq!(
        std::fs::read(plan.as_path()).expect("the plan's bytes still read"),
        bytes,
        "the settling command changed no byte of a plan it cannot read"
    );
    assert!(
        arm.exists(arm.clone.as_str()),
        "the settling command destroyed nothing"
    );
    println!("REMOVAL-FORCE arm=4a no-force=refused settling=refused plan=kept");
    arm.tear_down();
}

/// What:
///  Arm 4's one-force half:
///  the record's sentence says one force goes past a plan it cannot read,
///  destroys the clone and its snapshots from the pool's own list,
///  and deletes the plan's file.
/// Why:
///  This is the promise a person with one `--force` is given by the refusal
///  itself,
///  so the arm runs that command and reads the pool afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(arm.exists(arm.clone)).toBe(false);
/// ```
#[test]
fn zfs_removal_force_guest_a_plan_that_cannot_be_read_with_one_force_arm() {
    if !in_the_guest() {
        return;
    }
    let arm: Arm = Arm::make(5);
    arm.attach_at_the_slot();
    arm.snapshot(format!("{OWN_SNAPSHOT_PREFIX}a").as_str());
    let plan: PathBuf = plant_an_unreadable_plan(arm.place.common.as_path(), arm.clone_guid);
    let snapshots: Vec<String> = arm.the_snapshots();
    let sweeper: std::process::Child = arm.start_the_sweeper();
    let one: std::process::Output = arm.wrapper_git(&[
        "worktree",
        "remove",
        "--force",
        shown(arm.place.slot.as_path()),
    ]);
    arm.stop_the_sweeper(sweeper);
    let said: String = Arm::said(&one);
    println!("REMOVAL-FORCE arm=4b one-force said={said}");
    assert_eq!(
        one.status.code(),
        Some(0),
        "one force goes past the plan and destroys the clone, which is what the refusal's own sentence says: {said}"
    );
    assert!(
        said.contains(&format!(
            "a force was given, so this removal went past the destruction plan at {}, which could not be read",
            plan.display()
        )),
        "the report names the plan the run went past: {said}"
    );
    assert!(!arm.exists(arm.clone.as_str()), "the clone dataset is gone");
    assert!(
        arm.the_snapshots().is_empty(),
        "the snapshots the pool listed are gone: {:?}",
        arm.the_snapshots()
    );
    assert!(
        !snapshots.is_empty(),
        "the arm read the clone's own snapshot before the run"
    );
    assert!(!plan.exists(), "the plan's file is gone past");
    assert_eq!(arm.plan_entries(), 0, "no plan stands after the removal");
    println!("REMOVAL-FORCE arm=4b one-force=destroyed snapshots-before={snapshots:?}");
    arm.tear_down();
}

/// What:
///  Arm 4's two-force half:
///  the same state, the same promise, and the strongest count the grammar takes.
/// Why:
///  The project's answer makes two forces pass every refusal the removal has,
///  so a state two forces cannot pass is a sentence that does not hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(arm.exists(arm.clone)).toBe(false);
/// ```
#[test]
fn zfs_removal_force_guest_a_plan_that_cannot_be_read_with_two_forces_arm() {
    if !in_the_guest() {
        return;
    }
    let arm: Arm = Arm::make(6);
    arm.attach_at_the_slot();
    arm.snapshot(format!("{OWN_SNAPSHOT_PREFIX}a").as_str());
    let plan: PathBuf = plant_an_unreadable_plan(arm.place.common.as_path(), arm.clone_guid);
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
    println!("REMOVAL-FORCE arm=4c two-forces said={said}");
    assert_eq!(
        two.status.code(),
        Some(0),
        "two forces pass every refusal the removal has: {said}"
    );
    assert!(
        said.contains(&format!(
            "a force was given, so this removal went past the destruction plan at {}, which could not be read",
            plan.display()
        )),
        "the report names the plan the run went past: {said}"
    );
    assert!(!arm.exists(arm.clone.as_str()), "the clone dataset is gone");
    assert!(!plan.exists(), "the plan's file is gone past");
    assert_eq!(arm.plan_entries(), 0, "no plan stands after the removal");
    println!("REMOVAL-FORCE arm=4c two-forces=destroyed");
    arm.tear_down();
}

/// What:
///  The state the building round chose to refuse:
///  the plan cannot be read and the clone the record names is gone.
/// Why:
///  Deleting the plan would erase the only record of what an earlier run
///  destroyed,
///  so one force and two alike refuse,
///  and the arm reads what a person is left with:
///  the settling command refuses the same plan,
///  so the state has no route through this release.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(said).toContain('nothing proves what an earlier run destroyed');
/// ```
#[test]
fn zfs_removal_force_guest_a_plan_unreadable_and_the_clone_gone_arm() {
    if !in_the_guest() {
        return;
    }
    let arm: Arm = Arm::make(7);
    super::zfs_sweep_guest::zfs_ok(
        arm.sbin.as_str(),
        &["destroy", "-r", "-f", arm.clone.as_str()],
    );
    let plan: PathBuf = plant_an_unreadable_plan(arm.place.common.as_path(), arm.clone_guid);
    let bytes: Vec<u8> = std::fs::read(plan.as_path()).expect("the plan's bytes read");
    for (count, words) in [
        ("one", vec!["--force"]),
        ("two", vec!["--force", "--force"]),
    ] {
        let mut arguments: Vec<&str> = vec!["worktree", "remove"];
        arguments.extend(words);
        arguments.push(shown(arm.place.slot.as_path()));
        let answer: std::process::Output = arm.wrapper_git(arguments.as_slice());
        let said: String = Arm::said(&answer);
        println!("REMOVAL-FORCE arm=4d clone-gone count={count} said={said}");
        assert_eq!(
            answer.status.code(),
            Some(1),
            "a forced removal refuses when its plan cannot be read and its clone is gone: {said}"
        );
        assert!(
            said.contains(&format!(
                "a force was given, but the destruction plan at {} could not be read",
                plan.display()
            )),
            "the refusal names the plan's file: {said}"
        );
        assert!(
            said.contains("nothing proves what an earlier run destroyed"),
            "the refusal says what it cannot prove: {said}"
        );
        assert_eq!(
            std::fs::read(plan.as_path()).expect("the plan's bytes still read"),
            bytes,
            "a refused removal left the plan's bytes as they were"
        );
        assert!(
            !arm.exists(arm.clone.as_str()),
            "no removal put the clone back"
        );
    }
    let settled: std::process::Output = arm.wrapper_git(&[
        "monochromatic-git-policy-cli",
        "settle-worktree-clone-record",
        SLOT,
        "--confirm-removal-of",
        SLOT,
        "--confirm-clone-guid",
        last_four(arm.clone_guid.file_name().as_str()).as_str(),
    ]);
    let answered: String = Arm::said(&settled);
    println!("REMOVAL-FORCE arm=4d settling said={answered}");
    assert_eq!(
        settled.status.code(),
        Some(1),
        "the settling command refuses the plan as well: {answered}"
    );
    assert!(
        answered.contains("cannot be read"),
        "the settling command's refusal names the read that failed: {answered}"
    );
    assert_eq!(
        std::fs::read(plan.as_path()).expect("the plan's bytes still read"),
        bytes,
        "nothing of the release removed a plan it cannot read"
    );
    println!("REMOVAL-FORCE arm=4d left-with=no-route plan=kept clone=gone settling=refused");
    arm.tear_down();
}

/// The last four decimal digits of one GUID's name,
/// which the settling command's confirmation takes.
fn last_four(whole: &str) -> String {
    let start: usize = whole.len().saturating_sub(4);
    return String::from(&whole[start..]);
}
