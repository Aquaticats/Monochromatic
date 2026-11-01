// What:     Unit tests for `folder_index.rs`, pulled in by
//           `#[cfg(test)] #[path = "folder_index_tests.rs"] mod tests;` at the bottom of
//           `folder_index.rs`. Mirrors every `@Test` of `FolderIndexTest.kt`. The last two
//           cases are extras with no Kotlin counterpart. They cover the dot-segment drop and the
//           sanitizing of subfolder names, which no Kotlin case reaches. Reaches the module items
//           via `use super::*` because this file is the `tests` CHILD of folder_index.
// Why:      Keep the tests beside the code without inflating `folder_index.rs` or its max-lines budget
//           (sibling `*_tests.rs` files are exempt from the linter).

// What:     `use super::*;` imports everything from the parent module into the test scope.
// Why:      Tests need `folder_index`, the row and entry types, and `UNFILED_FOLDER_NAME`.
use super::*;

// What:     `fn track(display_path: &str) -> Track` test helper: builds a fixture track whose URI
//           is derived from its display path.
// Why:      Only the display path affects grouping, so the URI can be derived from it.
fn track(display_path: &str) -> Track {
    return Track {
        uri: format!("content://{display_path}"),
        display_path: display_path.to_string(),
    };
}

// What:     `fn track_item(tracks: &[Track], index: usize) -> FolderListItem` test helper: builds the
//           expected track row for the library track at one index.
// Why:      Expected lists are written with the library index, which keeps the test readable.
fn track_item(tracks: &[Track], index: usize) -> FolderListItem {
    return FolderListItem::TrackItem(FolderTrackRow {
        track: tracks[index].clone(),
        index_in_library: index,
    });
}

// What:     `fn header(name: &str, track_count: usize) -> FolderListItem` test helper: builds an
//           expected subfolder header row.
// Why:      Header expectations read more clearly as one call than as a struct literal.
fn header(name: &str, track_count: usize) -> FolderListItem {
    return FolderListItem::SubfolderHeader {
        name: name.to_string(),
        track_count,
    };
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Top-level folders come out sorted by name, with one entry each.
#[test]
fn groups_and_sorts_top_level_folders() {
    let entries = folder_index(&[track("Zed/z.flac"), track("Alpha/a.flac"), track("Mid/m.flac")]);
    let names: Vec<&str> = entries.iter().map(|entry| return entry.name.as_str()).collect();
    assert_eq!(names, vec!["Alpha", "Mid", "Zed"]);
    let counts: Vec<usize> = entries.iter().map(|entry| return entry.track_count).collect();
    assert_eq!(counts, vec![1, 1, 1]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A folder's own tracks lead, then each subfolder shows a header with its count and tracks (D5).
#[test]
fn tracks_come_before_subfolder_headers_with_counts() {
    let tracks = vec![
        track("Camellia/01.flac"),
        track("Camellia/Album/02.flac"),
        track("Camellia/Album/03.flac"),
        track("Camellia/Live/04.flac"),
        track("Camellia/05.flac"),
    ];
    let entries = folder_index(&tracks);
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    assert_eq!(
        entry.items,
        vec![
            track_item(&tracks, 0),
            track_item(&tracks, 4),
            header("Album", 2),
            track_item(&tracks, 1),
            track_item(&tracks, 2),
            header("Live", 1),
            track_item(&tracks, 3),
        ],
    );
    assert_eq!(entry.track_count, 5);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Subfolder headers sort by name regardless of letter case.
#[test]
fn subfolder_names_sort_case_insensitively() {
    let tracks = vec![
        track("Camellia/beta/1.flac"),
        track("Camellia/Alpha/2.flac"),
        track("Camellia/gamma/3.flac"),
    ];
    let entries = folder_index(&tracks);
    assert_eq!(
        entries[0].items,
        vec![
            header("Alpha", 1),
            track_item(&tracks, 1),
            header("beta", 1),
            track_item(&tracks, 0),
            header("gamma", 1),
            track_item(&tracks, 2),
        ],
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A track two folders below the top level sits under its full subfolder path header (D5).
#[test]
fn deeper_nesting_is_flattened_under_nearest_named_subfolder_path() {
    let tracks = vec![
        track("Camellia/Album/01.flac"),
        track("Camellia/Album/Disc 1/02.flac"),
        track("Camellia/Album/Disc 1/03.flac"),
    ];
    let entries = folder_index(&tracks);
    assert_eq!(
        entries[0].items,
        vec![
            header("Album", 1),
            track_item(&tracks, 0),
            header("Album/Disc 1", 2),
            track_item(&tracks, 1),
            track_item(&tracks, 2),
        ],
    );
    assert_eq!(entries[0].track_count, 3);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Root-level tracks form one Unfiled entry, listed after every named folder.
#[test]
fn root_level_tracks_go_to_unfiled_last() {
    let tracks = vec![
        track("Loose.flac"),
        track("Zed/z.flac"),
        track("Camellia/c.flac"),
        track("Another loose.flac"),
    ];
    let entries = folder_index(&tracks);
    let names: Vec<&str> = entries.iter().map(|entry| return entry.name.as_str()).collect();
    assert_eq!(names, vec!["Camellia", "Zed", UNFILED_FOLDER_NAME]);
    let last = &entries[2];
    assert_eq!(last.items, vec![track_item(&tracks, 0), track_item(&tracks, 3)]);
    assert_eq!(last.track_count, 2);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The Unfiled entry exists only when a root-level track does, so an all-folder library gets none.
#[test]
fn no_unfiled_entry_when_no_track_sits_at_root() {
    let entries = folder_index(&[track("Camellia/c.flac")]);
    let names: Vec<&str> = entries.iter().map(|entry| return entry.name.as_str()).collect();
    assert_eq!(names, vec!["Camellia"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Playback addresses tracks by library index, so each row keeps the index it had in the input.
#[test]
fn index_in_library_is_preserved() {
    let tracks = vec![
        track("Camellia/Album/a.flac"),
        track("Loose.flac"),
        track("Camellia/b.flac"),
        track("Zed/z.flac"),
    ];
    let mut rows: Vec<FolderTrackRow> = Vec::new();
    for entry in folder_index(&tracks) {
        for item in entry.items {
            if let FolderListItem::TrackItem(row) = item {
                rows.push(row);
            }
        }
    }
    let mut positions: Vec<usize> = rows.iter().map(|row| return row.index_in_library).collect();
    positions.sort();
    assert_eq!(positions, vec![0, 1, 2, 3]);
    for row in &rows {
        assert_eq!(tracks[row.index_in_library], row.track);
    }
}

// What:     `#[test]` marks the next function as a test case.
// Why:      An empty library produces no folder entries.
#[test]
fn empty_library_gives_empty_list() {
    assert_eq!(folder_index(&[]), Vec::<FolderEntry>::new());
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Top-level folder names sort without regard to letter case.
#[test]
fn folder_names_with_different_case_sort_case_insensitively() {
    let entries = folder_index(&[
        track("beta/1.flac"),
        track("Alpha/2.flac"),
        track("gamma/3.flac"),
        track("capsule/4.flac"),
        track("Camellia/5.flac"),
    ]);
    let names: Vec<&str> = entries.iter().map(|entry| return entry.name.as_str()).collect();
    assert_eq!(names, vec!["Alpha", "beta", "Camellia", "capsule", "gamma"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Two folder names that differ only in case come out in a fixed code point order.
#[test]
fn folders_that_tie_ignoring_case_sort_by_code_point() {
    let entries = folder_index(&[track("camellia/x.flac"), track("Camellia/y.flac")]);
    let names: Vec<&str> = entries.iter().map(|entry| return entry.name.as_str()).collect();
    assert_eq!(names, vec!["Camellia", "camellia"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A folder with no subfolders lists only its own track rows.
#[test]
fn folder_with_only_own_tracks_has_no_headers() {
    let tracks = vec![track("Camellia/a.flac"), track("Camellia/b.flac")];
    let entries = folder_index(&tracks);
    assert_eq!(entries[0].items, vec![track_item(&tracks, 0), track_item(&tracks, 1)]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Headers belong to their own top-level folder, so equal subfolder names do not merge.
#[test]
fn same_subfolder_name_in_two_folders_stays_separate() {
    let tracks = vec![track("A/Live/1.flac"), track("B/Live/2.flac")];
    let entries = folder_index(&tracks);
    assert_eq!(entries[0].items, vec![header("Live", 1), track_item(&tracks, 0)]);
    assert_eq!(entries[1].items, vec![header("Live", 1), track_item(&tracks, 1)]);
}

// What:     `#[test]` marks the next function as a test case. This case is an extra with no Kotlin counterpart.
// Why:      `.` and `..` segments are dropped, so they neither create a folder nor a level.
#[test]
fn dot_segments_are_dropped_before_grouping() {
    let tracks = vec![track("A/./x.flac"), track("A/x.flac"), track("../loose.flac")];
    let entries = folder_index(&tracks);
    let names: Vec<&str> = entries.iter().map(|entry| return entry.name.as_str()).collect();
    assert_eq!(names, vec!["A", UNFILED_FOLDER_NAME]);
    assert_eq!(entries[0].items, vec![track_item(&tracks, 0), track_item(&tracks, 1)]);
    assert_eq!(entries[1].items, vec![track_item(&tracks, 2)]);
}

// What:     `#[test]` marks the next function as a test case. This case is an extra with no Kotlin counterpart.
// Why:      A control character in a subfolder name becomes a space in its header.
#[test]
fn subfolder_header_name_is_sanitized() {
    let tracks = vec![track("A/Live\n/1.flac")];
    let entries = folder_index(&tracks);
    assert_eq!(entries[0].items, vec![header("Live ", 1), track_item(&tracks, 0)]);
}
