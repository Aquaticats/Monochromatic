//! Opening one render device through EGL:
//!  Mesa's surfaceless platform for the default device,
//!  `EGL_EXT_platform_device` for a render node a person named,
//!  and the `EGL_MESA_device_software` device for software rendering.
//!
//! Split out of `gpu_device.rs` to keep that file under the code-line limit.

/// A renderer or the reason an attempt failed.
use anyhow::Result;
/// The DRM node a person names,
///  its type,
///  and the EGL objects behind a renderer.
use smithay::backend::{
    drm::{
        DrmNode,
        NodeType,
    },
    egl::{
        EGLContext,
        EGLDevice,
        EGLDisplay,
        native::EGLSurfacelessDisplay,
    },
    renderer::gles::{
        GlesRenderer,
        ffi,
    },
};
/// Read the `GL_RENDERER` string the driver returns.
use std::{
    ffi::CStr,
    os::raw::c_char,
    path::Path,
};
/// Record every attempt and its outcome.
use tracing::{
    info,
    warn,
};

/// One attempt of the plan.
use crate::gpu_choice::{
    Attempt,
    default_device_rejection,
};
/// Error text that is built only on failure.
use crate::lazy_error_text::failed_text;

/// The renderer and the EGL device behind it,
///  before the report is built.
use super::Opened;
/// The device list every attempt enumerates.
use super::the_devices_egl_lists::{
    enumerate_devices,
    environment_hint,
};
/// The `GL_RENDERER` string of the current GL context.
///
/// What:
///      `fn read_renderer_string(gl: &ffi::Gles2) -> String`.
///  `"unknown"` when the driver returns no string.
/// Why:
///       `GlesRenderer::with_context` takes it by path instead of a closure that captured nothing.
fn read_renderer_string(gl: &ffi::Gles2) -> String {
    // SAFETY: GL_RENDERER is a valid string name in a current context;
    // the returned pointer is a NUL-terminated string owned by the driver and copied here.
    let pointer = unsafe { gl.GetString(ffi::RENDERER) } as *const c_char;
    if pointer.is_null() {
        return "unknown".to_string();
    }
    return unsafe { CStr::from_ptr(pointer) }
        .to_string_lossy()
        .into_owned();
}

/// The GL renderer string of a context,
/// for example `AMD Radeon RX 7600 (radeonsi, navi33, ...)` or `llvmpipe (LLVM 22.1.8, 256 bits)`.
///
/// What:
///      `pub fn renderer_name(renderer: &mut GlesRenderer) -> String`.
///  Reads `GL_RENDERER` in the renderer's context.
/// Why:
///       The renderer string is what tells a GPU from the CPU rasterizer in a report.
pub fn renderer_name(renderer: &mut GlesRenderer) -> String {
    let name = renderer.with_context(read_renderer_string);
    return match name {
        Ok(text) => text,
        Err(error) => {
            warn!(%error, "could not read GL_RENDERER");
            "unknown".to_string()
        }
    };
}

/// Create a GLES renderer on an EGL display.
fn renderer_on(display: &EGLDisplay) -> Result<GlesRenderer, String> {
    let context = failed_text(EGLContext::new(display), "creating the EGL context failed")?;
    // SAFETY: the context is fresh, not current on another thread, and this thread is its only user.
    return failed_text(
        unsafe { GlesRenderer::new(context) },
        "creating the GLES renderer failed",
    );
}

/// Open the surfaceless platform's device whatever it is.
fn open_mesa_default_any() -> Result<Opened, String> {
    // SAFETY: the surfaceless native display is a unit value that outlives the display.
    let display = failed_text(
        unsafe { EGLDisplay::new(EGLSurfacelessDisplay) },
        "opening the surfaceless EGL display failed",
    )?;
    let device = failed_text(
        EGLDevice::device_for_display(&display),
        "querying the EGL device failed",
    )?;
    let renderer = renderer_on(&display)?;
    return Ok(Opened {
        renderer,
        device,
    });
}

/// Open the device Mesa's surfaceless platform picks,
/// unless that is the CPU rasterizer.
fn open_mesa_default() -> Result<Opened, String> {
    let opened = open_mesa_default_any()?;
    if let Some(reason) = default_device_rejection(opened.device.is_software()) {
        return Err(reason);
    }
    return Ok(opened);
}

/// Open one EGL device through `EGL_EXT_platform_device`.
pub(super) fn open_device(device: EGLDevice) -> Result<Opened, String> {
    // SAFETY: the device value outlives the display it creates.
    let display = failed_text(
        unsafe { EGLDisplay::new(device.clone()) },
        "opening an EGL display on the device failed",
    )?;
    let renderer = renderer_on(&display)?;
    return Ok(Opened {
        renderer,
        device,
    });
}

/// Whether an EGL device's render node is the given DRM node.
///
/// What:
///      `fn device_renders_on(device: &EGLDevice, node: &DrmNode) -> bool`.
///  Compares device numbers,
///  so a symlink to the node matches.
/// Why:
///       Enumeration order is not node order,
///  so the device for a named node is found by its number.
fn device_renders_on(
    device: &EGLDevice,
    node: &DrmNode,
) -> bool {
    match device.try_get_render_node() {
        Ok(Some(candidate)) => return candidate.dev_id() == node.dev_id(),
        Ok(None) | Err(_) => return false,
    }
}

/// Open the render node a person named.
///
/// What:
///      `fn open_render_node(path: &Path) -> Result<Opened, String>`.
///  Matches the path to an EGL device by the node's device number,
///  so a symlink such as `/dev/dri/by-path/...-render` works,
///  and a primary node such as `/dev/dri/card0` is refused with the render node to use instead.
/// Why:
///       Enumeration order is not node order,
///  and a card node cannot be opened as a render device.
fn open_render_node(path: &Path) -> Result<Opened, String> {
    let node = match DrmNode::from_path(path) {
        Ok(node) => node,
        Err(error) => {
            return Err(format!(
                "{} is not a DRM device node ({error})",
                path.display()
            ));
        }
    };
    if node.ty() != NodeType::Render {
        let hint = match node.dev_path_with_type(NodeType::Render) {
            Some(render) => format!("; use its render node {}", render.display()),
            None => String::new(),
        };
        return Err(format!(
            "{} is a {} node, not a render node{hint}",
            path.display(),
            node.ty()
        ));
    }
    let devices = enumerate_devices()?;
    let mut found = None;
    for device in devices {
        if device_renders_on(&device, &node) {
            found = Some(device);
            break;
        }
    }
    let Some(device) = found else {
        return Err(format!(
            "EGL lists no device for {}, so Mesa cannot render on it.{}",
            path.display(),
            environment_hint(),
        ));
    };
    return open_device(device);
}

/// Open the CPU rasterizer device.
fn open_software() -> Result<Opened, String> {
    match enumerate_devices() {
        Ok(devices) => {
            let Some(device) = devices.into_iter().find(EGLDevice::is_software) else {
                return Err(
                    "EGL lists no software device (EGL_MESA_device_software); Mesa's llvmpipe is not installed or is disabled"
                        .to_string(),
                );
            };
            return open_device(device);
        }
        Err(reason) => {
            // Without enumeration the surfaceless platform is the only door;
            // it counts only when it answers with the CPU rasterizer.
            info!(%reason, "software device: trying the surfaceless platform instead");
            let opened = open_mesa_default_any()?;
            if !opened.device.is_software() {
                return Err(format!(
                    "{reason}; Mesa's default device is a GPU, not the software renderer"
                ));
            }
            return Ok(opened);
        }
    }
}

/// Perform one attempt and log its outcome.
///
/// What:
///      `fn open_and_log_attempt(attempt: &Attempt) -> Result<Opened, String>`.
/// Why:
///       `select` takes it by path instead of a closure that captured nothing.
pub(super) fn open_and_log_attempt(attempt: &Attempt) -> Result<Opened, String> {
    let outcome = open_attempt(attempt);
    match &outcome {
        Ok(_) => info!(%attempt, "headless: device attempt succeeded"),
        Err(reason) => info!(%attempt, %reason, "headless: device attempt failed"),
    }
    return outcome;
}

/// Perform one attempt.
fn open_attempt(attempt: &Attempt) -> Result<Opened, String> {
    return match attempt {
        Attempt::MesaDefault => open_mesa_default(),
        Attempt::RenderNode(path) => open_render_node(path),
        Attempt::Software => open_software(),
    };
}
