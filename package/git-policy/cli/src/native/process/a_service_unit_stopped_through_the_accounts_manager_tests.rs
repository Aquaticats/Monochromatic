//! Controls for the stop's own verdict and its own answer:
//!  a unit whose tasks and namespace are gone is stopped,
//!  a unit that is active again is refused as that,
//!  a task still in the control group refuses,
//!  a namespace that is still there refuses,
//!  every answer is the locked holder's word and carries what the manager or the
//!  wait said.

use crate::process::a_service_unit_stopped_through_the_accounts_manager::*;

/// The unit these controls stop,
/// as the manager spells it.
const UNIT: &str = "forced-act-holder.service";

/// What:
///  One task's whole control-group file,
///  for a task in this unit.
/// Why:
///  The wait's read is the kernel's own text,
///  so a control hands the text a live task would print.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = '0::/user.slice/user-0.slice/user@0.service/app.slice/forced-act-holder.service\n';
/// ```
fn a_task_in_the_unit() -> &'static str {
    return "0::/user.slice/user-0.slice/user@0.service/app.slice/forced-act-holder.service\n";
}

/// A unit whose tasks and namespace are gone is stopped,
/// whatever the manager's state for a unit it no longer runs.
#[test]
fn a_unit_whose_tasks_and_namespace_are_gone_is_stopped() {
    let gone: LeftBehind<'_> = LeftBehind {
        state: None,
        tasks_cgroup: None,
        namespace: false,
    };
    assert_eq!(the_stops_verdict(UNIT, &gone), Ok(()));
    let inactive: LeftBehind<'_> = LeftBehind {
        state: Some("inactive"),
        tasks_cgroup: None,
        namespace: false,
    };
    assert_eq!(the_stops_verdict(UNIT, &inactive), Ok(()));
    // A task of another unit in the same file is no task of this one.
    let elsewhere: LeftBehind<'_> = LeftBehind {
        state: Some("inactive"),
        tasks_cgroup: Some("0::/user.slice/user-0.slice/user@0.service/app.slice/other.service\n"),
        namespace: false,
    };
    assert_eq!(the_stops_verdict(UNIT, &elsewhere), Ok(()));
}

/// A unit the manager reports active again is refused as that,
/// which is the shape a unit with `Restart=` leaves.
#[test]
fn a_unit_active_again_is_refused_as_that() {
    for state in ["active", "activating", "reloading"] {
        let back: LeftBehind<'_> = LeftBehind {
            state: Some(state),
            tasks_cgroup: Some(a_task_in_the_unit()),
            namespace: true,
        };
        let refused: UnitNotStopped =
            the_stops_verdict(UNIT, &back).expect_err("a unit that came back is no stop");
        assert_eq!(
            refused,
            UnitNotStopped::TheUnitCameBack {
                unit: String::from(UNIT),
                said: format!("the manager reports the unit {state} again"),
            }
        );
    }
}

/// A task still in the unit's control group refuses,
/// which is the shape a unit with `KillMode=none` leaves.
#[test]
fn a_task_still_in_the_control_group_refuses() {
    let stayed: LeftBehind<'_> = LeftBehind {
        state: Some("inactive"),
        tasks_cgroup: Some(a_task_in_the_unit()),
        namespace: false,
    };
    let refused: UnitNotStopped =
        the_stops_verdict(UNIT, &stayed).expect_err("a task that stayed is no stop");
    assert_eq!(
        refused,
        UnitNotStopped::TheUnitStayed {
            unit: String::from(UNIT),
            said: String::from("a task is still in the unit's control group when the wait ends"),
        }
    );
}

/// A namespace that is still there refuses,
/// because the copy is what a stop has to take with it.
#[test]
fn a_namespace_still_there_refuses() {
    let here: LeftBehind<'_> = LeftBehind {
        state: Some("inactive"),
        tasks_cgroup: None,
        namespace: true,
    };
    let refused: UnitNotStopped =
        the_stops_verdict(UNIT, &here).expect_err("a namespace that stayed is no stop");
    assert_eq!(
        refused,
        UnitNotStopped::TheUnitStayed {
            unit: String::from(UNIT),
            said: String::from("the holder's mount namespace is still there when the wait ends"),
        }
    );
}

/// Every answer is the locked holder's word,
/// and each carries what the manager or the wait said.
#[test]
fn every_answer_is_the_word_and_carries_what_was_said() {
    let answers: Vec<UnitNotStopped> = vec![
        UnitNotStopped::TheBusRefused {
            said: String::from("the session bus could not be reached: Connection refused"),
        },
        UnitNotStopped::TheStopRefused {
            unit: String::from(UNIT),
            said: String::from("the manager answered org.freedesktop.systemd1.NoSuchUnit: gone"),
        },
        UnitNotStopped::TheUnitStayed {
            unit: String::from(UNIT),
            said: String::from("a task is still in the unit's control group when the wait ends"),
        },
        UnitNotStopped::TheUnitCameBack {
            unit: String::from(UNIT),
            said: String::from("the manager reports the unit active again"),
        },
    ];
    for answer in &answers {
        assert_eq!(answer.word(), HOLDER_LOCKED_WORD);
        assert!(!answer.said().is_empty(), "{answer:?}");
    }
}
