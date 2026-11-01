//! Compositor-originated drag-and-drop,
//!  for testing a hosted app's INBOUND file drop.
//!
//! A real inbound drag needs a second client (a file manager) to be the drag source.
//! Smithay 0.7 instead lets the compositor itself be the source via
//! [`start_dnd`](smithay::wayland::selection::data_device::start_dnd):
//!  it installs a
//! server pointer grab that,
//!  as the pointer moves over the hosted app's surface,
//!  sends
//! the app a `wl_data_offer` + `enter`,
//!  then (on button release) a `drop`.
//!  The app
//! then requests the data,
//!  which arrives in `ServerDndGrabHandler::send` (see
//! `handler.rs`).
//!  This module drives that sequence from a `drop-file` control command,
//! giving a deterministic,
//!  single-app inbound-drop test with no file manager involved.
//!
//! Why the release is deferred:
//!  Wayland DnD requires the target client to `accept` a
//! mime type and choose an action (`set_actions`) BETWEEN the enter and the drop.
//!  Those
//! are client round-trips.
//!  Releasing the button in the same synchronous call cancels the
//! drop as unvalidated,
//!  so the release is scheduled on a short dwell timer instead.

/// What:
///      `use std::{path::{Path, PathBuf}, time::Duration};`.
///  A borrowed filesystem path and a
///           span of time.
/// Why:
///       The drop source is named by path;
///  the dwell before release is a `Duration`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Path ~ string; Duration ~ a millisecond count.
/// ```
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

/// What:
///      `use anyhow::{anyhow, Context, Result};`.
///  Error construction (`anyhow!`),
///           context attachment (`.context`/`.with_context`),
///  and the `Result` alias.
/// Why:
///       `drop_file` reports human-readable failures to the control caller.
use anyhow::{anyhow, Context, Result};

/// What:
///      Grouped `use` of the input state enum,
///  the pointer event structs and the
///           server-grab start data,
///  the coordinate/serial utilities,
///  the calloop timer
///           types,
///  the `DndAction` bitflags,
///  and the `start_dnd` entry plus its
///           `SourceMetadata`.
/// Why:
///       Everything the synthetic drag sequence references.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ButtonState, MotionEvent, ButtonEvent, GrabStartData, startDnd, ... } from "smithay";
/// ```
use smithay::{
    backend::input::ButtonState,
    input::pointer::{ButtonEvent, GrabStartData as PointerGrabStartData, MotionEvent},
    reexports::{
        calloop::timer::{TimeoutAction, Timer},
        wayland_server::protocol::wl_data_device_manager::DndAction,
    },
    utils::{Logical, Point, SERIAL_COUNTER},
    wayland::selection::data_device::{start_dnd, SourceMetadata},
};

/// What:
///      `use tracing::info;`.
///  Structured info-level log macro.
/// Why:
///       Trace each stage of the drag so a failed drop can be localised.
use tracing::info;

/// What:
///      `use crate::{protocol::PointerButton, state::Compositor};`.
/// Why:
///       Reuse the button-to-evdev-code mapping and operate on the compositor state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { PointerButton } from "./protocol";
/// import { Compositor } from "./state";
/// ```
use crate::{protocol::PointerButton, state::Compositor};

/// What:
///      `const URI_LIST: &str = "text/uri-list";`.
///  The drag's advertised mime type.
/// Why:
///       File drops are carried as an RFC 2483 uri-list;
///  the app filters on this exact
///           string.
const URI_LIST: &str = "text/uri-list";

/// What:
///      `const DWELL_MS: u64 = 200;`.
///  Milliseconds the drag hovers before releasing.
/// Why:
///       Long enough for the app to `accept` + `set_actions` (the round-trips a real
///           drag needs) so the drop validates;
///  short enough to keep the test snappy.
const DWELL_MS: u64 = 200;

/// What:
///      `const NUDGE_PX: f64 = 1.0;`.
///  A one-pixel pointer move inside the grab.
/// Why:
///       A distinct motion after the grab installs makes the grab emit the data offer
///           + `enter` to the app.
const NUDGE_PX: f64 = 1.0;

/// Originate a compositor-side file drag toward the hosted app,
///  then release it after a
/// short dwell so the drop validates.
///
/// What:
///      `pub fn drop_file(state: &mut Compositor, path: &Path, x: Option<f64>, y:
///           Option<f64>) -> Result<()>`.
///  Mutably borrows the compositor state,
///  takes the
///           source file path,
///  and an optional logical drop point (both or neither).
/// Why:
///       Backs the `drop-file` control command:
///  a deterministic inbound-drop test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dropFile(state, path, x?, y?): void { ... } // throws on failure
/// ```
///
/// @example
/// ```ts
/// dropFile(state, "/tmp/hello.txt"); // drops at the window centre
/// ```
pub fn drop_file(state: &mut Compositor, paths: &[PathBuf], x: Option<f64>, y: Option<f64>) -> Result<()> {
    // What:     Resolve every path to an absolute, symlink-free path, failing the whole drop
    //           (naming the path) when one does not exist.
    // Why:      A `file:` URI must be absolute, and every file must exist to be dropped.
    let mut absolute_paths = Vec::with_capacity(paths.len());
    for path in paths {
        absolute_paths.push(
            path.canonicalize()
                .with_context(|| format!("resolving drop-file path {}", path.display()))?,
        );
    }
    let borrowed: Vec<&Path> = absolute_paths.iter().map(PathBuf::as_path).collect();

    // What:     `uri_list(&borrowed)`. One percent-encoded `file:` URI per line, CRLF-terminated
    //           per RFC 2483.
    // Why:      This is the exact payload the app parses back into filesystem paths; a space or a
    //           `%` in a name survives because the bytes are percent-encoded.
    let uri = crate::uri_list::uri_list(&borrowed);

    // What:     Stash the payload as owned bytes.
    // Why:      `ServerDndGrabHandler::send` (only `&mut Compositor` in scope) reads it when
    //           the app requests the data.
    state.pending_dnd_uri_list = Some(uri);

    // What:     `let (px, py) = match (x, y) { (Some(x), Some(y)) => (x, y), _ =>
    //           window_centre(state)? };`. Use the given point, or fall back to the window
    //           centre. `?` propagates the "no window" error.
    // Why:      A no-coordinate drop targets the middle of the app.
    let (px, py) = match (x, y) {
        (Some(point_x), Some(point_y)) => (point_x, point_y),
        _ => window_centre(state)?,
    };

    // What:     `let location: Point<f64, Logical> = (px, py).into();`. A logical-space
    //           point. `Point<f64, Logical>` is a compositor coordinate (siblings:
    //           `Point<i32, Logical>`, `Point<f64, Physical>`).
    // Why:      Pointer events and the grab start data are in logical coordinates.
    let location: Point<f64, Logical> = (px, py).into();

    // What:     `let (surface, surface_loc) = state.surface_under(location).ok_or_else(...)?;`.
    //           Hit-test which surface (and its origin) sits under the point; error if none.
    // Why:      The drag must target the app's surface; no surface means the app is not
    //           mapped yet.
    let (surface, surface_loc) = state
        .surface_under(location)
        .ok_or_else(|| anyhow!("no surface under ({px}, {py}); is the app mapped?"))?;

    // What:     `let pointer = state.seat.get_pointer().ok_or_else(...)?;`. Fetch the pointer
    //           handle (a cheap `Arc` clone, NOT a borrow of `state`, so `state` is still
    //           passable mutably below).
    // Why:      Motion / button events are sent through this handle.
    let pointer = state
        .seat
        .get_pointer()
        .ok_or_else(|| anyhow!("seat has no pointer"))?;

    // What:     `let button = PointerButton::Left.evdev_code();`. The `BTN_LEFT` evdev code.
    // Why:      The drag is a held-left-button gesture.
    let button = PointerButton::Left.evdev_code();

    // What:     `let time = event_time(state);`. A millisecond timestamp for the events.
    // Why:      Every pointer event needs a monotonic-ish time.
    let time = event_time(state);

    // What:     `pointer.motion(state, Some((surface.clone(), surface_loc)), &MotionEvent {
    //           ... });`. Move the pointer onto the surface. `Some((surface.clone(), loc))`
    //           is the pointer focus; `surface.clone()` bumps the surface's refcount because
    //           it is reused below. `SERIAL_COUNTER.next_serial()` mints a fresh event serial.
    // Why:      Establish pointer focus so the following button press is delivered there.
    pointer.motion(
        state,
        Some((surface.clone(), surface_loc)),
        &MotionEvent {
            location,
            serial: SERIAL_COUNTER.next_serial(),
            time,
        },
    );

    // What:     `pointer.frame(state);`. End the pointer event group.
    // Why:      Clients apply pointer events on frame boundaries.
    pointer.frame(state);

    // What:     `let press_serial = SERIAL_COUNTER.next_serial();`. Serial for the press.
    // Why:      `start_dnd` uses this serial to anchor the grab to the button press.
    let press_serial = SERIAL_COUNTER.next_serial();

    // What:     `pointer.button(state, &ButtonEvent { state: ButtonState::Pressed, ... });`.
    //           Press the left button, recording it as held.
    // Why:      The grab ends when no button is held, so the button must be down first.
    pointer.button(
        state,
        &ButtonEvent {
            button,
            state: ButtonState::Pressed,
            serial: press_serial,
            time,
        },
    );
    pointer.frame(state);

    // What:     `let dh = state.display_handle.clone(); let seat = state.seat.clone();`. Cheap
    //           handle clones (both `Arc`-backed).
    // Why:      `start_dnd` borrows the display handle and seat while also taking `state`
    //           mutably; cloning first avoids aliasing the same `state`.
    let dh = state.display_handle.clone();
    let seat = state.seat.clone();

    // What:     `let metadata = SourceMetadata { mime_types: vec![URI_LIST.to_string()],
    //           dnd_action: DndAction::Copy };`. Advertise the one mime type and a Copy
    //           action. `DndAction::Copy` is a bitflag (siblings: `Move`, `Ask`, `None`).
    // Why:      The app filters on `text/uri-list` and negotiates a Copy action.
    let metadata = SourceMetadata {
        mime_types: vec![URI_LIST.to_string()],
        dnd_action: DndAction::Copy,
    };

    // What:     `let start_data = PointerGrabStartData { focus: Some((surface.clone(),
    //           surface_loc)), button, location };`. The grab's anchor: what was focused, the
    //           held button, and where the press landed.
    // Why:      Smithay's server grab records this as the drag's start point.
    let start_data = PointerGrabStartData {
        focus: Some((surface.clone(), surface_loc)),
        button,
        location,
    };

    // What:     `start_dnd(&dh, &seat, state, press_serial, Some(start_data), None,
    //           metadata);`. Install the server DnD pointer grab. `Some(start_data)` is the
    //           pointer start; `None` is the (unused) touch start.
    // Why:      From here, pointer motion is turned into data-device enter/motion/drop toward
    //           the app.
    start_dnd(&dh, &seat, state, press_serial, Some(start_data), None, metadata);

    // What:     `let move_time = event_time(state);`. Timestamp for the in-grab motion.
    // Why:      A fresh time for the nudge event.
    let move_time = event_time(state);

    // What:     `let nudged: Point<f64, Logical> = (px + NUDGE_PX, py + NUDGE_PX).into();`. A
    //           one-pixel move still over the surface.
    // Why:      A distinct motion inside the grab makes it emit the data offer + `enter`.
    let nudged: Point<f64, Logical> = (px + NUDGE_PX, py + NUDGE_PX).into();

    // What:     `pointer.motion(state, Some((surface, surface_loc)), &MotionEvent { ... });`.
    //           Move within the grab. `surface` is moved (not cloned) as it is its last use.
    // Why:      Trigger the app's inbound `enter` with the offer.
    pointer.motion(
        state,
        Some((surface, surface_loc)),
        &MotionEvent {
            location: nudged,
            serial: SERIAL_COUNTER.next_serial(),
            time: move_time,
        },
    );
    pointer.frame(state);

    // What:     `info!(...)`. Log that the drag is now hovering over the app.
    // Why:      Mark the boundary between "offer sent" and "awaiting the app's accept".
    info!("drop-file: drag hovering over the app; releasing in {DWELL_MS}ms");

    // What:     `schedule_release(state, button)?;`. Arm the deferred button release.
    // Why:      Give the app time to accept + choose an action before the drop.
    schedule_release(state, button)?;

    // What:     `Ok(())`. The drag is under way; the drop completes on the timer.
    // Why:      Report success to the control caller.
    return Ok(())
}

/// Compute the centre of the single hosted window in logical coordinates.
///
/// What:
///      `fn window_centre(state: &Compositor) -> Result<(f64, f64)>`.
///  Read-only borrow;
///           returns the `(x, y)` centre or an error when no window is mapped.
/// Why:
///       A coordinate-less `drop-file` targets the middle of the app.
fn window_centre(state: &Compositor) -> Result<(f64, f64)> {
    // What:     `let window = state.space.elements().next().ok_or_else(...)?;`. The first (and
    //           only) mapped window; error if none.
    // Why:      There is nothing to target before the app maps its toplevel.
    let window = state
        .space
        .elements()
        .next()
        .ok_or_else(|| anyhow!("no window mapped; the app has not created its toplevel yet"))?;

    // What:     `let bbox = state.space.element_bbox(window).ok_or_else(...)?;`. The window's
    //           bounding box (`Rectangle<i32, Logical>`); error if it has none.
    // Why:      The centre is derived from the box's origin and size.
    let bbox = state
        .space
        .element_bbox(window)
        .ok_or_else(|| anyhow!("mapped window has no bounding box"))?;

    // What:     `let cx = f64::from(bbox.loc.x) + f64::from(bbox.size.w) / 2.0;`. Horizontal
    //           centre. `f64::from(i32)` is a lossless widening conversion (`.loc.x` is the
    //           left edge, `.size.w` the width).
    // Why:      Middle of the window horizontally.
    let cx = f64::from(bbox.loc.x) + f64::from(bbox.size.w) / 2.0;

    // What:     `let cy = f64::from(bbox.loc.y) + f64::from(bbox.size.h) / 2.0;`. Vertical
    //           centre.
    // Why:      Middle of the window vertically.
    let cy = f64::from(bbox.loc.y) + f64::from(bbox.size.h) / 2.0;

    // What:     `Ok((cx, cy))`. The centre point (tail expression).
    // Why:      Hand the drop point back to `drop_file`.
    return Ok((cx, cy))
}

/// Schedule the drag's button release after [`DWELL_MS`] on the event loop.
///
/// What:
///      `fn schedule_release(state: &mut Compositor, button: u32) -> Result<()>`.
///           Inserts a one-shot calloop timer that releases `button`.
/// Why:
///       The release must happen after the app's accept round-trips,
///  not synchronously.
fn schedule_release(state: &mut Compositor, button: u32) -> Result<()> {
    // What:     `let timer = Timer::from_duration(Duration::from_millis(DWELL_MS));`. A timer
    //           that fires once after the dwell.
    // Why:      Delay the release without blocking the event loop.
    let timer = Timer::from_duration(Duration::from_millis(DWELL_MS));

    // What:     `state.loop_handle.insert_source(timer, move |_, _, state| { ... })
    //           .map_err(...)?;`. Register the timer; its callback runs on the main thread
    //           with `&mut Compositor`. `move` captures `button` (a `Copy` `u32`) by value.
    // Why:      Perform the release where the seat lives, once the dwell elapses.
    state
        .loop_handle
        .insert_source(timer, move |_, _, event_state: &mut Compositor| {
            // What:     release_drag sends the deferred button release through the current seat.
            // Why:      The callback receives event-loop state, not the registration borrow.
            release_drag(event_state, button);

            // What:     `TimeoutAction::Drop`. Tell calloop not to re-arm the timer.
            // Why:      The release is a one-shot.
            return TimeoutAction::Drop
        })
        .map_err(|err| anyhow!("scheduling the drop-file release timer failed: {err}"))?;

    // What:     `Ok(())`. The timer is armed.
    // Why:      Signal the release is scheduled.
    return Ok(())
}

/// Release the held left button,
///  ending the server grab and delivering the drop.
///
/// What:
///      `fn release_drag(state: &mut Compositor, button: u32)`.
///  Injects a button
///           release through the seat's pointer.
/// Why:
///       The drop fires when the last button is released;
///  the app then requests the data.
fn release_drag(state: &mut Compositor, button: u32) {
    // What:     `let Some(pointer) = state.seat.get_pointer() else { return; };`. The pointer
    //           handle, or bail if the seat lost its pointer (it should not).
    // Why:      Nothing to release without a pointer.
    let Some(pointer) = state.seat.get_pointer() else {
        return;
    };

    // What:     `let time = event_time(state);`. Timestamp for the release.
    // Why:      The release event needs a time.
    let time = event_time(state);

    // What:     `pointer.button(state, &ButtonEvent { state: ButtonState::Released, ... });`.
    //           Release the button; the grab sees no buttons held and performs the drop.
    // Why:      Complete the drag.
    pointer.button(
        state,
        &ButtonEvent {
            button,
            state: ButtonState::Released,
            serial: SERIAL_COUNTER.next_serial(),
            time,
        },
    );
    pointer.frame(state);

    // What:     `state.held_pointer_buttons.release(button);`. Keep the `button down` record in
    //           step with the release the drag just delivered.
    // Why:      A later `button up` must not answer `ok` for a button the seat has let go.
    state.held_pointer_buttons.release(button);

    // What:     `info!(...)`. Log the release.
    // Why:      Mark that the drop was delivered (the app's own log confirms receipt).
    info!("drop-file: released the drag; drop delivered to the app");
}

/// Milliseconds since program start,
///  used as an event timestamp.
///
/// What:
///      `fn event_time(state: &Compositor) -> u32`.
///  Read-only borrow;
///  returns a 32-bit
///           millisecond count.
///  `.as_millis()` yields a 128-bit integer,
///  narrowed with `as
///           u32` (Wayland event times are 32-bit and wrap,
///  which clients tolerate).
/// Why:
///       Mirrors `input::event_time`;
///  kept local so this module does not reach into the
///           input module's private helper.
fn event_time(state: &Compositor) -> u32 {
    // What:     `state.start_time.elapsed().as_millis() as u32`. Elapsed ms, narrowed. Tail
    //           expression.
    // Why:      Provide the event timestamp.
    return state.start_time.elapsed().as_millis() as u32
}
