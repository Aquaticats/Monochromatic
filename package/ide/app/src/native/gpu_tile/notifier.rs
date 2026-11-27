//! The window's one rendering notifier:
//!  it sets up Vello on the first frame,
//!  draws the waiting scenes before each frame,
//!  releases them at teardown,
//!  and writes the debug frame records.

/// The window the textures are shown in,
///  and the chaining of the debug frame records.
use super::super::{
    AppWindow,
    frame_trace,
};
/// The handle the window state holds.
use super::GpuTile;
/// The window handle,
///  the rendering stages,
///  and the graphics API the device arrives as.
use slint::{
    ComponentHandle,
    GraphicsAPI,
    RenderingState,
};
/// Shared ownership of the two cells the notifier writes.
use std::rc::Rc;

/// What:
///  Install the one rendering notifier of `window`:
///  it sets up Vello on the first frame,
///  draws the waiting scenes before each frame,
///  releases them at teardown,
///  and writes the debug frame records.
///  `gpu` is the handle the window state holds.
/// Why:
///  A window takes one notifier,
///  so the frame records and the drawing share it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bind(window: AppWindow, gpu: GpuTile): void;
/// ```
pub(in crate::native) fn bind(
    window: &AppWindow,
    gpu: &GpuTile,
) {
    let inner = Rc::clone(&gpu.inner);
    let shown = Rc::clone(&gpu.shown);
    let weak = window.as_weak();
    let installed = window.window().set_rendering_notifier(move |state, api| {
        frame_trace::log_rendering_stage(&state);
        let GraphicsAPI::WGPU30 {
            device,
            queue,
            ..
        } = api
        else {
            tracing::error!("the window's renderer offers no wgpu 30 device");
            return;
        };
        let outcome = match state {
            RenderingState::RenderingSetup => inner.borrow_mut().setup(device, queue),
            RenderingState::BeforeRendering => inner.borrow_mut().draw_pending(),
            RenderingState::RenderingTeardown => {
                inner.borrow_mut().teardown();
                return;
            }
            _ => return,
        };
        if let Some(active) = weak.upgrade() {
            // What: New textures for the tile reach the window's list of slices inside the frame that made them,
            //       and a reset or replaced list is not drawn by that frame (`repeater_frame_tests`); the window
            //       asks for the frame that does.
            // Why: Nothing else asks for it. The window draws no frame by itself, and before the hint skip rule the
            //      answer to a hint request repainted the source a moment later and so drew one by chance.
            //      A press whose request is skipped has no answer, and its code column stayed blank.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const replaced = outcome.tile !== undefined; apply(outcome, window); if (replaced) window.requestRedraw();
            // ```
            let replaced = outcome.tile.is_some();
            outcome.apply(&active);
            if matches!(state, RenderingState::BeforeRendering) {
                // The offset the toolkit's scroll has reached against the offset the text is drawn at: the two differ
                // while the displayed tile does not hold the view.
                shown.set(Some(active.get_drawn_offset()));
                tracing::debug!(
                    target: "monochromatic_ide::native::frame",
                    real = -active.get_scroll_y(),
                    drawn = active.get_drawn_offset(),
                    held = active.get_held(),
                    "the frame's view"
                );
            }
            if replaced {
                tracing::debug!(
                    "replaced the slices of the source tile inside a frame, so asking for the next"
                );
                active.window().request_redraw();
            }
        }
    });
    if let Err(error) = installed {
        tracing::error!(%error, "the source view cannot be drawn: the renderer takes no notifier");
        window.set_error_message(slint::SharedString::from(format!(
            "Cannot draw source: the window's renderer takes no rendering notifier ({error})"
        )));
    }
}
