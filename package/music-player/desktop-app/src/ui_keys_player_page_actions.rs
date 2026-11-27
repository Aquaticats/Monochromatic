//! What the shortcut keys do on the player page.
//!
//! What:
//! `run_player_page_action` hands each player-page chord to the function that
//! performs it,
//! which opens the picker,
//! moves the keyboard between the folder and track lists,
//! steps through the folders and reveals the playing track.
//! Why:
//! those actions change the window and the page scope rather than the engine
//! alone,
//! so they are read together,
//! split from `src/ui_keys_actions.rs` to stay within the line budget.

/// What:
///      `use music_player::keymap::ShortcutAction` brings the classified action each
///           function matches.
/// Why:
///      The classifier names the action,
///  and this module only performs the ones the player page owns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { ShortcutAction } from "music_player/keymap";
/// ```
use music_player::keymap::ShortcutAction;

/// What:
///      `use music_player::command::Command` brings the engine command that stepping a
///           folder sends.
/// Why:
///      Alt+Up and Alt+Down scope playback to another page,
///  which only the engine can do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Command } from "music_player/command";
/// ```
use music_player::command::Command;

/// What:
///      `use crate::AppWindow` brings the window these actions change.
/// Why:
///      Every function here writes a window property or reads one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { AppWindow } from "./app.slint";
/// ```
use crate::AppWindow;
/// What:
///      `use crate::ui_host::Host` brings the host that carries commands to the engine.
/// Why:
///      Stepping a folder sends one command through it,
///  and tests pass a recording host.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { Host } from "./ui_host";
/// ```
use crate::ui_host::Host;
/// What:
///      `use crate::ui_keys_focus::{...}` brings the focus targets and the two calls that
///           move the keyboard.
/// Why:
///      Each of these chords ends by placing the keyboard somewhere,
///  and the focus rules live in that module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { FocusTarget, request_focus, set_target } from "./ui_keys_focus";
/// ```
use crate::ui_keys_focus::{
    FocusTarget,
    request_focus,
    set_target,
};
/// What:
///      `use crate::ui_screen::{...}` brings the projection and the state accessors.
/// Why:
///      A page change writes the state and re-projects the window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { project, read_screen, write_screen } from "./ui_screen";
/// ```
use crate::ui_screen::{
    project,
    read_screen,
    write_screen,
};
/// What:
///      `use crate::ui_screen_scroll::ScrollRequest` brings the request that revealing a
///           row sends.
/// Why:
///      Revealing the playing track scrolls the list even when the page did not change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ScrollRequest } from "./ui_screen_scroll";
/// ```
use crate::ui_screen_scroll::ScrollRequest;
/// What:
/// `run_player_page_action` runs the actions that change the page,
/// the folder or the focus on the player page.
/// Why:
/// they act on the window and the screen state.
pub(super) fn run_player_page_action(
    window: &AppWindow,
    host: &dyn Host,
    action: ShortcutAction,
) -> bool {
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
fn step_folder(
    window: &AppWindow,
    host: &dyn Host,
    delta: isize,
) {
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
