//! Opening the device a headless session renders on,
//!  and listing the devices a host offers.
//!
//! The order of attempts and the failure text are decided in `gpu_choice.rs`;
//!  this file performs them:
//!  Mesa's surfaceless platform for the default device,
//!  `EGL_EXT_platform_device` for a chosen render node,
//!  and the `EGL_MESA_device_software` device for software rendering.
//! Measured on a host with two AMD GPUs and Mesa 26 (see the README):
//!  every one of the three opens an EGL display that renders offscreen and imports client dmabufs.

/// Error context for device setup,
/// and the result type of `open_gpu`.
use anyhow::{bail, Result};
/// Device identity,
/// the render node type,
/// and the EGL objects behind a renderer.
use smithay::backend::{
    drm::{DrmNode, NodeType},
    egl::{native::EGLSurfacelessDisplay, EGLContext, EGLDevice, EGLDisplay},
    renderer::gles::{ffi, GlesRenderer},
};
/// Read sysfs for the PCI id and the kernel driver,
/// and the environment for the diagnostic hint.
use std::{
    ffi::CStr,
    os::raw::c_char,
    path::{Path, PathBuf},
};
/// Record every attempt and its outcome.
use tracing::{info, warn};

/// The pure decisions,
/// and the report and child environment they produce.
use crate::gpu_choice::{
    child_environment, default_device_rejection, failure_message, plan, select, Attempt, ChildGpuEnvironment,
    GpuChoice, GpuReport,
};

/// Error text that is built only on failure.
use crate::lazy_error_text::failed_text;

/// The sysfs reads of a render node's PCI id and kernel driver.
mod sysfs_identity;
/// The one function the device listing needs from them.
use sysfs_identity::pci_and_driver;

/// A renderer on the chosen device,
/// with what the session reports about it.
///
/// What:
///      `pub struct OpenedGpu { renderer, report, child_environment }`.
/// Why:
///       `backend_headless` builds the output and dmabuf state from the renderer,
///  and keeps the rest for the `gpu` control command and the hosted child's environment.
pub struct OpenedGpu {
    /// The GLES renderer on the chosen device.
    pub renderer: GlesRenderer,
    /// Which device it is,
    /// for the log and the `gpu` answer.
    pub report: GpuReport,
    /// Variables the hosted child needs to render on the same device.
    pub child_environment: ChildGpuEnvironment,
}

/// A renderer and the EGL device behind it,
/// before the report is built.
struct Opened {
    /// The GLES renderer.
    renderer: GlesRenderer,
    /// The EGL device of its display.
    device: EGLDevice,
}

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
        return "unknown".to_string()
    }
    return unsafe { CStr::from_ptr(pointer) }.to_string_lossy().into_owned()
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
    }
}

/// Create a GLES renderer on an EGL display.
fn renderer_on(display: &EGLDisplay) -> Result<GlesRenderer, String> {
    let context = failed_text(EGLContext::new(display), "creating the EGL context failed")?;
    // SAFETY: the context is fresh, not current on another thread, and this thread is its only user.
    return failed_text(unsafe { GlesRenderer::new(context) }, "creating the GLES renderer failed")
}

/// Every EGL device,
/// or the reason EGL cannot list them.
fn enumerate_devices() -> Result<Vec<EGLDevice>, String> {
    return failed_text(
        EGLDevice::enumerate().map(Iterator::collect),
        "EGL device enumeration is unavailable",
    )
}

/// The render node path of a hardware device.
fn render_path(device: &EGLDevice) -> Option<PathBuf> {
    if device.is_software() {
        return None
    }
    return match device.try_get_render_node() {
        Ok(Some(node)) => node.dev_path_with_type(NodeType::Render),
        Ok(None) | Err(_) => device.render_device_path().ok(),
    }
}

/// The render nodes of the hardware devices EGL lists.
fn hardware_nodes() -> Vec<PathBuf> {
    return match enumerate_devices() {
        Ok(devices) => devices.iter().filter_map(render_path).collect(),
        Err(reason) => {
            info!(%reason, "no render node list for the auto fallback");
            Vec::new()
        }
    }
}

/// A hint about the environment variables that change which devices Mesa lists.
fn environment_hint() -> String {
    let forced = matches!(std::env::var_os("LIBGL_ALWAYS_SOFTWARE"), Some(value) if !value.is_empty());
    if forced {
        return " The environment sets LIBGL_ALWAYS_SOFTWARE, which makes Mesa list only the software device; \
                 unset it to use a GPU."
            .to_string()
    }
    return String::new()
}

/// Open the surfaceless platform's device whatever it is.
fn open_mesa_default_any() -> Result<Opened, String> {
    // SAFETY: the surfaceless native display is a unit value that outlives the display.
    let display = failed_text(
        unsafe { EGLDisplay::new(EGLSurfacelessDisplay) },
        "opening the surfaceless EGL display failed",
    )?;
    let device = failed_text(EGLDevice::device_for_display(&display), "querying the EGL device failed")?;
    let renderer = renderer_on(&display)?;
    return Ok(Opened { renderer, device })
}

/// Open the device Mesa's surfaceless platform picks,
/// unless that is the CPU rasterizer.
fn open_mesa_default() -> Result<Opened, String> {
    let opened = open_mesa_default_any()?;
    if let Some(reason) = default_device_rejection(opened.device.is_software()) {
        return Err(reason)
    }
    return Ok(opened)
}

/// Open one EGL device through `EGL_EXT_platform_device`.
fn open_device(device: EGLDevice) -> Result<Opened, String> {
    // SAFETY: the device value outlives the display it creates.
    let display = failed_text(
        unsafe { EGLDisplay::new(device.clone()) },
        "opening an EGL display on the device failed",
    )?;
    let renderer = renderer_on(&display)?;
    return Ok(Opened { renderer, device })
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
fn device_renders_on(device: &EGLDevice, node: &DrmNode) -> bool {
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
        Err(error) => return Err(format!("{} is not a DRM device node ({error})", path.display())),
    };
    if node.ty() != NodeType::Render {
        let hint = match node.dev_path_with_type(NodeType::Render) {
            Some(render) => format!("; use its render node {}", render.display()),
            None => String::new(),
        };
        return Err(format!("{} is a {} node, not a render node{hint}", path.display(), node.ty()))
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
        ))
    };
    return open_device(device)
}

/// Open the CPU rasterizer device.
fn open_software() -> Result<Opened, String> {
    match enumerate_devices() {
        Ok(devices) => {
            let Some(device) = devices.into_iter().find(EGLDevice::is_software) else {
                return Err(
                    "EGL lists no software device (EGL_MESA_device_software); Mesa's llvmpipe is not installed or is disabled"
                        .to_string(),
                )
            };
            return open_device(device)
        }
        Err(reason) => {
            // Without enumeration the surfaceless platform is the only door;
            // it counts only when it answers with the CPU rasterizer.
            info!(%reason, "software device: trying the surfaceless platform instead");
            let opened = open_mesa_default_any()?;
            if !opened.device.is_software() {
                return Err(format!("{reason}; Mesa's default device is a GPU, not the software renderer"))
            }
            return Ok(opened)
        }
    }
}

/// Perform one attempt and log its outcome.
///
/// What:
///      `fn open_and_log_attempt(attempt: &Attempt) -> Result<Opened, String>`.
/// Why:
///       `select` takes it by path instead of a closure that captured nothing.
fn open_and_log_attempt(attempt: &Attempt) -> Result<Opened, String> {
    let outcome = open_attempt(attempt);
    match &outcome {
        Ok(_) => info!(%attempt, "headless: device attempt succeeded"),
        Err(reason) => info!(%attempt, %reason, "headless: device attempt failed"),
    }
    return outcome
}

/// Perform one attempt.
fn open_attempt(attempt: &Attempt) -> Result<Opened, String> {
    return match attempt {
        Attempt::MesaDefault => open_mesa_default(),
        Attempt::RenderNode(path) => open_render_node(path),
        Attempt::Software => open_software(),
    }
}

/// The device text of a report.
fn device_text(device: &EGLDevice) -> String {
    if device.is_software() {
        return "software".to_string()
    }
    match render_path(device) {
        Some(path) => return path.to_string_lossy().into_owned(),
        None => return "unknown".to_string(),
    }
}

/// Open the renderer for a `--gpu` choice.
///
/// What:
///      `pub fn open_gpu(choice: &GpuChoice) -> Result<OpenedGpu>`.
///  Plans the attempts,
///  opens them in order until one works,
///  logs each failure,
///  and returns the renderer with its report.
///  `auto` falls back across devices and then to software;
///  an explicit choice never does.
/// Why:
///       One place decides which device renders,
///  so the log,
///  the `gpu` answer,
///  and the hosted child's environment all describe the same device.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gpu = openGpu({ kind: 'auto' }); // { renderer, report, childEnvironment }
/// ```
pub fn open_gpu(choice: &GpuChoice) -> Result<OpenedGpu> {
    let attempts = plan(choice, &hardware_nodes());
    info!(choice = %choice.label(), attempts = ?attempts, "headless: choosing the render device");
    let search = select(&attempts, open_and_log_attempt);
    let selected = match search {
        Ok(selected) => selected,
        Err(failures) => bail!("{}", failure_message(choice, &failures, &devices_text())),
    };

    let fell_back = !selected.failures.is_empty();
    let mut opened = selected.value;
    let device = device_text(&opened.device);
    let software = opened.device.is_software();
    if fell_back {
        let mut tried: Vec<String> = Vec::new();
        for failure in &selected.failures {
            tried.push(format!("{}: {}", failure.attempt, failure.reason));
        }
        info!(
            using = %device,
            tried = ?tried,
            "headless: auto fell back because earlier devices could not be used",
        );
    }
    if software {
        warn!("headless: the EGL device is a software rasterizer, not a GPU; clients render on the CPU");
    }
    if selected.attempt == Attempt::MesaDefault {
        info!(
            render_node = ?opened.device.render_device_path().ok(),
            drm_node = ?opened.device.drm_device_path().ok(),
            "headless: surfaceless EGL display opened on this device",
        );
    } else {
        info!(
            render_node = ?opened.device.render_device_path().ok(),
            drm_node = ?opened.device.drm_device_path().ok(),
            "headless: EGL display opened on the chosen device",
        );
    }
    let report = GpuReport {
        choice: choice.label(),
        device,
        renderer: renderer_name(&mut opened.renderer),
        fell_back,
    };
    info!(
        device = %report.device,
        choice = %report.choice,
        renderer = %report.renderer,
        fell_back = report.fell_back,
        "headless: rendering on this device",
    );
    return Ok(OpenedGpu {
        renderer: opened.renderer,
        child_environment: child_environment(choice, software, fell_back),
        report,
    })
}

/// The key `--list-gpus` sorts devices by:
/// hardware first,
/// then render node path.
///
/// What:
///      `fn listing_order_key(device: &EGLDevice) -> (bool, Option<PathBuf>)`.
/// Why:
///       `sort_by_key` takes it by path instead of a closure.
fn listing_order_key(device: &EGLDevice) -> (bool, Option<PathBuf>) {
    return (device.is_software(), render_path(device))
}

/// One line describing a device for `--list-gpus`.
fn list_line(device: EGLDevice) -> String {
    let software = device.is_software();
    let path = render_path(&device);
    let node = device.try_get_render_node().ok().flatten();
    let (pci, driver) = node.as_ref().map_or((None, None), pci_and_driver);
    let label = if software {
        "software".to_string()
    } else {
        match path.as_ref() {
            Some(value) => value.to_string_lossy().into_owned(),
            None => "unknown-device".to_string(),
        }
    };
    let mut line = label;
    if let Some(value) = pci {
        line.push_str(&format!(" pci={value}"));
    }
    if let Some(value) = driver {
        line.push_str(&format!(" driver={value}"));
    }
    return match open_device(device) {
        Ok(mut opened) => format!("{line} renderer={}", renderer_name(&mut opened.renderer)),
        Err(reason) => format!("{line} unusable: {reason}"),
    }
}

/// The devices this host offers,
/// one per line,
/// for the failure message and `--list-gpus`.
///
/// What:
///      `fn devices_text() -> String`.
///  Each line starts with the value to pass to `--gpu`.
/// Why:
///       A person who passed a wrong device sees what to pass instead.
fn devices_text() -> String {
    let mut lines = Vec::new();
    match enumerate_devices() {
        Ok(devices) => {
            let mut sorted = devices;
            sorted.sort_by_key(listing_order_key);
            for device in sorted {
                lines.push(list_line(device));
            }
        }
        Err(reason) => lines.push(format!("devices: {reason}")),
    }
    return lines.join("\n")
}

/// The text `--list-gpus` prints.
///
/// What:
///      `pub fn list_gpus() -> String`.
/// Why:
///       A person chooses `--gpu` from what this host actually offers:
///  render nodes with their PCI id,
///  kernel driver and Mesa renderer,
///  and `software` when Mesa's CPU rasterizer is present.
pub fn list_gpus() -> String {
    let mut text = devices_text();
    match open_gpu(&GpuChoice::Auto) {
        Ok(opened) => text.push_str(&format!("\nauto uses: {} ({})", opened.report.device, opened.report.renderer)),
        Err(error) => {
            info!(%error, "auto found no device while listing");
            text.push_str("\nauto uses: nothing, see the lines above");
        }
    }
    return text
}
