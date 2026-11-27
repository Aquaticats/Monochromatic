//! One line per device for `--list-gpus` and for the failure message:
//!  its render node,
//!  PCI id,
//!  kernel driver,
//!  and Mesa renderer,
//!  or why it cannot be used.
//!
//! Split out of `gpu_device.rs` to keep that file under the code-line limit.

/// One device to describe.
use smithay::backend::egl::EGLDevice;
/// The path that names a render node.
use std::path::PathBuf;

/// Opening a device to ask its renderer,
///  and the renderer's own name.
use super::opening_a_render_device::{
    open_device,
    renderer_name,
};
/// The PCI id and kernel driver of a render node,
///  read from sysfs.
use super::sysfs_identity::pci_and_driver;
/// The device list to describe,
///  and the path that names a device.
use super::the_devices_egl_lists::{
    enumerate_devices,
    render_path,
};
/// The key `--list-gpus` sorts devices by:
/// hardware first,
/// then render node path.
///
/// What:
///      `fn listing_order_key(device: &EGLDevice) -> (bool, Option<PathBuf>)`.
/// Why:
///       `sort_by_key` takes it by path instead of a closure.
fn listing_order_key(device: &EGLDevice) -> (bool, Option<PathBuf>) {
    return (device.is_software(), render_path(device));
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
    };
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
pub(super) fn devices_text() -> String {
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
    return lines.join("\n");
}
