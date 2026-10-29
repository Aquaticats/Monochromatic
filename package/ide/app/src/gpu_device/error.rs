//! Why no device could be opened,
//!  and the message the person who started the program reads.

/// The adapter descriptions and the variables the messages show.
use super::adapter::{AdapterSummary, NAME_VARIABLE, POWER_VALUES, POWER_VARIABLE, Variables};
/// Error text for the failures below.
use std::fmt;

/// What:
///  What the choice saw,
///  for the error messages:
///  every adapter listed,
///  the order that was used,
///  and the variables that were read.
/// Why:
///  A failure must let the person see whether the machine,
///  the order,
///  or a variable they set is the cause.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SelectionReport = { seen: string[]; order: string[]; variables: Variables };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionReport {
    /// Every Vulkan adapter that was listed,
    ///  in the loader's order,
    ///  each as `name (kind, driver)`;
    ///  empty when the loader found no driver at all.
    pub seen: Vec<String>,
    /// The adapters that were tried,
    ///  in the order used;
    ///  empty when no adapter qualified.
    pub order: Vec<String>,
    /// The variables that were read.
    pub variables: Variables,
}

/// Every adapter name list is written the same way in every message.
fn listing(names: &[String]) -> String {
    if names.is_empty() {
        return "none".to_string();
    }
    return names.join(", ");
}

/// The common tail of the messages that concern the choice.
impl SelectionReport {
    /// What:
    ///  Describe every listed adapter,
    ///  and the adapters at `order` (indexes into the list) in that order.
    /// Why:
    ///  The same description serves every error that concerns the choice.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function newReport(adapters: AdapterSummary[], order: number[], variables: Variables): SelectionReport;
    /// ```
    pub fn new(adapters: &[AdapterSummary], order: &[usize], variables: Variables) -> Self {
        let mut seen: Vec<String> = Vec::new();
        for adapter in adapters {
            seen.push(adapter.to_string());
        }
        let mut used: Vec<String> = Vec::new();
        for index in order {
            used.push(adapters[*index].to_string());
        }
        return Self {
            seen,
            order: used,
            variables,
        };
    }

    /// What:
    ///  The sentence block with the adapters seen,
    ///  the order used,
    ///  the variables read,
    ///  and how to choose a GPU.
    /// Why:
    ///  Each choice error ends with it,
    ///  so every message names every remedy.
    fn details(&self) -> String {
        let seen = if self.seen.is_empty() {
            "none (the Vulkan loader found no driver at all)".to_string()
        } else {
            listing(&self.seen)
        };
        return format!(
            "Adapters seen, in the Vulkan loader's order: {seen}. \
             Order used: {}. \
             Variables read: {}. \
             Without a variable the IDE uses the first hardware adapter the loader lists, \
             which on a Mesa system is the GPU the display server uses. \
             To choose a GPU yourself: set {NAME_VARIABLE} to part of the adapter's name (case-insensitive, such as RADV); \
             set {POWER_VARIABLE} to low (integrated GPU first), high (discrete GPU first), or none (the loader's order); \
             or set DRI_PRIME (for example DRI_PRIME=1) or MESA_VK_DEVICE_SELECT (vendor_id:device_id, \
             as MESA_VK_DEVICE_SELECT=list prints) so that Mesa puts that GPU first in the Vulkan list.",
            listing(&self.order),
            self.variables
        );
    }
}

/// What:
///  Why no device could be opened.
///  An enum is a value that is exactly one of its variants (a TS discriminated union).
/// Why:
///  The caller shows the message to the person who started the program,
///  so each variant says what was seen and every way to fix it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GpuDeviceError =
///   | { kind: 'noRuntime'; reason: string }
///   | { kind: 'noHardwareAdapter'; report: SelectionReport }
///   | { kind: 'adapterNameUnmatched'; report: SelectionReport }
///   | { kind: 'adapterNameMatchesOnlyRefusedKinds'; report: SelectionReport; matched: string[] }
///   | { kind: 'badPowerPreference'; value: string }
///   | { kind: 'deviceRefused'; refusals: string[]; report: SelectionReport };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GpuDeviceError {
    /// The small runtime that waits for wgpu's two futures could not be built.
    NoRuntime(
        /// The operating system's reason.
        String,
    ),
    /// Vulkan lists no discrete or integrated GPU.
    NoHardwareAdapter {
        /// What was listed and read.
        report: SelectionReport,
    },
    /// `WGPU_ADAPTER_NAME` matches the name of no adapter at all.
    AdapterNameUnmatched {
        /// What was listed and read.
        report: SelectionReport,
    },
    /// `WGPU_ADAPTER_NAME` matches only adapters of kinds that are refused (CPU,
    ///  virtual,
    ///  unknown).
    AdapterNameMatchesOnlyRefusedKinds {
        /// What was listed and read.
        report: SelectionReport,
        /// The matching adapters,
        ///  each as `name (kind, driver)`.
        matched: Vec<String>,
    },
    /// `WGPU_POWER_PREF` holds something other than `low`,
    ///  `high`,
    ///  or `none`.
    BadPowerPreference {
        /// The value as read.
        value: String,
    },
    /// Hardware adapters exist and every one of them refused the device.
    DeviceRefused {
        /// One entry per adapter:
        ///  its name and the reason it gave.
        refusals: Vec<String>,
        /// What was listed and read,
        ///  with the order that was tried.
        report: SelectionReport,
    },
}

/// The sentence shown to the person who started the program.
impl fmt::Display for GpuDeviceError {
    /// Writes the problem,
    ///  what was seen,
    ///  and every remedy.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            Self::NoRuntime(reason) => write!(
                formatter,
                "The IDE could not start the small runtime that opens the GPU device: {reason}. \
                 Close other programs to free threads and memory, then start the IDE again."
            ),
            Self::NoHardwareAdapter { report } => write!(
                formatter,
                "The IDE draws every pixel on the GPU through Vulkan, and no hardware Vulkan adapter is available. \
                 {} \
                 An adapter of the CPU kind (lavapipe, llvmpipe) is refused on purpose, even when it is the only one, \
                 and SLINT_WGPU_CPU does not allow it here. \
                 Remedies: install the Vulkan driver of the GPU (the Mesa Vulkan package such as mesa-vulkan-drivers \
                 for AMD and Intel, the vendor driver for NVIDIA) together with the Vulkan loader \
                 (vulkan-loader or libvulkan1); \
                 give the process access to the GPU's render node (/dev/dri/renderD128 and its siblings: \
                 join the render group, or in a container pass --device /dev/dri and install the driver in the image); \
                 run `vulkaninfo --summary` to see what the loader finds; \
                 and if VK_DRIVER_FILES or VK_ICD_FILENAMES is set, make it name the driver file of a hardware GPU.",
                report.details()
            ),
            Self::AdapterNameUnmatched { report } => write!(
                formatter,
                "{NAME_VARIABLE} is set, and its value matches the name of no Vulkan adapter. \
                 {} \
                 Remedies: unset {NAME_VARIABLE}, or set it to part of one of the names above \
                 (the match ignores case); `vulkaninfo --summary` shows the names the loader reports.",
                report.details()
            ),
            Self::AdapterNameMatchesOnlyRefusedKinds { report, matched } => write!(
                formatter,
                "{NAME_VARIABLE} is set, and it matches only adapters that the IDE refuses on purpose: {}. \
                 The IDE never draws on the CPU, so an adapter of the CPU, virtual, or unknown kind is not used \
                 even when the name asks for it, and SLINT_WGPU_CPU does not allow it here. \
                 {} \
                 Remedies: unset {NAME_VARIABLE}, or set it to part of the name of a discrete or integrated GPU.",
                listing(matched),
                report.details()
            ),
            Self::BadPowerPreference { value } => write!(
                formatter,
                "{POWER_VARIABLE} is set to \"{value}\", which wgpu does not accept. \
                 The accepted values are {POWER_VALUES} (case-insensitive): low puts the integrated GPU first, \
                 high puts the discrete GPU first, and none keeps the order of the Vulkan loader. \
                 Remedies: unset {POWER_VARIABLE} or set it to one of those values; \
                 to choose a GPU by name set {NAME_VARIABLE}, \
                 or set DRI_PRIME or MESA_VK_DEVICE_SELECT so that Mesa puts the GPU you want first in the Vulkan list."
            ),
            Self::DeviceRefused { refusals, report } => write!(
                formatter,
                "A hardware Vulkan adapter was found, but none of them gave the IDE a device with wgpu's default limits \
                 (storage buffers and 8192-pixel textures among them, which the GPU text path needs): {}. \
                 {} \
                 Remedies: update the Vulkan driver (Mesa or the vendor driver) of the GPU, \
                 or run the IDE on a host whose GPU supports the Vulkan 1.1 level that wgpu requires; \
                 `vulkaninfo --summary` shows the driver version and the API version each adapter reports.",
                refusals.join("; "),
                report.details()
            ),
        };
    }
}

/// Lets `?` carry this error into `anyhow::Error` and lets callers use it as a standard error.
impl std::error::Error for GpuDeviceError {}
