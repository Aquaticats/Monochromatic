//! Unit tests for the timed-command planner.

use super::plan;
use crate::{
    protocol::{parse_command, Command, KeyAction, PointerButton},
    timeline::Step,
};

/// Plan a parsed line.
fn planned(line: &str) -> Option<Result<Vec<Step>, String>> {
    return match parse_command(line) {
        Ok(command) => plan(&command),
        Err(message) => Some(Err(message)),
    }
}

/// A hold presses the chord in order,
///  waits,
///  and releases in reverse.
#[test]
fn a_hold_presses_waits_and_releases_in_reverse() {
    assert_eq!(
        planned("key ctrl+right hold 1500"),
        Some(Ok(vec![Step::KeyDown(29), Step::KeyDown(106), Step::Wait(1500), Step::KeyUp(106), Step::KeyUp(29)])),
    );
}

/// A hold of an unknown key fails before any step.
#[test]
fn a_hold_of_an_unknown_key_fails() {
    assert_eq!(planned("key ctrl+nokey hold 100"), Some(Err("unknown key: nokey".to_string())));
}

/// Immediate commands are not planned.
#[test]
fn immediate_commands_are_not_planned() {
    assert_eq!(planned("key f10 tap"), None);
    assert_eq!(planned("click 1 2"), None);
    assert_eq!(planned("wheel 1 2 0 1"), None);
    assert_eq!(planned("ping"), None);
    assert_eq!(plan(&Command::Key { name: "a".to_string(), action: KeyAction::Press }), None);
}

/// A double click presses twice at one spot with the interval between.
#[test]
fn a_double_click_waits_between_presses() {
    let left = PointerButton::Left;
    assert_eq!(
        planned("click 10 20 count=2 interval=90"),
        Some(Ok(vec![
            Step::MoveTo { x: 10.0, y: 20.0 },
            Step::ButtonDown(left),
            Step::ButtonUp(left),
            Step::Wait(90),
            Step::ButtonDown(left),
            Step::ButtonUp(left),
        ])),
    );
}

/// A click with modifiers holds them around the press and releases them last in reverse.
#[test]
fn a_modified_click_holds_the_modifiers_around_it() {
    let right = PointerButton::Right;
    assert_eq!(
        planned("click 3 4 right mods=ctrl+shift"),
        Some(Ok(vec![
            Step::KeyDown(29),
            Step::KeyDown(42),
            Step::MoveTo { x: 3.0, y: 4.0 },
            Step::ButtonDown(right),
            Step::ButtonUp(right),
            Step::KeyUp(42),
            Step::KeyUp(29),
        ])),
    );
}

/// A drag moves in equal steps and ends exactly on the target before releasing.
#[test]
fn a_drag_moves_in_equal_steps_to_the_target() {
    let left = PointerButton::Left;
    assert_eq!(
        planned("drag 0 0 30 60 steps=3 step-ms=10"),
        Some(Ok(vec![
            Step::MoveTo { x: 0.0, y: 0.0 },
            Step::ButtonDown(left),
            Step::Wait(10),
            Step::MoveTo { x: 10.0, y: 20.0 },
            Step::Wait(10),
            Step::MoveTo { x: 20.0, y: 40.0 },
            Step::Wait(10),
            Step::MoveTo { x: 30.0, y: 60.0 },
            Step::Wait(10),
            Step::ButtonUp(left),
        ])),
    );
}

/// A touchpad scroll divides the travel and ends with the stop.
#[test]
fn a_scroll_divides_the_travel_and_stops() {
    assert_eq!(
        planned("scroll 5 6 0 40 steps=2 step-ms=7"),
        Some(Ok(vec![
            Step::Scroll { x: 5.0, y: 6.0, dx: 0.0, dy: 20.0 },
            Step::Wait(7),
            Step::Scroll { x: 5.0, y: 6.0, dx: 0.0, dy: 20.0 },
            Step::ScrollStop,
        ])),
    );
}

/// A wheel with modifiers is wrapped in them,
///  and only releases count as releasing steps.
#[test]
fn a_wheel_with_modifiers_is_wrapped_and_releases_are_marked() {
    assert_eq!(
        planned("wheel 1 2 0 0.5 mods=ctrl"),
        Some(Ok(vec![
            Step::KeyDown(29),
            Step::Wheel { x: 1.0, y: 2.0, horizontal: 0.0, vertical: 0.5 },
            Step::KeyUp(29),
        ])),
    );
    assert!(Step::KeyUp(1).releases());
    assert!(Step::ButtonUp(PointerButton::Left).releases());
    assert!(Step::ScrollStop.releases());
    assert!(!Step::KeyDown(1).releases());
    assert!(!Step::Wait(1).releases());
}
