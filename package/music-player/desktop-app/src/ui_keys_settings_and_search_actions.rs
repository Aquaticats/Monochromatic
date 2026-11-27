//! What the shortcut keys do on the Settings page and the Search page.
//!
//! What:
//! `run_settings_action` walks the Escape ladder and runs the editor chords,
//! `apply_template` and `reset_template` are the two editor chords,
//! and `run_search_action` moves through the Search results.
//! Why:
//! those chords act on a page and its editor rather than on the player,
//! so they are read together,
//! split from `src/ui_keys_actions.rs` to stay within the line budget.

/// What:
///      `use music_player::keymap::ShortcutAction` brings the classified action each
///           function matches.
/// Why:
///      The classifier names the action,
///  and this module only performs the ones these two pages own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { ShortcutAction } from "music_player/keymap";
/// ```
use music_player::keymap::ShortcutAction;

/// What:
///      `use crate::AppWindow` brings the window these actions change.
/// Why:
///      Every function here calls a page function that takes the window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { AppWindow } from "./app.slint";
/// ```
use crate::AppWindow;
/// What:
///      `use crate::ui_host::Host` brings the host that carries commands to the engine.
/// Why:
///      Opening a Search result goes through it,
///  and tests pass a recording host.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { Host } from "./ui_host";
/// ```
use crate::ui_host::Host;
/// What:
///      `use crate::ui_keys::KeyContext` brings the route and zone the key was pressed in.
/// Why:
///      Escape and the editor chords mean different things per page,
///  and the context carries the route.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { KeyContext } from "./ui_keys";
/// ```
use crate::ui_keys::KeyContext;
/// What:
///      `use crate::ui_keys_focus::{...}` brings the focus target and the call that moves
///           the keyboard.
/// Why:
///      Closing or applying a template returns the keyboard to the Settings list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { FocusTarget, request_focus } from "./ui_keys_focus";
/// ```
use crate::ui_keys_focus::{
    FocusTarget,
    request_focus,
};
/// What:
///      `use crate::ui_screen_view::Route` brings the page each chord is classified on.
/// Why:
///      The same Escape closes Settings or leaves the editor,
///  and the route decides which.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Route } from "./ui_screen_view";
/// ```
use crate::ui_screen_view::Route;
/// What:
/// `run_settings_action` runs the Escape ladder and the editor chords of Settings and the editor page.
/// Why:
/// D106 and the human's corrections:
/// the field's Escape returns to the list,
/// the list's Escape closes Settings,
/// and
/// the editor page's Escape leaves it like Back.
pub(super) fn run_settings_action(
    window: &AppWindow,
    action: ShortcutAction,
    context: &KeyContext,
) -> bool {
    match action {
        ShortcutAction::Back => match context.route {
            Route::Settings => crate::ui_settings::close_settings(window),
            Route::Search => crate::ui_search::close_search_route(window),
            Route::Licenses => crate::ui_licenses::close_licenses(window),
            Route::Player | Route::TemplateEditor => {
                tracing::warn!(route = ?context.route, "Back was classified on a page that has no Back target");
                return false;
            }
        },
        ShortcutAction::SettingsFocusList => request_focus(window, FocusTarget::SettingsList),
        ShortcutAction::EditorLeave => crate::ui_settings::editor_back(window),
        ShortcutAction::EditorApply => return apply_template(window, context),
        ShortcutAction::EditorReset => return reset_template(window, context),
        _ => return false,
    }
    return true;
}

/// What:
/// `apply_template` applies the open template and leaves the editor (Ctrl+Enter).
/// Why:
/// on the editor page that is Back;
/// on the two-pane Settings page the editor is a pane,
/// so leaving means returning to
/// the list.
/// On the single-column Settings list there is no editor to apply.
fn apply_template(
    window: &AppWindow,
    context: &KeyContext,
) -> bool {
    match context.route {
        Route::TemplateEditor => crate::ui_settings::editor_back(window),
        Route::Settings if window.get_unfolded() => {
            crate::ui_settings::apply_templates(window);
            request_focus(window, FocusTarget::SettingsList);
        }
        _ => return false,
    }
    return true;
}

/// What:
/// `reset_template` resets the open template to its default (Ctrl+Shift+Backspace).
/// Why:
/// the editor page and the two-pane Settings page have an editor with a Reset control;
/// the single-column list has none.
fn reset_template(
    window: &AppWindow,
    context: &KeyContext,
) -> bool {
    let has_editor = match context.route {
        Route::TemplateEditor => true,
        Route::Settings => window.get_unfolded(),
        _ => false,
    };
    if !has_editor {
        return false;
    }
    crate::ui_settings::reset_open_template(window);
    return true;
}

/// What:
/// `run_search_action` runs Up,
/// Down and Enter on the Search page.
/// Why:
/// the query field keeps the typing focus while these move through the results (D106).
pub(super) fn run_search_action(
    window: &AppWindow,
    host: &dyn Host,
    action: ShortcutAction,
) -> bool {
    match action {
        ShortcutAction::SearchUp => crate::ui_search::move_ring(window, -1),
        ShortcutAction::SearchDown => crate::ui_search::move_ring(window, 1),
        ShortcutAction::SearchOpen => crate::ui_search::open_ringed_result(window, host),
        _ => return false,
    }
    return true;
}
