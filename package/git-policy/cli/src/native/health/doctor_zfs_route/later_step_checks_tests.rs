//! What:
//!  Controls of the checks later steps build:
//!  each answers unknown,
//!  names its step and says what that step proves.
//! Why:
//!  An unknown without a reason sends a person nowhere,
//!  and an answer that drifted into a pass would claim a check this tree does
//!  not have,
//!  so the whole list is proved at once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const step of LaterStep.ALL) expect(check_later_step(step)).toMatchObject({ kind: 'unknown' });
//! ```

use super::{LaterStep, check_later_step};

/// Every later-step check answers unknown.
#[test]
fn every_later_step_answers_unknown() {
    for step in LaterStep::ALL {
        let outcome: super::CheckOutcome = check_later_step(step);
        assert!(matches!(outcome, super::CheckOutcome::Unknown { .. }), "{step:?} answered {outcome:?}");
    }
}

/// Every reason names its plan step,
/// and the deferred one names step 15.
#[test]
fn every_reason_names_its_step() {
    for step in LaterStep::ALL {
        let reason: String = step.reason();
        assert!(reason.contains("step "), "{step:?} says {reason}");
    }
    assert!(LaterStep::DeferredOrigin.reason().contains("step 15"));
    assert!(LaterStep::RemovalPlans.reason().contains("step 14") && LaterStep::RemovalPlans.reason().contains("step 16"));
    assert!(LaterStep::CancelCommand.reason().contains("step 17"));
    assert!(LaterStep::BackupCoverage.reason().contains("step 19"));
    assert!(LaterStep::Sanoid.reason().contains("step 20"));
}

/// Each later step reports under its own check name.
#[test]
fn each_later_step_reports_under_its_own_name() {
    let names: Vec<&'static str> = LaterStep::ALL.iter().map(|step| return step.name()).collect();
    assert_eq!(names, vec!["deferred-origin", "removal-plans", "cancel-command", "backup-coverage", "sanoid"]);
}
