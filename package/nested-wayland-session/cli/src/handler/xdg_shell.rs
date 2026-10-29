//! xdg-shell handler:
//!  map the one hosted toplevel fullscreen and focus it.
//!
//! A general compositor lets windows float,
//!  move,
//!  and resize.
//!  This fixture hosts
//! exactly one app and keeps it filling the whole nested screen,
//!  so `new_toplevel`
//! configures the window fullscreen and gives it keyboard focus,
//!  and the
//! move/resize/grab requests are deliberately ignored.

/// What:
///      Grouped `use` of the xdg-shell types,
///  popup helpers,
///  the toplevel state
///           enum,
///  focus/serial utilities,
///  and the delegate macro.
/// Why:
///       Everything the handler and the two free functions below reference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { XdgShellHandler, ToplevelSurface, ... } from "smithay";
/// ```
use smithay::{
    delegate_xdg_shell,
    desktop::{
        find_popup_root_surface, get_popup_toplevel_coords, PopupKind, PopupManager, Space, Window,
    },
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::protocol::{wl_seat, wl_surface::WlSurface},
    },
    utils::Serial,
    wayland::{
        compositor::with_states,
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
            XdgToplevelSurfaceData,
        },
    },
};

/// What:
///      `use crate::state::Compositor;`.
///  Our state type.
/// Why:
///       The handler is `impl XdgShellHandler for Compositor`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Compositor } from "../state";
/// ```
use crate::state::Compositor;

/// The logical screen size the toplevel is configured with.
use crate::screen_geometry::ScreenGeometry;

/// Configure one toplevel to fill the screen:
///  set fullscreen + activated + the
/// logical screen size as its pending size.
///
/// What:
///      `fn set_fullscreen(surface: &ToplevelSurface, geometry: ScreenGeometry)`.
///           Private helper borrowing the toplevel;
///  the geometry is a small copied value.
/// Why:
///       Both the initial map and every resize need the same "fill the screen"
///           configuration,
///  so it lives in one place.
///  The size is the logical one:
///  the
///           output mode is physical,
///  which at scale 2 would configure the client twice too
///           large and put every logical click in the wrong place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function setFullscreen(surface, geometry) { ... }
/// ```
fn set_fullscreen(surface: &ToplevelSurface, geometry: ScreenGeometry) {
    // What:     `surface.with_pending_state(|state| { ... });`. Runs the closure with a
    //           mutable borrow of the toplevel's not-yet-sent configure state.
    // Why:      Stage the fullscreen flags and size so the next `send_configure` carries
    //           them to the client.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // surface.withPendingState((state) => { ... });
    // ```
    surface.with_pending_state(|state| {
        // What:     `state.states.set(xdg_toplevel::State::Fullscreen);`. Adds the
        //           fullscreen flag to the toplevel's state set.
        // Why:      Tell the client it is fullscreen so it draws edge-to-edge.
        state.states.set(xdg_toplevel::State::Fullscreen);

        // What:     The activated flag is not set here; `crate::focus` sets and clears it
        //           for the window that has keyboard focus.
        // Why:      Only the focused window is active, and a dialog takes that from its parent.

        // What:     `state.size = Some((w, h).into());`. Sets the pending size. The
        //           `(i32, i32)` tuple `.into()` converts into `Size<i32, Logical>` (the
        //           type `state.size` expects). `Some(...)` marks the size as specified.
        // Why:      Ask the client for the logical screen size; the client multiplies
        //           it by the fractional scale to size its buffer.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // state.size = { w: geometry.logicalWidth, h: geometry.logicalHeight };
        // ```
        state.size = Some((geometry.logical_width, geometry.logical_height).into());
    });
}

/// Reconfigure every mapped toplevel to fill the (possibly resized) output.
///
/// What:
///      `pub fn reconfigure_fullscreen(state: &mut Compositor)`.
///  Iterates the
///           space's windows,
///  re-applies `set_fullscreen`,
///  and sends a fresh configure.
/// Why:
///       When the nested screen is resized,
///  the hosted app must be told the new
///           size so it redraws to fill it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reconfigureFullscreen(state) { ... }
/// ```
pub fn reconfigure_fullscreen(state: &mut Compositor) {
    // What:     `let toplevels: Vec<ToplevelSurface> = state.space.elements()
    //           .filter_map(|w| w.toplevel().cloned()).collect();`. `filter_map` keeps
    //           only windows that have a toplevel, cloning each; `.collect()` gathers
    //           them into an owned `Vec`.
    // Why:      Collect first so we no longer borrow `state.space` while sending
    //           configures, avoiding an overlapping-borrow error.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const toplevels = [...state.space.elements()].map((w) => w.toplevel()).filter(Boolean);
    // ```
    let toplevels: Vec<ToplevelSurface> = state
        .space
        .elements()
        .filter_map(|w| return w.toplevel().cloned())
        .collect();

    // What:     `for toplevel in &toplevels { ... }`. Borrow each collected toplevel in
    //           turn (`&toplevels` iterates references).
    // Why:      Reconfigure and notify each window.
    for toplevel in &toplevels {
        // What:     `set_fullscreen(toplevel, state.screen.geometry);`. Stage the new size.
        // Why:      Update the pending configure to the new logical screen size.
        set_fullscreen(toplevel, state.screen.geometry);

        // What:     `toplevel.send_configure();`. Send the staged configure to the client.
        // Why:      Deliver the new size so the app redraws.
        toplevel.send_configure();
    }
}

/// Implement the xdg-shell handler.
///
/// What:
///      `impl XdgShellHandler for Compositor`.
///  Provides the shell state accessor
///           and the toplevel/popup lifecycle hooks.
/// Why:
///       This is where a new app window becomes a fullscreen,
///  focused window.
impl XdgShellHandler for Compositor {
    /// What:
    ///      `fn xdg_shell_state(&mut self) -> &mut XdgShellState`.
    ///  Mutable accessor.
    /// Why:
    ///       Smithay mutates shell bookkeeping through it.
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        // What:     `&mut self.xdg_shell_state`. Mutable borrow (tail expression).
        // Why:      Return the shell state.
        return &mut self.xdg_shell_state
    }

    /// What:
    ///      `fn new_toplevel(&mut self, surface: ToplevelSurface)`.
    ///  Called when the
    ///           app creates its main window.
    ///  Consumes the `surface` by value.
    /// Why:
    ///       Configure it fullscreen,
    ///  map it at the origin,
    ///  and focus it.
    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        // What:     `let wl_surface = surface.wl_surface().clone();`. Grab an owned clone
        //           of the underlying `wl_surface` before `surface` is moved into the
        //           window below.
        // Why:      Keyboard focus is set on the `wl_surface`, which we still need after
        //           `surface` is consumed.
        let wl_surface = surface.wl_surface().clone();

        // What:     `set_fullscreen(&surface, self.screen.geometry);`. Stage the fullscreen
        //           configure on the new toplevel.
        // Why:      The first configure (sent on first commit) will carry the logical size.
        set_fullscreen(&surface, self.screen.geometry);

        // What:     `let window = Window::new_wayland_window(surface);`. Wrap the
        //           toplevel in a desktop `Window`, consuming `surface`.
        // Why:      The `Space` maps and renders `Window`s, not raw toplevels.
        let window = Window::new_wayland_window(surface);

        // What:     `self.space.map_element(window, (0, 0), true);`. Place the window at
        //           the origin `(0, 0)`; the `true` activates it.
        // Why:      Put the one window on screen, filling the output from the top-left.
        self.space.map_element(window, (0, 0), true);

        // What:     `crate::focus::focus_surface(self, wl_surface);`. Make the new toplevel the
        //           most recently focused one, give it keyboard focus, and mark it activated.
        // Why:      Synthetic key input reaches the newest window, as on a desktop where a new
        //           window takes focus, and the focus order remembers what to return to.
        crate::focus::focus_surface(self, wl_surface);
    }

    /// What:
    ///      `fn toplevel_destroyed(&mut self, surface: ToplevelSurface)`.
    ///  Called when a toplevel is destroyed or its client disconnects.
    /// Why:
    ///       Keyboard focus returns to the most recently focused remaining window,
    ///  as a desktop does when a dialog closes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toplevelDestroyed(surface: ToplevelSurface): void { focus.toplevelGone(this, surface.wlSurface); }
    /// ```
    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        crate::focus::toplevel_gone(self, surface.wl_surface());
    }

    /// What:
    ///      `fn new_popup(&mut self, surface: PopupSurface, _positioner:
    ///           PositionerState)`.
    ///  Called when the app opens a popup (menu/tooltip).
    /// Why:
    ///       Track and position it so it renders correctly.
    fn new_popup(&mut self, surface: PopupSurface, _positioner: PositionerState) {
        // What:     `self.unconstrain_popup(&surface);`. Adjust the popup so it stays on
        //           screen relative to its parent.
        // Why:      A popup positioned off the edge would be clipped.
        self.unconstrain_popup(&surface);

        // What:     `let _ = self.popups.track_popup(PopupKind::Xdg(surface));`.
        //           `track_popup` returns a `Result`; `let _ =` discards it (a popup we
        //           fail to track simply is not managed). `PopupKind::Xdg(surface)` wraps
        //           the popup in the popup-kind enum.
        // Why:      Register the popup so `handle_commit` configures it.
        let _ = self.popups.track_popup(PopupKind::Xdg(surface));
    }

    /// What:
    ///      `fn reposition_request(&mut self, surface: PopupSurface, positioner:
    ///           PositionerState, token: u32)`.
    ///  The app asks to move an existing popup.
    /// Why:
    ///       Recompute and acknowledge the new popup geometry.
    fn reposition_request(&mut self, surface: PopupSurface, positioner: PositionerState, token: u32) {
        // What:     `surface.with_pending_state(|state| { state.geometry =
        //           positioner.get_geometry(); state.positioner = positioner; });`. Stage
        //           the new geometry and positioner on the popup.
        // Why:      Prepare the popup's new placement before re-sending it.
        surface.with_pending_state(|state| {
            state.geometry = positioner.get_geometry();
            state.positioner = positioner;
        });

        // What:     `self.unconstrain_popup(&surface);`. Re-clamp onto the screen.
        // Why:      The requested position might fall off the edge.
        self.unconstrain_popup(&surface);

        // What:     `surface.send_repositioned(token);`. Acknowledge the reposition with
        //           the client's token.
        // Why:      The protocol requires echoing the token so the client matches it up.
        surface.send_repositioned(token);
    }

    /// What:
    ///      `fn move_request(&mut self, _surface: ToplevelSurface, _seat:
    ///           wl_seat::WlSeat, _serial: Serial) {}`.
    ///  The app asks to be dragged.
    /// Why:
    ///       A single fullscreen fixture window never moves;
    ///  ignore the request.
    fn move_request(&mut self, _surface: ToplevelSurface, _seat: wl_seat::WlSeat, _serial: Serial) {}

    /// What:
    ///      `fn resize_request(&mut self, _surface: ToplevelSurface, _seat:
    ///           wl_seat::WlSeat, _serial: Serial, _edges: xdg_toplevel::ResizeEdge) {}`.
    ///           The app asks to be interactively resized.
    /// Why:
    ///       The fixture controls the size (via the control API),
    ///  not the client;
    ///           ignore.
    fn resize_request(
        &mut self,
        _surface: ToplevelSurface,
        _seat: wl_seat::WlSeat,
        _serial: Serial,
        _edges: xdg_toplevel::ResizeEdge,
    ) {
    }

    /// What:
    ///      `fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat,
    ///           _serial: Serial) {}`.
    ///  The app requests a popup grab (keyboard capture).
    /// Why:
    ///       Not needed for scripted testing;
    ///  ignore.
    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {}
}

// What:     `delegate_xdg_shell!(Compositor);`. Generate the xdg-shell dispatch glue.
// Why:      Wire xdg_wm_base / xdg_toplevel / xdg_popup requests to the handler.
delegate_xdg_shell!(Compositor);

/// Send the initial configure for a toplevel or popup on its first commit.
///
/// What:
///      `pub fn handle_commit(popups: &mut PopupManager, space: &Space<Window>,
///           surface: &WlSurface)`.
///  Free function called from the compositor commit
///           hook.
///  Mutably borrows the popup manager,
///  read-only borrows the space and
///           surface.
/// Why:
///       A client cannot draw until it receives its first configure;
///  this sends it
///           exactly once for whichever kind of surface committed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function handleCommit(popups, space, surface) { ... }
/// ```
pub fn handle_commit(popups: &mut PopupManager, space: &Space<Window>, surface: &WlSurface) {
    // What:     `if let Some(window) = space.elements().find(|w|
    //           w.toplevel().unwrap().wl_surface() == surface).cloned() { ... }`. Find the
    //           window whose toplevel surface is the one that committed, cloning it out of
    //           the borrow so we can inspect its state.
    // Why:      Toplevel-specific: send the first configure if not yet sent.
    if let Some(window) = space
        .elements()
        .find(|w| return w.toplevel().unwrap().wl_surface() == surface)
        .cloned()
    {
        // What:     `let initial_configure_sent = with_states(surface, |states| { states
        //           .data_map.get::<XdgToplevelSurfaceData>().unwrap().lock().expect(..)
        //           .initial_configure_sent });`. `with_states` runs the closure with the
        //           surface's stored state maps. `data_map.get::<T>()` fetches the
        //           per-surface xdg data; `.lock().expect(...)` locks its mutex (it fails only
        //           when another thread panicked while holding it); we read the boolean flag.
        // Why:      Decide whether the first configure still needs sending.
        let initial_configure_sent = with_states(surface, |states| {
            return states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .unwrap()
                .lock()
                .expect("XdgToplevelSurfaceData mutex is poisoned: a thread panicked while it held the lock")
                .initial_configure_sent
        });

        // What:     `if !initial_configure_sent { window.toplevel().unwrap()
        //           .send_configure(); }`. Send the staged (fullscreen) configure once.
        // Why:      Kick the client into drawing its first frame at the fullscreen size.
        if !initial_configure_sent {
            window.toplevel().unwrap().send_configure();
        }
    }

    // What:     `popups.commit(surface);`. Advance popup bookkeeping for this surface.
    // Why:      Keep popup state consistent on every commit.
    popups.commit(surface);

    // What:     `if let Some(popup) = popups.find_popup(surface) { ... }`. Look up whether
    //           the committed surface is a tracked popup.
    // Why:      Popups also need their initial configure.
    if let Some(popup) = popups.find_popup(surface) {
        // What:     `let PopupKind::Xdg(ref xdg) = popup else { return; };`. A `let ... else`
        //           binding: if `popup` is the `Xdg` variant, bind `xdg` by reference;
        //           otherwise (e.g. an input-method popup) return early.
        // Why:      Only xdg popups have the configure API used below.
        let PopupKind::Xdg(ref xdg) = popup else {
            return;
        };

        // What:     `if !xdg.is_initial_configure_sent() { xdg.send_configure()
        //           .expect("initial configure failed"); }`. Send the popup's first
        //           configure; `.expect(msg)` panics with `msg` if it errors (the initial
        //           configure is always allowed, so this should never fire).
        // Why:      Let the popup start drawing.
        if !xdg.is_initial_configure_sent() {
            xdg.send_configure().expect("initial configure failed");
        }
    }
}

/// Popup-unconstraining helper attached to the compositor state.
///
/// What:
///      `impl Compositor { fn unconstrain_popup(&self, popup: &PopupSurface) { ... } }`.
///           A second `impl` block adding one private method.
/// Why:
///       Keep popup geometry clamping beside the rest of the xdg-shell code.
impl Compositor {
    /// What:
    ///      `fn unconstrain_popup(&self, popup: &PopupSurface)`.
    ///  Clamp the popup so
    ///           it fits on the output relative to its parent window.
    /// Why:
    ///       Prevent popups from rendering off the edge of the nested screen.
    fn unconstrain_popup(&self, popup: &PopupSurface) {
        // What:     `let Ok(root) = find_popup_root_surface(&PopupKind::Xdg(popup.clone()))
        //           else { return; };`. `find_popup_root_surface` returns a `Result`; the
        //           `let Ok(root) = ... else { return; }` binds the success value or
        //           returns early on error.
        // Why:      We need the popup's root toplevel to position relative to it.
        let Ok(root) = find_popup_root_surface(&PopupKind::Xdg(popup.clone())) else {
            return;
        };

        // What:     `let Some(window) = self.space.elements().find(|w|
        //           w.toplevel().unwrap().wl_surface() == &root) else { return; };`. Find
        //           the window for that root surface, or bail if none.
        // Why:      Need the window's geometry to compute the popup's target box.
        let Some(window) = self
            .space
            .elements()
            .find(|w| return w.toplevel().unwrap().wl_surface() == &root)
        else {
            return;
        };

        // What:     `let output = self.space.outputs().next().unwrap();`. Take the first
        //           (only) mapped output.
        // Why:      The unconstraining box is the output's geometry.
        let output = self.space.outputs().next().unwrap();

        // What:     `let output_geo = self.space.output_geometry(output).unwrap();`. The
        //           output's rectangle on the plane.
        // Why:      Defines the region the popup must stay inside.
        let output_geo = self.space.output_geometry(output).unwrap();

        // What:     `let window_geo = self.space.element_geometry(window).unwrap();`. The
        //           window's rectangle on the plane.
        // Why:      The popup is positioned relative to its parent window.
        let window_geo = self.space.element_geometry(window).unwrap();

        // What:     `let mut target = output_geo;`. Copy the output rectangle to adjust.
        //           `Rectangle` is `Copy`, so this is a value copy, not a move.
        // Why:      Build the target box in the popup's parent-relative coordinate space.
        let mut target = output_geo;

        // What:     `target.loc -= get_popup_toplevel_coords(&PopupKind::Xdg(popup.clone()));`.
        //           Subtract the popup's offset within its toplevel from the box origin.
        // Why:      Convert the output box into coordinates relative to the popup's anchor.
        target.loc -= get_popup_toplevel_coords(&PopupKind::Xdg(popup.clone()));

        // What:     `target.loc -= window_geo.loc;`. Also subtract the window's position.
        // Why:      Finish converting into parent-window-relative coordinates.
        target.loc -= window_geo.loc;

        // What:     `popup.with_pending_state(|state| { state.geometry =
        //           state.positioner.get_unconstrained_geometry(target); });`. Ask the
        //           positioner for the best-fit geometry inside `target` and stage it.
        // Why:      Keep the popup fully on screen.
        popup.with_pending_state(|state| {
            state.geometry = state.positioner.get_unconstrained_geometry(target);
        });
    }
}
