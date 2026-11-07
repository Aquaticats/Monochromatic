//! The PCI id and kernel driver name of a DRM render node,
//!  read from sysfs.
//!
//! Split out of `gpu_device.rs` to keep that file under the code-line limit.

/// The DRM node type whose major and minor numbers name the sysfs directory.
use smithay::backend::drm::DrmNode;
/// Read sysfs files and name the paths they live at.
use std::path::{Path, PathBuf};

/// The sysfs directory of a DRM node's PCI device.
fn sysfs_device(node: &DrmNode) -> PathBuf {
    return PathBuf::from(format!("/sys/dev/char/{}:{}/device", node.major(), node.minor()))
}

/// Read one sysfs attribute without its `0x` prefix.
fn sysfs_hex(directory: &Path, name: &str) -> Option<String> {
    let text = std::fs::read_to_string(directory.join(name)).ok()?;
    return Some(text.trim().trim_start_matches("0x").to_string())
}

/// The kernel driver name a sysfs `driver` link points to.
///
/// What:
///      `fn driver_name_of(target: PathBuf) -> Option<String>`.
///  The last path component,
///  or nothing when the link target has none.
/// Why:
///       `Option::and_then` takes it by path instead of a closure.
fn driver_name_of(target: PathBuf) -> Option<String> {
    return Some(target.file_name()?.to_string_lossy().into_owned())
}

/// The PCI id (`vendor:device`) and kernel driver name of a render node,
/// when sysfs names them.
pub(super) fn pci_and_driver(node: &DrmNode) -> (Option<String>, Option<String>) {
    let directory = sysfs_device(node);
    let pci = match (sysfs_hex(&directory, "vendor"), sysfs_hex(&directory, "device")) {
        (Some(vendor), Some(device)) => Some(format!("{vendor}:{device}")),
        _ => None,
    };
    let driver = std::fs::read_link(directory.join("driver"))
        .ok()
        .and_then(driver_name_of);
    return (pci, driver)
}
