//! Tests for the pure Settings and editor models in `ui_settings_model`.

use super::*;
use music_player::template_editor_state::TemplateEditorState;

#[test]
fn settings_lists_both_templates_in_kind_order() {
    let editor = TemplateEditorState::initial(&[]);
    let rows = settings_rows(&editor);
    assert_eq!(rows.len(), TemplateKind::all().len());
    assert_eq!(rows[0].label, TemplateKind::TrackRows.label());
    assert!(!rows[0].line.is_empty());
}

#[test]
fn kind_of_index_maps_the_settings_rows_back_to_templates() {
    assert_eq!(kind_of_index(0), Some(TemplateKind::TrackRows));
    assert_eq!(kind_of_index(9), None);
}

#[test]
fn editor_view_opens_on_the_track_rows_template_with_its_text() {
    let editor = TemplateEditorState::initial(&[]);
    let view = editor_view(&editor);
    assert_eq!(view.title, TemplateKind::TrackRows.label());
    assert_eq!(view.text, TemplateKind::TrackRows.default_template());
    assert!(!view.fields.is_empty());
    assert!(!view.can_reset);
}

#[test]
fn editor_view_reports_the_kept_note_only_while_the_template_does_not_apply() {
    let editor = TemplateEditorState::initial(&[]);
    let view = editor_view(&editor);
    assert_eq!(view.kept_note, "");
    assert_eq!(view.notes.len(), 1, "only the source note is a plain note");
    let broken = editor.with_edit(music_player::template_editor::TemplateEditorEdit {
        text: "$mi(nope)$".to_string(),
        selection_start: 10,
        selection_end: 10,
    });
    let broken_view = editor_view(&broken);
    assert_eq!(broken_view.kept_note, music_player::template_editor::TEMPLATE_KEPT_NOTE);
    assert_eq!(broken_view.mistakes, vec!["mi has no field called nope. Pick a name from the Fields list."]);
}

#[test]
fn each_template_row_leads_with_its_material_symbol() {
    assert_eq!(icon_name(TemplateKind::TrackRows), "queue_music");
    assert_eq!(icon_name(TemplateKind::PlayingTrack), "play_circle");
    let editor = TemplateEditorState::initial(&[]);
    let icons: Vec<&str> = settings_rows(&editor).iter().map(|row| return row.icon).collect();
    assert_eq!(icons, vec!["queue_music", "play_circle"]);
}

#[test]
fn index_of_kind_is_the_inverse_of_kind_of_index() {
    for kind in TemplateKind::all() {
        let index = index_of_kind(kind);
        assert_eq!(kind_of_index(index), Some(kind));
    }
    assert_eq!(index_of_kind(TemplateKind::TrackRows), 0);
    assert_eq!(index_of_kind(TemplateKind::PlayingTrack), 1);
}
