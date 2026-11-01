// What:     Unit tests for `player_content.rs`, pulled in by
//           `#[cfg(test)] #[path = "player_content_tests.rs"] mod tests;` at the bottom of
//           `player_content.rs`. Mirrors every `@Test` of `PlayerContentModelTest.kt`. The supporting
//           line is a stub that returns `None`, because these cases do not read it. The one deliberate
//           adaptation is `subtitleIsEmptyWhenNoRowIsPlaying`, which passes `None` where the Kotlin
//           passed `-1`. Reaches the module items via `use super::*` because this file is the `tests`
//           CHILD of player_content.
// Why:      Keep the tests beside the code without inflating `player_content.rs` or its max-lines budget
//           (sibling `*_tests.rs` files are exempt from the linter).

// What:     `use super::*;` imports everything from the parent module into the test scope.
// Why:      Tests need the builder, its input and output types, and the page helpers.
use super::*;
use crate::template_model::TrackFields;

// What:     `const FIXTURE_POSITION_SEC: f64 = 66.0;` is the playhead fixture in seconds.
// Why:      Each deck case checks that the same playhead value reaches the deck.
const FIXTURE_POSITION_SEC: f64 = 66.0;

// What:     `const FIXTURE_DURATION_SEC: f64 = 275.0;` is the duration fixture in seconds.
// Why:      Each deck case checks that the same duration value reaches the deck.
const FIXTURE_DURATION_SEC: f64 = 275.0;

// What:     `fn page_entry(index: usize, name: &str) -> PageEntry` test helper: builds a page entry from
//           a queue index and a display path.
// Why:      Expected inputs are written with explicit queue positions, which keeps the test readable.
fn page_entry(index: usize, name: &str) -> PageEntry {
    return PageEntry {
        index,
        name: name.to_string(),
    };
}

// What:     `fn no_line(_fields: &TrackFields) -> Option<String>` test stub: stands in for a template that
//           shows nothing.
// Why:      These cases do not check the supporting line.
fn no_line(_fields: &TrackFields) -> Option<String> {
    return None;
}

// What:     `fn build(labels, selected_page, items, current_index) -> PlayerContentModel` test helper:
//           builds the content for one state, with the Kotlin defaults of in-order mode and playing.
// Why:      Most cases differ only in labels, page, items and playing index.
fn build(labels: &[&str], selected_page: usize, items: Vec<PageEntry>, current_index: Option<usize>) -> PlayerContentModel {
    let input = PlayerContentInput {
        page_labels: labels.iter().map(|label| return label.to_string()).collect(),
        selected_page,
        page_items: items,
        current_index,
        playing: true,
        mode: PlaybackMode::InOrder,
        position_sec: FIXTURE_POSITION_SEC,
        duration_sec: FIXTURE_DURATION_SEC,
        measures: HashMap::new(),
    };
    return build_player_content(&input, &no_line);
}

// What:     `fn row_keys(entries: &[TrackListEntry]) -> Vec<String>` test helper: returns the key of each
//           entry, `row:<index>` for a row and `header:<name>:<count>` for a header.
// Why:      One list of keys shows both the order and the load indexes in a single assertion.
fn row_keys(entries: &[TrackListEntry]) -> Vec<String> {
    return entries
        .iter()
        .map(|entry| return match entry {
            TrackListEntry::RowEntry(row) => format!("row:{}", row.index_in_library),
            TrackListEntry::HeaderEntry { name, track_count } => format!("header:{name}:{track_count}"),
        })
        .collect();
}

// What:     `fn rows_of(entries: &[TrackListEntry]) -> Vec<&TrackRowModel>` test helper: returns the row
//           entries, without the headers.
// Why:      Current-row checks read the rows only.
fn rows_of(entries: &[TrackListEntry]) -> Vec<&TrackRowModel> {
    return entries
        .iter()
        .filter_map(|entry| return match entry {
            TrackListEntry::RowEntry(row) => Some(row),
            TrackListEntry::HeaderEntry { .. } => None,
        })
        .collect();
}

// What:     `#[test]` marks the next function as a test case.
// Why:      An empty library gives empty rail, rows, and deck text.
#[test]
fn empty_library_gives_empty_content() {
    let content = build(&[], 0, Vec::new(), None);
    assert!(content.sections.is_empty());
    assert_eq!(content.current_folder, None);
    assert_eq!(content.folder_title, "");
    assert!(content.entries.is_empty());
    assert_eq!(content.deck.title, "");
    assert_eq!(content.deck.subtitle, "");
    assert_eq!(content.deck.shuffle_folder_label, "");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A folder page lists its own tracks first, then headers, each with its queue index.
#[test]
fn folder_page_gives_own_tracks_then_headers_with_load_indices() {
    let items = vec![
        page_entry(5, "Camellia/Live/Ghost (Live).flac"),
        page_entry(9, "Camellia/01 Song.flac"),
        page_entry(12, "Camellia/Live/Dokuhebi (Live).flac"),
        page_entry(13, "Camellia/02 Other.flac"),
    ];
    let content = build(&["Camellia"], 0, items, None);
    assert_eq!(
        row_keys(&content.entries),
        vec!["row:9", "row:13", "header:Live:2", "row:5", "row:12"],
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A root letter page lists its root tracks with their queue indexes.
#[test]
fn root_letter_page_lists_root_tracks_with_load_indices() {
    let items = vec![page_entry(3, "Apple.flac"), page_entry(7, "Ant.mp3")];
    let content = build(&["Camellia", "A"], 1, items, Some(7));
    assert_eq!(row_keys(&content.entries), vec!["row:3", "row:7"]);
    assert_eq!(content.current_folder.as_deref(), Some("A"));
    assert_eq!(content.folder_title, "A");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The playing track marks its row, and the deck shows its title and position.
#[test]
fn playing_track_inside_page_marks_its_row_and_deck() {
    let items = vec![
        page_entry(9, "Camellia/01 Song.flac"),
        page_entry(10, "Camellia/02 Other.flac"),
    ];
    let content = build(&["Camellia"], 0, items, Some(10));
    let current_rows: Vec<bool> = rows_of(&content.entries).iter().map(|row| return row.is_current).collect();
    assert_eq!(current_rows, vec![false, true]);
    assert_eq!(content.deck.title, "02 Other");
    assert_eq!(content.deck.subtitle, "2 of 2");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A playing track outside the page marks no row and leaves the deck text empty.
#[test]
fn playing_track_outside_page_gives_empty_deck_text() {
    let items = vec![page_entry(9, "Camellia/01 Song.flac")];
    let content = build(&["Camellia"], 0, items, Some(40));
    assert!(rows_of(&content.entries).iter().all(|row| return !row.is_current));
    assert_eq!(content.deck.title, "");
    assert_eq!(content.deck.subtitle, "");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A single page gives one rail section that carries its name.
#[test]
fn single_page_gives_one_rail_section_with_its_name() {
    let items = vec![page_entry(0, "Camellia/01 Song.flac")];
    let content = build(&["Camellia"], 0, items, Some(0));
    assert_eq!(content.sections.len(), 1);
    assert_eq!(content.sections[0].cells.len(), 1);
    assert_eq!(content.sections[0].cells[0].names, vec!["Camellia"]);
    assert_eq!(content.current_folder.as_deref(), Some("Camellia"));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A selected index past the page list gives no folder and empty labels.
#[test]
fn selected_index_outside_page_list_gives_no_folder() {
    let content = build(&["Camellia"], 4, Vec::new(), None);
    assert_eq!(content.current_folder, None);
    assert_eq!(content.folder_title, "");
    assert_eq!(content.deck.shuffle_folder_label, "");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The deck title drops only the last extension.
#[test]
fn deck_title_drops_only_the_last_extension() {
    let items = vec![page_entry(2, "Camellia/01 Song.v2.flac")];
    let content = build(&["Camellia"], 0, items, Some(2));
    assert_eq!(content.deck.title, "01 Song.v2");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The deck passes the playback position, duration, state, and mode through unchanged.
#[test]
fn deck_passes_playback_values_through() {
    let input = PlayerContentInput {
        page_labels: vec!["Camellia".to_string()],
        selected_page: 0,
        page_items: vec![page_entry(0, "Camellia/01 Song.flac")],
        current_index: Some(0),
        playing: false,
        mode: PlaybackMode::ShuffleAll,
        position_sec: FIXTURE_POSITION_SEC,
        duration_sec: FIXTURE_DURATION_SEC,
        measures: HashMap::new(),
    };
    let content = build_player_content(&input, &no_line);
    let deck = &content.deck;
    assert_eq!(deck.position_sec, FIXTURE_POSITION_SEC);
    assert_eq!(deck.duration_sec, FIXTURE_DURATION_SEC);
    assert!(!deck.playing);
    assert_eq!(deck.mode, PlaybackMode::ShuffleAll);
    assert_eq!(deck.shuffle_folder_label, "Camellia");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The row count in the subtitle follows playback order, including tracks under headers.
#[test]
fn subtitle_counts_rows_in_playback_order() {
    let items = vec![
        page_entry(1, "Camellia/01 Song.flac"),
        page_entry(2, "Camellia/Live/02 Live.flac"),
    ];
    let content = build(&["Camellia"], 0, items, Some(2));
    assert_eq!(content.deck.subtitle, "2 of 2");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The load index rewrite changes track rows only and leaves headers alone.
#[test]
fn with_load_indices_replaces_track_positions_only() {
    let track = FolderListItem::TrackItem(FolderTrackRow {
        track: Track {
            uri: String::new(),
            display_path: "Camellia/A.flac".to_string(),
        },
        index_in_library: 0,
    });
    let header = FolderListItem::SubfolderHeader {
        name: "Live".to_string(),
        track_count: 1,
    };
    let page_items = vec![page_entry(42, "Camellia/A.flac")];
    let expected = vec![
        FolderListItem::TrackItem(FolderTrackRow {
            track: Track {
                uri: String::new(),
                display_path: "Camellia/A.flac".to_string(),
            },
            index_in_library: 42,
        }),
        header.clone(),
    ];
    assert_eq!(with_load_indices(vec![track, header], &page_items), expected);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A page folder keeps every page track in one folder, with one header for its subfolder.
#[test]
fn page_folder_for_keeps_every_page_track_as_one_folder() {
    let items = vec![
        page_entry(4, "Camellia/01 Song.flac"),
        page_entry(5, "Camellia/Live/02 Live.flac"),
        page_entry(6, "Camellia/03 Other.flac"),
    ];
    let folder = page_folder_for("Camellia", &items);
    assert_eq!(folder.name, "Camellia");
    assert_eq!(folder.track_count, 3);
    let track_items = folder
        .items
        .iter()
        .filter(|item| return matches!(item, FolderListItem::TrackItem(_)))
        .count();
    let header_items = folder
        .items
        .iter()
        .filter(|item| return matches!(item, FolderListItem::SubfolderHeader { .. }))
        .count();
    assert_eq!(track_items, 3);
    assert_eq!(header_items, 1);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The subtitle is empty when no row plays, and reads "1 of 5" for the first row.
#[test]
fn subtitle_is_empty_when_no_row_is_playing() {
    assert_eq!(subtitle_for(None, 5), "");
    assert_eq!(subtitle_for(Some(0), 5), "1 of 5");
}
