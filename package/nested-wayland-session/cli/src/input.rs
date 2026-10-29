//! Synthetic pointer and keyboard input injected through the compositor's own seat.
//!
//! Because the fixture owns the seat,
//!  injecting input is a direct in-process call into
//! the seat's pointer and keyboard handles:
//!  no `/dev/uinput`,
//!  no global input,
//!  and the
//! events reach only the hosted client.
//!  Coordinates are logical;
//!  keys are evdev codes
//! (translated to the xkb keycode system by adding the 8-offset winit also applies).

/// What:
///      Grouped `use` of the input state enums,
///  keyboard focus/keycode types,
///  the
///           pointer event structs,
///  and the coordinate/serial utilities.
/// Why:
///       Everything the injection functions reference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ButtonState, KeyState, Keycode, MotionEvent, ButtonEvent, ... } from "smithay";
/// ```
use smithay::{
    backend::input::{ButtonState, KeyState},
    input::{
        keyboard::{FilterResult, Keycode},
        pointer::{ButtonEvent, MotionEvent},
    },
    utils::{Logical, Point, SERIAL_COUNTER},
};

/// What:
///      `use crate::{protocol::PointerButton, state::Compositor};`.
/// Why:
///       Injection reads the protocol's button enum and operates on the compositor state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { PointerButton } from "./protocol";
/// import { Compositor } from "./state";
/// ```
use crate::{protocol::PointerButton, state::Compositor};

/// Milliseconds since program start,
///  used as the event timestamp.
///
/// What:
///      `fn event_time(state: &Compositor) -> u32`.
///  Read-only borrow;
///  returns a
///           32-bit millisecond count.
///  `.as_millis()` yields a 128-bit integer,
///  cast to
///           `u32` (Wayland event times are 32-bit and wrap,
///  which clients tolerate).
/// Why:
///       Every synthetic event needs a monotonic-ish timestamp.
fn event_time(state: &Compositor) -> u32 {
    // What:     `state.start_time.elapsed().as_millis() as u32`. Elapsed time, in ms,
    //           narrowed to `u32`. Tail expression.
    // Why:      Provide the event timestamp.
    return state.start_time.elapsed().as_millis() as u32
}

/// Click a button at a logical point:
///  move the pointer there,
///  press,
///  and release.
///
/// What:
///      `pub fn click(state: &mut Compositor, x: f64, y: f64, button:
///           PointerButton)`.
///  Mutably borrows the state;
///  `x`/`y` are logical
///           coordinates.
/// Why:
///       The `click` control command lands a full press+release at a spot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function click(state, x, y, button) { ... }
/// ```
///
/// @example
/// ```ts
/// click(state, 100, 40, PointerButton.Left);
/// ```
pub fn click(state: &mut Compositor, x: f64, y: f64, button: PointerButton) {
    // What:     `let pointer = state.seat.get_pointer().unwrap();`. Fetch the pointer
    //           handle. This is a cheap reference-counted clone, NOT a borrow of the seat,
    //           so `state` can still be passed mutably below.
    // Why:      Need the handle to send motion/button events.
    let pointer = state.seat.get_pointer().unwrap();

    // What:     `let location: Point<f64, Logical> = (x, y).into();`. Build a logical
    //           point from the coordinates.
    // Why:      Pointer events use logical coordinates.
    let location: Point<f64, Logical> = (x, y).into();

    // What:     `let under = state.surface_under(location);`. Hit-test which surface is at
    //           that point (`Option<(WlSurface, Point)>`). Borrows `state` read-only only
    //           for this statement.
    // Why:      Motion sets the pointer focus to whatever is under the cursor.
    let under = state.surface_under(location);

    // What:     `let time = event_time(state);`. Timestamp for all three events.
    // Why:      Consistent timing across the click.
    let time = event_time(state);

    // What:     `pointer.motion(state, under, &MotionEvent { location, serial:
    //           SERIAL_COUNTER.next_serial(), time });`. Move the pointer, setting focus to
    //           `under`. `SERIAL_COUNTER.next_serial()` mints a fresh event serial.
    // Why:      The button events must land on a focused surface.
    pointer.motion(
        state,
        under,
        &MotionEvent {
            location,
            serial: SERIAL_COUNTER.next_serial(),
            time,
        },
    );

    // What:     `pointer.frame(state);`. Marks the end of a pointer event group.
    // Why:      Clients apply pointer events on frame boundaries.
    pointer.frame(state);

    // What:     `let code = button.evdev_code();`. The `BTN_*` code for this button.
    // Why:      `ButtonEvent.button` is the raw evdev code.
    let code = button.evdev_code();

    // What:     `pointer.button(state, &ButtonEvent { button: code, state:
    //           ButtonState::Pressed, serial: ..., time });`. Press the button.
    // Why:      Begin the click.
    pointer.button(
        state,
        &ButtonEvent {
            button: code,
            state: ButtonState::Pressed,
            serial: SERIAL_COUNTER.next_serial(),
            time,
        },
    );

    // What:     `pointer.frame(state);`. End the press group.
    // Why:      Flush the press.
    pointer.frame(state);

    // What:     `pointer.button(state, &ButtonEvent { ..., state: ButtonState::Released,
    //           ... });`. Release the button.
    // Why:      Complete the click.
    pointer.button(
        state,
        &ButtonEvent {
            button: code,
            state: ButtonState::Released,
            serial: SERIAL_COUNTER.next_serial(),
            time,
        },
    );

    // What:     `pointer.frame(state);`. End the release group.
    // Why:      Flush the release.
    pointer.frame(state);

    // What:     `state.held_pointer_buttons.release(code);`. Forget the button in the record
    //           `button down` keeps, because the release above also ends a held press.
    // Why:      The record must match what the seat delivered, or a later `button up` would
    //           answer `ok` for a button the seat no longer holds.
    state.held_pointer_buttons.release(code);
}

/// Send one keyboard key event (press or release) through the seat's keyboard.
///
/// What:
///      `pub fn send_key(state: &mut Compositor, evdev: u32, key_state: KeyState)`.
/// Why:
///       Both `key` and `type_text` funnel through one place that does the 8-offset
///           and the seat call.
pub fn send_key(state: &mut Compositor, evdev: u32, key_state: KeyState) {
    // What:     `let keyboard = state.seat.get_keyboard().unwrap();`. The keyboard handle
    //           (a reference-counted clone, not a borrow of the seat).
    // Why:      Need it to inject the key.
    let keyboard = state.seat.get_keyboard().unwrap();

    // What:     `let time = event_time(state);`. Timestamp before the mutable seat call.
    // Why:      The event needs a time and this borrow must end before the `&mut state`
    //           call below.
    let time = event_time(state);

    // What:     `let keycode: Keycode = (evdev + 8).into();`. Convert the evdev code to the
    //           xkb keycode by adding 8 (the X11 keycode offset winit's backend also
    //           applies) and `into()`-ing it to `Keycode`.
    // Why:      Smithay's keyboard state is keyed by xkb keycodes, not raw evdev codes.
    let keycode: Keycode = (evdev + 8).into();

    // What:     `keyboard.input::<(), _>(state, keycode, key_state,
    //           SERIAL_COUNTER.next_serial(), time, |_, _, _| FilterResult::Forward);`. Feed
    //           the event. The turbofish `::<(), _>` sets the filter's return payload type
    //           to `()` and infers the closure type. The filter `|_, _, _|
    //           FilterResult::Forward` always forwards the key to the focused client (no
    //           compositor shortcut handling).
    // Why:      Deliver the synthetic key to the hosted app.
    keyboard.input::<(), _>(
        state,
        keycode,
        key_state,
        SERIAL_COUNTER.next_serial(),
        time,
        |_, _, _| return FilterResult::Forward,
    );
}
