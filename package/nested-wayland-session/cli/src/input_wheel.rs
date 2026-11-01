//! Native wheel and touchpad scroll injection through the nested seat,
//!  not a client test API.

/// Application diagnostics describe malformed or currently unmapped input.
use anyhow::{bail, Context, Result};
/// Smithay distinguishes wheel axes from touchpad/finger gestures.
use smithay::{
    backend::input::{Axis, AxisSource},
    input::pointer::{AxisFrame, MotionEvent},
    utils::{Logical, Point, SERIAL_COUNTER},
};
/// Only the nested compositor's seat is touched.
use crate::state::Compositor;

/// Wayland `axis_value120` units in one wheel notch.
///
/// What:
///      `const V120_PER_NOTCH: f64 = 120.0;`.
/// Why:
///       `wl_pointer.axis_value120` counts 1/120 notch,
///  so a fractional notch is a smaller whole number of these units.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const V120_PER_NOTCH = 120;
/// ```
const V120_PER_NOTCH: f64 = 120.0;

/// Logical pixels a real wheel reports per notch in the continuous `axis` value.
///
/// What:
///      `const PIXELS_PER_NOTCH: f64 = 10.0;`.
/// Why:
///       Clients without `axis_value120` support scroll by this continuous value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PIXELS_PER_NOTCH = 10;
/// ```
const PIXELS_PER_NOTCH: f64 = 10.0;

/// Convert notches to Wayland v120 units.
///
/// What:
///      `pub fn notches_to_v120(notches: f64) -> i32`.
///  Rounds to the nearest unit,
///  so a quarter notch is 30 and anything under a quarter of a unit is 0.
/// Why:
///       The conversion is the rule a fractional wheel command depends on,
///  and it is tested without a display.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notchesToV120 = (notches: number) => Math.round(notches * 120);
/// ```
#[must_use]
pub fn notches_to_v120(notches: f64) -> i32 {
    return (notches * V120_PER_NOTCH).round() as i32
}

/// Move the pointer to a logical point,
///  failing when no client surface is there.
///
/// What:
///      `fn point_pointer_at(state: &mut Compositor, x: f64, y: f64, what: &str) -> Result<()>`.
/// Why:
///       Wheel and scroll deliver to the surface under the pointer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pointPointerAt(state: Compositor, x: number, y: number): void { ... }
/// ```
fn point_pointer_at(state: &mut Compositor, x: f64, y: f64, what: &str) -> Result<()> {
    if !x.is_finite() || !y.is_finite() {
        bail!("{what} coordinates must be finite");
    }
    let pointer = state.seat.get_pointer().context("nested seat has no pointer")?;
    // Logical coordinates are local to the nested output, never the host desktop.
    let location: Point<f64, Logical> = (x, y).into();
    let under = state.surface_under(location);
    if under.is_none() {
        bail!(
            "no nested client surface at {what} position {x},{y}: no hosted window covers that point, \
             and before its first frame (the client's first committed buffer) no window covers any; \
             nothing was sent to the client. Wait until `windows` lists a window, or check the point lies inside the window, \
             then send it again"
        );
    }
    let time = state.start_time.elapsed().as_millis() as u32;
    pointer.motion(state, under, &MotionEvent {
        location, serial: SERIAL_COUNTER.next_serial(), time,
    });
    return Ok(())
}

/// Inject wheel notches at logical coordinates;
///  positive means down/right.
///
/// What:
///  Each notch becomes 120 discrete Wayland units plus its logical delta;
///  a fraction becomes a proportionally smaller number of units.
/// Why:
///  Clients can distinguish a physical wheel from cancelled/programmatic scroll,
///  and a high-resolution wheel sends partial notches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pointer.move(x, y);
/// pointer.wheel({ horizontalNotches, verticalNotches });
/// ```
pub fn wheel(state: &mut Compositor, x: f64, y: f64, horizontal: f64, vertical: f64) -> Result<()> {
    let horizontal_v120 = notches_to_v120(horizontal);
    let vertical_v120 = notches_to_v120(vertical);
    if horizontal_v120 == 0 && vertical_v120 == 0 {
        bail!("wheel amounts are both smaller than one 120th of a notch");
    }
    point_pointer_at(state, x, y, "wheel")?;
    let pointer = state.seat.get_pointer().context("nested seat has no pointer")?;
    let time = state.start_time.elapsed().as_millis() as u32;
    let mut frame = AxisFrame::new(time).source(AxisSource::Wheel);
    if horizontal_v120 != 0 {
        frame = frame.value(Axis::Horizontal, f64::from(horizontal_v120) / V120_PER_NOTCH * PIXELS_PER_NOTCH);
        frame = frame.v120(Axis::Horizontal, horizontal_v120);
    }
    if vertical_v120 != 0 {
        frame = frame.value(Axis::Vertical, f64::from(vertical_v120) / V120_PER_NOTCH * PIXELS_PER_NOTCH);
        frame = frame.v120(Axis::Vertical, vertical_v120);
    }
    pointer.axis(state, frame);
    pointer.frame(state);
    tracing::debug!(x, y, horizontal, vertical, horizontal_v120, vertical_v120, "injected nested wheel notches");
    return Ok(());
}

/// Inject one touchpad pixel scroll event at logical coordinates.
///
/// What:
///      `pub fn scroll(state: &mut Compositor, x: f64, y: f64, dx: f64, dy: f64) -> Result<()>`.
///  Source finger,
///  continuous logical pixels,
///  no v120 units.
/// Why:
///       A touchpad's two-finger scroll reaches clients as pixel deltas followed by a stop,
///  which clients treat differently from notches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pointer.move(x, y);
/// pointer.touchpadScroll({ dx, dy });
/// ```
pub fn scroll(state: &mut Compositor, x: f64, y: f64, dx: f64, dy: f64) -> Result<()> {
    point_pointer_at(state, x, y, "scroll")?;
    let pointer = state.seat.get_pointer().context("nested seat has no pointer")?;
    let time = state.start_time.elapsed().as_millis() as u32;
    let mut frame = AxisFrame::new(time).source(AxisSource::Finger);
    if dx != 0.0 {
        frame = frame.value(Axis::Horizontal, dx);
    }
    if dy != 0.0 {
        frame = frame.value(Axis::Vertical, dy);
    }
    pointer.axis(state, frame);
    pointer.frame(state);
    tracing::debug!(x, y, dx, dy, "injected a nested touchpad scroll");
    return Ok(())
}

/// Inject the finger-lift stop that ends a touchpad scroll.
///
/// What:
///      `pub fn scroll_stop(state: &mut Compositor) -> Result<()>`.
/// Why:
///       Clients start kinetic scrolling or snap to a position when the fingers leave.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pointer.touchpadScrollStop();
/// ```
pub fn scroll_stop(state: &mut Compositor) -> Result<()> {
    let pointer = state.seat.get_pointer().context("nested seat has no pointer")?;
    let time = state.start_time.elapsed().as_millis() as u32;
    let frame = AxisFrame::new(time).source(AxisSource::Finger).stop(Axis::Horizontal).stop(Axis::Vertical);
    pointer.axis(state, frame);
    pointer.frame(state);
    tracing::debug!("injected a nested touchpad scroll stop");
    return Ok(())
}

/// What:
///      `#[cfg(test)] #[path = "input_wheel_tests.rs"] mod tests;`.
/// Why:
///       Keep the conversion tests beside the conversion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./input_wheel_tests";
/// ```
#[cfg(test)]
#[path = "input_wheel_tests.rs"]
mod tests;
