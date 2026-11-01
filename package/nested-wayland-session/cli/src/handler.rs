//! Wayland protocol handler implementations for `Compositor`.
//!
//! Smithay dispatches each protocol's events by calling trait methods on the state.
//! This file holds the seat (input focus),
//!  data-device (clipboard),
//!  and output
//! handlers plus their `delegate_*!` glue;
//!  the compositor/shm,
//!  xdg-shell,
//!  and dmabuf
//! handlers live in the submodules declared below.

/// What:
///      `pub mod compositor;`.
///  Declares the `compositor` submodule from
///           `src/handler/compositor.rs` and re-exports it publicly.
/// Why:
///       Holds the surface-commit and shared-memory handlers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as compositor from "./handler/compositor";
/// ```
pub mod compositor;

/// What:
///      `pub mod xdg_shell;`.
///  Declares the xdg-shell submodule.
/// Why:
///       Holds the toplevel/popup handler that maps the app fullscreen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as xdgShell from "./handler/xdg_shell";
/// ```
pub mod xdg_shell;

/// What:
///      `pub mod dmabuf;`.
///  Declares the dmabuf submodule.
/// Why:
///       Holds the GPU-buffer import handler.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as dmabuf from "./handler/dmabuf";
/// ```
pub mod dmabuf;

/// Nested-only clipboard manager protocols share the existing seat selection.
pub mod clipboard;

/// Fractional-scale and viewporter handlers tell new surfaces the output scale.
pub mod scale;

/// What:
///      Grouped `use` of the seat,
///  output,
///  data-device,
///  and delegate items.
/// Why:
///       Bring the traits and macros the impls below need into scope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { SeatHandler, OutputHandler, ... } from "smithay";
/// ```
use smithay::{
    delegate_data_device, delegate_output, delegate_seat,
    input::{Seat, SeatHandler, SeatState},
    reexports::wayland_server::{protocol::wl_surface::WlSurface, Resource},
    wayland::{
        output::OutputHandler,
        selection::{
            data_device::{
                set_data_device_focus, ClientDndGrabHandler, DataDeviceHandler, DataDeviceState,
                ServerDndGrabHandler,
            },
            SelectionHandler,
        },
    },
};

/// What:
///      Grouped `use` of a file wrapper,
///  the write trait,
///  and an owned file
///           descriptor.
/// Why:
///       `ServerDndGrabHandler::send` writes the pending uri-list bytes into the fd the
///           app handed us.
///  `OwnedFd` is an owning file descriptor (closing it on drop
///           signals EOF to the app's reader);
///  `File::from(fd)` gives it `Write`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // fs.writeSync(fd, bytes); then close(fd).
/// ```
use std::{fs::File, io::Write, os::fd::OwnedFd};

/// What:
///      `use tracing::{info, warn};`.
///  Structured log macros.
/// Why:
///       Trace the server-drag hooks (drop,
///  data request,
///  finish,
///  cancel).
use tracing::{info, warn};

/// What:
///      `use crate::state::Compositor;`.
///  `crate::` is "this crate's root".
/// Why:
///       All the handler impls are `impl Trait for Compositor`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Compositor } from "../state";
/// ```
use crate::state::Compositor;

/// Implement the seat (input focus) handler for the compositor.
///
/// What:
///      `impl SeatHandler for Compositor`.
///  Declares the focus types and the
///           hooks Smithay calls when focus changes.
///  The three associated types are
///           all `WlSurface`:
///  keyboard,
///  pointer,
///  and touch focus are all surfaces.
/// Why:
///       Tells Smithay how focus is represented and lets us mirror keyboard focus
///           onto the data device (clipboard).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // implements SeatHandler with focus types = WlSurface
/// ```
impl SeatHandler for Compositor {
    /// Keyboard focus is a surface.
    type KeyboardFocus = WlSurface;
    /// Pointer focus is a surface.
    type PointerFocus = WlSurface;
    /// Touch focus is a surface.
    type TouchFocus = WlSurface;

    /// What:
    ///      `fn seat_state(&mut self) -> &mut SeatState<Compositor>`.
    ///  Hands
    ///           Smithay a mutable borrow of our seat state.
    /// Why:
    ///       Smithay mutates seat bookkeeping through this accessor.
    fn seat_state(&mut self) -> &mut SeatState<Compositor> {
        // What:     `&mut self.seat_state`. A mutable borrow of the field (tail expr).
        // Why:      Return the seat state Smithay asked for.
        return &mut self.seat_state
    }

    /// What:
    ///      `fn cursor_image(&mut self, _seat: &Seat<Self>, _image:
    ///           CursorImageStatus) {}`.
    ///  Called when the client sets a cursor;
    ///  both
    ///           arguments ignored.
    /// Why:
    ///       A headless testing fixture draws no cursor,
    ///  so this is a no-op.
    fn cursor_image(
        &mut self,
        _seat: &Seat<Self>,
        image: smithay::input::pointer::CursorImageStatus,
    ) {
        // What:     Remember the cursor the hovered client asked for.
        // Why:      The `cursor` control command reports it, so a test can check that a button
        //           shows a pointer cursor and a text field an I-beam.
        tracing::debug!(?image, "the hosted client set the pointer cursor");
        self.cursor_image = image;
    }

    /// What:
    ///      `fn focus_changed(&mut self, seat: &Seat<Self>, focused:
    ///           Option<&WlSurface>)`.
    ///  Called when keyboard focus moves;
    ///  `focused` is
    ///           the newly focused surface or `None`.
    /// Why:
    ///       Keep the data device's focus in sync so clipboard offers reach the
    ///           focused client.
    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
        // What:     `let dh = &self.display_handle;`. Borrow the display handle.
        // Why:      Needed to resolve a surface to its client below.
        let dh = &self.display_handle;

        // What:     `let client = focused.and_then(|s| dh.get_client(s.id()).ok());`.
        //           `.and_then(closure)` runs only when a surface is focused. `s.id()`
        //           is the surface's protocol id; `dh.get_client(id)` returns
        //           `Result<Client, _>`; `.ok()` converts that to `Option<Client>`
        //           (dropping the error). The overall result is `Option<Client>`.
        // Why:      Find which client owns the focused surface.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const client = focused ? tryGetClient(dh, focused.id()) : undefined;
        // ```
        let client = focused.and_then(|s| return dh.get_client(s.id()).ok());

        // What:     `set_data_device_focus(dh, seat, client);`. Free function that
        //           points the seat's data device at that client.
        // Why:      So the focused client can read/write the clipboard selection.
        set_data_device_focus(dh, seat, client);
    }
}

// What:     `delegate_seat!(Compositor);`. A macro that generates the wayland-server
//           `Dispatch`/`GlobalDispatch` boilerplate wiring `wl_seat` to our handler.
// Why:      Without it the protocol requests would have nowhere to dispatch.
//
// In TS you'd write (pseudocode):
// ```ts
// // registerSeatDispatch(Compositor);
// ```
delegate_seat!(Compositor);

/// Accept tablet tools on the seat,
///  which `wp_cursor_shape_manager_v1` requires of its handler.
///
/// What:
///      `impl TabletSeatHandler for Compositor {}`.
/// Why:
///       smithay's cursor-shape support is generic over seats with a tablet handler;
///  no tablet exists here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no tablet support needed; the interface is satisfied by its defaults.
/// ```
impl smithay::wayland::tablet_manager::TabletSeatHandler for Compositor {}

smithay::delegate_cursor_shape!(Compositor);

/// Implement the selection (clipboard) handler.
///
/// What:
///      `impl SelectionHandler for Compositor { type SelectionUserData = (); }`.
///           The associated type `()` is the unit type (an empty tuple,
///  "no data"),
///           meaning we attach no extra data to selections.
/// Why:
///       Required companion trait for the data device;
///  we keep it minimal.
impl SelectionHandler for Compositor {
    /// No extra per-selection data.
    type SelectionUserData = ();
}

/// Implement the data-device (clipboard / DnD) handler.
///
/// What:
///      `impl DataDeviceHandler for Compositor`.
///  Exposes the data-device state.
/// Why:
///       Smithay routes clipboard requests through this accessor.
impl DataDeviceHandler for Compositor {
    /// What:
    ///      `fn data_device_state(&self) -> &DataDeviceState`.
    ///  Read-only borrow.
    /// Why:
    ///       Smithay reads the data-device state through it.
    fn data_device_state(&self) -> &DataDeviceState {
        // What:     `&self.data_device_state`. Borrow the field (tail expression).
        // Why:      Hand Smithay the state.
        return &self.data_device_state
    }
}

/// What:
///      `impl ClientDndGrabHandler for Compositor {}`.
///  Empty impl:
///  accept the
///           default (no custom client drag-and-drop grab behaviour).
/// Why:
///       Required by the data-device delegate;
///  defaults suffice for a fixture.
impl ClientDndGrabHandler for Compositor {}

/// Implement server-initiated drag-and-drop so the compositor can BE the drag source.
///
/// What:
///      `impl ServerDndGrabHandler for Compositor`.
///  Overrides the hooks Smithay's
///           server DnD grab calls:
///  `send` (the app requests the drag data),
///  `dropped`,
///           `finished`,
///  and `cancelled`.
/// Why:
///       The `drop-file` control command (via `crate::dnd`) drives a compositor-side
///           drag toward the hosted app;
///  these hooks deliver the `text/uri-list` payload and
///           trace the outcome,
///  making the inbound-drop path testable without a file manager.
impl ServerDndGrabHandler for Compositor {
    /// What:
    ///      `fn send(&mut self, mime_type: String, fd: OwnedFd, _seat: Seat<Self>)`.
    ///  The
    ///           app requested the drag data for `mime_type` on the receive `fd`;
    ///  the seat is
    ///           unused.
    /// Why:
    ///       Write the pending uri-list bytes so the app's inbound reader receives them.
    fn send(&mut self, mime_type: String, fd: OwnedFd, _seat: Seat<Self>) {
        // What:     `info!(...)`. Note which mime type the app asked for.
        // Why:      Confirm the app reached the receive step of the drop.
        info!("drop-file: app requested drag data for mime {mime_type}");

        // What:     `let Some(bytes) = self.pending_dnd_uri_list.clone() else { warn!; return;
        //           };`. Take a copy of the pending payload; bail (with a warning) if none is
        //           set. `.clone()` copies the small `Vec<u8>`.
        // Why:      Never write stale or absent data.
        let Some(bytes) = self.pending_dnd_uri_list.clone() else {
            warn!("drop-file: app requested data but no uri-list is pending");
            return;
        };

        // What:     `let mut file = File::from(fd);`. Wrap the owned fd in a `File` so it has
        //           `Write`. Dropping `file` at scope end closes the fd.
        // Why:      Writing then closing signals end-of-data to the app's reader.
        let mut file = File::from(fd);

        // What:     `if let Err(err) = file.write_all(&bytes) { warn!(...); }`. Write the whole
        //           payload; log a failure. `&bytes` lends the buffer read-only.
        // Why:      The uri-list is tiny (one path), well under the pipe buffer, so one
        //           `write_all` suffices; a broken pipe is worth logging.
        if let Err(err) = file.write_all(&bytes) {
            warn!("drop-file: writing the uri-list to the app failed: {err}");
        }
    }

    /// What:
    ///      `fn dropped(&mut self, _seat: Seat<Self>)`.
    ///  The drag was released over the
    ///           app (a validated drop).
    /// Why:
    ///       Marks that the app accepted the offer and the drop is proceeding.
    fn dropped(&mut self, _seat: Seat<Self>) {
        // What:     `info!(...)`. Log the validated drop.
        // Why:      Distinguish a real drop from a cancellation.
        info!("drop-file: server drag dropped onto the app (accepted)");
    }

    /// What:
    ///      `fn finished(&mut self, _seat: Seat<Self>)`.
    ///  The app finished with the offer.
    /// Why:
    ///       Clear the pending payload now that the transfer is complete.
    fn finished(&mut self, _seat: Seat<Self>) {
        // What:     `info!(...)` then `self.pending_dnd_uri_list = None;`. Log and clear.
        // Why:      Release the payload so a later drag starts clean.
        info!("drop-file: app finished the drag; clearing the pending uri-list");
        self.pending_dnd_uri_list = None;
    }

    /// What:
    ///      `fn cancelled(&mut self, _seat: Seat<Self>)`.
    ///  The drag ended without a
    ///           validated drop (the app did not accept a mime type or choose an action).
    /// Why:
    ///       Surface the failure loudly (this is the exact symptom being diagnosed) and
    ///           clear the payload.
    fn cancelled(&mut self, _seat: Seat<Self>) {
        // What:     `warn!(...)` then clear. A cancellation is the "drop did not register"
        //           case.
        // Why:      Make an unaccepted drop obvious in the log.
        warn!("drop-file: server drag cancelled (app did not accept a valid drop)");
        self.pending_dnd_uri_list = None;
    }
}

// What:     `delegate_data_device!(Compositor);`. Generates the data-device dispatch glue.
// Why:      Wire `wl_data_device_manager` to our handler.
delegate_data_device!(Compositor);

/// What:
///      `impl OutputHandler for Compositor {}`.
///  Empty impl;
///  the default output
///           behaviour is all a single-output fixture needs.
/// Why:
///       Required to delegate `wl_output`.
impl OutputHandler for Compositor {}

// What:     `delegate_output!(Compositor);`. Generates the output dispatch glue.
// Why:      Wire `wl_output` / xdg-output to our (default) handler.
delegate_output!(Compositor);
