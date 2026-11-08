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
//!
//! Every click writes one `info` line naming the surface the hit test found,
//!  the number of windows on
//! screen and the cursor the hosted client last asked for,
//!  then a second one naming the surface the seat
//! kept the pointer focus on.
//!  A click that changes nothing in the client can then be read from this side of the
//! boundary:
//!  no surface under the point,
//!  a point over a client-drawn popup that takes no input,
//!  and a
//! press delivered to a client that had already taken the pointer there are three different stories.

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
        keyboard::{FilterResult, Keycode, KeysymHandle, ModifiersState},
        pointer::{ButtonEvent, MotionEvent},
    },
    reexports::wayland_server::{
        protocol::wl_surface::WlSurface,
        Resource,
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

    // What:     `let hit = under.as_ref().map(hit_note);`. The protocol id of the hit surface
    //           with its origin.
    // Why:      `WlSurface` prints no useful identity, and the protocol id matches the ids the
    //           hosted client's own `WAYLAND_DEBUG` lines carry.
    let hit = under.as_ref().map(hit_note);

    // What:     `tracing::info!` with the point, the hit, the window count, and the cursor the
    //           hosted client last asked for.
    // Why:      The end-to-end failure file keeps this line beside the app's own log, so a click
    //           that changed nothing says whether the seat had a target at all and whether the app
    //           had already taken the pointer there.
    tracing::info!(
        x,
        y,
        hit = ?hit,
        windows = state.space.elements().count(),
        cursor = %crate::control_session::describe_cursor(&state.cursor_image),
        "sending a click: motion, press and release",
    );

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

    // What:     `tracing::info!` with the surface the pointer focus ended on, read after the
    //           release.
    // Why:      A button event reaches a client only through the pointer's focus, so a click that
    //           left the seat focused on nothing was delivered to nobody whatever the hit test found.
    tracing::info!(
        focus = ?pointer.current_focus().as_ref().map(protocol_id_of),
        "the click left the seat's pointer focus",
    );
}

/// What:
///      `fn hit_note((surface, point): &(WlSurface, Point<f64, Logical>)) -> (u32, f64, f64)`.
///  The protocol id of a hit surface with the point the hit test returned for it.
/// Why:
///       A click line names its target by protocol id,
///  the only name that matches the hosted client's own
/// `WAYLAND_DEBUG` lines.
fn hit_note((surface, point): &(WlSurface, Point<f64, Logical>)) -> (u32, f64, f64) {
    return (surface.id().protocol_id(), point.x, point.y);
}

/// What:
///      `fn protocol_id_of(surface: &WlSurface) -> u32`.
///  The protocol id of one surface.
/// Why:
///       The pointer focus is logged by id,
///  so a reader can match it against the target the same click named.
fn protocol_id_of(surface: &WlSurface) -> u32 {
    return surface.id().protocol_id();
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
    //           SERIAL_COUNTER.next_serial(), time, forward_every_key);`. Feed
    //           the event. The turbofish `::<(), _>` sets the filter's return payload type
    //           to `()` and infers the callable type. The filter `forward_every_key`
    //           always forwards the key to the focused client (no
    //           compositor shortcut handling).
    // Why:      Deliver the synthetic key to the hosted app.
    keyboard.input::<(), _>(
        state,
        keycode,
        key_state,
        SERIAL_COUNTER.next_serial(),
        time,
        forward_every_key,
    );
}

/// The keyboard filter that forwards every key to the focused client.
///
/// What:
///      `fn forward_every_key(_state: &mut Compositor, _modifiers: &ModifiersState, _handle: KeysymHandle<'_>) -> FilterResult<()>`.
///  Always `FilterResult::Forward`.
/// Why:
///       There is no compositor shortcut handling;
///  a named function replaces the closure that ignored its three arguments.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const forwardEveryKey = (_state, _modifiers, _handle): FilterResult => "forward";
/// ```
fn forward_every_key(_state: &mut Compositor, _modifiers: &ModifiersState, _handle: KeysymHandle<'_>) -> FilterResult<()> {
    return FilterResult::Forward
}
