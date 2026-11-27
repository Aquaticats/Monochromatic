//! Pure library screen state and its projection into the window's view model.
//!
//! What:
//! `ScreenState` holds the facts the engine has reported,
//! and `view` turns them into the rows,
//! picker and deck values the window shows.
//! Why:
//! the page rules (which folder is selected,
//! which tracks the engine scopes to) are decided
//! here without a Slint window,
//! so they can be tested directly.
//! The projection itself lives in `ui_screen_view/projection.rs`,
//! which this module re-exports.

/// What:
/// `mod projection;` the `view` that turns the state into the window's models
/// (`src/ui_screen_view/projection.rs`).
/// Why:
/// the state and the rules that decide it are one role,
/// and building models from it is another,
/// so
/// each can be read on its own.
mod projection;

/// Re-exports the projection,
/// which the screen and the tests still call as `ui_screen_view::view`.
pub use self::projection::view;

/// Imports the playback mode that the view shows.
use music_player::command::PlaybackMode;
/// Imports the page index that the state caches.
use music_player::page_index::PageIndex;
/// Imports the track measure type.
use music_player::track_measure::TrackMeasure;
/// Imports the map type that holds measures by path.
use std::collections::HashMap;
/// Imports the shared ownership type of the name list that the state shares.
use std::sync::Arc;

/// Imports the row and picker types that the state caches.
use crate::ui_screen_rows::{
    ListRow,
    Picker,
    PickerName,
};
/// Imports the scroll request that a view may carry.
use crate::ui_screen_scroll::ScrollRequest;
/// Imports the template editor state that the settings pages keep.
use music_player::template_editor_state::TemplateEditorState;

/// What:
/// `Route` names the screen the window shows over the player.
/// Why:
/// the window's `route` property is an integer,
/// and the Rust side keeps the names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Route = "player" | "search" | "settings" | "templateEditor" | "licenses";
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Route {
    /// The player:
    /// folder list,
    /// track list and transport deck.
    #[default]
    Player,
    /// The search page drawn over the player.
    Search,
    /// The settings page.
    Settings,
    /// The template editor drawn over the settings page.
    TemplateEditor,
    /// The licenses page,
    /// opened by the Licenses row of Settings.
    Licenses,
}

/// Methods of `Route`.
impl Route {
    /// What:
    /// `to_int` gives the integer the window's `route` property uses.
    /// Why:
    /// Slint has no enum binding in this crate,
    /// so routes cross as 0 to 4.
    pub fn to_int(self) -> i32 {
        return match self {
            Route::Player => 0,
            Route::Search => 1,
            Route::Settings => 2,
            Route::TemplateEditor => 3,
            Route::Licenses => 4,
        };
    }
}

/// What:
/// `ScreenState` is the library and playback facts reported by the engine.
/// Why:
/// one value holds everything the view needs,
/// so a projection never reads the window back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScreenState = {
///   paths: string[]; names: string[]; currentIndex: number | null;
///   selectedKey: string; pickerOpen: boolean; trackName: string; mode: PlaybackMode;
///   playing: boolean; position: number; duration: number;
/// };
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScreenState {
    /// Absolute file paths of the queue,
    /// in queue order.
    pub paths: Vec<std::path::PathBuf>,
    /// Display names of the queue,
    /// in queue order.
    /// Shared with the Search feeder;
    /// assign through `set_names`.
    pub names: Arc<Vec<String>>,
    /// Pages of `names`,
    /// built when the queue arrives.
    pub page_index: PageIndex,
    /// Number of queues received;
    /// the page index and the watched page are valid for one value of it.
    pub queue_generation: u64,
    /// Paths of the selected page,
    /// for deciding whether a measure report changes what is shown.
    pub watched_page: Option<crate::ui_screen_pages::WatchedPage>,
    /// Queue index of the playing track.
    pub current_index: Option<usize>,
    /// Identity of the selected folder page;
    /// empty selects the first page.
    pub selected_key: String,
    /// Whether the folder picker replaces the track list on the folded layout.
    pub picker_open: bool,
    /// Display name of the playing track,
    /// or empty when none is loaded.
    pub track_name: String,
    /// Playback mode from the engine.
    pub mode: PlaybackMode,
    /// Whether audio is playing.
    pub playing: bool,
    /// Playback position in seconds.
    pub position: f64,
    /// Duration of the playing track in seconds.
    pub duration: f64,
    /// Screen shown over the player.
    pub route: Route,
    /// Search query of the current Search visit;
    /// a new visit starts empty (D66).
    pub search_query: String,
    /// What the Search page shows and the newest query sequence,
    /// kept between answers.
    pub search_ui: crate::ui_search_apply::SearchUi,
    /// Queue index of the row whose track menu is open,
    /// or None.
    pub menu_index: Option<usize>,
    /// Whether the engine has reported the library at least once.
    pub loaded: bool,
    /// Whether the engine is reading a folder right now:
    /// set when the walk starts,
    /// cleared when its queue arrives.
    pub reading: bool,
    /// Library paths that could not be opened for playback,
    /// which their rows mark.
    pub unplayable: std::collections::HashSet<std::path::PathBuf>,
    /// Saved track rows template text,
    /// or None for the default.
    pub track_rows_template: Option<String>,
    /// Saved playing track template text,
    /// or None for the default.
    pub playing_template: Option<String>,
    /// Auto-play on select (D129):
    /// whether a click,
    /// Enter or Search result on another track also starts it.
    pub auto_play_on_select: bool,
    /// Text size setting:
    /// the pixel size of each of four texts,
    /// which the window draws them at.
    pub text_sizes: music_player::text_size::TextSizes,
    /// Template editor value while Settings or the editor is open.
    pub editor: Option<TemplateEditorState>,
    /// Length and peak of each library path,
    /// as the engine has reported them.
    pub measures: HashMap<std::path::PathBuf, TrackMeasure>,
    /// Whether the engine has reported the opened folder,
    /// even when it reported that none is open.
    pub root_known: bool,
    /// Display name of the opened folder,
    /// or `None` when no folder is open.
    pub source_root_name: Option<String>,
    /// Scroll request set by a reveal action,
    /// used by the next projection and then cleared.
    pub scroll_request: Option<ScrollRequest>,
    /// Identity of the page shown by the last projection that had rows,
    /// or `None` before any such projection.
    pub last_page_key: Option<String>,
    /// Whether the one-time reveal of the restored playing track has happened or the user has already acted,
    /// so a
    /// later change of the playing track never scrolls the list.
    pub startup_reveal_done: bool,
    /// Rows of the track list as the last projection showed them,
    /// for the list keys.
    pub shown_rows: Vec<ListRow>,
    /// Name rows of the folder picker as the last projection showed them,
    /// for the list keys.
    pub shown_names: Vec<PickerName>,
    /// Title of the folder the last projection showed,
    /// which the folder ring starts on.
    pub shown_folder: String,
    /// Library index of the track row with the keyboard ring.
    pub ring_track: Option<i32>,
    /// Name of the folder with the keyboard ring.
    pub ring_folder: Option<String>,
    /// Highlighted row of the open track menu,
    /// which the keyboard moves.
    pub menu_ring: Option<usize>,
    /// True when the next track menu was asked for from the keyboard,
    /// so it opens with its first row highlighted.
    pub menu_by_key: bool,
    /// Highlighted result of the Search page,
    /// which the keyboard moves.
    pub search_ring: Option<usize>,
}

/// Methods of `ScreenState`.
impl ScreenState {
    /// What:
    /// `library_heading` names the opened folder for the heading above the unfolded folder pane.
    /// Why:
    /// the pane is headed by the folder it lists;
    /// before any folder is open,
    /// or when the engine reports none,
    /// the generic word "Folders" stands in so the heading is never blank.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const libraryHeading = (s: ScreenState): string => s.sourceRootName?.trim() || "Folders";
    /// ```
    pub fn library_heading(&self) -> String {
        return match self.source_root_name.as_deref().map(str::trim) {
            Some(name) if !name.is_empty() => name.to_string(),
            _ => "Folders".to_string(),
        };
    }

    /// What:
    /// `deck_disabled` tells whether the transport deck is switched off.
    /// Why:
    /// with no track to play (a folder being read,
    /// or a library without audio) the seek bar,
    /// the transport and
    /// the mode group would look live and do nothing (the polish review finding EM2).
    pub fn deck_disabled(&self) -> bool {
        return self.reading || self.names.is_empty();
    }

    /// What:
    /// `menu_target` gives the open track menu's queue index,
    /// display name and absolute path.
    /// Why:
    /// a menu action runs on the row the menu was opened for,
    /// and that row can leave the queue meanwhile.
    pub fn menu_target(&self) -> Option<(usize, String, std::path::PathBuf)> {
        let index = self.menu_index?;
        let name = self.names.get(index)?.clone();
        let path = self.paths.get(index)?.clone();
        return Some((index, name, path));
    }
}

/// What:
/// `ScreenView` is every value the window's folder,
/// list and deck views show.
/// Why:
/// the window receives one value per projection,
/// so its models are set together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScreenView = { folderTitle: string; rows: ListRow[]; pageKey: string; picker: Picker; ... };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenView {
    /// Title of the selected folder page.
    pub folder_title: String,
    /// Rows of the selected folder page,
    /// headers included.
    pub rows: Vec<ListRow>,
    /// Identity of the selected folder page,
    /// or empty when the queue has no pages.
    pub page_key: String,
    /// Folder picker rail and names.
    pub picker: Picker,
    /// Title of the playing track on the deck,
    /// or empty.
    pub deck_title: String,
    /// Deck subtitle,
    /// such as "3 of 12",
    /// or empty.
    pub deck_subtitle: String,
    /// Folder label the shuffle-from-folder choice names.
    pub shuffle_folder_label: String,
    /// Folder that directly holds the playing track,
    /// or empty when nothing plays.
    pub deck_folder: String,
}

/// Page selection helpers live in `ui_screen_pages.rs` and keep their old paths here.
pub use crate::ui_screen_pages::{
    select_folder_label,
    selected_page_scope,
};

/// Unit tests of `ui_screen_view`,
/// in the sidecar file `ui_screen_view_tests.rs`.
/// Role of the module:
/// pure library screen state and its projection into the window's view model.
#[cfg(test)]
#[path = "ui_screen_view_tests.rs"]
mod tests;
