//! Pure template helpers for the desktop window: track fields from display paths, the playing-track subtitle,
//! and the offset conversion the template editor needs.
//!
//! What: this module turns queue names into the `TrackFields` and `PlayingTrackFields` the template engine reads,
//! and converts the editor's UTF-8 byte offsets into the UTF-16 positions its state uses.
//! Why: the engine evaluates templates over these structs only, and the editor's offsets come from Slint in bytes.

use music_player::template_model::{PlayingTrackFields, TrackFields, playing_template_fields};
use music_player::template_eval::evaluate_template;
use music_player::track_entries::track_title_for;
use music_player::track_measure::{TrackMeasure, fields_with_measure};
use std::collections::HashMap;

/// What: `track_fields` builds the template fields of one track from its display path.
/// Why: the display path is the only track identity the window has; the length and peak stay unknown until the engine reports them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const trackFields = (displayPath: string): TrackFields => ({ title, file, ext, folder, path, len: null, peak: null });
/// ```
pub fn track_fields(display_path: &str) -> TrackFields {
    let (folder, file_name) = match display_path.rsplit_once('/') {
        Some((folder, file_name)) => (folder.to_string(), file_name.to_string()),
        None => (String::new(), display_path.to_string()),
    };
    let (file, ext) = match file_name.rsplit_once('.') {
        Some((stem, extension)) if !stem.is_empty() => (stem.to_string(), extension.to_string()),
        _ => (file_name.clone(), String::new()),
    };
    return TrackFields {
        title: track_title_for(display_path),
        file,
        ext,
        folder,
        path: display_path.to_string(),
        len: None,
        peak: None,
    };
}

/// What: `playing_track` adds the place and total of one track to its fields, with its measure when known.
/// Why: the playing template reads `track` and `total`, which are the place in the list and the list size; the measure supplies `len` and `peak`.
pub fn playing_track(display_path: &str, place: usize, total: usize, measure: Option<&TrackMeasure>) -> PlayingTrackFields {
    return PlayingTrackFields {
        track: fields_with_measure(track_fields(display_path), measure),
        place_in_folder: Some(place),
        tracks_in_folder: Some(total),
    };
}

/// What: `editor_library` lists every queue name as a preview track for the template editor.
/// Why: the editor's preview reads the library in queue order, with the place and total of the whole queue, and each track's known measure.
pub fn editor_library(names: &[String], measures: &HashMap<String, TrackMeasure>) -> Vec<PlayingTrackFields> {
    let total = names.len();
    return names
        .iter()
        .enumerate()
        .map(|(position, name)| return playing_track(name, position + 1, total, measures.get(name)))
        .collect();
}

/// What: `playing_subtitle` applies the playing template to one track; None when the template does not apply.
/// Why: the deck subtitle shows the applied line, and the caller falls back to the position text when none applies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const playingSubtitle = (text: string, track: PlayingTrackFields): string | null => { ... };
/// ```
pub fn playing_subtitle(text: &str, track: &PlayingTrackFields) -> Option<String> {
    let fields = playing_template_fields();
    return match evaluate_template(text, track, &fields) {
        music_player::template_model::TemplateResult::Shown(line) => Some(line.trim().to_string()).filter(|trimmed| return !trimmed.is_empty()),
        music_player::template_model::TemplateResult::Refused(_) => None,
    };
}

/// What: `utf16_offset` converts a UTF-8 byte offset in `text` to a UTF-16 code-unit offset.
/// Why: the editor reports byte offsets, and the template state counts UTF-16 units; an offset inside a character rounds down to its start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const utf16Offset = (text: string, byteOffset: number): number => new TextEncoder().encode(text.slice(0, ...)).length;
/// ```
pub fn utf16_offset(text: &str, byte_offset: usize) -> usize {
    let mut end = byte_offset.min(text.len());
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    return text[..end].encode_utf16().count();
}

#[cfg(test)]
#[path = "ui_templates_tests.rs"]
mod tests;
