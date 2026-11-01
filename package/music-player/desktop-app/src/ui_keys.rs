//! Keyboard shortcuts for the player: each key becomes a `ShortcutRequest`, the `keymap` classifier names the action, and
//! the action runs against the window and the host (D102, D106).
//!
//! What: `on_key_pressed` and `on_key_released` take the raw key events of the window's key scope, feed the key hint
//! badges and the double tap of Shift detector, and hand every other key to `dispatch`, which classifies it and runs its
//! action; they return true when the key was used.
//! Why: the shortcut map lives in `keymap` and its table; this module only connects it to the window. Text fields keep
//! their own keys because the classifier reads the focus zone the focused element reported.

use std::cell::Cell;

use music_player::double_shift::{DoubleShift, ShiftInput};
use music_player::keymap::{FocusZone, KeyRoute, KeyboardPlatform, ShortcutAction, ShortcutRequest, shortcut_action};
use music_player::keymap_bindings::HeldModifiers;
use slint::ComponentHandle;

use crate::ui_host::Host;
use crate::ui_screen::read_state;
use crate::ui_screen_view::Route;
use crate::{AppWindow, Focus};

thread_local! {
    /// What: the double tap of Shift detector of this window thread.
    /// Why: Search opens on two quick taps of Shift, which spans several key events.
    static DOUBLE_SHIFT: Cell<DoubleShift> = Cell::new(DoubleShift::default());
}

/// What: `RawKey` is one key event exactly as the window's key scope reports it.
/// Why: the Slint callback passes seven values, and one struct keeps the handler's signature small and its call sites
/// readable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RawKey = { text: string; ctrl: boolean; meta: boolean; alt: boolean; shift: boolean; repeat: boolean; inPopup: boolean };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RawKey {
    /// The key as Slint reports it: a character, or a private character for a special key.
    pub text: String,
    /// Control flag as Slint reports it.
    pub ctrl: bool,
    /// Meta flag as Slint reports it.
    pub meta: bool,
    /// Alt flag.
    pub alt: bool,
    /// Shift flag.
    pub shift: bool,
    /// True for an auto-repeat press.
    pub repeat: bool,
    /// True while the track menu popup is open.
    pub in_popup: bool,
}

/// What: `KeyInput` is one key press with the key already named.
/// Why: the window passes the key name and modifiers as plain values, so the mapping is testable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type KeyInput = { key: string; ctrl: boolean; meta: boolean; alt: boolean; shift: boolean; repeat: boolean; inPopup: boolean };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyInput {
    /// DOM-style key name, such as `Escape`, `ArrowLeft` or a printable character.
    pub key: String,
    /// Control modifier.
    pub ctrl: bool,
    /// Command (Mac) or Windows key modifier.
    pub meta: bool,
    /// Alt modifier.
    pub alt: bool,
    /// Shift modifier.
    pub shift: bool,
    /// True for an auto-repeat press.
    pub repeat: bool,
    /// True while a popup such as the track menu is open.
    pub in_popup: bool,
}

/// What: `KeyContext` is the page, the focus zone and the picker overlay state a key is read in.
/// Why: the same key means different things per page and per zone, and the classifier needs them as one value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type KeyContext = { route: Route; zone: FocusZone; pickerOverlay: boolean };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyContext {
    /// Page the window shows over the player.
    pub route: Route,
    /// Kind of element that holds keyboard focus.
    pub zone: FocusZone,
    /// True while the folder picker covers the track list on the folded layout.
    pub picker_overlay: bool,
    /// True while the track menu popup is open.
    pub popup_open: bool,
}

/// What: `platform` gives the keyboard convention of the running OS.
/// Why: the shortcut map differs between Windows and Linux (Ctrl) and macOS (Command).
pub fn platform() -> KeyboardPlatform {
    return if cfg!(target_os = "macos") {
        KeyboardPlatform::Mac
    } else {
        KeyboardPlatform::WindowsLinux
    };
}

/// What: `key_route` maps the window's route to the route the classifier reads.
/// Why: the classifier lives in the library crate and does not know the window's route type.
pub fn key_route(route: Route) -> KeyRoute {
    return match route {
        Route::Player => KeyRoute::Player,
        Route::Search => KeyRoute::Search,
        Route::Settings => KeyRoute::Settings,
        Route::TemplateEditor => KeyRoute::EditorPage,
        Route::Licenses => KeyRoute::Licenses,
    };
}

/// What: `held_modifiers` reads the modifier flags of a Slint key event for a map.
/// Why: on macOS Slint reports the Command key as Control and the Control key as Meta (its winit backend swaps them to
/// match Qt), while the classifier reads Command as Meta, so the two flags swap there. The swap is not verified here: this
/// machine runs neither macOS nor Windows.
pub fn held_modifiers(ctrl: bool, meta: bool, alt: bool, shift: bool, apple: bool) -> HeldModifiers {
    return if apple {
        HeldModifiers { ctrl: meta, meta: ctrl, alt, shift }
    } else {
        HeldModifiers { ctrl, meta, alt, shift }
    };
}

/// What: `key_name_of` maps the text of a Slint key event to the DOM-style name the classifier reads.
/// Why: Slint reports special keys as private characters, and the classifier and its table speak in names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const keyNameOf = (text: string): string => SPECIAL[text] ?? text;
/// ```
pub fn key_name_of(text: &str) -> String {
    return match text {
        "\u{1b}" => "Escape".to_owned(),
        "\n" => "Enter".to_owned(),
        "\u{8}" => "Backspace".to_owned(),
        "\u{7f}" => "Delete".to_owned(),
        "\t" | "\u{19}" => "Tab".to_owned(),
        "\u{f700}" => "ArrowUp".to_owned(),
        "\u{f701}" => "ArrowDown".to_owned(),
        "\u{f702}" => "ArrowLeft".to_owned(),
        "\u{f703}" => "ArrowRight".to_owned(),
        "\u{f708}" => "F5".to_owned(),
        "\u{f70d}" => "F10".to_owned(),
        "\u{f729}" => "Home".to_owned(),
        "\u{f72b}" => "End".to_owned(),
        "\u{f72c}" => "PageUp".to_owned(),
        "\u{f72d}" => "PageDown".to_owned(),
        "\u{f735}" => "ContextMenu".to_owned(),
        other => other.to_owned(),
    };
}

/// What: `is_shift_key` tells whether a key event is the left or right Shift key.
/// Why: the double tap counts both Shift keys.
fn is_shift_key(text: &str) -> bool {
    return matches!(text, "\u{10}" | "\u{15}");
}

/// What: `is_modifier_key` tells whether a key event is a modifier key on its own (Shift, Control, Alt, AltGr, Meta,
/// Caps Lock).
/// Why: a modifier alone is never a shortcut; it only feeds the key hint badges and the double tap of Shift.
fn is_modifier_key(text: &str) -> bool {
    return matches!(text, "\u{10}" | "\u{11}" | "\u{12}" | "\u{13}" | "\u{14}" | "\u{15}" | "\u{16}" | "\u{17}" | "\u{18}");
}

/// What: `request_for` builds the classifier input for one key press in a context.
/// Why: the classifier needs the platform, the page, the focus zone and the popup state, which come from the window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const requestFor = (input: KeyInput, context: KeyContext, platform: Platform): ShortcutRequest => ({ ... });
/// ```
pub fn request_for(input: &KeyInput, context: &KeyContext, platform: KeyboardPlatform) -> ShortcutRequest {
    return ShortcutRequest {
        platform,
        key: input.key.clone(),
        ctrl: input.ctrl,
        meta: input.meta,
        alt: input.alt,
        shift: input.shift,
        repeat: input.repeat,
        zone: context.zone,
        route: key_route(context.route),
        in_popup: input.in_popup,
        picker_overlay: context.picker_overlay,
        composing: false,
    };
}

/// What: `context_of` reads the page, the focus zone and the picker overlay state from the screen state and the window.
/// Why: the page and the picker flag live in the state, and the zone lives in the window's `Focus` global.
pub(crate) fn context_of(window: &AppWindow) -> KeyContext {
    let (route, picker_open) = read_state(|state| return (state.route, state.picker_open));
    return KeyContext {
        route,
        zone: FocusZone::from_int(window.global::<Focus>().get_zone()),
        picker_overlay: picker_open && !window.get_unfolded() && route == Route::Player,
        popup_open: false,
    };
}

/// What: `classify` names the action of one key press on the page and zone the window shows now.
/// Why: dispatching and the tests classify through the same call, so a key means one thing wherever it is read.
pub(crate) fn classify(window: &AppWindow, input: &KeyInput) -> (ShortcutAction, KeyContext) {
    let context = KeyContext { popup_open: input.in_popup, ..context_of(window) };
    let action = shortcut_action(&request_for(input, &context, platform()));
    tracing::debug!(
        key = %input.key,
        action = action.wire_name(),
        route = ?context.route,
        zone = ?context.zone,
        picker_overlay = context.picker_overlay,
        "shortcut classified"
    );
    return (action, context);
}

/// What: `dispatch` classifies one named key press and runs its action; returns true when the key was used.
/// Why: a used key stops the window from delivering it to the focused field, and an unused key continues to it.
pub(crate) fn dispatch(window: &AppWindow, host: &dyn Host, input: &KeyInput) -> bool {
    let (action, context) = classify(window, input);
    let used = crate::ui_keys_actions::run(window, host, action, &context);
    if used {
        tracing::info!(key = %input.key, action = action.wire_name(), "shortcut used");
        crate::ui_state_log::emit(window, "key");
    }
    return used;
}

/// What: `track_double_shift` feeds one input to the double tap detector and opens Search when a double tap completes
/// on the player page.
/// Why: D106: double Shift opens Search on the player page only; on Settings, its editor page and Search the detector
/// still runs (so its state stays honest) but the tap does nothing.
fn track_double_shift(window: &AppWindow, input: ShiftInput) {
    let now = crate::ui_hints::now_ms();
    let before = DOUBLE_SHIFT.with(Cell::get);
    let (after, completed) = before.step(input, now);
    DOUBLE_SHIFT.with(|cell| cell.set(after));
    if !completed {
        return;
    }
    let context = context_of(window);
    if context.route == Route::Player {
        tracing::info!("double Shift opens Search");
        crate::ui_search::open_search_route(window);
        crate::ui_state_log::emit(window, "double-shift");
    } else {
        tracing::info!(route = ?context.route, "double Shift ignored: Search is offered on the player page only");
    }
}

/// What: `on_key_pressed` handles one key press from the window's key scope; returns true when the key was used.
/// Why: it keeps the badge and double tap state in step with every key event, ignores modifier keys as shortcuts, and
/// sends every other key through the classifier. A key press also turns the focus ring on, because the keyboard is in use.
pub(crate) fn on_key_pressed(window: &AppWindow, host: &dyn Host, key: &RawKey) -> bool {
    let held = held_modifiers(key.ctrl, key.meta, key.alt, key.shift, cfg!(target_os = "macos"));
    crate::ui_hints::modifiers_changed(window, held);
    if is_modifier_key(&key.text) {
        if is_shift_key(&key.text) {
            track_double_shift(window, ShiftInput::ShiftDown { repeat: key.repeat });
        } else {
            track_double_shift(window, ShiftInput::OtherKey);
        }
        return false;
    }
    track_double_shift(window, ShiftInput::OtherKey);
    window.global::<Focus>().set_ring_visible(true);
    let input = KeyInput {
        key: key_name_of(&key.text),
        ctrl: held.ctrl,
        meta: held.meta,
        alt: held.alt,
        shift: held.shift,
        repeat: key.repeat,
        in_popup: key.in_popup,
    };
    return dispatch(window, host, &input);
}

/// What: `on_key_released` handles one key release from the window's key scope.
/// Why: a release updates the held modifiers (so the badges narrow or hide) and ends a Shift tap for the double tap.
pub(crate) fn on_key_released(window: &AppWindow, key: &RawKey) {
    let held = held_modifiers(key.ctrl, key.meta, key.alt, key.shift, cfg!(target_os = "macos"));
    crate::ui_hints::modifiers_changed(window, held);
    if is_shift_key(&key.text) {
        track_double_shift(window, ShiftInput::ShiftUp);
    }
}

#[cfg(test)]
#[path = "ui_keys_tests.rs"]
mod tests;
