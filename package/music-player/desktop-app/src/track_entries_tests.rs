// What:     Unit tests for `track_entries.rs`, pulled in by
//           `#[cfg(test)] #[path = "track_entries_tests.rs"] mod tests;` at the bottom of
//           `track_entries.rs`. Mirrors every `@Test` of `TrackListEntriesTest.kt`. The template
//           cases pass a stub closure in place of the template engine. Each stub reads the fields
//           the way the template would, so the trim and empty-to-`None` step is exercised for real.
//           Template evaluation itself is not exercised here. Reaches the module items via
//           `use super::*` because this file is the `tests` CHILD of track_entries.
// Why:      Keep the tests beside the code without inflating `track_entries.rs` or its max-lines budget
//           (sibling `*_tests.rs` files are exempt from the linter).

// What:     `use super::*;` imports everything from the parent module into the test scope.
// Why:      Tests need the entry types, `track_title_for`, and `track_list_entries`.
use super::*;
use crate::folder_index::{UNFILED_FOLDER_NAME, folder_index};
use crate::track::Track;
use crate::template_model::DEFAULT_TRACK_TEMPLATE;
use crate::track_supporting_line::supporting_line_from_template;

// What:     `fn track(display_path: &str) -> Track` test helper: builds a fixture track whose URI is
//           derived from its display path.
// Why:      Only the display path affects grouping, so the URI can be derived from it.
fn track(display_path: &str) -> Track {
    return Track {
        uri: format!("content://{display_path}"),
        display_path: display_path.to_string(),
    };
}

// What:     `fn only_folder(tracks: Vec<Track>) -> FolderEntry` test helper: returns the first folder
//           the library produces.
// Why:      Most cases work on one folder, and the Kotlin uses `first()` for the same purpose.
fn only_folder(tracks: Vec<Track>) -> FolderEntry {
    return folder_index(&tracks)
        .into_iter()
        .next()
        .expect("the fixture library has at least one folder");
}

// What:     `fn no_line(_fields: &TrackFields) -> Option<String>` test stub: stands in for a template
//           that shows nothing.
// Why:      Cases that do not read the supporting line pass this stub.
fn no_line(_fields: &TrackFields) -> Option<String> {
    return None;
}

// What:     `fn rows(entries: &[TrackListEntry]) -> Vec<&TrackRowModel>` test helper: returns the row
//           entries of a list, without the headers.
// Why:      Most assertions read the rows only.
fn rows(entries: &[TrackListEntry]) -> Vec<&TrackRowModel> {
    return entries
        .iter()
        .filter_map(|entry| return match entry {
            TrackListEntry::RowEntry(row) => Some(row),
            TrackListEntry::HeaderEntry { .. } => None,
        })
        .collect();
}

// What:     `fn only_row(entries: &[TrackListEntry]) -> &TrackRowModel` test helper: returns the only
//           row of a list, failing the test when there is not exactly one.
// Why:      The template cases each check one row's supporting line.
fn only_row(entries: &[TrackListEntry]) -> &TrackRowModel {
    let found = rows(entries);
    assert_eq!(found.len(), 1, "expected exactly one row");
    return found[0];
}

// What:     `fn plain_row(index_in_library: usize, title: &str, is_current: bool) -> TrackListEntry`
//           test helper: builds an expected row entry whose supporting line is absent.
// Why:      Most expected rows carry no supporting line, so this keeps the expectations short.
fn plain_row(index_in_library: usize, title: &str, is_current: bool) -> TrackListEntry {
    return TrackListEntry::RowEntry(TrackRowModel {
        index_in_library,
        title: title.to_string(),
        supporting_line: None,
        is_current,
    });
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Only the last extension is dropped, so `.v2` survives in the title.
#[test]
fn title_drops_only_the_last_extension() {
    assert_eq!(track_title_for("Camellia/Album/01 Song.v2.flac"), "01 Song.v2");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Each common audio extension is dropped from the title.
#[test]
fn title_drops_each_kind_of_extension() {
    let paths = ["a.mp3", "b.m4a", "c.opus", "d.ogg"];
    let titles: Vec<String> = paths.iter().map(|path| return track_title_for(path)).collect();
    assert_eq!(titles, vec!["a", "b", "c", "d"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Every folder segment is stripped from the title.
#[test]
fn title_strips_every_folder_segment() {
    assert_eq!(track_title_for("Camellia/Album/Disc 1/03 X.flac"), "03 X");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A name with no extension is kept whole.
#[test]
fn title_keeps_name_without_extension() {
    assert_eq!(track_title_for("Camellia/Song"), "Song");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A hidden file name keeps its leading dot.
#[test]
fn title_keeps_hidden_file_name_whole() {
    assert_eq!(track_title_for("Camellia/.hidden"), ".hidden");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Rows come out in folder order, each with its library index.
#[test]
fn rows_follow_folder_order_with_library_indexes() {
    let folder = only_folder(vec![
        track("Camellia/b.flac"),
        track("Camellia/a.flac"),
        track("Camellia/Album/c.flac"),
    ]);
    assert_eq!(
        track_list_entries(&folder, None, &HashMap::new(), &no_line),
        vec![
            plain_row(0, "b", false),
            plain_row(1, "a", false),
            TrackListEntry::HeaderEntry {
                name: "Album".to_string(),
                track_count: 1,
            },
            plain_row(2, "c", false),
        ],
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The current flag is set on the one row whose library index matches.
#[test]
fn current_flag_marks_only_the_matching_index() {
    let folder = only_folder(vec![
        track("Camellia/a.flac"),
        track("Camellia/b.flac"),
        track("Camellia/c.flac"),
    ]);
    let entries = track_list_entries(&folder, Some(1), &HashMap::new(), &no_line);
    let flags: Vec<bool> = rows(&entries).iter().map(|row| return row.is_current).collect();
    assert_eq!(flags, vec![false, true, false]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      No row is marked current when the playing index is outside the folder.
#[test]
fn current_index_outside_folder_marks_no_row() {
    let folder = only_folder(vec![track("Camellia/a.flac"), track("Camellia/b.flac")]);
    let entries = track_list_entries(&folder, Some(99), &HashMap::new(), &no_line);
    let flags: Vec<bool> = rows(&entries).iter().map(|row| return row.is_current).collect();
    assert_eq!(flags, vec![false, false]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A subfolder header counts only the tracks directly under it.
#[test]
fn header_counts_only_direct_tracks() {
    let folder = only_folder(vec![
        track("Camellia/Album/Disc 1/x.flac"),
        track("Camellia/Album/y.flac"),
    ]);
    assert_eq!(
        track_list_entries(&folder, None, &HashMap::new(), &no_line),
        vec![
            TrackListEntry::HeaderEntry {
                name: "Album".to_string(),
                track_count: 1,
            },
            plain_row(1, "y", false),
            TrackListEntry::HeaderEntry {
                name: "Album/Disc 1".to_string(),
                track_count: 1,
            },
            plain_row(0, "x", false),
        ],
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The default template gives no line while length and peak are unknown.
#[test]
fn default_template_with_unknown_length_and_peak_gives_no_line() {
    let folder = only_folder(vec![track("Camellia/Song.flac")]);
    let default_line = supporting_line_from_template(DEFAULT_TRACK_TEMPLATE);
    let entries = track_list_entries(&folder, None, &HashMap::new(), &default_line);
    assert_eq!(only_row(&entries).supporting_line, None);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A length-only template gives no line while the length is unknown.
#[test]
fn length_only_template_gives_no_line() {
    let folder = only_folder(vec![track("Camellia/Song.flac")]);
    let length_line = supporting_line_from_template("$tf(mi(len), m:ss)$");
    let entries = track_list_entries(&folder, None, &HashMap::new(), &length_line);
    assert_eq!(only_row(&entries).supporting_line, None);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A peak-only template gives no line before analysis.
#[test]
fn peak_only_template_gives_no_line() {
    let folder = only_folder(vec![track("Camellia/Song.flac")]);
    let peak_line = supporting_line_from_template("$mi(peak)$");
    let entries = track_list_entries(&folder, None, &HashMap::new(), &peak_line);
    assert_eq!(only_row(&entries).supporting_line, None);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A template that joins the folder and file names shows both, with the text trimmed.
#[test]
fn custom_template_shows_folder_and_file_name() {
    let folder = only_folder(vec![track("Camellia/Album/01 Song.flac")]);
    let folder_and_file = supporting_line_from_template("$mi(folder)$ / $mi(file)$");
    let entries = track_list_entries(&folder, None, &HashMap::new(), &folder_and_file);
    assert_eq!(only_row(&entries).supporting_line.as_deref(), Some("Album / 01 Song"));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Trailing blank text left by an empty field is trimmed away.
#[test]
fn custom_template_trims_blank_from_unknown_field() {
    let folder = only_folder(vec![track("Camellia/01 Song.flac")]);
    let trailing_blank = supporting_line_from_template("$mi(file)$ $mi(peak)$");
    let entries = track_list_entries(&folder, None, &HashMap::new(), &trailing_blank);
    assert_eq!(only_row(&entries).supporting_line.as_deref(), Some("01 Song"));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      An invalid template, which the engine reports as not shown, gives no line.
#[test]
fn invalid_template_gives_no_line() {
    let folder = only_folder(vec![track("Camellia/Song.flac")]);
    let not_shown = supporting_line_from_template("$nosuch(mi(file))$");
    let entries = track_list_entries(&folder, None, &HashMap::new(), &not_shown);
    assert_eq!(only_row(&entries).supporting_line, None);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A root-level track's empty folder field gives no line, since blank text is dropped.
#[test]
fn root_file_folder_field_gives_no_line() {
    let folder = only_folder(vec![track("Loose.flac")]);
    assert_eq!(folder.name, UNFILED_FOLDER_NAME);
    let folder_field = supporting_line_from_template("$mi(folder)$");
    let entries = track_list_entries(&folder, None, &HashMap::new(), &folder_field);
    assert_eq!(only_row(&entries).supporting_line, None);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The folder field shows the innermost folder name, not the whole path.
#[test]
fn nested_folder_field_shows_innermost_name() {
    let folder = only_folder(vec![track("Camellia/Album/Disc 1/03 X.flac")]);
    let folder_field = supporting_line_from_template("$mi(folder)$");
    let entries = track_list_entries(&folder, None, &HashMap::new(), &folder_field);
    assert_eq!(only_row(&entries).supporting_line.as_deref(), Some("Disc 1"));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      An empty folder gives an empty list.
#[test]
fn empty_folder_gives_empty_list() {
    let folder = FolderEntry {
        name: "Empty".to_string(),
        items: Vec::new(),
        track_count: 0,
    };
    assert_eq!(track_list_entries(&folder, None, &HashMap::new(), &no_line), Vec::<TrackListEntry>::new());
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A header's track count equals the number of tracks in its subfolder.
#[test]
fn header_track_count_matches_subfolder_size() {
    let folder = only_folder(vec![
        track("Camellia/Album/a.flac"),
        track("Camellia/Album/b.flac"),
        track("Camellia/Album/c.flac"),
    ]);
    let entries = track_list_entries(&folder, None, &HashMap::new(), &no_line);
    assert_eq!(
        entries.into_iter().take(1).collect::<Vec<TrackListEntry>>(),
        vec![TrackListEntry::HeaderEntry {
            name: "Album".to_string(),
            track_count: 3,
        }],
    );
}

// What:     `fn supporting_text(entries: &[TrackListEntry]) -> Option<String>` reads the first row's supporting line.
// Why:      The measure cases assert on the line the closure produced, not on the whole entry.
fn supporting_text(entries: &[TrackListEntry]) -> Option<String> {
    return match entries.first() {
        Some(TrackListEntry::RowEntry(row)) => row.supporting_line.clone(),
        _ => None,
    };
}

// What:     `#[test] fn exact_measure_reaches_the_supporting_line_fields()` checks that length and exact peak arrive.
// Why:      Positive control: the row's template fields must carry the measure looked up by display name.
#[test]
fn exact_measure_reaches_the_supporting_line_fields() {
    let folder = only_folder(vec![track("Song.flac")]);
    let measure = TrackMeasure {
        duration_secs: Some(20.0),
        peak_dbtp: Some(-21.1),
        peak_exact: true,
    };
    let measures = HashMap::from([("Song.flac".to_string(), measure)]);
    let line = |fields: &TrackFields| return Some(format!("{:?}|{:?}", fields.len, fields.peak));
    let entries = track_list_entries(&folder, None, &measures, &line);
    assert_eq!(supporting_text(&entries), Some("Some(20.0)|Some(\"\u{2212}21.1 dBTP\")".to_string()));
}

// What:     `#[test] fn sampled_measure_gives_length_without_peak()` checks that a sampled peak is left out of the row.
// Why:      Only exact peaks show in rows, so the sampled case must carry the length alone.
#[test]
fn sampled_measure_gives_length_without_peak() {
    let folder = only_folder(vec![track("Song.flac")]);
    let measure = TrackMeasure {
        duration_secs: Some(20.0),
        peak_dbtp: Some(-21.1),
        peak_exact: false,
    };
    let measures = HashMap::from([("Song.flac".to_string(), measure)]);
    let line = |fields: &TrackFields| return Some(format!("{:?}|{:?}", fields.len, fields.peak));
    let entries = track_list_entries(&folder, None, &measures, &line);
    assert_eq!(supporting_text(&entries), Some("Some(20.0)|None".to_string()));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The deck names the innermost folder that holds a nested track.
#[test]
fn deck_folder_names_the_innermost_directory() {
    assert_eq!(deck_folder_label("Camellia/Live/Ghost.flac", "Camellia"), "Live");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A track directly under the page folder names that folder.
#[test]
fn deck_folder_of_a_direct_child_is_its_folder() {
    assert_eq!(deck_folder_label("Camellia/Ghost.flac", "Camellia"), "Camellia");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A path with no directory part falls back to the page label.
#[test]
fn deck_folder_without_a_directory_uses_the_page_label() {
    assert_eq!(deck_folder_label("Ghost.flac", "L"), "L");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A single trailing component with no page label gives an empty folder.
#[test]
fn deck_folder_of_a_lone_trailing_component_without_page_label_is_empty() {
    assert_eq!(deck_folder_label("Ghost.flac", ""), "");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      An empty path gives an empty folder even when a page label is given.
#[test]
fn deck_folder_of_an_empty_path_is_empty() {
    assert_eq!(deck_folder_label("", "Camellia"), "");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A trailing separator leaves the last directory as the folder.
#[test]
fn deck_folder_with_a_trailing_separator_names_the_last_directory() {
    assert_eq!(deck_folder_label("Camellia/Live/", "Camellia"), "Live");
}
