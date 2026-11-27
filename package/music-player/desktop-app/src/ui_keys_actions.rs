//! What each classified shortcut does (D102,
//! D106).
//!
//! What:
//! `run` takes the action the classifier named and performs it on the window and the host;
//! it returns true when the
//! key was used,
//! so the window does not deliver it to the focused element.
//! Why:
//! classifying and acting are separate steps,
//! so the classifier stays a table and its tests need no window,
//! and the
//! window tests can press every key through a recording host.

/// Imports the commands and the playback mode that shortcuts send to the engine.
use music_player::command::{
    Command,
    PlaybackMode,
};
/// Imports the shortcut action type that `run` matches.
use music_player::keymap::ShortcutAction;

/// Imports the row and menu actions that `run` hands the row shortcuts to.
use crate::ui_keys_row_actions::run_row_action;

/// Imports the window that actions change.
use crate::AppWindow;
/// Imports the host through which actions leave the window.
use crate::ui_host::Host;
/// Imports the key context that tells where the key was pressed.
use crate::ui_keys::KeyContext;
/// Imports the state accessors the playback actions read.
use crate::ui_screen::{
    read_screen,
    read_state,
};
/// Imports the page scope lookup the mode actions read.
use crate::ui_screen_view::selected_page_scope;

/// Seconds a plain seek moves.
const SEEK_SHORT_SECONDS: f64 = 10.0;
/// Seconds a seek with Shift moves.
const SEEK_LONG_SECONDS: f64 = 60.0;

/// What:
/// `run` performs one classified action;
/// returns true when the key was used.
/// Why:
/// see the module description.
/// `Owner` and `Unbound` are the two answers that leave the key to the focused element.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function run(win: AppWindow, host: Host, action: ShortcutAction, context: KeyContext): boolean { ... }
/// ```
pub(crate) fn run(
    window: &AppWindow,
    host: &dyn Host,
    action: ShortcutAction,
    context: &KeyContext,
) -> bool {
    return match action {
        ShortcutAction::Owner | ShortcutAction::Unbound => false,
        ShortcutAction::IgnoreRepeat => true,
        ShortcutAction::Toggle
        | ShortcutAction::Previous
        | ShortcutAction::Next
        | ShortcutAction::Mode
        | ShortcutAction::SeekBack
        | ShortcutAction::SeekForward
        | ShortcutAction::SeekBackLong
        | ShortcutAction::SeekForwardLong
        | ShortcutAction::ModeRepeat
        | ShortcutAction::ModeInOrder
        | ShortcutAction::ModeShuffleFolder
        | ShortcutAction::ModeShuffleAll
        | ShortcutAction::Restart
        | ShortcutAction::OpenFolderDialog
        | ShortcutAction::Quit => run_playback_action(host, action),
        ShortcutAction::Search
        | ShortcutAction::Picker
        | ShortcutAction::Settings
        | ShortcutAction::Reveal
        | ShortcutAction::RevealFolder
        | ShortcutAction::PreviousFolder
        | ShortcutAction::NextFolder
        | ShortcutAction::FocusFolders
        | ShortcutAction::FocusTracks
        | ShortcutAction::ClosePicker => run_player_page_action(window, host, action),
        ShortcutAction::ListUp
        | ShortcutAction::ListDown
        | ShortcutAction::ListHome
        | ShortcutAction::ListEnd
        | ShortcutAction::ListPageUp
        | ShortcutAction::ListPageDown
        | ShortcutAction::ListActivate
        | ShortcutAction::ListLetter(_) => {
            crate::ui_lists::apply(window, host, context.zone, action)
        }
        ShortcutAction::RowMenu
        | ShortcutAction::RowTrash
        | ShortcutAction::RowCopy
        | ShortcutAction::RowShow
        | ShortcutAction::MenuAccess(_)
        | ShortcutAction::MenuUp
        | ShortcutAction::MenuDown
        | ShortcutAction::MenuChoose
        | ShortcutAction::ClosePopup => run_row_action(window, host, action, context),
        ShortcutAction::Back
        | ShortcutAction::SettingsFocusList
        | ShortcutAction::EditorApply
        | ShortcutAction::EditorReset
        | ShortcutAction::EditorLeave => run_settings_action(window, action, context),
        ShortcutAction::SearchUp | ShortcutAction::SearchDown | ShortcutAction::SearchOpen => {
            run_search_action(window, host, action)
        }
    };
}

/// What:
/// `run_playback_action` runs the actions that end as engine commands or process requests.
/// Why:
/// they need the host and no window.
fn run_playback_action(
    host: &dyn Host,
    action: ShortcutAction,
) -> bool {
    match action {
        ShortcutAction::Toggle => crate::ui_wire::toggle_play(host),
        ShortcutAction::Previous => host.send(Command::Prev),
        ShortcutAction::Next => host.send(Command::Next),
        ShortcutAction::Mode => cycle_mode(host),
        ShortcutAction::SeekBack => seek_by(host, -SEEK_SHORT_SECONDS),
        ShortcutAction::SeekForward => seek_by(host, SEEK_SHORT_SECONDS),
        ShortcutAction::SeekBackLong => seek_by(host, -SEEK_LONG_SECONDS),
        ShortcutAction::SeekForwardLong => seek_by(host, SEEK_LONG_SECONDS),
        ShortcutAction::ModeRepeat => set_mode(host, PlaybackMode::Repeat),
        ShortcutAction::ModeInOrder => set_mode(host, PlaybackMode::InOrder),
        ShortcutAction::ModeShuffleFolder => set_mode(host, PlaybackMode::ShufflePage),
        ShortcutAction::ModeShuffleAll => set_mode(host, PlaybackMode::ShuffleAll),
        ShortcutAction::Restart => host.send(Command::Seek(0.0)),
        ShortcutAction::OpenFolderDialog => host.pick_folder(),
        ShortcutAction::Quit => host.quit(),
        _ => return false,
    }
    return true;
}

/// What:
/// `next_mode` gives the playback mode that follows `mode` in the Ctrl+M cycle.
/// Why:
/// D102 cycles the end-of-track mode through the four modes in the segmented control's order.
pub fn next_mode(mode: PlaybackMode) -> PlaybackMode {
    return match mode {
        PlaybackMode::Repeat => PlaybackMode::InOrder,
        PlaybackMode::InOrder => PlaybackMode::ShufflePage,
        PlaybackMode::ShufflePage => PlaybackMode::ShuffleAll,
        PlaybackMode::ShuffleAll => PlaybackMode::Repeat,
    };
}

/// What:
/// `cycle_mode` moves the playback mode to the next one and scopes the engine to the selected page.
/// Why:
/// the scope must match the page the user sees before a shuffle or in-order mode starts.
fn cycle_mode(host: &dyn Host) {
    let mode = read_screen().mode;
    set_mode(host, next_mode(mode));
}

/// What:
/// `set_mode` scopes the engine to the selected page and picks a playback mode.
/// Why:
/// a shuffle of the folder or an in-order run must start from the page the user is looking at.
fn set_mode(
    host: &dyn Host,
    mode: PlaybackMode,
) {
    let scope = read_state(selected_page_scope);
    host.send(Command::SetPageScope(scope));
    host.send(Command::SetPlaybackMode(mode));
}

/// What:
/// `seek_target` gives where a seek of `delta` seconds from `position` lands in a track of `duration` seconds,
/// or `None` when nothing is loaded.
/// Why:
/// a seek never leaves the track,
/// and with no duration there is nothing to seek in.
pub fn seek_target(
    position: f64,
    duration: f64,
    delta: f64,
) -> Option<f64> {
    if duration <= 0.0 {
        return None;
    }
    return Some((position + delta).clamp(0.0, duration));
}

/// What:
/// `seek_by` seeks `delta` seconds from the current position.
/// Why:
/// Alt+Left and Alt+Right move 10 seconds and Shift makes it 60 (D106).
fn seek_by(
    host: &dyn Host,
    delta: f64,
) {
    let (position, duration) = {
        let state = read_screen();
        (state.position, state.duration)
    };
    match seek_target(position, duration, delta) {
        Some(target) => host.send(Command::Seek(target)),
        None => tracing::debug!("a seek key was ignored: no track is loaded"),
    }
}

/// What:
///      `mod ui_keys_player_page_actions;` loads the sibling
///           `src/ui_keys_player_page_actions.rs`,
///  the chords that act on the player page.
/// Why:
///       They change the window's pages and focus,
///  and `run` only dispatches to them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as uiKeysPlayerPageActions from "./ui_keys_player_page_actions";
/// ```
#[path = "ui_keys_player_page_actions.rs"]
mod ui_keys_player_page_actions;

/// What:
///      `use ui_keys_player_page_actions::run_player_page_action` brings the dispatcher
///           the player-page chords go through.
/// Why:
///       `run` matches the action and hands it on,
///  and the chords themselves are in the sibling file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { run_player_page_action } from "./ui_keys_player_page_actions";
/// ```
use ui_keys_player_page_actions::run_player_page_action;

/// What:
///      `pub(crate) use ui_keys_player_page_actions::{...}` re-exports the two reveal
///           actions at this module's path.
/// Why:
///       The deck's title and folder name call `ui_keys_actions::reveal_playing` and
///  its sibling,
///  and only where they are declared moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { reveal_playing, reveal_playing_folder } from "./ui_keys_player_page_actions";
/// ```
pub(crate) use ui_keys_player_page_actions::{
    reveal_playing,
    reveal_playing_folder,
};

/// What:
///      `mod ui_keys_settings_and_search_actions;` loads the sibling
///           `src/ui_keys_settings_and_search_actions.rs`,
///  the chords that act on the Settings page and the Search page.
/// Why:
///       The Escape ladder and the editor chords read the route,
///  which the player-page actions do not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as uiKeysSettingsAndSearchActions from "./ui_keys_settings_and_search_actions";
/// ```
#[path = "ui_keys_settings_and_search_actions.rs"]
mod ui_keys_settings_and_search_actions;

/// What:
///      `use ui_keys_settings_and_search_actions::{...}` brings the two dispatchers
///           `run` hands those chords to.
/// Why:
///       `run` matches the action and hands it on,
///  and the chords themselves are in the sibling file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { run_search_action, run_settings_action } from "./ui_keys_settings_and_search_actions";
/// ```
use ui_keys_settings_and_search_actions::{
    run_search_action,
    run_settings_action,
};
