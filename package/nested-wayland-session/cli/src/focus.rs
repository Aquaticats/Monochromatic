//! Keyboard focus across several toplevels:
//!  a new window takes focus,
//!  and focus returns to the most recently focused remaining window when one goes away.
//!
//! A hosted app opens native dialogs (a folder chooser from `zenity`,
//!  `kdialog`,
//!  or a desktop portal) as further toplevels of other client processes.
//!  A desktop compositor hands focus back to the window underneath when the dialog closes;
//!  without this module the app would keep no keyboard focus.

/// Smithay compositor types.
use smithay::{
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::{protocol::wl_surface::WlSurface, Resource},
    },
    utils::SERIAL_COUNTER,
};
/// Structured logging.
use tracing::{debug, info};

/// Sibling modules of this crate.
use crate::state::Compositor;

/// Put a surface first in the focus order,
///  dropping any earlier entry for it.
///
/// What:
///      `pub fn promote<T: PartialEq>(order: &mut Vec<T>, item: T)`.
///  Generic over the entry type so the order logic is tested without a display.
/// Why:
///       The order is most recently focused first,
///  and it is the list focus returns through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function promote(order: Surface[], surface: Surface): void { order.splice(order.indexOf(surface), 1); order.unshift(surface); }
/// ```
pub fn promote<T: PartialEq>(order: &mut Vec<T>, item: T) {
    order.retain(|entry| return *entry != item);
    order.insert(0, item);
}

/// Mark exactly one toplevel as activated,
///  configuring the ones whose state changed.
///
/// What:
///      `fn apply_activation(state: &mut Compositor, active: Option<&WlSurface>)`.
/// Why:
///       Toolkits draw focus styling and pause or resume work from the activated state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function applyActivation(state: Compositor, active?: Surface): void { ... }
/// ```
fn apply_activation(state: &mut Compositor, active: Option<&WlSurface>) {
    for window in state.space.elements() {
        let Some(toplevel) = window.toplevel() else {
            continue;
        };
        let wanted = active == Some(toplevel.wl_surface());
        let changed = toplevel.with_pending_state(|pending| {
            let current = pending.states.contains(xdg_toplevel::State::Activated);
            if wanted {
                pending.states.set(xdg_toplevel::State::Activated);
            } else {
                pending.states.unset(xdg_toplevel::State::Activated);
            }
            return current != wanted
        });
        if changed && toplevel.is_initial_configure_sent() {
            toplevel.send_pending_configure();
        }
    }
}

/// Give a surface keyboard focus and make it the most recently focused window.
///
/// What:
///      `pub fn focus_surface(state: &mut Compositor, surface: WlSurface)`.
/// Why:
///       Used when a window is mapped and when focus returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function focusSurface(state: Compositor, surface: Surface): void { ... }
/// ```
pub fn focus_surface(state: &mut Compositor, surface: WlSurface) {
    promote(&mut state.focus_order, surface.clone());
    apply_activation(state, Some(&surface));
    let Some(keyboard) = state.seat.get_keyboard() else {
        return;
    };
    keyboard.set_focus(state, Some(surface), SERIAL_COUNTER.next_serial());
    info!("keyboard focus moved to a toplevel; {} toplevels in the focus order", state.focus_order.len());
}

/// Handle a toplevel that was destroyed or whose client disconnected.
///
/// What:
///      `pub fn toplevel_gone(state: &mut Compositor, gone: &WlSurface)`.
///  Removes it from the focus order and from the space;
///  when it had keyboard focus the most recently focused remaining toplevel gets focus,
///  and with none left the keyboard focus is cleared.
/// Why:
///       Closing a dialog must return the keyboard to the window underneath.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function toplevelGone(state: Compositor, gone: Surface): void { ... }
/// ```
pub fn toplevel_gone(state: &mut Compositor, gone: &WlSurface) {
    state.focus_order.retain(|entry| return entry != gone && entry.is_alive());
    let doomed = state
        .space
        .elements()
        .find(|window| return window.toplevel().is_some_and(|toplevel| return toplevel.wl_surface() == gone))
        .cloned();
    if let Some(window) = doomed {
        state.space.unmap_elem(&window);
    }
    let Some(keyboard) = state.seat.get_keyboard() else {
        return;
    };
    let had_focus = keyboard.current_focus().is_none_or(|focused| return focused == *gone);
    if !had_focus {
        debug!("a toplevel went away without keyboard focus; focus stays");
        return;
    }
    match state.focus_order.first().cloned() {
        Some(next) => {
            info!("the focused toplevel went away; focus returns to the most recently focused remaining toplevel");
            focus_surface(state, next);
        }
        None => {
            info!("the focused toplevel went away and no toplevel remains; clearing keyboard focus");
            keyboard.set_focus(state, None, SERIAL_COUNTER.next_serial());
        }
    }
}

/// What:
///      `#[cfg(test)] #[path = "focus_tests.rs"] mod tests;`.
/// Why:
///       Keep the focus-order tests beside the order logic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./focus_tests";
/// ```
#[cfg(test)]
#[path = "focus_tests.rs"]
mod tests;
