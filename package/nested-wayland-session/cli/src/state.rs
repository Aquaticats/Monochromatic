//! The compositor's central state and its construction.
//!
//! `Compositor` is the single value the calloop event loop carries.
//!  It owns every
//! Wayland protocol sub-state (compositor,
//!  xdg-shell,
//!  shm,
//!  output,
//!  seat,
//! data-device),
//!  the desktop `Space` the one window lives on,
//!  the winit rendering
//! backend,
//!  and the dmabuf import state.
//!  Smithay compositors are structured as one
//! big owning struct like this;
//!  the per-protocol behaviour is split into the
//! `handler` module.

/// What:
///      `use std::{ffi::OsString, process::Child, sync::Arc};`.
///  Three std types:
///             - `OsString`:
///  an owned,
///  OS-native string (bytes the OS uses for names;
///               sibling:
///  the UTF-8 `String`).
///  Wayland socket names come back as this.
///             - `Child`:
///  a handle to a spawned OS process (from `std::process`).
///             - `Arc<T>`:
///  an Atomically Reference-Counted shared owner of a heap
///               value (thread-safe sibling of the single-threaded `Rc<T>`;
///  both
///               unlike `Box<T>`,
///  which is a single owner).
/// Why:
///       The socket name is stored owned;
///  the hosted app is a `Child`;
///  new
///           Wayland clients are inserted behind an `Arc` because wayland-server
///           shares client data across threads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // OsString ~ string; Child ~ a Node ChildProcess handle; Arc<T> ~ a shared ref.
/// ```
use std::{ffi::OsString, process::Child, sync::Arc};

/// Context turns a failed startup step into an error that names the step,
///  so the process reports it and exits nonzero instead of panicking.
use anyhow::{Context, Result};

/// Clipboard management globals are owned by this nested display,
///  never the host.
use crate::handler::clipboard::ClipboardProtocols;

/// The private Settings portal is owned by this session,
///  never shared with the host.
use crate::appearance_portal::AppearancePortal;

/// The private session bus and the taskbar progress listener that reads it.
use crate::{
    input_pointer::HeldButtons, launcher_listen::LauncherLog, plasma_services::PlasmaServices,
    private_bus::PrivateBus,
};

/// Stall bookkeeping that keeps the hosted client drawing while the parent is silent.
use crate::frame_pacing::FramePacing;

/// The logical screen size and output scale,
///  and the globals that tell clients the scale.
use crate::{render_backend::RenderBackend, screen::ScreenState, screen_geometry::ScreenGeometry};

/// What:
///      A grouped `use` of Smithay items.
///  Each path names a type used below;
///  the
///           braces just avoid repeating the common `smithay::...` prefix.
/// Why:
///       Bring the compositor building blocks into scope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Space, Window, Seat, ... } from "smithay";
/// ```
use smithay::{
    backend::renderer::{damage::OutputDamageTracker, gles::GlesTexture},
    desktop::{PopupManager, Space, Window},
    input::{Seat, SeatState},
    output::Output,
    reexports::{
        calloop::{generic::Generic, EventLoop, Interest, LoopHandle, LoopSignal, Mode, PostAction},
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            Display, DisplayHandle,
        },
    },
    utils::{Logical, Point},
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        dmabuf::{DmabufFeedback, DmabufGlobal, DmabufState},
        output::OutputManagerState,
        selection::data_device::DataDeviceState,
        shell::xdg::XdgShellState,
        shm::ShmState,
        socket::ListeningSocketSource,
    },
};

/// Key repeat rate announced to clients unless `--repeat-rate` says otherwise,
///  in repeats per second.
///
/// What:
///      `pub const DEFAULT_REPEAT_RATE: i32 = 25;`.
/// Why:
///       The value earlier versions announced,
///  and winit's own default.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DEFAULT_REPEAT_RATE = 25;
/// ```
pub const DEFAULT_REPEAT_RATE: i32 = 25;

/// Key repeat delay announced to clients unless `--repeat-delay` says otherwise,
///  in milliseconds.
///
/// What:
///      `pub const DEFAULT_REPEAT_DELAY_MS: i32 = 200;`.
/// Why:
///       The value earlier versions announced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DEFAULT_REPEAT_DELAY_MS = 200;
/// ```
pub const DEFAULT_REPEAT_DELAY_MS: i32 = 200;

/// The whole compositor,
///  carried by the calloop event loop as its shared data.
///
/// What:
///      `pub struct Compositor { ... }`.
///  A large record type owning every piece
///           of compositor state.
///  Fields prefixed `_` (like `_dmabuf_global`) are
///           kept only so their `Drop` does not run early (dropping a global would
///           tear down that Wayland global);
///  the leading underscore silences the
///           "unused field" lint.
/// Why:
///       Smithay dispatches protocol events by calling handler methods on one
///           `&mut State`,
///  so all state a handler might touch lives here together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Compositor { startTime; socketName; space; seat; backend; /* ... */ }
/// ```
pub struct Compositor {
    /// Monotonic clock start,
    ///  used to timestamp frame-callback events to clients.
    ///
    /// What:
    ///      `pub start_time: std::time::Instant`.
    ///  `Instant` is an opaque
    ///           monotonic timestamp (not a wall clock;
    ///  siblings:
    ///  `SystemTime`).
    /// Why:
    ///       `Window::send_frame` wants elapsed time since start.
    pub start_time: std::time::Instant,

    /// Name of the Wayland socket this compositor listens on,
    ///  e.g. `wayland-1`.
    ///
    /// What:
    ///      `pub socket_name: OsString`.
    ///  Owned OS string.
    /// Why:
    ///       Passed to the hosted child via `WAYLAND_DISPLAY` so it connects here.
    pub socket_name: OsString,

    /// Cloneable handle to the wayland-server display for inserting clients / globals.
    ///
    /// What:
    ///      `pub display_handle: DisplayHandle`.
    ///  A cheap clone of the display's
    ///           control handle.
    /// Why:
    ///       Handlers create globals and look up clients through it.
    pub display_handle: DisplayHandle,

    /// The two-dimensional plane the single hosted window is mapped onto.
    ///
    /// What:
    ///      `pub space: Space<Window>`.
    ///  `Space` maps `Window`s and `Output`s onto
    ///           a shared coordinate plane and drives rendering.
    /// Why:
    ///       Even a single-app fixture uses `Space` so it can reuse Smithay's
    ///           `render_output` helper.
    pub space: Space<Window>,

    /// Signal used to stop the event loop from anywhere (e.g. child exit).
    ///
    /// What:
    ///      `pub loop_signal: LoopSignal`.
    ///  A handle whose `.stop()` ends the loop.
    /// Why:
    ///       The child-exit timer and the `CloseRequested` event both stop the loop.
    pub loop_signal: LoopSignal,

    /// wl_compositor / wl_subcompositor protocol state.
    ///
    /// What:
    ///      `pub compositor_state: CompositorState`.
    /// Why:
    ///       Tracks surface trees and buffer commits for all clients.
    pub compositor_state: CompositorState,

    /// xdg-shell protocol state (toplevels and popups).
    ///
    /// What:
    ///      `pub xdg_shell_state: XdgShellState`.
    /// Why:
    ///       The hosted app's main window is an xdg_toplevel.
    pub xdg_shell_state: XdgShellState,

    /// wl_shm (shared-memory buffers) protocol state.
    ///
    /// What:
    ///      `pub shm_state: ShmState`.
    /// Why:
    ///       Advertised for completeness;
    ///  the GPU app uses dmabuf,
    ///  but clients
    ///           expect wl_shm to exist,
    ///  and cursor themes may use it.
    pub shm_state: ShmState,

    /// wl_output / xdg-output manager state.
    ///
    /// What:
    ///      `pub output_manager_state: OutputManagerState`.
    /// Why:
    ///       Advertises the nested screen's geometry to the client.
    pub output_manager_state: OutputManagerState,

    /// Seat (input device group) protocol state.
    ///
    /// What:
    ///      `pub seat_state: SeatState<Compositor>`.
    ///  Generic over the state type
    ///           so it can call back into focus handling.
    /// Why:
    ///       Owns the keyboard/pointer the fixture injects synthetic input through.
    pub seat_state: SeatState<Compositor>,

    /// wl_data_device (clipboard / drag-and-drop) protocol state.
    ///
    /// What:
    ///      `pub data_device_state: DataDeviceState`.
    /// Why:
    ///       Needed so keyboard focus can carry a data-device offer;
    ///  clients
    ///           expect it to exist.
    pub data_device_state: DataDeviceState,

    /// Nested clipboard manager globals for wlr and ext data-control clients.
    pub clipboard: ClipboardProtocols,

    /// Tracker for xdg-shell popups (menus,
    ///  tooltips).
    ///
    /// What:
    ///      `pub popups: PopupManager`.
    /// Why:
    ///       Slint may open popups;
    ///  this keeps them configured and cleaned up.
    pub popups: PopupManager,

    /// The single seat all input flows through.
    ///
    /// What:
    ///      `pub seat: Seat<Compositor>`.
    /// Why:
    ///       Synthetic pointer/keyboard events are sent via this seat's handles.
    pub seat: Seat<Compositor>,

    /// The single nested output (screen) the app fills.
    ///
    /// What:
    ///      `pub output: Output`.
    ///  A cloneable output handle.
    /// Why:
    ///       Resizing changes its mode;
    ///  rendering and frame callbacks reference it.
    pub output: Output,

    /// The logical screen size and output scale,
    ///  plus the globals that carry the scale.
    ///
    /// What:
    ///      `pub screen: ScreenState`.
    /// Why:
    ///       The `scale` and `resize` commands and the toplevel configure read the logical
    ///           size and scale from here;
    ///  the output mode holds only the physical framebuffer.
    pub screen: ScreenState,

    /// The winit graphics backend:
    ///  the nested window plus its GLES/EGL renderer.
    ///
    /// What:
    ///      `pub backend: RenderBackend`.
    ///  The winit window with its renderer,
    ///           or the surfaceless renderer in `--headless` mode.
    ///  Parameterised by
    ///           the renderer type it drives.
    /// Why:
    ///       Rendering,
    ///  dmabuf import,
    ///  and screenshot readback all go through it.
    pub backend: RenderBackend,

    /// Offscreen texture screenshots and recordings render into,
    ///  reused while its size fits.
    ///
    /// What:
    ///      `pub capture_texture: Option<GlesTexture>`.
    ///  `None` until the first capture.
    /// Why:
    ///       Captures must have the output's current size at once;
    ///  the window's back buffer
    ///           only takes a new size after the next swap.
    pub capture_texture: Option<GlesTexture>,

    /// Damage tracker that decides which screen regions need redrawing.
    ///
    /// What:
    ///      `pub damage_tracker: OutputDamageTracker`.
    /// Why:
    ///       `render_output` needs it to compute and submit damage efficiently.
    pub damage_tracker: OutputDamageTracker,

    /// dmabuf (GPU buffer sharing) protocol state.
    ///
    /// What:
    ///      `pub dmabuf_state: DmabufState`.
    /// Why:
    ///       The `DmabufHandler` imports client GPU buffers through it;
    ///  this is
    ///           the whole point of hosting the real GPU render path.
    pub dmabuf_state: DmabufState,

    /// The dmabuf global,
    ///  kept alive for the program's lifetime.
    ///
    /// What:
    ///      `pub _dmabuf_global: DmabufGlobal`.
    ///  Underscore-prefixed:
    ///  never read,
    ///           only held so dropping it does not remove the `zwp_linux_dmabuf_v1`
    ///           global from the display.
    /// Why:
    ///       Advertise dmabuf support to the client for as long as we run.
    pub _dmabuf_global: DmabufGlobal,

    /// The dmabuf v4 default feedback,
    ///  or `None` when we fell back to v3.
    ///
    /// What:
    ///      `pub _dmabuf_feedback: Option<DmabufFeedback>`.
    ///  Held for its lifetime.
    /// Why:
    ///       Keeps the negotiated modifier feedback alive;
    ///  `None` means v3 (no
    ///           feedback) was used instead.
    pub _dmabuf_feedback: Option<DmabufFeedback>,

    /// The single hosted client process,
    ///  if it has been spawned.
    ///
    /// What:
    ///      `pub child: Option<Child>`.
    ///  `Some(child)` once spawned,
    ///  `None` before.
    /// Why:
    ///       The exit-poll timer calls `try_wait` on it and stops the loop when it
    ///           exits,
    ///  propagating the app's exit code.
    pub child: Option<Child>,

    /// Deadline for force-stopping client that ignores compositor close request.
    pub shutdown_deadline: Option<std::time::Instant>,

    /// The private appearance portal,
    ///  when `--color-scheme` started one.
    ///
    /// What:
    ///      `pub appearance_portal: Option<AppearancePortal>`.
    ///  `Some` owns the private
    ///           session bus and its Settings service;
    ///  `None` means the hosted client
    ///           inherited its usual session bus.
    /// Why:
    ///       The `color-scheme` control command has only `&mut Compositor`,
    ///  so the
    ///           handle it switches must be reachable from state.
    ///  Dropping the state stops
    ///           the private bus and removes its socket directory.
    pub appearance_portal: Option<AppearancePortal>,

    /// The fake notification and Plasma job services,
    ///  when `--notifications` asked for them.
    ///
    /// What:
    ///      `pub notifications: Option<PlasmaServices>`.
    /// Why:
    ///       The notification and job verbs read and drive these records.
    ///  Declared before the private bus so the services' connection closes first.
    pub notifications: Option<PlasmaServices>,

    /// The private session bus,
    ///  when an option that needs one started it.
    ///
    /// What:
    ///      `pub private_bus: Option<Arc<PrivateBus>>`.
    ///  `Some` holds the `dbus-daemon` the hosted client's session bus address points at.
    /// Why:
    ///       The bus must outlive every holder of its address,
    ///  so the compositor keeps one owner here and the portal keeps another.
    ///  Declared after the portal so the portal's connection closes first.
    pub private_bus: Option<Arc<PrivateBus>>,

    /// The taskbar progress listener on the private bus,
    ///  when a bus exists.
    ///
    /// What:
    ///      `pub launcher: Option<LauncherLog>`.
    /// Why:
    ///       The `launcher` verbs read its history;
    ///  `None` means no private bus,
    ///  so those verbs refuse with the reason.
    pub launcher: Option<LauncherLog>,

    /// Pointer buttons that `button down` is holding.
    ///
    /// What:
    ///      `pub held_pointer_buttons: HeldButtons`.
    /// Why:
    ///       `button down` and `button up` must refuse a repeated request,
    ///  which needs the record of what the seat is holding.
    pub held_pointer_buttons: HeldButtons,

    /// When the parent compositor last presented,
    ///  and whether a timer paces the client instead.
    ///
    /// What:
    ///      `pub frame_pacing: FramePacing`.
    ///  A small owned record.
    /// Why:
    ///       The live redraw path records each presentation here,
    ///  and the fallback timer
    ///           reads it to decide whether the hosted client is being starved.
    pub frame_pacing: FramePacing,

    /// The hosted app's exit code once it has exited,
    ///  else `None`.
    ///
    /// What:
    ///      `pub child_exit_code: Option<i32>`.
    ///  Signed 32-bit,
    ///  matching a process
    ///           exit status code.
    /// Why:
    ///       `main` propagates this as its own exit code.
    pub child_exit_code: Option<i32>,

    /// The failure that stopped the event loop,
    ///  else `None`.
    ///
    /// What:
    ///      `pub fatal_error: Option<anyhow::Error>`.
    ///  Set by `Compositor::fail`,
    ///  taken by `run` after the loop stops.
    /// Why:
    ///       Event-source callbacks return `()`,
    ///  so a failure inside one (the nested window's frame,
    ///  a client that cannot be registered) reaches `main` through this slot and becomes an error exit,
    ///  not a panic.
    pub fatal_error: Option<anyhow::Error>,

    /// The active 60fps frame recorder,
    ///  if `record` is running.
    ///
    /// What:
    ///      `pub recorder: Option<crate::recorder::Recorder>`.
    ///  `Some` while recording.
    /// Why:
    ///       Holds the capture timer registration,
    ///  encoder pool,
    ///  and counters;
    ///  taken
    ///           out during each tick so the readback can borrow the rest of the state.
    pub recorder: Option<crate::recorder::Recorder>,

    /// A cloneable handle to the event loop,
    ///  for registering the recorder's timer.
    ///
    /// What:
    ///      `pub loop_handle: LoopHandle<'static, Compositor>`.
    ///  calloop's refcounted
    ///           handle;
    ///  storing it in the loop's own data is a supported calloop pattern.
    /// Why:
    ///       The `record` control command (which only has `&mut Compositor`) needs it to
    ///           insert the capture timer source.
    pub loop_handle: LoopHandle<'static, Compositor>,

    /// The `text/uri-list` bytes an in-flight `drop-file` drag will hand the app.
    ///
    /// What:
    ///      `pub pending_dnd_uri_list: Option<Vec<u8>>`.
    ///  `Some(bytes)` while a
    ///           compositor-originated drag is being driven toward the hosted app;
    ///  `None`
    ///           otherwise.
    ///  Sibling shapes:
    ///  a `String` would force UTF-8,
    ///  but the wire
    ///           format is raw bytes written to the client's receive fd.
    /// Why:
    ///       `ServerDndGrabHandler::send` (called when the app requests the drag data)
    ///           has only `&mut Compositor`,
    ///  so the payload must be reachable from state.
    pub pending_dnd_uri_list: Option<Vec<u8>>,

    /// Toplevel surfaces,
    ///  most recently focused first;
    ///  focus returns through this list when the focused toplevel goes away.
    pub focus_order: Vec<smithay::reexports::wayland_server::protocol::wl_surface::WlSurface>,

    /// The private accessibility bus services,
    ///  when `--accessibility` was given.
    pub a11y: Option<crate::a11y_service::A11yService>,

    /// The cursor the hosted client last asked for.
    pub cursor_image: smithay::input::pointer::CursorImageStatus,

    /// Keeps the `wp_cursor_shape_manager_v1` global alive.
    pub _cursor_shape: smithay::wayland::cursor_shape::CursorShapeManagerState,

    /// The announced key repeat pair as `(rate per second, delay in milliseconds)`.
    pub repeat_info: (i32, i32),
}

/// Owning bundle of the pieces the winit backend produces before the state exists.
///
/// What:
///      `pub struct BackendPieces { ... }`.
///  A small carrier so `run` can build
///           the backend,
///  output,
///  and dmabuf state,
///  then hand them to
///           `Compositor::new` as one argument instead of six positional ones.
/// Why:
///       Keeps `Compositor::new` to a single grouped parameter (clearer than a
///           long positional list) and matches the repo's named-parameter preference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BackendPieces = { backend; output; dmabufState; dmabufGlobal; dmabufFeedback; geometry };
/// ```
pub struct BackendPieces {
    /// The winit backend (window + GLES renderer) or the surfaceless renderer.
    pub backend: RenderBackend,
    /// The nested output.
    pub output: Output,
    /// dmabuf protocol state.
    pub dmabuf_state: DmabufState,
    /// The dmabuf global to keep alive.
    pub dmabuf_global: DmabufGlobal,
    /// The dmabuf v4 feedback,
    ///  or `None` for v3.
    pub dmabuf_feedback: Option<DmabufFeedback>,
    /// The starting logical size and scale the window and output were built for.
    pub geometry: ScreenGeometry,
}

/// Constructors and helpers for the compositor state.
///
/// What:
///      `impl Compositor { ... }`.
///  The inherent method block:
///  construction,
///  the
///           Wayland listener setup,
///  and surface hit-testing.
/// Why:
///       Group the state's own (non-trait) behaviour.
impl Compositor {
    /// Build the full compositor state from an event loop,
    ///  a display,
    ///  and the
    /// already-initialised winit/dmabuf backend pieces.
    ///
    /// What:
    ///      `pub fn new(event_loop: &mut EventLoop<Compositor>, display:
    ///           Display<Compositor>, pieces: BackendPieces) -> Result<Self>`.
    ///  Takes a mutable
    ///           borrow of the loop (to register sources and read its stop signal),
    ///           consumes the `display` by value (it is moved into an event source),
    ///           and consumes the backend pieces.
    ///  Returns a fully built `Self`.
    /// Why:
    ///       One place that wires every protocol global and the client-listening
    ///           socket,
    ///  then packages it all into the state value the loop carries.
    ///
    /// Errors:
    ///       The default keymap cannot be compiled,
    ///  or the Wayland listening socket cannot be created
    ///  (`XDG_RUNTIME_DIR` unset,
    ///  missing,
    ///  or not writable).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static new(eventLoop, display, pieces): Compositor { ... } // throws on setup failure
    /// ```
    ///
    /// @example
    /// ```ts
    /// const state = Compositor.new(eventLoop, display, pieces);
    /// ```
    pub fn new(
        event_loop: &mut EventLoop<'static, Compositor>,
        display: Display<Compositor>,
        pieces: BackendPieces,
    ) -> Result<Self> {
        // What:     `let start_time = std::time::Instant::now();`. Reads the monotonic
        //           clock now.
        // Why:      Baseline for frame-callback timestamps.
        let start_time = std::time::Instant::now();

        // What:     `let dh = display.handle();`. `.handle()` returns a cheap,
        //           cloneable `DisplayHandle` without consuming the display.
        // Why:      Global constructors below need the handle; the display itself is
        //           moved into the client-dispatch source afterwards.
        let dh = display.handle();

        // What:     `CompositorState::new::<Self>(&dh)`. The `::<Self>` turbofish tells
        //           the constructor which state type implements the handler trait;
        //           `&dh` lends the display handle read-only.
        // Why:      Registers the `wl_compositor` global.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const compositorState = CompositorState.new(dh);
        // ```
        let compositor_state = CompositorState::new::<Self>(&dh);

        // What:     `XdgShellState::new::<Self>(&dh)`. Registers `xdg_wm_base`.
        // Why:      The hosted app creates its window through xdg-shell.
        let xdg_shell_state = XdgShellState::new::<Self>(&dh);

        // What:     `ShmState::new::<Self>(&dh, vec![])`. `vec![]` is an empty `Vec` of
        //           extra SHM formats (the mandatory ARGB/XRGB are always added).
        // Why:      Advertise `wl_shm` even though the GPU app uses dmabuf.
        let shm_state = ShmState::new::<Self>(&dh, vec![]);

        // What:     `OutputManagerState::new_with_xdg_output::<Self>(&dh)`. Registers
        //           `wl_output` plus the `xdg-output` extension.
        // Why:      Report the nested screen's logical geometry to the client.
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&dh);

        // What:     `let mut seat_state = SeatState::new();`. Mutable because
        //           `new_wl_seat` below borrows it mutably to create the seat.
        // Why:      Owns the collection of seats (we create exactly one).
        let mut seat_state = SeatState::new();

        // What:     `DataDeviceState::new::<Self>(&dh)`. Registers `wl_data_device_manager`.
        // Why:      Clipboard / DnD plumbing focus handling expects.
        let data_device_state = DataDeviceState::new::<Self>(&dh);
        // Lend the nested display handle; clipboard protocols use its existing seat storage.
        let clipboard = ClipboardProtocols::new(&dh);
        // Register the fractional-scale and viewporter globals with the starting geometry.
        let screen = ScreenState::new(&dh, pieces.geometry);
        let cursor_shape = smithay::wayland::cursor_shape::CursorShapeManagerState::new::<Self>(&dh);

        // What:     `PopupManager::default()`. Builds an empty popup tracker.
        // Why:      Ready to track any popups the app opens.
        let popups = PopupManager::default();

        // What:     `let mut seat: Seat<Self> = seat_state.new_wl_seat(&dh, "winit");`.
        //           Creates a named seat and its `wl_seat` global. `mut` so we can add
        //           input capabilities to it next.
        // Why:      The single seat all synthetic input is routed through.
        let mut seat: Seat<Self> = seat_state.new_wl_seat(&dh, "winit");

        // What:     `seat.add_keyboard(Default::default(), 200, 25).context(...)?;`.
        //           `Default::default()` supplies a default `XkbConfig` (US layout);
        //           `200, 25` are the key repeat delay (ms) and rate (keys/s).
        //           `.context(...)?` returns the keymap compilation failure to `run`.
        // Why:      Advertise a keyboard so the app accepts synthetic key input.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // seat.addKeyboard(defaultXkbConfig, 200, 25); // throws on keymap failure
        // ```
        seat.add_keyboard(Default::default(), 200, 25).context(
            "compiling the default US keyboard layout for the nested seat failed; \
             check that the xkeyboard-config package (the xkb data files) is installed",
        )?;

        // What:     `seat.add_pointer();`. Adds a pointer capability, returning a
        //           `PointerHandle` we ignore here (fetched later via `get_pointer`).
        // Why:      Advertise a pointer so the app accepts synthetic clicks.
        seat.add_pointer();

        // What:     `let mut space = Space::default();`. Empty plane. `mut` because we
        //           map the output into it next.
        // Why:      Holds the output and (later) the one window.
        let mut space = Space::default();

        // What:     `space.map_output(&pieces.output, (0, 0));`. Places the output at
        //           origin `(0, 0)` on the plane. `(0, 0)` is an `(i32, i32)` tuple
        //           coerced into Smithay's `Point`.
        // Why:      The window is rendered relative to this output's position.
        space.map_output(&pieces.output, (0, 0));

        // What:     `let damage_tracker = OutputDamageTracker::from_output(&pieces.output);`.
        //           Builds a damage tracker sized to the output.
        // Why:      `render_output` needs it to compute per-frame damage.
        let damage_tracker = OutputDamageTracker::from_output(&pieces.output);

        // What:     `let socket_name = Self::init_wayland_listener(display, event_loop);`.
        //           Consumes the display (moves it into the client-dispatch source) and
        //           returns the chosen socket name.
        // Why:      Start accepting Wayland client connections; capture the name for the
        //           child's `WAYLAND_DISPLAY`.
        let socket_name = Self::init_wayland_listener(display, event_loop)?;

        // What:     `let loop_signal = event_loop.get_signal();`. A handle to stop the
        //           loop later.
        // Why:      Stored so child-exit / close handling can end the program.
        let loop_signal = event_loop.get_signal();

        // What:     `let loop_handle = event_loop.handle();`. A cloneable registration
        //           handle for the loop.
        // Why:      Stored so the recorder can insert its capture timer from the control
        //           handler (which only receives `&mut Compositor`).
        let loop_handle = event_loop.handle();

        // What:     `Self { ... }`. Construct and return the state (tail expression).
        //           Shorthand fields reuse local names; the dmabuf pieces are unpacked
        //           from `pieces`.
        // Why:      Package every wired-up piece into the loop's shared data.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { startTime, socketName, displayHandle: dh, space, /* ... */ };
        // ```
        return Ok(Self {
            start_time,
            socket_name,
            display_handle: dh,
            space,
            loop_signal,
            compositor_state,
            xdg_shell_state,
            shm_state,
            output_manager_state,
            seat_state,
            data_device_state,
            clipboard,
            popups,
            seat,
            output: pieces.output,
            screen,
            backend: pieces.backend,
            capture_texture: None,
            damage_tracker,
            dmabuf_state: pieces.dmabuf_state,
            _dmabuf_global: pieces.dmabuf_global,
            _dmabuf_feedback: pieces.dmabuf_feedback,
            child: None,
            shutdown_deadline: None,
            appearance_portal: None,
            notifications: None,
            private_bus: None,
            launcher: None,
            held_pointer_buttons: HeldButtons::default(),
            frame_pacing: FramePacing::new(start_time),
            child_exit_code: None,
            fatal_error: None,
            recorder: None,
            loop_handle,
            pending_dnd_uri_list: None,
            focus_order: Vec::new(),
            a11y: None,
            cursor_image: smithay::input::pointer::CursorImageStatus::default_named(),
            _cursor_shape: cursor_shape,
            repeat_info: (DEFAULT_REPEAT_RATE, DEFAULT_REPEAT_DELAY_MS),
        })
    }

    /// Record the failure that ends the session and stop the event loop.
    ///
    /// What:
    ///      `pub fn fail(&mut self, error: anyhow::Error)`.
    ///  Keeps the first error given,
    ///  drops later ones,
    ///  and signals the loop to stop.
    /// Why:
    ///       Event-source callbacks cannot return an error,
    ///  and the first failure is the cause of any that follow it.
    pub fn fail(&mut self, error: anyhow::Error) {
        // What:     Keep the error when no earlier failure is stored,
        //           otherwise log it as a later consequence.
        // Why:      `run` returns the first failure,
        //           and a later one must not vanish without a trace.
        if self.fatal_error.is_none() {
            self.fatal_error = Some(error);
        } else {
            tracing::warn!("a later failure after the session was already stopping: {error:#}");
        }
        self.loop_signal.stop();
    }

    /// Create the client-listening socket and register both it and the display with
    /// the event loop,
    ///  returning the socket name.
    ///
    /// What:
    ///      `fn init_wayland_listener(display: Display<Compositor>, event_loop:
    ///           &mut EventLoop<Compositor>) -> Result<OsString>`.
    ///  Private helper;
    ///  consumes the
    ///           display (moves it into a loop source),
    ///  returns the chosen socket name.
    /// Why:
    ///       Wayland needs two event sources:
    ///  one accepting new client connections,
    ///           one dispatching existing clients' requests.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function initWaylandListener(display, eventLoop): string { ... }
    /// ```
    fn init_wayland_listener(
        display: Display<Compositor>,
        event_loop: &mut EventLoop<Compositor>,
    ) -> Result<OsString> {
        // What:     `let source = ListeningSocketSource::new_auto().context(...)?;`.
        //           Creates a listening Unix socket, auto-picking the next free
        //           `wayland-N` name in `$XDG_RUNTIME_DIR`. `.context(...)?` returns the
        //           failure (no runtime dir) to `run`.
        // Why:      This is the socket the hosted child connects to.
        let source = ListeningSocketSource::new_auto().context(
            "creating the Wayland listening socket wayland-N in XDG_RUNTIME_DIR failed; \
             set XDG_RUNTIME_DIR to an existing directory you can write to, such as /run/user/<your user id>",
        )?;

        // What:     `let socket_name = source.socket_name().to_os_string();`.
        //           `.socket_name()` borrows the name as `&OsStr`; `.to_os_string()`
        //           copies it into an owned `OsString`.
        // Why:      Return an owned name that outlives the source.
        let socket_name = source.socket_name().to_os_string();

        // What:     `let handle = event_loop.handle();`. A cloneable handle used to
        //           register sources.
        // Why:      Needed to insert both sources below.
        let handle = event_loop.handle();

        // What:     `handle.insert_source(source, move |stream, _, state| { ... })`.
        //           Registers the listening socket; the `move` closure takes ownership
        //           of nothing here but captures by move per calloop's requirement. On
        //           each new connection it receives the client `stream`, the source
        //           metadata (`_`, ignored), and `&mut Compositor` (`state`).
        // Why:      Insert every arriving client into the display with its per-client data.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // handle.insertSource(source, (stream, _, state) => {
        //   state.displayHandle.insertClient(stream, new ClientState());
        // });
        // ```
        handle
            .insert_source(source, move |stream, _, state: &mut Compositor| {
                // What:     `state.display_handle.insert_client(stream,
                //           Arc::new(ClientState::default())).unwrap();`. Registers the
                //           new client. `Arc::new(...)` puts the per-client data on the
                //           heap behind a shared, thread-safe pointer (wayland-server
                //           requires `Arc`, not `Box`, because it may access the data
                //           from its own threads). A failure goes to `fail`, which ends the
                //           session with an error.
                // Why:      Give the client a place to keep its compositor-side state.
                let inserted = state
                    .display_handle
                    .insert_client(stream, Arc::new(ClientState::default()));
                if let Err(error) = inserted {
                    state.fail(
                        anyhow::Error::new(error)
                            .context("registering a connecting Wayland client with the nested display failed"),
                    );
                }
            })
            .expect("failed to register the Wayland listening socket");

        // What:     `handle.insert_source(Generic::new(display, Interest::READ,
        //           Mode::Level), |_, display, state| { ... })`. Wraps the display's
        //           file descriptor as a generic readable source. On readability it
        //           dispatches queued client requests.
        // Why:      This is what actually processes client protocol messages.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // handle.insertSource(new Generic(display, READ, LEVEL), (_, display, state) => {
        //   display.dispatchClients(state);
        //   return Continue;
        // });
        // ```
        handle
            .insert_source(
                Generic::new(display, Interest::READ, Mode::Level),
                |_, pending_display, state: &mut Compositor| {
                    // What:     `unsafe { display.get_mut().dispatch_clients(state)...?; }`.
                    //           `unsafe` marks a block whose safety the compiler cannot
                    //           check: here it is sound because we never drop the display
                    //           while dispatching. `get_mut` yields the inner `Display`;
                    //           `dispatch_clients` runs queued requests against `state`.
                    // Why:      Advance every connected client's protocol state.
                    // Gotcha:   `unsafe` in Rust does not mean "wrong"; it means the
                    //           caller vouches for an invariant (display stays alive).
                    // A failure ends the event loop; `run` reports it as "event loop failed".
                    unsafe {
                        pending_display.get_mut().dispatch_clients(state).map_err(|error| {
                            return std::io::Error::new(
                                error.kind(),
                                format!("dispatching Wayland client requests failed: {error}"),
                            )
                        })?;
                    }
                    // What:     `Ok(PostAction::Continue)`. Tell calloop the source is
                    //           healthy and should keep listening. Tail expression.
                    // Why:      Keep dispatching on future readiness.
                    return Ok(PostAction::Continue)
                },
            )
            .expect("failed to register the Wayland display source");

        // What:     `Ok(socket_name)`. Tail expression returning the owned name.
        // Why:      Hand the chosen socket name back to `new`.
        return Ok(socket_name)
    }

    /// Find the surface (and its position) under a point on the plane,
    ///  if any.
    ///
    /// What:
    ///      `pub fn surface_under(&self, pos: Point<f64, Logical>) ->
    ///           Option<(WlSurface, Point<f64, Logical>)>`.
    ///  Borrows self read-only,
    ///           takes a floating-point logical point,
    ///  returns the topmost surface
    ///           there plus its origin,
    ///  or `None`.
    /// Why:
    ///       Pointer input needs to know which surface a click lands on so it can
    ///           set that surface as the pointer focus.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// surfaceUnder(pos): [WlSurface, Point] | undefined { ... }
    /// ```
    pub fn surface_under(
        &self,
        pos: Point<f64, Logical>,
    ) -> Option<(smithay::reexports::wayland_server::protocol::wl_surface::WlSurface, Point<f64, Logical>)>
    {
        // What:     `self.space.element_under(pos).and_then(|(window, location)| { ... })`.
        //           `element_under` returns `Option<(&Window, Point<i32, Logical>)>`.
        //           `.and_then(closure)` runs the closure only when present, and the
        //           closure itself returns an `Option`, flattening the result. The
        //           closure destructures the tuple into `window` and `location`.
        // Why:      Delegate hit-testing to the window's own surface tree.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const hit = space.elementUnder(pos);
        // if (!hit) return undefined;
        // const [window, location] = hit;
        // ...
        // ```
        return self.space.element_under(pos).and_then(|(window, location)| {
            // What:     `window.surface_under(pos - location.to_f64(),
            //           WindowSurfaceType::ALL).map(|(s, p)| (s, (p + location).to_f64()))`.
            //           `location.to_f64()` converts the integer point to floating so it
            //           can be subtracted from `pos`. `surface_under` returns the surface
            //           under that window-local point. `.map(closure)` rewrites the
            //           found `(surface, point)` back into plane coordinates by adding
            //           `location`.
            // Why:      Translate between plane and window-local coordinates for the hit.
            return window
                .surface_under(
                    pos - location.to_f64(),
                    smithay::desktop::WindowSurfaceType::ALL,
                )
                .map(|(surface, point)| return (surface, (point + location).to_f64()))
        })
    }
}

/// Per-client compositor-side state stored behind an `Arc` for each Wayland client.
///
/// What:
///      `#[derive(Default)] pub struct ClientState { pub compositor_state:
///           CompositorClientState }`.
///  `#[derive(Default)]` auto-generates a
///           `default()` constructor.
///  Holds the compositor's per-client bookkeeping.
/// Why:
///       wayland-server hands this back whenever it needs client-scoped data (for
///           example,
///  which `CompositorClientState` a surface belongs to).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ClientState { compositorState = new CompositorClientState(); }
/// ```
#[derive(Default)]
pub struct ClientState {
    /// The compositor protocol's per-client sub-state.
    pub compositor_state: CompositorClientState,
}

/// Implement the wayland-server `ClientData` hooks for our per-client state.
///
/// What:
///      `impl ClientData for ClientState { ... }`.
///  `ClientData` is the trait
///           wayland-server calls on connect/disconnect.
///  Both hooks are empty here.
/// Why:
///       We do not need to react to client lifecycle beyond default behaviour,
///  but
///           the trait must be implemented for `insert_client` to accept the type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // implements ClientData { initialized() {} disconnected() {} }
/// ```
impl ClientData for ClientState {
    /// What:
    ///      `fn initialized(&self, _client_id: ClientId) {}`.
    ///  Called once the
    ///           client is registered;
    ///  the id is ignored (underscore prefix).
    /// Why:
    ///       Nothing to do on connect.
    fn initialized(&self, _client_id: ClientId) {}

    /// What:
    ///      `fn disconnected(&self, _client_id: ClientId, _reason:
    ///           DisconnectReason) {}`.
    ///  Called when the client goes away.
    /// Why:
    ///       Nothing to do on disconnect;
    ///  the exit-poll timer handles shutdown.
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}
