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
//!
//! The methods the state itself answers for live in `the_compositors_methods.rs`,
//!  and each client's state with the two callbacks that keep it in `wayland_clients.rs`.

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
///  the shared owner keeps the private bus alive for as long as the session.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // OsString ~ string; Child ~ a Node ChildProcess handle; Arc<T> ~ a shared ref.
/// ```
use std::{
    ffi::OsString,
    process::Child,
    sync::Arc,
};

/// Clipboard management globals are owned by this nested display,
///  never the host.
use crate::handler::clipboard::ClipboardProtocols;

/// The private Settings portal is owned by this session,
///  never shared with the host.
use crate::appearance_portal::AppearancePortal;

/// The private session bus and the taskbar progress listener that reads it.
use crate::{
    input_pointer::HeldButtons,
    launcher_listen::LauncherLog,
    plasma_services::PlasmaServices,
    private_bus::PrivateBus,
};

/// Stall bookkeeping that keeps the hosted client drawing while the parent is silent.
use crate::frame_pacing::FramePacing;

/// The logical screen size and output scale,
///  and the globals that tell clients the scale.
use crate::{
    render_backend::RenderBackend,
    screen::ScreenState,
    screen_geometry::ScreenGeometry,
};

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
    backend::renderer::{
        damage::OutputDamageTracker,
        gles::GlesTexture,
    },
    desktop::{
        PopupManager,
        Space,
        Window,
    },
    input::{
        Seat,
        SeatState,
    },
    output::Output,
    reexports::{
        calloop::{
            LoopHandle,
            LoopSignal,
        },
        wayland_server::DisplayHandle,
    },
    wayland::{
        compositor::CompositorState,
        dmabuf::{
            DmabufFeedback,
            DmabufGlobal,
            DmabufState,
        },
        output::OutputManagerState,
        selection::data_device::DataDeviceState,
        shell::xdg::XdgShellState,
        shm::ShmState,
    },
};

/// The methods the state itself answers for.
mod the_compositors_methods;

/// The state each connected client keeps.
mod wayland_clients;

/// The per-client state wayland-server hands back,
///  part of this module's interface.
pub use wayland_clients::ClientState;

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
