//! The render devices EGL lists on this host,
//!  the paths that name them,
//!  and the hint about the environment variables that hide some of them.
//!
//! Split out of `gpu_device.rs` to keep that file under the code-line limit.

/// The DRM node type a render node is named by.
use smithay::backend::drm::NodeType;
/// One device EGL lists,
///  whether it is the CPU rasterizer or a GPU.
use smithay::backend::egl::EGLDevice;
/// The path that names a render node.
use std::path::PathBuf;
/// Record why a device list is missing.
use tracing::info;

/// Error text that is built only on failure.
use crate::lazy_error_text::failed_text;
/// Every EGL device,
/// or the reason EGL cannot list them.
pub(super) fn enumerate_devices() -> Result<Vec<EGLDevice>, String> {
    return failed_text(
        EGLDevice::enumerate().map(Iterator::collect),
        "EGL device enumeration is unavailable",
    );
}

/// The render node path of a hardware device.
pub(super) fn render_path(device: &EGLDevice) -> Option<PathBuf> {
    if device.is_software() {
        return None;
    }
    return match device.try_get_render_node() {
        Ok(Some(node)) => node.dev_path_with_type(NodeType::Render),
        Ok(None) | Err(_) => device.render_device_path().ok(),
    };
}

/// The render nodes of the hardware devices EGL lists.
pub(super) fn hardware_nodes() -> Vec<PathBuf> {
    return match enumerate_devices() {
        Ok(devices) => devices.iter().filter_map(render_path).collect(),
        Err(reason) => {
            info!(%reason, "no render node list for the auto fallback");
            Vec::new()
        }
    };
}

/// A hint about the environment variables that change which devices Mesa lists.
pub(super) fn environment_hint() -> String {
    let forced =
        matches!(std::env::var_os("LIBGL_ALWAYS_SOFTWARE"), Some(value) if !value.is_empty());
    if forced {
        return " The environment sets LIBGL_ALWAYS_SOFTWARE, which makes Mesa list only the software device; \
                 unset it to use a GPU."
            .to_string();
    }
    return String::new();
}
