//! wl_compositor and wl_shm handlers:
//!  surface commits and shared-memory buffers.

/// What:
///      Grouped `use` of the compositor/shm handler traits,
///  the buffer handler,
///           the on-commit helper,
///  and the delegate macros.
/// Why:
///       Bring in everything the two impls below reference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CompositorHandler, ShmHandler, ... } from "smithay";
/// ```
use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    delegate_compositor, delegate_shm,
    reexports::wayland_server::{
        protocol::{wl_buffer, wl_surface::WlSurface},
        Client,
    },
    wayland::{
        buffer::BufferHandler,
        compositor::{
            get_parent, is_sync_subsurface, CompositorClientState, CompositorHandler, CompositorState,
        },
        shm::{ShmHandler, ShmState},
    },
};

/// What:
///      `use crate::state::{ClientState, Compositor};`.
///  Pull in our state types.
/// Why:
///       The impls are for `Compositor`,
///  and one method reaches per-client state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ClientState, Compositor } from "../state";
/// ```
use crate::state::{ClientState, Compositor};

/// Implement the wl_compositor handler (surface lifecycle and commits).
///
/// What:
///      `impl CompositorHandler for Compositor`.
///  Provides the compositor state
///           accessors and the per-commit hook.
/// Why:
///       Every buffer the client attaches arrives through `commit`;
///  this is where
///           the fixture reacts to the app drawing a frame.
impl CompositorHandler for Compositor {
    /// What:
    ///      `fn compositor_state(&mut self) -> &mut CompositorState`.
    ///  Mutable
    ///           accessor for the compositor protocol state.
    /// Why:
    ///       Smithay mutates surface bookkeeping through it.
    fn compositor_state(&mut self) -> &mut CompositorState {
        // What:     `&mut self.compositor_state`. Mutable borrow (tail expression).
        // Why:      Return the state Smithay asked for.
        return &mut self.compositor_state
    }

    /// What:
    ///      `fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a
    ///           CompositorClientState`.
    ///  The `<'a>` is a lifetime parameter:
    ///  it says
    ///           the returned borrow lives exactly as long as the `client` borrow
    ///           passed in.
    ///  Reaches into the client's per-connection data.
    /// Why:
    ///       Smithay needs the compositor sub-state scoped to a specific client.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// clientCompositorState(client) { return client.getData().compositorState; }
    /// ```
    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        // What:     `&client.get_data::<ClientState>().unwrap().compositor_state`.
        //           `get_data::<ClientState>()` returns `Option<&ClientState>` (the
        //           data we inserted at connect); `.unwrap()` asserts it is present;
        //           then we borrow its `compositor_state` field.
        // Why:      Hand back this client's compositor sub-state.
        return &client.get_data::<ClientState>().unwrap().compositor_state
    }

    /// What:
    ///      `fn commit(&mut self, surface: &WlSurface)`.
    ///  Called every time a
    ///           client commits new surface content (a drawn frame or a state change).
    /// Why:
    ///       Drive buffer import,
    ///  send the initial configure,
    ///  and keep the
    ///           window's mapping fresh.
    fn commit(&mut self, surface: &WlSurface) {
        // What:     `on_commit_buffer_handler::<Self>(surface);`. Smithay helper that
        //           processes the just-attached buffer (including importing dmabuf).
        // Why:      Required so the renderer can later access the committed buffer.
        on_commit_buffer_handler::<Self>(surface);

        // What:     `if !is_sync_subsurface(surface) { ... }`. `is_sync_subsurface`
        //           is `true` for a synchronized child surface whose commit should be
        //           deferred to its parent; `!` negates it.
        // Why:      Only walk up to the root and refresh the window for real, top-level
        //           commits, not synchronized subsurface commits.
        if !is_sync_subsurface(surface) {
            // What:     `let mut root = surface.clone();`. Clone the surface handle (a
            //           cheap reference-counted clone) into a mutable local so we can
            //           climb to the root.
            // Why:      Find the toplevel that owns this (possibly nested) surface.
            let mut root = surface.clone();

            // What:     `while let Some(parent) = get_parent(&root) { root = parent; }`.
            //           `get_parent` returns `Option<WlSurface>`; the `while let` loops
            //           while a parent exists, walking upward.
            // Why:      Reach the toplevel surface at the top of the subsurface tree.
            while let Some(parent) = get_parent(&root) {
                root = parent;
            }

            // What:     `if let Some(window) = self.space.elements().find(|w|
            //           w.toplevel().unwrap().wl_surface() == &root) { window.on_commit(); }`.
            //           `space.elements()` iterates mapped windows; `.find(closure)`
            //           returns the first matching one as `Option<&Window>`. Inside,
            //           `w.toplevel().unwrap()` gets the window's toplevel (asserting it
            //           is a Wayland, not X, window) and compares its surface to `root`.
            // Why:      Tell the matching window that its content changed.
            if let Some(window) = self
                .space
                .elements()
                .find(|w| return w.toplevel().unwrap().wl_surface() == &root)
            {
                window.on_commit();
            }
        }

        // What:     `crate::handler::xdg_shell::handle_commit(&mut self.popups,
        //           &self.space, surface);`. Delegate the configure-on-first-commit and
        //           popup handling to the xdg-shell module.
        // Why:      Keep xdg-specific commit logic beside the rest of xdg-shell.
        crate::handler::xdg_shell::handle_commit(&mut self.popups, &self.space, surface);
    }
}

/// Implement the buffer handler (buffer destruction callback).
///
/// What:
///      `impl BufferHandler for Compositor`.
///  One hook,
///  called when a `wl_buffer`
///           is destroyed.
/// Why:
///       Required companion to shm/dmabuf;
///  we need no cleanup,
///  so it is a no-op.
impl BufferHandler for Compositor {
    /// What:
    ///      `fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}`.
    ///           Ignores the destroyed buffer.
    /// Why:
    ///       Smithay's renderer already drops the imported texture;
    ///  nothing extra
    ///           to do.
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

/// Implement the wl_shm handler (shared-memory buffer pool state).
///
/// What:
///      `impl ShmHandler for Compositor`.
///  Exposes the shm state.
/// Why:
///       Even though the GPU app uses dmabuf,
///  wl_shm must be functional.
impl ShmHandler for Compositor {
    /// What:
    ///      `fn shm_state(&self) -> &ShmState`.
    ///  Read-only accessor.
    /// Why:
    ///       Smithay reads advertised SHM formats through it.
    fn shm_state(&self) -> &ShmState {
        // What:     `&self.shm_state`. Borrow the field (tail expression).
        // Why:      Return the shm state.
        return &self.shm_state
    }
}

// What:     `delegate_compositor!(Compositor);`. Generate wl_compositor dispatch glue.
// Why:      Wire surface/subsurface requests to the handler above.
delegate_compositor!(Compositor);

// What:     `delegate_shm!(Compositor);`. Generate wl_shm dispatch glue.
// Why:      Wire shared-memory pool requests to the handler above.
delegate_shm!(Compositor);
