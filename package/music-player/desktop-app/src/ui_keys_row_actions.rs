//! What the shortcuts of a focused track row and of the track menu do.
//!
//! What:
//!  `run_row_action` performs a row shortcut or a menu shortcut:
//!  it opens the menu on the focused row,
//!  moves the menu ring,
//!  chooses the highlighted menu row,
//!  and runs a menu action by its chord or access letter.
//! Why:
//!  These actions share the menu ring and the focused row,
//!  and they fill a file of their own beside the other action groups of `ui_keys_actions.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function runRowAction(window: AppWindow, host: Host, action: ShortcutAction, context: KeyContext): boolean;
//! ```

/// Imports the shortcut action that the row shortcuts are classified as.
use music_player::keymap::ShortcutAction;

/// Imports the menu actions,
/// the lookup of an action by its access letter and the list of actions for a track.
use music_player::track_menu::{TrackMenuAction, access_action, track_menu_actions_for};

/// Imports the window that the actions change.
use crate::AppWindow;

/// Imports the host through which actions leave the window.
use crate::ui_host::Host;

/// Imports the key context that tells whether the menu is open.
use crate::ui_keys::KeyContext;

/// Imports the state accessors of the screen.
use crate::ui_screen::{read_screen, write_screen};

/// Imports the row offset calculation that places the menu beside its row.
use crate::ui_screen_scroll::row_offset_px;

/// What:
/// `menu_action_of` maps a track row shortcut to the menu action it runs.
/// Why:
/// the row chords and the menu rows are the same actions.
fn menu_action_of(action: ShortcutAction) -> Option<TrackMenuAction> {
    return match action {
        ShortcutAction::RowTrash => Some(TrackMenuAction::MoveToTrash),
        ShortcutAction::RowCopy => Some(TrackMenuAction::CopyFilename),
        ShortcutAction::RowShow => Some(TrackMenuAction::ShowInFileManager),
        _ => None,
    };
}

/// What:
/// `run_row_action` runs the track row chords,
/// the Menu key,
/// and the keys of the open track menu.
/// Why:
/// they act on the row with the keyboard ring,
/// or on the row the open menu belongs to.
pub(super) fn run_row_action(window: &AppWindow, host: &dyn Host, action: ShortcutAction, context: &KeyContext) -> bool {
    match action {
        ShortcutAction::RowMenu => request_row_menu(window),
        ShortcutAction::ClosePopup => window.invoke_close_menu(),
        ShortcutAction::MenuUp => move_menu_ring(window, -1),
        ShortcutAction::MenuDown => move_menu_ring(window, 1),
        ShortcutAction::MenuChoose => choose_menu_row(window, host),
        ShortcutAction::MenuAccess(letter) => {
            let Some(menu_action) = access_action(letter) else {
                return false;
            };
            run_chord_on_row(window, host, menu_action, true);
        }
        other => {
            let Some(menu_action) = menu_action_of(other) else {
                return false;
            };
            run_chord_on_row(window, host, menu_action, context.popup_open);
        }
    }
    return true;
}

/// What:
/// `run_chord_on_row` runs a menu action on the row the keyboard names.
/// Why:
/// with the menu open the row is the menu's;
/// otherwise it is the row with the ring,
/// and with no ring nothing happens.
fn run_chord_on_row(window: &AppWindow, host: &dyn Host, action: TrackMenuAction, popup_open: bool) {
    if !popup_open {
        let ring_track = read_screen().ring_track;
        let ring_index: Option<usize> = match ring_track {
            Some(ring) => usize::try_from(ring).ok(),
            None => None,
        };
        let Some(index) = ring_index else {
            tracing::debug!("a track row chord was ignored: no row has the keyboard ring");
            return;
        };
        {
            let mut state = write_screen();
            state.menu_index = Some(index);
        }
    }
    host.run_row_action(window, action);
    if popup_open {
        window.invoke_close_menu();
    }
}

/// What:
/// `request_row_menu` asks the track list to open the menu of the row with the ring,
/// as a right click would.
/// Why:
/// Menu and Alt+Enter open the focused row's menu (D106,
/// D114);
/// the list knows where the row is on screen.
fn request_row_menu(window: &AppWindow) {
    let (rows, ring) = { let state = read_screen(); (state.shown_rows.clone(), state.ring_track) };
    let mut ring_position: Option<usize> = None;
    for (at, row) in rows.iter().enumerate() {
        if row.kind == 0 && Some(row.index) == ring {
            ring_position = Some(at);
            break;
        }
    }
    let Some(position) = ring_position else {
        tracing::debug!("the menu key was ignored: no row has the keyboard ring");
        return;
    };
    {
        let mut state = write_screen();
        state.menu_by_key = true;
    }
    window.set_track_menu_index(ring.unwrap_or(-1));
    window.set_track_menu_offset(row_offset_px(&rows, position));
    window.set_track_menu_tick(window.get_track_menu_tick() + 1);
}

/// What:
/// `move_menu_ring` moves the highlighted row of the open menu by `delta`,
/// staying inside the menu.
/// Why:
/// Up and Down choose a menu row (D106);
/// the first Down with nothing highlighted lands on the first row.
fn move_menu_ring(window: &AppWindow, delta: isize) {
    let count = track_menu_actions_for().len();
    let row = {
        let mut state = write_screen();
        let next = match (state.menu_ring, delta) {
            (None, d) if d < 0 => count.saturating_sub(1),
            (None, _) => 0,
            (Some(at), d) => at.saturating_add_signed(d).min(count.saturating_sub(1)),
        };
        state.menu_ring = Some(next);
        next
    };
    window.set_menu_ring(i32::try_from(row).unwrap_or(-1));
    tracing::debug!(row, "the track menu highlight moved");
}

/// What:
/// `choose_menu_row` runs the highlighted menu row and closes the menu.
/// Why:
/// Enter chooses a row (D106);
/// with no row highlighted,
/// Enter does nothing and leaves the menu open.
fn choose_menu_row(window: &AppWindow, host: &dyn Host) {
    let Some(row) = read_screen().menu_ring else {
        return;
    };
    let Some(action) = track_menu_actions_for().get(row).copied() else {
        return;
    };
    host.run_row_action(window, action);
    window.invoke_close_menu();
}
