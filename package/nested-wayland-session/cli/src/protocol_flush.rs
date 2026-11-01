//! End-of-dispatch protocol delivery is independent of GPU frame production.

/// The handle owns access to queued Wayland events without exposing compositor state.
use smithay::reexports::wayland_server::DisplayHandle;

/// Deliver queued replies and input events even when the parent compositor supplies no redraw.
/// A disconnected client must not produce a bare shutdown error or stop unrelated clients.
pub fn finish_dispatch(display: &mut DisplayHandle) {
    if let Err(error) = display.flush_clients() {
        tracing::warn!(%error, "failed to flush nested Wayland protocol events");
    }
}
