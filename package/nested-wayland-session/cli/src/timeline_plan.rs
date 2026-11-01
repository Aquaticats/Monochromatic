//! Turn the commands that take time into step lists.

/// Sibling modules of this crate.
use crate::{
    key_chord::{parse_chord, Chord},
    protocol::{Command, KeyAction, PointerButton},
    timeline::Step,
};

/// Wrap steps in a modifier chord:
///  press the modifiers first,
///  release them last in reverse.
///
/// What:
///      `fn with_modifiers(modifiers: &str, inner: Vec<Step>) -> Result<Vec<Step>, String>`.
///  An empty `modifiers` returns `inner` unchanged.
/// Why:
///       A Ctrl click needs Ctrl down before the press and up after the release.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withModifiers(modifiers: string, inner: Step[]): Step[] { ... }
/// ```
fn with_modifiers(modifiers: &str, inner: Vec<Step>) -> Result<Vec<Step>, String> {
    if modifiers.is_empty() {
        return Ok(inner)
    }
    let chord = parse_chord(modifiers)?;
    let mut steps: Vec<Step> = chord.keys.iter().map(|code| return Step::KeyDown(*code)).collect();
    steps.extend(inner);
    steps.extend(chord.keys.iter().rev().map(|code| return Step::KeyUp(*code)));
    return Ok(steps)
}

/// The steps of a key held for a duration.
///
/// What:
///      `pub fn hold_steps(chord: &Chord, millis: u64) -> Vec<Step>`.
/// Why:
///       The client sees the key down for `millis`,
///  and runs its own repeat from the announced delay and rate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function holdSteps(chord: Chord, millis: number): Step[] { ... }
/// ```
#[must_use]
pub fn hold_steps(chord: &Chord, millis: u64) -> Vec<Step> {
    let mut steps: Vec<Step> = chord.keys.iter().map(|code| return Step::KeyDown(*code)).collect();
    steps.push(Step::Wait(millis));
    steps.extend(chord.keys.iter().rev().map(|code| return Step::KeyUp(*code)));
    return steps
}

/// The steps of `count` presses of a button at a point.
///
/// What:
///      `fn click_steps(x, y, button, count, interval_ms) -> Vec<Step>`.
/// Why:
///       A double click is two presses `interval_ms` apart at one spot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function clickSteps(x: number, y: number, button: PointerButton, count: number, intervalMs: number): Step[] { ... }
/// ```
fn click_steps(x: f64, y: f64, button: PointerButton, count: u32, interval_ms: u64) -> Vec<Step> {
    let mut steps = vec![Step::MoveTo { x, y }];
    for index in 0..count {
        if index > 0 {
            steps.push(Step::Wait(interval_ms));
        }
        steps.push(Step::ButtonDown(button));
        steps.push(Step::ButtonUp(button));
    }
    return steps
}

/// The steps of a drag with intermediate motion.
///
/// What:
///      `fn drag_steps(from, to, button, steps, step_ms) -> Vec<Step>`.
///  Press at `from`,
///  `steps` equal moves ending exactly at `to`,
///  a last wait so the client handles the final motion,
///  then release.
/// Why:
///       Controls that react to motion between press and release need those motion events.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dragSteps(from: Point, to: Point, button: PointerButton, steps: number, stepMs: number): Step[] { ... }
/// ```
fn drag_steps(from: (f64, f64), to: (f64, f64), button: PointerButton, steps: u32, step_ms: u64) -> Vec<Step> {
    let mut sequence = vec![Step::MoveTo { x: from.0, y: from.1 }, Step::ButtonDown(button)];
    for index in 1..=steps {
        let fraction = f64::from(index) / f64::from(steps);
        sequence.push(Step::Wait(step_ms));
        sequence.push(Step::MoveTo {
            x: from.0 + (to.0 - from.0) * fraction,
            y: from.1 + (to.1 - from.1) * fraction,
        });
    }
    sequence.push(Step::Wait(step_ms));
    sequence.push(Step::ButtonUp(button));
    return sequence
}

/// The steps of a touchpad scroll split into events.
///
/// What:
///      `fn scroll_steps(x, y, dx, dy, steps, step_ms) -> Vec<Step>`.
///  The travel is divided evenly and ends with the finger-lift stop.
/// Why:
///       A touchpad delivers many small pixel deltas,
///  then a stop event that starts any kinetic scrolling in the client.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scrollSteps(x: number, y: number, dx: number, dy: number, steps: number, stepMs: number): Step[] { ... }
/// ```
fn scroll_steps(x: f64, y: f64, dx: f64, dy: f64, steps: u32, step_ms: u64) -> Vec<Step> {
    let mut sequence = Vec::new();
    for index in 0..steps {
        if index > 0 {
            sequence.push(Step::Wait(step_ms));
        }
        sequence.push(Step::Scroll { x, y, dx: dx / f64::from(steps), dy: dy / f64::from(steps) });
    }
    sequence.push(Step::ScrollStop);
    return sequence
}

/// Plan a command that takes time or holds modifier keys.
///
/// What:
///      `pub fn plan(command: &Command) -> Option<Result<Vec<Step>, String>>`.
///  `None` for commands the immediate executor handles;
///  `Some(Err)` when a key name does not resolve.
/// Why:
///       One decision point for which commands go through the timer runner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function plan(command: Command): Step[] | Error | undefined { ... }
/// ```
#[must_use]
pub fn plan(command: &Command) -> Option<Result<Vec<Step>, String>> {
    match command {
        Command::Key { name, action: KeyAction::Hold { millis } } => {
            return Some(parse_chord(name).map(|chord| return hold_steps(&chord, *millis)))
        }
        Command::Click { x, y, button, count, interval_ms, modifiers } if *count > 1 || !modifiers.is_empty() => {
            return Some(with_modifiers(modifiers, click_steps(*x, *y, *button, *count, *interval_ms)))
        }
        Command::Drag { from, to, button, steps, step_ms, modifiers } => {
            return Some(with_modifiers(modifiers, drag_steps(*from, *to, *button, *steps, *step_ms)))
        }
        Command::Scroll { x, y, dx, dy, steps, step_ms, modifiers } => {
            return Some(with_modifiers(modifiers, scroll_steps(*x, *y, *dx, *dy, *steps, *step_ms)))
        }
        Command::Wheel { x, y, horizontal, vertical, modifiers } if !modifiers.is_empty() => {
            let wheel = vec![Step::Wheel { x: *x, y: *y, horizontal: *horizontal, vertical: *vertical }];
            return Some(with_modifiers(modifiers, wheel))
        }
        _ => return None,
    }
}

/// What:
///      `#[cfg(test)] #[path = "timeline_plan_tests.rs"] mod tests;`.
/// Why:
///       Keep the planner tests beside the planner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./timeline_plan_tests";
/// ```
#[cfg(test)]
#[path = "timeline_plan_tests.rs"]
mod tests;
