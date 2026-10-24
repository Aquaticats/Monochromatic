//! A GPU snapshot harness for the native tests:
//!  Slint's FemtoVG renderer draws a window into a texture on a hardware Vulkan device,
//!  without a display server,
//!  and `Window::take_snapshot` reads the pixels back.
//!
//! A test calls `install_gpu_platform` first,
//!  before it creates any window,
//!  and then reads pixels with `window.window().take_snapshot()`
//!  or the `snapshot` helper here.
//!  Slint keeps one platform per thread,
//!  and the harness installs its platform once per thread;
//!  a thread that already has another platform (the headless testing backend,
//!  because a window was created first) makes the install panic.
//!  No CPU rasterizer is used:
//!  an adapter of the CPU kind (lavapipe,
//!  llvmpipe) is refused with a message naming every adapter that was seen.

/// The device the application itself opens:
///  a hardware Vulkan adapter with full limits,
///  and never a CPU one.
use crate::gpu_device::open_hardware_device_capped;
/// What:
///  `FemtoVGWGPURenderer` is Slint's FemtoVG renderer that draws into a `wgpu` texture,
///  `Platform` is the trait a custom Slint platform implements,
///  `PlatformError` is its error type,
///  and `WindowAdapter` is the trait that connects one Slint window to a renderer and a size.
/// Why:
///  The harness is a custom platform whose windows render on the GPU;
///  Slint's own testing backend can only be built with its software or Skia renderers
///  (`i-slint-backend-testing-1.18.1/testing_backend.rs`,
///  `create_headless_renderer`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { FemtoVGWGPURenderer, Platform, PlatformError, WindowAdapter } from 'slint/platform';
/// ```
use slint::platform::{
    Clipboard, Platform, PlatformError, WindowAdapter, WindowEvent,
    femtovg_renderer::FemtoVGWGPURenderer, set_platform,
};
/// The `wgpu` crate as Slint re-exports it,
///  so the harness uses exactly the wgpu release (30) that Slint's renderer was built against.
use slint::wgpu_30::wgpu;
/// Window sizes,
///  the pixel type of a snapshot,
///  and the window handle.
use slint::{PhysicalSize, Rgba8Pixel, SharedPixelBuffer, Window, WindowSize};
/// Single-threaded shared ownership,
///  as Slint's adapters are shared.
use std::{
    cell::{Cell, OnceCell, RefCell},
    rc::Rc,
};

/// Window size before the component states a preferred size,
///  the size Slint's testing backend also uses.
const DEFAULT_WIDTH: u32 = 800;
/// Height of the same default window.
const DEFAULT_HEIGHT: u32 = 600;
/// The largest 2D texture the harness device offers:
/// 8192 pixels,
/// wgpu's guaranteed minimum.
const HARNESS_TEXTURE_LIMIT: u32 = 8192;

/// What:
///  One Slint window of the harness:
///  the `Window` handle,
///  the renderer that draws it on the GPU,
///  and its size in physical pixels (`Cell` is a value that can be replaced through a shared reference).
/// Why:
///  Slint asks the adapter for its renderer and its size,
///  and the test's `take_snapshot` call reaches the renderer through it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class GpuWindow implements WindowAdapter { window: Window; renderer: Renderer; size: PhysicalSize }
/// ```
struct GpuWindow {
    /// The handle the component and the tests use.
    window: Window,
    /// Draws the scene into a wgpu texture;
    ///  its snapshot reads that texture back.
    renderer: FemtoVGWGPURenderer,
    /// Physical size;
    ///  zero width means no size was set yet.
    size: Cell<PhysicalSize>,
}

/// The window adapter contract:
///  the handle,
///  the size,
///  the renderer,
///  and the first layout.
impl WindowAdapter for GpuWindow {
    /// The handle the component and the tests use.
    fn window(&self) -> &Window {
        return &self.window;
    }

    /// Size before the component states one:
    ///  the default,
    ///  as in Slint's testing backend.
    fn size(&self) -> PhysicalSize {
        if self.size.get().width == 0 {
            return PhysicalSize::new(DEFAULT_WIDTH, DEFAULT_HEIGHT);
        }
        return self.size.get();
    }

    /// Mirrors Slint's testing window:
    ///  the window learns the logical size at scale 1 first,
    ///  then the adapter stores the physical size.
    fn set_size(&self, size: WindowSize) {
        self.window.dispatch_event(WindowEvent::Resized {
            size: size.to_logical(1.0),
        });
        self.size.set(size.to_physical(1.0));
    }

    /// The GPU renderer that draws and reads back this window.
    fn renderer(&self) -> &dyn slint::platform::Renderer {
        return &self.renderer;
    }

    /// The first layout gives the window the preferred size of its component.
    fn update_window_properties(&self, properties: slint::platform::WindowProperties<'_>) {
        if self.size.get().width == 0 {
            let preferred = properties.layout_constraints().preferred;
            self.size
                .set(preferred.to_physical(self.window.scale_factor()));
        }
    }
}

/// What:
///  The one GPU device of the process:
///  the `wgpu` instance,
///  device,
///  and queue.
/// Why:
///  `Instance`,
///  `Device`,
///  and `Queue` are cheap handles to the same device,
///  so each window clones them.
///  The struct is leaked in `install_gpu_platform` and never dropped:
///  wgpu's queue reads a thread-local value while it is dropped,
///  and Slint drops its platform while the test thread's thread-locals are being destroyed,
///  which aborts the process ("cannot access a Thread Local Storage value during or after destruction").
///  A test process ends right after its one test,
///  so the leak lasts as long as the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GpuDevice = { instance: GPUInstance; device: GPUDevice; queue: GPUQueue };
/// ```
struct GpuDevice {
    /// The wgpu instance.
    instance: wgpu::Instance,
    /// The hardware device.
    device: wgpu::Device,
    /// The queue of that device.
    queue: wgpu::Queue,
}

/// What:
///  The platform every window of the process is created from;
///  `&'static GpuDevice` is a borrowed reference that lives for the whole process.
/// Why:
///  A leaked device (see `GpuDevice`) can be lent for `'static`,
///  so dropping the platform drops nothing of wgpu.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class GpuPlatform implements Platform { gpu: GpuDevice }
/// ```
struct GpuPlatform {
    /// The leaked device shared by all windows.
    gpu: &'static GpuDevice,
    /// The text of the default clipboard;
    ///  `RefCell` lets the platform change it through a shared reference.
    clipboard: RefCell<Option<String>>,
}

/// The platform contract:
///  every window is a `GpuWindow` on the shared device.
impl Platform for GpuPlatform {
    /// A new window with its own FemtoVG renderer on the shared GPU device.
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        let renderer = FemtoVGWGPURenderer::new(
            self.gpu.instance.clone(),
            self.gpu.device.clone(),
            self.gpu.queue.clone(),
        )?;
        // What: `Rc::new_cyclic` builds the adapter and lets it hold a weak link to itself, which `Window::new` needs.
        // Why: The window and the adapter refer to each other.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const adapter = new GpuWindow(); adapter.window = new Window(weakRef(adapter));
        // ```
        let adapter: Rc<GpuWindow> = Rc::new_cyclic(|weak| {
            return GpuWindow {
                window: Window::new(weak.clone() as _),
                renderer,
                size: Cell::new(PhysicalSize::new(0, 0)),
            };
        });
        return Ok(adapter);
    }

    /// The default clipboard lives in memory,
    ///  as in Slint's testing backend,
    ///  so Copy and Paste in a test never touch the desktop clipboard.
    fn set_clipboard_text(&self, text: &str, clipboard: Clipboard) {
        if clipboard == Clipboard::DefaultClipboard {
            *self.clipboard.borrow_mut() = Some(text.to_string());
        }
    }

    /// The text last copied to the default clipboard;
    ///  the selection clipboard is not kept.
    fn clipboard_text(&self, clipboard: Clipboard) -> Option<String> {
        if clipboard == Clipboard::DefaultClipboard {
            return self.clipboard.borrow().clone();
        }
        return None;
    }
}

/// What:
///  `install_once` opens the hardware device with the largest texture `limit` it may offer
///  (`None` for the adapter's own),
///  makes a `GpuPlatform` of it,
///  and registers it with Slint for the rest of the thread.
///  It returns the adapter's name.
/// Why:
///  It panics when there is no hardware adapter or when another platform was installed before,
///  so a test can never silently draw with another renderer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function installOnce(limit?: number): string { /* throws */ }
/// ```
fn install_once(limit: Option<u32>) -> String {
    // The error text lists the adapters seen and every remedy, so a failed test run explains itself.
    // The usual device offers textures of 8192 pixels on a side at most, the smallest limit wgpu guarantees,
    // although this host's GPU offers more: every native test cuts its tiles for the weakest device the application
    // supports, so the texture rule is proven on every host.
    let opened = match open_hardware_device_capped(limit) {
        Ok(device) => device,
        Err(error) => panic!("no hardware Vulkan device for the GPU snapshot harness: {error}"),
    };
    let name = opened.summary.name.clone();
    // What: `Box::new` puts the device on the heap and `Box::leak` gives up ownership, returning a `&'static` reference.
    // Why: Nothing may drop the wgpu handles at thread exit (see `GpuDevice`).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const gpu = new GpuDevice(instance, device, queue); // never released
    // ```
    let gpu: &'static GpuDevice = Box::leak(Box::new(GpuDevice {
        instance: opened.instance,
        device: opened.device,
        queue: opened.queue,
    }));
    let platform = GpuPlatform {
        gpu,
        clipboard: RefCell::new(None),
    };
    set_platform(Box::new(platform))
        .expect("no Slint platform was installed before the GPU snapshot harness");
    DEVICE.with(|held| held.set(Some(gpu)));
    return name;
}

std::thread_local! {
    /// The leaked device once this thread installed the platform.
    static DEVICE: Cell<Option<&'static GpuDevice>> = const { Cell::new(None) };
    /// The adapter name once this thread installed the platform;
    ///  Slint keeps its platform per thread.
    static INSTALLED: OnceCell<String> = const { OnceCell::new() };
}

/// What:
///  `install_gpu_platform` installs the GPU platform on the calling thread the first time
///  and returns the adapter's name so a test can print what it ran on;
///  later calls on the same thread return the name without installing again.
/// Why:
///  A test or the fixture it shares calls it before the first window is created.
///  Repeated calls are harmless,
///  so every window a fixture builds can call it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function installGpuPlatform(): string { return installed ??= installOnce(); }
/// ```
pub(super) fn install_gpu_platform() -> String {
    return INSTALLED.with(|installed| {
        return installed
            .get_or_init(|| return install_once(Some(HARNESS_TEXTURE_LIMIT)))
            .clone();
    });
}

/// What:
///  `install_uncapped_gpu_platform` installs the GPU platform with the adapter's own texture limit
///  (16384 pixels on a side on most GPUs of this decade),
///  as `install_gpu_platform` does otherwise.
///  Only the first install of a thread counts.
/// Why:
///  A test of the tallest tile the application can build on a capable device needs a device that offers it,
///  and the application opens its device without a cap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function installUncappedGpuPlatform(): string { return installed ??= installOnce(undefined); }
/// ```
pub(super) fn install_uncapped_gpu_platform() -> String {
    return INSTALLED.with(|installed| {
        return installed.get_or_init(|| return install_once(None)).clone();
    });
}

/// The pixels of a window as RGBA bytes (red,
///  green,
///  blue,
///  alpha),
///  row by row from the top,
///  read back from the GPU texture the window was drawn into:
///  the GPU counterpart of the software renderer's `take_snapshot`.
pub(super) fn snapshot(window: &Window) -> SharedPixelBuffer<Rgba8Pixel> {
    return window
        .take_snapshot()
        .expect("a GPU snapshot of the window");
}

/// The device and queue of the platform this thread installed,
///  for a test that renders a scene without a window.
pub(super) fn device_handles() -> (wgpu::Device, wgpu::Queue) {
    install_gpu_platform();
    let gpu = DEVICE
        .with(|held| return held.get())
        .expect("the GPU platform was installed");
    return (gpu.device.clone(), gpu.queue.clone());
}
