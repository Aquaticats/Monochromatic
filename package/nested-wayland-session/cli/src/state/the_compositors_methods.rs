//! The methods the compositor state answers for:
//!  building it from the event loop,
//!  the display and the backend pieces,
//!  recording the failure that ends the session,
//!  and finding the surface under a point.
//!
//! Split out of `state.rs` to keep that file under the code-line limit.

/// The Wayland socket name the constructor hands back.
use std::ffi::OsString;

/// Context turns a failed startup step into an error that names the step,
///  so the process reports it and exits nonzero instead of panicking.
use anyhow::{
    Context,
    Result,
};

/// Stall bookkeeping that keeps the hosted client drawing while the parent is silent.
use crate::frame_pacing::FramePacing;
/// Clipboard management globals are owned by this nested display,
///  never the host.
use crate::handler::clipboard::ClipboardProtocols;
/// The held-button bookkeeping the seat starts with.
use crate::input_pointer::HeldButtons;
/// The logical screen size and output scale the output starts at.
use crate::screen::ScreenState;

/// What:
///      A grouped `use` of the Smithay items the constructor and the hit test name.
///  Each path names a type used below;
///  the braces just avoid repeating the common `smithay::...` prefix.
/// Why:
///       Bring the compositor building blocks into scope.
use smithay::{
    backend::renderer::damage::OutputDamageTracker,
    desktop::{
        PopupManager,
        Space,
    },
    input::{
        Seat,
        SeatState,
    },
    reexports::{
        calloop::{
            EventLoop,
            Interest,
            Mode,
            generic::Generic,
        },
        wayland_server::Display,
    },
    utils::{
        Logical,
        Point,
    },
    wayland::{
        compositor::CompositorState,
        output::OutputManagerState,
        selection::data_device::DataDeviceState,
        shell::xdg::XdgShellState,
        shm::ShmState,
        socket::ListeningSocketSource,
    },
};

/// The state the constructor fills in,
///  the backend pieces it is handed,
///  and the repeat values it announces.
use super::{
    BackendPieces,
    Compositor,
    DEFAULT_REPEAT_DELAY_MS,
    DEFAULT_REPEAT_RATE,
};

/// The two event-loop callbacks the listening socket and the display source take.
use super::wayland_clients::{
    dispatch_display_clients,
    on_new_client,
};
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
        let cursor_shape =
            smithay::wayland::cursor_shape::CursorShapeManagerState::new::<Self>(&dh);

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
        });
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
    pub fn fail(
        &mut self,
        error: anyhow::Error,
    ) {
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
            .insert_source(source, on_new_client)
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
                dispatch_display_clients,
            )
            .expect("failed to register the Wayland display source");

        // What:     `Ok(socket_name)`. Tail expression returning the owned name.
        // Why:      Hand the chosen socket name back to `new`.
        return Ok(socket_name);
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
    ) -> Option<(
        smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
        Point<f64, Logical>,
    )> {
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
        let (window, location) = self.space.element_under(pos)?;
        // What:     `window.surface_under(pos - location.to_f64(), WindowSurfaceType::ALL)?`.
        //           `location.to_f64()` converts the integer point to floating so it
        //           can be subtracted from `pos`. `surface_under` returns the surface
        //           under that window-local point; `?` returns `None` when there is none.
        // Why:      Translate between plane and window-local coordinates for the hit.
        let (surface, point) = window.surface_under(
            pos - location.to_f64(),
            smithay::desktop::WindowSurfaceType::ALL,
        )?;
        return Some((surface, (point + location).to_f64()));
    }
}
