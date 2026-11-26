//! What:
//!  Controls of the arm and step vocabulary:
//!  every arm and step has one name,
//!  the names parse back,
//!  the worker's order holds every step but the holder's,
//!  and the steps that refuse a removal are the move,
//!  the unmount and the scans.
//! Why:
//!  The record folds lines by step and the mode names the step in flight by
//!  the same order,
//!  so a duplicated or missing step would misname a time-out.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(new Set(ALL.map(name)).size).toBe(ALL.length);
//! ```

use super::{
    ProbeArm,
    ProbeStep,
};

/// Every arm and step has a name a reader can parse back.
#[test]
fn every_name_parses_back() {
    for &arm in ProbeArm::ALL {
        assert!(!arm.name().is_empty());
    }
    for &step in ProbeStep::ALL {
        assert_eq!(ProbeStep::from_name(step.name()), Some(step));
    }
    assert_eq!(ProbeStep::from_name("no-such-step"), None);
}

/// The worker's order holds every step but the holder's two,
/// and the holder's order holds exactly those.
#[test]
fn the_worker_order_holds_every_step_but_the_holders() {
    let worker: Vec<ProbeStep> = ProbeStep::WORKER_ORDER.to_vec();
    let holder: Vec<ProbeStep> = ProbeStep::HOLDER_ORDER.to_vec();
    for &step in ProbeStep::ALL {
        let in_worker: bool = worker.contains(&step);
        let in_holder: bool = holder.contains(&step);
        assert!(
            in_worker != in_holder,
            "{} is in both orders or in neither",
            step.name()
        );
    }
    assert_eq!(worker.len() + holder.len(), ProbeStep::ALL.len());
}

/// A removal is refused after exactly the move,
/// the unmount and the scans.
#[test]
fn only_the_move_unmount_and_scans_refuse_a_removal() {
    for &step in ProbeStep::ALL {
        let refuses: bool = step.refuses_removal_of_a_mounted_clone();
        let expected: bool = matches!(
            step,
            ProbeStep::Move
                | ProbeStep::Unmount
                | ProbeStep::ScanWhileHeld
                | ProbeStep::ScanAfterHolder
                | ProbeStep::ClosingScan
        );
        assert_eq!(refuses, expected, "{}", step.name());
    }
}

/// Each arm owns at least one step,
/// and the step's arm is the one its name sits under.
#[test]
fn every_arm_owns_a_step() {
    for &arm in ProbeArm::ALL {
        let owned: Vec<ProbeStep> = ProbeStep::ALL
            .iter()
            .copied()
            .filter(|step| return step.arm() == arm)
            .collect();
        assert!(!owned.is_empty(), "{} owns no step", arm.name());
    }
}
