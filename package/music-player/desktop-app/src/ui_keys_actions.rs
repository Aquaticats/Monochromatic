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
use music_player::command::{Command, PlaybackMode};
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
/// Imports the focus target and the calls that move keyboard focus.
use crate::ui_keys_focus::{FocusTarget, request_focus, set_target};
/// Imports the projection and the state accessors of the screen.
use crate::ui_screen::{project, read_screen, read_state, write_screen};
/// Imports the scroll request that revealing the playing row sends.
use crate::ui_screen_scroll::ScrollRequest;
/// Imports the route,
/// the screen state and the page scope lookup.
use crate::ui_screen_view::{Route, selected_page_scope};

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
pub(crate) fn run(window: &AppWindow, host: &dyn Host, action: ShortcutAction, context: &KeyContext) -> bool {
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
        | ShortcutAction::ListLetter(_) => crate::ui_lists::apply(window, host, context.zone, action),
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
fn run_playback_action(host: &dyn Host, action: ShortcutAction) -> bool {
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
fn set_mode(host: &dyn Host, mode: PlaybackMode) {
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
pub fn seek_target(position: f64, duration: f64, delta: f64) -> Option<f64> {
    if duration <= 0.0 {
        return None;
    }
    return Some((position + delta).clamp(0.0, duration));
}

/// What:
/// `seek_by` seeks `delta` seconds from the current position.
/// Why:
/// Alt+Left and Alt+Right move 10 seconds and Shift makes it 60 (D106).
fn seek_by(host: &dyn Host, delta: f64) {
    let (position, duration) = { let state = read_screen(); (state.position, state.duration) };
    match seek_target(position, duration, delta) {
        Some(target) => host.send(Command::Seek(target)),
        None => tracing::debug!("a seek key was ignored: no track is loaded"),
    }
}

/// What:
/// `run_player_page_action` runs the actions that change the page,
/// the folder or the focus on the player page.
/// Why:
/// they act on the window and the screen state.
fn run_player_page_action(window: &AppWindow, host: &dyn Host, action: ShortcutAction) -> bool {
    match action {
        ShortcutAction::Search => crate::ui_search::open_search_route(window),
        ShortcutAction::Settings => crate::ui_settings::open_settings(window),
        ShortcutAction::Picker => toggle_picker(window),
        ShortcutAction::Reveal => reveal_playing(window),
        ShortcutAction::RevealFolder => reveal_playing_folder(window),
        ShortcutAction::PreviousFolder => step_folder(window, host, -1),
        ShortcutAction::NextFolder => step_folder(window, host, 1),
        ShortcutAction::FocusFolders => focus_folders(window),
        ShortcutAction::FocusTracks => focus_tracks(window),
        ShortcutAction::ClosePicker => close_picker(window),
        _ => return false,
    }
    return true;
}

/// What:
/// `toggle_picker` opens or closes the folder picker overlay on the folded layout,
/// and focuses the picker on the
/// unfolded layout where it is always shown.
/// Why:
/// Ctrl+O is "the folder picker" (D25,
/// D102);
/// on the two-pane layout the picker is already open,
/// so the useful
/// meaning is to move the keyboard to it.
fn toggle_picker(window: &AppWindow) {
    if window.get_unfolded() {
        focus_folders(window);
        return;
    }
    let was_open = read_screen().picker_open;
    if was_open {
        close_picker(window);
    } else {
        focus_folders(window);
    }
}

/// What:
/// `focus_folders` moves the keyboard to the folder picker,
/// opening the overlay first on the folded layout.
/// On the
/// folded layout it does nothing while the queue has no names.
/// Why:
/// Alt+1 (D106).
/// The target is set before the projection so a picker created by it takes focus when it appears.
/// The folder button already refuses to open a picker with nothing to list (EM5);
/// the keys must refuse too,
/// or Alt+1 or
/// Ctrl+O pressed at launch,
/// before the engine has reported the saved library,
/// opens a Folders pane with no rail and no
/// names that the first-run pane does not cover either.
fn focus_folders(window: &AppWindow) {
    if !window.get_unfolded() && read_screen().names.is_empty() {
        tracing::debug!("the folder picker was not opened: the library has no folders to list yet");
        return;
    }
    set_target(window, FocusTarget::Folders);
    if !window.get_unfolded() {
        {
            let mut state = write_screen();
            state.picker_open = true;
        }
        project(window);
    }
    request_focus(window, FocusTarget::Folders);
}

/// What:
/// `focus_tracks` moves the keyboard to the track list,
/// closing the picker overlay first on the folded layout.
/// Why:
/// Alt+2 (D106).
/// The target is set before the projection so a list created by it takes focus when it appears.
fn focus_tracks(window: &AppWindow) {
    set_target(window, FocusTarget::Tracks);
    if !window.get_unfolded() {
        {
            let mut state = write_screen();
            state.picker_open = false;
        }
        project(window);
    }
    request_focus(window, FocusTarget::Tracks);
}

/// What:
/// `close_picker` closes the folder picker overlay and moves focus to the track list it uncovers.
/// Why:
/// Escape closes the innermost thing first (D106),
/// and the keyboard must not be left on a destroyed list.
fn close_picker(window: &AppWindow) {
    focus_tracks(window);
}

/// What:
/// `step_folder` shows the folder `delta` pages from the selected one and scopes the engine to it.
/// Why:
/// Alt+Up and Alt+Down move to the previous and next folder (D106);
/// the ends do not wrap.
fn step_folder(window: &AppWindow, host: &dyn Host, delta: isize) {
    let chosen: Option<Vec<usize>> = {
        let mut state = write_screen();
        let target_identity = {
            let pages = state.current_pages();
            let count = pages.pages().count();
            let here = state.selected_page_position();
            let target = match here.checked_add_signed(delta) {
                Some(position) if position < count => Some(position),
                _ => None,
            };
            match target {
                Some(position) => pages.identity_at(position),
                None => None,
            }
        };
        match target_identity {
            Some(identity) => {
                state.selected_key = identity;
                Some(state.selected_page_scope_indices())
            }
            None => None,
        }
    };
    match chosen {
        Some(scope) => {
            crate::ui_wire::mark_user_acted();
            host.send(Command::SetPageScope(scope));
            project(window);
        }
        None => tracing::debug!(delta, "there is no folder in that direction"),
    }
}

/// What:
/// `reveal_playing` moves the list to the folder that holds the playing track,
/// closes the picker and asks
/// the list to scroll so the playing row is at the top.
/// Why:
/// Ctrl+G and a click on the deck's title jump to the playing track;
/// the request makes the list scroll even
/// when the folder and the row did not change.
pub(crate) fn reveal_playing(window: &AppWindow) {
    show_playing_page(ScrollRequest::PlayingRow);
    project(window);
}

/// What:
/// `reveal_playing_folder` moves the list to the folder that holds the playing track,
/// closes the picker and
/// asks the list to scroll so the subfolder header above the playing row is at the top,
/// or the folder start for a
/// track directly in it.
/// Why:
/// a click on the deck's folder name shows the folder;
/// the request makes the list scroll even when the folder
/// and the header did not change.
pub(crate) fn reveal_playing_folder(window: &AppWindow) {
    show_playing_page(ScrollRequest::PlayingFolder);
    project(window);
}

/// What:
/// `show_playing_page` selects the page that holds the playing track,
/// closes the picker,
/// and queues the scroll
/// request for the next projection when a track is playing.
/// Why:
/// both reveal actions show the playing track's folder,
/// so they share this state change;
/// with nothing playing
/// there is no row to reveal,
/// so the list keeps its position.
fn show_playing_page(request: ScrollRequest) {
    {
        let mut state = write_screen();
        state.picker_open = false;
        if let Some(index) = state.current_index {
            state.scroll_request = Some(request);
            let page = state.page_key_of_index(index);
            if let Some(page_key) = page {
                state.selected_key = page_key;
            }
        }
    }
}

/// What:
/// `run_settings_action` runs the Escape ladder and the editor chords of Settings and the editor page.
/// Why:
/// D106 and the human's corrections:
/// the field's Escape returns to the list,
/// the list's Escape closes Settings,
/// and
/// the editor page's Escape leaves it like Back.
fn run_settings_action(window: &AppWindow, action: ShortcutAction, context: &KeyContext) -> bool {
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
fn apply_template(window: &AppWindow, context: &KeyContext) -> bool {
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
fn reset_template(window: &AppWindow, context: &KeyContext) -> bool {
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
fn run_search_action(window: &AppWindow, host: &dyn Host, action: ShortcutAction) -> bool {
    match action {
        ShortcutAction::SearchUp => crate::ui_search::move_ring(window, -1),
        ShortcutAction::SearchDown => crate::ui_search::move_ring(window, 1),
        ShortcutAction::SearchOpen => crate::ui_search::open_ringed_result(window, host),
        _ => return false,
    }
    return true;
}
