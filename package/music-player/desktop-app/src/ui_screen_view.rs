//! Pure library screen state and its projection into the window's view model.
//!
//! What: `ScreenState` holds the facts the engine has reported, and `view` turns them into the rows,
//! picker and deck values the window shows.
//! Why: the page rules (which folder is selected, which tracks the engine scopes to) are decided
//! here without a Slint window, so they can be tested directly.

use music_player::command::PlaybackMode;
use music_player::pagination::PageEntry;
use music_player::page_index::PageIndex;
use std::sync::Arc;
use music_player::player_content::{PlayerContentInput, build_player_content};
use music_player::template_model::TrackFields;
use music_player::track_entries::{deck_folder_label, track_title_for};
use music_player::track_measure::{TrackMeasure, lookup_for_positions};
use std::collections::HashMap;

use crate::ui_screen_rows::{ListRow, Picker, PickerName, list_rows, picker, strip_shared_prefixes, mark_unplayable};
use music_player::title_prefix::{display_title, shared_prefix_len};
use crate::ui_screen_scroll::ScrollRequest;
use crate::ui_templates::{playing_subtitle, playing_track};
use music_player::template_editor_state::TemplateEditorState;
use music_player::template_model::DEFAULT_PLAYING_TEMPLATE;

/// What: `Route` names the screen the window shows over the player.
/// Why: the window's `route` property is an integer, and the Rust side keeps the names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Route = "player" | "search" | "settings" | "templateEditor" | "licenses";
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Route {
    /// The player: folder list, track list and transport deck.
    #[default]
    Player,
    /// The search page drawn over the player.
    Search,
    /// The settings page.
    Settings,
    /// The template editor drawn over the settings page.
    TemplateEditor,
    /// The licenses page, opened by the Licenses row of Settings.
    Licenses,
}

/// Methods of `Route`.
impl Route {
    /// What: `to_int` gives the integer the window's `route` property uses.
    /// Why: Slint has no enum binding in this crate, so routes cross as 0 to 4.
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

/// What: `ScreenState` is the library and playback facts reported by the engine.
/// Why: one value holds everything the view needs, so a projection never reads the window back.
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
    /// Absolute file paths of the queue, in queue order.
    pub paths: Vec<std::path::PathBuf>,
    /// Display names of the queue, in queue order. Shared with the Search feeder; assign through `set_names`.
    pub names: Arc<Vec<String>>,
    /// Pages of `names`, built when the queue arrives.
    pub page_index: PageIndex,
    /// Number of queues received; the page index and the watched page are valid for one value of it.
    pub queue_generation: u64,
    /// Paths of the selected page, for deciding whether a measure report changes what is shown.
    pub watched_page: Option<crate::ui_screen_pages::WatchedPage>,
    /// Queue index of the playing track.
    pub current_index: Option<usize>,
    /// Identity of the selected folder page; empty selects the first page.
    pub selected_key: String,
    /// Whether the folder picker replaces the track list on the folded layout.
    pub picker_open: bool,
    /// Display name of the playing track, or empty when none is loaded.
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
    /// Search query of the current Search visit; a new visit starts empty (D66).
    pub search_query: String,
    /// What the Search page shows and the newest query sequence, kept between answers.
    pub search_ui: crate::ui_search_apply::SearchUi,
    /// Queue index of the row whose track menu is open, or None.
    pub menu_index: Option<usize>,
    /// Whether the engine has reported the library at least once.
    pub loaded: bool,
    /// Whether the engine is reading a folder right now: set when the walk starts, cleared when its queue arrives.
    pub reading: bool,
    /// Library paths that could not be opened for playback, which their rows mark.
    pub unplayable: std::collections::HashSet<std::path::PathBuf>,
    /// Saved track rows template text, or None for the default.
    pub track_rows_template: Option<String>,
    /// Saved playing track template text, or None for the default.
    pub playing_template: Option<String>,
    /// Auto-play on select (D129): whether a click, Enter or Search result on another track also starts it.
    pub auto_play_on_select: bool,
    /// Text size setting: the pixel size of each of four texts, which the window draws them at.
    pub text_sizes: music_player::text_size::TextSizes,
    /// Template editor value while Settings or the editor is open.
    pub editor: Option<TemplateEditorState>,
    /// Length and peak of each library path, as the engine has reported them.
    pub measures: HashMap<std::path::PathBuf, TrackMeasure>,
    /// Whether the engine has reported the opened folder, even when it reported that none is open.
    pub root_known: bool,
    /// Display name of the opened folder, or `None` when no folder is open.
    pub source_root_name: Option<String>,
    /// Scroll request set by a reveal action, used by the next projection and then cleared.
    pub scroll_request: Option<ScrollRequest>,
    /// Identity of the page shown by the last projection that had rows, or `None` before any such projection.
    pub last_page_key: Option<String>,
    /// Whether the one-time reveal of the restored playing track has happened or the user has already acted, so a
    /// later change of the playing track never scrolls the list.
    pub startup_reveal_done: bool,
    /// Rows of the track list as the last projection showed them, for the list keys.
    pub shown_rows: Vec<ListRow>,
    /// Name rows of the folder picker as the last projection showed them, for the list keys.
    pub shown_names: Vec<PickerName>,
    /// Title of the folder the last projection showed, which the folder ring starts on.
    pub shown_folder: String,
    /// Library index of the track row with the keyboard ring.
    pub ring_track: Option<i32>,
    /// Name of the folder with the keyboard ring.
    pub ring_folder: Option<String>,
    /// Highlighted row of the open track menu, which the keyboard moves.
    pub menu_ring: Option<usize>,
    /// True when the next track menu was asked for from the keyboard, so it opens with its first row highlighted.
    pub menu_by_key: bool,
    /// Highlighted result of the Search page, which the keyboard moves.
    pub search_ring: Option<usize>,
}

/// Methods of `ScreenState`.
impl ScreenState {
    /// What: `library_heading` names the opened folder for the heading above the unfolded folder pane.
    /// Why: the pane is headed by the folder it lists; before any folder is open, or when the engine reports none,
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

    /// What: `deck_disabled` tells whether the transport deck is switched off.
    /// Why: with no track to play (a folder being read, or a library without audio) the seek bar, the transport and
    /// the mode group would look live and do nothing (the polish review finding EM2).
    pub fn deck_disabled(&self) -> bool {
        return self.reading || self.names.is_empty();
    }

    /// What: `menu_target` gives the open track menu's queue index, display name and absolute path.
    /// Why: a menu action runs on the row the menu was opened for, and that row can leave the queue meanwhile.
    pub fn menu_target(&self) -> Option<(usize, String, std::path::PathBuf)> {
        let index = self.menu_index?;
        let name = self.names.get(index)?.clone();
        let path = self.paths.get(index)?.clone();
        return Some((index, name, path));
    }
}

/// What: `ScreenView` is every value the window's folder, list and deck views show.
/// Why: the window receives one value per projection, so its models are set together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScreenView = { folderTitle: string; rows: ListRow[]; pageKey: string; picker: Picker; ... };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenView {
    /// Title of the selected folder page.
    pub folder_title: String,
    /// Rows of the selected folder page, headers included.
    pub rows: Vec<ListRow>,
    /// Identity of the selected folder page, or empty when the queue has no pages.
    pub page_key: String,
    /// Folder picker rail and names.
    pub picker: Picker,
    /// Title of the playing track on the deck, or empty.
    pub deck_title: String,
    /// Deck subtitle, such as "3 of 12", or empty.
    pub deck_subtitle: String,
    /// Folder label the shuffle-from-folder choice names.
    pub shuffle_folder_label: String,
    /// Folder that directly holds the playing track, or empty when nothing plays.
    pub deck_folder: String,
}

/// What: `playing_page_label` gives the label of the page that holds queue index `index`, or empty when none does.
/// Why: a root-level track has no folder of its own, so the deck falls back to the label of the page it is listed under.
fn playing_page_label(pages: &PageIndex, index: Option<usize>) -> String {
    return index
        .and_then(|position| return pages.page_of_index(position))
        .and_then(|page| return pages.page(page))
        .map_or_else(String::new, |page| return page.label.clone());
}

/// What: `parent_of` is the directory part of a display name, or empty for a loose file.
/// Why: tracks of one directory share it, and only they can share a filename prefix.
fn parent_of(name: &str) -> &str {
    return name.rsplit_once('/').map_or("", |(parent, _file)| return parent);
}

/// What: `deck_title_of` is the title the deck and the window title show for the playing track: its file name without
/// the extension and without the prefix its directory's files share (D86), or without the directory's own name.
/// Why: the deck heading reads `folder / title`; with the whole file name it repeated the folder
/// (`かめりあ(Camellia) / かめりあ(Camellia) - Aegis`, the polish review finding LS3). The directory is the playing
/// track's own, which need not be the folder on screen, so its files are found through the page index.
/// In TS you'd write (pseudocode):
/// ```ts
/// const deckTitleOf = (s: ScreenState) => displayTitle(stem(s.trackName), sharedPrefixLen(siblingStems(s)), folderName(s));
/// ```
fn deck_title_of(state: &ScreenState, pages: &PageIndex) -> String {
    if state.track_name.is_empty() {
        return String::new();
    }
    let stem: String = track_title_for(&state.track_name);
    let Some(page) = state
        .current_index
        .and_then(|index| return pages.page_of_index(index))
        .and_then(|position| return pages.page(position))
    else {
        return stem;
    };
    let parent: &str = parent_of(&state.track_name);
    let siblings: Vec<String> = page
        .indices()
        .iter()
        .filter_map(|index| return state.names.get(*index as usize))
        .filter(|name| return parent_of(name) == parent)
        .map(|name| return track_title_for(name))
        .collect();
    let borrowed: Vec<&str> = siblings.iter().map(String::as_str).collect();
    let prefix: usize = shared_prefix_len(&borrowed);
    let folder: &str = if parent.is_empty() { "" } else { parent.rsplit_once('/').map_or(parent, |(_up, name)| return name) };
    return display_title(&stem, prefix, folder).to_string();
}

/// What: `view` projects the state into the window's models using the supporting-line function.
/// Why: rows, picker and deck all come from the one shared `build_player_content` result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const view = (state: ScreenState, supportingLine: (t: TrackFields) => string | null): ScreenView => { ... };
/// ```
pub fn view(state: &ScreenState, supporting_line: &dyn Fn(&TrackFields) -> Option<String>) -> ScreenView {
    let all_pages = state.current_pages();
    let labels: Vec<String> = all_pages.pages().map(|page| return page.label.clone()).collect();
    let selected = state.selected_page_position();
    let page_items: Vec<PageEntry> = all_pages.page(selected).map_or_else(Vec::new, |page| {
        return page
            .indices()
            .iter()
            .map(|index| return PageEntry {
                index: *index as usize,
                name: state.names.get(*index as usize).cloned().unwrap_or_default(),
            })
            .collect();
    });
    let playing_position = page_items.iter().position(|item| return Some(item.index) == state.current_index);
    let playing_count = page_items.len();
    let playing_name = playing_position.and_then(|position| return page_items.get(position)).map(|item| return item.name.clone());
    let positions: Vec<usize> = page_items.iter().map(|item| return item.index).collect();
    let measures = lookup_for_positions(&positions, &state.names, &state.paths, &state.measures);
    let input = PlayerContentInput {
        page_labels: labels,
        selected_page: selected,
        page_items,
        current_index: state.current_index,
        playing: state.playing,
        mode: state.mode,
        position_sec: state.position,
        duration_sec: state.duration,
        measures,
    };
    let content = build_player_content(&input, supporting_line);
    // A virtual letter folder (loose files) is not a directory, so its name is never stripped from a title.
    let strip_folder: &str = if all_pages.page(selected).is_some_and(|page| return page.identity.starts_with("root:")) { "" } else { content.folder_title.as_str() };
    let mut rows = list_rows(&content.entries);
    strip_shared_prefixes(&mut rows, strip_folder);
    mark_unplayable(&mut rows, &state.paths, &state.unplayable);
    let page_key = all_pages.page(selected).map_or_else(String::new, |page| return page.identity.clone());
    let picker_model = picker(&content.sections, &content.folder_title);
    let deck_subtitle = match (playing_position, playing_name) {
        (Some(position), Some(name)) => {
            let template = state.playing_template.as_deref().unwrap_or(DEFAULT_PLAYING_TEMPLATE);
            playing_subtitle(template, &playing_track(&name, position + 1, playing_count, input.measures.get(&name)))
                .unwrap_or_else(|| return content.deck.subtitle.clone())
        }
        _ => content.deck.subtitle.clone(),
    };
    let deck_title = deck_title_of(state, &all_pages);
    let deck_folder = deck_folder_label(&state.track_name, &playing_page_label(&all_pages, state.current_index));
    return ScreenView {
        folder_title: content.folder_title,
        rows,
        page_key,
        picker: picker_model,
        deck_title,
        deck_subtitle,
        shuffle_folder_label: content.deck.shuffle_folder_label,
        deck_folder,
    };
}

/// Page selection helpers live in `ui_screen_pages.rs` and keep their old paths here.
pub use crate::ui_screen_pages::{select_folder_label, selected_page_scope};

#[cfg(test)]
#[path = "ui_screen_view_tests.rs"]
mod tests;
