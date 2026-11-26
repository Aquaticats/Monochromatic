//! What:
//!  Controls of one clone's destruction-plan line:
//!  no plan,
//!  a plan that cannot be read,
//!  a plan whose first write stands,
//!  a name that later lines settled,
//!  and the merge rule that picks a name's current outcome.
//! Why:
//!  The removal's refusals send a person to this file,
//!  and a name's state is the outcome on its highest attempt number,
//!  so the rule and the line that prints it are proved together.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(checkRemovalPlan({ clone, reading: { kind: 'none' } })).toMatchObject({ kind: 'pass' });
//! ```

use super::{
    PlanFileReading,
    RemovalPlanFacts,
    check_removal_plan,
    recorded_outcomes,
};
use crate::health::doctor_zfs_route::CheckOutcome;
use crate::worktree_clone_removal_proof_and_plan::fixtures::{
    guid,
    sample_plan,
    sample_snapshots,
};
use crate::worktree_clone_removal_proof_and_plan::plan_record::{
    DestructionPlan,
    PlanAttempt,
};
use crate::worktree_clone_removal_proof_and_plan::plan_words::SnapshotOutcome;
use std::path::PathBuf;

/// The clone the plans of these controls are about.
const CLONE: &str = "tank/wt/fix-flaky-test";

/// The plan's file.
fn plan_path() -> PathBuf {
    return PathBuf::from("/repo/.git/monochromatic-git-policy-cli/plans/7828944690444299989");
}

/// The sample plan as a standing reading.
fn standing(plan: DestructionPlan) -> RemovalPlanFacts {
    return RemovalPlanFacts {
        clone: String::from(CLONE),
        reading: PlanFileReading::Standing {
            path: plan_path(),
            plan: Box::new(plan),
        },
    };
}

/// No plan passes,
/// and says every snapshot is the clone's own.
#[test]
fn no_plan_passes() {
    let facts: RemovalPlanFacts = RemovalPlanFacts {
        clone: String::from(CLONE),
        reading: PlanFileReading::NoPlan,
    };
    let answer: CheckOutcome = check_removal_plan(&facts);
    let CheckOutcome::Pass {
        detail,
    } = answer
    else {
        panic!("no plan passes: {answer:?}");
    };
    assert!(detail.contains("no destruction plan stands"), "{detail}");
}

/// A file that does not read as a plan is unknown,
/// because it may hold the only record of a destroyed snapshot.
#[test]
fn an_unreadable_plan_is_unknown() {
    let facts: RemovalPlanFacts = RemovalPlanFacts {
        clone: String::from(CLONE),
        reading: PlanFileReading::Unreadable {
            path: plan_path(),
            reason: String::from("the header ends early"),
        },
    };
    let answer: CheckOutcome = check_removal_plan(&facts);
    let CheckOutcome::Unknown {
        reason,
    } = answer
    else {
        panic!("an unreadable plan is unknown: {answer:?}");
    };
    assert!(reason.contains("the header ends early"), "{reason}");
    assert!(reason.contains("only record"), "{reason}");
}

/// A standing plan is a finding naming the moment,
/// the writer,
/// the head and every name with its outcome.
#[test]
fn a_standing_plan_names_every_field() {
    let answer: CheckOutcome = check_removal_plan(&standing(sample_plan()));
    let CheckOutcome::Finding {
        field,
        detail,
    } = answer
    else {
        panic!("a standing plan is a finding: {answer:?}");
    };
    assert_eq!(field, "plan");
    assert!(
        detail.contains(plan_path().display().to_string().as_str()),
        "{detail}"
    );
    assert!(detail.contains("2026-11-22T18:40:12-05:00"), "{detail}");
    assert!(detail.contains("48213:linux:9481733"), "{detail}");
    assert!(detail.contains("head=present"), "{detail}");
    for snapshot in sample_snapshots() {
        assert!(
            detail.contains(snapshot.name.as_str()),
            "{snapshot:?} is missing from {detail}"
        );
    }
    assert!(detail.contains("no removal for a clone"), "{detail}");
}

/// A name settled by a later line prints the later outcome,
/// which is the record's merge rule.
#[test]
fn a_later_attempt_is_the_names_current_state() {
    let mut plan: DestructionPlan = sample_plan();
    let first = sample_snapshots()
        .into_iter()
        .next()
        .expect("the sample holds a snapshot");
    plan.record_line(PlanAttempt {
        snapshot: first.clone(),
        attempt: 1,
        outcome: SnapshotOutcome::Destroyed,
        errno: None,
    })
    .expect("the later line is inside the record's rule");
    let outcomes = recorded_outcomes(&plan);
    let settled = outcomes
        .iter()
        .find(|line| return line.name == first.name)
        .expect("the name keeps its line");
    assert_eq!(settled.outcome, SnapshotOutcome::Destroyed);
    assert_eq!(settled.attempt, 1);
    let answer: CheckOutcome = check_removal_plan(&standing(plan));
    let CheckOutcome::Finding {
        detail,
        ..
    } = answer
    else {
        panic!("a standing plan is a finding: {answer:?}");
    };
    assert!(detail.contains("destroyed on attempt 1"), "{detail}");
    assert!(detail.contains("cannot be restored"), "{detail}");
}

/// A refused line names the kernel's error number,
/// so a person can look it up.
#[test]
fn a_refused_line_names_its_errno() {
    let mut plan: DestructionPlan = sample_plan();
    let first = sample_snapshots()
        .into_iter()
        .next()
        .expect("the sample holds a snapshot");
    plan.record_line(PlanAttempt {
        snapshot: first,
        attempt: 1,
        outcome: SnapshotOutcome::Refused,
        errno: Some(16),
    })
    .expect("the later line is inside the record's rule");
    let answer: CheckOutcome = check_removal_plan(&standing(plan));
    let CheckOutcome::Finding {
        detail,
        ..
    } = answer
    else {
        panic!("a standing plan is a finding: {answer:?}");
    };
    assert!(detail.contains("errno 16"), "{detail}");
}

/// Every name's clause carries the GUID the plan recorded,
/// so a person compares the tuple rather than the name alone.
#[test]
fn every_clause_carries_the_recorded_guid() {
    let plan: DestructionPlan = sample_plan();
    let outcomes = recorded_outcomes(&plan);
    assert_eq!(outcomes.len(), sample_snapshots().len());
    for outcome in &outcomes {
        let recorded = sample_snapshots()
            .into_iter()
            .find(|snapshot| return snapshot.name == outcome.name)
            .expect("the sample holds every name");
        assert_eq!(outcome.guid, recorded.guid, "{outcome:?}");
        assert!(
            outcome
                .clause()
                .contains(recorded.guid.file_name().as_str()),
            "{outcome:?}"
        );
    }
    let _ = guid(crate::worktree_clone_removal_proof_and_plan::fixtures::SAMPLE_OWN_GUID);
}
