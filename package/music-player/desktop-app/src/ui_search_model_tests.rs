//! Tests for the pure Search model in `ui_search_model`.

use super::*;
use music_player::search_matcher::SearchHighlight;

/// What: `library()` builds three tracks in two folders for the search tests.
fn library() -> Vec<Track> {
    return ["Abba/Dancing Queen.mp3", "Abba/Waterloo.mp3", "Beta/Ghost.mp3"]
        .iter()
        .map(|display_path| return Track {
            uri: String::new(),
            display_path: display_path.to_string(),
        })
        .collect();
}

#[test]
fn segments_split_plain_and_emphasized_pieces_by_character_offsets() {
    let pieces = segments("Ghost", &[SearchHighlight { start: 0, end: 2 }]);
    assert_eq!(
        pieces,
        vec![
            Segment {
                text: "Gh".to_string(),
                emphasized: true,
            },
            Segment {
                text: "ost".to_string(),
                emphasized: false,
            },
        ]
    );
}

#[test]
fn segments_count_characters_not_bytes_for_non_ascii_text() {
    let pieces = segments("Ünï ok", &[SearchHighlight { start: 1, end: 3 }]);
    let joined: String = pieces.iter().map(|piece| return piece.text.as_str()).collect();
    assert_eq!(joined, "Ünï ok");
    let emphasized: String = pieces
        .iter()
        .filter(|piece| return piece.emphasized)
        .map(|piece| return piece.text.as_str())
        .collect();
    assert_eq!(emphasized, "nï");
}

#[test]
fn segments_without_highlights_is_one_plain_piece() {
    let pieces = segments("Plain", &[]);
    assert_eq!(
        pieces,
        vec![Segment {
            text: "Plain".to_string(),
            emphasized: false,
        }]
    );
}

#[test]
fn empty_query_shows_the_prompt_status_and_no_rows() {
    let view = search_view("", &library(), None, false);
    assert!(view.rows.is_empty());
    assert_eq!(view.status_title, music_player::search_page_model::SEARCH_PROMPT_TITLE);
    assert!(!view.status_detail.is_empty());
}

#[test]
fn empty_library_shows_the_empty_inventory_status() {
    let view = search_view("abba", &[], None, false);
    assert!(view.rows.is_empty());
    assert_eq!(view.status_title, music_player::search_page_model::SEARCH_EMPTY_INVENTORY_TITLE);
}

#[test]
fn query_with_no_match_shows_the_no_match_status_with_the_query() {
    let view = search_view("zzzz", &library(), None, false);
    assert!(view.rows.is_empty());
    assert!(view.status_title.contains("zzzz"));
}

#[test]
fn matching_query_gives_rows_with_track_kind_and_parent_folder() {
    let view = search_view("waterloo", &library(), None, false);
    assert!(view.status_title.is_empty());
    assert_eq!(view.rows.len(), 1);
    assert_eq!(view.rows[0].kind, 1);
    assert!(view.rows[0].has_parent);
    assert_eq!(view.rows[0].index, 0);
    assert_eq!(view.results.len(), view.rows.len());
}

#[test]
fn row_for_the_playing_track_is_current_and_playing() {
    let tracks = library();
    let view = search_view("ghost", &tracks, Some(2), true);
    assert_eq!(view.rows.len(), 1);
    assert!(view.rows[0].current);
    assert!(view.rows[0].playing);
}

#[test]
fn row_for_the_paused_current_track_is_current_but_not_playing() {
    let view = search_view("ghost", &library(), Some(2), false);
    assert!(view.rows[0].current);
    assert!(!view.rows[0].playing);
}

#[test]
fn title_segments_join_back_to_the_title_text() {
    let view = search_view("queen", &library(), None, false);
    let joined: String = view.rows[0].title.iter().map(|piece| return piece.text.as_str()).collect();
    assert_eq!(joined, "Dancing Queen");
}

#[test]
fn a_track_row_is_spoken_as_its_name_its_kind_and_its_parent_folder() {
    let view = search_view("waterloo", &library(), None, false);
    assert_eq!(view.rows[0].spoken, "Waterloo, Track, Abba");
}

#[test]
fn a_folder_row_is_spoken_as_its_name_and_its_kind() {
    let view = search_view("abba", &library(), None, false);
    let folder = view.rows.iter().find(|row| return row.kind == 0).expect("a folder result for abba");
    assert_eq!(folder.spoken, "Abba, Folder");
}

#[test]
fn a_root_level_track_has_no_parent_in_its_spoken_label() {
    let tracks = vec![Track { uri: String::new(), display_path: "Solo.mp3".to_string() }];
    let view = search_view("solo", &tracks, None, false);
    assert_eq!(view.rows[0].spoken, "Solo, Track");
}
