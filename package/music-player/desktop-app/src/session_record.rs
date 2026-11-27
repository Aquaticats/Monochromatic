//! The record a session file is read and written as.
//!
//! What:
//!  The shapes a session file is read and written as:
//!  the current `Session` record,
//!  the permissive `StoredSession` read shape that also recognizes the former
//!  `shuffle` and `repeat_track` fields,
//!  and the first-run values each starts from.
//! Why:
//!  The reading, migrating and saving code lives in the sibling file `session.rs`,
//!  so the record shapes and their defaults get a file of their own.
//!  `Session` is re-exported as `session::Session`,
//!  so every caller keeps the path it already names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Session = {
//!   sourceRoot: string | null;
//!   selected: string | null;
//!   positionSecs: number;
//!   volume: number;
//!   playbackMode: PlaybackMode;
//!   trackRowsTemplate: string | null;
//!   playingTemplate: string | null;
//!   autoPlayOnSelect: boolean;
//!   textSizes: TextSizes;
//! };
//! // The permissive read shape holds every field of Session as optional, plus the former
//! // `shuffle` and `repeat_track` keys the migration still reads.
//! type StoredSession = { playbackMode?: string; shuffle?: string; repeatTrack?: boolean };
//! const firstRunSession = (): Session => ({ positionSecs: 0, volume: 1, /* ... */ });
//! ```

/// Owned filesystem path used for the source root and selected track.
use std::path::PathBuf;

/// Serde derives used by current output and the private migration input.
use serde::{
    Deserialize,
    Serialize,
};

/// Current four-state playback model.
use crate::command::PlaybackMode;

/// The four text sizes the Settings page changes.
use crate::text_size::TextSizes;

/// Current session shape written to disk.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Session {
    /// Opened directory whose scan produces the queue.
    pub source_root: Option<PathBuf>,
    /// Selected track path,
    ///  or no selection.
    pub selected: Option<PathBuf>,
    /// Resume position in seconds.
    pub position_secs: f64,
    /// Output gain from zero to one.
    pub volume: f32,
    /// Single selected completion and transport behavior.
    pub playback_mode: PlaybackMode,
    /// Track rows template text the user edited,
    /// or None to use the default.
    pub track_rows_template: Option<String>,
    /// Playing track template text the user edited,
    /// or None to use the default.
    pub playing_template: Option<String>,
    /// Auto-play on select (D129):
    /// whether selecting a track also starts it.
    /// On when the file has no such field.
    pub auto_play_on_select: bool,
    /// Text size setting:
    /// one pixel size for each of four texts,
    /// written as four flat keys beginning `text_size_`.
    /// The defaults are the sizes the app drew before the setting existed.
    #[serde(flatten)]
    pub text_sizes: TextSizes,
}

/// Private permissive input shape that recognizes current and former fields.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub(super) struct StoredSession {
    /// Current source root.
    pub(super) source_root: Option<PathBuf>,
    /// Current selected track.
    pub(super) selected: Option<PathBuf>,
    /// Current resume position.
    pub(super) position_secs: f64,
    /// Current output gain.
    pub(super) volume: f32,
    /// Current wire text when it is a string.
    pub(super) playback_mode: Option<String>,
    /// Former shuffle enum text,
    ///  used only when `playback_mode` is absent.
    pub(super) shuffle: Option<String>,
    /// Former independent repeat flag,
    ///  including whether the key was present.
    pub(super) repeat_track: Option<bool>,
    /// Track rows template text,
    /// when the user edited it.
    pub(super) track_rows_template: Option<String>,
    /// Playing track template text,
    /// when the user edited it.
    pub(super) playing_template: Option<String>,
    /// Auto-play on select,
    /// when a newer build saved it.
    pub(super) auto_play_on_select: bool,
    /// Stored text size of each element in the order of `TextElement::ALL`,
    /// `None` where the key is absent.
    /// Kept as raw
    /// JSON and filled by `parse_stored`,
    /// not by serde,
    /// so one corrupt value falls back to its own default and never
    /// fails the whole file,
    /// and an explicit null is told apart from a missing key.
    #[serde(skip)]
    pub(super) text_sizes: [Option<serde_json::Value>; 4],
}

/// Supplies defaults for missing fields in the permissive input shape.
impl Default for StoredSession {
    /// Returns missing-field defaults for current and former JSON shapes.
    fn default() -> StoredSession {
        return StoredSession {
            source_root: None,
            selected: None,
            position_secs: 0.0,
            volume: 1.0,
            playback_mode: None,
            shuffle: None,
            repeat_track: None,
            track_rows_template: None,
            playing_template: None,
            auto_play_on_select: true,
            text_sizes: [None, None, None, None],
        };
    }
}

/// Supplies first-run values for the current session shape.
impl Default for Session {
    /// Returns first-run desktop session values.
    fn default() -> Session {
        return Session {
            source_root: None,
            selected: None,
            position_secs: 0.0,
            volume: 1.0,
            playback_mode: PlaybackMode::InOrder,
            track_rows_template: None,
            playing_template: None,
            auto_play_on_select: true,
            text_sizes: TextSizes::default(),
        };
    }
}
