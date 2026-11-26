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
//!
//! The devices EGL lists live in `the_devices_egl_lists.rs`,
//!  the attempts that open one in `opening_a_render_device.rs`,
//!  and the line each device gets in `describing_the_devices.rs`.

/// Error context for device setup,
/// and the result type of `open_gpu`.
use anyhow::{
    Result,
    bail,
};

/// The device behind a renderer,
///  and the renderer itself.
use smithay::backend::{
    egl::EGLDevice,
    renderer::gles::GlesRenderer,
};

/// Record every attempt and its outcome.
use tracing::{
    info,
    warn,
};

/// The pure decisions,
/// and the report and child environment they produce.
use crate::gpu_choice::{
    Attempt,
    ChildGpuEnvironment,
    GpuChoice,
    GpuReport,
    child_environment,
    failure_message,
    plan,
    select,
};

/// The sysfs reads of a render node's PCI id and kernel driver.
mod sysfs_identity;

/// The render devices EGL lists on this host.
mod the_devices_egl_lists;

/// Opening one render device through EGL.
mod opening_a_render_device;

/// One line per device for `--list-gpus` and the failure text.
mod describing_the_devices;

/// The device list,
///  and the path that names a device.
use the_devices_egl_lists::{
    hardware_nodes,
    render_path,
};

/// One attempt of the plan,
///  opened and logged.
use opening_a_render_device::open_and_log_attempt;

/// The GL renderer string of a context,
///  part of this module's interface.
pub use opening_a_render_device::renderer_name;

/// One line per device,
///  for `--list-gpus` and the failure text.
use describing_the_devices::devices_text;

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

/// The device text of a report.
fn device_text(device: &EGLDevice) -> String {
    if device.is_software() {
        return "software".to_string();
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
        warn!(
            "headless: the EGL device is a software rasterizer, not a GPU; clients render on the CPU"
        );
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
    });
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
        Ok(opened) => text.push_str(&format!(
            "\nauto uses: {} ({})",
            opened.report.device, opened.report.renderer
        )),
        Err(error) => {
            info!(%error, "auto found no device while listing");
            text.push_str("\nauto uses: nothing, see the lines above");
        }
    }
    return text;
}
