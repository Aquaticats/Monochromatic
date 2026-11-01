//! Pure builder for the player's content:
//!  the rail,
//!  the folder's rows,
//!  and the deck text.
//!
//! Ported from `core/PlayerContentModel.kt`.
//!  The supporting line is a parameter,
//!  so the template
//! engine stays outside this module.
//!  The Kotlin deck title comes from `trackTitleFor` and not
//! from a template,
//!  so the builder has no playing-line parameter.

/// Folder list types and the grouping function that builds one page's folder.
use crate::folder_index::{FolderEntry, FolderListItem, FolderTrackRow, folder_index};
/// Rail section type and the builder that derives the rail from the page labels.
use crate::folder_rail::{RailSection, rail_for};
/// Playback mode,
///  carried through to the deck unchanged.
use crate::command::PlaybackMode;
/// One track of the displayed page,
///  with its queue position.
use crate::pagination::PageEntry;
/// One track with its URI and display path.
use crate::track::Track;
/// Track row types and the helpers that build rows and titles.
use crate::track_entries::{TrackListEntry, TrackRowModel, track_list_entries, track_title_for};
/// Template fields that the supporting-line function reads.
use crate::template_model::TrackFields;
use crate::track_measure::TrackMeasure;
use std::collections::HashMap;

/// What:
///      `pub struct TransportDeckModel`.
///  The playback snapshot the transport deck draws.
/// Why:
///       The builder fills the title,
///  subtitle and shuffle label,
///  and passes the playback values through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TransportDeckModel = { title: string; subtitle: string; positionSec: number; durationSec: number;
///   playing: boolean; mode: PlaybackMode; shuffleFolderLabel: string };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct TransportDeckModel {
    /// Title of the playing track.
    pub title: String,
    /// Preformatted line under the title,
    ///  such as the row position.
    pub subtitle: String,
    /// Current playhead position in seconds.
    pub position_sec: f64,
    /// Track duration in seconds.
    pub duration_sec: f64,
    /// Whether playback is running.
    pub playing: bool,
    /// Active playback mode.
    pub mode: PlaybackMode,
    /// Folder label used by the shuffle controls.
    pub shuffle_folder_label: String,
}

/// What:
///      `pub struct PlayerContentInput`.
///  The playback and page values that `build_player_content` reads.
/// Why:
///       The builder takes eight values,
///  so they travel as one named object.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlayerContentInput = { pageLabels: string[]; selectedPage: number; pageItems: PageEntry[];
///   currentIndex: number | null; playing: boolean; mode: PlaybackMode; positionSec: number; durationSec: number };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerContentInput {
    /// Folder labels of the pages,
    ///  one per page.
    pub page_labels: Vec<String>,
    /// Index into `page_labels` of the page on screen.
    pub selected_page: usize,
    /// Tracks of the displayed page,
    ///  in playback order.
    pub page_items: Vec<PageEntry>,
    /// Queue index of the playing track,
    ///  or `None` when nothing plays.
    pub current_index: Option<usize>,
    /// Whether playback is running.
    pub playing: bool,
    /// Active playback mode.
    pub mode: PlaybackMode,
    /// Current playhead position in seconds.
    pub position_sec: f64,
    /// Track duration in seconds.
    pub duration_sec: f64,
    /// Length and exact peak of each track, keyed by its display name.
    pub measures: HashMap<String, TrackMeasure>,
}

/// What:
///      `pub struct PlayerContentModel`.
///  Everything the player screen draws for one state.
/// Why:
///       One value carries the rail,
///  the folder rows,
///  and the deck,
///  so the screen updates in one step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlayerContentModel = { sections: RailSection[]; currentFolder: string | null;
///   folderTitle: string; entries: TrackListEntry[]; deck: TransportDeckModel };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerContentModel {
    /// Rail sections for the folder picker.
    pub sections: Vec<RailSection>,
    /// Label of the selected page,
    ///  or `None` when the selected index is out of range.
    pub current_folder: Option<String>,
    /// Label of the selected page,
    ///  or empty when there is none.
    pub folder_title: String,
    /// Rows and headers of the selected folder.
    pub entries: Vec<TrackListEntry>,
    /// Playback snapshot for the transport deck.
    pub deck: TransportDeckModel,
}

/// What:
///      `fn row_entry(entry: &TrackListEntry) -> Option<&TrackRowModel>`.
///  Returns the row of a row entry.
/// Why:
///       The row position counts track rows only,
///  so headers are skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rowEntry = (e: TrackListEntry) => (e.kind === "row" ? e.row : undefined);
/// ```
fn row_entry(entry: &TrackListEntry) -> Option<&TrackRowModel> {
    return match entry {
        TrackListEntry::RowEntry(row) => Some(row),
        TrackListEntry::HeaderEntry { .. } => None,
    };
}

/// What:
///      `pub fn build_player_content(input, supporting_line) -> PlayerContentModel`.
///  The pure builder
///  for one player state.
/// Why:
///       The rail,
///  folder,
///  rows and deck are all derived from the same inputs,
///  so they stay in step.
///  The supporting line is passed in so that the template engine is not called here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function buildPlayerContent(input: PlayerContentInput, supportingLine: (f: TrackFields) => string | null): PlayerContentModel { /* ... */ }
/// ```
pub fn build_player_content(input: &PlayerContentInput, supporting_line: &dyn Fn(&TrackFields) -> Option<String>) -> PlayerContentModel {
    let sections: Vec<RailSection> = rail_for(&input.page_labels);
    let current_folder: Option<String> = input.page_labels.get(input.selected_page).cloned();
    let folder_title: String = current_folder.clone().unwrap_or_default();
    let folder: FolderEntry = page_folder_for(&folder_title, &input.page_items);
    let entries: Vec<TrackListEntry> = track_list_entries(&folder, input.current_index, &input.measures, supporting_line);
    let row_position: Option<usize> = entries.iter().filter_map(row_entry).position(|row| return row.is_current);
    let playing_title: String = input
        .page_items
        .iter()
        .find(|item| return Some(item.index) == input.current_index)
        .map(|item| return track_title_for(&item.name))
        .unwrap_or_default();
    let deck = TransportDeckModel {
        title: playing_title,
        subtitle: subtitle_for(row_position, input.page_items.len()),
        position_sec: input.position_sec,
        duration_sec: input.duration_sec,
        playing: input.playing,
        mode: input.mode,
        shuffle_folder_label: folder_title.clone(),
    };
    return PlayerContentModel {
        sections,
        current_folder,
        folder_title,
        entries,
        deck,
    };
}

/// What:
///      `pub fn page_folder_for(folder_title, page_items) -> FolderEntry`.
///  Builds the folder list
///  of one page,
///  with each row's index set to its queue position.
/// Why:
///       The page's tracks go through the same folder layout as the library,
///  then their
///  positions are rewritten to queue indexes so playback addresses the right tracks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tracks = pageItems.map((item) => ({ uri: "", displayPath: item.name }));
/// ```
pub fn page_folder_for(folder_title: &str, page_items: &[PageEntry]) -> FolderEntry {
    let tracks: Vec<Track> = page_items
        .iter()
        .map(|item| return Track {
            uri: String::new(),
            display_path: item.name.clone(),
        })
        .collect();
    let items: Vec<FolderListItem> = folder_index(&tracks)
        .into_iter()
        .flat_map(|folder| return folder.items)
        .collect();
    return FolderEntry {
        name: folder_title.to_string(),
        items: with_load_indices(items, page_items),
        track_count: page_items.len(),
    };
}

/// What:
///      `pub fn with_load_indices(items, page_items) -> Vec<FolderListItem>`.
///  Rewrites the library
///  index of every track row to the queue position of the page item it came from.
/// Why:
///       Headers have no position,
///  so they pass through unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// items.map((item) => item.kind === "track" ? { ...item, row: { ...item.row, indexInLibrary: pageItems[item.row.indexInLibrary].index } } : item);
/// ```
pub fn with_load_indices(items: Vec<FolderListItem>, page_items: &[PageEntry]) -> Vec<FolderListItem> {
    return items
        .into_iter()
        .map(|item| return with_load_index(item, page_items))
        .collect();
}

/// What:
///      `fn with_load_index(item, page_items) -> FolderListItem`.
///  Rewrites one track row's index.
/// Why:
///       Only track rows carry a position,
///  so headers come back as they went in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withLoadIndex = (item, pageItems) => item.kind === "track" ? { kind: "track", row: { track: item.row.track, indexInLibrary: pageItems[item.row.indexInLibrary].index } } : item;
/// ```
fn with_load_index(item: FolderListItem, page_items: &[PageEntry]) -> FolderListItem {
    return match item {
        FolderListItem::TrackItem(row) => FolderListItem::TrackItem(FolderTrackRow {
            index_in_library: page_items[row.index_in_library].index,
            track: row.track,
        }),
        header => header,
    };
}

/// What:
///      `pub fn subtitle_for(row_position, page_track_count) -> String`.
///  The "N of M" line under
///  the title,
///  or empty when no row plays.
/// Why:
///       `row_position` is `None` when no row of the folder is playing.
///  The Kotlin used `-1` for that,
///  and the `Option` makes the case explicit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const subtitleFor = (pos: number | null, total: number) => (pos === null ? "" : `${pos + 1} of ${total}`);
/// ```
pub fn subtitle_for(row_position: Option<usize>, page_track_count: usize) -> String {
    return match row_position {
        Some(position) => format!("{} of {page_track_count}", position + 1),
        None => String::new(),
    };
}

/// What:
///      `#[cfg(test)] #[path = "player_content_tests.rs"] mod tests;` declares a test-only submodule
///  whose code lives in the sibling file `player_content_tests.rs`.
/// Why:
///       Keep `player_content.rs` to production code,
///  with the tests beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // playerContent.unit.test.ts, run only by the test runner
/// ```
#[cfg(test)]
#[path = "player_content_tests.rs"]
mod tests;
