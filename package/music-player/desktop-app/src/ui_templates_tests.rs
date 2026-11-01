//! Tests for the pure template helpers in `ui_templates`.

use super::*;
use music_player::template_model::DEFAULT_TRACK_TEMPLATE;
use music_player::track_supporting_line::supporting_line_from_template;

#[test]
fn track_fields_split_folder_file_stem_and_extension() {
    let fields = track_fields("Abba/Live/Dancing Queen.flac");
    assert_eq!(fields.folder, "Abba/Live");
    assert_eq!(fields.file, "Dancing Queen");
    assert_eq!(fields.ext, "flac");
    assert_eq!(fields.path, "Abba/Live/Dancing Queen.flac");
    assert_eq!(fields.len, None);
    assert_eq!(fields.peak, None);
}

#[test]
fn track_fields_of_a_root_file_have_no_folder() {
    let fields = track_fields("loose.mp3");
    assert_eq!(fields.folder, "");
    assert_eq!(fields.file, "loose");
    assert_eq!(fields.ext, "mp3");
}

#[test]
fn hidden_file_without_stem_keeps_its_whole_name() {
    let fields = track_fields(".hidden");
    assert_eq!(fields.file, ".hidden");
    assert_eq!(fields.ext, "");
}

#[test]
fn editor_library_numbers_each_track_within_the_queue() {
    let names = vec!["a/one.mp3".to_string(), "a/two.mp3".to_string()];
    let library = editor_library(&names, &HashMap::new());
    assert_eq!(library[1].place_in_folder, Some(2));
    assert_eq!(library[1].tracks_in_folder, Some(2));
}

#[test]
fn exact_measure_renders_length_and_peak_in_the_default_row_template() {
    let measure = TrackMeasure {
        duration_secs: Some(20.0),
        peak_dbtp: Some(-21.1),
        peak_exact: true,
    };
    let fields = fields_with_measure(track_fields("a/one.flac"), Some(&measure));
    let line = supporting_line_from_template(DEFAULT_TRACK_TEMPLATE)(&fields);
    assert_eq!(line, Some("0:20 \u{2212}21.1 dBTP".to_string()));
}

#[test]
fn unmeasured_track_renders_no_peak_in_the_default_row_template() {
    let fields = fields_with_measure(track_fields("a/one.flac"), None);
    let line = supporting_line_from_template(DEFAULT_TRACK_TEMPLATE)(&fields);
    assert_eq!(line.unwrap_or_default().trim(), "");
}

#[test]
fn utf16_offset_counts_code_units_for_non_ascii_text() {
    let text = "ü€𝄞x";
    assert_eq!(utf16_offset(text, 0), 0);
    assert_eq!(utf16_offset(text, 2), 1);
    assert_eq!(utf16_offset(text, 5), 2);
    assert_eq!(utf16_offset(text, 9), 4);
    assert_eq!(utf16_offset(text, text.len()), 5);
}

#[test]
fn utf16_offset_rounds_inside_a_character_down_to_its_start() {
    let text = "ü!";
    assert_eq!(utf16_offset(text, 1), 0);
}

#[test]
fn utf16_offset_clamps_past_the_end() {
    assert_eq!(utf16_offset("ab", 99), 2);
}
