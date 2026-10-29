//! Timed input sequences:
//!  a key held for a duration,
//!  a drag with intermediate motion,
//!  a double click.
//!
//! The control thread waits for one reply per command,
//!  so a timed sequence keeps its reply channel and answers when the last step has run.
//!  Steps run on the compositor's event loop between waits,
//!  so the hosted client keeps being served while a key is held.

/// Standard library items.
use std::{collections::VecDeque, sync::mpsc::SyncSender, time::Duration};

/// Error handling with context.
use anyhow::Result;
/// Smithay compositor types.
use smithay::reexports::calloop::timer::{TimeoutAction, Timer};
/// Structured logging.
use tracing::{debug, warn};

/// Sibling modules of this crate.
use crate::{
    input, input_pointer, input_wheel,
    protocol::{ButtonAction, PointerButton, Response},
    state::Compositor,
};

/// Smithay compositor types.
use smithay::backend::input::KeyState;

/// One action of a timed input sequence.
///
/// What:
///      `pub enum Step { KeyDown(u32), KeyUp(u32), MoveTo { x, y }, ButtonDown(PointerButton), ButtonUp(PointerButton), Wheel { .. }, Scroll { .. }, ScrollStop { .. }, Wait(u64) }`.
/// Why:
///       A command becomes a list of these,
///  so hold,
///  drag,
///  and multi-click share one runner and one test seam.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Step = { key: "down" | "up"; code: number } | { move: [number, number] } | { wait: number } | ...;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    /// Press a key by evdev code.
    KeyDown(
        /// Evdev code.
        u32,
    ),
    /// Release a key by evdev code.
    KeyUp(
        /// Evdev code.
        u32,
    ),
    /// Move the pointer to a logical point.
    MoveTo {
        /// Logical x coordinate.
        x: f64,
        /// Logical y coordinate.
        y: f64,
    },
    /// Press a pointer button where the pointer is.
    ButtonDown(
        /// Which button.
        PointerButton,
    ),
    /// Release a pointer button.
    ButtonUp(
        /// Which button.
        PointerButton,
    ),
    /// Wheel notches at a logical point.
    Wheel {
        /// Logical x coordinate.
        x: f64,
        /// Logical y coordinate.
        y: f64,
        /// Horizontal notches,
        ///  fractions allowed.
        horizontal: f64,
        /// Vertical notches,
        ///  fractions allowed.
        vertical: f64,
    },
    /// Touchpad pixel scroll at a logical point.
    Scroll {
        /// Logical x coordinate.
        x: f64,
        /// Logical y coordinate.
        y: f64,
        /// Horizontal logical pixels.
        dx: f64,
        /// Vertical logical pixels.
        dy: f64,
    },
    /// The fingers leave the touchpad.
    ScrollStop,
    /// Pause for milliseconds before the next step.
    Wait(
        /// Milliseconds.
        u64,
    ),
}

/// Classification of steps.
impl Step {
    /// Whether the step releases something.
    ///
    /// What:
    ///      `fn releases(&self) -> bool`.
    /// Why:
    ///       An aborted sequence still runs its release steps,
    ///  so no key or button stays held.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const releases = (step: Step) => step.key === "up" || step.button === "up";
    /// ```
    #[must_use]
    pub fn releases(&self) -> bool {
        return matches!(self, Step::KeyUp(_) | Step::ButtonUp(_) | Step::ScrollStop)
    }
}

/// Run one step.
///
/// What:
///      `fn run_step(state: &mut Compositor, step: &Step) -> Result<()>`.
/// Why:
///       Maps each step onto the seat injection functions the immediate commands use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runStep(state: Compositor, step: Step): void { ... }
/// ```
fn run_step(state: &mut Compositor, step: &Step) -> Result<()> {
    match step {
        Step::KeyDown(code) => input::send_key(state, *code, KeyState::Pressed),
        Step::KeyUp(code) => input::send_key(state, *code, KeyState::Released),
        Step::MoveTo { x, y } => input_pointer::move_pointer(state, *x, *y)?,
        Step::ButtonDown(button) => input_pointer::button(state, ButtonAction::Down, *button)?,
        Step::ButtonUp(button) => input_pointer::button(state, ButtonAction::Up, *button)?,
        Step::Wheel { x, y, horizontal, vertical } => input_wheel::wheel(state, *x, *y, *horizontal, *vertical)?,
        Step::Scroll { x, y, dx, dy } => input_wheel::scroll(state, *x, *y, *dx, *dy)?,
        Step::ScrollStop => input_wheel::scroll_stop(state)?,
        Step::Wait(_) => {}
    }
    return Ok(())
}

/// Run steps until the next wait,
///  then ask to be called again after it.
///
/// What:
///      `fn advance(state, queue, reply) -> TimeoutAction`.
///  A failing step releases whatever the remaining steps would have released,
///  answers `err`,
///  and stops.
/// Why:
///       The timer callback must be a small state machine,
///  and a failure must not leave a modifier key down in the client.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function advance(state: Compositor, queue: Step[], reply: Reply): TimeoutAction { ... }
/// ```
fn advance(state: &mut Compositor, queue: &mut VecDeque<Step>, reply: &SyncSender<Response>) -> TimeoutAction {
    while let Some(step) = queue.pop_front() {
        if let Step::Wait(millis) = step {
            return TimeoutAction::ToDuration(Duration::from_millis(millis))
        }
        if let Err(error) = run_step(state, &step) {
            warn!("timed input step {step:?} failed: {error:#}");
            while let Some(rest) = queue.pop_front() {
                if !rest.releases() {
                    continue;
                }
                if let Err(release_error) = run_step(state, &rest) {
                    debug!("release after a failed step also failed: {release_error:#}");
                }
            }
            let _ = reply.send(Response::Err(format!("{error:#}")));
            return TimeoutAction::Drop
        }
    }
    debug!("timed input sequence finished");
    let _ = reply.send(Response::Ok);
    return TimeoutAction::Drop
}

/// Start a timed sequence and answer `reply` when it has finished.
///
/// What:
///      `pub fn start(state: &mut Compositor, steps: Vec<Step>, reply: SyncSender<Response>)`.
///  The first steps run on the next loop turn.
/// Why:
///       Lets the event loop keep serving the client during the waits,
///  where a blocking sleep would freeze every surface.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function start(state: Compositor, steps: Step[], reply: (r: Response) => void): void { ... }
/// ```
pub fn start(state: &mut Compositor, steps: Vec<Step>, reply: SyncSender<Response>) {
    let mut queue: VecDeque<Step> = steps.into();
    let callback_reply = reply.clone();
    let inserted = state.loop_handle.insert_source(Timer::immediate(), move |_, _, event_state: &mut Compositor| {
        return advance(event_state, &mut queue, &callback_reply)
    });
    if let Err(error) = inserted {
        warn!("scheduling a timed input sequence failed: {error}");
        let _ = reply.send(Response::Err(format!("scheduling the input sequence failed: {error}")));
    }
}
