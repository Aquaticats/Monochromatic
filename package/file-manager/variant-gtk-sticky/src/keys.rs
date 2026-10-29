//! Keyboard navigation for the sticky pane strip.
//!
//! Left/Right move keyboard focus (and the model's focus) between columns,
//!  captured before the
//! focused list so its Up/Down row navigation is untouched;
//!  Backspace closes the model-active
//! pane.
//!  The bindings match the Electron prototype so both nested-Wayland boundary tests drive
//! their apps with identical key sequences.

/// What:
///  imports the reference-counted pointer and its weak companion.
/// Why:
///  the key controller holds a `Weak<StripInner>` and upgrades it per key press.
use std::rc::Rc;

/// What:
///  imports the event-controller extension trait.
/// Why:
///  the key controller needs capture-phase propagation.
use gtk4::prelude::EventControllerExt;
/// What:
///  imports the key-symbol type.
/// Why:
///  the handler matches Left/Right/Backspace.
use gtk4::gdk::Key;
/// What:
///  imports the key event controller,
///  its propagation phase,
///  and the glib module.
/// Why:
///  a capture-phase `EventControllerKey` intercepts navigation keys and returns a
///      `glib::Propagation`.
use gtk4::{EventControllerKey, PropagationPhase, glib};

/// What:
///  imports the strip's shared inner state and its focus/close entry points.
/// Why:
///  navigation reads the pane model,
///  asks the layout adapter to move focus,
///  and closes panes.
use crate::strip::{StripInner, close_active, focus_pane};

/// What:
///  install a capture-phase key controller on the scroller:
///  Left/Right move column focus,
///       Backspace closes the model-active pane.
/// Why:
///  capture phase intercepts these keys before the focused list,
///  leaving its Up/Down row
///      navigation and Enter activation untouched.
pub(crate) fn install_strip_keys(inner: &Rc<StripInner>) {
    let keys = EventControllerKey::new();
    keys.set_propagation_phase(PropagationPhase::Capture);
    let weak = Rc::downgrade(inner);
    keys.connect_key_pressed(move |_, key, _, _| {
        let Some(upgraded) = weak.upgrade() else {
            return glib::Propagation::Proceed;
        };
        match key {
            Key::Left => return focus_relative_column(&upgraded, -1),
            Key::Right => return focus_relative_column(&upgraded, 1),
            Key::BackSpace => {
                close_active(&upgraded);
                return glib::Propagation::Stop
            }
            _ => return glib::Propagation::Proceed,
        }
    });
    inner.layout.add_column_key_controller(keys);
}

/// What:
///  move keyboard and model focus to the first pane of the column `delta` away,
///  if it exists.
/// Why:
///  Left/Right walk the lineage;
///  an out-of-range move is ignored and the key passes through.
fn focus_relative_column(inner: &Rc<StripInner>, delta: i32) -> glib::Propagation {
    let target = inner.layout.focused_column() as i32 + delta;
    let columns = inner.state.borrow().column_count() as i32;
    if target < 0 || target >= columns {
        return glib::Propagation::Proceed;
    }
    // The checked offset becomes a named column index so the `usize` never
    // shadows the `i32` it was cast from.
    let target_column = target as usize;
    let first = inner.state.borrow().first_pane_in_column(target_column);
    let Some(id) = first else {
        return glib::Propagation::Proceed;
    };
    if !inner.layout.focus_widget(id) {
        return glib::Propagation::Proceed;
    }
    inner.layout.set_focused_column(target_column);
    focus_pane(inner, id);
    return glib::Propagation::Stop
}
