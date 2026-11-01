# Slint 1.18.1 `WGPUConfiguration::Automatic` takes the first adapter wgpu lists, CPU adapters included, with limits that give Vello no storage buffers

Tool under test:
Slint 1.18.1 (`slint`,
`i-slint-core`,
`i-slint-backend-selector`,
`i-slint-backend-winit`) with the
`renderer-femtovg-wgpu` and `unstable-wgpu-30` features,
on wgpu 30.0.1 (`wgpu-core` 30.0.1,
Vulkan backend),
Vello 0.11.0,
Mesa on Fedora 44 with three Vulkan adapters (RADV on an AMD RX 7600,
RADV on the Ryzen 9 7950X
integrated GPU,
and llvmpipe).
The studies that found it are the sections "Why the launcher opens the device" and "The CPU adapter question" of
`doc/planning/ide-gpu-snapshot-harness.md`.

## Symptom

Three separate facts,
all visible when an application hands Slint `WGPUConfiguration::Automatic` and lets Slint pick
the GPU:

- **The adapter is whichever one wgpu lists first.**
  With no power preference wgpu does not sort,
  so the first adapter of the Vulkan loader's list wins,
  and that can be
  the CPU rasterizer llvmpipe while hardware GPUs are present.
  Slint's own start-up check does not stop this:
  it only asks that some adapter is not of the CPU kind.
  On this host,
  with Mesa told to list llvmpipe first,
  a Slint window started on it:

  ```text
  slint chose llvmpipe (LLVM 22.1.8, 256 bits) (Cpu, driver llvmpipe); max_storage_buffers_per_shader_stage=0
  ```

- **The device has no storage buffers.**
  `WGPUSettings::default()` requests `Limits::downlevel_webgl2_defaults()`,
  whose
  `max_storage_buffers_per_shader_stage` is 0 (the line above ends with it).
  Slint's own renderers draw with that,
  but Vello's compute shaders bind storage buffers,
  so Vello 0.11 on that
  device panics in its shader initialisation threads:

  ```text
  thread 'Vello shader initialisation worker thread' panicked at .../wgpu-30.0.1/src/backend/wgpu_core.rs:1148:26:
  wgpu error: Validation Error

  Caused by:
    In Device::create_bind_group_layout
      Too many bindings of type StorageBuffers in Stage ShaderStages(COMPUTE), limit is 0, count was 1. Check the limit `max_storage_buffers_per_shader_stage` passed to `Adapter::request_device`
  ```

  A Slint run with no wgpu configuration at all (`None`) gets `adapter.limits()` instead,
  so the two Slint paths
  disagree.
- **Which hardware GPU wins depends on Mesa and not on Slint.**
  Mesa's Vulkan device-select layer puts the display server's GPU first unless `DRI_PRIME` or `MESA_VK_DEVICE_SELECT`
  say otherwise (see the Mesa note).
  The same Slint binary on the same host drew on the RX 7600 in the real session and on the integrated GPU inside the
  headless nested compositor,
  because that compositor renders on the integrated GPU.

## Root cause

### wgpu returns the first adapter unless a power preference sorts

```rust
// wgpu-core-30.0.1/src/instance.rs:511-513 and 575-578
            if desc.force_fallback_adapter {
                log::debug!("Filtering `{backend:?}` for `force_fallback_adapter`");
                backend_adapters.retain(|exposed| {
...
        match desc.power_preference {
            PowerPreference::LowPower => {
                sort(&mut adapters, true);
            }
            PowerPreference::HighPerformance => {
                sort(&mut adapters, false);
            }
            PowerPreference::None => {}
        };
```

The CPU filter exists only for `force_fallback_adapter: true`,
and that filter keeps only the CPU adapters.
The sort puts `DeviceType::Cpu` last (`get_order`,
`instance.rs:595-606`),
but `PowerPreference::None` does not sort,
and it is the default (`wgpu-types-30.0.1/src/adapter.rs:78-81`,
documented as
"Power usage is not considered when choosing an adapter").
This is the documented behavior of the option,
not a defect of wgpu.

### Slint passes `None` and `force_fallback_adapter: false`, and checks something weaker

```rust
// i-slint-core-1.18.1/graphics/wgpu_30.rs:346-358
            let adapter = match wgpu::util::initialize_adapter_from_env(&instance, Some(&surface))
                .await
            {
                Ok(adapter) => Ok(adapter),
                Err(_) => {
                    instance
                        .request_adapter(&wgpu::RequestAdapterOptions {
                            power_preference: wgpu30_settings.power_preference,
                            force_fallback_adapter: false,
```

`power_preference` comes from `WGPUSettings::default()`,
which reads `WGPU_POWER_PREF` and otherwise uses `None`
(`wgpu_30.rs:62`).
`initialize_adapter_from_env` (`wgpu-30.0.1/src/util/init.rs:9-44`) returns the first adapter,
in list order,
whose
lower-cased name contains `WGPU_ADAPTER_NAME`,
and returns an error when that variable is unset;
that is how
`WGPU_ADAPTER_NAME` wins over everything else.

The start-up check that `BackendSelector::select` runs for `require_wgpu_30` asks only for existence:

```rust
// i-slint-core-1.18.1/graphics/wgpu_30.rs:173-175, 194-196 and 237-240
/// Internal helper to see if there are any GPU adapters for hardware accelerated rendering.
/// This is used to determine if we should fall back to software rendering (instead of using WGPU
/// software rendering,
such as DX12's Warp adapter).
...
    let allow_cpu = std::env::var("SLINT_WGPU_CPU").is_ok();
    if allow_cpu {
        return true;
    }
...
    poll_once(instance.enumerate_adapters(backends))
        .unwrap()
        .into_iter()
        .any(|adapter| adapter.get_info().device_type != wgpu::DeviceType::Cpu)
```

```rust
// i-slint-backend-selector-1.18.1/api.rs:310-322
        #[cfg(feature = "unstable-wgpu-30")]
        if matches!(self.requested_graphics_api, Some(RequestedGraphicsAPI::WGPU30(_)))
            && !i_slint_core::graphics::wgpu_30::any_wgpu30_adapters_with_gpu(
...
            return Err(
                "WGPU 30.x rendering was required but no GPU-backed WGPU adapter is available \
                 for the requested backends. Set SLINT_WGPU_CPU=1 to allow CPU adapters."
```

So the check refuses a host whose only adapters are CPU ones,
unless `SLINT_WGPU_CPU` is set (any value),
and then the
selection that follows can still take the CPU adapter when a hardware one exists.
The check was added so that `select()` fails early instead of falling through to the software renderer
(Slint pull request 11433),
and the variable was added for virtual machines without GPU access
(pull request 10252).
Neither says the selection itself is meant to exclude CPU adapters,
but Slint's linuxkms backend does exactly that for its own adapter:
pull request 14023 (merged 2026-10-13,
after 1.18.1) refuses a CPU Vulkan adapter "unless `SLINT_WGPU_CPU` is set"
in `internal/backends/linuxkms/renderer/dmabuf.rs`:

```rust
// slint-ui/slint pull request 14023, internal/backends/linuxkms/renderer/dmabuf.rs (hunk at line 100)
    if info.device_type == wgpu::DeviceType::Cpu && std::env::var_os("SLINT_WGPU_CPU").is_none() {
        return Err(PlatformError::Other(format!(
            "The Vulkan adapter for dma-buf rendering, {}, renders on the CPU. \
             Set SLINT_WGPU_CPU to use it anyway",
```

### Slint's default limits are the downlevel set

```rust
// i-slint-core-1.18.1/graphics/wgpu_30.rs:62-66 and 273-277
                power_preference: wgpu_30::PowerPreference::from_env().unwrap_or_default(),
...
                device_required_limits: wgpu_30::Limits::downlevel_webgl2_defaults(),
...
fn default_device_descriptor(adapter: &wgpu::Adapter) -> wgpu::DeviceDescriptor<'static> {
    ...
        required_limits: adapter.limits(),
```

`WGPUConfiguration::default()` is `Automatic(WGPUSettings::default())` (`wgpu_30.rs:95-98`),
so the shortest way to ask
for wgpu gets the downlevel limits;
`default_device_descriptor` (no configuration) gets the adapter's.
Slint's own examples override the limits when they need more:
`examples/wgpu_texture/lib.rs:131-133` sets features and
a limit on the settings,
and `examples/bevy/slint-hosts-bevy/main.rs:107-109` replaces `device_required_limits` with
`Limits::default()` (in the tag `v1.18.1`).

### Mesa note: the default GPU is the display server's

Mesa documents (`docs.mesa3d.org/envvars.html`,
entry `DRI_PRIME`):
"the default GPU is the one used by Wayland/Xorg or the one connected to a display."
and that for Vulkan "select" means the GPU is first in the reported physical devices list.
`MESA_VK_DEVICE_SELECT` with `vid:did` makes that PCI device "the first device in `vkEnumeratePhysicalDevices`".
The layer reports its decision with `MESA_VK_DEVICE_SELECT_DEBUG=1`.
Inside the headless nested compositor of this repository,
which opened the integrated GPU (`renderD129`):

```text
device-select: device_select_find_wayland_pci_default selected 1
slint chose AMD Ryzen 9 7950X 16-Core Processor (RADV RAPHAEL_MENDOCINO) (IntegratedGpu, driver radv); ...
```

In the real session,
with `WAYLAND_DISPLAY=wayland-0`,
the same layer selected the discrete GPU (index 0).
`DRI_PRIME=1` put the integrated GPU first on that listing,
with the debug lines
`DRI_PRIME=1, default_idx so far: 0` and `find_non_cpu_skip selected 1`.
The agent that measured the IDE's GPU choice saw `selected 1` as well when `DRI_PRIME=1` was exported to the nested
compositor too (`doc/handover/slint-ide-0x.md`,
the entry of agent `a090eeb1ae38efc05`);
I did not repeat that run.
Two more details of the loader,
from `docs/LoaderInterfaceArchitecture.md` of `KhronosGroup/Vulkan-Loader`:
the loader sorts physical devices itself and `VK_LOADER_DISABLE_SELECT=1` turns that sort off,
and
`VK_LOADER_DEVICE_SELECT=vid:did` moves one device first.
On this host `VK_DRIVER_FILES` with the llvmpipe driver file named before the RADV one did not put llvmpipe first
(the listing stayed RADV,
RADV,
llvmpipe,
with or without `VK_LOADER_DISABLE_SELECT=1`),
and `VK_LOADER_DEVICE_SELECT=0x10005:0x0` moved llvmpipe second,
not first;
`MESA_VK_DEVICE_SELECT=10005:0` is what put it first.

## Verification

Versions:
wgpu 30.0.1,
`wgpu-core` 30.0.1 (crates.io checksum
`14c018fce9b6270aa203c2fdd56f3cce996713534bd757e4ea58c8560b121f14`),
Slint 1.18.1 (git commit
`372cf0ee5577c3dfec309a45e7b778ba4e81b734`,
the `vcs_info` of the registry copy of `i-slint-core` and the tag
`v1.18.1`),
Vello 0.11.0,
rustc 1.101.0-nightly (2026-10-17),
Mesa 26.2.x.
Host adapters,
`vulkaninfo --summary`:
RX 7600 (discrete,
`0x1002:0x7480`),
Raphael (integrated,
`0x1002:0x164e`),
llvmpipe (CPU,
`0x10005:0x0`).

### Program 1: wgpu alone

```toml
# adapter-pick/Cargo.toml
[package]
name = "adapter-pick"
version = "0.1.0"
edition = "2024"

[workspace]

[dependencies]
wgpu = { version = "=30.0.1", default-features = false, features = ["vulkan", "wgsl", "std"] }
pollster = "0.4"
```

```rust
// adapter-pick/src/main.rs
// Usage: adapter-pick [none|low|high] [--fallback]
use wgpu::{
    Backends, DeviceDescriptor, Instance, InstanceDescriptor, Limits, PowerPreference, RequestAdapterOptions,
};

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let power_preference = match arguments.first().map(String::as_str) {
        Some("low") => PowerPreference::LowPower,
        Some("high") => PowerPreference::HighPerformance,
        _ => PowerPreference::None,
    };
    let force_fallback_adapter = arguments.iter().any(|argument| argument == "--fallback");

    let mut descriptor = InstanceDescriptor::new_without_display_handle();
    descriptor.backends = Backends::VULKAN;
    let instance = Instance::new(descriptor);
    for (position, adapter) in pollster::block_on(instance.enumerate_adapters(Backends::VULKAN)).iter().enumerate() {
        let info = adapter.get_info();
        println!("listed {position}: {} ({:?}, driver {})", info.name, info.device_type, info.driver);
    }

    let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference,
        force_fallback_adapter,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .expect("an adapter");
    let info = adapter.get_info();
    println!("chosen with {power_preference:?} fallback={force_fallback_adapter}: {} ({:?}, driver {})",
        info.name, info.device_type, info.driver);

    for (label, limits) in [
        ("downlevel_webgl2_defaults (Slint WGPUSettings::default)", Limits::downlevel_webgl2_defaults()),
        ("adapter.limits() (Slint with no configuration)", adapter.limits()),
    ] {
        let descriptor = DeviceDescriptor {
            required_limits: limits.using_resolution(adapter.limits()),
            ..Default::default()
        };
        match pollster::block_on(adapter.request_device(&descriptor)) {
            Ok((device, _queue)) => println!(
                "device limits {label}: max_storage_buffers_per_shader_stage={}",
                device.limits().max_storage_buffers_per_shader_stage
            ),
            Err(error) => println!("device refused with {label}: {error}"),
        }
    }
}
```

### Program 2: Slint's own selection

```toml
# slint-adapter/Cargo.toml
[package]
name = "slint-adapter"
version = "0.1.0"
edition = "2024"

[workspace]

[dependencies]
slint = { version = "=1.18.1", default-features = false, features = [
    "std",
    "backend-default",
    "renderer-femtovg-wgpu",
    "unstable-wgpu-30",
    "unstable-winit-030",
    "compat-1-2",
] }
```

```rust
// slint-adapter/src/main.rs
use slint::wgpu_30::{WGPUConfiguration, WGPUSettings};
use slint::{BackendSelector, GraphicsAPI, RenderingState};

slint::slint! {
    export component App inherits Window {
        width: 200px;
        height: 100px;
        Text { text: "adapter probe"; }
    }
}

fn main() {
    let configuration = WGPUConfiguration::Automatic(WGPUSettings::default());
    let selected = BackendSelector::new()
        .backend_name("winit".to_string())
        .renderer_name("femtovg-wgpu".to_string())
        .require_wgpu_30(configuration)
        .select();
    if let Err(error) = selected {
        println!("selector refused: {error}");
        std::process::exit(3);
    }
    let app = App::new().unwrap();
    app.window()
        .set_rendering_notifier(|state, api| {
            if let (RenderingState::RenderingSetup, GraphicsAPI::WGPU30 { device, .. }) = (state, api) {
                let info = device.adapter_info();
                println!(
                    "slint chose {} ({:?}, driver {}); max_storage_buffers_per_shader_stage={}",
                    info.name,
                    info.device_type,
                    info.driver,
                    device.limits().max_storage_buffers_per_shader_stage
                );
                let _ = slint::quit_event_loop();
            }
        })
        .unwrap();
    app.run().unwrap();
}
```

The window needs a Wayland display;
the run uses the repository's nested compositor in headless mode,
which hosts the
program and exits with it:

```sh
monochromatic-nested-wayland-session --headless --size 400x300 -- ./target/debug/slint-adapter
```

### Program 3: Vello on each limit set

```toml
# vello-limits/Cargo.toml
[package]
name = "vello-limits"
version = "0.1.0"
edition = "2024"

[workspace]

[dependencies]
vello = { version = "=0.11.0", default-features = false, features = ["wgpu"] }
pollster = "0.4"
wgpu = { version = "=30.0.1", default-features = false, features = ["vulkan", "wgsl", "std"] }
```

```rust
// vello-limits/src/main.rs
// Usage: vello-limits downlevel | default
use vello::wgpu;
use vello::{AaSupport, Renderer, RendererOptions};

fn main() {
    let which = std::env::args().nth(1).unwrap_or_else(|| "downlevel".to_string());
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::VULKAN;
    let instance = wgpu::Instance::new(descriptor);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
        .expect("an adapter");
    let limits = match which.as_str() {
        "downlevel" => wgpu::Limits::downlevel_webgl2_defaults(),
        _ => wgpu::Limits::default(),
    };
    let (device, _queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_limits: limits.using_resolution(adapter.limits()),
        ..Default::default()
    }))
    .expect("a device");
    println!("{which}: max_storage_buffers_per_shader_stage={}", device.limits().max_storage_buffers_per_shader_stage);
    let renderer = Renderer::new(
        &device,
        RendererOptions {
            use_cpu: false,
            antialiasing_support: AaSupport::area_only(),
            num_init_threads: None,
            pipeline_cache: None,
        },
    );
    match renderer {
        Ok(_) => println!("{which}: Renderer::new succeeded"),
        Err(error) => println!("{which}: Renderer::new returned an error: {error}"),
    }
}
```

### Catalog: runs that show the quirk

Program 1,
`MESA_VK_DEVICE_SELECT=10005:0 ./adapter-pick`:

```text
listed 0: llvmpipe (LLVM 22.1.8, 256 bits) (Cpu, driver llvmpipe)
listed 1: AMD Radeon RX 7600 (RADV NAVI33) (DiscreteGpu, driver radv)
listed 2: AMD Ryzen 9 7950X 16-Core Processor (RADV RAPHAEL_MENDOCINO) (IntegratedGpu, driver radv)
chosen with None fallback=false: llvmpipe (LLVM 22.1.8, 256 bits) (Cpu, driver llvmpipe)
device limits downlevel_webgl2_defaults (Slint WGPUSettings::default): max_storage_buffers_per_shader_stage=0
device limits adapter.limits() (Slint with no configuration): max_storage_buffers_per_shader_stage=48
```

Program 2 in the nested compositor,
with the variable in the environment of the compositor and so of the child:

- `MESA_VK_DEVICE_SELECT=10005:0`:
  `slint chose llvmpipe (LLVM 22.1.8, 256 bits) (Cpu, driver llvmpipe); max_storage_buffers_per_shader_stage=0`.
  `SLINT_WGPU_CPU` is unset,
  and the start-up check passed because the RX 7600 exists.
- `SLINT_WGPU_CPU=1 VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json`:
  `slint chose llvmpipe ...; max_storage_buffers_per_shader_stage=0` (only llvmpipe is installed for the process).
- Program 3,
  `./vello-limits downlevel`:
  `downlevel: max_storage_buffers_per_shader_stage=0`,
  then the panic in the
  symptom,
  exit status 101.

### Catalog: runs that work

- Program 1 with no variable:
  listed RX 7600,
  Raphael,
  llvmpipe;
  chosen the RX 7600.
- Program 1,
  `MESA_VK_DEVICE_SELECT=10005:0 ./adapter-pick high`:
  chosen the RX 7600.
  `... low`:
  chosen Raphael.
  `... none --fallback`:
  chosen llvmpipe (the filter that keeps only CPU adapters,
  as documented).
  A power preference sorts CPU adapters last.
- Program 2 with `MESA_VK_DEVICE_SELECT=10005:0 WGPU_POWER_PREF=high`:
  `slint chose AMD Radeon RX 7600 (RADV NAVI33) (DiscreteGpu, driver radv); max_storage_buffers_per_shader_stage=0`.
- Program 2 with `MESA_VK_DEVICE_SELECT=10005:0 WGPU_ADAPTER_NAME="RADV NAVI33"`:
  the RX 7600 again.
- Program 2 with `VK_DRIVER_FILES` naming only the llvmpipe driver file and `SLINT_WGPU_CPU` unset:
  the program prints `selector refused:` followed by the selector's message,
  which begins `WGPU 30.x rendering was required but no GPU-backed WGPU adapter is available`
  and ends `Set SLINT_WGPU_CPU=1 to allow CPU adapters.`,
  and exits with status 3.
- Program 3,
  `./vello-limits default`:
  `default: max_storage_buffers_per_shader_stage=8`,
  `default: Renderer::new succeeded`.

## Verified workarounds

### Open the device in the application and give it to Slint

`package/ide/app/src/launcher.rs` (function `install_backend`) calls `open_hardware_device`
(`package/ide/app/src/gpu_device.rs`),
which lists the Vulkan adapters,
keeps `DiscreteGpu` and `IntegratedGpu`
only (`gpu_device/adapter.rs:38-39`,
`is_hardware`),
requests the device with
`wgpu::Limits::default().using_resolution(adapter.limits())` (`gpu_device.rs:212`),
and passes it as
`WGPUConfiguration::Manual { instance, adapter, device, queue }` to `BackendSelector::require_wgpu_30`.
The Stage 2 section of `doc/planning/ide-gpu-snapshot-harness.md` records the guard run:
with the guard removed in a
debug binary and the lavapipe driver file,
Slint refused without `SLINT_WGPU_CPU` and started on llvmpipe with it.
Tradeoffs:

- The application owns adapter policy:
  it must implement the order,
  `WGPU_ADAPTER_NAME` and `WGPU_POWER_PREF` itself
  if it wants them,
  and a host without a hardware Vulkan adapter cannot start it.
- The leaked handles that this path needs in debug builds are a separate finding:
  `doc/troubleshooting/wgpu-core-snatch-lock-trace-aborts-at-thread-local-destruction.md`.

### Keep `Automatic` and state the limits

```rust
let mut settings = WGPUSettings::default();
settings.device_required_limits = wgpu::Limits::default();
BackendSelector::new().require_wgpu_30(WGPUConfiguration::Automatic(settings)).select()?;
```

This is what Slint's own bevy example does with `Limits::default()`.
Verified for the limit only:
Program 3 renders with those limits (`Limits::default()` gives 8 storage buffers per stage).
Tradeoff:
it does not change which adapter is taken;
the next workaround does.
Setting `WGPU_POWER_PREF=high` or `WGPU_ADAPTER_NAME=<substring>` in the environment fixes the choice (both verified
above),
at the price of making an environment variable part of the deployment,
and `high` prefers a discrete GPU over
an integrated one,
which a laptop user may not want.

## What does not work

- `SLINT_WGPU_CPU` unset as a guard:
  it only gates the start-up check,
  so a hardware GPU anywhere in the list lets the
  CPU adapter through when it is listed first.
- Ordering the loader with `VK_DRIVER_FILES` or `VK_LOADER_DEVICE_SELECT` to show the CPU-first case:
  llvmpipe stayed
  last or second on this host (see the Mesa note).
  `MESA_VK_DEVICE_SELECT` is the variable that works.
- Counting on the surface check:
  Slint passes `compatible_surface: Some(&surface)`,
  but llvmpipe can present to a
  Wayland surface,
  so the check does not remove it.

## Upstream filing artifact

### Upstream filing decision

`.out-of-scope/` entries (`bun-install.md`,
`cargo-workspace.md`,
`claude-code-upstream-bugs.md`,
`codex-harness.md`,
`jsr.md`,
`lightningcss.md`,
`low-impact-typescript-formatting.md`,
`module-es-monolith.md`,
`pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`,
`typescript-project-references.md`) name neither Slint nor wgpu for this
behavior;
`cargo-workspace.md` mentions Slint only as a library to keep off the host.

Duplicate search,
`gh search issues` and `gh search prs`,
open and closed:

- `slint-ui/slint`:
  `SLINT_WGPU_CPU` (issues:
  none;
  pull requests 14023,
  11433,
  10252,
  all merged),
  `llvmpipe`,
  `lavapipe`,
  `downlevel`,
  `downlevel_webgl2_defaults`,
  `WGPUSettings`,
  `force_fallback_adapter`,
  `request_adapter`,
  `wgpu power preference`,
  `wgpu adapter selection`,
  `WGPU adapter software`,
  `unstable-wgpu-30 limits`.
  Pull requests 10252 and 11433 were read in full (descriptions and comments);
  14023's description and diff were read.
  The nearest issue is 13260 "WGPU:
  Unused Mesa software/GL stack (EGL,
  gallium) is mapped & initialized at instance
  creation" (closed),
  about loading lavapipe and the GL stack at instance creation,
  not about which adapter is chosen.
  Nothing covers the winit selection or the default limits.
- `gfx-rs/wgpu`:
  the behavior is documented,
  so no defect is alleged and no search was needed;
  `PowerPreference::None` is specified as not considering power.

Two separate questions,
because they have different answers.

#### The CPU adapter in the `Automatic` selection

1. **Is it really upstream's fault?**
   Not wgpu's:
   `PowerPreference::None` is documented as not sorting,
   and `force_fallback_adapter` as keeping only
   fallback adapters.
   Slint's,
   softly yes:
   Slint's start-up check and `SLINT_WGPU_CPU` treat CPU adapters as opt-in,
   and its linuxkms backend enforces that at selection time (pull request 14023),
   while the winit path in `wgpu_30.rs` does not.
   Soft because the helper's own comment says it decides the software-renderer fallback,
   and because the only way I made Slint take llvmpipe was to tell Mesa to list it first;
   I found no unforced configuration on this host.
2. **Can upstream fix it?**
   Yes:
   the prototype is one helper and two call sites.
3. **Are they supporting this use case?**
   Yes for `Automatic` (the documented configuration,
   used by Slint's examples),
   and the linuxkms guard shows that CPU
   exclusion is wanted.
4. **Would the repo welcome our contribution?**
   Yes.
   `CONTRIBUTING.md` welcomes issues and pull requests,
   points AI assistants to `AGENTS.md`,
   asks that public API
   changes be discussed first,
   and asks contributors to make sure they wrote the contribution themselves;
   there are bug-report and feature-request issue templates;
   no AI ban or disclosure rule was found.
5. **Will they likely fix it?**
   Plausibly:
   the same maintainers added the guard for linuxkms on 2026-10-13,
   and nothing in the tracker declines it.
   Trunk's `wgpu_30.rs` selection code is unchanged from 1.18.1 (diffed;
   only a surfaceless descriptor helper and a
   `poll_once` visibility change differ).
6. **Have we prototyped a minimal fix compatible with their architecture?**
   Yes,
   see the prototype.

Result:
all six hold,
with 1 and 5 soft;
the draft below is fileable as written,
but the motivation is weak
(no unforced trigger found).
My recommendation is to hold it until a natural CPU-first ordering is documented (a hybrid laptop,
a virtual machine
with a virtual GPU next to llvmpipe),
and to not file for now.
Nothing was filed.

#### The default limits

Constraint 1 fails:
`WGPUSettings::default()` requests the downlevel limits,
Slint's own renderers draw with them,
and Slint's examples override `device_required_limits` when they need more (`examples/wgpu_texture/lib.rs:131-133`,
`examples/bevy/slint-hosts-bevy/main.rs:107-109`).
The inconsistency with the no-configuration path (`adapter.limits()`) is real but a design choice about a documented
field,
so the auto-prototype step is not triggered and there is no draft.
A documentation line on `WGPUSettings::device_required_limits` naming the default would be a docs change;
I did not draft it.

### Prototype

Disposable clone:
`gh repo clone slint-ui/slint <dir> -- --depth 1 --branch v1.18.1`,
`origin`
`https://github.com/slint-ui/slint.git`,
`HEAD` `372cf0ee5577c3dfec309a45e7b778ba4e81b734`
(equal to the `vcs_info` of the registry copy of `i-slint-core` 1.18.1).
The change is in `internal/core/graphics/wgpu_30.rs`:
a helper `without_cpu_adapter`,
called after both adapter
selections that go through `request_adapter` (the `Automatic` branch and the no-configuration branch),
never for `Manual`.
It leaves the adapter alone when `SLINT_WGPU_CPU` or `WGPU_ADAPTER_NAME` is set or when the adapter is not a CPU one;
otherwise it takes the first hardware adapter of the list that can present to the surface,
and returns an error that
names `SLINT_WGPU_CPU` when there is none.
The diff is 53 lines,
in one file:
[`slint-wgpu-automatic-configuration-adapter-and-limits.patch`](slint-wgpu-automatic-configuration-adapter-and-limits.patch).
It also applies to the current `internal/core/graphics/wgpu_30.rs` of trunk (`git apply --check`).
`wgpu_29.rs` has the same code and was not patched.

Built by pointing `[patch.crates-io]` of Program 2 at the clone for every `slint` and `i-slint-*` crate of its lock
(`i-slint-backend-android-activity`,
`-linuxkms`,
`-selector`,
`-winit`,
`i-slint-common`,
`-compiler`,
`-core`,
`-core-macros`,
`i-slint-renderer-femtovg`,
`-skia`,
`-software`,
`slint`,
`slint-macros`).
Before and after,
in the headless nested compositor:

- `MESA_VK_DEVICE_SELECT=10005:0`,
  unpatched:
  `slint chose llvmpipe (LLVM 22.1.8, 256 bits) (Cpu, driver llvmpipe); max_storage_buffers_per_shader_stage=0`.
  Patched:
  `slint chose AMD Radeon RX 7600 (RADV NAVI33) (DiscreteGpu, driver radv); ...`.
- `SLINT_WGPU_CPU=1 MESA_VK_DEVICE_SELECT=10005:0`,
  patched:
  llvmpipe,
  as asked.
- No variable,
  patched:
  `slint chose AMD Ryzen 9 7950X 16-Core Processor (RADV RAPHAEL_MENDOCINO) (IntegratedGpu, ...)`,
  the same as unpatched (the compositor's GPU,
  see the Mesa note).
- `--gpu software` compositor (llvmpipe),
  patched and unpatched:
  the RX 7600,
  so that configuration does not trigger the
  quirk either.

The error branch (hardware adapters exist,
none can present,
no `SLINT_WGPU_CPU`) was not exercised:
I could not build
that situation here.
No Slint test suite was run.

### Draft (do not file as-is: the motivation is weak, see the decision)

Title:
`require_wgpu_30(Automatic)` can select a CPU adapter although `SLINT_WGPU_CPU` is unset

Labels:
`bug` (suggested)

````md
## Description

`BackendSelector::require_wgpu_30(WGPUConfiguration::Automatic(WGPUSettings::default()))` picks its adapter with
`request_adapter(power_preference: None, force_fallback_adapter: false)`
(`internal/core/graphics/wgpu_30.rs`, the `Automatic` branch of `async_init_instance_adapter_device_queue_surface`).
wgpu does not sort for `PowerPreference::None` (`wgpu-core-30.0.1/src/instance.rs:571-578`), so the first adapter of the
Vulkan loader's list wins. `any_wgpu30_adapters_with_gpu` only checks that some adapter is not `DeviceType::Cpu`,
and the linuxkms backend refuses a CPU adapter unless `SLINT_WGPU_CPU` is set (pull request 14023), but the winit
path does not.

With Mesa told to list llvmpipe first (`MESA_VK_DEVICE_SELECT=10005:0`) on a machine with an AMD RX 7600 and
`SLINT_WGPU_CPU` unset, a Slint 1.18.1 window on `renderer-femtovg-wgpu` is created on llvmpipe:

    slint chose llvmpipe (LLVM 22.1.8, 256 bits) (Cpu, driver llvmpipe)

I could not find a configuration that does this without selecting llvmpipe explicitly.

## Reproduction

Program 2 of this document (Slint 1.18.1 with `renderer-femtovg-wgpu`, `unstable-wgpu-30`, `unstable-winit-030`),
run in a Wayland session with `MESA_VK_DEVICE_SELECT=10005:0` (llvmpipe's `vendor:device`) in the environment.

## Suggested fix

After the adapter is selected in the `Automatic` and no-configuration branches, replace a CPU adapter by the first
hardware adapter that supports the surface unless `SLINT_WGPU_CPU` or `WGPU_ADAPTER_NAME` is set. The 53-line patch
against `v1.18.1` (it applies to trunk) goes here when filing; I tested it only with the program above.
Is this the intended behavior of the winit path?
````

## Evidence and limits

- Source read in the registry copies named above and in a clone of `slint-ui/slint` at tag `v1.18.1`
  (`372cf0ee5577c3dfec309a45e7b778ba4e81b734`,
  `origin` `https://github.com/slint-ui/slint.git`),
  which is also the base of the prototype.
- The three programs are scratch crates on the machine that found the quirk (`~/temp/agent/gpu-findings/`),
  not in the
  repository;
  their source is in this document.
- One host,
  one Mesa,
  AMD only;
  a hybrid laptop and NVIDIA's proprietary driver were not available.
- Program 2 ran on FemtoVG over wgpu,
  not on a Vello-based Slint renderer.
- The surrounding study is `doc/planning/ide-gpu-snapshot-harness.md` (sections "The CPU adapter question" and "Why
  the launcher opens the device") and the README section "Renderer and GPU" of `package/ide/app/README.md`.
