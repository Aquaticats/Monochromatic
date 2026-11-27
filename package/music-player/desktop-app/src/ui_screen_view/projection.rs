//! The projection of the library screen's state into the window's view model:
//!  one `view` call per redraw,
//!  and the deck title it derives from the playing track's siblings.
//!
//! What:
//! `view` builds the rows,
//!  the picker and the deck values from one `ScreenState`,
//!  through the shared `build_player_content` result.
//! Why:
//! the state and the rules that decide it are one role (in `ui_screen_view.rs`),
//!  and turning that
//! state into models is another,
//!  so the projection lives here and the owner module re-exports `view`.

/// What:
///      `use music_player::page_index::PageIndex;`.
///  The index that maps a queue position to its page.
/// Why:
///       The deck label and the title both ask which page holds a track.
///       The siblings are a bare `Vec<PageEntry>` (no lookup) and `ScreenState`'s own cache of pages.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { PageIndex } from "music-player/page_index";
/// ```
use music_player::page_index::PageIndex;
/// What:
///      `use music_player::pagination::PageEntry;`.
///  One queue entry as a page sees it:
///  its library index and its display name.
/// Why:
///       The projection hands the page's entries to the shared content builder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { PageEntry } from "music-player/pagination";
/// ```
use music_player::pagination::PageEntry;
/// What:
///      `use music_player::player_content::{PlayerContentInput, build_player_content};`.
///  The deck and row builder the player,
///  the search page and this projection share.
/// Why:
///       One builder keeps the deck's wording identical wherever it is drawn.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { buildPlayerContent } from "music-player/player_content";
/// ```
use music_player::player_content::{
    PlayerContentInput,
    build_player_content,
};
/// What:
///      `use music_player::template_model::DEFAULT_PLAYING_TEMPLATE;`.
///  The playing-track template used when the saved text says nothing.
/// Why:
///       The deck subtitle is evaluated from that template.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DEFAULT_PLAYING_TEMPLATE } from "music-player/template_model";
/// ```
use music_player::template_model::DEFAULT_PLAYING_TEMPLATE;
/// What:
///      `use music_player::title_prefix::{display_title, shared_prefix_len};`.
///  The title shortening that drops the prefix a folder's files share.
/// Why:
///       A deck title that repeats its folder reads as noise (LS3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { displayTitle, sharedPrefixLen } from "music-player/title_prefix";
/// ```
use music_player::title_prefix::{
    display_title,
    shared_prefix_len,
};
/// What:
///      `use music_player::track_entries::{deck_folder_label, track_title_for};`.
///  The deck's folder label and the title of one track's file name.
/// Why:
///       The deck heading and the title shortening read them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { deckFolderLabel, trackTitleFor } from "music-player/track_entries";
/// ```
use music_player::track_entries::{
    deck_folder_label,
    track_title_for,
};
/// What:
///      `use music_player::track_measure::lookup_for_positions;`.
///  The length and peak of each path on the page.
/// Why:
///       The rows show them and the deck subtitle may read one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { lookupForPositions } from "music-player/track_measure";
/// ```
use music_player::track_measure::lookup_for_positions;

/// What:
///      `use super::{ScreenState, ScreenView};`.
///  The state this module projects,
///  and the view it produces.
/// Why:
///       One projection of one record into one record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { ScreenState, ScreenView } from "./ui_screen_view";
/// ```
use super::{
    ScreenState,
    ScreenView,
};
/// What:
///      `use crate::ui_screen_rows::{list_rows, mark_unplayable, picker, strip_shared_prefixes};`.
///  The row and picker builders,
///  the unplayable marking and the shared-prefix stripping.
/// Why:
///       Rows and picker names come from the content builder's output through these.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { listRows, markUnplayable, picker, stripSharedPrefixes } from "./ui_screen_rows";
/// ```
use crate::ui_screen_rows::{
    list_rows,
    mark_unplayable,
    picker,
    strip_shared_prefixes,
};
/// What:
///      `use crate::ui_templates::{playing_subtitle, playing_track};`.
///  The subtitle evaluation over the playing track.
/// Why:
///       The deck subtitle is a template's answer,
///  not a hard-coded string (D112).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { playingSubtitle, playingTrack } from "./ui_templates";
/// ```
use crate::ui_templates::{
    playing_subtitle,
    playing_track,
};
/// What:
/// `playing_page_label` gives the label of the page that holds queue index `index`,
/// or empty when none does.
/// Why:
/// a root-level track has no folder of its own,
/// so the deck falls back to the label of the page it is listed under.
fn playing_page_label(
    pages: &PageIndex,
    index: Option<usize>,
) -> String {
    let position: Option<usize> = match index {
        Some(position) => pages.page_of_index(position),
        None => None,
    };
    let group = match position {
        Some(page) => pages.page(page),
        None => None,
    };
    return match group {
        Some(page) => page.label.clone(),
        None => String::new(),
    };
}

/// What:
/// `parent_of` is the directory part of a display name,
/// or empty for a loose file.
/// Why:
/// tracks of one directory share it,
/// and only they can share a filename prefix.
fn parent_of(name: &str) -> &str {
    return match name.rsplit_once('/') {
        Some((parent, _file)) => parent,
        None => "",
    };
}

/// What:
/// `deck_title_of` is the title the deck and the window title show for the playing track:
/// its file name without
/// the extension and without the prefix its directory's files share (D86),
/// or without the directory's own name.
/// Why:
/// the deck heading reads `folder / title`;
/// with the whole file name it repeated the folder
/// (`かめりあ(Camellia) / かめりあ(Camellia) - Aegis`,
/// the polish review finding LS3).
/// The directory is the playing
/// track's own,
/// which need not be the folder on screen,
/// so its files are found through the page index.
/// In TS you'd write (pseudocode):
/// ```ts
/// const deckTitleOf = (s: ScreenState) => displayTitle(stem(s.trackName), sharedPrefixLen(siblingStems(s)), folderName(s));
/// ```
fn deck_title_of(
    state: &ScreenState,
    pages: &PageIndex,
) -> String {
    if state.track_name.is_empty() {
        return String::new();
    }
    let stem: String = track_title_for(&state.track_name);
    let page_position: Option<usize> = match state.current_index {
        Some(index) => pages.page_of_index(index),
        None => None,
    };
    let playing_page = match page_position {
        Some(position) => pages.page(position),
        None => None,
    };
    let Some(page) = playing_page else {
        return stem;
    };
    let parent: &str = parent_of(&state.track_name);
    let mut siblings: Vec<String> = Vec::new();
    for index in page.indices() {
        if let Some(name) = state.names.get(*index as usize)
            && parent_of(name) == parent
        {
            siblings.push(track_title_for(name));
        }
    }
    let borrowed: Vec<&str> = siblings.iter().map(String::as_str).collect();
    let prefix: usize = shared_prefix_len(&borrowed);
    let folder: &str = if parent.is_empty() {
        ""
    } else {
        music_player::display_path::last_segment(parent)
    };
    return display_title(&stem, prefix, folder).to_string();
}

/// What:
/// `view` projects the state into the window's models using the supporting-line function.
/// Why:
/// rows,
/// picker and deck all come from the one shared `build_player_content` result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const view = (state: ScreenState, supportingLine: (t: TrackFields) => string | null): ScreenView => { ... };
/// ```
pub fn view(
    state: &ScreenState,
    supporting_line: &dyn music_player::track_supporting_line::SupportingLine,
) -> ScreenView {
    let all_pages = state.current_pages();
    let mut labels: Vec<String> = Vec::new();
    for page in all_pages.pages() {
        labels.push(page.label.clone());
    }
    let selected = state.selected_page_position();
    let page_items: Vec<PageEntry> = match all_pages.page(selected) {
        Some(page) => {
            let mut items: Vec<PageEntry> = Vec::with_capacity(page.indices().len());
            for index in page.indices() {
                items.push(PageEntry {
                    index: *index as usize,
                    name: state
                        .names
                        .get(*index as usize)
                        .cloned()
                        .unwrap_or_default(),
                });
            }
            items
        }
        None => Vec::new(),
    };
    let mut playing_position: Option<usize> = None;
    for (position, item) in page_items.iter().enumerate() {
        if Some(item.index) == state.current_index {
            playing_position = Some(position);
            break;
        }
    }
    let playing_count = page_items.len();
    let mut playing_name: Option<String> = None;
    if let Some(position) = playing_position
        && let Some(item) = page_items.get(position)
    {
        playing_name = Some(item.name.clone());
    }
    let mut positions: Vec<usize> = Vec::with_capacity(page_items.len());
    for item in &page_items {
        positions.push(item.index);
    }
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
    let strip_folder: &str = if matches!(all_pages.page(selected), Some(page) if page.identity.starts_with("root:"))
    {
        ""
    } else {
        content.folder_title.as_str()
    };
    let mut rows = list_rows(&content.entries);
    strip_shared_prefixes(&mut rows, strip_folder);
    mark_unplayable(&mut rows, &state.paths, &state.unplayable);
    let page_key = match all_pages.page(selected) {
        Some(page) => page.identity.clone(),
        None => String::new(),
    };
    let picker_model = picker(&content.sections, &content.folder_title);
    let deck_subtitle = match (playing_position, playing_name) {
        (Some(position), Some(name)) => {
            let template = state
                .playing_template
                .as_deref()
                .unwrap_or(DEFAULT_PLAYING_TEMPLATE);
            match playing_subtitle(
                template,
                &playing_track(
                    &name,
                    position + 1,
                    playing_count,
                    input.measures.get(&name),
                ),
            ) {
                Some(subtitle) => subtitle,
                None => content.deck.subtitle.clone(),
            }
        }
        _ => content.deck.subtitle.clone(),
    };
    let deck_title = deck_title_of(state, &all_pages);
    let deck_folder = deck_folder_label(
        &state.track_name,
        &playing_page_label(&all_pages, state.current_index),
    );
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
