//! Backend pieces both render backends build the same way:
//!  the nested output and the dmabuf protocol state.
//!
//! The output carries the same mode,
//!  the same `Flipped180` transform,
//!  and the same scale in the nested window mode and in the headless mode,
//!  so screenshots and recordings (which render the output off screen and flip the rows on encode)
//!  are identical in both.
//! The dmabuf global advertises the formats and modifiers of the renderer's own EGL display
//!  and,
//! when the display names a DRM render node,
//! the v4 default feedback for that node.

/// Error context for the v4 feedback build,
/// and the result type of `build_dmabuf`.
use anyhow::{Context, Result};
/// The output and dmabuf types,
/// the EGL device query,
/// and the renderer traits both builders use.
use smithay::{
    backend::{
        egl::EGLDevice,
        renderer::{gles::GlesRenderer, ImportDma},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::wayland_server::DisplayHandle,
    utils::Transform,
    wayland::dmabuf::{DmabufFeedback, DmabufFeedbackBuilder, DmabufGlobal, DmabufState},
};
/// Report the dmabuf version advertised and the fallback to version 3.
use tracing::{info, warn};

/// The nominal refresh rate and the geometry the output is built from.
use crate::{backend::OUTPUT_REFRESH_MHZ, screen_geometry::ScreenGeometry};

/// Build the nested output for `geometry` and register its `wl_output` global.
///
/// What:
///      `pub fn build_output(display_handle: &DisplayHandle, geometry: ScreenGeometry) -> Output`.
///  The mode is the physical size (logical size times scale) at the nominal refresh rate.
/// Why:
///       One construction for both backends keeps clients and captures identical across modes.
pub fn build_output(display_handle: &DisplayHandle, geometry: ScreenGeometry) -> Output {
    let (physical_width, physical_height) = geometry.physical_size();

    // What:     `let mode = Mode { size: (config.width, config.height).into(), refresh:
    //           OUTPUT_REFRESH_MHZ };`. The output's video mode. `(w, h).into()` builds
    //           a `Size<i32, Physical>` from the tuple.
    // Why:      Describe the nested screen's resolution and refresh to clients.
    let mode = Mode {
        size: (physical_width, physical_height).into(),
        refresh: OUTPUT_REFRESH_MHZ,
    };

    // What:     `let output = Output::new("nested".to_string(), PhysicalProperties { ... });`.
    //           Creates the output object. `"nested".to_string()` allocates its owned
    //           name; `PhysicalProperties` describes make/model/physical size (0x0 mm for
    //           a virtual screen). `.into()` builds each owned string / size.
    // Why:      The one screen the fixture presents.
    let output = Output::new(
        "nested".to_string(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "Monochromatic".into(),
            model: "NestedWaylandSession".into(),
        },
    );

    // What:     `let _global = output.create_global::<Compositor>(display_handle);`.
    //           Registers the `wl_output` global. The returned id is bound to `_global`;
    //           dropping it does NOT remove the global (the `Output` keeps it), so
    //           letting it fall out of scope is fine.
    // Why:      Advertise the screen to clients.
    let _global = output.create_global::<crate::state::Compositor>(display_handle);

    // What:     `output.change_current_state(Some(mode), Some(Transform::Flipped180),
    //           Some(geometry.scale.advertised()), Some((0, 0).into()));`. Sets the active
    //           mode, a Y-flip transform (winit's framebuffer origin is top-left, opposite
    //           Wayland's), the starting output scale, and position `(0, 0)`.
    // Why:      Make the mode current, correct the vertical flip so screenshots are upright,
    //           and advertise `--scale` through `wl_output.scale` from the first bind on.
    output.change_current_state(
        Some(mode),
        Some(Transform::Flipped180),
        Some(geometry.scale.advertised()),
        Some((0, 0).into()),
    );

    // What:     `output.set_preferred(mode);`. Marks this mode as the preferred one.
    // Why:      Clients pick the preferred mode when choosing a size.
    output.set_preferred(mode);

    return output
}

/// Create the `zwp_linux_dmabuf_v1` global from what `renderer` can import.
///
/// What:
///      `pub fn build_dmabuf(display_handle: &DisplayHandle, renderer: &mut GlesRenderer) ->
///           Result<(DmabufState, DmabufGlobal, Option<DmabufFeedback>)>`.
///  Version 4 with default feedback when the renderer's EGL display names a render node,
///  version 3 otherwise.
/// Why:
///       The client allocates its buffers from the advertised format and modifier pairs on that node,
///  so the pairs must come from the renderer that imports them.
pub fn build_dmabuf(
    display_handle: &DisplayHandle,
    renderer: &mut GlesRenderer,
) -> Result<(DmabufState, DmabufGlobal, Option<DmabufFeedback>)> {
    // What:     `let render_node = EGLDevice::device_for_display(renderer
    //           .egl_context().display()).and_then(|device| device.try_get_render_node());`.
    //           Walks renderer -> EGL context -> EGL display -> the DRM render node.
    //           `device_for_display` returns `Result<EGLDevice>`; `.and_then(closure)`
    //           runs the next fallible step only on success, yielding
    //           `Result<Option<DrmNode>>`.
    // Why:      dmabuf v4 feedback needs the render node's device id so the client
    //           allocates buffers on the right GPU.
    let render_node = EGLDevice::device_for_display(renderer.egl_context().display())
        .and_then(|device| return device.try_get_render_node());

    // What:     `let mut dmabuf_state = DmabufState::new();`. Fresh dmabuf state; `mut`
    //           because creating the global borrows it mutably.
    // Why:      Owns the dmabuf protocol bookkeeping.
    let mut dmabuf_state = DmabufState::new();

    // What:     `let (dmabuf_global, dmabuf_feedback) = match render_node { Ok(Some(node))
    //           => { ... build v4 ... }, _ => { ... build v3 ... } };`. Pattern-match the
    //           render-node result: a present node builds v4 default-feedback; any other
    //           outcome (query error or no node) falls back to plain v3.
    // Why:      Prefer v4 modifier feedback (what the app negotiates), but never fail if
    //           the render node cannot be determined.
    let (dmabuf_global, dmabuf_feedback) = match render_node {
        Ok(Some(node)) => {
            // What:     `let formats = renderer.dmabuf_formats();`. The set of
            //           dmabuf formats/modifiers the renderer can import (from `ImportDma`).
            // Why:      Feedback advertises exactly these to the client.
            let formats = renderer.dmabuf_formats();

            // What:     `let feedback = DmabufFeedbackBuilder::new(node.dev_id(), formats)
            //           .build().context(...)?;`. Builds the default feedback for that
            //           device id and format set; `.context(msg)?` turns a build error
            //           into a clean failure.
            // Why:      Assemble the v4 feedback object.
            let feedback = DmabufFeedbackBuilder::new(node.dev_id(), formats)
                .build()
                .context("building dmabuf v4 default feedback failed")?;

            // What:     `let global = dmabuf_state.create_global_with_default_feedback::
            //           <Compositor>(display_handle, &feedback);`. Registers the
            //           `zwp_linux_dmabuf_v1` global at version 4 with this feedback.
            // Why:      Advertise dmabuf v4 to the client.
            let global = dmabuf_state
                .create_global_with_default_feedback::<crate::state::Compositor>(
                    display_handle,
                    &feedback,
                );

            info!("dmabuf: advertising v4 with modifier feedback");

            // What:     `(global, Some(feedback))`. Tuple of the global and the kept
            //           feedback (tail of this match arm).
            // Why:      Keep the feedback alive alongside the global.
            (global, Some(feedback))
        }
        _ => {
            // What:     `warn!(...)`. Log that we could not get a render node.
            // Why:      Make the v3 fallback visible in logs.
            warn!("dmabuf: no render node available, falling back to v3");

            // What:     `let formats = renderer.dmabuf_formats();`. Same format
            //           set for the v3 global.
            // Why:      v3 advertises the format list without per-device feedback.
            let formats = renderer.dmabuf_formats();

            // What:     `let global = dmabuf_state.create_global::<Compositor>(
            //           display_handle, formats);`. Registers the dmabuf global at v3.
            // Why:      Still let the client import GPU buffers, just without feedback.
            let global = dmabuf_state
                .create_global::<crate::state::Compositor>(display_handle, formats);

            // What:     `(global, None)`. No feedback in the v3 path.
            // Why:      Signal v3 was used.
            (global, None)
        }
    };

    return Ok((dmabuf_state, dmabuf_global, dmabuf_feedback))
}
