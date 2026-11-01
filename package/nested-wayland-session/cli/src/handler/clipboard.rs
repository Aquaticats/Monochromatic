//! Clipboard management stays on the nested seat,
//!  without a host clipboard bridge.

/// What:
///  Import Smithay's existing protocol states and dispatch macros.
/// Why:
///  Both clipboard-manager protocols use the same seat selection as wl_data_device.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { WlrDataControl, ExtDataControl } from 'smithay/selection';
/// ```
use smithay::{
    delegate_data_control, delegate_ext_data_control,
    reexports::wayland_server::DisplayHandle,
    wayland::selection::{ext_data_control, wlr_data_control},
};
/// Protocol callbacks operate on the compositor's existing owning state.
use crate::state::Compositor;

/// Own both protocol globals for this nested Wayland display.
pub struct ClipboardProtocols {
    /// Protocol used by existing Wayland clipboard tools and libraries.
    wlr: wlr_data_control::DataControlState,
    /// Standardized successor sharing the same nested clipboard contents.
    ext: ext_data_control::DataControlState,
}

/// Construct both clipboard-manager globals together so neither exists without the other.
impl ClipboardProtocols {
    /// Register clipboard management only on the supplied nested display.
    pub fn new(display: &DisplayHandle) -> Self {
        // What: None disables primary-selection advertising; |_client| is a callback.
        // Why: All clients admitted to this private nested socket may exchange its clipboard,
        // but no unsupported primary-selection or host-clipboard bridge is advertised.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const wlr = new WlrDataControl(display, undefined, client => true);
        // ```
        let wlr = wlr_data_control::DataControlState::new::<Compositor, _>(
            display, None, |_client| return true,
        );
        // Enable the successor protocol with identical nested-only permissions.
        let ext = ext_data_control::DataControlState::new::<Compositor, _>(
            display, None, |_client| return true,
        );
        tracing::info!("nested clipboard protocols ready: wlr-data-control and ext-data-control");
        return Self { wlr, ext };
    }
}

/// Expose the wlr protocol state to Smithay's generated request handlers.
impl wlr_data_control::DataControlHandler for Compositor {
    /// What:
    ///  & lends the protocol state rather than transferring its ownership.
    /// Why:
    ///  Every request uses the same display-owned global and nested seat data.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// dataControlState() { return this.clipboard.wlr; }
    /// ```
    fn data_control_state(&self) -> &wlr_data_control::DataControlState {
        return &self.clipboard.wlr;
    }
}

/// Expose the standardized protocol through its separate handler interface.
impl ext_data_control::DataControlHandler for Compositor {
    /// Lend the retained ext-data-control state to request dispatch.
    fn data_control_state(&self) -> &ext_data_control::DataControlState {
        return &self.clipboard.ext;
    }
}

// What: Smithay's macros implement protocol request dispatch for our state type.
// Why: Reuse the library's offer, transfer, replacement, and disconnect handling.
//
// In TS you'd write (pseudocode):
// ```ts
// registerProtocolHandlers(Compositor, [wlrDataControl, extDataControl]);
// ```
delegate_data_control!(Compositor);
// Wire the successor protocol to the same nested seat implementation.
delegate_ext_data_control!(Compositor);
