//! Session settings the control socket reads and changes:
//!  the key repeat announcement.

/// Smithay compositor types.
use smithay::input::pointer::CursorImageStatus;

/// Sibling modules of this crate.
use crate::{gpu_choice::GpuReport, protocol::Response, state::Compositor};
/// Structured logging.
use tracing::info;

/// Answer or change `repeat-info`.
///
/// What:
///      `pub fn repeat_info(state: &mut Compositor, change: Option<(i32, i32)>) -> Response`.
///  `None` answers `rate=R delay=D`.
///  A pair sends `wl_keyboard.repeat_info` to every client and answers `changed` or `unchanged`.
/// Why:
///       Wayland clients generate key repeat themselves from the announced rate and delay,
///  so a test sets them to make a hold yield a known number of repeats.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repeatInfo(state: Compositor, change?: [number, number]): Response { ... }
/// ```
pub fn repeat_info(state: &mut Compositor, change: Option<(i32, i32)>) -> Response {
    let Some((rate, delay)) = change else {
        return Response::OkWith(format!("rate={} delay={}", state.repeat_info.0, state.repeat_info.1))
    };
    if state.repeat_info == (rate, delay) {
        return Response::OkWith("unchanged".to_string())
    }
    let Some(keyboard) = state.seat.get_keyboard() else {
        return Response::Err("nested seat has no keyboard".to_string())
    };
    keyboard.change_repeat_info(rate, delay);
    state.repeat_info = (rate, delay);
    info!(rate, delay, "announced a new key repeat rate and delay");
    return Response::OkWith("changed".to_string())
}

/// Answer `gpu`.
///
/// What:
///      `pub fn gpu(state: &mut Compositor) -> Response`.
///  `device=D choice=C fallback=yes|no renderer=R`,
///  with the renderer string last because it contains spaces.
///  The nested window answers `device=parent choice=parent`.
/// Why:
///       A caller that asked for `--gpu auto` sees which device it got and whether the session fell back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gpu = (state: Compositor) => ok(state.gpuReport.answer());
/// ```
pub fn gpu(state: &mut Compositor) -> Response {
    if let Some(report) = state.backend.gpu_report() {
        return Response::OkWith(report.answer())
    }
    let renderer = crate::gpu_device::renderer_name(state.backend.renderer());
    return Response::OkWith(
        GpuReport {
            choice: "parent".to_string(),
            device: "parent".to_string(),
            renderer,
            fell_back: false,
        }
        .answer(),
    )
}

/// Name a cursor image for the `cursor` answer.
///
/// What:
///      `pub fn describe_cursor(image: &CursorImageStatus) -> String`.
///  The cursor-shape name (`default`,
///  `pointer`,
///  `text`,
///  `grab`),
///  `hidden`,
///  or `surface` for a bitmap the client drew itself.
/// Why:
///       The names are the CSS-derived ones of `wp_cursor_shape_v1`,
///  which toolkits send for their cursor settings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function describeCursor(image: CursorImageStatus): string { ... }
/// ```
#[must_use]
pub fn describe_cursor(image: &CursorImageStatus) -> String {
    match image {
        CursorImageStatus::Hidden => return "hidden".to_string(),
        CursorImageStatus::Named(icon) => return icon.name().to_string(),
        CursorImageStatus::Surface(_) => return "surface".to_string(),
    }
}

/// Answer `cursor`.
///
/// What:
///      `pub fn cursor(state: &Compositor) -> Response`.
/// Why:
///       Reports the cursor the hovered client chose,
///  after a `move` onto its control.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cursor = (state: Compositor) => ok(describeCursor(state.cursorImage));
/// ```
#[must_use]
pub fn cursor(state: &Compositor) -> Response {
    return Response::OkWith(describe_cursor(&state.cursor_image))
}

/// What:
///      `#[cfg(test)] #[path = "control_session_tests.rs"] mod tests;`.
/// Why:
///       Keep the cursor-name tests beside the describer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./control_session_tests";
/// ```
#[cfg(test)]
#[path = "control_session_tests.rs"]
mod tests;
