//! The window's observable state as one log line, for the end-to-end run.
//!
//! What: `emit` writes one `ui_state` line with the page, the focus zone, the rings, the playback facts and the badges the
//! window shows now.
//! Why: the end-to-end run presses real keys through the nested compositor and needs to read what happened. The release
//! binary has no MCP server, so the state goes to the log, and each line is written where the effect lands: after a key is
//! used, when focus moves, when the badges change, and when the engine reports a position, a track, a mode or play state.

use std::cell::Cell;

use music_player::keymap::FocusZone;
use music_player::text_size::TextElement;
use slint::ComponentHandle;

use crate::ui_screen::read_state;
use crate::{AppWindow, Focus, FolderLayout, Keys, TrackLayout};

thread_local! {
    /// What: the number of state lines written so far.
    /// Why: a line's sequence number lets a reader tell a new line from one it already read.
    static SEQUENCE: Cell<u64> = const { Cell::new(0) };
}

/// What: `zone_name` names a focus zone for the log.
/// Why: the log is read by people and by the end-to-end run, and names read better than numbers.
fn zone_name(zone: FocusZone) -> &'static str {
    return match zone {
        FocusZone::Surface => "surface",
        FocusZone::FolderList => "folder-list",
        FocusZone::TrackList => "track-list",
        FocusZone::Seek => "seek",
        FocusZone::ModeGroup => "mode-group",
        FocusZone::DeckButton => "deck-button",
        FocusZone::Button => "button",
        FocusZone::TextField => "text-field",
        FocusZone::SettingsList => "settings-list",
    };
}

/// What: `badges_shown` lists the badge texts that are showing, as `name=text` pairs.
/// Why: the end-to-end run checks which badges appear after a hold and which disappear on release.
fn badges_shown(window: &AppWindow) -> Vec<String> {
    let badges = window.global::<Keys>().get_badges();
    let pairs: [(&str, &slint::SharedString); 18] = [
        ("search", &badges.search),
        ("settings", &badges.settings),
        ("open-folder", &badges.open_folder),
        ("folder-trigger", &badges.folder_trigger),
        ("folder-list", &badges.folder_list),
        ("track-list", &badges.track_list),
        ("previous", &badges.previous),
        ("next", &badges.next),
        ("seek-back", &badges.seek_back),
        ("seek-forward", &badges.seek_forward),
        ("mode-repeat", &badges.mode_repeat),
        ("mode-in-order", &badges.mode_in_order),
        ("mode-shuffle-folder", &badges.mode_shuffle_folder),
        ("mode-shuffle-all", &badges.mode_shuffle_all),
        ("mode-cycle", &badges.mode_cycle),
        ("deck-title", &badges.deck_title),
        ("deck-folder", &badges.deck_folder),
        ("editor-reset", &badges.editor_reset),
    ];
    return pairs
        .iter()
        .filter(|(_, text)| return !text.is_empty())
        .map(|(name, text)| return format!("{name}={text}"))
        .collect();
}

/// What: `menu_pixels` gives a menu length in whole window pixels, or 0 while no menu is open.
/// Why: the menu's geometry properties keep the last menu's values after it closes, and the log should show only an open menu's.
fn menu_pixels(window: &AppWindow, length: f32) -> i32 {
    if !window.get_menu_open() {
        return 0;
    }
    return length.round() as i32;
}

/// What: `StateSnapshot` is the window's observable state, serialized as one JSON object.
/// Why: a typed struct keeps the field names in one place and needs no macro that unwraps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StateSnapshot = { seq: number; reason: string; route: string; zone: string; /* ... */ };
/// ```
#[derive(serde::Serialize)]
struct StateSnapshot {
    /// Sequence number of this line.
    seq: u64,
    /// Why the line was written: `key`, `focus`, `position` and so on.
    reason: String,
    /// Page the window shows.
    route: String,
    /// Kind of element that holds keyboard focus.
    zone: &'static str,
    /// Name of the element that holds keyboard focus.
    focus: String,
    /// Whether the window is on the unfolded layout.
    unfolded: bool,
    /// Whether the folder picker overlay is open.
    picker_open: bool,
    /// Whether the track menu is open.
    menu_open: bool,
    /// Left edge of the open track menu in window pixels, 0 while no menu is open.
    menu_x: i32,
    /// Top edge of the open track menu in window pixels, 0 while no menu is open.
    menu_y: i32,
    /// Width of the open track menu in window pixels, 0 while no menu is open.
    menu_width: i32,
    /// Height of the open track menu in window pixels, 0 while no menu is open.
    menu_height: i32,
    /// Title of the folder the track list shows.
    folder: String,
    /// How far the track list on screen is scrolled down, in whole pixels (0 at the top).
    track_scroll: i32,
    /// How far the folder picker's names are scrolled down, in whole pixels (0 at the top).
    folder_scroll: i32,
    /// Library index of the track row with the keyboard ring.
    ring_track: Option<i32>,
    /// Title of that row.
    ring_track_title: Option<String>,
    /// Name of the folder with the keyboard ring.
    ring_folder: Option<String>,
    /// Highlighted row of the open track menu.
    menu_ring: Option<usize>,
    /// Highlighted result of the Search page.
    search_ring: Option<usize>,
    /// Text of the Search query.
    search_query: String,
    /// Queue index of the playing track.
    current_index: Option<usize>,
    /// Whether audio plays.
    playing: bool,
    /// Playback mode number the window shows.
    mode: i32,
    /// Playback position in seconds.
    position: f64,
    /// Duration of the playing track in seconds.
    duration: f64,
    /// Index of the selected Settings row.
    settings_selected: i32,
    /// Whether the focus ring is turned on.
    ring_visible: bool,
    /// The element a focus request named.
    focus_request: i32,
    /// The badges showing, as `name=text` pairs.
    badges: Vec<String>,
    /// Headline of the first-run pane, or empty while the pane is not shown (reading, empty or without audio).
    first_run_title: String,
    /// Whether the transport deck is switched off.
    deck_disabled: bool,
    /// Whether Auto-play on select is on (D129).
    auto_play_on_select: bool,
    /// Size in pixels of the pane titles, which the window draws them at.
    text_size_pane_title_px: u16,
    /// Size in pixels of the group heading letter.
    text_size_group_heading_px: u16,
    /// Size in pixels of the folder names.
    text_size_folder_name_px: u16,
    /// Size in pixels of the track titles.
    text_size_track_title_px: u16,
}

/// What: `emit` writes one `ui_state` line with the window's observable state as JSON.
/// Why: see the module description. The line starts with `ui_state ` so a reader can pick it out of the log.
pub(crate) fn emit(window: &AppWindow, reason: &str) {
    let sequence = SEQUENCE.with(|cell| {
        cell.set(cell.get() + 1);
        return cell.get();
    });
    let zone = FocusZone::from_int(window.global::<Focus>().get_zone());
    let badges = badges_shown(window);
    let snapshot = read_state(|state| {
        let ring_title = state
            .ring_track
            .and_then(|ring| return state.shown_rows.iter().find(|row| return row.kind == 0 && row.index == ring))
            .map(|row| return row.title.clone());
        return StateSnapshot {
            seq: sequence,
            reason: reason.to_owned(),
            route: format!("{:?}", state.route),
            zone: zone_name(zone),
            focus: window.global::<Focus>().get_label().to_string(),
            unfolded: window.get_unfolded(),
            picker_open: state.picker_open,
            menu_open: window.get_menu_open(),
            menu_x: menu_pixels(window, window.get_menu_card_x()),
            menu_y: menu_pixels(window, window.get_menu_card_y()),
            menu_width: menu_pixels(window, window.get_menu_card_width()),
            menu_height: menu_pixels(window, window.get_menu_card_height()),
            folder: state.shown_folder.clone(),
            track_scroll: (-window.global::<TrackLayout>().get_scroll_y()).round() as i32,
            folder_scroll: (-window.global::<FolderLayout>().get_scroll_y()).round() as i32,
            ring_track: state.ring_track,
            ring_track_title: ring_title,
            ring_folder: state.ring_folder.clone(),
            menu_ring: state.menu_ring,
            search_ring: state.search_ring,
            search_query: state.search_query.clone(),
            current_index: state.current_index,
            playing: state.playing,
            mode: window.get_playback_mode(),
            position: state.position,
            duration: state.duration,
            settings_selected: window.get_settings_selected(),
            ring_visible: window.global::<Focus>().get_ring_visible(),
            focus_request: window.get_focus_request(),
            badges,
            first_run_title: if window.get_first_run_state() == 0 { String::new() } else { window.get_first_run_title().to_string() },
            deck_disabled: window.global::<crate::DeckState>().get_disabled(),
            auto_play_on_select: state.auto_play_on_select,
            text_size_pane_title_px: state.text_sizes.get(TextElement::PaneTitle),
            text_size_group_heading_px: state.text_sizes.get(TextElement::GroupHeading),
            text_size_folder_name_px: state.text_sizes.get(TextElement::FolderName),
            text_size_track_title_px: state.text_sizes.get(TextElement::TrackTitle),
        };
    });
    let line = match serde_json::to_string(&snapshot) {
        Ok(text) => text,
        Err(error) => {
            tracing::warn!(?error, "the window state could not be written as a log line");
            return;
        }
    };
    if reason == "position" {
        // A position report arrives several times a second, so it is logged at debug level.
        tracing::debug!("ui_state {line}");
    } else {
        tracing::info!("ui_state {line}");
    }
}
