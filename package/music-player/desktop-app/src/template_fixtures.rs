//! Shared track fixtures for the template tests,
//!  copied from the reference's
//! `analysed`,
//!  `waiting`,
//!  `bare`,
//!  and `playing` tracks and from the editor
//! test's two library tracks.
//!
//! Compiled only under `cargo test`;
//!  the tests call these builders so each
//! expected line has one source of track values.

/// What:
///  imports the edit record the typing helper builds.
/// Why:
///  The fixtures type text through the real state API.
use crate::template_editor::TemplateEditorEdit;
/// What:
///  imports the editor state the fixtures build.
/// Why:
///  The editor tests start from it.
use crate::template_editor_state::TemplateEditorState;
/// What:
///  imports the UTF-16 length helper.
/// Why:
///  `typed` places the caret at the end of the text.
use crate::template_format::utf16_length;
/// What:
///  imports the track record types.
/// Why:
///  Every fixture builds one of them.
use crate::template_model::{PlayingTrackFields, TrackFields};

/// What:
///  `pub fn typed(state: &TemplateEditorState, text: &str) -> TemplateEditorState` types a whole text with the caret at its end.
/// Why:
///  Most editor cases only need the text replaced,
///  so one call keeps them short.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const typed = (state, text) => state.withEdit({ text, selectionStart: text.length, selectionEnd: text.length });
/// ```
pub fn typed(state: &TemplateEditorState, text: &str) -> TemplateEditorState {
    let length = utf16_length(text);
    return edit_at(state, text, length, length);
}

/// What:
///  `pub fn edit_at(state: &TemplateEditorState, text: &str, start: usize, end: usize) -> TemplateEditorState` sets a text and a selection.
/// Why:
///  Insertion and help cases start from a specific caret or selection,
///  and this builds that start point in one call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// state.withEdit({ text, selectionStart: start, selectionEnd: end });
/// ```
pub fn edit_at(state: &TemplateEditorState, text: &str, start: usize, end: usize) -> TemplateEditorState {
    return state.with_edit(TemplateEditorEdit {
        text: text.to_string(),
        selection_start: start,
        selection_end: end,
    });
}

/// What:
///  `pub fn analysed() -> TrackFields` is the track whose true peak is already analysed.
/// Why:
///  Most reference cases read a complete track,
///  so one fixture keeps them comparable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const analysed = { title: "Another Xronixle", ..., len: 275, peak: "−0.3 dBTP" };
/// ```
pub fn analysed() -> TrackFields {
    return TrackFields {
        title: "Another Xronixle".to_string(),
        file: "かめりあ(Camellia) - Another Xronixle".to_string(),
        ext: "flac".to_string(),
        folder: "Camellia".to_string(),
        path: "Camellia/かめりあ(Camellia) - Another Xronixle.flac".to_string(),
        len: Some(275.0),
        peak: Some("\u{2212}0.3 dBTP".to_string()),
    };
}

/// What:
///  `pub fn waiting() -> TrackFields` is the track with a title of its own and no peak yet.
/// Why:
///  An empty peak leaves the surrounding literal text in place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const waiting = { ...analysed, title: "Exit This Earth's Atomosphere", peak: undefined };
/// ```
pub fn waiting() -> TrackFields {
    return TrackFields {
        title: "Exit This Earth's Atomosphere".to_string(),
        peak: None,
        ..analysed()
    };
}

/// What:
///  `pub fn bare() -> TrackFields` is the track with nothing but a path.
/// Why:
///  Missing values must show as empty text rather than as placeholders.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bare = { title: "x", file: "x", ext: "mp3", folder: "", path: "x.mp3", len: undefined, peak: undefined };
/// ```
pub fn bare() -> TrackFields {
    return TrackFields {
        title: "x".to_string(),
        file: "x".to_string(),
        ext: "mp3".to_string(),
        folder: String::new(),
        path: "x.mp3".to_string(),
        len: None,
        peak: None,
    };
}

/// What:
///  `pub fn bare_playing() -> PlayingTrackFields` is a track with no duration,
///  peak,
///  place,
///  or folder total.
/// Why:
///  Production tracks often carry no metadata,
///  and the defaults must still apply to them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bare = { title: "Bare", file: "Bare", ext: "mp3", folder: "", path: "Bare.mp3", len: undefined, place: undefined, total: undefined };
/// ```
pub fn bare_playing() -> PlayingTrackFields {
    return PlayingTrackFields {
        track: TrackFields {
            title: "Bare".to_string(),
            file: "Bare".to_string(),
            ext: "mp3".to_string(),
            folder: String::new(),
            path: "Bare.mp3".to_string(),
            len: None,
            peak: None,
        },
        place_in_folder: None,
        tracks_in_folder: None,
    };
}

/// What:
///  `pub fn playing() -> PlayingTrackFields` is the analysed track at place 1 of 16.
/// Why:
///  The playing template reads the place and total as well as the track's own fields.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const playing = { ...analysed, track: 1, total: 16 };
/// ```
pub fn playing() -> PlayingTrackFields {
    return PlayingTrackFields {
        track: analysed(),
        place_in_folder: Some(1),
        tracks_in_folder: Some(16),
    };
}

/// What:
///  `pub fn another_xronixle() -> PlayingTrackFields` is the first authored library track of the editor study.
/// Why:
///  The expected preview lines in the editor tests come from this track.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const anotherXronixle = { title: "Another Xronixle", len: 275, peak: "−1.2 dBTP", place: 1, total: 16 };
/// ```
pub fn another_xronixle() -> PlayingTrackFields {
    return PlayingTrackFields {
        track: TrackFields {
            title: "Another Xronixle".to_string(),
            file: "かめりあ(Camellia) - Another Xronixle".to_string(),
            ext: "flac".to_string(),
            folder: "Camellia".to_string(),
            path: "Camellia/かめりあ(Camellia) - Another Xronixle.flac".to_string(),
            len: Some(275.0),
            peak: Some("\u{2212}1.2 dBTP".to_string()),
        },
        place_in_folder: Some(1),
        tracks_in_folder: Some(16),
    };
}

/// What:
///  `pub fn burning_aquamarine() -> PlayingTrackFields` is the second authored library track,
///  not yet analysed.
/// Why:
///  Its missing peak shows that an empty field leaves the literal text around it in place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const burningAquamarine = { title: "Burning Aquamarine", len: 312, peak: undefined, place: 2, total: 16 };
/// ```
pub fn burning_aquamarine() -> PlayingTrackFields {
    return PlayingTrackFields {
        track: TrackFields {
            title: "Burning Aquamarine".to_string(),
            file: "Burning Aquamarine".to_string(),
            ext: "flac".to_string(),
            folder: "Camellia".to_string(),
            path: "Camellia/Burning Aquamarine.flac".to_string(),
            len: Some(312.0),
            peak: None,
        },
        place_in_folder: Some(2),
        tracks_in_folder: Some(16),
    };
}

/// What:
///  `pub fn library() -> Vec<PlayingTrackFields>` is the two authored library tracks in folder order.
/// Why:
///  Most editor tests start from the same two tracks the study previews.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const library = [anotherXronixle, burningAquamarine];
/// ```
pub fn library() -> Vec<PlayingTrackFields> {
    return vec![another_xronixle(), burning_aquamarine()];
}
