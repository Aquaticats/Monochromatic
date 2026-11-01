//! Winit backend initialisation:
//!  the nested window,
//!  its GLES renderer,
//!  the output,
//! and the dmabuf protocol state.
//!
//! This runs while the process's `WAYLAND_DISPLAY` still points at the PARENT
//! compositor (the host session),
//!  so the winit window becomes a client of the host.
//! It mirrors anvil's proven winit path:
//!  build the renderer,
//!  query the render node so
//! we can advertise dmabuf v4 modifier feedback (falling back to v3),
//!  and bind the
//! EGL display for Mesa's legacy hardware-acceleration path.

/// What:
///      Grouped `use` of the renderer traits,
///  EGL device query,
///  winit backend,
///           output types,
///  transform,
///  dmabuf types,
///  and the display handle.
/// Why:
///       Everything `init_backend` references.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { GlesRenderer, EGLDevice, winit, Output, ... } from "smithay";
/// ```
use smithay::{
    backend::{
        renderer::{gles::GlesRenderer, ImportEgl},
        winit::{self, WinitEventLoop},
    },
    reexports::{
        wayland_server::DisplayHandle,
        winit::{dpi::PhysicalSize, window::WindowAttributes},
    },
};

/// What:
///      `use anyhow::Result;`.
///  Error helpers.
/// Why:
///       `init_backend` returns `Result` and attaches context to winit failures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // throw/rethrow-with-message helpers
/// ```
use anyhow::Result;

/// What:
///      `use tracing::info;`.
///  Structured log macros.
/// Why:
///       Report the chosen dmabuf version and hardware-acceleration status.
use tracing::info;

/// What:
///      `use crate::{cli::Config, state::BackendPieces};`.
///  Our config input and the
///           carrier struct for the built pieces.
/// Why:
///       `init_backend` reads the requested size from `Config` and returns
///           `BackendPieces`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Config } from "./cli";
/// import { BackendPieces } from "./state";
/// ```
use crate::{
    backend_parts::{build_dmabuf, build_output},
    backend_select::{nested_failure_message, ParentEnvironment},
    cli::Config,
    render_backend::RenderBackend,
    screen_geometry::ScreenGeometry,
    state::BackendPieces,
};

/// Milli-hertz refresh rate reported for the nested output (60.000 Hz).
///
/// What:
///      `const OUTPUT_REFRESH_MHZ: i32 = 60_000;`.
///  Signed `i32`;
///  Smithay reports
///           refresh in millihertz,
///  so 60 Hz is 60000.
/// Why:
///       Named so the magic number is not repeated at the mode-construction site.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const OUTPUT_REFRESH_MHZ = 60000;
/// ```
pub const OUTPUT_REFRESH_MHZ: i32 = 60_000;

/// Build the winit backend,
///  the nested output,
///  and the dmabuf state.
///
/// What:
///      `pub fn init_backend(display_handle: &DisplayHandle, config: &Config) ->
///           Result<(BackendPieces, WinitEventLoop)>`.
///  Borrows the display handle and
///           config,
///  returns the built pieces plus the winit event source to register.
/// Why:
///       Isolate all the winit/EGL/dmabuf construction from the state wiring.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function initBackend(displayHandle, config): [BackendPieces, WinitEventLoop] { ... }
/// ```
///
/// @example
/// ```ts
/// const [pieces, winit] = initBackend(displayHandle, config);
/// ```
pub fn init_backend(
    display_handle: &DisplayHandle,
    config: &Config,
) -> Result<(BackendPieces, WinitEventLoop)> {
    // What:     `ScreenGeometry { ... }` records the logical size and scale from the command
    //           line; `physical_size()` multiplies them into the framebuffer size.
    // Why:      The window is the framebuffer, so it starts at logical size times scale. The
    //           parent's own scale is unknown until the window exists, so the first request is
    //           in physical pixels; `screen::parent_resized` corrects any rounding afterwards.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const geometry = { logicalWidth: config.width, logicalHeight: config.height, scale: config.scale };
    // const [physicalWidth, physicalHeight] = geometry.physicalSize();
    // ```
    let geometry = ScreenGeometry {
        logical_width: config.width,
        logical_height: config.height,
        scale: config.scale,
    };
    let (physical_width, physical_height) = geometry.physical_size();

    // What:     `let attributes = WindowAttributes::default().with_title(...)
    //           .with_inner_size(PhysicalSize::new(w, h));`. Builds the winit window
    //           request. `physical_width as u32` casts the signed dimension to the
    //           unsigned type `PhysicalSize::new` wants.
    // Why:      The nested window's inner size is the framebuffer the app is drawn into.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const attributes = { title: "...", innerSize: { width: physicalWidth, height: physicalHeight } };
    // ```
    let attributes = WindowAttributes::default()
        .with_title("nested-wayland-session")
        .with_inner_size(PhysicalSize::new(physical_width as u32, physical_height as u32));

    // What:     `let (mut backend, winit) = winit::init_from_attributes::<GlesRenderer>(
    //           attributes).map_err(...)?;`. Creates the backend + event loop. The
    //           `::<GlesRenderer>` picks the renderer type. `.map_err(|e| anyhow!(...))`
    //           converts Smithay's error into an `anyhow::Error`; the trailing `?`
    //           returns it on failure. `mut backend` because we call `renderer()`
    //           (a `&mut` method) below.
    // Why:      This is the nested window and its GPU renderer.
    let (mut backend, winit) = winit::init_from_attributes::<GlesRenderer>(attributes)
        .map_err(|err| {
            return anyhow::anyhow!(nested_failure_message(&err.to_string(), ParentEnvironment::from_process()))
        })?;

    // What:     `backend.window().set_minimized(true);`. Asks the parent compositor to minimize
    //           the new window; winit sends the request for `xdg_toplevel.set_minimized` on
    //           Wayland and ignores un-minimize there. The parent may ignore the request.
    // Why:      The nested session is a fixture, not something the user watches; it launches
    //           minimized so it never covers the desktop. Capture does not depend on the parent
    //           presenting the window: the frame pacing timer and the recorder's own timer
    //           (see `frame_pacing.rs` and the README's Frame pacing section) keep the hosted app drawing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // window.setMinimized(true);
    // ```
    backend.window().set_minimized(true);

    // What:     `build_output(display_handle, geometry)`. Creates the output, its mode, its
    //           `wl_output` global, the `Flipped180` transform, and the starting scale.
    // Why:      Shared with the headless backend so both modes present the same screen.
    let output = build_output(display_handle, geometry);

    // What:     `build_dmabuf(display_handle, backend.renderer())?`. Queries the render node of the
    //           renderer's EGL display and creates the dmabuf v4 (or v3) global.
    // Why:      dmabuf v4 feedback needs the render node's device id so the client allocates
    //           buffers on the right GPU.
    let (dmabuf_state, dmabuf_global, dmabuf_feedback) = build_dmabuf(display_handle, backend.renderer())?;

    // What:     `if backend.renderer().bind_wl_display(display_handle).is_ok() { ... }`.
    //           `bind_wl_display` (from `ImportEgl`, enabled by `use_system_lib`) sets up
    //           the legacy EGL `wl_drm` path; `.is_ok()` is `true` when it succeeds.
    // Why:      Mesa accepts either dmabuf v4 OR this wl_drm path for hardware
    //           acceleration; binding it is belt-and-braces so the GPU import always has a
    //           route.
    if backend.renderer().bind_wl_display(display_handle).is_ok() {
        info!("EGL hardware-acceleration (wl_drm) enabled");
    }

    // What:     `Ok((BackendPieces { ... }, winit))`. Bundle the built pieces and the
    //           winit event source; tail expression.
    // Why:      Hand everything back to `run` for state construction and source
    //           registration.
    return Ok((
        BackendPieces {
            backend: RenderBackend::Winit(Box::new(backend)),
            output,
            dmabuf_state,
            dmabuf_global,
            dmabuf_feedback,
            geometry,
        },
        winit,
    ))
}
