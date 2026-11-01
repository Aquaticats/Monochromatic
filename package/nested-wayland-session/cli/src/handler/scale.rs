//! wp_fractional_scale_v1 and wp_viewporter handlers:
//!  a new surface learns the output scale.

/// What:
///      Grouped `use` of the two delegate macros,
///  the surface type,
///  the per-surface state
///           accessor,
///  and the fractional-scale handler trait and accessor.
/// Why:
///       The impl and the dispatch glue below reference exactly these.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { FractionalScaleHandler, withFractionalScale, withStates, ... } from "smithay";
/// ```
use smithay::{
    delegate_fractional_scale, delegate_viewporter,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::{
        compositor::with_states,
        fractional_scale::{with_fractional_scale, FractionalScaleHandler},
    },
};

/// What:
///      `use crate::state::Compositor;`.
///  Our state type.
/// Why:
///       The handler is `impl FractionalScaleHandler for Compositor`.
use crate::state::Compositor;

/// Tell each new fractional-scale object the current output scale at once.
///
/// What:
///      `impl FractionalScaleHandler for Compositor { ... }`.
///  Smithay calls
///           `new_fractional_scale` when a client asks for a surface's fractional-scale object.
/// Why:
///       winit asks for it when it creates a window,
///  before the first configure,
///  and
///           expects the scale before that configure so its first buffer already has the right
///           size.
///  Smithay only replays a scale stored earlier,
///  so a fresh surface needs it here.
impl FractionalScaleHandler for Compositor {
    /// What:
    ///      `fn new_fractional_scale(&mut self, surface: WlSurface)`.
    ///  Receives the surface
    ///           by value (an owned,
    ///  cheap handle).
    /// Why:
    ///       Store and send the current scale for this surface.
    fn new_fractional_scale(&mut self, surface: WlSurface) {
        // Read the factor first so the closures below borrow nothing from `self`.
        let factor = self.screen.geometry.scale.factor();
        // What:     `with_states(&surface, |states| { ... })` lends the surface's stored data
        //           to the closure; `with_fractional_scale` lends its fractional record inside.
        //           `set_preferred_scale` sends `preferred_scale(round(factor * 120))`.
        // Why:      The client renders its first frame at the output scale, not at 1.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // fractionalScaleOf(surface).setPreferredScale(factor);
        // ```
        with_states(&surface, |states| {
            with_fractional_scale(states, |fractional| {
                fractional.set_preferred_scale(factor);
            });
        });
        tracing::debug!(factor, "told a new surface the output scale");
    }
}

// What:     `delegate_fractional_scale!(Compositor);`. Generate the fractional-scale dispatch glue.
// Why:      Wire `wp_fractional_scale_manager_v1` and `wp_fractional_scale_v1` to the handler.
delegate_fractional_scale!(Compositor);

// What:     `delegate_viewporter!(Compositor);`. Generate the viewporter dispatch glue.
// Why:      Smithay stores each viewport's destination size, and the renderer draws the buffer
//           at that logical size; a fractional-scale client needs it to say how big it is.
delegate_viewporter!(Compositor);
