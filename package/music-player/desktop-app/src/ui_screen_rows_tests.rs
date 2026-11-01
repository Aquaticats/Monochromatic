//! Tests for the pure row and picker conversions in `ui_screen_rows`.

use super::*;
use music_player::folder_rail::rail_for;
use music_player::track_entries::TrackRowModel;

/// What: `track(index, title, current)` builds a track entry for the row tests.
fn track(index: usize, title: &str, current: bool) -> TrackListEntry {
    return TrackListEntry::RowEntry(TrackRowModel {
        index_in_library: index,
        title: title.to_string(),
        supporting_line: Some(format!("line {index}")),
        is_current: current,
    });
}

#[test]
fn track_rows_keep_index_title_supporting_line_and_current_flag() {
    let rows = list_rows(&[track(4, "Ghost", true)]);
    assert_eq!(
        rows,
        vec![ListRow {
            kind: 0,
            title: "Ghost".to_string(),
            supporting: "line 4".to_string(),
            current: true,
            index: 4,
        }]
    );
}

#[test]
fn track_without_supporting_line_has_empty_second_line() {
    let entry = TrackListEntry::RowEntry(TrackRowModel {
        index_in_library: 0,
        title: "Bare".to_string(),
        supporting_line: None,
        is_current: false,
    });
    assert_eq!(list_rows(&[entry])[0].supporting, "");
}

#[test]
fn header_row_has_kind_one_count_text_and_no_index() {
    let rows = list_rows(&[TrackListEntry::HeaderEntry {
        name: "Live".to_string(),
        track_count: 3,
    }]);
    assert_eq!(rows[0].kind, 1);
    assert_eq!(rows[0].title, "Live");
    assert_eq!(rows[0].supporting, "3 tracks");
    assert_eq!(rows[0].index, -1);
    assert!(!rows[0].current);
}

#[test]
fn track_count_text_uses_singular_only_for_one() {
    assert_eq!(track_count_text(0), "0 tracks");
    assert_eq!(track_count_text(1), "1 track");
    assert_eq!(track_count_text(2), "2 tracks");
}

#[test]
fn current_row_position_counts_headers_and_returns_minus_one_without_current() {
    let rows = list_rows(&[
        TrackListEntry::HeaderEntry {
            name: "Live".to_string(),
            track_count: 1,
        },
        track(1, "Ghost", false),
        track(2, "Hyperflux", true),
    ]);
    assert_eq!(current_row_position(&rows), 2);
    let none = list_rows(&[track(0, "Ghost", false)]);
    assert_eq!(current_row_position(&none), -1);
    assert_eq!(current_row_position(&[]), -1);
}

#[test]
fn picker_heading_rows_precede_each_cell_and_hold_its_names() {
    let sections = rail_for(&["Abba".to_string(), "Beta".to_string()]);
    let picker = picker(&sections, "Beta");
    let headings: Vec<&PickerName> = picker.names.iter().filter(|row| return row.section_start).collect();
    assert_eq!(headings.len(), 2);
    assert_eq!(headings[0].heading, "A");
    let beta_names: Vec<&str> = picker
        .names
        .iter()
        .filter(|row| return !row.section_start && row.heading == "B")
        .map(|row| return row.name.as_str())
        .collect();
    assert_eq!(beta_names, vec!["Beta"]);
}

#[test]
fn picker_highlights_the_cell_holding_the_current_folder() {
    let sections = rail_for(&["Abba".to_string(), "Beta".to_string()]);
    assert_eq!(picker(&sections, "Beta").current_cell, "B");
    assert_eq!(picker(&sections, "Missing").current_cell, "");
}

#[test]
fn picker_cell_key_equals_its_label() {
    let sections = rail_for(&["Abba".to_string()]);
    let picker = picker(&sections, "Abba");
    assert_eq!(picker.cells.len(), 1);
    assert_eq!(picker.cells[0].key, picker.cells[0].label);
}

/// What: `row` builds a track row (kind 0) or a header row (kind 1) with the given title.
/// Why: the prefix tests only read titles and kinds.
fn plain(kind: i32, title: &str) -> ListRow {
    return ListRow { kind, title: title.to_string(), supporting: String::new(), current: false, index: if kind == 0 { 1 } else { -1 } };
}

/// What: `titles` lists the titles of rows.
/// Why: the tests compare what a person would read.
fn titles(rows: &[ListRow]) -> Vec<&str> {
    return rows.iter().map(|row| return row.title.as_str()).collect();
}

#[test]
fn rows_of_one_folder_lose_the_prefix_they_share() {
    let mut rows = vec![plain(0, "かめりあ(Camellia) - Aegis"), plain(0, "かめりあ(Camellia) - Another Xronixle")];
    strip_shared_prefixes(&mut rows, "かめりあ(Camellia)");
    assert_eq!(titles(&rows), vec!["Aegis", "Another Xronixle"]);
}

#[test]
fn each_subfolder_run_finds_its_own_prefix() {
    let mut rows = vec![
        plain(0, "Top - one"),
        plain(0, "Top - two"),
        plain(1, "Disc 1"),
        plain(0, "Sub - a"),
        plain(0, "Sub - b"),
        plain(1, "Disc 2"),
        plain(0, "01 Lonely"),
    ];
    strip_shared_prefixes(&mut rows, "Artist");
    assert_eq!(titles(&rows), vec!["one", "two", "Disc 1", "a", "b", "Disc 2", "01 Lonely"]);
}

#[test]
fn a_single_row_loses_the_name_of_its_directory() {
    let mut rows = vec![plain(0, "Artist - Intro"), plain(1, "Album"), plain(0, "Album - Song")];
    strip_shared_prefixes(&mut rows, "Artist");
    assert_eq!(titles(&rows), vec!["Intro", "Album", "Song"]);
}

#[test]
fn a_nested_header_names_its_last_component() {
    let mut rows = vec![plain(1, "Album/Disc 1"), plain(0, "Disc 1 - Intro")];
    strip_shared_prefixes(&mut rows, "Artist");
    assert_eq!(titles(&rows), vec!["Album/Disc 1", "Intro"]);
}

#[test]
fn no_rows_and_numbered_rows_are_left_alone() {
    let mut none: Vec<ListRow> = Vec::new();
    strip_shared_prefixes(&mut none, "x");
    assert!(none.is_empty());
    let mut numbered = vec![plain(0, "01 Song"), plain(0, "02 Song")];
    strip_shared_prefixes(&mut numbered, "x");
    assert_eq!(titles(&numbered), vec!["01 Song", "02 Song"]);
}

#[test]
fn the_first_cell_of_each_writing_system_after_the_first_starts_a_rail_section() {
    // A Latin M, a Cyrillic М and a Greek Α look alike in the rail; D28 separates the writing systems with a hairline.
    let names: Vec<String> = ["Madonna", "Muse", "Мумий Тролль", "Кино", "Αρλέτα"].iter().map(|name| return (*name).to_string()).collect();
    let sections = rail_for(&names);
    let picked = picker(&sections, "");
    let starts: Vec<(&str, bool)> = picked.cells.iter().map(|cell| return (cell.label.as_str(), cell.section_start)).collect();
    assert!(!starts[0].1, "the first cell has nothing above it: {starts:?}");
    let section_starts = picked.cells.iter().filter(|cell| return cell.section_start).count();
    assert_eq!(section_starts, sections.len() - 1, "one separator between each pair of writing systems: {starts:?}");
    assert!(section_starts >= 2, "Latin, Cyrillic and Greek are three sections: {starts:?}");
}
