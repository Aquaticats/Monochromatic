//! The state each connected Wayland client keeps in this compositor,
//!  and the two event-loop callbacks that register clients and dispatch their
//!  queued requests.
//!
//! Split out of `state.rs` to keep that file under the code-line limit.

/// The event loop's post-action,
///  a readable source's readiness,
///  and the display the source holds.
use smithay::reexports::{
    calloop::{
        PostAction,
        Readiness,
        generic::NoIoDrop,
    },
    wayland_server::{
        Display,
        backend::{
            ClientData,
            ClientId,
            DisconnectReason,
        },
    },
};
/// The compositor protocol's per-client sub-state.
use smithay::wayland::compositor::CompositorClientState;
/// A connected client's socket,
///  and the shared owner wayland-server keeps its per-client data behind.
use std::{
    os::unix::net::UnixStream,
    sync::Arc,
};

/// The compositor these callbacks hand their state to.
use super::Compositor;
/// The listening socket's callback:
///  register each new client with the display.
///
/// What:
///      `fn on_new_client(stream: UnixStream, _metadata: &mut (), state: &mut Compositor)`.
///  `insert_client` registers the connection with `Arc::new(ClientState::default())` as its per-client data
///  (wayland-server requires `Arc`,
///  not `Box`,
///  because it may access the data from its own threads).
///  A failure goes to `fail`,
///  which ends the session with an error.
///  The unit-typed `_metadata` is the socket source's metadata slot,
///  which it leaves empty.
/// Why:
///       Give each client a place to keep its compositor-side state.
///  A named function replaces the closure the socket source took.
pub(super) fn on_new_client(
    stream: UnixStream,
    _metadata: &mut (),
    state: &mut Compositor,
) {
    let inserted = state
        .display_handle
        .insert_client(stream, Arc::new(ClientState::default()));
    if let Err(error) = inserted {
        state.fail(
            anyhow::Error::new(error)
                .context("registering a connecting Wayland client with the nested display failed"),
        );
    }
}

/// The display source's callback:
///  dispatch the queued requests of every connected client.
///
/// What:
///      `fn dispatch_display_clients(_readiness: Readiness, pending_display: &mut NoIoDrop<Display<Compositor>>, state: &mut Compositor) -> std::io::Result<PostAction>`.
///  A failure ends the event loop;
///  `run` reports it as "event loop failed".
///  `Ok(PostAction::Continue)` tells calloop the source is healthy and should keep listening.
/// Why:
///       This is what actually processes client protocol messages.
///  A named function replaces the closure the display source took.
pub(super) fn dispatch_display_clients(
    _readiness: Readiness,
    pending_display: &mut NoIoDrop<Display<Compositor>>,
    state: &mut Compositor,
) -> std::io::Result<PostAction> {
    // SAFETY: the display is never dropped while dispatching.
    let dispatched = unsafe { pending_display.get_mut().dispatch_clients(state) };
    if let Err(error) = dispatched {
        return Err(std::io::Error::new(
            error.kind(),
            format!("dispatching Wayland client requests failed: {error}"),
        ));
    }
    return Ok(PostAction::Continue);
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
    fn initialized(
        &self,
        _client_id: ClientId,
    ) {
    }

    /// What:
    ///      `fn disconnected(&self, _client_id: ClientId, _reason:
    ///           DisconnectReason) {}`.
    ///  Called when the client goes away.
    /// Why:
    ///       Nothing to do on disconnect;
    ///  the exit-poll timer handles shutdown.
    fn disconnected(
        &self,
        _client_id: ClientId,
        _reason: DisconnectReason,
    ) {
    }
}
