//! Desktop-shell integration:
//!  the Wayland app id,
//!  stamped before the window exists,
//!  and the GPU the window is drawn on.
//!
//! What:
//!  [`install_backend`] opens a hardware Vulkan device
//!       ([`crate::gpu_device`]),
//!  and installs Slint's winit backend with FemtoVG over wgpu 30 on that device
//!       and a window-attributes hook,
//!       [`set_window_app_id`],
//!  that names the window `monochromatic.ide`,
//!  as the sibling terminal
//!       and music player do in their own `launcher.rs`.
//! Why:
//!  Desktop shells match a window to its launcher entry by app id (`StartupWMClass` in
//!      `share/applications/monochromatic.ide.desktop`);
//!  Slint's default backend sets none,
//!  and the id
//!      cannot be changed after the window is created.
//!  The device is opened here and handed to Slint
//!       because nothing in the IDE paints on the CPU:
//!  Slint's own adapter choice may land on a CPU adapter (lavapipe),
//!       and its default limits forbid the storage buffers the GPU text path needs.
//!  The device is opened on the first hardware adapter the Vulkan loader lists
//!       (the system's choice),
//!       unless `WGPU_ADAPTER_NAME` or `WGPU_POWER_PREF` choose another.
//!  The application has no OpenGL renderer and no software renderer;
//!  a host without a hardware Vulkan adapter stops with a message instead of drawing some other way.

/// The device the window is drawn on:
///  a hardware Vulkan adapter,
///  never a CPU one.
use crate::gpu_device::open_hardware_device;
/// What:
///  The trait that adds `with_name` (the Wayland app id) to winit window attributes;
///  `use`
///       brings it into scope like a TS import.
/// Why:
///  Without the trait in scope the method does not exist on the attributes value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { withWaylandName } from 'winit/platform/wayland';
/// ```
use i_slint_backend_winit::winit::platform::wayland::WindowAttributesExtWayland;
/// The creation-time window settings record.
/// Why:
///  The hook receives and returns attributes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { WindowAttributes } from 'winit';
/// ```
use i_slint_backend_winit::winit::window::WindowAttributes;
/// What:
///  `BackendSelector` installs a Slint backend from requirements,
///  and `WGPUConfiguration::Manual` hands it a wgpu instance,
///  adapter,
///  device,
///  and queue that this program opened.
/// Why:
///  The selector is Slint's public way to pass a wgpu device to its winit backend together with the window hook.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BackendSelector, WGPUConfiguration } from 'slint';
/// ```
use slint::{BackendSelector, wgpu_30::WGPUConfiguration};

/// What:
///  The app id.
///  `&str` is text stored in the program (sibling `String`,
///  owned text built at run time).
/// Why:
///  The window's app id and the launcher entry's `StartupWMClass` and file name must match exactly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const APP_ID = 'monochromatic.ide';
/// ```
pub const APP_ID: &str = "monochromatic.ide";

/// What:
///  Slint's name for the renderer of this build:
///  FemtoVG drawing through wgpu.
/// Why:
///  It is the only renderer compiled in,
///  and `SLINT_BACKEND=winit-femtovg-wgpu` spells it.
const RENDERER: &str = "femtovg-wgpu";

/// What:
///  The window-attributes hook:
///  stamp the app id onto the window being created and hand the
///       attributes back (taken and returned by value).
/// Why:
///  Passed to `BackendSelector::with_winit_window_attributes_hook(...)`,
///  which calls it at window creation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function setWindowAppId(attributes: WindowAttributes): WindowAttributes {
///   return attributes.withName(APP_ID, APP_ID);
/// }
/// ```
pub fn set_window_app_id(attributes: WindowAttributes) -> WindowAttributes {
    // `with_name(general, instance)`: on Wayland the first is the app id; X11 uses both as WM_CLASS.
    return attributes.with_name(APP_ID, APP_ID);
}

/// What:
///  Who handles the window backend for a `SLINT_BACKEND` value.
///  An enum is a value that is exactly one of its variants (a TS union of string literals).
/// Why:
///  Only the winit backend takes the device and the app id from this program.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BackendChoice = 'thisProgram' | 'slint';
/// ```
#[derive(Debug, PartialEq, Eq)]
enum BackendChoice {
    /// Winit with this build's renderer,
    ///  on the device opened here.
    ThisProgram,
    /// Another backend (`qt`,
    ///  `linuxkms`,
    ///  ...):
    ///  Slint's own selector decides,
    ///  and the window gets no app id.
    Slint,
}

/// What:
///  Split a lowercase `SLINT_BACKEND` value into backend and renderer names,
///  as Slint's `parse_backend_env_var` does
///       (`i-slint-backend-selector-1.18.1/lib.rs`).
///  The result is two borrowed text slices,
///       a tuple.
/// Why:
///  A value with a dash splits at the first dash (`winit-femtovg-wgpu` is `winit` and `femtovg-wgpu`);
///  a few single words are shorthands for a backend with a renderer
///       (`femtovg` is `winit` with `femtovg`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseSpelling(spelling: string): [backend: string, renderer: string];
/// ```
fn parse_spelling(spelling: &str) -> (&str, &str) {
    if let Some(pair) = spelling.split_once('-') {
        return pair;
    }
    return match spelling {
        "gl" | "winit" => ("winit", ""),
        "femtovg" => ("winit", "femtovg"),
        "skia" => ("winit", "skia"),
        "sw" | "software" => ("winit", "software"),
        "vello" => ("winit", "vello"),
        other => (other, ""),
    };
}

/// What:
///  Decide from the lowercase `SLINT_BACKEND` value who installs the window backend.
///  `Result<T, E>` is a value or an error (a function that may throw).
/// Why:
///  An empty value,
///  `winit`,
///  and `winit-femtovg-wgpu` are this program's.
///  A winit renderer that this build does not contain (`femtovg` is the OpenGL one,
///  `software`,
///  `skia`,
///  `vello`) is refused:
///  Slint would print a warning and draw with its default renderer on a device chosen by Slint,
///  which is not what the person asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function chooseBackend(requested: string): BackendChoice; // throws for a renderer this build lacks
/// ```
fn choose_backend(requested: &str) -> anyhow::Result<BackendChoice> {
    if requested.is_empty() {
        return Ok(BackendChoice::ThisProgram);
    }
    let (backend, renderer) = parse_spelling(requested);
    if backend != "winit" {
        return Ok(BackendChoice::Slint);
    }
    if renderer.is_empty() || renderer == RENDERER {
        return Ok(BackendChoice::ThisProgram);
    }
    return Err(anyhow::anyhow!(
        "SLINT_BACKEND={requested} asks for the renderer '{renderer}', which this build does not contain. \
         The IDE draws only with FemtoVG on wgpu (Vulkan), because nothing in it paints on the CPU \
         and it has no OpenGL renderer. \
         Unset SLINT_BACKEND, or set it to winit or winit-{RENDERER}."
    ));
}

/// What:
///  Open the GPU,
///  then install the winit backend on it with the app-id hook,
///  unless the environment asks Slint for
///       another backend.
///  `anyhow::Result<()>` is success without a value,
///  or an error.
/// Why:
///  The app id must be stamped before the window exists,
///  which only an explicit backend allows,
///  and the GPU must be chosen by this program (see the module comment).
/// Gotcha:
///  A `SLINT_BACKEND` naming a backend other than winit
///         (`qt`,
///  `linuxkms`) is honored:
///  Slint's own selector then decides,
///  and there is no app id and no adapter policy.
///  A winit renderer other than `femtovg-wgpu` is an error.
///  The adapter is chosen by `gpu_device`:
///  the first hardware adapter in the Vulkan loader's order,
///  unless `WGPU_ADAPTER_NAME` or `WGPU_POWER_PREF` (read by `gpu_device`) say otherwise.
///  The info log line says which one decided.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function installBackend(): void // throws when no GPU device or no backend can be created
/// ```
pub fn install_backend() -> anyhow::Result<()> {
    // What: `std::env::var(...)` reads the variable as text; `unwrap_or_default()` gives empty text
    //       when it is unset or not UTF-8; Slint lowercases the value too.
    // Why: Only a value naming winit, optionally with a renderer (`winit-femtovg-wgpu`), stays explicit here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const requested = (env.SLINT_BACKEND ?? '').toLowerCase();
    // ```
    let requested = std::env::var("SLINT_BACKEND")
        .unwrap_or_default()
        .to_lowercase();
    if choose_backend(&requested)? == BackendChoice::Slint {
        tracing::debug!(backend = %requested, "SLINT_BACKEND names another backend; the window gets no app id");
        return Ok(());
    }
    // What: the trailing `?` hands a failure to `main`, which prints it; the message lists the adapters seen.
    // Why: There is no CPU fallback: a host without a hardware Vulkan adapter must stop here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const gpu = openHardwareDevice(); // throws GpuDeviceError
    // ```
    let gpu = open_hardware_device()?;
    let limits = gpu.device.limits();
    tracing::info!(
        adapter = %gpu.summary,
        why = %gpu.why,
        max_storage_buffers_per_shader_stage = limits.max_storage_buffers_per_shader_stage,
        max_texture_dimension_2d = limits.max_texture_dimension_2d,
        "the window is drawn on this GPU"
    );
    // What: `Box::leak` keeps one more handle of each wgpu object alive until the process ends.
    // Why: Slint drops its platform, and with it the last handles, while the main thread's thread-local values are
    //      destroyed at exit. A debug build of wgpu-core reads a thread-local value when it frees a device or queue
    //      (`snatch::trace::LockTrace::enter`), so that drop panics and aborts the process
    //      ("cannot access a Thread Local Storage value during or after destruction"). The native test harness leaks
    //      its device for the same reason (`native/gpu_snapshot.rs`). The operating system frees the memory.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // keepForever(instance, adapter, device, queue); // never released
    // ```
    Box::leak(Box::new((
        gpu.instance.clone(),
        gpu.adapter.clone(),
        gpu.device.clone(),
        gpu.queue.clone(),
    )));
    // What: `Manual` gives Slint the instance, adapter, device, and queue opened above.
    // Why: Slint then neither lists adapters nor requests a device of its own.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const configuration = { instance, adapter, device, queue };
    // ```
    let configuration = WGPUConfiguration::Manual {
        instance: gpu.instance,
        adapter: gpu.adapter,
        device: gpu.device,
        queue: gpu.queue,
    };
    // What: the selector installs the winit backend as the platform of the process, and fails when it cannot.
    //       Both names are set, so it does not read SLINT_BACKEND again.
    // Why: Slint keeps one platform per process; it must be set before the first window.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // slint.select({ backend: 'winit', renderer: 'femtovg-wgpu', wgpu: configuration, windowHook: setWindowAppId });
    // ```
    let selected = BackendSelector::new()
        .backend_name("winit".to_string())
        .renderer_name(RENDERER.to_string())
        .require_wgpu_30(configuration)
        .with_winit_window_attributes_hook(set_window_app_id)
        .select();
    if let Err(error) = selected {
        return Err(anyhow::anyhow!(
            "Cannot install the window backend: {error}"
        ));
    }
    tracing::debug!(
        app_id = APP_ID,
        "winit backend installed with the Wayland app id"
    );
    return Ok(());
}

/// The `SLINT_BACKEND` spellings are checked as text.
#[cfg(test)]
#[path = "launcher_tests.rs"]
mod tests;
