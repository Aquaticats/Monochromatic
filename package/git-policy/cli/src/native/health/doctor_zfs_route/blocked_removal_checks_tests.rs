//! What:
//!  Controls of one clone's blocked-removal line:
//!  nothing pending,
//!  an intent with no plan,
//!  a plan no holder blocks,
//!  a holder,
//!  a scan that could not read every task,
//!  the caller's own session among the holders,
//!  the forced arm before and after its measurement,
//!  and the snapshots already destroyed.
//! Why:
//!  A person who meets a blocked removal needs the loss,
//!  the holders and the one thing that may end them,
//!  and a removal never ends the caller or its session,
//!  so each arm is shown with the words it must print.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(checkBlockedRemoval(facts)).toMatchObject({ kind: 'finding', field: 'removal' });
//! ```

use super::{
    BlockedRemovalFacts,
    check_blocked_removal,
};
use crate::health::doctor_zfs_route::CheckOutcome;
use crate::health::doctor_zfs_route::holder_checks::{
    HolderFacts,
    NamespaceHold,
};
use crate::health::doctor_zfs_route::removal_plan_checks::RecordedSnapshot;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::plan_words::SnapshotOutcome;
use std::path::PathBuf;

/// The slot these controls are about.
const SLOT: &str = "topic";

/// The clone these controls are about.
const CLONE: &str = "tank/wt/fix-flaky-test";

/// The intent nonce the removal wrote.
const NONCE: &str = "0123456789abcdef0123456789abcdef";

/// One name the plan recorded.
fn recorded(
    name: &str,
    outcome: SnapshotOutcome,
) -> RecordedSnapshot {
    return RecordedSnapshot {
        name: String::from(name),
        guid: DatasetGuid::parse("21").expect("a canonical GUID"),
        outcome,
        attempt: 1,
        errno: None,
    };
}

/// The scan's answer with one holder.
fn one_holder() -> HolderFacts {
    return HolderFacts {
        clone: String::from(CLONE),
        holders: vec![NamespaceHold {
            namespace: String::from("mnt:[4026532001]"),
            representative_pid: 4242,
            source: String::from(CLONE),
            mount_point: PathBuf::from("/home/u/repo/.claude/worktrees/topic"),
        }],
        incomplete: Vec::new(),
    };
}

/// The facts of a blocked removal.
fn facts(
    recorded_names: Vec<RecordedSnapshot>,
    holders: HolderFacts,
) -> BlockedRemovalFacts {
    return BlockedRemovalFacts {
        slot: String::from(SLOT),
        clone: String::from(CLONE),
        intent: Some(String::from(NONCE)),
        plan_stands: !recorded_names.is_empty(),
        recorded: recorded_names,
        holders,
        in_the_callers_session: Vec::new(),
        the_forced_arm_is_measured: false,
    };
}

/// Nothing pending passes,
/// and says nothing is blocked and nothing is to cancel.
#[test]
fn nothing_pending_passes() {
    let mut held: BlockedRemovalFacts = facts(
        Vec::new(),
        HolderFacts {
            clone: String::from(CLONE),
            holders: Vec::new(),
            incomplete: Vec::new(),
        },
    );
    held.intent = None;
    held.plan_stands = false;
    let answer: CheckOutcome = check_blocked_removal(&held);
    let CheckOutcome::Pass {
        detail,
    } = answer
    else {
        panic!("nothing pending passes: {answer:?}");
    };
    assert!(detail.contains("no removal is pending"), "{detail}");
}

/// A plan whose first write recorded no snapshot line still stands,
/// so a holder blocks it and the line never calls it an intent with nothing
/// behind it.
#[test]
fn a_plan_with_no_snapshot_line_still_stands() {
    let mut held: BlockedRemovalFacts = facts(Vec::new(), one_holder());
    held.plan_stands = true;
    let answer: CheckOutcome = check_blocked_removal(&held);
    let CheckOutcome::Finding {
        field,
        detail,
    } = answer
    else {
        panic!("a holder blocks a plan with no lines: {answer:?}");
    };
    assert_eq!(field, "removal");
    assert!(detail.contains("nothing is lost yet"), "{detail}");
}

/// An intent with no plan is a finding naming the nonce.
#[test]
fn an_intent_without_a_plan_is_a_finding() {
    let mut held: BlockedRemovalFacts = facts(
        Vec::new(),
        HolderFacts {
            clone: String::from(CLONE),
            holders: Vec::new(),
            incomplete: Vec::new(),
        },
    );
    held.plan_stands = false;
    let answer: CheckOutcome = check_blocked_removal(&held);
    let CheckOutcome::Finding {
        field,
        detail,
    } = answer
    else {
        panic!("an intent without a plan is a finding: {answer:?}");
    };
    assert_eq!(field, "intent");
    assert!(detail.contains(NONCE), "{detail}");
    assert!(
        detail.contains("no destruction plan was written"),
        "{detail}"
    );
    assert!(
        detail.contains(crate::worktree_clone_binding_record::THE_REMOVALS_WORDS),
        "{detail}"
    );
}

/// A plan no holder blocks passes,
/// so the plan's own line carries the in-flight state.
#[test]
fn a_plan_without_a_holder_passes() {
    let held: BlockedRemovalFacts = facts(
        vec![recorded(
            "tank/wt/fix-flaky-test@own",
            SnapshotOutcome::Planned,
        )],
        HolderFacts {
            clone: String::from(CLONE),
            holders: Vec::new(),
            incomplete: Vec::new(),
        },
    );
    let answer: CheckOutcome = check_blocked_removal(&held);
    let CheckOutcome::Pass {
        detail,
    } = answer
    else {
        panic!("a plan without a holder passes: {answer:?}");
    };
    assert!(detail.contains("not blocked by a holder"), "{detail}");
}

/// A holder is a finding naming the namespace and the task,
/// the loss,
/// the objects that stand,
/// and the self-cancelling removal.
#[test]
fn a_holder_is_a_finding_naming_the_loss_and_the_holder() {
    let held: BlockedRemovalFacts = facts(
        vec![
            recorded("tank/wt/fix-flaky-test@own", SnapshotOutcome::Destroyed),
            recorded("tank/wt/fix-flaky-test@late", SnapshotOutcome::Planned),
        ],
        one_holder(),
    );
    let answer: CheckOutcome = check_blocked_removal(&held);
    let CheckOutcome::Finding {
        field,
        detail,
    } = answer
    else {
        panic!("a holder is a finding: {answer:?}");
    };
    assert_eq!(field, "removal");
    assert!(detail.contains("mnt:[4026532001]"), "{detail}");
    assert!(detail.contains("4242"), "{detail}");
    assert!(detail.contains(NONCE), "{detail}");
    assert!(
        detail.contains("1 snapshot(s) are recorded as destroyed"),
        "{detail}"
    );
    assert!(detail.contains("tank/wt/fix-flaky-test@own"), "{detail}");
    assert!(detail.contains("tank/wt/fix-flaky-test@late"), "{detail}");
    assert!(detail.contains("cancels itself"), "{detail}");
    assert!(detail.contains("has no measurement yet"), "{detail}");
    assert!(detail.contains("cancel-worktree-removal"), "{detail}");
    assert!(
        !detail.contains("not part of this release"),
        "the command this release's grammar admits is not called absent: {detail}"
    );
}

/// A scan that could not read every task leaves the blocked state unproved.
#[test]
fn an_incomplete_scan_is_unknown() {
    let held: BlockedRemovalFacts = facts(
        vec![recorded(
            "tank/wt/fix-flaky-test@own",
            SnapshotOutcome::Planned,
        )],
        HolderFacts {
            clone: String::from(CLONE),
            holders: Vec::new(),
            incomplete: vec![String::from("task 90's namespace link is unreadable")],
        },
    );
    let answer: CheckOutcome = check_blocked_removal(&held);
    let CheckOutcome::Unknown {
        reason,
    } = answer
    else {
        panic!("an incomplete scan is unknown: {answer:?}");
    };
    assert!(reason.contains("task 90"), "{reason}");
    assert!(reason.contains("unproved"), "{reason}");
}

/// A holder of the caller's own session is named,
/// and the removal refuses instead of ending it.
#[test]
fn the_callers_session_is_named_and_spared() {
    let mut held: BlockedRemovalFacts = facts(
        vec![recorded(
            "tank/wt/fix-flaky-test@own",
            SnapshotOutcome::Planned,
        )],
        one_holder(),
    );
    held.in_the_callers_session = vec![4242];
    let answer: CheckOutcome = check_blocked_removal(&held);
    let CheckOutcome::Finding {
        detail,
        ..
    } = answer
    else {
        panic!("a holder is a finding: {answer:?}");
    };
    assert!(detail.contains("4242"), "{detail}");
    assert!(
        detail.contains("never ends the caller or its session"),
        "{detail}"
    );
    assert!(detail.contains("leave the folder"), "{detail}");
}

/// The forced arm's words change when its measurement lands,
/// and nothing before it offers the arm.
#[test]
fn the_forced_arm_is_offered_only_once_it_is_measured() {
    let mut held: BlockedRemovalFacts = facts(
        vec![recorded(
            "tank/wt/fix-flaky-test@own",
            SnapshotOutcome::Planned,
        )],
        one_holder(),
    );
    held.the_forced_arm_is_measured = true;
    let answer: CheckOutcome = check_blocked_removal(&held);
    let CheckOutcome::Finding {
        detail,
        ..
    } = answer
    else {
        panic!("a holder is a finding: {answer:?}");
    };
    assert!(
        detail.contains("every holder other than the caller and its session is ended"),
        "{detail}"
    );
    assert!(!detail.contains("has no measurement yet"), "{detail}");
}

/// A plan that recorded no destroyed name says nothing is lost yet.
#[test]
fn a_plan_with_no_destroyed_name_says_nothing_is_lost() {
    let held: BlockedRemovalFacts = facts(
        vec![recorded(
            "tank/wt/fix-flaky-test@own",
            SnapshotOutcome::Planned,
        )],
        one_holder(),
    );
    let answer: CheckOutcome = check_blocked_removal(&held);
    let CheckOutcome::Finding {
        detail,
        ..
    } = answer
    else {
        panic!("a holder is a finding: {answer:?}");
    };
    assert!(detail.contains("nothing is lost yet"), "{detail}");
}

/// A blocked removal with no intent of the clone's own says so,
/// rather than leaving the property out.
#[test]
fn a_missing_intent_is_named() {
    let mut held: BlockedRemovalFacts = facts(
        vec![recorded(
            "tank/wt/fix-flaky-test@own",
            SnapshotOutcome::Planned,
        )],
        one_holder(),
    );
    held.intent = None;
    let answer: CheckOutcome = check_blocked_removal(&held);
    let CheckOutcome::Finding {
        detail,
        ..
    } = answer
    else {
        panic!("a holder is a finding: {answer:?}");
    };
    assert!(detail.contains("No `unmount` intent"), "{detail}");
}
