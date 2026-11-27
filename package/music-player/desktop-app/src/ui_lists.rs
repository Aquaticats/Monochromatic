//! The keyboard ring inside the folder picker and the track list (D106).
//!
//! What:
//! `apply` runs a list key (Up,
//! Down,
//! Home,
//! End,
//! the page keys,
//! Enter,
//! a letter) on the list that holds focus;
//! `sync_rings` keeps the ring on a row that still exists after a projection;
//! `list_focused` starts the ring when a list
//! gains focus.
//! Why:
//! the ring position is state the row chords act on (Delete,
//! copy,
//! details),
//! so the Rust side owns it and pushes it
//! to the window;
//! the movement rules themselves are the pure functions of `list_focus`.
//! The row layout the window's pane asks back lives in `ui_lists/track_layout.rs`.

/// What:
/// `mod track_layout;` the published row tops and the window's two layout questions,
/// with the callbacks that answer them (`src/ui_lists/track_layout.rs`).
/// Why:
/// the pane builds only the rows on screen,
/// so the geometry of the whole list crosses to
/// the window as plain numbers,
/// apart from the ring keys below.
mod track_layout;

/// Re-exports the layout publication,
/// which the screen calls after each projection through this module's path.
pub(crate) use self::track_layout::publish_track_layout;

/// Imports the focus zone and the shortcut action that list keys carry.
use music_player::keymap::{
    FocusZone,
    ShortcutAction,
};
/// Imports the focus movement rules and the letter jump.
use music_player::list_focus::{
    ListMove,
    first_starting_with,
    move_focus,
    start_focus,
};
/// Imports the component handle that reaches the window's globals.
use slint::ComponentHandle;

/// Imports the window that shows the lists.
use crate::AppWindow;
/// Imports the generated focus type.
use crate::Focus;
/// Imports the host through which actions leave the window.
use crate::ui_host::Host;
/// Imports the projection and the state accessors of the screen.
use crate::ui_screen::{
    project,
    read_screen,
    write_screen,
};
/// Imports the row types of the track list and the folder picker.
use crate::ui_screen_rows::{
    ListRow,
    PickerName,
};
/// Imports the scroll geometry of the two rings:
/// a row's height and where it starts.
use crate::ui_screen_scroll::{
    row_height_px,
    row_offset_px,
};

/// What:
/// `track_position_of` finds the list position of the track row with library index `ring`.
/// Why:
/// the ring is stored by library index,
/// which survives a re-projection,
/// while movement works on list positions.
fn track_position_of(
    rows: &[ListRow],
    ring: Option<i32>,
) -> Option<usize> {
    let wanted = ring?;
    for (position, row) in rows.iter().enumerate() {
        if row.kind == 0 && row.index == wanted {
            return Some(position);
        }
    }
    return None;
}

/// What:
/// `folder_position_of` finds the position of the folder name `ring` among the picker's name rows.
/// Why:
/// the folder ring is stored by name,
/// like the picker's own current folder.
fn folder_position_of(
    names: &[PickerName],
    ring: Option<&str>,
) -> Option<usize> {
    let wanted = ring?;
    for (position, row) in names.iter().enumerate() {
        if !row.section_start && row.name == wanted {
            return Some(position);
        }
    }
    return None;
}

/// What:
/// `track_selectable` marks the rows that can hold the ring:
/// tracks,
/// not subfolder headers.
/// Why:
/// `move_focus` skips the rest.
fn track_selectable(rows: &[ListRow]) -> Vec<bool> {
    let mut selectable: Vec<bool> = Vec::with_capacity(rows.len());
    for row in rows {
        selectable.push(row.kind == 0);
    }
    return selectable;
}

/// What:
/// `folder_selectable` marks the name rows that can hold the ring:
/// folder names,
/// not headings.
/// Why:
/// `move_focus` skips the rest.
fn folder_selectable(names: &[PickerName]) -> Vec<bool> {
    let mut selectable: Vec<bool> = Vec::with_capacity(names.len());
    for row in names {
        selectable.push(!row.section_start);
    }
    return selectable;
}

/// What:
/// `place_track_ring` puts the track ring on the row at `position` and asks the list to scroll it into view.
/// Why:
/// the window needs the library index to draw the ring and the row's offset and height to scroll to it.
fn place_track_ring(
    window: &AppWindow,
    rows: &[ListRow],
    position: usize,
) {
    let Some(row) = rows.get(position) else {
        return;
    };
    {
        let mut state = write_screen();
        state.ring_track = Some(row.index);
    }
    window.set_ring_track(row.index);
    window.set_track_ring_offset(row_offset_px(rows, position));
    window.set_track_ring_height(row_height_px(row));
    window.set_track_ring_tick(window.get_track_ring_tick() + 1);
    tracing::debug!(index = row.index, position, "the track ring moved");
}

/// What:
/// `place_folder_ring` puts the folder ring on the name row at `position` and asks the picker to scroll to it.
/// Why:
/// the picker finds the row's place itself,
/// so only the name and a tick cross.
fn place_folder_ring(
    window: &AppWindow,
    names: &[PickerName],
    position: usize,
) {
    let Some(row) = names.get(position) else {
        return;
    };
    {
        let mut state = write_screen();
        state.ring_folder = Some(row.name.clone());
    }
    window.set_ring_folder(row.name.as_str().into());
    window.set_folder_ring_tick(window.get_folder_ring_tick() + 1);
    tracing::debug!(folder = %row.name, position, "the folder ring moved");
}

/// What:
/// `list_move_of` maps a list action to the movement it asks for.
/// Why:
/// the classifier names actions and `list_focus` names movements.
fn list_move_of(action: ShortcutAction) -> Option<ListMove> {
    return match action {
        ShortcutAction::ListUp => Some(ListMove::Up),
        ShortcutAction::ListDown => Some(ListMove::Down),
        ShortcutAction::ListHome => Some(ListMove::Home),
        ShortcutAction::ListEnd => Some(ListMove::End),
        ShortcutAction::ListPageUp => Some(ListMove::PageUp),
        ShortcutAction::ListPageDown => Some(ListMove::PageDown),
        _ => None,
    };
}

/// What:
/// `move_track_ring` moves the track ring and reports that the key was used.
/// Why:
/// a list key that finds no row (an empty list) is still used,
/// so it does not fall through to the list's scroll.
fn move_track_ring(
    window: &AppWindow,
    movement: ListMove,
) -> bool {
    let (rows, ring) = {
        let state = read_screen();
        (state.shown_rows.clone(), state.ring_track)
    };
    let page = usize::try_from(window.get_track_page_rows()).unwrap_or(1);
    let position = track_position_of(&rows, ring);
    if let Some(target) = move_focus(&track_selectable(&rows), position, movement, page) {
        place_track_ring(window, &rows, target);
    }
    return true;
}

/// What:
/// `move_folder_ring` moves the folder ring and reports that the key was used.
/// Why:
/// the same reasons as `move_track_ring`.
fn move_folder_ring(
    window: &AppWindow,
    movement: ListMove,
) -> bool {
    let (names, ring) = {
        let state = read_screen();
        (state.shown_names.clone(), state.ring_folder.clone())
    };
    let page = usize::try_from(window.get_folder_page_rows()).unwrap_or(1);
    let position = folder_position_of(&names, ring.as_deref());
    if let Some(target) = move_focus(&folder_selectable(&names), position, movement, page) {
        place_folder_ring(window, &names, target);
    }
    return true;
}

/// What:
/// `jump_to_letter` moves the folder ring to the first folder that starts with `letter`.
/// Why:
/// D106 gives the folder list a type-to-jump key;
/// a letter with no folder changes nothing.
fn jump_to_letter(
    window: &AppWindow,
    letter: char,
) -> bool {
    let names = read_screen().shown_names.clone();
    let mut labels: Vec<Option<&str>> = Vec::with_capacity(names.len());
    for row in &names {
        labels.push(if row.section_start {
            None
        } else {
            Some(row.name.as_str())
        });
    }
    match first_starting_with(&labels, letter) {
        Some(position) => place_folder_ring(window, &names, position),
        None => tracing::debug!(%letter, "no folder starts with the typed letter"),
    }
    return true;
}

/// What:
/// `activate_track` selects the row under the track ring,
/// or toggles play and pause when it is the selected row.
/// Why:
/// Enter on a list does what a click does (D124,
/// D129):
/// selecting starts playback only with Auto-play on
/// select on,
/// and otherwise the second Enter does.
fn activate_track(
    window: &AppWindow,
    host: &dyn Host,
) -> bool {
    let ring_track = read_screen().ring_track;
    let ring_index: Option<usize> = match ring_track {
        Some(ring) => usize::try_from(ring).ok(),
        None => None,
    };
    let Some(index) = ring_index else {
        return true;
    };
    crate::ui_wire::mark_user_acted();
    crate::ui_wire::play_row(host, index);
    project(window);
    return true;
}

/// What:
/// `activate_folder` shows the folder under the folder ring.
/// Why:
/// Enter on the picker opens the folder like a click;
/// on the folded layout the picker closes,
/// so focus moves to the
/// track list it uncovers.
fn activate_folder(
    window: &AppWindow,
    host: &dyn Host,
) -> bool {
    let Some(name) = read_screen().ring_folder.clone() else {
        return true;
    };
    crate::ui_wire::mark_user_acted();
    crate::ui_wire::select_folder(host, &name);
    if !window.get_unfolded() {
        crate::ui_keys_focus::request_focus(window, crate::ui_keys_focus::FocusTarget::Tracks);
    }
    project(window);
    return true;
}

/// What:
/// `apply` runs one list action on the list that holds focus;
/// returns true when the key was used.
/// Why:
/// the classifier already decided the action and the zone,
/// so this only dispatches by zone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function apply(win: AppWindow, host: Host, zone: FocusZone, action: ShortcutAction): boolean { ... }
/// ```
pub(crate) fn apply(
    window: &AppWindow,
    host: &dyn Host,
    zone: FocusZone,
    action: ShortcutAction,
) -> bool {
    let tracks = zone == FocusZone::TrackList;
    if let Some(movement) = list_move_of(action) {
        return if tracks {
            move_track_ring(window, movement)
        } else {
            move_folder_ring(window, movement)
        };
    }
    return match action {
        ShortcutAction::ListActivate if tracks => activate_track(window, host),
        ShortcutAction::ListActivate => activate_folder(window, host),
        ShortcutAction::ListLetter(letter) => jump_to_letter(window, letter),
        _ => false,
    };
}

/// What:
/// `list_focused` starts the ring when a list gains focus and has none that is still on a row.
/// Why:
/// Tab onto a list must show where the keys will act:
/// the playing track,
/// or the current folder,
/// or the first entry.
pub(crate) fn list_focused(
    window: &AppWindow,
    zone: i32,
) {
    if FocusZone::from_int(zone) == FocusZone::TrackList {
        let (rows, ring, playing) = {
            let state = read_screen();
            (
                state.shown_rows.clone(),
                state.ring_track,
                state.current_index,
            )
        };
        // A ring that is already on a row is placed again, which scrolls the row into view: Tab back onto a list whose ring
        // was left out of view must show the ring (LS2).
        if let Some(position) = track_position_of(&rows, ring) {
            place_track_ring(window, &rows, position);
            return;
        }
        let preferred = match playing {
            Some(current) => match i32::try_from(current) {
                Ok(ring_value) => track_position_of(&rows, Some(ring_value)),
                Err(_error) => None,
            },
            None => None,
        };
        if let Some(position) = start_focus(&track_selectable(&rows), preferred) {
            place_track_ring(window, &rows, position);
        }
    } else {
        let (names, ring, folder) = {
            let state = read_screen();
            (
                state.shown_names.clone(),
                state.ring_folder.clone(),
                state.shown_folder.clone(),
            )
        };
        if let Some(position) = folder_position_of(&names, ring.as_deref()) {
            place_folder_ring(window, &names, position);
            return;
        }
        let preferred = folder_position_of(&names, Some(folder.as_str()));
        if let Some(position) = start_focus(&folder_selectable(&names), preferred) {
            place_folder_ring(window, &names, position);
        }
    }
}

/// What:
/// `sync_rings` drops a ring whose row left the list,
/// pushes the rings to the window,
/// and starts a ring on a list
/// that holds focus without one.
/// Why:
/// a projection replaces the rows;
/// the ring is kept by library index and folder name,
/// and must not point at a
/// row that no longer exists.
pub(crate) fn sync_rings(window: &AppWindow) {
    {
        let mut state = write_screen();
        if track_position_of(&state.shown_rows, state.ring_track).is_none() {
            state.ring_track = None;
        }
        if folder_position_of(&state.shown_names, state.ring_folder.as_deref()).is_none() {
            state.ring_folder = None;
        }
    }
    let (track, folder) = {
        let state = read_screen();
        (state.ring_track, state.ring_folder.clone())
    };
    window.set_ring_track(track.unwrap_or(-1));
    window.set_ring_folder(folder.clone().unwrap_or_default().as_str().into());
    let zone = FocusZone::from_int(window.global::<Focus>().get_zone());
    if (zone == FocusZone::TrackList && track.is_none())
        || (zone == FocusZone::FolderList && folder.is_none())
    {
        list_focused(window, if zone == FocusZone::TrackList { 2 } else { 1 });
    }
}
