//! Pointer hover and held-button injection through the nested seat.
//!
//! `move` moves the pointer without a press, so the hosted client sees real enter, motion,
//!  and leave events.
//!  `button` presses or releases at the current pointer position and keeps the button held
//!  until a matching release, so a test can pause or move between the two.

/// Application diagnostics describe a button that is already in the requested state.
use anyhow::{bail, Context, Result};

/// Smithay's pointer events and the seat's serial counter.
use smithay::{
    backend::input::ButtonState,
    input::pointer::{ButtonEvent, MotionEvent},
    utils::{Logical, Point, SERIAL_COUNTER},
};

/// Only the nested compositor's seat is touched.
use crate::{
    protocol::{ButtonAction, PointerButton},
    state::Compositor,
};

/// Buttons currently held by `button down`, by evdev code.
///
/// What:
///      `pub struct HeldButtons { codes: Vec<u32> }`.
///  A small list of the `BTN_*` codes that are down.
/// Why:
///       Smithay keeps its own pressed list private to grab handlers,
///  so the compositor records what it pressed and answers `button` requests from that record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class HeldButtons { private codes: number[] = []; }
/// ```
#[derive(Debug, Default)]
pub struct HeldButtons {
    /// Evdev codes of the buttons that are down, in press order.
    codes: Vec<u32>,
}

/// Recording presses and releases so the record matches what the seat delivered.
impl HeldButtons {
    /// Reports whether a button is down.
    ///
    /// What:
    ///      `pub fn contains(&self, code: u32) -> bool`.
    /// Why:
    ///       `button down` on a held button and `button up` on a free one must answer `err`.
    #[must_use]
    pub fn contains(&self, code: u32) -> bool {
        return self.codes.contains(&code)
    }

    /// Records a press, keeping one entry per button.
    ///
    /// What:
    ///      `fn press(&mut self, code: u32)`.
    /// Why:
    ///       A second press of a held button must not add a duplicate entry.
    fn press(&mut self, code: u32) {
        if !self.contains(code) {
            self.codes.push(code);
        }
    }

    /// Records a release, which also clears the button after a `click` releases it.
    ///
    /// What:
    ///      `pub fn release(&mut self, code: u32)`.
    /// Why:
    ///       Every path that releases a button goes through here,
    ///  so the record cannot claim a button is down after the seat released it.
    pub fn release(&mut self, code: u32) {
        self.codes.retain(|held| return *held != code);
    }
}

/// Move the pointer to a logical point without any button event.
///
/// What:
///      `pub fn move_pointer(state: &mut Compositor, x: f64, y: f64) -> Result<()>`.
///  The coordinates must be finite; the protocol parser already enforces that.
/// Why:
///       Motion alone changes the pointer focus,
///  which is how the hosted client receives enter and leave events.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pointer.motion(x, y); // no button, so no press and no release
/// ```
pub fn move_pointer(state: &mut Compositor, x: f64, y: f64) -> Result<()> {
    let pointer = state.seat.get_pointer().context("nested seat has no pointer")?;
    let location: Point<f64, Logical> = (x, y).into();
    let under = state.surface_under(location);
    let time = state.start_time.elapsed().as_millis() as u32;
    pointer.motion(
        state,
        under,
        &MotionEvent {
            location,
            serial: SERIAL_COUNTER.next_serial(),
            time,
        },
    );
    pointer.frame(state);
    tracing::debug!(x, y, "moved the nested pointer without a button");
    return Ok(())
}

/// Press or release a button at the pointer's current position.
///
/// What:
///      `pub fn button(state: &mut Compositor, action: ButtonAction, button: PointerButton) ->
///           Result<()>`.
///  Fails when the button is already down for `Down`, or already up for `Up`.
/// Why:
///       The state check keeps a repeated request from sending a second press
///  that the hosted client would read as a double click.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function button(action, button) { if (held(button) === (action === "down")) throw ...; }
/// ```
pub fn button(state: &mut Compositor, action: ButtonAction, button: PointerButton) -> Result<()> {
    let code = button.evdev_code();
    let held = state.held_pointer_buttons.contains(code);
    match (action, held) {
        (ButtonAction::Down, true) => bail!("{} button is already down", button.name()),
        (ButtonAction::Up, false) => bail!("{} button is not down", button.name()),
        (ButtonAction::Down, false) | (ButtonAction::Up, true) => {}
    }
    let pointer = state.seat.get_pointer().context("nested seat has no pointer")?;
    let time = state.start_time.elapsed().as_millis() as u32;
    let button_state = match action {
        ButtonAction::Down => ButtonState::Pressed,
        ButtonAction::Up => ButtonState::Released,
    };
    pointer.button(
        state,
        &ButtonEvent {
            button: code,
            state: button_state,
            serial: SERIAL_COUNTER.next_serial(),
            time,
        },
    );
    pointer.frame(state);
    match action {
        ButtonAction::Down => state.held_pointer_buttons.press(code),
        ButtonAction::Up => state.held_pointer_buttons.release(code),
    }
    tracing::debug!(code, ?action, "sent a nested pointer button event");
    return Ok(())
}
