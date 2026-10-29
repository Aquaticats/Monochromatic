//! Headless backend initialisation:
//!  an EGL display on the chosen device,
//!  its GLES renderer,
//!  the output,
//!  and the dmabuf protocol state.
//!
//! Nothing here connects to a parent display,
//!  so the session runs on a machine with no Wayland or X11 session (CI,
//! SSH)
//!  and never opens a window on a desktop.
//! The nested window path inherits the parent compositor's EGL device;
//!  this path has no parent to inherit from,
//!  so `--gpu` decides (see `gpu_device.rs`):
//!  Mesa's default render node,
//!  a named render node,
//!  or the software rasterizer.
//! The dmabuf global advertises exactly the format and modifier pairs that display can import,
//!  and names its render node as the default feedback main device when it has one.
//!  A client therefore allocates its dmabufs on the GPU that imports them.

/// Error context for the EGL setup steps,
/// and the result type of `init_headless_backend`.
use anyhow::Result;
/// The renderer trait that binds the legacy `wl_drm` path,
/// and the display handle.
use smithay::{backend::renderer::ImportEgl, reexports::wayland_server::DisplayHandle};
/// Log the legacy acceleration path.
use tracing::info;

/// The shared output and dmabuf builders,
/// the config,
/// the headless backend types,
/// and the state carrier.
use crate::{
    backend_parts::{build_dmabuf, build_output},
    cli::Config,
    gpu_device::open_gpu,
    render_backend::{HeadlessBackend, RenderBackend},
    screen_geometry::ScreenGeometry,
    state::BackendPieces,
};

/// Build the headless renderer,
///  the output,
///  and the dmabuf state.
///
/// What:
///      `pub fn init_headless_backend(display_handle: &DisplayHandle, config: &Config) ->
///           Result<BackendPieces>`.
///  Opens the device `config.gpu` names,
///  and builds the same output and dmabuf global the nested path builds.
/// Why:
///       The only difference from the nested path is where the pixels go:
///  nowhere until a screenshot or recording renders the output off screen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function initHeadlessBackend(displayHandle, config): BackendPieces { ... }
/// ```
pub fn init_headless_backend(display_handle: &DisplayHandle, config: &Config) -> Result<BackendPieces> {
    let geometry = ScreenGeometry {
        logical_width: config.width,
        logical_height: config.height,
        scale: config.scale,
    };
    let (physical_width, physical_height) = geometry.physical_size();

    // What:     `open_gpu(&config.gpu)` opens the EGL display and GLES renderer on the device `--gpu` names:
    //           Mesa's default render node (`auto`, falling back to other nodes and then software),
    //           one render node, or the software rasterizer. An explicit choice never falls back.
    // Why:      Headless has no parent compositor to inherit a device from. The report and the child
    //           environment it returns keep the control answer and the hosted client on the same device.
    let opened = open_gpu(&config.gpu)?;
    let mut renderer = opened.renderer;

    let output = build_output(display_handle, geometry);
    let (dmabuf_state, dmabuf_global, dmabuf_feedback) = build_dmabuf(display_handle, &mut renderer)?;

    // What:     The legacy `wl_drm` acceleration path; the surfaceless display may not offer it.
    // Why:      dmabuf v4 is the route in headless mode, so a failure here is only logged.
    match renderer.bind_wl_display(display_handle) {
        Ok(()) => info!("headless: EGL hardware-acceleration (wl_drm) enabled"),
        Err(error) => info!(%error, "headless: wl_drm acceleration is not offered; clients use dmabuf"),
    }

    return Ok(BackendPieces {
        backend: RenderBackend::Headless(Box::new(HeadlessBackend::new(
            renderer,
            (physical_width, physical_height).into(),
            opened.report,
            opened.child_environment,
        ))),
        output,
        dmabuf_state,
        dmabuf_global,
        dmabuf_feedback,
        geometry,
    })
}
