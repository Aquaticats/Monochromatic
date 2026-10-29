//! The one GPU device of the process:
//!  a hardware Vulkan adapter with wgpu's full default limits,
//!  chosen by the application and never by Slint.
//!
//! The application and the native test harness (`native/gpu_snapshot.rs`) open their device here,
//!  so both refuse the same adapters.
//!  Nothing in the IDE paints on the CPU (owner decision of 2026-10-19),
//!  and a CPU adapter such as lavapipe (`llvmpipe`) would be exactly that,
//!  so it is never chosen and never a fallback.
//!  Slint cannot give that guarantee:
//!  with no power preference wgpu does not sort the adapters it lists,
//!  and Slint's own check only asks that some adapter is not of the CPU kind.
//!
//! Which hardware adapter:
//!  The candidates are the discrete and integrated GPUs,
//!  in the order the Vulkan loader lists them.
//!  That order is the system's choice:
//!  Mesa documents that without `DRI_PRIME` the default GPU is the one the display server uses,
//!  that `DRI_PRIME` and `MESA_VK_DEVICE_SELECT` put the selected GPU first in the physical device list,
//!  and desktops set these per application through switcheroo-control.
//!  The IDE is a read-only editor,
//!  so it does not rank discrete above integrated and does not wake a second GPU on its own.
//!  Two variables that wgpu documents are honored as explicit choices of the person who started the program:
//!  `WGPU_ADAPTER_NAME` keeps only the hardware adapters whose name contains the value (case-insensitive),
//!  and `WGPU_POWER_PREF` (`low`,
//!  `high`,
//!  or `none`,
//!  case-insensitive) moves the integrated or the discrete GPUs first.
//!  When both are set the name filters and the power preference orders what is left,
//!  as wgpu's own helper lets the name win over the power preference.
//!  An empty value counts as unset.
//!  The reason for the choice is returned with the device so the launcher can log it.

/// What an adapter reports about itself and the two variables that choose one.
mod adapter;
/// Which adapters may be used and in which order.
mod choice;
/// Why no device could be opened,
///  and the message for it.
mod error;

/// Reads a `WGPU_POWER_PREF` value before any driver is touched.
use adapter::parse_power_order;
/// The adapter descriptions and the variables of the choice.
pub use adapter::{AdapterSummary, Variables};
/// The choice.
pub use choice::choose_adapters;
/// The failures and the report they carry.
pub use error::{GpuDeviceError, SelectionReport};

/// The `wgpu` crate as Slint re-exports it,
///  so the device is built with exactly the wgpu release (30) that Slint's renderer was built against.
use slint::wgpu_30::wgpu;

/// What:
///  Turn the operating system's failure to build the small runtime into this module's error.
/// Why:
///  `map_err` takes it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function noRuntime(error: Error): GpuDeviceError;
/// ```
fn no_runtime(error: std::io::Error) -> GpuDeviceError {
    return GpuDeviceError::NoRuntime(error.to_string());
}

/// What:
///  The opened device:
///  the wgpu instance,
///  the chosen adapter,
///  and the device and queue made from it.
/// Why:
///  Slint takes all four to render on a device the application opened
///  (`WGPUConfiguration::Manual`),
///  and the harness hands the instance,
///  device,
///  and queue to its renderer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HardwareDevice = { instance: GPUInstance; adapter: GPUAdapter; device: GPUDevice; queue: GPUQueue; summary: AdapterSummary; why: string };
/// ```
pub struct HardwareDevice {
    /// The instance that lists only Vulkan adapters.
    pub instance: wgpu::Instance,
    /// The chosen hardware adapter.
    pub adapter: wgpu::Adapter,
    /// The device,
    ///  with wgpu's default limits raised to what the adapter allows for image sizes.
    pub device: wgpu::Device,
    /// The queue of that device.
    pub queue: wgpu::Queue,
    /// What the adapter reported about itself.
    pub summary: AdapterSummary,
    /// Why this adapter was used:
    ///  the loader's order,
    ///  or the variable that decided.
    pub why: String,
}

/// What:
///  The description of the device the application asks every adapter for:
///  `limits` as its limits,
///  and device memory handed out in blocks of 8 to 64 MiB (`MemoryHints::MemoryUsage`).
/// Why:
///  wgpu's default hint (`Performance`) reserves device memory in blocks of 128 to 256 MiB.
///  A device with no renderer already holds 193.6 MiB under it and 13.6 MiB under `MemoryUsage`,
///  and Vello's first render adds a 256 MiB block under it,
///  so a process measured 458.2 MiB of video memory with the default hint and 234.4 MiB with this one
///  (`doc/planning/ide-vello-buffer-sizes.md`).
///  The card is shared with the desktop,
///  and every native test process opens a device of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deviceDescription(limits: Limits): DeviceDescriptor;
/// ```
fn device_description(limits: wgpu::Limits) -> wgpu::DeviceDescriptor<'static> {
    return wgpu::DeviceDescriptor {
        label: Some("ide gpu device"),
        required_limits: limits,
        memory_hints: wgpu::MemoryHints::MemoryUsage,
        ..wgpu::DeviceDescriptor::default()
    };
}

/// What:
///  `open_hardware_device` opens a Vulkan device on the first hardware adapter,
///  in the order of `choose_adapters`,
///  that accepts wgpu's default limits.
///  It reads `WGPU_ADAPTER_NAME` and `WGPU_POWER_PREF` from the process environment.
///  `Result<T, E>` is a value or an error (a function that may throw).
///  `Vec` is a growable array.
/// Why:
///  The device must have full limits (`wgpu::Limits::default()` raised to the adapter's resolution limits,
///  as Slint does for its own device),
///  because Vello's compute shaders need storage buffers,
///  which Slint's downlevel default limits forbid.
///  Only the Vulkan backend is asked for,
///  because Slint keeps the GL backend away from wgpu too.
///  A machine with no hardware adapter fails with the list of the adapters it has,
///  and never falls back to a CPU one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openHardwareDevice(): HardwareDevice; // throws GpuDeviceError
/// ```
pub fn open_hardware_device() -> Result<HardwareDevice, GpuDeviceError> {
    return open_hardware_device_capped(None);
}

/// What:
///  `open_hardware_device` with the largest 2D texture the device offers lowered to `texture_cap` when that is smaller
///  than the adapter's own limit.
///  The device's real limit is recorded as the process's tile texture limit
///  (`ide_app::shaped_text::set_texture_dimension`).
/// Why:
///  A test can ask for a device that offers only wgpu's default 8192 pixels,
/// as many GPUs do,
///  on a host whose GPU offers 16384.
///  The record is made here,
/// the one place a device is opened,
/// so every window of the process cuts its tiles for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openHardwareDeviceCapped(textureCap?: number): HardwareDevice; // throws GpuDeviceError
/// ```
pub fn open_hardware_device_capped(
    texture_cap: Option<u32>,
) -> Result<HardwareDevice, GpuDeviceError> {
    // What: the two variables, read once, before any GPU work.
    // Why: a misspelled `WGPU_POWER_PREF` should fail without touching a driver.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const variables = { adapterName: env.WGPU_ADAPTER_NAME, powerPreference: env.WGPU_POWER_PREF };
    // ```
    let variables = Variables::from_environment();
    if let Some(value) = &variables.power_preference
        && parse_power_order(value).is_none()
    {
        return Err(GpuDeviceError::BadPowerPreference {
            value: value.clone(),
        });
    }
    // What: a current-thread tokio runtime runs the two wgpu futures to completion; `map_err` turns its error into ours.
    // Why: wgpu's adapter and device requests are async; tokio is already a dependency of the application.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const runtime = new Runtime(); // throws GpuDeviceError on failure
    // ```
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .map_err(no_runtime)?;
    // What: an instance that may open only the Vulkan backend (not GL), with default flags and options.
    // Why: wgpu documents the display handle as unused on Vulkan, and Slint's own automatic path passes none either.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const instance = new GPUInstance({ backends: ['vulkan'] });
    // ```
    let mut description = wgpu::InstanceDescriptor::new_without_display_handle();
    description.backends = wgpu::Backends::VULKAN;
    let instance = wgpu::Instance::new(description);
    // What: every Vulkan adapter, as an owned list in the loader's order, and what each one says about itself.
    // Why: the choice needs all of them, and the failure message names them.
    //      wgpu 30 does not sort this list when no power preference is given to it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const adapters = await instance.enumerateAdapters('vulkan');
    // ```
    let adapters = runtime.block_on(instance.enumerate_adapters(wgpu::Backends::VULKAN));
    let mut summaries: Vec<AdapterSummary> = Vec::new();
    for adapter in &adapters {
        let info = adapter.get_info();
        summaries.push(AdapterSummary {
            name: info.name,
            device_type: info.device_type,
            driver: info.driver,
        });
    }
    let choice = choose_adapters(&summaries, &variables)?;
    // What: try the adapters in the chosen order; the first that gives a device wins.
    // Why: an integrated GPU with the limits is better than a discrete one without them, and the refusals are reported.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const index of order) { try { return await open(adapters[index]); } catch (error) { refusals.push(...); } }
    // ```
    let mut refusals: Vec<String> = Vec::new();
    for (position, index) in choice.indexes.iter().copied().enumerate() {
        let adapter = adapters[index].clone();
        let summary = summaries[index].clone();
        let mut limits = wgpu::Limits::default().using_resolution(adapter.limits());
        if let Some(cap) = texture_cap {
            limits.max_texture_dimension_2d = limits.max_texture_dimension_2d.min(cap);
        }
        let device_description = device_description(limits);
        match runtime.block_on(adapter.request_device(&device_description)) {
            Ok((device, queue)) => {
                ide_app::shaped_text::set_texture_dimension(
                    device.limits().max_texture_dimension_2d,
                );
                return Ok(HardwareDevice {
                    instance,
                    adapter,
                    device,
                    queue,
                    summary,
                    why: choice.why(position, index),
                });
            }
            Err(error) => {
                tracing::warn!(adapter = %summary, %error, "a hardware adapter refused the device");
                refusals.push(format!("{}: {error}", summary.name));
            }
        }
    }
    let report = SelectionReport::new(&summaries, &choice.indexes, variables);
    return Err(GpuDeviceError::DeviceRefused { refusals, report });
}

/// The adapter choice and the messages of the errors are checked from adapter descriptions alone.
#[cfg(test)]
#[path = "gpu_device_tests.rs"]
mod tests;
