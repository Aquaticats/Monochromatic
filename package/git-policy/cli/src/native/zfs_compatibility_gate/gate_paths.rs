//! What:
//!  The six paths one compatibility check reads,
//! and the live set both executables use.
//! Why:
//!  The paths are the caller's to name,
//! because the binding performs no configuration of
//!  its own and owns no environment;
//! a test names a fixture set instead of the live one,
//!  so no test reaches this machine's kernel files or its device.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const paths = { device: '/dev/zfs', moduleVersion: '/sys/module/zfs/version', ... };
//! ```

use std::path::{Path, PathBuf};

/// The one device file OpenZFS serves requests on,
/// which the binding owns the name of.
pub const DEVICE_PATH: &str = monochromatic_zfs_request_to_openzfs_kernel_device::DEVICE_PATH;

/// The loaded module's release string,
/// which is identity and reporting,
/// never a rule.
pub const MODULE_VERSION_PATH: &str = "/sys/module/zfs/version";
/// The loaded module's source version,
/// which moves when the module is built from other source.
pub const MODULE_SRCVERSION_PATH: &str = "/sys/module/zfs/srcversion";
/// The loaded module's build ID note,
/// which moves when it is recompiled even from identical source.
pub const MODULE_BUILD_ID_PATH: &str = "/sys/module/zfs/notes/.note.gnu.build-id";
/// The kernel's own type information,
/// which a module's file extends.
pub const BASE_BTF_PATH: &str = "/sys/kernel/btf/vmlinux";
/// The loaded module's type information.
pub const MODULE_BTF_PATH: &str = "/sys/kernel/btf/zfs";

/// What:
///  The paths one check reads.
/// Why:
///  A test hands its own six paths in the same shape the live check uses,
///  so
///  the read order and every refusal are exercised without this machine's
///  kernel.
pub struct GatePaths {
    /// The device file a raw request would open.
    pub device: PathBuf,
    /// The module's release string.
    pub module_version: PathBuf,
    /// The module's source version.
    pub module_srcversion: PathBuf,
    /// The module's build ID note.
    pub module_build_id: PathBuf,
    /// The kernel's type information.
    pub base_btf: PathBuf,
    /// The module's type information.
    pub module_btf: PathBuf,
}

impl GatePaths {
    /// The paths the two installed programs read.
    pub fn live() -> Self {
        return GatePaths {
            device: PathBuf::from(DEVICE_PATH),
            module_version: PathBuf::from(MODULE_VERSION_PATH),
            module_srcversion: PathBuf::from(MODULE_SRCVERSION_PATH),
            module_build_id: PathBuf::from(MODULE_BUILD_ID_PATH),
            base_btf: PathBuf::from(BASE_BTF_PATH),
            module_btf: PathBuf::from(MODULE_BTF_PATH),
        };
    }

    /// The device path,
    /// borrowed.
    pub fn device_path(&self) -> &Path {
        return self.device.as_path();
    }

    /// The base type information's path,
    /// borrowed.
    pub fn base_btf_path(&self) -> &Path {
        return self.base_btf.as_path();
    }

    /// The module's type information path,
    /// borrowed.
    pub fn module_btf_path(&self) -> &Path {
        return self.module_btf.as_path();
    }
}
