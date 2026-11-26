//! What:
//!  Controls for the refusals:
//!  each arm's exit code,
//!  and the two sentences whose state the open item asks about,
//!  with the unknown-outcome arm and the head-gone arm naming what they read.
//! Why:
//!  Only the caller's two mistakes exit `2` and every other state exits `1`,
//!  and a person acts on these sentences
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  "What it refuses" and "Exit codes and streams").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(refusal.exitCode()).toBe(2);
//! ```

use super::{CALLERS_MISTAKE, CancelRefusal, THE_OPERATION_FAILED, of_pointer};
use crate::worktree_clone_mount::clone_slot_git_pointer::GitPointerRefusal;
use std::path::PathBuf;

/// Every arm a person can meet,
/// one of each,
/// so a control that reads the list cannot pass on an arm it forgot.
fn every_arm() -> Vec<CancelRefusal> {
    return vec![
        CancelRefusal::NotBeingRemoved { slot: String::from("one"), found: String::from("Its plan is not there.") },
        CancelRefusal::ASlotOfAnotherRepository {
            slot: String::from("one"),
            points_at: String::from("/other/.git/worktrees/one"),
            this_repository: String::from("/this/.git/worktrees/one"),
        },
        CancelRefusal::ThePointerIsNotOneGitdirLine { slot: String::from("one"), said: String::from("it has no final newline") },
        CancelRefusal::NoBinding { slot: String::from("one"), reason: String::from("the file is absent") },
        CancelRefusal::TheBindingIsNotThisSlots { slot: String::from("one"), said: String::from("is filed as the slot `two`") },
        CancelRefusal::TheRecordsDisagree {
            what: String::from("the plan and the binding name different clones"),
            first: String::from("a"),
            second: String::from("b"),
        },
        CancelRefusal::ALiveRemoval { owner: String::from("pid 4242") },
        CancelRefusal::TheClaimCouldNotBeTaken { said: String::from("the folder refused") },
        CancelRefusal::TheCloneDoesNotAnswer { dataset: String::from("tank/clone"), reason: String::from("EIO") },
        CancelRefusal::TheCloneIsNotTheRecordedOne {
            field: "cloneGuid",
            recorded: String::from("7828944690444299989"),
            live: String::from("17157080670377245572"),
        },
        CancelRefusal::TheIntentIsNotALocalRequest { clone: String::from("tank/clone"), said: String::from("no local value") },
        CancelRefusal::TheInventoryIsUnreadable { clone: String::from("tank/clone"), reason: String::from("EIO") },
        CancelRefusal::AnUnknownOutcome {
            snapshot: String::from("tank/clone@sanoid-hourly-2026-11-22_03"),
            guid: String::from("9332110456720018274"),
            said: String::from("the plan marks it `unknown`"),
        },
        CancelRefusal::ASnapshotNameIsReused {
            name: String::from("tank/clone@one"),
            recorded_guid: String::from("1"),
            live_guid: String::from("2"),
        },
        CancelRefusal::TheHeadIsGone {
            slot: String::from("one"),
            dataset: String::from("tank/clone"),
            guid: String::from("7828944690444299989"),
            said: String::from("no child of its pinned parent carries the GUID either"),
        },
        CancelRefusal::ThePlanIsUnreadable { path: PathBuf::from("/repo/.git/monochromatic-git-policy-cli/clone-destruction-plans/1"), said: String::from("the header is short") },
        CancelRefusal::ThePlanIsOfAnotherClone {
            path: PathBuf::from("/repo/.git/plan"),
            plan_guid: String::from("1"),
            recorded_guid: String::from("2"),
        },
        CancelRefusal::ThePlanCouldNotGo { path: PathBuf::from("/repo/.git/plan"), said: String::from("EACCES") },
        CancelRefusal::TheMountRequestCouldNotBeWritten { said: String::from("the gate refused") },
        CancelRefusal::TheHandlerRefused {
            outcome: String::from("refused"),
            step: String::from("slot-walk"),
            errno: String::from("EBUSY"),
        },
        CancelRefusal::NoAnswerAtTheDeadline { said: String::from("No receipt was on the file at the deadline.") },
        CancelRefusal::TheStagedStateIsKept { step: String::from("mount"), said: String::from("a mount covers the folder") },
        CancelRefusal::TheMountStateIsNotTheAnswers { said: String::from("the answer named no mount") },
    ];
}

#[test]
fn only_a_callers_mistake_exits_two() {
    for refusal in every_arm() {
        let expected: i32 = match &refusal {
            CancelRefusal::NotBeingRemoved { .. } | CancelRefusal::ASlotOfAnotherRepository { .. } => CALLERS_MISTAKE,
            _ => THE_OPERATION_FAILED,
        };
        assert_eq!(refusal.exit_code(), expected, "{refusal:?}");
        assert!(expected == 1 || expected == 2, "a status this route uses: {expected}");
    }
}

#[test]
fn every_arm_says_what_it_found_and_the_way_forward() {
    for refusal in every_arm() {
        let said: String = refusal.message();
        assert!(!said.is_empty(), "{refusal:?} says nothing");
        assert!(said.contains("Ways forward:") || said.contains("way forward"), "{refusal:?} names no way forward: {said}");
    }
}

#[test]
fn the_unknown_outcome_names_the_snapshot_its_guid_and_the_removal_to_resume() {
    let refusal: CancelRefusal = CancelRefusal::AnUnknownOutcome {
        snapshot: String::from("tank/clone@sanoid-hourly-2026-11-22_03"),
        guid: String::from("9332110456720018274"),
        said: String::from("the plan marks it `unknown`"),
    };
    let said: String = refusal.message();
    assert!(said.contains("tank/clone@sanoid-hourly-2026-11-22_03"), "{said}");
    assert!(said.contains("9332110456720018274"), "{said}");
    assert!(said.contains("Nothing was mounted and nothing was changed"), "{said}");
    assert!(said.contains("git worktree remove"), "{said}");
    assert!(said.contains("doctor"), "{said}");
    assert_eq!(refusal.exit_code(), THE_OPERATION_FAILED);
}

#[test]
fn the_head_gone_refusal_names_the_clone_the_guid_and_the_settle_step() {
    let refusal: CancelRefusal = CancelRefusal::TheHeadIsGone {
        slot: String::from("one"),
        dataset: String::from("tank/clone"),
        guid: String::from("7828944690444299989"),
        said: String::from("no child of its pinned parent carries the GUID either"),
    };
    let said: String = refusal.message();
    assert!(said.contains("its clone is gone"), "{said}");
    assert!(said.contains("tank/clone"), "{said}");
    assert!(said.contains("7828944690444299989"), "{said}");
    assert!(said.contains("git worktree remove"), "{said}");
    assert!(said.contains("origin cleanup"), "{said}");
    assert_eq!(refusal.exit_code(), THE_OPERATION_FAILED);
}

#[test]
fn a_pointer_to_another_checkout_is_the_callers_mistake_and_a_damaged_one_is_not() {
    let other: GitPointerRefusal = GitPointerRefusal::AnotherRegistration {
        slot: String::from("one"),
        points_at: PathBuf::from("/other/.git/worktrees/one"),
        wanted: PathBuf::from("/this/.git/worktrees/one"),
    };
    assert_eq!(of_pointer("one", &other).exit_code(), CALLERS_MISTAKE);
    let damaged: GitPointerRefusal = GitPointerRefusal::Absent { slot: String::from("one") };
    assert_eq!(of_pointer("one", &damaged).exit_code(), THE_OPERATION_FAILED);
    let said: String = of_pointer("one", &other).message();
    assert!(said.contains("/other/.git/worktrees/one"), "{said}");
    assert!(said.contains("/this/.git/worktrees/one"), "{said}");
}
